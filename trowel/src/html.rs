use std::fmt::Write;

pub type Attribute = (String, Option<String>);

pub trait HtmlWriter {
    fn write_tag(&mut self, tag: &str, single: bool, attributes: Vec<Attribute>);
    fn write_end_tag(&mut self, tag: &str);
    fn write_string_lit(&mut self, lit: &str);
}

pub trait ToHtml {
    fn to_html(self, writer: &mut dyn HtmlWriter);
}

impl<F: FnOnce(&mut dyn crate::HtmlWriter)> ToHtml for F {
    fn to_html(self, writer: &mut dyn HtmlWriter) {
        self(writer)
    }
}

impl ToHtml for &str {
    fn to_html(self, writer: &mut dyn HtmlWriter) {
        writer.write_string_lit(self);
    }
}

impl ToHtml for String {
    fn to_html(self, writer: &mut dyn HtmlWriter) {
        writer.write_string_lit(self.as_str());
    }
}

impl<T> ToHtml for Vec<T>
where
    T: Fn(&mut dyn HtmlWriter),
{
    fn to_html(self, writer: &mut dyn HtmlWriter) {
        for el in self {
            el(writer)
        }
    }
}

impl<T: ToHtml> ToHtml for Option<T> {
    fn to_html(self, writer: &mut dyn HtmlWriter) {
        if let Some(x) = self {
            x.to_html(writer)
        }
    }
}

pub struct WriteHtml<T: Write> {
    writer: T,
}

impl<T: Write> WriteHtml<T> {
    pub fn new(writer: T) -> Self {
        WriteHtml { writer }
    }
}

impl<T: Write> HtmlWriter for WriteHtml<T> {
    fn write_tag(&mut self, tag: &str, single: bool, attr: Vec<Attribute>) {
        let end = if single { " /" } else { "" };
        let mut attr_str = String::new();
        for (key, value) in attr {
            attr_str.push(' ');
            attr_str.push_str(&key);
            if let Some(value) = value {
                attr_str.push('=');
                attr_str.push('"');
                attr_str.push_str(&value);
                attr_str.push('"');
            }
        }
        write!(self.writer, "<{tag}{attr_str}{end}>").unwrap();
    }

    fn write_end_tag(&mut self, tag: &str) {
        write!(self.writer, "</{tag}>").unwrap();
    }

    fn write_string_lit(&mut self, lit: &str) {
        writeln!(self.writer, "{lit}").unwrap()
    }
}
