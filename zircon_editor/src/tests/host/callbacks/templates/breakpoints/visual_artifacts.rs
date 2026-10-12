use super::super::super::support::{
    env_lock, BuiltinWorkbenchWindowTemplateSurfaceBridge, UiEventKind, UiFrame, UiSize,
};
use super::support::{
    assert_frame_value, workbench_window_node, COMPACT_WORKBENCH_HEIGHT, COMPACT_WORKBENCH_WIDTH,
    FULL_WORKBENCH_HEIGHT, FULL_WORKBENCH_WIDTH, NARROW_WORKBENCH_HEIGHT, NARROW_WORKBENCH_WIDTH,
};
use crate::ui::retained_host::paint_runtime_render_commands_for_test;
use std::path::PathBuf;

const MODULE_OVERFLOW_SCREENSHOT: &str = "editor-window-m3-workbench-module-overflow-900x620.png";
const MVP_RUN_CONTROLS_SCREENSHOT: &str = "editor-window-m3-workbench-mvp-run-controls-640x520.png";
const RUN_MODE_SCREENSHOT: &str = "editor-window-m3-workbench-run-mode-1672x941.png";

#[test]
#[ignore = "writes a visual artifact under docs/tests/editor"]
fn capture_workbench_module_overflow_visual_artifact() {
    let _guard = match env_lock().lock() {
        Ok(guard) => guard,
        Err(error) => panic!("test environment lock is poisoned: {error}"),
    };

    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(
        COMPACT_WORKBENCH_WIDTH as f32,
        COMPACT_WORKBENCH_HEIGHT as f32,
    ))
    .expect("workbench bridge should build");
    for control_id in [
        "WorkbenchPopupMenu",
        "WorkbenchDropdownButton",
        "WorkbenchInputDropdown",
    ] {
        bridge
            .close_popup(control_id)
            .expect("component lab popup samples should close for focused screenshot capture");
    }
    bridge
        .dispatch_control_state("WorkbenchModuleMore", UiEventKind::Click)
        .expect("module overflow should dispatch")
        .expect("module overflow should expose a binding");
    let menu_frame = bridge
        .control_frame("WorkbenchModuleOverflowMenu")
        .expect("opened module overflow menu should have a frame");
    let bytes = paint_runtime_render_commands_for_test(
        COMPACT_WORKBENCH_WIDTH,
        COMPACT_WORKBENCH_HEIGHT,
        &bridge.surface().render_extract.list.commands,
    );
    assert!(
        first_non_black_pixel_in_frame(
            &bytes,
            COMPACT_WORKBENCH_WIDTH,
            COMPACT_WORKBENCH_HEIGHT,
            menu_frame,
        )
        .is_some(),
        "opened module overflow menu should paint visible pixels"
    );

    save_screenshot(
        screenshot_path(MODULE_OVERFLOW_SCREENSHOT),
        &bytes,
        COMPACT_WORKBENCH_WIDTH,
        COMPACT_WORKBENCH_HEIGHT,
    );
}

#[test]
#[ignore = "writes a visual artifact under docs/tests/editor"]
fn capture_narrow_workbench_mvp_run_controls_visual_artifact() {
    let _guard = match env_lock().lock() {
        Ok(guard) => guard,
        Err(error) => panic!("test environment lock is poisoned: {error}"),
    };

    let bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(
        NARROW_WORKBENCH_WIDTH as f32,
        NARROW_WORKBENCH_HEIGHT as f32,
    ))
    .expect("narrow workbench bridge should build");
    let run_group = bridge
        .control_frame("WorkbenchToolbarRunGroup")
        .expect("narrow toolbar should keep the MVP run group reachable");
    let play = bridge
        .control_frame("WorkbenchRunPlay")
        .expect("narrow toolbar should keep Play reachable");
    let run_mode = bridge
        .control_frame("WorkbenchRunMode")
        .expect("narrow toolbar should keep Run Mode reachable");
    assert_frame_value("narrow MVP run group width", run_group.width, 70.0);
    assert_frame_value(
        "narrow Play to Run Mode gap",
        run_mode.x - play.right(),
        4.0,
    );

    let bytes = paint_runtime_render_commands_for_test(
        NARROW_WORKBENCH_WIDTH,
        NARROW_WORKBENCH_HEIGHT,
        &bridge.surface().render_extract.list.commands,
    );
    for (label, frame) in [("Play", play), ("Run Mode", run_mode)] {
        assert!(
            first_non_black_pixel_in_frame(
                &bytes,
                NARROW_WORKBENCH_WIDTH,
                NARROW_WORKBENCH_HEIGHT,
                frame,
            )
            .is_some(),
            "narrow {label} control should paint visible pixels"
        );
    }

    save_screenshot(
        screenshot_path(MVP_RUN_CONTROLS_SCREENSHOT),
        &bytes,
        NARROW_WORKBENCH_WIDTH,
        NARROW_WORKBENCH_HEIGHT,
    );
}

fn screenshot_path(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_editor should live under the repository root")
        .join("docs")
        .join("tests")
        .join("editor")
        .join(file_name)
}

fn save_screenshot(path: PathBuf, bytes: &[u8], width: u32, height: u32) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("screenshot directory should exist");
    }
    image::save_buffer_with_format(
        &path,
        bytes,
        width,
        height,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("workbench screenshot should be written");
    println!("wrote {}", path.display());
}

fn first_non_black_pixel_in_frame(
    bytes: &[u8],
    width: u32,
    height: u32,
    frame: UiFrame,
) -> Option<[u8; 4]> {
    let start_x = frame.x.floor().max(0.0) as u32;
    let start_y = frame.y.floor().max(0.0) as u32;
    let end_x = (frame.x + frame.width).ceil().min(width as f32) as u32;
    let end_y = (frame.y + frame.height).ceil().min(height as f32) as u32;

    (start_y..end_y)
        .flat_map(|y| (start_x..end_x).map(move |x| (x, y)))
        .map(|(x, y)| pixel(bytes, width, x, y))
        .find(|pixel| *pixel != [0, 0, 0, 255])
}

fn pixel(bytes: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
    let offset = ((y as usize * width as usize) + x as usize) * 4;
    [
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]
}

#[test]
#[ignore = "writes a visual artifact under docs/tests/editor"]
fn capture_full_workbench_run_mode_visual_artifact() {
    let _guard = match env_lock().lock() {
        Ok(guard) => guard,
        Err(error) => panic!("test environment lock is poisoned: {error}"),
    };

    let bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(
        FULL_WORKBENCH_WIDTH as f32,
        FULL_WORKBENCH_HEIGHT as f32,
    ))
    .expect("workbench bridge should build");
    let run_mode_frame = bridge
        .control_frame("WorkbenchRunMode")
        .expect("full toolbar should expose the run mode trigger");
    let run_mode = workbench_window_node(&bridge, "WorkbenchRunMode");
    assert_eq!(
        run_mode.icon_name.as_str(),
        "zircon_editor_shell/toolbar/dropdown.svg",
        "run mode should use a toolbar dropdown glyph for the visual artifact"
    );

    let bytes = paint_runtime_render_commands_for_test(
        FULL_WORKBENCH_WIDTH,
        FULL_WORKBENCH_HEIGHT,
        &bridge.surface().render_extract.list.commands,
    );
    assert!(
        first_non_black_pixel_in_frame(
            &bytes,
            FULL_WORKBENCH_WIDTH,
            FULL_WORKBENCH_HEIGHT,
            run_mode_frame
        )
        .is_some(),
        "visible run mode trigger should paint pixels in the full-width artifact"
    );

    let path = screenshot_path(RUN_MODE_SCREENSHOT);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("run mode screenshot directory should exist");
    }
    image::save_buffer_with_format(
        &path,
        &bytes,
        FULL_WORKBENCH_WIDTH,
        FULL_WORKBENCH_HEIGHT,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("run mode screenshot should be written");
    println!("wrote {}", path.display());
}
