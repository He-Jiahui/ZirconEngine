use super::*;
use zircon_runtime_interface::ui::surface::UiVisualAssetRef;

fn icon_extract(icon: &str) -> UiRenderExtract {
    UiRenderExtract {
        tree_id: UiTreeId::new("runtime.ui.builtin-icon"),
        list: UiRenderList {
            commands: vec![UiRenderCommand {
                node_id: UiNodeId::new(41),
                kind: UiRenderCommandKind::Image,
                frame: UiFrame::new(12.0, 8.0, 24.0, 24.0),
                clip_frame: None,
                z_index: 0,
                style: UiResolvedStyle {
                    foreground_color: Some("#5da9ff".to_string()),
                    ..UiResolvedStyle::default()
                },
                text_layout: None,
                text: None,
                image: Some(UiVisualAssetRef::Icon(icon.to_string())),
                opacity: 1.0,
            }],
        },
        raster_scale: 1.0,
    }
}

#[test]
fn screen_space_ui_plan_paints_builtin_feedback_icon_geometry() {
    let plan = plan_screen_space_ui_batches(&icon_extract("info"), UVec2::new(64, 48));

    assert!(plan.images.is_empty());
    assert_eq!(plan.draws.len(), 1);
    assert!(
        plan.vertices.len() > 6,
        "the built-in icon must not collapse to the single generic fallback square"
    );
    assert!(plan.vertices.iter().all(|vertex| {
        vertex.position.iter().all(|value| value.is_finite())
            && vertex.local_position.iter().all(|value| value.is_finite())
    }));
}

#[test]
fn screen_space_ui_plan_keeps_unknown_icons_on_the_explicit_fallback_path() {
    let plan =
        plan_screen_space_ui_batches(&icon_extract("project-custom-icon"), UVec2::new(64, 48));

    assert!(plan.images.is_empty());
    assert_eq!(plan.draws.len(), 1);
    assert_eq!(plan.vertices.len(), 6);
}

#[test]
fn screen_space_ui_plan_honors_named_builtin_icon_size_tiers() {
    let compact = plan_screen_space_ui_batches(&icon_extract("info@s"), UVec2::new(64, 48));
    let focal = plan_screen_space_ui_batches(&icon_extract("info@xl"), UVec2::new(64, 48));

    let compact_extent = compact
        .vertices
        .iter()
        .map(|vertex| vertex.half_extent[0] * 2.0)
        .fold(0.0_f32, f32::max);
    let focal_extent = focal
        .vertices
        .iter()
        .map(|vertex| vertex.half_extent[0] * 2.0)
        .fold(0.0_f32, f32::max);
    assert!(focal_extent > compact_extent);
}
