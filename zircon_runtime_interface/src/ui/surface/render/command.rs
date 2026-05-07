use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;

use crate::ui::event_ui::UiNodeId;
use crate::ui::layout::{UiFrame, UiGeometry, UiLayoutMetrics, UiPixelSnapping};

use super::text_shape::text_paint_runs_from_shaped;
use super::{
    UiBrushPayload, UiBrushSet, UiClipMode, UiClipState, UiEditableTextState, UiPaintEffects,
    UiPaintElement, UiPaintPayload, UiRenderCommandKind, UiRenderResourceKey, UiRenderResourceKind,
    UiResolvedStyle, UiResolvedTextLayout, UiResolvedTextLine, UiTextPaint, UiTextPaintDecoration,
    UiTextPaintDecorationKind, UiTextRange, UiVisualAssetRef,
};

const TEXT_SELECTION_COLOR: &str = "#4d89ff66";
const TEXT_CARET_COLOR: &str = "#e8eef7";
const TEXT_COMPOSITION_UNDERLINE_COLOR: &str = "#4d89ff";
const TEXT_CARET_WIDTH: f32 = 1.0;
const TEXT_COMPOSITION_UNDERLINE_HEIGHT: f32 = 2.0;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiRenderCommand {
    pub node_id: UiNodeId,
    pub kind: UiRenderCommandKind,
    pub frame: UiFrame,
    pub clip_frame: Option<UiFrame>,
    pub z_index: i32,
    pub style: UiResolvedStyle,
    pub text_layout: Option<UiResolvedTextLayout>,
    pub text: Option<String>,
    pub image: Option<UiVisualAssetRef>,
    pub opacity: f32,
}

impl UiRenderCommand {
    pub fn to_paint_element(&self, paint_order: u64) -> UiPaintElement {
        self.to_paint_element_with_metrics(paint_order, UiLayoutMetrics::default())
    }

    pub fn to_paint_element_with_metrics(
        &self,
        paint_order: u64,
        metrics: UiLayoutMetrics,
    ) -> UiPaintElement {
        self.base_paint_element(paint_order, self.paint_payload(metrics), metrics)
    }

    pub fn to_paint_elements(&self, first_paint_order: u64) -> Vec<UiPaintElement> {
        self.to_paint_elements_with_metrics(first_paint_order, UiLayoutMetrics::default())
    }

    pub fn to_paint_elements_with_metrics(
        &self,
        first_paint_order: u64,
        metrics: UiLayoutMetrics,
    ) -> Vec<UiPaintElement> {
        let mut elements = Vec::new();
        if let Some(payload) = self.brush_payload(metrics) {
            elements.push(self.base_paint_element(first_paint_order, payload, metrics));
        }
        if let Some(payload) = self.text_payload() {
            elements.push(self.base_paint_element(
                first_paint_order + elements.len() as u64,
                payload,
                metrics,
            ));
        }
        if elements.is_empty() {
            elements.push(self.base_paint_element(
                first_paint_order,
                UiPaintPayload::Empty,
                metrics,
            ));
        }
        elements
    }

    fn base_paint_element(
        &self,
        paint_order: u64,
        payload: UiPaintPayload,
        metrics: UiLayoutMetrics,
    ) -> UiPaintElement {
        UiPaintElement {
            node_id: self.node_id,
            geometry: UiGeometry {
                clip_frame: self.clip_frame,
                ..UiGeometry::from_frame_with_metrics(self.frame, metrics)
            },
            clip: self.clip_frame.map(|frame| UiClipState {
                mode: UiClipMode::Scissor,
                frame: render_clip_frame(frame, metrics),
            }),
            z_index: self.z_index,
            paint_order,
            payload,
            effects: UiPaintEffects {
                opacity: self.opacity.clamp(0.0, 1.0),
                effects: Vec::new(),
            },
            cache_generation: Some(self.cache_generation()),
            debug_label: Some(format!("{:?}", self.kind)),
        }
    }

    fn cache_generation(&self) -> u64 {
        let bytes = serde_json::to_vec(self).unwrap_or_default();
        stable_hash64(&bytes)
    }

    fn paint_payload(&self, metrics: UiLayoutMetrics) -> UiPaintPayload {
        match self.kind {
            UiRenderCommandKind::Text => self.text_payload().unwrap_or(UiPaintPayload::Empty),
            UiRenderCommandKind::Quad | UiRenderCommandKind::Image => self
                .brush_payload(metrics)
                .or_else(|| self.text_payload())
                .unwrap_or(UiPaintPayload::Empty),
            UiRenderCommandKind::Group => self
                .brush_payload(metrics)
                .or_else(|| self.text_payload())
                .unwrap_or(UiPaintPayload::Empty),
        }
    }

    fn brush_payload(&self, metrics: UiLayoutMetrics) -> Option<UiPaintPayload> {
        let brushes = self.brush_set(metrics);
        if brushes.fill.is_some() || brushes.border.is_some() {
            Some(UiPaintPayload::Brush { brushes })
        } else {
            None
        }
    }

    fn text_payload(&self) -> Option<UiPaintPayload> {
        (self.text.as_ref().is_some_and(|text| !text.is_empty())
            || matches!(self.kind, UiRenderCommandKind::Text))
        .then(|| UiPaintPayload::Text {
            text: self.text_paint(),
        })
    }

    fn brush_set(&self, metrics: UiLayoutMetrics) -> UiBrushSet {
        let fill = if let Some(image) = self.image.as_ref() {
            Some(image_brush_payload(
                image_resource_key(image),
                self.frame,
                metrics,
            ))
        } else if let Some(color) = self.style.background_color.as_ref() {
            Some(if self.style.corner_radius > 0.0 {
                UiBrushPayload::rounded(color.clone(), self.style.corner_radius)
            } else {
                UiBrushPayload::solid(color.clone())
            })
        } else if matches!(self.kind, UiRenderCommandKind::Image) {
            Some(image_brush_payload(
                UiRenderResourceKey::new(UiRenderResourceKind::Image, "missing:image"),
                self.frame,
                metrics,
            ))
        } else {
            None
        };

        let border = self
            .style
            .border_color
            .as_ref()
            .filter(|_| self.style.border_width > 0.0)
            .map(|color| {
                let mut border = UiBrushPayload::border(color.clone(), self.style.border_width);
                if let UiBrushPayload::Border(payload) = &mut border {
                    payload.radius = self.style.corner_radius;
                }
                border
            });

        UiBrushSet { fill, border }
    }

    fn text_paint(&self) -> UiTextPaint {
        let source_text = self.text.clone().unwrap_or_default();
        let shaped = self.text_layout.as_ref().map(|layout| {
            let mut shaped = super::UiShapedText::from_resolved_layout(
                source_text.clone(),
                layout,
                self.style.text_render_mode,
            );
            let font_key = text_font_resource_key(&self.style);
            let atlas_resource = text_atlas_resource_key(&self.style, &font_key);
            shaped.font_key = Some(font_key);
            shaped.atlas_resource = Some(atlas_resource.clone());
            for line in &mut shaped.lines {
                for glyph in &mut line.glyphs {
                    if glyph.atlas_resource.is_none() {
                        glyph.atlas_resource = Some(atlas_resource.clone());
                    }
                }
            }
            shaped
        });

        let editable = self
            .text_layout
            .as_ref()
            .and_then(|layout| layout.editable.as_ref());

        let runs = shaped
            .as_ref()
            .map(|shaped| {
                text_paint_runs_from_shaped(
                    shaped,
                    &self.style.foreground_color,
                    &self.style.font,
                    &self.style.font_family,
                    self.style.font_size,
                    self.style.line_height,
                )
            })
            .unwrap_or_default();

        UiTextPaint {
            source_text,
            color: self.style.foreground_color.clone(),
            font: self.style.font.clone(),
            font_family: self.style.font_family.clone(),
            font_size: self.style.font_size,
            line_height: self.style.line_height,
            render_mode: self.style.text_render_mode,
            overflow: self.style.text_overflow,
            shaped,
            selection: editable.and_then(|editable| editable.selection.clone()),
            caret: editable.map(|editable| editable.caret.clone()),
            composition: editable.and_then(|editable| editable.composition.clone()),
            decorations: self
                .text_layout
                .as_ref()
                .zip(editable)
                .map(|(layout, editable)| editable_text_decorations(layout, editable))
                .unwrap_or_default(),
            runs,
        }
    }
}

fn stable_hash64(bytes: &[u8]) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    bytes.iter().fold(FNV_OFFSET, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(FNV_PRIME)
    })
}

fn text_font_resource_key(style: &UiResolvedStyle) -> UiRenderResourceKey {
    UiRenderResourceKey::new(UiRenderResourceKind::Font, text_font_resource_id(style))
}

fn text_font_resource_id(style: &UiResolvedStyle) -> String {
    style
        .font
        .as_deref()
        .or(style.font_family.as_deref())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("default")
        .to_string()
}

fn text_atlas_resource_key(
    style: &UiResolvedStyle,
    font_key: &UiRenderResourceKey,
) -> UiRenderResourceKey {
    UiRenderResourceKey::new(
        UiRenderResourceKind::Texture,
        format!(
            "font-atlas:{}:{:.1}:{:?}",
            font_key.id,
            style.font_size.max(1.0),
            style.text_render_mode
        ),
    )
}

fn editable_text_decorations(
    layout: &UiResolvedTextLayout,
    editable: &UiEditableTextState,
) -> Vec<UiTextPaintDecoration> {
    let mut decorations = Vec::new();
    if let Some(selection) = editable.selection.as_ref() {
        let range = selection.range();
        if range.start < range.end {
            for frame in text_range_frames(layout, range, TextDecorationMetric::Selection) {
                decorations.push(UiTextPaintDecoration::selection(
                    range,
                    frame,
                    TEXT_SELECTION_COLOR,
                ));
            }
        }
    }

    if let Some(composition) = editable.composition.as_ref() {
        for frame in text_range_frames(
            layout,
            composition.range,
            TextDecorationMetric::CompositionUnderline,
        ) {
            decorations.push(UiTextPaintDecoration::composition_underline(
                composition.range,
                frame,
                TEXT_COMPOSITION_UNDERLINE_COLOR,
            ));
        }
    }

    if let Some(frame) = caret_frame(layout, editable.caret.offset) {
        decorations.push(UiTextPaintDecoration {
            kind: UiTextPaintDecorationKind::Caret,
            range: UiTextRange {
                start: editable.caret.offset,
                end: editable.caret.offset,
            },
            frame,
            color: TEXT_CARET_COLOR.to_string(),
        });
    }
    decorations
}

#[derive(Clone, Copy)]
enum TextDecorationMetric {
    Selection,
    CompositionUnderline,
}

fn text_range_frames(
    layout: &UiResolvedTextLayout,
    range: UiTextRange,
    metric: TextDecorationMetric,
) -> Vec<UiFrame> {
    let mut frames = Vec::new();
    for line in &layout.lines {
        for run in &line.runs {
            let start = range.start.max(run.source_range.start);
            let end = range.end.min(run.source_range.end);
            if start >= end {
                continue;
            }
            let visual_start =
                run.visual_range.start + start.saturating_sub(run.source_range.start);
            let visual_end = run.visual_range.start + end.saturating_sub(run.source_range.start);
            let visual_start = grapheme_floor(line.text.as_str(), visual_start);
            let visual_end = grapheme_ceil(line.text.as_str(), visual_end);
            let x0 = visual_x(line, visual_start);
            let x1 = visual_x(line, visual_end);
            let (y, height) = match metric {
                TextDecorationMetric::Selection => (line.frame.y, line.frame.height),
                TextDecorationMetric::CompositionUnderline => (
                    line.frame.bottom() - TEXT_COMPOSITION_UNDERLINE_HEIGHT,
                    TEXT_COMPOSITION_UNDERLINE_HEIGHT,
                ),
            };
            frames.push(UiFrame::new(
                x0.min(x1),
                y,
                (x1 - x0).abs().max(TEXT_CARET_WIDTH),
                height,
            ));
        }
    }
    frames
}

fn caret_frame(layout: &UiResolvedTextLayout, offset: usize) -> Option<UiFrame> {
    let line = layout
        .lines
        .iter()
        .find(|line| offset >= line.source_range.start && offset <= line.source_range.end)
        .or_else(|| layout.lines.last())?;
    let visual_offset = line
        .runs
        .iter()
        .find_map(|run| {
            (offset >= run.source_range.start && offset <= run.source_range.end)
                .then(|| run.visual_range.start + offset.saturating_sub(run.source_range.start))
        })
        .unwrap_or(line.visual_range.end);
    Some(UiFrame::new(
        visual_x(line, visual_offset),
        line.frame.y,
        TEXT_CARET_WIDTH,
        line.frame.height,
    ))
}

fn visual_x(line: &UiResolvedTextLine, visual_offset: usize) -> f32 {
    let text = line.text.as_str();
    let offset = grapheme_floor(text, visual_offset.min(text.len()));
    let total_units = text.graphemes(true).count().max(1) as f32;
    let before_units = text[..offset].graphemes(true).count() as f32;
    line.frame.x + (line.frame.width.max(0.0) * before_units / total_units)
}

fn grapheme_floor(text: &str, offset: usize) -> usize {
    let mut offset = offset.min(text.len());
    while offset > 0 && !text.is_char_boundary(offset) {
        offset -= 1;
    }
    for (start, grapheme) in text.grapheme_indices(true) {
        let end = start + grapheme.len();
        if start < offset && offset < end {
            return start;
        }
        if start >= offset {
            break;
        }
    }
    offset
}

fn grapheme_ceil(text: &str, offset: usize) -> usize {
    let mut offset = offset.min(text.len());
    while offset < text.len() && !text.is_char_boundary(offset) {
        offset += 1;
    }
    for (start, grapheme) in text.grapheme_indices(true) {
        let end = start + grapheme.len();
        if start < offset && offset < end {
            return end;
        }
        if start >= offset {
            break;
        }
    }
    offset
}

fn render_clip_frame(frame: UiFrame, metrics: UiLayoutMetrics) -> UiFrame {
    if metrics.pixel_snapping == UiPixelSnapping::Enabled {
        frame.pixel_snapped(metrics.dpi_scale)
    } else {
        frame
    }
}

fn image_brush_payload(
    resource: UiRenderResourceKey,
    frame: UiFrame,
    metrics: UiLayoutMetrics,
) -> UiBrushPayload {
    let (width, height) = resource_pixel_size(frame, metrics);
    UiBrushPayload::image(resource).with_image_size(width, height)
}

fn resource_pixel_size(frame: UiFrame, metrics: UiLayoutMetrics) -> (f32, f32) {
    let dpi_scale = sanitized_resource_scale(metrics.dpi_scale);
    let render_bounds = UiGeometry::from_frame_with_metrics(frame, metrics).render_bounds;
    (
        (render_bounds.width.max(0.0) * dpi_scale).ceil(),
        (render_bounds.height.max(0.0) * dpi_scale).ceil(),
    )
}

fn sanitized_resource_scale(scale: f32) -> f32 {
    if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    }
}

fn image_resource_key(image: &UiVisualAssetRef) -> UiRenderResourceKey {
    match image {
        UiVisualAssetRef::Icon(icon) => {
            UiRenderResourceKey::new(UiRenderResourceKind::Icon, icon.clone())
        }
        UiVisualAssetRef::Image(image) => {
            UiRenderResourceKey::new(UiRenderResourceKind::Image, image.clone())
        }
    }
}
