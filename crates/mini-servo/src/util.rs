use euclid::{Scale, Size2D};
use style::{animation::DocumentAnimationSet, context::{QuirksMode, RegisteredSpeculativePainters, SharedStyleContext}, global_style_data::GLOBAL_STYLE_DATA, media_queries::{Device, MediaType}, properties::{style_structs::Font, ComputedValues}, queries::values::PrefersColorScheme, selector_parser::SnapshotMap, servo::media_queries::FontMetricsProvider, shared_lock::StylesheetGuards, stylist::Stylist, traversal_flags::TraversalFlags};
use style_traits::CSSPixel;

use crate::dummy::DummyFontMetricsProvider;

pub(crate) fn make_device(
    viewport_size: Size2D<f32, CSSPixel>,
) -> Device {
    let font = Font::initial_values();
    
    Device::new(
        MediaType::screen(),
        QuirksMode::NoQuirks,
        viewport_size,
        Scale::new(1.0),
        Box::new(DummyFontMetricsProvider),
        ComputedValues::initial_values_with_font_override(font),
        PrefersColorScheme::Light,
    )
}

pub(crate) fn make_stylist(
    device: Device,
) -> Stylist {
    Stylist::new(device, QuirksMode::NoQuirks)
}

pub(crate) fn make_shared_style_context<'a>(
    stylist: &'a Stylist, 
    guards: StylesheetGuards<'a>,
    snapshot_map: &'a SnapshotMap,
    registered_speculative_painters: &'a dyn RegisteredSpeculativePainters,
) -> SharedStyleContext<'a>  {
    SharedStyleContext {
        stylist,
        visited_styles_enabled: false,
        options: GLOBAL_STYLE_DATA.options.clone(),
        guards,
        current_time_for_animations: 0.0,
        traversal_flags: TraversalFlags::empty(),
        snapshot_map,
        animations: DocumentAnimationSet::default(),
        registered_speculative_painters,
    }
}

#[cfg(test)]
mod tests {
    use style::shared_lock::SharedRwLock;

    use crate::dummy::DummyRegisteredSpeculativePainters;

    use super::*;

    #[test]
    fn test_make_shared_style_context() {
        let device = make_device(Size2D::new(800.0, 600.0));
        let stylist = make_stylist(device);
        let guard = SharedRwLock::new();
        let guards = StylesheetGuards {
            author: &guard.read(),
            ua_or_user: &guard.read(),
        };
        let snapshot_map = SnapshotMap::new();
        let registered_speculative_painters = DummyRegisteredSpeculativePainters;

        let _shared_style_context = make_shared_style_context(&stylist, guards, &snapshot_map, &registered_speculative_painters);
    }
}