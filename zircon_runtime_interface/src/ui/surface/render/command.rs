use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;
use crate::ui::layout::{UiFrame, UiGeometry};

use super::{
    UiBrushPayload, UiBrushSet, UiClipMode, UiClipState, UiPaintElement, UiPaintEffects,
    UiPaintPayload, UiRenderCommandKind, UiRenderResourceKey, UiRenderResourceKind,
    UiResolvedStyle, UiResolvedTextLayout, UiTextPaint, UiVisualAssetRef,
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

impl UiRenderCommand {
    pub fn to_paint_element(&self, paint_order: u64) -> UiPaintElement {
        self.base_paint_element(paint_order, self.paint_payload())
    }

    pub fn to_paint_elements(&self, first_paint_order: u64) -> Vec<UiPaintElement> {
        let mut elements = Vec::new();
        if let Some(payload) = self.brush_payload() {
            elements.push(self.base_paint_element(first_paint_order, payload));
        }
        if let Some(payload) = self.text_payload() {
            elements.push(self.base_paint_element(first_paint_order + elements.len() as u64, payload));
        }
        if elements.is_empty() {
            elements.push(self.base_paint_element(first_paint_order, UiPaintPayload::Empty));
        }
        elements
    }

    fn base_paint_element(&self, paint_order: u64, payload: UiPaintPayload) -> UiPaintElement {
        UiPaintElement {
            node_id: self.node_id,
            geometry: UiGeometry {
                clip_frame: self.clip_frame,
                ..UiGeometry::from_frame(self.frame)
            },
            clip: self.clip_frame.map(|frame| UiClipState {
                mode: UiClipMode::Scissor,
                frame,
            }),
            z_index: self.z_index,
            paint_order,
            payload,
            effects: UiPaintEffects {
                opacity: self.opacity.clamp(0.0, 1.0),
                effects: Vec::new(),
            },
            cache_generation: None,
            debug_label: Some(format!("{:?}", self.kind)),
        }
    }

    fn paint_payload(&self) -> UiPaintPayload {
        if self.text.as_ref().is_some_and(|text| !text.is_empty())
            || matches!(self.kind, UiRenderCommandKind::Text)
        {
            return self.text_payload().unwrap_or(UiPaintPayload::Empty);
        }

        self.brush_payload().unwrap_or(UiPaintPayload::Empty)
    }

    fn brush_payload(&self) -> Option<UiPaintPayload> {
        let brushes = self.brush_set();
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

    fn brush_set(&self) -> UiBrushSet {
        let fill = if let Some(image) = self.image.as_ref() {
            Some(UiBrushPayload::image(image_resource_key(image)))
        } else if let Some(color) = self.style.background_color.as_ref() {
            Some(if self.style.corner_radius > 0.0 {
                UiBrushPayload::rounded(color.clone(), self.style.corner_radius)
            } else {
                UiBrushPayload::solid(color.clone())
            })
        } else if matches!(self.kind, UiRenderCommandKind::Image) {
            Some(UiBrushPayload::image(UiRenderResourceKey::new(
                UiRenderResourceKind::Image,
                "missing:image",
            )))
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
            super::UiShapedText::from_resolved_layout(
                source_text.clone(),
                layout,
                self.style.text_render_mode,
            )
        });

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
        }
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
