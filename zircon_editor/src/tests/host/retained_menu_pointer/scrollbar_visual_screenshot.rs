use std::path::{Path, PathBuf};

use crate::ui::retained_host::paint_scrollbar_component_for_test;

const SCROLLBAR_COMPONENT_SCREENSHOT: &str = "editor-components-scrollbar-900x360.png";

#[test]
#[ignore = "writes local scrollbar component screenshot artifact for visual review"]
fn capture_scrollbar_component_visual_artifact() {
    let width = 900;
    let height = 360;
    let bytes = paint_scrollbar_component_for_test(width, height);
    let output_path = visual_layout_output_path(SCROLLBAR_COMPONENT_SCREENSHOT);

    image::save_buffer_with_format(
        &output_path,
        &bytes,
        width,
        height,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("scrollbar component screenshot should be written as PNG");

    assert!(
        output_path.exists(),
        "expected visual screenshot at {}",
        output_path.display()
    );
}

fn visual_layout_output_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("editor crate should live under the repository root")
        .join("docs")
        .join("tests")
        .join("editor")
}

fn visual_layout_output_path(filename: &str) -> PathBuf {
    let output_dir = visual_layout_output_dir();
    std::fs::create_dir_all(&output_dir).expect("visual-layout output directory should exist");
    output_dir.join(filename)
}
