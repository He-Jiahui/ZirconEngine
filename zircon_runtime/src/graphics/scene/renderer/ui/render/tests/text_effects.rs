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
