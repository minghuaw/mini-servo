use style::{context::SharedStyleContext, dom::TElement, driver::traverse_dom, traversal::DomTraversal};

pub(crate) fn resolve_style<E, D>(root: E, traversal: D, shared_context: &SharedStyleContext<'_>) -> Option<E>
where
    E: TElement,
    D: DomTraversal<E>,
{
    let token = D::pre_traverse(root, shared_context);
    if token.should_traverse() {
        return Some(traverse_dom(&traversal, token, None))
    }

    None
}

#[cfg(test)]
mod tests {
    use blitz_dom::BaseDocument;

    use crate::parse::ParseHtml;

    const SIMPLE_TEST_HTML: &str = "<p>Hello, world!</p>";

    #[test]
    fn test_blitz_dom_style_traversal() {
        let doc = BaseDocument::parse_html(SIMPLE_TEST_HTML, Default::default()).unwrap();
    }
}