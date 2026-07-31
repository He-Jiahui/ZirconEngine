use std::{path::PathBuf, sync::Arc};

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    RenderFrameProfile, RenderFramework, RenderViewportDescriptor,
};
use crate::core::math::UVec2;
use crate::graphics::debug_markers;

use super::fixture::{
    full_chain_material, full_chain_product_extract, insert_user_lut_texture,
    register_full_chain_material,
};
use super::{
    assert_terminal_signal_covers_frame, assert_terminal_signal_has_chromatic_content,
    assert_transient_texture_pool_aliases_logical_resources, create_full_chain_product_viewport,
    full_chain_product_framework,
};

#[test]
#[ignore = "writes Render17 cold/warm WGPU framebuffer evidence under docs/tests/runtime/render"]
fn export_render17_pfm1_render_graph_cold_warm_wgpu_png() {
    let viewport_size = UVec2::new(320, 240);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let receiver_material = register_full_chain_material(
        asset_manager.as_ref(),
        "res://materials/plan01_full_chain_receiver.zmaterial",
        full_chain_material(
            "Plan01FullChainReceiver",
            [0.03, 0.04, 0.055, 1.0],
            1.0,
            0.04,
            [0.0, 0.0, 0.0],
            false,
            true,
        ),
    );
    let caster_material = register_full_chain_material(
        asset_manager.as_ref(),
        "res://materials/plan01_full_chain_caster.zmaterial",
        full_chain_material(
            "Plan01FullChainCaster",
            [1.0, 0.18, 0.04, 1.0],
            0.0,
            0.22,
            [4.2, 0.32, 0.10],
            false,
            false,
        ),
    );
    let user_lut = insert_user_lut_texture(
        asset_manager.as_ref(),
        "res://textures/plan01_full_chain_lut.png",
    );
    let framework = full_chain_product_framework(asset_manager);
    let viewport = create_full_chain_product_viewport(
        &framework,
        viewport_size,
        "plan17-pfm1-render-graph-cold-warm",
        true,
    );
    let extract = full_chain_product_extract(
        viewport_size,
        receiver_material,
        caster_material,
        user_lut,
        true,
    );

    framework
        .submit_frame_extract(viewport, extract.clone())
        .unwrap();
    let first = framework.query_stats().unwrap();
    framework
        .submit_frame_extract(viewport, extract.clone())
        .unwrap();
    let history_ready = framework.query_stats().unwrap();

    // The first full-chain frame has no temporal history. The second frame
    // intentionally compiles the one history-enabled graph variant; the third
    // frame below must reuse that settled variant.
    assert_eq!(
        history_ready.last_graph_compiled_cache_miss_count,
        first.last_graph_compiled_cache_miss_count + 1,
        "the initial temporal-history transition must compile exactly one final graph variant"
    );
    assert!(
        history_ready.last_graph_compiled_cache_hit_count
            > first.last_graph_compiled_cache_hit_count,
        "the history transition must still reuse the unchanged base graph"
    );
    framework
        .request_graphics_debugger_capture(viewport)
        .unwrap();
    assert!(
        framework
            .query_graphics_debugger_status()
            .unwrap()
            .capture_pending
    );
    framework.submit_frame_extract(viewport, extract).unwrap();
    let warm = framework.query_stats().unwrap();
    let capture_status = framework.query_graphics_debugger_status().unwrap();
    assert!(!capture_status.capture_pending);
    assert_eq!(capture_status.last_capture_frame, warm.last_generation);
    let frame = framework
        .capture_frame(viewport)
        .unwrap()
        .expect("Render17 PF-M1 full-chain frame should be capturable");
    let graph_dump = frame
        .graph_dump
        .as_deref()
        .expect("Render17 capture must retain the compiled graph dump");
    let profile: RenderFrameProfile = serde_json::from_str(
        frame
            .frame_profile_json
            .as_deref()
            .expect("Render17 capture must retain the frame profile JSON"),
    )
    .expect("Render17 capture frame profile JSON must remain decodable");

    assert_eq!(
        warm.last_graph_compiled_cache_miss_count,
        history_ready.last_graph_compiled_cache_miss_count,
        "the settled warm frame must not compile another graph variant"
    );
    assert!(
        warm.last_graph_compiled_cache_hit_count
            > history_ready.last_graph_compiled_cache_hit_count,
        "the settled warm frame must reuse at least one compiled graph: history_ready_hits={}, warm_hits={}",
        history_ready.last_graph_compiled_cache_hit_count,
        warm.last_graph_compiled_cache_hit_count,
    );
    let pool = warm
        .last_graph_execution_resource_report
        .transient_pool_report;
    assert!(pool.texture_reused_count > 0 || pool.buffer_reused_count > 0);
    assert_transient_texture_pool_aliases_logical_resources(&warm);
    assert_eq!(
        warm.last_graph_executed_debug_markers.len(),
        warm.last_graph_executed_pass_count
    );
    assert_eq!(
        profile
            .passes
            .iter()
            .map(|pass| debug_markers::marker_for_render_graph_pass(&pass.pass_name))
            .collect::<Vec<_>>(),
        warm.last_graph_executed_debug_markers,
        "the capture profile pass names and emitted RenderDoc graph markers must stay aligned",
    );
    assert_eq!(profile.frame_generation, frame.generation);
    assert_eq!(profile.passes.len(), warm.last_graph_executed_pass_count);
    assert_eq!(
        profile
            .passes
            .iter()
            .map(|pass| pass.pass_name.as_str())
            .collect::<Vec<_>>(),
        warm.last_graph_executed_passes
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
    );
    assert!(profile
        .passes
        .iter()
        .all(|pass| graph_dump.contains(&pass.pass_name)));
    assert_terminal_signal_covers_frame(&frame);
    assert_terminal_signal_has_chromatic_content(
        &frame,
        None,
        Some(format!("warm={:?}", warm.last_exposure_readback_report)),
    );

    let output = repository_root()
        .join("docs")
        .join("tests")
        .join("runtime")
        .join("render")
        .join("plan17_pfm1_render_graph_cold_warm_wgpu_current_source_20260801.png");
    std::fs::create_dir_all(output.parent().unwrap()).unwrap();
    image::save_buffer_with_format(
        &output,
        &frame.rgba,
        frame.width,
        frame.height,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .unwrap();
    assert!(
        output.is_file(),
        "missing visual evidence: {}",
        output.display()
    );
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_runtime should live below the repository root")
        .to_path_buf()
}
