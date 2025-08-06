use atomic_refcell::{AtomicRef, AtomicRefMut};
use blitz_dom::stylo::Traverser;
use layout_api::wrapper_traits::ThreadSafeLayoutElement;
use style::{dom::{LayoutIterator, TElement}, selector_parser::SelectorImpl};
use script::layout_dom::DOMDescendantIterator;

use crate::layout::{BlitzLayoutNode, BlitzLayoutNodeIterator};

use super::BlitzNode;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct BlitzLayoutElement<'dom> {
    pub value: BlitzNode<'dom>,
}

impl<'dom> selectors::Element for BlitzLayoutElement<'dom> {
    type Impl = SelectorImpl;

    fn opaque(&self) -> selectors::OpaqueElement {
        self.value.opaque()
    }

    fn parent_element(&self) -> Option<Self> {
        self.value.parent_element().map(|value| Self { value })
    }

    fn parent_node_is_shadow_root(&self) -> bool {
        self.value.parent_node_is_shadow_root()
    }

    fn containing_shadow_host(&self) -> Option<Self> {
        self.value.containing_shadow_host().map(|value| Self { value })
    }

    fn is_pseudo_element(&self) -> bool {
        self.value.is_pseudo_element()
    }

    fn prev_sibling_element(&self) -> Option<Self> {
        self.value.prev_sibling_element().map(|value| Self { value })
    }

    fn next_sibling_element(&self) -> Option<Self> {
        self.value.next_sibling_element().map(|value| Self { value })
    }

    fn first_element_child(&self) -> Option<Self> {
        self.value.first_element_child().map(|value| Self { value })
    }

    fn is_html_element_in_html_document(&self) -> bool {
        self.value.is_html_element_in_html_document()
    }

    fn has_local_name(&self, local_name: &<Self::Impl as selectors::SelectorImpl>::BorrowedLocalName) -> bool {
        self.value.has_local_name(local_name)
    }

    fn has_namespace(&self, ns: &<Self::Impl as selectors::SelectorImpl>::BorrowedNamespaceUrl) -> bool {
        self.value.has_namespace(ns)
    }

    fn is_same_type(&self, other: &Self) -> bool {
        self.value.is_same_type(&other.value)
    }

    fn attr_matches(
        &self,
        ns: &selectors::attr::NamespaceConstraint<&<Self::Impl as selectors::SelectorImpl>::NamespaceUrl>,
        local_name: &<Self::Impl as selectors::SelectorImpl>::LocalName,
        operation: &selectors::attr::AttrSelectorOperation<&<Self::Impl as selectors::SelectorImpl>::AttrValue>,
    ) -> bool {
        self.value.attr_matches(ns, local_name, operation)
    }

    fn match_non_ts_pseudo_class(
        &self,
        pc: &<Self::Impl as selectors::SelectorImpl>::NonTSPseudoClass,
        context: &mut selectors::context::MatchingContext<Self::Impl>,
    ) -> bool {
        self.value.match_non_ts_pseudo_class(pc, context)
    }

    fn match_pseudo_element(
        &self,
        pe: &<Self::Impl as selectors::SelectorImpl>::PseudoElement,
        context: &mut selectors::context::MatchingContext<Self::Impl>,
    ) -> bool {
        self.value.match_pseudo_element(pe, context)
    }

    fn apply_selector_flags(&self, flags: selectors::matching::ElementSelectorFlags) {
        self.value.apply_selector_flags(flags);
    }

    fn is_link(&self) -> bool {
        self.value.is_link()
    }

    fn is_html_slot_element(&self) -> bool {
        self.value.is_html_slot_element()
    }

    fn has_id(
        &self,
        id: &<Self::Impl as selectors::SelectorImpl>::Identifier,
        case_sensitivity: selectors::attr::CaseSensitivity,
    ) -> bool {
        self.value.has_id(id, case_sensitivity)
    }

    fn has_class(
        &self,
        name: &<Self::Impl as selectors::SelectorImpl>::Identifier,
        case_sensitivity: selectors::attr::CaseSensitivity,
    ) -> bool {
        self.value.has_class(name, case_sensitivity)
    }

    fn has_custom_state(
        &self,
        name: &<Self::Impl as selectors::SelectorImpl>::Identifier,
    ) -> bool {
        self.value.has_custom_state(name)
    }

    fn imported_part(
        &self,
        name: &<Self::Impl as selectors::SelectorImpl>::Identifier,
    ) -> Option<<Self::Impl as selectors::SelectorImpl>::Identifier> {
        self.value.imported_part(name)
    }

    fn is_part(&self, name: &<Self::Impl as selectors::SelectorImpl>::Identifier) -> bool {
        self.value.is_part(name)
    }

    fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    fn is_root(&self) -> bool {
        self.value.is_root()
    }

    fn add_element_unique_hashes(&self, filter: &mut selectors::bloom::BloomFilter) -> bool {
        self.value.add_element_unique_hashes(filter)
    }
}

impl<'dom> TElement for BlitzLayoutElement<'dom> {
    type ConcreteNode = BlitzLayoutNode<'dom>;

    type TraversalChildrenIterator = BlitzLayoutNodeIterator<'dom>;

    fn as_node(&self) -> Self::ConcreteNode {
        BlitzLayoutNode {
            value: self.value
        }
    }

    fn traversal_children(&self) -> style::dom::LayoutIterator<Self::TraversalChildrenIterator> {
        let traverser = self.value.traversal_children();
        LayoutIterator(BlitzLayoutNodeIterator { traverser: traverser.0 })
    }

    fn is_html_element(&self) -> bool {
        self.value.is_html_element()
    }

    fn is_mathml_element(&self) -> bool {
        self.value.is_mathml_element()
    }

    fn is_svg_element(&self) -> bool {
        self.value.is_svg_element()
    }

    fn style_attribute(&self) -> Option<style::servo_arc::ArcBorrow<style::shared_lock::Locked<style::properties::PropertyDeclarationBlock>>> {
        self.value.style_attribute()
    }

    fn animation_rule(
        &self,
        rule: &style::context::SharedStyleContext,
    ) -> Option<style::servo_arc::Arc<style::shared_lock::Locked<style::properties::PropertyDeclarationBlock>>> {
        self.value.animation_rule(rule)
    }

    fn transition_rule(
        &self,
        context: &style::context::SharedStyleContext,
    ) -> Option<style::servo_arc::Arc<style::shared_lock::Locked<style::properties::PropertyDeclarationBlock>>> {
        self.value.transition_rule(context)
    }

    fn state(&self) -> stylo_dom::ElementState {
        self.value.state()
    }

    fn has_part_attr(&self) -> bool {
        self.value.has_part_attr()
    }

    fn exports_any_part(&self) -> bool {
        self.value.exports_any_part()
    }

    fn id(&self) -> Option<&style::Atom> {
        self.value.id()
    }

    fn each_class<F>(&self, callback: F)
    where
        F: FnMut(&style::values::AtomIdent) {
        self.value.each_class(callback);
    }

    fn each_custom_state<F>(&self, callback: F)
    where
        F: FnMut(&style::values::AtomIdent) {
        self.value.each_custom_state(callback);
    }

    fn each_attr_name<F>(&self, callback: F)
    where
        F: FnMut(&style::LocalName) {
        self.value.each_attr_name(callback);
    }

    fn has_dirty_descendants(&self) -> bool {
        self.value.has_dirty_descendants()
    }

    fn has_snapshot(&self) -> bool {
        self.value.has_snapshot()
    }

    fn handled_snapshot(&self) -> bool {
        self.value.handled_snapshot()
    }

    unsafe fn set_handled_snapshot(&self) {
        unsafe { self.value.set_handled_snapshot() }
    }

    unsafe fn set_dirty_descendants(&self) {
        unsafe { self.value.set_dirty_descendants() }
    }

    unsafe fn unset_dirty_descendants(&self) {
        unsafe { self.value.unset_dirty_descendants() }
    }

    fn store_children_to_process(&self, n: isize) {
        self.value.store_children_to_process(n);
    }

    fn did_process_child(&self) -> isize {
        self.value.did_process_child()
    }

    unsafe fn ensure_data(&self) -> AtomicRefMut<style::data::ElementData> {
        unsafe { self.value.ensure_data() }
    }

    unsafe fn clear_data(&self) {
        unsafe { self.value.clear_data() }
    }

    fn has_data(&self) -> bool {
        self.value.has_data()
    }

    fn borrow_data(&self) -> Option<AtomicRef<style::data::ElementData>> {
        self.value.borrow_data()
    }

    fn mutate_data(&self) -> Option<AtomicRefMut<style::data::ElementData>> {
        self.value.mutate_data()
    }

    fn skip_item_display_fixup(&self) -> bool {
        self.value.skip_item_display_fixup()
    }

    fn may_have_animations(&self) -> bool {
        self.value.may_have_animations()
    }

    fn has_animations(&self, context: &style::context::SharedStyleContext) -> bool {
        self.value.has_animations(context)
    }

    fn has_css_animations(
        &self,
        context: &style::context::SharedStyleContext,
        pseudo_element: Option<style::selector_parser::PseudoElement>,
    ) -> bool {
        self.value.has_css_animations(context, pseudo_element)
    }

    fn has_css_transitions(
        &self,
        context: &style::context::SharedStyleContext,
        pseudo_element: Option<style::selector_parser::PseudoElement>,
    ) -> bool {
        self.value.has_css_transitions(context, pseudo_element)
    }

    fn shadow_root(&self) -> Option<<Self::ConcreteNode as style::dom::TNode>::ConcreteShadowRoot> {
        self.value.shadow_root().map(|value| BlitzLayoutNode { value })
    }

    fn containing_shadow(&self) -> Option<<Self::ConcreteNode as style::dom::TNode>::ConcreteShadowRoot> {
        self.value.containing_shadow().map(|value| BlitzLayoutNode { value })
    }

    fn lang_attr(&self) -> Option<style::selector_parser::AttrValue> {
        self.value.lang_attr()
    }

    fn match_element_lang(&self, override_lang: Option<Option<style::selector_parser::AttrValue>>, value: &style::selector_parser::Lang) -> bool {
        self.value.match_element_lang(override_lang, value)
    }

    fn is_html_document_body_element(&self) -> bool {
        self.value.is_html_document_body_element()
    }

    fn synthesize_presentational_hints_for_legacy_attributes<V>(
        &self,
        visited_handling: selectors::context::VisitedHandlingMode,
        hints: &mut V,
    ) where
        V: selectors::sink::Push<style::applicable_declarations::ApplicableDeclarationBlock> {
        self.value.synthesize_presentational_hints_for_legacy_attributes(visited_handling, hints)
    }

    fn local_name(&self) -> &<style::selector_parser::SelectorImpl as selectors::parser::SelectorImpl>::BorrowedLocalName {
        self.value.local_name()
    }

    fn namespace(&self)
        -> &<style::selector_parser::SelectorImpl as selectors::parser::SelectorImpl>::BorrowedNamespaceUrl {
            self.value.namespace()
    }

    fn query_container_size(
        &self,
        display: &style::values::computed::Display,
    ) -> euclid::default::Size2D<Option<app_units::Au>> {
        self.value.query_container_size(display)
    }

    fn has_selector_flags(&self, flags: selectors::matching::ElementSelectorFlags) -> bool {
        self.value.has_selector_flags(flags)
    }

    fn relative_selector_search_direction(&self) -> selectors::matching::ElementSelectorFlags {
        self.value.relative_selector_search_direction()
    }
}
