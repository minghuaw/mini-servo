use style::{
    context::RegisteredSpeculativePainters, servo::media_queries::FontMetricsProvider,
    values::computed::Au,
};

#[derive(Debug)]
pub struct DummyFontMetricsProvider;
impl FontMetricsProvider for DummyFontMetricsProvider {
    fn query_font_metrics(
        &self,
        _vertical: bool,
        _font: &style::properties::style_structs::Font,
        _base_size: style::values::computed::CSSPixelLength,
        _flags: style::values::computed::font::QueryFontMetricsFlags,
    ) -> style::font_metrics::FontMetrics {
        Default::default()
    }

    fn base_size_for_generic(
        &self,
        _generic: style::values::computed::font::GenericFontFamily,
    ) -> style::values::computed::Length {
        style::values::computed::Length::from(Au::from_f32_px(13.0))
    }
}

pub struct DummyRegisteredSpeculativePainters;
impl RegisteredSpeculativePainters for DummyRegisteredSpeculativePainters {
    fn get(&self, _name: &style::Atom) -> Option<&dyn style::context::RegisteredSpeculativePainter> {
        None
    }
}
