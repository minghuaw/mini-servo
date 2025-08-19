use std::cell::RefCell;

use ::layout::{dom::NodeExt, fragment_tree::Fragment, replaced::CanvasInfo};
use blitz_dom::{ElementData, local_name, node::NodeFlags};
use html5ever::ns;
use layout::{
    ArcRefCell,
    dom::{DOMLayoutData, LayoutBox, PseudoLayoutData},
};
use layout_api::{
    LayoutDamage, LayoutElementType, LayoutNodeType, StyleData,
    wrapper_traits::{LayoutNode, ThreadSafeLayoutNode},
};
use script::layout_dom::LayoutNodeExt;
use style::{
    Atom,
    dom::{NodeInfo, TDocument, TNode, TShadowRoot},
    selector_parser::RestyleDamage,
};

use crate::layout::{BlitzLayoutElement, SafeBlitzLayoutNode};

use super::BlitzNode;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct BlitzLayoutNode<'dom> {
    pub value: BlitzNode<'dom>,
}

impl<'dom> NodeInfo for BlitzLayoutNode<'dom> {
    fn is_element(&self) -> bool {
        NodeInfo::is_element(&self.value)
    }

    fn is_text_node(&self) -> bool {
        NodeInfo::is_text_node(&self.value)
    }
}

impl<'dom> TDocument for BlitzLayoutNode<'dom> {
    type ConcreteNode = Self;

    fn as_node(&self) -> Self::ConcreteNode {
        Self {
            value: TDocument::as_node(&self.value),
        }
    }

    fn is_html_document(&self) -> bool {
        TDocument::is_html_document(&self.value)
    }

    fn quirks_mode(&self) -> style::context::QuirksMode {
        TDocument::quirks_mode(&self.value)
    }

    fn shared_lock(&self) -> &style::shared_lock::SharedRwLock {
        TDocument::shared_lock(&self.value)
    }
}

impl<'dom> TShadowRoot for BlitzLayoutNode<'dom> {
    type ConcreteNode = Self;

    fn as_node(&self) -> Self::ConcreteNode {
        Self {
            value: TShadowRoot::as_node(&self.value),
        }
    }

    fn host(&self) -> <Self::ConcreteNode as TNode>::ConcreteElement {
        BlitzLayoutElement {
            value: TShadowRoot::host(&self.value),
        }
    }

    fn style_data<'a>(&self) -> Option<&'a style::stylist::CascadeData>
    where
        Self: 'a,
    {
        TShadowRoot::style_data(&self.value)
    }
}

impl<'dom> TNode for BlitzLayoutNode<'dom> {
    type ConcreteElement = BlitzLayoutElement<'dom>;

    type ConcreteDocument = BlitzLayoutNode<'dom>;

    type ConcreteShadowRoot = BlitzLayoutNode<'dom>;

    fn parent_node(&self) -> Option<Self> {
        self.value.parent_node().map(|value| Self { value })
    }

    fn first_child(&self) -> Option<Self> {
        self.value.first_child().map(|value| Self { value })
    }

    fn last_child(&self) -> Option<Self> {
        self.value.last_child().map(|value| Self { value })
    }

    fn prev_sibling(&self) -> Option<Self> {
        self.value.prev_sibling().map(|value| Self { value })
    }

    fn next_sibling(&self) -> Option<Self> {
        self.value.next_sibling().map(|value| Self { value })
    }

    fn owner_doc(&self) -> Self::ConcreteDocument {
        Self {
            value: self.value.owner_doc(),
        }
    }

    fn is_in_document(&self) -> bool {
        self.value.is_in_document()
    }

    fn traversal_parent(&self) -> Option<Self::ConcreteElement> {
        self.value
            .traversal_parent()
            .map(|value| BlitzLayoutElement { value })
    }

    fn opaque(&self) -> style::dom::OpaqueNode {
        self.value.opaque()
    }

    fn debug_id(self) -> usize {
        self.value.debug_id()
    }

    fn as_element(&self) -> Option<Self::ConcreteElement> {
        self.value
            .as_element()
            .map(|value| BlitzLayoutElement { value })
    }

    fn as_document(&self) -> Option<Self::ConcreteDocument> {
        self.value.as_document().map(|value| Self { value })
    }

    fn as_shadow_root(&self) -> Option<Self::ConcreteShadowRoot> {
        self.value.as_shadow_root().map(|value| Self { value })
    }
}

fn element_data_local_name_to_layout_node_type(el: &ElementData) -> LayoutElementType {
    match el.name.ns {
        ns!(svg) => match el.name.local {
            local_name!("image") => LayoutElementType::SVGImageElement,
            local_name!("svg") => LayoutElementType::SVGSVGElement,
            _ => LayoutElementType::Element,
        },
        ns!(html) => match el.name.local {
            local_name!("body") => LayoutElementType::HTMLBodyElement,
            local_name!("br") => LayoutElementType::HTMLBRElement,
            local_name!("canvas") => LayoutElementType::HTMLCanvasElement,
            local_name!("html") => LayoutElementType::HTMLHtmlElement,
            local_name!("iframe") => LayoutElementType::HTMLIFrameElement,
            local_name!("img") | local_name!("image") => LayoutElementType::HTMLImageElement,
            local_name!("audio") | local_name!("video") => LayoutElementType::HTMLMediaElement,
            local_name!("input") => LayoutElementType::HTMLInputElement,
            local_name!("optgroup") => LayoutElementType::HTMLOptGroupElement,
            local_name!("option") => LayoutElementType::HTMLOptionElement,
            local_name!("object") => LayoutElementType::HTMLObjectElement,
            local_name!("p") => LayoutElementType::HTMLParagraphElement,
            local_name!("pre") => LayoutElementType::HTMLPreElement,
            local_name!("select") => LayoutElementType::HTMLSelectElement,
            local_name!("td") => LayoutElementType::HTMLTableCellElement,
            local_name!("col") => LayoutElementType::HTMLTableColElement,
            local_name!("table") => LayoutElementType::HTMLTableElement,
            local_name!("tr") => LayoutElementType::HTMLTableRowElement,
            local_name!("thead") => LayoutElementType::HTMLTableSectionElement,
            local_name!("textarea") => LayoutElementType::HTMLTextAreaElement,
            _ => LayoutElementType::Element,
        },
        _ => LayoutElementType::Element,
    }
}

impl<'dom> LayoutNode<'dom> for BlitzLayoutNode<'dom> {
    type ConcreteThreadSafeLayoutNode = SafeBlitzLayoutNode<'dom>;

    fn to_threadsafe(&self) -> Self::ConcreteThreadSafeLayoutNode {
        SafeBlitzLayoutNode {
            node: *self,
            pseudo: None,
        }
    }

    fn type_id(&self) -> LayoutNodeType {
        match &self.value.data {
            blitz_dom::NodeData::Document => {
                LayoutNodeType::Element(LayoutElementType::HTMLHtmlElement)
            } // TODO: do they match?
            blitz_dom::NodeData::Element(el) => {
                LayoutNodeType::Element(element_data_local_name_to_layout_node_type(el))
            }
            blitz_dom::NodeData::AnonymousBlock(el) => {
                LayoutNodeType::Element(element_data_local_name_to_layout_node_type(el))
            }
            blitz_dom::NodeData::Text(_text_node_data) => LayoutNodeType::Text,
            blitz_dom::NodeData::Comment => unreachable!(),
        }
    }

    unsafe fn initialize_style_and_layout_data<
        RequestedLayoutDataType: layout_api::wrapper_traits::LayoutDataTrait,
    >(
        &self,
    ) {
        let mut stylo_element_data = self.value.stylo_element_data.borrow_mut();
        *stylo_element_data = Default::default();
        let mut layout_data = self.value.layout_data.borrow_mut();
        *layout_data = Default::default();
    }

    fn initialize_layout_data<
        RequestedLayoutDataType: layout_api::wrapper_traits::LayoutDataTrait,
    >(
        &self,
    ) {
        let mut layout_data = self.value.layout_data.borrow_mut();
        *layout_data = Some(Box::<RequestedLayoutDataType>::default());
    }

    fn style_data(&self) -> Option<&'dom StyleData> {
        // FIXME: this is a hack, may be UB
        let sd = unsafe { &*(self.value.stylo_element_data.as_ptr()) };
        sd.as_ref()
    }

    fn layout_data(&self) -> Option<&'dom layout_api::GenericLayoutData> {
        let ld = unsafe { &*(self.value.layout_data.as_ptr()) };
        ld.as_ref().map(|d| &**d)
    }

    fn is_connected(&self) -> bool {
        // FIXME: hack, currently just checks it the node belongs to a document
        self.value.flags == NodeFlags::IS_IN_DOCUMENT
    }
}

impl<'dom> LayoutNodeExt<'dom> for BlitzLayoutNode<'dom> {
    fn is_text_input(&self) -> bool {
        match &self.value.data {
            blitz_dom::NodeData::Element(el) => match el.name.local {
                local_name!("textarea") => true,
                local_name!("input") => el.attr(local_name!("type")).map_or(true, |s| s != "color"),
                _ => false,
            },
            _ => false,
        }
    }
}

impl<'dom> NodeExt<'dom> for BlitzLayoutNode<'dom> {
    fn as_image(
        &self,
    ) -> Option<(
        Option<net_traits::image_cache::Image>,
        layout::geom::PhysicalSize<f64>,
    )> {
        None
    }

    fn as_canvas(&self) -> Option<(CanvasInfo, layout::geom::PhysicalSize<f64>)> {
        None
    }

    fn as_iframe(&self) -> Option<(base::id::PipelineId, base::id::BrowsingContextId)> {
        None
    }

    fn as_video(
        &self,
    ) -> Option<(
        Option<webrender_api::ImageKey>,
        Option<layout::geom::PhysicalSize<f64>>,
    )> {
        None
    }

    fn as_typeless_object_with_data_attribute(&self) -> Option<String> {
        // NOTE: this is not supported but need to return something so it won't panic
        None
    }

    fn style(
        &self,
        context: &style::context::SharedStyleContext,
    ) -> style::servo_arc::Arc<style::properties::ComputedValues> {
        self.to_threadsafe().style(context)
    }

    fn layout_data_mut(
        &self,
    ) -> atomic_refcell::AtomicRefMut<'dom, layout::dom::InnerDOMLayoutData> {
        if LayoutNode::layout_data(self).is_none() {
            self.initialize_layout_data::<DOMLayoutData>();
        }
        LayoutNode::layout_data(self)
            .unwrap()
            .as_any()
            .downcast_ref::<DOMLayoutData>()
            .unwrap()
            .0
            .borrow_mut()
    }

    fn layout_data(
        &self,
    ) -> Option<atomic_refcell::AtomicRef<'dom, layout::dom::InnerDOMLayoutData>> {
        LayoutNode::layout_data(self).map(|data| {
            data.as_any()
                .downcast_ref::<DOMLayoutData>()
                .unwrap()
                .0
                .borrow()
        })
    }

    fn element_box_slot(&self) -> layout::dom::BoxSlot<'dom> {
        self.layout_data_mut().self_box.clone().into()
    }

    fn pseudo_element_box_slot(
        &self,
        pseudo_element: style::selector_parser::PseudoElement,
    ) -> layout::dom::BoxSlot<'dom> {
        let mut layout_data = self.layout_data_mut();
        let box_slot = ArcRefCell::new(None);
        layout_data.pseudo_boxes.push(PseudoLayoutData {
            pseudo: pseudo_element,
            box_slot: box_slot.clone(),
        });
        box_slot.into()
    }

    fn unset_all_boxes(&self) {
        let mut layout_data = self.layout_data_mut();
        *layout_data.self_box.borrow_mut() = None;
        layout_data.pseudo_boxes.clear();
    }

    fn unset_all_pseudo_boxes(&self) {
        self.layout_data_mut().pseudo_boxes.clear();
    }

    fn fragments_for_pseudo(
        &self,
        pseudo_element: Option<style::selector_parser::PseudoElement>,
    ) -> Vec<Fragment> {
        let Some(layout_data) = NodeExt::layout_data(self) else {
            return vec![];
        };
        let Some(layout_data) = layout_data.for_pseudo(pseudo_element) else {
            return vec![];
        };
        layout_data
            .as_ref()
            .map(LayoutBox::fragments)
            .unwrap_or_default()
    }

    fn clear_fragment_layout_cache(&self) {
        let data = self.layout_data_mut();
        if let Some(data) = data.self_box.borrow_mut().as_ref() {
            data.clear_fragment_layout_cache();
        }

        for pseudo_layout_data in data.pseudo_boxes.iter() {
            if let Some(layout_box) = pseudo_layout_data.box_slot.borrow().as_ref() {
                layout_box.clear_fragment_layout_cache();
            }
        }
    }

    fn repair_style(&self, context: &style::context::SharedStyleContext) {
        let data = self.layout_data_mut();
        if let Some(layout_object) = &*data.self_box.borrow() {
            let style = self.to_threadsafe().style(context);
            layout_object.repair_style(context, self, &style);
        }

        for pseudo_layout_data in data.pseudo_boxes.iter() {
            if let Some(layout_box) = pseudo_layout_data.box_slot.borrow().as_ref() {
                if let Some(node) = self.to_threadsafe().with_pseudo(pseudo_layout_data.pseudo) {
                    layout_box.repair_style(context, self, &node.style(context));
                }
            }
        }
    }

    fn take_restyle_damage(&self) -> layout_api::LayoutDamage {
        let damage = LayoutNode::style_data(self)
            .map(|style_data| std::mem::take(&mut style_data.element_data.borrow_mut().damage))
            .unwrap_or_else(RestyleDamage::reconstruct);
        LayoutDamage::from_bits_retain(damage.bits())
    }
}
