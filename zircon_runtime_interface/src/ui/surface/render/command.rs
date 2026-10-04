use std::io::{self, Write};

use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;
use crate::ui::layout::{UiFrame, UiGeometry, UiLayoutMetrics, UiPixelSnapping};

use super::text_geometry::editable_text_decorations;
use super::text_shape::text_paint_runs_from_resolved_layout;
use super::{
    UiBrushPayload, UiBrushSet, UiClipMode, UiClipState, UiPaintEffects, UiPaintElement,
    UiPaintPayload, UiRenderCommandKind, UiRenderResourceKey, UiRenderResourceKind,
    UiResolvedStyle, UiResolvedTextBox, UiResolvedTextLayout, UiTextPaint, UiTextPaintDecoration,
    UiTextShapeArtifact, UiVisualAssetRef,
};

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

#[derive(Clone, Copy)]
enum PaintElementMetadata {
    Cached { generation: u64 },
    Transient,
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
        let metrics = self.resolved_paint_metrics(metrics);
        self.base_paint_element(
            paint_order,
            self.paint_payload(metrics),
            metrics,
            PaintElementMetadata::Cached {
                generation: self.cache_generation(),
            },
        )
    }

    pub fn to_paint_elements(&self, first_paint_order: u64) -> Vec<UiPaintElement> {
        self.to_paint_elements_with_metrics(first_paint_order, UiLayoutMetrics::default())
    }

    pub fn to_paint_elements_with_metrics(
        &self,
        first_paint_order: u64,
        metrics: UiLayoutMetrics,
    ) -> Vec<UiPaintElement> {
        self.build_paint_elements_with_metrics(
            first_paint_order,
            metrics,
            PaintElementMetadata::Cached {
                generation: self.cache_generation(),
            },
        )
    }

    /// Builds immediate-consumption elements without cache or debug metadata.
    pub fn to_transient_paint_elements(&self, first_paint_order: u64) -> Vec<UiPaintElement> {
        self.to_transient_paint_elements_with_metrics(first_paint_order, UiLayoutMetrics::default())
    }

    /// Builds immediate-consumption elements without serializing a cache generation.
    pub fn to_transient_paint_elements_with_metrics(
        &self,
        first_paint_order: u64,
        metrics: UiLayoutMetrics,
    ) -> Vec<UiPaintElement> {
        self.build_paint_elements_with_metrics(
            first_paint_order,
            metrics,
            PaintElementMetadata::Transient,
        )
    }

    /// Fills caller-owned scratch for immediate-consumption paint planning.
    ///
    /// Render planners visit many commands per frame; reusing this buffer avoids a
    /// temporary vector allocation for every command while preserving the returned
    /// vector APIs used by retained/debug consumers.
    pub fn fill_transient_paint_elements(
        &self,
        first_paint_order: u64,
        metrics: UiLayoutMetrics,
        elements: &mut Vec<UiPaintElement>,
    ) {
        self.fill_paint_elements_with_metrics(
            first_paint_order,
            metrics,
            PaintElementMetadata::Transient,
            elements,
        );
    }

    /// Fills caller-owned scratch while preserving retained paint metadata.
    pub fn fill_paint_elements(
        &self,
        first_paint_order: u64,
        metrics: UiLayoutMetrics,
        elements: &mut Vec<UiPaintElement>,
    ) {
        self.fill_paint_elements_with_metrics(
            first_paint_order,
            metrics,
            PaintElementMetadata::Cached {
                generation: self.cache_generation(),
            },
            elements,
        );
    }

    pub(super) fn append_paint_elements(
        &self,
        first_paint_order: u64,
        metrics: UiLayoutMetrics,
        elements: &mut Vec<UiPaintElement>,
    ) {
        self.append_paint_elements_with_metrics(
            first_paint_order,
            metrics,
            PaintElementMetadata::Cached {
                generation: self.cache_generation(),
            },
            elements,
        );
    }

    fn build_paint_elements_with_metrics(
        &self,
        first_paint_order: u64,
        metrics: UiLayoutMetrics,
        metadata: PaintElementMetadata,
    ) -> Vec<UiPaintElement> {
        let mut elements = Vec::new();
        self.fill_paint_elements_with_metrics(first_paint_order, metrics, metadata, &mut elements);
        elements
    }

    fn fill_paint_elements_with_metrics(
        &self,
        first_paint_order: u64,
        metrics: UiLayoutMetrics,
        metadata: PaintElementMetadata,
        elements: &mut Vec<UiPaintElement>,
    ) {
        // fill 接口接管目标缓冲区内容；append 接口则保留旧元素并从当前长度续排。
        elements.clear();
        self.append_paint_elements_with_metrics(first_paint_order, metrics, metadata, elements);
    }

    fn append_paint_elements_with_metrics(
        &self,
        first_paint_order: u64,
        metrics: UiLayoutMetrics,
        metadata: PaintElementMetadata,
        elements: &mut Vec<UiPaintElement>,
    ) {
        let metrics = self.resolved_paint_metrics(metrics);
        let first_element_index = elements.len();
        if self.uses_image_brush() {
            // Image-bearing controls can still own background and border styling.
            // Emit separate paint elements so icon/vector content does not replace
            // the styled control chrome.
            for payload in [
                self.background_payload(metrics),
                self.image_payload(metrics),
                self.text_payload(),
                self.border_payload(),
            ]
            .into_iter()
            .flatten()
            {
                elements.push(self.base_paint_element(
                    first_paint_order + (elements.len() - first_element_index) as u64,
                    payload,
                    metrics,
                    metadata,
                ));
            }
        } else {
            if let Some(payload) = self.brush_payload(metrics) {
                elements.push(self.base_paint_element(
                    first_paint_order,
                    payload,
                    metrics,
                    metadata,
                ));
            }
            if let Some(payload) = self.text_payload() {
                elements.push(self.base_paint_element(
                    first_paint_order + (elements.len() - first_element_index) as u64,
                    payload,
                    metrics,
                    metadata,
                ));
            }
        }

        if elements.len() == first_element_index {
            elements.push(self.base_paint_element(
                first_paint_order,
                UiPaintPayload::Empty,
                metrics,
                metadata,
            ));
        }
    }

    fn base_paint_element(
        &self,
        paint_order: u64,
        payload: UiPaintPayload,
        metrics: UiLayoutMetrics,
        metadata: PaintElementMetadata,
    ) -> UiPaintElement {
        let (cache_generation, debug_label) = match metadata {
            PaintElementMetadata::Cached { generation } => (
                Some(generation),
                Some(render_command_debug_label(self.kind)),
            ),
            PaintElementMetadata::Transient => (None, None),
        };
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
            cache_generation,
            debug_label,
        }
    }

    fn resolved_paint_metrics(&self, mut metrics: UiLayoutMetrics) -> UiLayoutMetrics {
        metrics.pixel_snapping = self.style.pixel_snapping.resolve(metrics.pixel_snapping);
        metrics
    }

    pub fn cache_generation(&self) -> u64 {
        stable_json_generation(self)
    }

    #[cfg(test)]
    pub(crate) fn cache_generation_test_calls() -> usize {
        CACHE_GENERATION_TEST_CALLS.with(std::cell::Cell::get)
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

    fn background_payload(&self, metrics: UiLayoutMetrics) -> Option<UiPaintPayload> {
        self.background_brush(metrics)
            .map(|fill| UiPaintPayload::Brush {
                brushes: UiBrushSet {
                    fill: Some(fill),
                    border: None,
                },
            })
    }

    fn border_payload(&self) -> Option<UiPaintPayload> {
        self.border_brush().map(|border| UiPaintPayload::Brush {
            brushes: UiBrushSet {
                fill: None,
                border: Some(border),
            },
        })
    }

    fn image_payload(&self, metrics: UiLayoutMetrics) -> Option<UiPaintPayload> {
        self.image_brush(metrics).map(|fill| UiPaintPayload::Brush {
            brushes: UiBrushSet {
                fill: Some(fill),
                border: None,
            },
        })
    }

    fn text_payload(&self) -> Option<UiPaintPayload> {
        (self.text.as_ref().is_some_and(|text| !text.is_empty())
            || matches!(self.kind, UiRenderCommandKind::Text))
        .then(|| UiPaintPayload::Text {
            text: self.text_paint(),
        })
    }

    fn uses_image_brush(&self) -> bool {
        self.image.is_some() || matches!(self.kind, UiRenderCommandKind::Image)
    }

    fn brush_set(&self, metrics: UiLayoutMetrics) -> UiBrushSet {
        UiBrushSet {
            fill: self
                .image_brush(metrics)
                .or_else(|| self.background_brush(metrics)),
            border: self.border_brush(),
        }
    }

    fn background_brush(&self, _metrics: UiLayoutMetrics) -> Option<UiBrushPayload> {
        self.style.background_color.as_ref().map(|color| {
            if self.style.corner_radius > 0.0 {
                UiBrushPayload::rounded(color.clone(), self.style.corner_radius)
            } else {
                UiBrushPayload::solid(color.clone())
            }
        })
    }

    fn border_brush(&self) -> Option<UiBrushPayload> {
        self.style
            .border_color
            .as_ref()
            .filter(|_| self.style.border_width > 0.0)
            .map(|color| {
                let mut border = UiBrushPayload::border(color.clone(), self.style.border_width);
                if let UiBrushPayload::Border(payload) = &mut border {
                    payload.radius = self.style.corner_radius;
                }
                border
            })
    }

    fn image_brush(&self, metrics: UiLayoutMetrics) -> Option<UiBrushPayload> {
        if let Some(image) = self.image.as_ref() {
            Some(image_brush_payload(
                image_resource_key(image),
                self.frame,
                metrics,
            ))
        } else if matches!(self.kind, UiRenderCommandKind::Image) {
            Some(image_brush_payload(
                UiRenderResourceKey::new(UiRenderResourceKind::Image, "missing:image"),
                self.frame,
                metrics,
            ))
        } else {
            None
        }
    }

    fn text_paint(&self) -> UiTextPaint {
        let source_text = self.text.clone().unwrap_or_default();

        let editable = self
            .text_layout
            .as_ref()
            .and_then(|layout| layout.editable.as_ref());

        let runs = self
            .text_layout
            .as_ref()
            .map(|layout| {
                text_paint_runs_from_resolved_layout(
                    layout,
                    &self.style.foreground_color,
                    &self.style.font,
                    &self.style.font_family,
                    self.style.font_weight,
                    self.style.font_size,
                    self.style.line_height,
                )
            })
            .unwrap_or_default();

        let mut decorations = Vec::with_capacity(
            self.text_layout
                .as_ref()
                .map_or(0, |layout| layout.boxes.len().saturating_mul(2)),
        );
        if let Some(layout) = self.text_layout.as_ref() {
            append_text_box_background_decorations(&layout.boxes, &mut decorations);
        }
        if let Some((layout, editable)) = self.text_layout.as_ref().zip(editable) {
            decorations.extend(editable_text_decorations(layout, editable));
        }
        if let Some(layout) = self.text_layout.as_ref() {
            append_text_box_border_decorations(&layout.boxes, &mut decorations);
        }

        UiTextPaint {
            source_text,
            color: self.style.foreground_color.clone(),
            font: self.style.font.clone(),
            font_family: self.style.font_family.clone(),
            font_weight: self.style.font_weight,
            font_size: self.style.font_size,
            line_height: self.style.line_height,
            writing_mode: self.style.text_writing_mode,
            render_mode: self.style.text_render_mode,
            text_effects: self.style.text_effects.normalized(),
            text_decorations: self.style.text_decorations.clone(),
            overflow: self.style.text_overflow,
            // A resolved layout carries geometry only. Glyph/face authority remains in the
            // runtime-owned artifact and must never be inferred from presentation style.
            shaped: UiTextShapeArtifact::Unavailable,
            selection: editable.and_then(|editable| editable.selection.clone()),
            caret: editable.map(|editable| editable.caret.clone()),
            composition: editable.and_then(|editable| editable.composition.clone()),
            decorations,
            runs,
        }
    }
}

fn stable_json_generation<T>(value: &T) -> u64
where
    T: Serialize + ?Sized,
{
    #[cfg(test)]
    CACHE_GENERATION_TEST_CALLS.with(|calls| calls.set(calls.get() + 1));
    let mut writer = StableHashWriter::default();
    if serde_json::to_writer(&mut writer, value).is_err() {
        return FNV_OFFSET;
    }
    writer.finish()
}

#[cfg(test)]
thread_local! {
    static CACHE_GENERATION_TEST_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

fn append_text_box_background_decorations(
    boxes: &[UiResolvedTextBox],
    decorations: &mut Vec<UiTextPaintDecoration>,
) {
    decorations.extend(boxes.iter().filter_map(|text_box| {
        text_box.background_color.map(|color| {
            UiTextPaintDecoration::table_cell_background(
                text_box.range,
                text_box.frame,
                rgba_hex(color),
            )
        })
    }))
}

fn append_text_box_border_decorations(
    boxes: &[UiResolvedTextBox],
    decorations: &mut Vec<UiTextPaintDecoration>,
) {
    decorations.extend(boxes.iter().filter_map(|text_box| {
        text_box.border_color.map(|color| {
            UiTextPaintDecoration::table_cell_border(
                text_box.range,
                text_box.frame,
                rgba_hex(color),
                text_box.border_width,
            )
        })
    }))
}

fn render_command_debug_label(kind: UiRenderCommandKind) -> String {
    render_command_kind_name(kind).to_owned()
}

fn render_command_kind_name(kind: UiRenderCommandKind) -> &'static str {
    match kind {
        UiRenderCommandKind::Group => "Group",
        UiRenderCommandKind::Quad => "Quad",
        UiRenderCommandKind::Text => "Text",
        UiRenderCommandKind::Image => "Image",
    }
}

#[cfg(test)]
fn render_command_debug_label_formatting(kind: UiRenderCommandKind) -> String {
    format!("{kind:?}")
}

fn rgba_hex(color: crate::ui::style::UiRgbaColor) -> String {
    const RGBA_HEX_UPPER: &[u8; 16] = b"0123456789ABCDEF";

    let mut encoded = String::with_capacity(9);
    encoded.push('#');
    for channel in color.to_u8() {
        encoded.push(RGBA_HEX_UPPER[usize::from(channel >> 4)] as char);
        encoded.push(RGBA_HEX_UPPER[usize::from(channel & 0x0f)] as char);
    }
    encoded
}

const FNV_OFFSET: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

struct StableHashWriter {
    hash: u64,
}

impl Default for StableHashWriter {
    fn default() -> Self {
        Self { hash: FNV_OFFSET }
    }
}

impl StableHashWriter {
    fn finish(self) -> u64 {
        self.hash
    }
}

impl Write for StableHashWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.hash = bytes.iter().fold(self.hash, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(FNV_PRIME)
        });
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "command/tests/rgba_hex_performance_tests.rs"]
mod rgba_hex_performance_tests;

#[cfg(test)]
#[path = "command/tests/text_box_decoration_performance_tests.rs"]
mod text_box_decoration_performance_tests;

#[cfg(test)]
#[path = "command/tests/debug_label_performance_tests.rs"]
mod debug_label_performance_tests;

#[cfg(test)]
#[path = "command/tests/clip_tests.rs"]
mod clip_tests;

#[cfg(test)]
#[path = "tests/command_cache_generation_tests.rs"]
mod cache_generation_tests;

fn render_clip_frame(frame: UiFrame, metrics: UiLayoutMetrics) -> UiFrame {
    if frame.width <= 0.0 || frame.height <= 0.0 {
        return frame;
    }
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
