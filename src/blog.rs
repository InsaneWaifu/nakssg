use std::collections::HashMap;

use chrono::DateTime;
use nakssg::util::{Doctype, HeadDefault};
use nakssg::{html::Attribute, ToHtml};
use nakssg::{nakssg_html, HtmlWriter};

#[allow(non_snake_case)]
pub fn BlogPageBase(attrs: Vec<Attribute>, children: impl Fn(&mut dyn HtmlWriter)) -> impl ToHtml {
    let mut attrs: HashMap<_, _> = attrs.into_iter().collect();
    let title = attrs.remove("title").expect("No title").unwrap();
    let timestamp = attrs.remove("timestamp").flatten();

    nakssg_html! {
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
                                vec![("/", "Home"), ("/about", "About")].into_iter().map(|x|
                                    nakssg_html!{
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
                    {if let Some(timestamp) = timestamp.as_ref() {
                        Some(nakssg_html! {
                            time(datetime: {Some(timestamp.as_str())}) {
                                {timestamp.as_str()}
                            },
                        })
                    } else {
                        None
                    }},

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
