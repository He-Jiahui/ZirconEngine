use serde::{Deserialize, Serialize};

/// Maximum logical-pixel extent accepted by the public text-effect contract.
///
/// The renderer applies a second glyph-distance-range clamp because a font bake
/// can expose less padding than this authoring limit.
pub const MAX_TEXT_EFFECT_EXTENT_PX: f32 = 64.0;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiTextDistanceFieldEffects {
    pub outline: Option<UiTextOutlineEffect>,
    pub shadow: Option<UiTextShadowEffect>,
    pub glow: Option<UiTextGlowEffect>,
}

impl UiTextDistanceFieldEffects {
    pub fn normalized(&self) -> Self {
        Self {
            outline: self.outline.as_ref().and_then(normalized_active_outline),
            shadow: self.shadow.as_ref().and_then(normalized_active_shadow),
            glow: self.glow.as_ref().and_then(normalized_active_glow),
        }
    }

    pub fn requires_distance_field(&self) -> bool {
        self.outline
            .as_ref()
            .is_some_and(UiTextOutlineEffect::is_active)
            || self
                .shadow
                .as_ref()
                .is_some_and(UiTextShadowEffect::is_active)
            || self.glow.as_ref().is_some_and(UiTextGlowEffect::is_active)
    }

    /// 当前只有有效 Glow 要求 true-distance 表示；有效描边和阴影只要求距离场。
    pub fn requires_true_distance(&self) -> bool {
        self.glow.as_ref().is_some_and(UiTextGlowEffect::is_active)
    }
}

fn normalized_active_outline(value: &UiTextOutlineEffect) -> Option<UiTextOutlineEffect> {
    if !value.width_px.is_finite() || value.width_px <= 0.0 {
        return None;
    }
    if !value.color.trim().is_empty() && !color_has_visible_alpha(&value.color) {
        return None;
    }
    Some(value.normalized())
}

fn normalized_active_shadow(value: &UiTextShadowEffect) -> Option<UiTextShadowEffect> {
    let offset_x_px = normalized_signed_extent(value.offset_x_px);
    let offset_y_px = normalized_signed_extent(value.offset_y_px);
    if offset_x_px.abs() <= f32::EPSILON && offset_y_px.abs() <= f32::EPSILON {
        return None;
    }
    if !value.color.trim().is_empty() && !color_has_visible_alpha(&value.color) {
        return None;
    }
    Some(value.normalized())
}

fn normalized_active_glow(value: &UiTextGlowEffect) -> Option<UiTextGlowEffect> {
    if !value.radius_px.is_finite() || value.radius_px <= 0.0 {
        return None;
    }
    if !value.color.trim().is_empty() && !color_has_visible_alpha(&value.color) {
        return None;
    }
    Some(value.normalized())
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiTextOutlineEffect {
    pub width_px: f32,
    pub color: String,
}

impl UiTextOutlineEffect {
    pub fn normalized(&self) -> Self {
        Self {
            width_px: normalized_extent(self.width_px),
            color: normalized_color(&self.color, Self::DEFAULT_COLOR),
        }
    }

    pub fn is_active(&self) -> bool {
        self.width_px.is_finite() && self.width_px > 0.0 && color_has_visible_alpha(&self.color)
    }

    const DEFAULT_COLOR: &'static str = "#000000ff";
}

impl Default for UiTextOutlineEffect {
    fn default() -> Self {
        Self {
            width_px: 0.0,
            color: Self::DEFAULT_COLOR.to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiTextShadowEffect {
    pub offset_x_px: f32,
    pub offset_y_px: f32,
    pub color: String,
}

impl UiTextShadowEffect {
    pub fn normalized(&self) -> Self {
        Self {
            offset_x_px: normalized_signed_extent(self.offset_x_px),
            offset_y_px: normalized_signed_extent(self.offset_y_px),
            color: normalized_color(&self.color, Self::DEFAULT_COLOR),
        }
    }

    pub fn is_active(&self) -> bool {
        self.offset_x_px.is_finite()
            && self.offset_y_px.is_finite()
            && (self.offset_x_px.abs() > f32::EPSILON || self.offset_y_px.abs() > f32::EPSILON)
            && color_has_visible_alpha(&self.color)
    }

    const DEFAULT_COLOR: &'static str = "#00000080";
}

impl Default for UiTextShadowEffect {
    fn default() -> Self {
        Self {
            offset_x_px: 0.0,
            offset_y_px: 0.0,
            color: Self::DEFAULT_COLOR.to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiTextGlowEffect {
    pub radius_px: f32,
    pub color: String,
}

impl UiTextGlowEffect {
    pub fn normalized(&self) -> Self {
        Self {
            radius_px: normalized_extent(self.radius_px),
            color: normalized_color(&self.color, Self::DEFAULT_COLOR),
        }
    }

    pub fn is_active(&self) -> bool {
        self.radius_px.is_finite() && self.radius_px > 0.0 && color_has_visible_alpha(&self.color)
    }

    const DEFAULT_COLOR: &'static str = "#ffffffff";
}

impl Default for UiTextGlowEffect {
    fn default() -> Self {
        Self {
            radius_px: 0.0,
            color: Self::DEFAULT_COLOR.to_string(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiTextDecorations {
    pub underline: bool,
    pub strikethrough: bool,
    pub underline_color: Option<String>,
    pub strikethrough_color: Option<String>,
}

fn normalized_extent(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, MAX_TEXT_EFFECT_EXTENT_PX)
    } else {
        0.0
    }
}

fn normalized_signed_extent(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(-MAX_TEXT_EFFECT_EXTENT_PX, MAX_TEXT_EFFECT_EXTENT_PX)
    } else {
        0.0
    }
}

fn normalized_color(value: &str, fallback: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        fallback.to_string()
    } else {
        value.to_string()
    }
}

fn color_has_visible_alpha(value: &str) -> bool {
    let value = value.trim();
    let Some(hex) = value.strip_prefix('#') else {
        return !value.is_empty();
    };
    match hex.len() {
        6 => true,
        8 => u8::from_str_radix(&hex[6..8], 16).map_or(true, |alpha| alpha > 0),
        _ => true,
    }
}

#[cfg(test)]
#[path = "text_effects/tests/performance_tests.rs"]
mod performance_tests;

#[cfg(test)]
#[path = "tests/text_effects.rs"]
mod tests;
