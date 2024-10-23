extern crate self as nakssg;
pub use nakssg_macro::nakssg_html;
pub mod html;
pub mod util;
pub use html::{HtmlWriter, ToHtml};
