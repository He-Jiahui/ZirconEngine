use zircon_runtime_interface::ui::surface::UiTextDistanceFieldEffects;

use super::color::parse_hex_color;

const DEFAULT_OUTLINE_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const DEFAULT_SHADOW_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 0.5];
const DEFAULT_GLOW_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(in crate::graphics::scene::scene_renderer::ui) struct ScreenSpaceUiTextEffects {
    pub(in crate::graphics::scene::scene_renderer::ui) outline: Option<ScreenSpaceUiTextOutline>,
    pub(in crate::graphics::scene::scene_renderer::ui) shadow: Option<ScreenSpaceUiTextShadow>,
    pub(in crate::graphics::scene::scene_renderer::ui) glow: Option<ScreenSpaceUiTextGlow>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::graphics::scene::scene_renderer::ui) struct ScreenSpaceUiTextOutline {
    pub(in crate::graphics::scene::scene_renderer::ui) width_px: f32,
    pub(in crate::graphics::scene::scene_renderer::ui) color: [f32; 4],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::graphics::scene::scene_renderer::ui) struct ScreenSpaceUiTextShadow {
    pub(in crate::graphics::scene::scene_renderer::ui) offset_px: [f32; 2],
    pub(in crate::graphics::scene::scene_renderer::ui) color: [f32; 4],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::graphics::scene::scene_renderer::ui) struct ScreenSpaceUiTextGlow {
    pub(in crate::graphics::scene::scene_renderer::ui) radius_px: f32,
    pub(in crate::graphics::scene::scene_renderer::ui) color: [f32; 4],
}

pub(super) fn resolve_text_effects(
    effects: &UiTextDistanceFieldEffects,
    opacity: f32,
) -> ScreenSpaceUiTextEffects {
    let effects = effects.normalized();
    ScreenSpaceUiTextEffects {
        outline: effects.outline.map(|effect| ScreenSpaceUiTextOutline {
            width_px: effect.width_px,
            color: resolved_effect_color(&effect.color, DEFAULT_OUTLINE_COLOR, opacity),
        }),
        shadow: effects.shadow.map(|effect| ScreenSpaceUiTextShadow {
            offset_px: [effect.offset_x_px, effect.offset_y_px],
            color: resolved_effect_color(&effect.color, DEFAULT_SHADOW_COLOR, opacity),
        }),
        glow: effects.glow.map(|effect| ScreenSpaceUiTextGlow {
            radius_px: effect.radius_px,
            color: resolved_effect_color(&effect.color, DEFAULT_GLOW_COLOR, opacity),
        }),
    }
}

fn resolved_effect_color(value: &str, fallback: [f32; 4], opacity: f32) -> [f32; 4] {
    parse_hex_color(value, opacity).unwrap_or([
        fallback[0],
        fallback[1],
        fallback[2],
        fallback[3] * opacity.clamp(0.0, 1.0),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime_interface::ui::surface::{UiTextOutlineEffect, UiTextShadowEffect};

    #[test]
    fn text_effect_material_projection_resolves_colors_and_opacity() {
        let resolved = resolve_text_effects(
            &UiTextDistanceFieldEffects {
                outline: Some(UiTextOutlineEffect {
                    width_px: 2.0,
                    color: "#ff000080".to_string(),
                }),
                shadow: Some(UiTextShadowEffect {
                    offset_x_px: 3.0,
                    offset_y_px: -2.0,
                    color: "invalid".to_string(),
                }),
                ..Default::default()
            },
            0.5,
        );

        assert_eq!(resolved.outline.unwrap().color[0..3], [1.0, 0.0, 0.0]);
        assert!((resolved.outline.unwrap().color[3] - (128.0 / 255.0) * 0.5).abs() < 0.0001);
        assert_eq!(resolved.shadow.unwrap().offset_px, [3.0, -2.0]);
        assert_eq!(resolved.shadow.unwrap().color, [0.0, 0.0, 0.0, 0.25]);
    }
}
