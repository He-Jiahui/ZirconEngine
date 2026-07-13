use std::collections::HashMap;

use crate::asset::{FontAssetFaceMetrics, FontAssetLineMetrics};
use crate::core::framework::render::FontFaceId;
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::UiTextWritingMode;

use super::FontDatabase;

const FALLBACK_UNDERLINE_POSITION_EM: f32 = -0.1;
const FALLBACK_LINE_THICKNESS_EM: f32 = 0.05;
const FALLBACK_STRIKEOUT_POSITION_EM: f32 = 0.3;
const FALLBACK_ASCENDER_EM: f32 = 0.8;
const MIN_VISIBLE_TEXT_DECORATION_PX: f32 = 1.0;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct TextDecorationLineMetrics {
    pub(crate) position_px: f32,
    pub(crate) thickness_px: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct TextDecorationMetrics {
    pub(crate) ascender_px: f32,
    pub(crate) underline: TextDecorationLineMetrics,
    pub(crate) strikeout: TextDecorationLineMetrics,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TextDecorationKind {
    Underline,
    Strikethrough,
}

#[derive(Default)]
pub(crate) struct TextDecorationMetricsCache {
    entries: HashMap<(FontFaceId, u32), TextDecorationMetrics>,
}

impl TextDecorationMetrics {
    pub(crate) fn from_font_units(metrics: FontAssetFaceMetrics, display_px: f32) -> Self {
        let display_px = normalized_display_px(display_px);
        let units_per_em = f32::from(metrics.units_per_em.max(1));
        let unit_scale = display_px / units_per_em;
        Self {
            ascender_px: if metrics.units_per_em == 0 {
                display_px * FALLBACK_ASCENDER_EM
            } else {
                f32::from(metrics.ascender) * unit_scale
            },
            underline: scaled_line_metrics(
                metrics.underline,
                unit_scale,
                display_px * FALLBACK_UNDERLINE_POSITION_EM,
                display_px * FALLBACK_LINE_THICKNESS_EM,
            ),
            strikeout: scaled_line_metrics(
                metrics.strikeout,
                unit_scale,
                display_px * FALLBACK_STRIKEOUT_POSITION_EM,
                display_px * FALLBACK_LINE_THICKNESS_EM,
            ),
        }
    }

    pub(crate) fn from_face_bytes(bytes: &[u8], face_index: u32, display_px: f32) -> Option<Self> {
        let face = ttf_parser::Face::parse(bytes, face_index).ok()?;
        Some(Self::from_font_units(
            FontAssetFaceMetrics {
                units_per_em: face.units_per_em(),
                ascender: face.ascender(),
                descender: face.descender(),
                line_gap: face.line_gap(),
                uses_typographic_metrics: face
                    .tables()
                    .os2
                    .is_some_and(|table| table.use_typographic_metrics()),
                windows_ascender: face
                    .tables()
                    .os2
                    .map(|table| table.windows_ascender())
                    .unwrap_or(0),
                windows_descender: face
                    .tables()
                    .os2
                    .map(|table| table.windows_descender())
                    .unwrap_or(0),
                underline: face.underline_metrics().map(asset_line_metrics),
                strikeout: face.strikeout_metrics().map(asset_line_metrics),
            },
            display_px,
        ))
    }

    pub(crate) fn fallback(display_px: f32) -> Self {
        Self::from_font_units(FontAssetFaceMetrics::default(), display_px)
    }

    pub(crate) fn aggregate_fallback_thicknesses(
        mut self,
        fallback_metrics: impl IntoIterator<Item = Self>,
    ) -> Self {
        for metrics in fallback_metrics {
            self.underline.thickness_px = self
                .underline
                .thickness_px
                .max(metrics.underline.thickness_px);
            self.strikeout.thickness_px = self
                .strikeout
                .thickness_px
                .max(metrics.strikeout.thickness_px);
        }
        self
    }

    pub(crate) fn line(self, kind: TextDecorationKind) -> TextDecorationLineMetrics {
        match kind {
            TextDecorationKind::Underline => self.underline,
            TextDecorationKind::Strikethrough => self.strikeout,
        }
    }
}

impl TextDecorationMetricsCache {
    pub(crate) fn resolve(
        &mut self,
        font_database: &FontDatabase,
        face: FontFaceId,
        display_px: f32,
    ) -> TextDecorationMetrics {
        let display_px = normalized_display_px(display_px);
        let key = (face, display_px.to_bits());
        if let Some(metrics) = self.entries.get(&key) {
            return *metrics;
        }
        let metrics = font_database
            .face_bytes(face)
            .ok()
            .zip(font_database.face_index(face).ok())
            .and_then(|(bytes, face_index)| {
                TextDecorationMetrics::from_face_bytes(bytes.as_ref(), face_index, display_px)
            })
            .unwrap_or_else(|| TextDecorationMetrics::fallback(display_px));
        self.entries.insert(key, metrics);
        metrics
    }
}

pub(crate) fn text_decoration_frame(
    run_frame: UiFrame,
    baseline_coordinate: f32,
    writing_mode: UiTextWritingMode,
    metrics: TextDecorationMetrics,
    kind: TextDecorationKind,
) -> UiFrame {
    let line = metrics.line(kind);
    let thickness = line.thickness_px.abs().max(MIN_VISIBLE_TEXT_DECORATION_PX);
    let center = baseline_coordinate - line.position_px;
    if matches!(writing_mode, UiTextWritingMode::VerticalRl) {
        UiFrame::new(
            center - thickness * 0.5,
            run_frame.y,
            thickness,
            run_frame.height,
        )
    } else {
        UiFrame::new(
            run_frame.x,
            center - thickness * 0.5,
            run_frame.width,
            thickness,
        )
    }
}

fn scaled_line_metrics(
    metrics: Option<FontAssetLineMetrics>,
    unit_scale: f32,
    fallback_position_px: f32,
    fallback_thickness_px: f32,
) -> TextDecorationLineMetrics {
    metrics.map_or(
        TextDecorationLineMetrics {
            position_px: fallback_position_px,
            thickness_px: fallback_thickness_px,
        },
        |metrics| TextDecorationLineMetrics {
            position_px: f32::from(metrics.position) * unit_scale,
            thickness_px: f32::from(metrics.thickness).abs() * unit_scale,
        },
    )
}

fn asset_line_metrics(metrics: ttf_parser::LineMetrics) -> FontAssetLineMetrics {
    FontAssetLineMetrics {
        position: metrics.position,
        thickness: metrics.thickness,
    }
}

fn normalized_display_px(display_px: f32) -> f32 {
    if display_px.is_finite() {
        display_px.max(1.0)
    } else {
        1.0
    }
}

#[cfg(test)]
#[path = "decoration_metrics/tests.rs"]
mod tests;
