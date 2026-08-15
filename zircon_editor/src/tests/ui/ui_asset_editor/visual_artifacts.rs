use std::path::{Path, PathBuf};

use crate::ui::asset_editor::ui_asset_editor_surface_for_test;
use crate::ui::retained_host::paint_runtime_render_commands_for_test;
use zircon_runtime::ui::surface::extract_ui_render_tree;
use zircon_runtime_interface::ui::layout::UiSize;

const UI_ASSET_EDITOR_ARTIFACTS: &[(u32, u32, &str)] = &[
    (640, 420, "editor-ui-asset-workbench-640x420.png"),
    (900, 620, "editor-ui-asset-workbench-900x620.png"),
    (1280, 720, "editor-ui-asset-workbench-1280x720.png"),
];

#[test]
#[ignore = "writes actual UI Asset Editor screenshots under docs/tests/editor"]
fn capture_ui_asset_editor_v2_surface_visual_artifacts() {
    for &(width, height, filename) in UI_ASSET_EDITOR_ARTIFACTS {
        let surface = ui_asset_editor_surface_for_test(UiSize::new(width as f32, height as f32));
        let render_tree = extract_ui_render_tree(&surface.tree);
        assert!(
            !render_tree.list.commands.is_empty(),
            "UI Asset Editor V2 surface must emit render commands at {width}x{height}"
        );
        let pixels =
            paint_runtime_render_commands_for_test(width, height, &render_tree.list.commands);
        assert!(
            pixels
                .chunks_exact(4)
                .any(|pixel| pixel[0] != 0 || pixel[1] != 0 || pixel[2] != 0 || pixel[3] != 255),
            "UI Asset Editor V2 surface must paint visible pixels at {width}x{height}"
        );
        let output_path = ui_asset_editor_artifact_path(filename);

        image::save_buffer_with_format(
            &output_path,
            &pixels,
            width,
            height,
            image::ColorType::Rgba8,
            image::ImageFormat::Png,
        )
        .expect("UI Asset Editor V2 screenshot should be written as PNG");
        assert!(
            output_path.exists(),
            "expected UI Asset Editor screenshot at {}",
            output_path.display()
        );
    }
}

fn ui_asset_editor_artifact_path(filename: &str) -> PathBuf {
    let output_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("editor crate should live under the repository root")
        .join("docs")
        .join("tests")
        .join("editor");
    std::fs::create_dir_all(&output_dir)
        .expect("UI Asset Editor screenshot directory should exist");
    output_dir.join(filename)
}
