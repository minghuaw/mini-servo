use style::{
    context::{SharedStyleContext, StyleContext},
    dom::{NodeInfo, TElement, TNode},
    driver::traverse_dom,
    traversal::{DomTraversal, PerLevelTraversalData, recalc_style_at},
};

// mod rcdom;

pub fn resolve_style<E, D>(
    root: E,
    traversal: D,
    shared_context: &SharedStyleContext<'_>,
    pool: Option<&rayon::ThreadPool>,
) -> Option<E>
where
    E: TElement,
    D: DomTraversal<E>,
{
    let token = D::pre_traverse(root, shared_context);
    if token.should_traverse() {
        return Some(traverse_dom(&traversal, token, pool));
    }

    None
}

pub struct RecalcStyle<'a> {
    context: &'a SharedStyleContext<'a>,
}

impl<'a> RecalcStyle<'a> {
    pub fn new(context: &'a SharedStyleContext<'a>) -> Self {
        RecalcStyle { context }
    }
}

#[allow(unsafe_code)]
impl<E> DomTraversal<E> for RecalcStyle<'_>
where
    E: TElement,
{
    fn process_preorder<F: FnMut(E::ConcreteNode)>(
        &self,
        traversal_data: &PerLevelTraversalData,
        context: &mut StyleContext<E>,
        node: E::ConcreteNode,
        note_child: F,
    ) {
        // Don't process textnodees in this traversal
        if node.is_text_node() {
            return;
        }

        let el = node.as_element().unwrap();
        // let mut data = el.mutate_data().unwrap();
        let mut data = unsafe { el.ensure_data() };
        recalc_style_at(self, traversal_data, context, el, &mut data, note_child);

        // Gets set later on
        unsafe { el.unset_dirty_descendants() }
    }

    #[inline]
    fn needs_postorder_traversal() -> bool {
        false
    }

    fn process_postorder(&self, _style_context: &mut StyleContext<E>, _node: E::ConcreteNode) {
        panic!("this should never be called")
    }

    #[inline]
    fn shared_context(&self) -> &SharedStyleContext {
        &self.context
    }
}

#[cfg(test)]
mod tests {
    use ::rcdom::RcDom;
    use blitz_dom::BaseDocument;
    use euclid::Size2D;
    use selectors::Element;
    use style::{
        dom::TDocument,
        selector_parser::SnapshotMap,
        shared_lock::{SharedRwLock, StylesheetGuards},
        thread_state::{self, ThreadState},
    };

    use crate::{
        dummy::DummyRegisteredSpeculativePainters,
        parse::ParseHtml,
        util::{make_device, make_shared_style_context, make_stylist},
    };

    use super::*;

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
    fn test_blitz_dom_style_traversal() {
        thread_state::enter(ThreadState::LAYOUT);
        let doc = BaseDocument::parse_html(SIMPLE_TEST_HTML, Default::default()).unwrap();

        // Create a dummy shared style context
        let device = make_device(Size2D::new(800.0, 600.0));
        let stylist = make_stylist(device);
        let guard = SharedRwLock::new();
        let guards = StylesheetGuards {
            author: &guard.read(),
            ua_or_user: &guard.read(),
        };
        let snapshot_map = SnapshotMap::new();
        let registered_speculative_painters = DummyRegisteredSpeculativePainters;

        let shared_context = make_shared_style_context(
            &stylist,
            guards,
            &snapshot_map,
            &registered_speculative_painters,
        );
        let traversal = RecalcStyle::new(&shared_context);

        let root = TDocument::as_node(&doc.get_node(0).unwrap())
            .first_element_child()
            .unwrap()
            .as_element()
            .unwrap();
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(6) // STYLO_MAX_THREADS is not public
            .build()
            .unwrap();
        let dirty_root = resolve_style(root, traversal, &shared_context, None).unwrap();
        assert!(dirty_root.is_html_document());

        thread_state::exit(ThreadState::LAYOUT);
    }

    #[test]
    fn test_rcdom_style_traversal() {
        let doc = RcDom::parse_html(SIMPLE_TEST_HTML, Default::default()).unwrap();

        // Create a dummy shared style context
        let device = make_device(Size2D::new(800.0, 600.0));
        let stylist = make_stylist(device);
        let guard = SharedRwLock::new();
        let guards = StylesheetGuards {
            author: &guard.read(),
            ua_or_user: &guard.read(),
        };
        let snapshot_map = SnapshotMap::new();
        let registered_speculative_painters = DummyRegisteredSpeculativePainters;

        let shared_context = make_shared_style_context(
            &stylist,
            guards,
            &snapshot_map,
            &registered_speculative_painters,
        );
        let traversal = RecalcStyle::new(&shared_context);
        let root = doc.document.clone();

        // resolve_style(root, traversal, &shared_context, None);
    }
}
