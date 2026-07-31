use serde::{Deserialize, Serialize};

use super::super::{
    UiBrushPayload, UiClipState, UiDrawEffect, UiPaintElement, UiPaintPayload, UiRenderResourceKey,
    UiTextRenderMode,
};
use super::clip::UiBatchClipStates;

/// Backend-neutral state that must match before adjacent paint elements merge.
///
/// Layering intentionally does not belong here. Extraction orders elements by
/// layer and paint order first, then this key decides whether adjacent elements
/// are safe to submit in one draw call.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiBatchKey {
    pub clip: Option<UiClipState>,
    pub primitive: UiBatchPrimitive,
    pub shader: UiBatchShader,
    pub resource: Option<UiRenderResourceKey>,
    pub text_backend: Option<UiTextRenderMode>,
    pub draw_effects: Vec<UiDrawEffect>,
    pub opacity_class: UiOpacityClass,
}

impl UiBatchKey {
    pub fn from_paint_element(element: &UiPaintElement) -> Self {
        let mut clip_states = UiBatchClipStates::default();
        Self::from_paint_element_with_clip_states(element, &mut clip_states)
    }

    pub(super) fn from_paint_element_with_clip_states(
        element: &UiPaintElement,
        clip_states: &mut UiBatchClipStates,
    ) -> Self {
        let (primitive, shader, resource, text_backend) = match &element.payload {
            UiPaintPayload::Empty => (UiBatchPrimitive::Empty, UiBatchShader::None, None, None),
            UiPaintPayload::Text { text } => (
                UiBatchPrimitive::Text,
                UiBatchShader::Text,
                None,
                Some(text.render_mode),
            ),
            UiPaintPayload::Brush { brushes } => {
                brush_batch_key(brushes.fill.as_ref().or(brushes.border.as_ref()))
            }
        };

        Self {
            clip: element.clip.clone().map(|clip| clip_states.intern(clip)),
            primitive,
            shader,
            resource,
            text_backend,
            draw_effects: element.effects.effects.clone(),
            opacity_class: UiOpacityClass::from_opacity(element.effects.opacity),
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
        Some(UiBrushPayload::Solid(_)) => {
            (UiBatchPrimitive::Quad, UiBatchShader::Color, None, None)
        }
        Some(UiBrushPayload::Rounded(_)) => (
            UiBatchPrimitive::RoundedRect,
            UiBatchShader::Color,
            None,
            None,
        ),
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
        Some(UiBrushPayload::Gradient(_)) => (
            UiBatchPrimitive::Gradient,
            UiBatchShader::Gradient,
            None,
            None,
        ),
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
pub enum UiOpacityClass {
    #[default]
    Opaque,
    Translucent,
    Hidden,
}

impl UiOpacityClass {
    pub(super) fn from_opacity(opacity: f32) -> Self {
        if opacity <= 0.0 {
            Self::Hidden
        } else if opacity >= 1.0 {
            Self::Opaque
        } else {
            Self::Translucent
        }
    }
}
