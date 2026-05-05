use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;

use super::{
    UiBrushPayload, UiPaintElement, UiPaintPayload, UiRenderResourceKey, UiTextRenderMode,
};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiBatchPlan {
    pub batches: Vec<UiBatch>,
    pub stats: UiBatchStats,
}

impl UiBatchPlan {
    pub fn from_paint_elements(elements: &[UiPaintElement]) -> Self {
        let mut batches = Vec::new();
        let mut active_key: Option<UiBatchKey> = None;
        let mut active_start = 0usize;
        let mut active_node_ids: Vec<UiNodeId> = Vec::new();
        let mut active_split_reason = UiBatchSplitReason::FirstBatch;

        for (index, element) in elements.iter().enumerate() {
            let key = UiBatchKey::from_paint_element(element);
            if let Some(current_key) = active_key.as_ref() {
                if current_key == &key {
                    active_node_ids.push(element.node_id);
                    continue;
                }

                batches.push(UiBatch {
                    key: current_key.clone(),
                    range: UiBatchRange {
                        first_element: active_start,
                        element_count: index - active_start,
                    },
                    node_ids: std::mem::take(&mut active_node_ids),
                    split_reason: active_split_reason,
                });
                active_split_reason = current_key.split_reason(&key);
                active_start = index;
                active_key = Some(key);
                active_node_ids.push(element.node_id);
            } else {
                active_key = Some(key);
                active_node_ids.push(element.node_id);
            }
        }

        if let Some(key) = active_key {
            batches.push(UiBatch {
                key,
                range: UiBatchRange {
                    first_element: active_start,
                    element_count: elements.len() - active_start,
                },
                node_ids: active_node_ids,
                split_reason: active_split_reason,
            });
        }

        let stats = UiBatchStats {
            element_count: elements.len(),
            batch_count: batches.len(),
            draw_call_count: batches.len(),
        };
        Self { batches, stats }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiBatch {
    pub key: UiBatchKey,
    pub range: UiBatchRange,
    pub node_ids: Vec<UiNodeId>,
    pub split_reason: UiBatchSplitReason,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiBatchRange {
    pub first_element: usize,
    pub element_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiBatchKey {
    pub z_index: i32,
    pub clip: Option<String>,
    pub primitive: UiBatchPrimitive,
    pub shader: UiBatchShader,
    pub resource: Option<UiRenderResourceKey>,
    pub text_backend: Option<UiTextRenderMode>,
    pub opacity_class: UiOpacityClass,
}

impl UiBatchKey {
    pub fn from_paint_element(element: &UiPaintElement) -> Self {
        let (primitive, shader, resource, text_backend) = match &element.payload {
            UiPaintPayload::Empty => (UiBatchPrimitive::Empty, UiBatchShader::None, None, None),
            UiPaintPayload::Text { text } => (
                UiBatchPrimitive::Text,
                UiBatchShader::Text,
                None,
                Some(text.render_mode),
            ),
            UiPaintPayload::Brush { brushes } => brush_batch_key(
                brushes
                    .fill
                    .as_ref()
                    .or(brushes.border.as_ref()),
            ),
        };

        Self {
            z_index: element.z_index,
            clip: element.clip.as_ref().map(|clip| {
                format!(
                    "{:?}:{:.2}:{:.2}:{:.2}:{:.2}",
                    clip.mode, clip.frame.x, clip.frame.y, clip.frame.width, clip.frame.height
                )
            }),
            primitive,
            shader,
            resource,
            text_backend,
            opacity_class: UiOpacityClass::from_opacity(element.effects.opacity),
        }
    }

    fn split_reason(&self, next: &Self) -> UiBatchSplitReason {
        if self.z_index != next.z_index {
            UiBatchSplitReason::LayerChanged
        } else if self.clip != next.clip {
            UiBatchSplitReason::ClipChanged
        } else if self.primitive != next.primitive {
            UiBatchSplitReason::PrimitiveChanged
        } else if self.shader != next.shader {
            UiBatchSplitReason::ShaderChanged
        } else if self.resource != next.resource {
            UiBatchSplitReason::ResourceChanged
        } else if self.text_backend != next.text_backend {
            UiBatchSplitReason::TextBackendChanged
        } else if self.opacity_class != next.opacity_class {
            UiBatchSplitReason::OpacityChanged
        } else {
            UiBatchSplitReason::Merged
        }
    }
}

fn brush_batch_key(
    brush: Option<&UiBrushPayload>,
) -> (
    UiBatchPrimitive,
    UiBatchShader,
    Option<UiRenderResourceKey>,
    Option<UiTextRenderMode>,
) {
    match brush {
        Some(UiBrushPayload::Solid(_)) => (UiBatchPrimitive::Quad, UiBatchShader::Color, None, None),
        Some(UiBrushPayload::Rounded(_)) => {
            (UiBatchPrimitive::RoundedRect, UiBatchShader::Color, None, None)
        }
        Some(UiBrushPayload::Border(_)) => {
            (UiBatchPrimitive::Border, UiBatchShader::Color, None, None)
        }
        Some(UiBrushPayload::Image(payload)) | Some(UiBrushPayload::Box(payload)) => (
            UiBatchPrimitive::Image,
            UiBatchShader::Image,
            Some(payload.resource.clone()),
            None,
        ),
        Some(UiBrushPayload::Vector(payload)) => (
            UiBatchPrimitive::Vector,
            UiBatchShader::Vector,
            Some(payload.resource.clone()),
            None,
        ),
        Some(UiBrushPayload::Material(payload)) => (
            UiBatchPrimitive::Material,
            UiBatchShader::Material,
            Some(payload.resource_key()),
            None,
        ),
        Some(UiBrushPayload::Gradient(_)) => {
            (UiBatchPrimitive::Gradient, UiBatchShader::Gradient, None, None)
        }
        None => (UiBatchPrimitive::Empty, UiBatchShader::None, None, None),
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiBatchPrimitive {
    #[default]
    Empty,
    Quad,
    RoundedRect,
    Border,
    Image,
    Text,
    Vector,
    Gradient,
    Material,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiBatchShader {
    #[default]
    None,
    Color,
    Image,
    Text,
    Vector,
    Gradient,
    Material,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiBatchSplitReason {
    #[default]
    FirstBatch,
    Merged,
    LayerChanged,
    ClipChanged,
    PrimitiveChanged,
    ShaderChanged,
    ResourceChanged,
    TextBackendChanged,
    OpacityChanged,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiOpacityClass {
    #[default]
    Opaque,
    Translucent,
    Hidden,
}

impl UiOpacityClass {
    fn from_opacity(opacity: f32) -> Self {
        if opacity <= 0.0 {
            Self::Hidden
        } else if opacity >= 1.0 {
            Self::Opaque
        } else {
            Self::Translucent
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiBatchStats {
    pub element_count: usize,
    pub batch_count: usize,
    pub draw_call_count: usize,
}
