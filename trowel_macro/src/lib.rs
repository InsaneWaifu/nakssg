use quote::{quote, quote_spanned};
use syn::{
    braced, bracketed, parenthesized,
    parse::{self, Parse},
    punctuated::Punctuated,
    spanned::Spanned,
    token::Brace,
    Block, Expr, Ident, LitStr, Path, Stmt, Token,
};

enum HtmlAttribute {
    JustIdent(Ident),
    IdentIf(Ident, Expr),
    Expression(Ident, Expr),
}

impl Parse for HtmlAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;
        if input.peek(syn::token::Bracket) {
            let if_expr;
            bracketed!(if_expr in input);
            let if_expr: Expr = if_expr.parse()?;
            Ok(HtmlAttribute::IdentIf(ident, if_expr))
        } else if input.peek(Token![:]) {
            input.parse::<Token![:]>()?;
            if input.peek(LitStr) {
                let value = input.parse::<LitStr>()?;
                Ok(HtmlAttribute::Expression(
                    ident,
                    syn::parse2(quote_spanned! {value.span() => Some(#value)})?,
                ))
            } else {
                let expr;
                braced!(expr in input);
                let expr: Expr = expr.parse()?;
                Ok(HtmlAttribute::Expression(ident, expr))
            }
        } else {
            Ok(HtmlAttribute::JustIdent(ident))
        }
    }
}

enum HtmlAstElem {
    Plain {
        name: Ident,
        children: Punctuated<HtmlAstElem, Token![,]>,
        attributes: Punctuated<HtmlAttribute, Token![,]>,
        single_tag: bool,
    },
    Fragment {
        name: Path,
        children: Punctuated<HtmlAstElem, Token![,]>,
        attributes: Punctuated<HtmlAttribute, Token![,]>,
    },
    Text(LitStr),
    Expression(Expr),
    Code(Vec<Stmt>),
}

impl Parse for HtmlAstElem {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Brace) {
            let expr;
            braced!(expr in input);
            let expr: Expr = expr.parse()?;
            return Ok(HtmlAstElem::Expression(expr));
        }
        if input.peek(LitStr) {
            return Ok(HtmlAstElem::Text(input.parse()?));
        }
        let is_single = input.peek(Token![<]);
        if is_single {
            input.parse::<Token![<]>()?;
        }
        let is_fragment = input.peek(Token![!]);
        if is_fragment {
            input.parse::<Token![!]>()?;
            if input.peek(Brace) {
                let expr;
                braced!(expr in input);
                let expr = Block::parse_within(&expr)?;
                return Ok(HtmlAstElem::Code(expr));
            }
        }
        let name = input.parse::<Path>()?;
        let attributes;
        let children;
        if !is_single {
            attributes = if input.peek(syn::token::Paren) {
                let attributes;
                parenthesized!(attributes in input);
                Punctuated::<HtmlAttribute, Token![,]>::parse_terminated(&attributes)?
            } else {
                Punctuated::new()
            };

            let p_children;
            braced!(p_children in input);
            children = Punctuated::<HtmlAstElem, Token![,]>::parse_terminated(&p_children)?;
        } else {
            attributes = Punctuated::new();
            children = Punctuated::new();
        }

        if is_fragment {
            Ok(HtmlAstElem::Fragment {
                name,
                children,
                attributes,
            })
        } else {
            let name = name.get_ident().cloned().unwrap();

            if !is_single {
                Ok(HtmlAstElem::Plain {
                    name,
                    children,
                    attributes,
                    single_tag: false,
                })
            } else {
                Ok(HtmlAstElem::Plain {
                    name,
                    children: Punctuated::new(),
                    attributes,
                    single_tag: true,
                })
            }
        }
    }
}

#[proc_macro]
pub fn trowel_html(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    struct HtmlMacroInput {
        move_token: Option<Token![move]>,
        tree: Vec<proc_macro2::TokenStream>,
    }
    impl Parse for HtmlMacroInput {
        fn parse(input: parse::ParseStream) -> parse::Result<Self> {
            let move_token = input.parse::<Option<Token![move]>>()?;
            let tree = Punctuated::<HtmlAstElem, Token![,]>::parse_terminated(input)
                .map(|x| x.into_iter().map(generate_html))?.collect();
            Ok(HtmlMacroInput { move_token, tree })
        }
    }
    let HtmlMacroInput { move_token, tree } = syn::parse::<HtmlMacroInput>(input).unwrap();
    quote! {
        #move_token |writer: &mut dyn (::trowel::HtmlWriter)| {
            #(#tree);*
        }
    }
    .into()
}

fn html_expr(expr: Expr) -> proc_macro2::TokenStream {
    quote_spanned! {
        expr.span() =>
        {
            (::trowel::ToHtml::to_html(#expr, writer));
        }
    }
}

fn generate_attributes_list(
    attributes: Punctuated<HtmlAttribute, syn::Token![,]>,
) -> proc_macro2::TokenStream {
    let mut code = vec![];
    for attribute in attributes {
        code.push(match attribute {
            HtmlAttribute::IdentIf(ident, cond) => {
                quote_spanned! {
                    cond.span() => {
                        if #cond {
                            attr.push((stringify!(#ident).to_string(), None))
                        }
                    }
                }
            }
            HtmlAttribute::Expression(ident, expr) => {
                quote_spanned! {
                    expr.span() => {
                        let expr: Option<_> = #expr;
                        attr.push((stringify!(#ident).to_string(), expr.map(|x|x.to_string())));
                    }
                }
            }
            HtmlAttribute::JustIdent(ident) => {
                quote_spanned! {
                    ident.span() => {
                        attr.push((stringify!(#ident).to_string(), None));
                    }
                }
            }
        });
    }

    quote! {
        let mut attr: Vec<(String, Option<String>)> = Vec::new();
        #(
            #code
        )*
        attr
    }
}

fn generate_html(elem: HtmlAstElem) -> proc_macro2::TokenStream {
    let inner = match elem {
        HtmlAstElem::Text(str) => {
            quote! {writer.write_string_lit(#str);}
        }
        HtmlAstElem::Fragment {
            name,
            children,
            attributes,
        } => {
            let attributes_list = generate_attributes_list(attributes);
            let children = children.into_iter().map(generate_html).collect::<Vec<_>>();
            quote! {
                {
                    (::trowel::ToHtml::to_html(#name (
                        {#attributes_list}, |writer: &mut dyn (::trowel::HtmlWriter)| {
                            #(#children);*
                        }
                    ), writer));
                }
            }
        }
        HtmlAstElem::Plain {
            name,
            children,
            attributes,
            single_tag,
        } => {
            let attributes_list = generate_attributes_list(attributes);
            let children = children.into_iter().map(generate_html).collect::<Vec<_>>();
            quote! {

                    writer.write_tag(stringify!(#name), #single_tag, {
                        #attributes_list
                    });
                    #(
                        #children
                    );*
                    writer.write_end_tag(stringify!(#name));
            }
        }
        HtmlAstElem::Expression(expr) => html_expr(expr),
        HtmlAstElem::Code(expr) => {
            let expr = expr.into_iter().collect::<Vec<_>>();
            quote! {
                #(
                    #expr ;
                )*
            }
        }
    };
    quote! {
        #inner
    }
}
