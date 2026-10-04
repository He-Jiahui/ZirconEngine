use super::*;

#[test]
fn text_effect_contract_normalizes_non_finite_and_out_of_range_values() {
    let effects = UiTextDistanceFieldEffects {
        outline: Some(UiTextOutlineEffect {
            width_px: f32::INFINITY,
            color: String::new(),
        }),
        shadow: Some(UiTextShadowEffect {
            offset_x_px: -128.0,
            offset_y_px: f32::NAN,
            color: "  #11223344  ".to_string(),
        }),
        glow: Some(UiTextGlowEffect {
            radius_px: 96.0,
            color: String::new(),
        }),
    }
    .normalized();

    assert!(effects.outline.is_none());
    assert_eq!(
        effects.shadow,
        Some(UiTextShadowEffect {
            offset_x_px: -MAX_TEXT_EFFECT_EXTENT_PX,
            offset_y_px: 0.0,
            color: "#11223344".to_string(),
        })
    );
    assert_eq!(
        effects.glow,
        Some(UiTextGlowEffect {
            radius_px: MAX_TEXT_EFFECT_EXTENT_PX,
            color: "#ffffffff".to_string(),
        })
    );
}

#[test]
fn text_effect_contract_only_requests_true_distance_for_active_glow() {
    let outline = UiTextDistanceFieldEffects {
        outline: Some(UiTextOutlineEffect {
            width_px: 2.0,
            color: "#123456".to_string(),
        }),
        ..Default::default()
    };
    assert!(outline.requires_distance_field());
    assert!(!outline.requires_true_distance());

    let glow = UiTextDistanceFieldEffects {
        glow: Some(UiTextGlowEffect {
            radius_px: 3.0,
            color: "#ffffff".to_string(),
        }),
        ..Default::default()
    };
    assert!(glow.requires_true_distance());

    let transparent = UiTextDistanceFieldEffects {
        outline: Some(UiTextOutlineEffect {
            width_px: 2.0,
            color: "#00000000".to_string(),
        }),
        ..Default::default()
    };
    assert!(!transparent.requires_distance_field());
    assert!(transparent.normalized().outline.is_none());
}
