use style::{context::RegisteredSpeculativePainters, servo::media_queries::FontMetricsProvider, values::computed::Au};

#[derive(Debug)]
pub(crate) struct DummyFontMetricsProvider;
impl FontMetricsProvider for DummyFontMetricsProvider {
    fn query_font_metrics(
        &self,
        vertical: bool,
        font: &style::properties::style_structs::Font,
        base_size: style::values::computed::CSSPixelLength,
        flags: style::values::computed::font::QueryFontMetricsFlags,
    ) -> style::font_metrics::FontMetrics {
        Default::default()
    }

    fn base_size_for_generic(&self, generic: style::values::computed::font::GenericFontFamily) -> style::values::computed::Length {
        style::values::computed::Length::from(Au::from_f32_px(13.0))
    }
}

pub(crate) struct DummyRegisteredSpeculativePainters;
impl RegisteredSpeculativePainters for DummyRegisteredSpeculativePainters {
    fn get(&self, name: &style::Atom) -> Option<&dyn style::context::RegisteredSpeculativePainter> {
        None
    }
}