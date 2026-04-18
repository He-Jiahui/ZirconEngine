use std::sync::Arc;

use zircon_asset::ProjectAssetManager;
use zircon_framework::render::{RenderFramework, RenderQualityProfile, RenderViewportDescriptor};
use zircon_math::UVec2;

use crate::{runtime::WgpuRenderFramework, RuntimeUiFixture, RuntimeUiManager};

#[test]
fn runtime_ui_manager_builds_all_builtin_fixtures_into_shared_surfaces() {
    let viewport_size = UVec2::new(1280, 720);
    let mut manager = RuntimeUiManager::new(viewport_size);

    for fixture in [
        RuntimeUiFixture::HudOverlay,
        RuntimeUiFixture::PauseMenu,
        RuntimeUiFixture::SettingsDialog,
        RuntimeUiFixture::InventoryList,
    ] {
        manager.load_builtin_fixture(fixture).unwrap();

        let surface = manager.surface();
        assert_eq!(surface.tree.roots.len(), 1);
        assert!(
            surface.render_extract.list.commands.len() >= 4,
            "expected fixture {fixture:?} to build a non-trivial shared visual tree"
        );
        assert_eq!(
            manager.build_frame().viewport_size,
            viewport_size,
            "runtime UI frame should preserve viewport size for {fixture:?}"
        );
        assert!(
            manager.build_frame().ui.is_some(),
            "runtime UI frame should carry a shared UI render extract for {fixture:?}"
        );
    }
}

#[test]
fn render_framework_submits_runtime_ui_frames_and_renders_pause_menu_panels() {
    let viewport_size = UVec2::new(640, 360);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("runtime-ui")
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_history_resolve(false),
        )
        .unwrap();

    let mut manager = RuntimeUiManager::new(viewport_size);
    manager
        .load_builtin_fixture(RuntimeUiFixture::PauseMenu)
        .unwrap();

    server
        .submit_runtime_frame(viewport, manager.build_frame())
        .unwrap();

    let stats = server.query_stats().unwrap();
    assert!(
        stats.last_ui_command_count >= 8,
        "expected runtime UI submission to report draw-list command stats"
    );
    assert!(
        stats.last_ui_quad_count >= 4,
        "expected pause menu fixture to contribute multiple quad-like UI draws"
    );

    let frame = server.capture_frame(viewport).unwrap().unwrap();
    assert!(
        count_non_background_pixels(&frame.rgba) > 4_096,
        "expected pause menu UI pass to contribute a visible screen-space footprint"
    );
    let center_red = average_region_channel(&frame.rgba, viewport_size, 0, 0.35, 0.25, 0.65, 0.75);
    let corner_red = average_region_channel(&frame.rgba, viewport_size, 0, 0.0, 0.0, 0.18, 0.18);
    assert!(
        center_red > corner_red + 24.0,
        "expected centered pause dialog to brighten the middle of the frame above the corner background; center_red={center_red:.2}, corner_red={corner_red:.2}"
    );
}

#[test]
fn render_framework_reports_clipped_ui_commands_for_inventory_fixture() {
    let viewport_size = UVec2::new(960, 540);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("runtime-ui-list")
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_history_resolve(false),
        )
        .unwrap();

    let mut manager = RuntimeUiManager::new(viewport_size);
    manager
        .load_builtin_fixture(RuntimeUiFixture::InventoryList)
        .unwrap();

    server
        .submit_runtime_frame(viewport, manager.build_frame())
        .unwrap();

    let stats = server.query_stats().unwrap();
    assert!(
        stats.last_ui_clipped_command_count >= 1,
        "expected inventory fixture to route at least one clipped UI command through the runtime UI pass"
    );
}

fn count_non_background_pixels(rgba: &[u8]) -> usize {
    rgba.chunks_exact(4)
        .filter(|pixel| pixel[0] > 8 || pixel[1] > 8 || pixel[2] > 8)
        .count()
}

fn average_region_channel(
    rgba: &[u8],
    viewport_size: UVec2,
    channel: usize,
    left_norm: f32,
    top_norm: f32,
    right_norm: f32,
    bottom_norm: f32,
) -> f32 {
    let width = viewport_size.x as usize;
    let height = viewport_size.y as usize;
    let left = ((width as f32) * left_norm).floor() as usize;
    let top = ((height as f32) * top_norm).floor() as usize;
    let right = ((width as f32) * right_norm).ceil() as usize;
    let bottom = ((height as f32) * bottom_norm).ceil() as usize;

    let mut total = 0.0;
    let mut count = 0usize;
    for y in top.min(height)..bottom.min(height) {
        for x in left.min(width)..right.min(width) {
            let pixel_index = (y * width + x) * 4;
            total += rgba[pixel_index + channel] as f32;
            count += 1;
        }
    }

    if count == 0 {
        0.0
    } else {
        total / count as f32
    }
}
