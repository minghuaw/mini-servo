use std::io::Cursor;

use blitz_dom::{BaseDocument, DocumentConfig};
use blitz_html::DocumentHtmlParser;
use html5ever::{interface::TreeSink, tendril::TendrilSink, ParseOpts};
use rcdom::RcDom;

use crate::{core::Dom};

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

impl ParseHtml for BaseDocument {
    type Error = std::io::Error;

    fn parse_html(html: &str, opts: ParseOpts) -> Result<Self, Self::Error> {
        let mut reader = Cursor::new(html);

        let config = DocumentConfig::default();
        let mut doc = BaseDocument::new(config);
        let sink = DocumentHtmlParser::new(&mut doc);

        html5ever::parse_document(sink, opts)
            .from_utf8()
            .read_from(&mut reader)?;

        Ok(doc)
    }
}

#[cfg(test)]
mod tests {
    use blitz_dom::BaseDocument;
    use rcdom::RcDom;

    use crate::parse::ParseHtml;

    // const SIMPLE_TEST_HTML: &str = "<p>Hello, world!</p>";
    // const SIMPLE_TEST_HTML: &str = "<html></html>";
    const SIMPLE_TEST_HTML: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>This is title</title>
</head>
<body>
    <h1>header 1</h1>
    <p>This is a simple paragraph in my HTML document</p>
    Here's some additional text outside of the paragraph tags
    <p>This is a para</p>
</body>
</html>
"#;

    #[test]
    fn test_rcdom_parse_html() {
        let _parsed = RcDom::parse_html(SIMPLE_TEST_HTML, Default::default()).unwrap();
    }

    #[test]
    fn test_blitz_dom_parse_html() {
        let _parsed = BaseDocument::parse_html(SIMPLE_TEST_HTML, Default::default()).unwrap();
    }
}