use std::collections::HashMap;

use trowel::util::{Doctype, HeadDefault};
use trowel::{html::Attribute, ToHtml};
use trowel::{trowel_html, HtmlWriter};

#[allow(non_snake_case)]
pub fn BlogPageBase(attrs: Vec<Attribute>, children: impl Fn(&mut dyn HtmlWriter)) -> impl ToHtml {
    let mut attrs: HashMap<_, _> = attrs.into_iter().collect();
    let title = attrs.remove("title").expect("No title").unwrap();
    let timestamp = attrs.remove("timestamp").flatten();

    trowel_html! { move
        <!Doctype,
        html {
            head {
                <!HeadDefault,
                title {
                    {title.as_str()}
                },
                style {
                    r#"
nav {
  display: flex;
  justify-content: space-between;
}

aside {
  width: 40%;
  padding-left: 0.5rem;
  margin-left: 0.5rem;
  float: right;
  box-shadow: inset 5px 0 5px -5px #49002d;
  font-style: italic;
  color: #49002d;
}

@media (min-width: calc(38em * 2.2)) {
  aside {
    float: none; /* Disable floating */
    position: absolute; /* Take it out of the document flow */
    right: -40%;
    width: calc(38em * 0.4);
  }
  body {
    position: relative;
  }
}
                    "#
                },
            },
            body {
                header {
                    nav {
                        div {
                            "TODO"
                        },
                        div {
                            {
                                vec![("/", "Home"), ("/about", "Bbout")].into_iter().map(|x|
                                    trowel_html!{ move
                                        a(href: {Some(x.0.to_string())}) {
                                            {x.1}
                                        }
                                    }
                                ).collect::<Vec<_>>()
                            }
                        }
                    }
                },
                article {
                    h1 {
                        {title.as_str()}
                    },
                    {timestamp.as_ref().map(|timestamp| trowel_html! {
                            time(datetime: {Some(timestamp.as_str())}) {
                                {timestamp.as_str()}
                            },
                        })},

                    {children}
                },
                script {
                    r#"
document.querySelectorAll('time').forEach($e => {
  const date = new Date($e.dateTime);
  // output the localized date and time
  $e.innerHTML = date.toLocaleString();
});
                    "#
                }
            },
        }
    }
}
