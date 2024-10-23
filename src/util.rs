use nakssg::nakssg_html;

use crate::{html::Attribute, HtmlWriter, ToHtml};

#[allow(non_snake_case)]
pub fn Doctype(attrs: Vec<Attribute>, _children: impl Fn(&mut dyn HtmlWriter)) -> impl ToHtml {
    assert!(attrs.is_empty());
    nakssg_html!(
        r#"<!DOCTYPE html>"#
    )
}

#[allow(non_snake_case)]
pub fn HeadDefault(attrs: Vec<Attribute>, _children: impl Fn(&mut dyn HtmlWriter)) -> impl ToHtml {
    assert!(attrs.is_empty());
    nakssg_html!{
        r#"<meta charset="utf-8" />"#,
        r#"<link rel="stylesheet" href="https://unpkg.com/normalize.css@7.0.0/normalize.css" type="text/css" />"#,
        r#"<link rel="stylesheet" href="https://unpkg.com/sakura.css@1.5.0/css/sakura-pink.css" type="text/css" />"#,
        r#"<meta name="viewport" content="width=device-width, initial-scale=1" />"#
    }
}

pub fn html_to_string(x: impl FnOnce(&mut dyn HtmlWriter)) -> String {
    let mut buf = String::new();
    let mut writer = crate::html::WriteHtml::new(&mut buf);
    x(&mut writer);
    buf
}