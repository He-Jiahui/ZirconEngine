#[cfg(feature = "runtime-ui-integration-tests")]
#[test]
fn render_framework_submits_runtime_ui_frames_and_renders_pause_menu_panels() {
    use std::sync::Arc;

    use crate::core::framework::render::{
        RenderFramework, RenderQualityProfile, RenderViewportDescriptor,
    };

    let viewport_size = crate::core::math::UVec2::new(640, 360);
    let asset_manager = Arc::new(crate::asset::ProjectAssetManager::default());
    let server = crate::graphics::WgpuRenderFramework::new(asset_manager).unwrap();
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

    let mut manager = crate::ui::RuntimeUiManager::new(viewport_size);
    manager
        .load_builtin_fixture(crate::ui::RuntimeUiFixture::PauseMenu)
        .unwrap();

    server
        .submit_runtime_frame(viewport, manager.build_frame().into())
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

#[cfg(feature = "runtime-ui-integration-tests")]
#[test]
fn render_framework_reports_clipped_ui_commands_for_inventory_fixture() {
    use std::sync::Arc;

    use crate::core::framework::render::{
        RenderFramework, RenderQualityProfile, RenderViewportDescriptor,
    };

    let viewport_size = crate::core::math::UVec2::new(960, 540);
    let asset_manager = Arc::new(crate::asset::ProjectAssetManager::default());
    let server = crate::graphics::WgpuRenderFramework::new(asset_manager).unwrap();
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

    let mut manager = crate::ui::RuntimeUiManager::new(viewport_size);
    manager
        .load_builtin_fixture(crate::ui::RuntimeUiFixture::InventoryList)
        .unwrap();

    server
        .submit_runtime_frame(viewport, manager.build_frame().into())
        .unwrap();

    let stats = server.query_stats().unwrap();
    assert!(
        stats.last_ui_clipped_command_count >= 1,
        "expected inventory fixture to route at least one clipped UI command through the runtime UI pass"
    );
}

#[cfg(feature = "runtime-ui-integration-tests")]
fn count_non_background_pixels(rgba: &[u8]) -> usize {
    rgba.chunks_exact(4)
        .filter(|pixel| pixel[0] > 8 || pixel[1] > 8 || pixel[2] > 8)
        .count()
}

#[cfg(feature = "runtime-ui-integration-tests")]
fn average_region_channel(
    rgba: &[u8],
    viewport_size: crate::core::math::UVec2,
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
