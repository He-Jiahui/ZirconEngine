use super::*;
use zircon_runtime_interface::ui::surface::{
    UiTextDistanceFieldEffects, UiTextGlowEffect, UiTextOutlineEffect,
};

#[test]
fn screen_space_ui_effects_force_small_native_text_onto_distance_field_batches() {
    let command = UiRenderCommand {
        node_id: UiNodeId::new(701),
        kind: UiRenderCommandKind::Text,
        frame: UiFrame::new(4.0, 4.0, 120.0, 24.0),
        clip_frame: None,
        z_index: 0,
        style: UiResolvedStyle {
            font_size: 12.0,
            line_height: 16.0,
            text_render_mode: UiTextRenderMode::Native,
            text_effects: UiTextDistanceFieldEffects {
                outline: Some(UiTextOutlineEffect {
                    width_px: 1.5,
                    color: "#000000ff".to_string(),
                }),
                ..Default::default()
            },
            ..UiResolvedStyle::default()
        },
        text_layout: None,
        text: Some("outlined".to_string()),
        image: None,
        opacity: 1.0,
    };

    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.effects"),
            list: UiRenderList {
                commands: vec![command],
            },
        },
        UVec2::new(160, 48),
    );

    assert!(plan.native_texts.is_empty());
    assert_eq!(plan.sdf_texts.len(), 1);
    assert_eq!(
        plan.sdf_texts[0].distance_field_mode,
        crate::graphics::text::sdf::SdfMode::Sdf
    );
    assert!(plan.sdf_texts[0].text_effects.outline.is_some());
}

#[test]
fn screen_space_ui_glow_selects_mtsdf_true_distance() {
    let command = UiRenderCommand {
        node_id: UiNodeId::new(702),
        kind: UiRenderCommandKind::Text,
        frame: UiFrame::new(4.0, 4.0, 120.0, 24.0),
        clip_frame: None,
        z_index: 0,
        style: UiResolvedStyle {
            font_size: 12.0,
            line_height: 16.0,
            text_render_mode: UiTextRenderMode::Msdf,
            text_effects: UiTextDistanceFieldEffects {
                glow: Some(UiTextGlowEffect {
                    radius_px: 4.0,
                    color: "#66ccffff".to_string(),
                }),
                ..Default::default()
            },
            ..UiResolvedStyle::default()
        },
        text_layout: None,
        text: Some("glow".to_string()),
        image: None,
        opacity: 1.0,
    };

    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.glow"),
            list: UiRenderList {
                commands: vec![command],
            },
        },
        UVec2::new(160, 48),
    );

    assert_eq!(plan.sdf_texts.len(), 1);
    assert_eq!(
        plan.sdf_texts[0].distance_field_mode,
        crate::graphics::text::sdf::SdfMode::Mtsdf
    );
}
