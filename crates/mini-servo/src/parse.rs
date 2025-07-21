use std::io::Cursor;

use html5ever::{interface::TreeSink, tendril::TendrilSink, ParseOpts};
use rcdom::RcDom;

use crate::core::Dom;

// pub fn parse_html<D, S>(html: &str, sink: S, opts: ParseOpts) -> std::io::Result<D> 
// where 
//     D: Dom,
//     S: TreeSink,
// {
//     html5ever::parse_document(sink, opts)
//         .from_utf8()
//         .read_from(&mut html.as_bytes())?;

//     todo!()
// }

pub trait ParseHtml: Sized {
    type Error;

    fn parse_html(html: &str, opts: ParseOpts) -> Result<Self, Self::Error>;
}

impl ParseHtml for RcDom {
    type Error = std::io::Error;

    fn parse_html(html: &str, opts: ParseOpts) -> Result<Self, Self::Error> {
        let mut reader = Cursor::new(html);
        html5ever::parse_document(RcDom::default(), opts)
            .from_utf8()
            .read_from(&mut reader)
    }
}

#[cfg(test)]
mod tests {
    use rcdom::RcDom;

    use crate::parse::ParseHtml;

    #[test]
    fn test_rcdom_parse_html() {
        let test_html = "<p>Hello, world!</p>";
        let _parsed = RcDom::parse_html(test_html, Default::default()).unwrap();
    }
}