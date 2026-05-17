use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    ProjectionMode, RenderFrameExtract, RenderFramework, RenderPipelineHandle,
    RenderQualityProfile, RenderViewportDescriptor, RenderWorldSnapshotHandle,
};
use crate::core::math::UVec2;
use crate::graphics::WgpuRenderFramework;
use crate::{RenderPassStage, RenderPipelineAsset};
use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiTreeId};
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiRenderCommand, UiRenderCommandKind, UiRenderExtract, UiRenderList, UiResolvedStyle,
    UiTextAlign, UiTextRenderMode, UiTextWrap, UiVisualAssetRef,
};

#[test]
fn render_product_ui_compiles_after_postprocess_before_overlay_for_core2d_and_core3d() {
    let core2d = RenderPipelineAsset::default_core2d()
        .compile(&orthographic_extract())
        .unwrap();
    assert_ui_after_postprocess_before_overlay(&core2d.pass_stages);

    let forward = RenderPipelineAsset::default_forward_plus()
        .compile(&perspective_extract())
        .unwrap();
    assert_ui_after_postprocess_before_overlay(&forward.pass_stages);

    let deferred = RenderPipelineAsset::default_deferred()
        .compile(&perspective_extract())
        .unwrap();
    assert_ui_after_postprocess_before_overlay(&deferred.pass_stages);
}

#[test]
fn render_product_ui_submit_records_graph_pass_order_and_payload_stats() {
    let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("runtime-ui-product")
                .with_pipeline_asset(RenderPipelineHandle::new(1))
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_history_resolve(false)
                .with_bloom(false)
                .with_color_grading(false),
        )
        .unwrap();

    framework
        .submit_frame_extract_with_ui(
            viewport,
            perspective_extract(),
            Some(runtime_ui_extract_with_image_and_clip()),
        )
        .unwrap();
    let stats = framework.query_stats().unwrap();

    assert_eq!(stats.last_ui_command_count, 2);
    assert_eq!(stats.last_ui_quad_count, 1);
    assert_eq!(stats.last_ui_text_payload_count, 1);
    assert_eq!(stats.last_ui_image_payload_count, 1);
    assert_eq!(stats.last_ui_clipped_command_count, 1);
    assert_eq!(stats.last_ui_graph_executed_pass_count, 1);
    assert_eq!(stats.last_ui_target_size, Some(UVec2::new(320, 240)));
    assert_eq!(
        stats.last_ui_graph_pass_order.as_deref(),
        Some("postprocess-ui-overlay"),
        "executed passes: {:?}; executor ids: {:?}",
        stats.last_graph_executed_passes,
        stats.last_graph_executed_executor_ids
    );

    let post = stats
        .last_graph_executed_passes
        .iter()
        .position(|pass| pass == "post-process")
        .expect("postprocess pass should stay before runtime UI");
    let ui = stats
        .last_graph_executed_passes
        .iter()
        .position(|pass| pass == "runtime-ui")
        .expect("runtime UI pass should be graph-executed");
    let overlay = stats
        .last_graph_executed_passes
        .iter()
        .position(|pass| pass == "overlay-gizmo")
        .expect("overlay pass should stay after runtime UI");

    assert!(post < ui && ui < overlay);
    assert!(stats
        .last_graph_executed_executor_ids
        .contains(&"ui.screen-space".to_string()));
}

fn assert_ui_after_postprocess_before_overlay(
    pass_stages: &[crate::graphics::CompiledRenderPipelinePassStage],
) {
    let postprocess = pass_stages
        .iter()
        .position(|entry| entry.stage == RenderPassStage::PostProcess)
        .expect("pipeline should compile at least one postprocess graph pass");
    let ui = pass_stages
        .iter()
        .position(|entry| entry.stage == RenderPassStage::Ui)
        .expect("pipeline should compile a runtime UI graph pass");
    let overlay = pass_stages
        .iter()
        .position(|entry| entry.stage == RenderPassStage::Debug)
        .expect("pipeline should compile an overlay/debug graph pass");

    assert!(postprocess < ui && ui < overlay);
}

fn perspective_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(700),
        super::render_product_submit::snapshot_with_projection_for_sprite_tests(
            ProjectionMode::Perspective,
        ),
    )
}

fn orthographic_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(701),
        super::render_product_submit::snapshot_with_projection_for_sprite_tests(
            ProjectionMode::Orthographic,
        ),
    )
}

fn runtime_ui_extract_with_image_and_clip() -> UiRenderExtract {
    UiRenderExtract {
        tree_id: UiTreeId::new("runtime.ui.product"),
        list: UiRenderList {
            commands: vec![
                UiRenderCommand {
                    node_id: UiNodeId::new(1),
                    kind: UiRenderCommandKind::Quad,
                    frame: UiFrame::new(8.0, 8.0, 180.0, 28.0),
                    clip_frame: None,
                    z_index: 0,
                    style: UiResolvedStyle {
                        background_color: Some("#1b2330cc".to_string()),
                        foreground_color: Some("#f5f7fb".to_string()),
                        font_size: 14.0,
                        line_height: 18.0,
                        text_align: UiTextAlign::Center,
                        wrap: UiTextWrap::None,
                        text_render_mode: UiTextRenderMode::Auto,
                        ..UiResolvedStyle::default()
                    },
                    text_layout: None,
                    text: Some("Runtime HUD".to_string()),
                    image: None,
                    opacity: 1.0,
                },
                UiRenderCommand {
                    node_id: UiNodeId::new(2),
                    kind: UiRenderCommandKind::Image,
                    frame: UiFrame::new(20.0, 48.0, 32.0, 32.0),
                    clip_frame: Some(UiFrame::new(16.0, 44.0, 40.0, 40.0)),
                    z_index: 1,
                    style: UiResolvedStyle::default(),
                    text_layout: None,
                    text: None,
                    image: Some(UiVisualAssetRef::Image(
                        "res://ui/runtime/hud-icon.png".to_string(),
                    )),
                    opacity: 1.0,
                },
            ],
        },
    }
}
