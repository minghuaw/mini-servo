use ::rcdom::{Handle, RcDom};
use style::dom::TElement;

use crate::wrapper::RcNode;

impl<'a> selectors::Element for &'a RcNode {
    type Impl;

    fn opaque(&self) -> selectors::OpaqueElement {
        todo!()
    }

    fn parent_element(&self) -> Option<Self> {
        todo!()
    }

    fn parent_node_is_shadow_root(&self) -> bool {
        todo!()
    }

    fn containing_shadow_host(&self) -> Option<Self> {
        todo!()
    }

    fn is_pseudo_element(&self) -> bool {
        todo!()
    }

    fn prev_sibling_element(&self) -> Option<Self> {
        todo!()
    }

    fn next_sibling_element(&self) -> Option<Self> {
        todo!()
    }

    fn first_element_child(&self) -> Option<Self> {
        todo!()
    }

    fn is_html_element_in_html_document(&self) -> bool {
        todo!()
    }

    fn has_local_name(&self, local_name: &<Self::Impl as selectors::SelectorImpl>::BorrowedLocalName) -> bool {
        todo!()
    }

    fn has_namespace(&self, ns: &<Self::Impl as selectors::SelectorImpl>::BorrowedNamespaceUrl) -> bool {
        todo!()
    }

    fn is_same_type(&self, other: &Self) -> bool {
        todo!()
    }

    fn attr_matches(
        &self,
        ns: &selectors::attr::NamespaceConstraint<&<Self::Impl as selectors::SelectorImpl>::NamespaceUrl>,
        local_name: &<Self::Impl as selectors::SelectorImpl>::LocalName,
        operation: &selectors::attr::AttrSelectorOperation<&<Self::Impl as selectors::SelectorImpl>::AttrValue>,
    ) -> bool {
        todo!()
    }

    fn match_non_ts_pseudo_class(
        &self,
        pc: &<Self::Impl as selectors::SelectorImpl>::NonTSPseudoClass,
        context: &mut selectors::context::MatchingContext<Self::Impl>,
    ) -> bool {
        todo!()
    }

    fn match_pseudo_element(
        &self,
        pe: &<Self::Impl as selectors::SelectorImpl>::PseudoElement,
        context: &mut selectors::context::MatchingContext<Self::Impl>,
    ) -> bool {
        todo!()
    }

    fn apply_selector_flags(&self, flags: selectors::matching::ElementSelectorFlags) {
        todo!()
    }

    fn is_link(&self) -> bool {
        todo!()
    }

    fn is_html_slot_element(&self) -> bool {
        todo!()
    }

    fn has_id(
        &self,
        id: &<Self::Impl as selectors::SelectorImpl>::Identifier,
        case_sensitivity: selectors::attr::CaseSensitivity,
    ) -> bool {
        todo!()
    }

    fn has_class(
        &self,
        name: &<Self::Impl as selectors::SelectorImpl>::Identifier,
        case_sensitivity: selectors::attr::CaseSensitivity,
    ) -> bool {
        todo!()
    }

    fn has_custom_state(
        &self,
        name: &<Self::Impl as selectors::SelectorImpl>::Identifier,
    ) -> bool {
        todo!()
    }

    fn imported_part(
        &self,
        name: &<Self::Impl as selectors::SelectorImpl>::Identifier,
    ) -> Option<<Self::Impl as selectors::SelectorImpl>::Identifier> {
        todo!()
    }

    fn is_part(&self, name: &<Self::Impl as selectors::SelectorImpl>::Identifier) -> bool {
        todo!()
    }

    fn is_empty(&self) -> bool {
        todo!()
    }

    fn is_root(&self) -> bool {
        todo!()
    }

    fn add_element_unique_hashes(&self, filter: &mut selectors::bloom::BloomFilter) -> bool {
        todo!()
    }
}

impl<'a> TElement for &'a RcNode {
    type ConcreteNode;

    type TraversalChildrenIterator;

    fn as_node(&self) -> Self::ConcreteNode {
        todo!()
    }

    fn traversal_children(&self) -> style::dom::LayoutIterator<Self::TraversalChildrenIterator> {
        todo!()
    }

    fn is_html_element(&self) -> bool {
        todo!()
    }

    fn is_mathml_element(&self) -> bool {
        todo!()
    }

    fn is_svg_element(&self) -> bool {
        todo!()
    }

    fn style_attribute(&self) -> Option<style::servo_arc::ArcBorrow<style::shared_lock::Locked<style::properties::PropertyDeclarationBlock>>> {
        todo!()
    }

    fn animation_rule(
        &self,
        _: &style::context::SharedStyleContext,
    ) -> Option<style::servo_arc::Arc<style::shared_lock::Locked<style::properties::PropertyDeclarationBlock>>> {
        todo!()
    }

    fn transition_rule(
        &self,
        context: &style::context::SharedStyleContext,
    ) -> Option<style::servo_arc::Arc<style::shared_lock::Locked<style::properties::PropertyDeclarationBlock>>> {
        todo!()
    }

    fn state(&self) -> stylo_dom::ElementState {
        todo!()
    }

    fn has_part_attr(&self) -> bool {
        todo!()
    }

    fn exports_any_part(&self) -> bool {
        todo!()
    }

    fn id(&self) -> Option<&style::Atom> {
        todo!()
    }

    fn each_class<F>(&self, callback: F)
    where
        F: FnMut(&style::values::AtomIdent) {
        todo!()
    }

    fn each_custom_state<F>(&self, callback: F)
    where
        F: FnMut(&style::values::AtomIdent) {
        todo!()
    }

    fn each_attr_name<F>(&self, callback: F)
    where
        F: FnMut(&style::LocalName) {
        todo!()
    }

    fn has_dirty_descendants(&self) -> bool {
        todo!()
    }

    fn has_snapshot(&self) -> bool {
        todo!()
    }

    fn handled_snapshot(&self) -> bool {
        todo!()
    }

    unsafe fn set_handled_snapshot(&self) {
        todo!()
    }

    unsafe fn set_dirty_descendants(&self) {
        todo!()
    }

    unsafe fn unset_dirty_descendants(&self) {
        todo!()
    }

    fn store_children_to_process(&self, n: isize) {
        todo!()
    }

    fn did_process_child(&self) -> isize {
        todo!()
    }

    unsafe fn ensure_data(&self) -> AtomicRefMut<style::data::ElementData> {
        todo!()
    }

    unsafe fn clear_data(&self) {
        todo!()
    }

    fn has_data(&self) -> bool {
        todo!()
    }

    fn borrow_data(&self) -> Option<AtomicRef<style::data::ElementData>> {
        todo!()
    }

    fn mutate_data(&self) -> Option<AtomicRefMut<style::data::ElementData>> {
        todo!()
    }

    fn skip_item_display_fixup(&self) -> bool {
        todo!()
    }

    fn may_have_animations(&self) -> bool {
        todo!()
    }

    fn has_animations(&self, context: &style::context::SharedStyleContext) -> bool {
        todo!()
    }

    fn has_css_animations(
        &self,
        context: &style::context::SharedStyleContext,
        pseudo_element: Option<style::selector_parser::PseudoElement>,
    ) -> bool {
        todo!()
    }

    fn has_css_transitions(
        &self,
        context: &style::context::SharedStyleContext,
        pseudo_element: Option<style::selector_parser::PseudoElement>,
    ) -> bool {
        todo!()
    }

    fn shadow_root(&self) -> Option<<Self::ConcreteNode as style::dom::TNode>::ConcreteShadowRoot> {
        todo!()
    }

    fn containing_shadow(&self) -> Option<<Self::ConcreteNode as style::dom::TNode>::ConcreteShadowRoot> {
        todo!()
    }

    fn lang_attr(&self) -> Option<style::selector_parser::AttrValue> {
        todo!()
    }

    fn match_element_lang(&self, override_lang: Option<Option<style::selector_parser::AttrValue>>, value: &style::selector_parser::Lang) -> bool {
        todo!()
    }

    fn is_html_document_body_element(&self) -> bool {
        todo!()
    }

    fn synthesize_presentational_hints_for_legacy_attributes<V>(
        &self,
        visited_handling: selectors::context::VisitedHandlingMode,
        hints: &mut V,
    ) where
        V: selectors::sink::Push<style::applicable_declarations::ApplicableDeclarationBlock> {
        todo!()
    }

    fn local_name(&self) -> &<style::selector_parser::SelectorImpl as selectors::parser::SelectorImpl>::BorrowedLocalName {
        todo!()
    }

    fn namespace(&self)
        -> &<style::selector_parser::SelectorImpl as selectors::parser::SelectorImpl>::BorrowedNamespaceUrl {
        todo!()
    }

    fn query_container_size(
        &self,
        display: &style::values::computed::Display,
    ) -> euclid::default::Size2D<Option<app_units::Au>> {
        todo!()
    }

    fn has_selector_flags(&self, flags: selectors::matching::ElementSelectorFlags) -> bool {
        todo!()
    }

    fn relative_selector_search_direction(&self) -> selectors::matching::ElementSelectorFlags {
        todo!()
    }
}