use std::borrow::Cow;

use base::id::{BrowsingContextId, PipelineId};
use fonts_traits::ByteIndex;
use html5ever::local_name;
use layout_api::wrapper_traits::{LayoutNode, ThreadSafeLayoutNode};
use net_traits::image_cache::Image;
use pixels::ImageMetadata;
use range::Range;
use servo_url::ServoUrl;
use style::{attr::AttrValue, dom::{LayoutIterator, NodeInfo, TElement, TNode}, selector_parser::PseudoElement};

use crate::layout::{BlitzLayoutElement, SafeBlitzChildrenIterator, SafeBlitzLayoutElement};

use super::{BlitzNode, BlitzLayoutNode};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SafeBlitzLayoutNode<'dom> {
    pub node: BlitzLayoutNode<'dom>,
    pub pseudo: Option<PseudoElement>,
}

impl<'dom> NodeInfo for SafeBlitzLayoutNode<'dom> {
    fn is_element(&self) -> bool {
        self.node.is_element()
    }

    fn is_text_node(&self) -> bool {
        self.node.is_text_node()
    }
}


impl<'dom> ThreadSafeLayoutNode<'dom> for SafeBlitzLayoutNode<'dom> {
    type ConcreteNode = BlitzLayoutNode<'dom>;

    type ConcreteElement = BlitzLayoutElement<'dom>;

    type ConcreteThreadSafeLayoutElement = SafeBlitzLayoutElement<'dom>;

    type ChildrenIterator = SafeBlitzChildrenIterator<'dom>;

    fn opaque(&self) -> style::dom::OpaqueNode {
        self.node.opaque()
    }

    fn type_id(&self) -> Option<layout_api::LayoutNodeType> {
        if self.pseudo.is_none() {
            Some(LayoutNode::type_id(&self.node))
        } else {
            None
        }
    }

    fn parent_style(&self) -> style::servo_arc::Arc<style::properties::ComputedValues> {
        let parent_element = self.node.traversal_parent().unwrap();
        let parent_data = parent_element.borrow_data().unwrap();
        parent_data.styles.primary().clone()
    }

    fn debug_id(self) -> usize {
        self.node.debug_id()
    }

    fn children(&self) -> style::dom::LayoutIterator<Self::ChildrenIterator> {
        let traverser = self.node.value.traversal_children();
        LayoutIterator(SafeBlitzChildrenIterator { traverser: traverser.0 })
    }

    fn as_element(&self) -> Option<Self::ConcreteThreadSafeLayoutElement> {
        self.node.as_element().map(|el| {
            SafeBlitzLayoutElement { element: el, pseudo: None }
        })
    }

    fn as_html_element(&self) -> Option<Self::ConcreteThreadSafeLayoutElement> {
        self.as_element().filter(|el| el.element.is_html_element())
    }

    fn style_data(&self) -> Option<&'dom layout_api::StyleData> {
        self.node.style_data()
    }

    fn layout_data(&self) -> Option<&'dom layout_api::GenericLayoutData> {
        self.node.layout_data()
    }

    fn unsafe_get(self) -> Self::ConcreteNode {
        self.node
    }

    fn node_text_content(self) -> std::borrow::Cow<'dom, str> {
        Cow::Owned(self.node.value.text_content())
    }

    fn selection(&self) -> Option<Range<ByteIndex>> {
        unreachable!("selection doesn't seem to get used at all")
    }

    fn image_url(&self) -> Option<ServoUrl> {
        unimplemented!()
    }

    fn image_density(&self) -> Option<f64> {
        unimplemented!()
    }

    fn image_data(&self) -> Option<(Option<Image>, Option<ImageMetadata>)> {
        unimplemented!()
    }

    fn canvas_data(&self) -> Option<layout_api::HTMLCanvasData> {
        unimplemented!()
    }

    fn svg_data(&self) -> Option<layout_api::SVGSVGData> {
        unimplemented!()
    }

    fn media_data(&self) -> Option<layout_api::HTMLMediaData> {
        unimplemented!()
    }

    fn iframe_browsing_context_id(&self) -> Option<BrowsingContextId> {
        unimplemented!()
    }

    fn iframe_pipeline_id(&self) -> Option<PipelineId> {
        unimplemented!()
    }

    fn get_span(&self) -> Option<u32> {
        self.node.value.attr(local_name!("span")).map(|value| AttrValue::String(value.to_string()).as_uint())
    }

    fn get_colspan(&self) -> Option<u32> {
        self.node.value.attr(local_name!("colspan")).map(|value| AttrValue::String(value.to_string()).as_uint())
    }

    fn get_rowspan(&self) -> Option<u32> {
        self.node.value.attr(local_name!("rowspan")).map(|value| AttrValue::String(value.to_string()).as_uint())
    }

    fn pseudo_element(&self) -> Option<PseudoElement> {
        self.pseudo
    }
}
