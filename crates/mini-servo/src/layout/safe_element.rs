use atomic_refcell::AtomicRef;
use blitz_dom::node::NodeKind;
use layout_api::wrapper_traits::{ThreadSafeLayoutElement, ThreadSafeLayoutNode};
use style::{dom::TElement, selector_parser::{PseudoElement, SelectorImpl}};

use crate::layout::{BlitzLayoutElement, BlitzLayoutNode, SafeBlitzLayoutNode};

use super::BlitzNode;

#[derive(Debug, Clone, Copy,)]
pub struct SafeBlitzLayoutElement<'dom> {
    pub element: BlitzLayoutElement<'dom>,
    pub pseudo: Option<PseudoElement>,
}

impl<'dom> ::selectors::Element for SafeBlitzLayoutElement<'dom> {
    type Impl = SelectorImpl;

    fn opaque(&self) -> selectors::OpaqueElement {
        self.element.opaque()
    }

    fn parent_element(&self) -> Option<Self> {
        self.element.parent_element().map(|value| Self { element: value, pseudo: None }  )
    }

    fn parent_node_is_shadow_root(&self) -> bool {
        self.element.parent_node_is_shadow_root()
    }

    fn containing_shadow_host(&self) -> Option<Self> {
        self.element.containing_shadow_host().map(|value| Self { element: value, pseudo: None } )
    }

    fn is_pseudo_element(&self) -> bool {
        self.element.is_pseudo_element()
    }

    // Skip non-element nodes
    fn prev_sibling_element(&self) -> Option<Self> {
        log::warn!("Element::prev_sibling_element called");
        None
    }

    // Skip non-element nodes
    fn next_sibling_element(&self) -> Option<Self> {
        log::warn!("Element::next_sibling_element called");
        None
    }

    // Skip non-element nodes
    fn first_element_child(&self) -> Option<Self> {
        log::warn!("Element::first_element_child called");
        None
    }

    fn is_html_element_in_html_document(&self) -> bool {
        self.element.is_html_element_in_html_document()
    }

    fn has_local_name(&self, local_name: &<Self::Impl as selectors::SelectorImpl>::BorrowedLocalName) -> bool {
        self.element.has_local_name(local_name)
    }

    fn has_namespace(&self, ns: &<Self::Impl as selectors::SelectorImpl>::BorrowedNamespaceUrl) -> bool {
        self.element.has_namespace(ns)
    }

    fn is_same_type(&self, other: &Self) -> bool {
        self.element.is_same_type(&other.element)
    }

    fn attr_matches(
        &self,
        ns: &selectors::attr::NamespaceConstraint<&<Self::Impl as selectors::SelectorImpl>::NamespaceUrl>,
        local_name: &<Self::Impl as selectors::SelectorImpl>::LocalName,
        operation: &selectors::attr::AttrSelectorOperation<&<Self::Impl as selectors::SelectorImpl>::AttrValue>,
    ) -> bool {
        self.element.attr_matches(ns, local_name, operation)
    }

    fn match_non_ts_pseudo_class(
        &self,
        pc: &<Self::Impl as selectors::SelectorImpl>::NonTSPseudoClass,
        context: &mut selectors::context::MatchingContext<Self::Impl>,
    ) -> bool {
        self.element.match_non_ts_pseudo_class(pc, context)
    }

    fn match_pseudo_element(
        &self,
        pe: &<Self::Impl as selectors::SelectorImpl>::PseudoElement,
        context: &mut selectors::context::MatchingContext<Self::Impl>,
    ) -> bool {
        self.element.match_pseudo_element(pe, context)
    }

    fn apply_selector_flags(&self, flags: selectors::matching::ElementSelectorFlags) {
        self.element.apply_selector_flags(flags);
    }

    fn is_link(&self) -> bool {
        self.element.is_link()
    }

    fn is_html_slot_element(&self) -> bool {
        self.element.is_html_slot_element()
    }

    fn has_id(
        &self,
        id: &<Self::Impl as selectors::SelectorImpl>::Identifier,
        case_sensitivity: selectors::attr::CaseSensitivity,
    ) -> bool {
        self.element.has_id(id, case_sensitivity)
    }

    fn has_class(
        &self,
        name: &<Self::Impl as selectors::SelectorImpl>::Identifier,
        case_sensitivity: selectors::attr::CaseSensitivity,
    ) -> bool {
        self.element.has_class(name, case_sensitivity)
    }

    fn has_custom_state(
        &self,
        name: &<Self::Impl as selectors::SelectorImpl>::Identifier,
    ) -> bool {
        self.element.has_custom_state(name)
    }

    fn imported_part(
        &self,
        name: &<Self::Impl as selectors::SelectorImpl>::Identifier,
    ) -> Option<<Self::Impl as selectors::SelectorImpl>::Identifier> {
        self.element.imported_part(name)
    }

    fn is_part(&self, name: &<Self::Impl as selectors::SelectorImpl>::Identifier) -> bool {
        self.element.is_part(name)
    }

    fn is_empty(&self) -> bool {
        self.element.is_empty()
    }

    fn is_root(&self) -> bool {
        self.element.is_root()
    }

    fn add_element_unique_hashes(&self, filter: &mut selectors::bloom::BloomFilter) -> bool {
        self.element.add_element_unique_hashes(filter)
    }
}

impl<'dom> ThreadSafeLayoutElement<'dom> for SafeBlitzLayoutElement<'dom> {
    type ConcreteThreadSafeLayoutNode = SafeBlitzLayoutNode<'dom>;

    type ConcreteElement = BlitzLayoutElement<'dom>;

    fn as_node(&self) -> Self::ConcreteThreadSafeLayoutNode {
        let node = BlitzLayoutNode {
            value: self.element.value
        };
        SafeBlitzLayoutNode {
            node,
            pseudo: None,
        }
    }

    fn with_pseudo(&self, pseudo: style::selector_parser::PseudoElement) -> Option<Self> {
        log::warn!("ThreadSafeLayoutElement::with_pseudo called. Pseudo is not supported");
        None
    }

    fn type_id(&self) -> Option<layout_api::LayoutNodeType> {
        self.as_node().type_id()
    }

    fn unsafe_get(self) -> Self::ConcreteElement {
        self.element
    }

    fn get_local_name(&self) -> &blitz_dom::LocalName {
        self.element.local_name()
    }

    fn get_attr(&self, namespace: &blitz_dom::Namespace, name: &blitz_dom::LocalName) -> Option<&str> {
        self.element.value.attrs().and_then(|attrs| attrs.iter().find(|attr| {
            attr.name.local == *name && attr.name.ns == *namespace
        }).map(|attr| attr.value.as_str()))
    }

    fn get_attr_enum(&self, _namespace: &blitz_dom::Namespace, _name: &blitz_dom::LocalName) -> Option<&style::attr::AttrValue> {
        // self.element.value.attrs().and_then(|attrs| attrs.iter().find(|attr| {
        //     attr.name.local == *name && attr.name.ns == *namespace
        // }).map(|attr| &style::attr::AttrValue::String(attr.value.clone())))
        unreachable!("get_attr_enum should not be used")
    }

    fn style_data(&self) -> AtomicRef<style::data::ElementData> {
        self.element.borrow_data().expect("Unstyled layout node")
    }

    fn pseudo_element(&self) -> Option<style::selector_parser::PseudoElement> {
        self.pseudo
    }

    fn is_shadow_host(&self) -> bool {
        self.element.shadow_root().is_some()
    }

    fn is_body_element_of_html_element_root(&self) -> bool {
        self.element.value.is_html_document_body_element()
    }

    fn is_root(&self) -> bool {
        matches!(self.element.value.data.kind(), NodeKind::Document) // Document node is the root node in blitz
    }
}