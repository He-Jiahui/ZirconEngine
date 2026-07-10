use std::{path::PathBuf, sync::Arc};

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{RenderFramework, RenderViewportDescriptor};
use crate::core::math::UVec2;

use super::fixture::{
    full_chain_material, full_chain_product_extract, insert_user_lut_texture,
    register_full_chain_material,
};
use super::{create_full_chain_product_viewport, full_chain_product_framework};

#[test]
#[ignore = "writes Plan 01 WGPU framebuffer evidence under docs/tests/runtime/render"]
fn export_render_graph_transient_cache_full_chain_wgpu_png() {
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
        "plan01-render-graph-transient-cache",
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
    framework.submit_frame_extract(viewport, extract).unwrap();
    let second = framework.query_stats().unwrap();
    let frame = framework
        .capture_frame(viewport)
        .unwrap()
        .expect("Plan 01 full-chain frame should be capturable");

    assert_eq!(
        second.last_graph_compiled_cache_miss_count,
        first.last_graph_compiled_cache_miss_count
    );
    assert_eq!(
        second.last_graph_compiled_cache_hit_count,
        first.last_graph_compiled_cache_hit_count + 1
    );
    let pool = second
        .last_graph_execution_resource_report
        .transient_pool_report;
    assert!(pool.texture_reused_count > 0 || pool.buffer_reused_count > 0);
    assert_eq!(
        second.last_graph_executed_debug_markers.len(),
        second.last_graph_executed_pass_count
    );
    assert!(frame
        .rgba
        .chunks_exact(4)
        .any(|pixel| { pixel[0] > 8 || pixel[1] > 8 || pixel[2] > 8 }));

    let output = repository_root()
        .join("docs")
        .join("tests")
        .join("runtime")
        .join("render")
        .join("plan01_render_graph_transient_cache_full_chain_wgpu_20260710.png");
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
