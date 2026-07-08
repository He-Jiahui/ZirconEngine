use std::path::{Path, PathBuf};

use crate::ui::retained_host::paint_scrollbar_component_for_test;

const SCROLLBAR_COMPONENT_SCREENSHOT: &str = "editor-components-scrollbar-900x360.png";
const SCROLLBAR_ATLAS_WIDTH: u32 = 900;
const SCROLLBAR_ATLAS_HEIGHT: u32 = 360;
const SCROLLBAR_ATLAS_BACKGROUND: [u8; 4] = [17, 20, 22, 255];

#[test]
fn scrollbar_component_visual_paints_clipped_rows_tracks_and_thumb_states() {
    let bytes = scrollbar_component_bytes();

    assert!(
        distinct_pixel_count(&bytes, 56, 18, 320, 24, &[SCROLLBAR_ATLAS_BACKGROUND]) > 0,
        "scrollbar screenshot should paint a retained text title for visual context"
    );

    let shell = pixel_at(&bytes, 8, 8);
    let left_panel = pixel_at(&bytes, 78, 90);
    assert_ne!(
        left_panel, shell,
        "scrollbar demo panel should paint above the shell background"
    );
    assert_eq!(
        pixel_at(&bytes, 100, 330),
        shell,
        "scrollbar demo rows must be clipped to the panel instead of spilling below it"
    );

    let left_track = pixel_at(&bytes, 271, 160);
    let left_thumb = pixel_at(&bytes, 271, 100);
    assert_ne!(
        left_track, left_panel,
        "inactive scrollbar should paint a narrow track at the panel edge"
    );
    assert_ne!(
        left_thumb, left_track,
        "inactive scrollbar should paint a distinct thumb at the top of the track"
    );

    let active_track = pixel_at(&bytes, 548, 110);
    let active_thumb = pixel_at(&bytes, 548, 186);
    assert_ne!(
        active_thumb, active_track,
        "active scrollbar should paint a distinct hovered thumb"
    );
    assert_ne!(
        active_thumb, left_thumb,
        "active thumb should use the highlighted retained scrollbar state"
    );

    let end_track = pixel_at(&bytes, 825, 120);
    let end_thumb = pixel_at(&bytes, 825, 286);
    assert_ne!(
        end_thumb, end_track,
        "end-scroll scrollbar should move the thumb to the bottom of the track"
    );
}

#[test]
#[ignore = "writes local scrollbar component screenshot artifact for visual review"]
fn capture_scrollbar_component_visual_artifact() {
    let bytes = scrollbar_component_bytes();
    let output_path = visual_layout_output_path(SCROLLBAR_COMPONENT_SCREENSHOT);

    image::save_buffer_with_format(
        &output_path,
        &bytes,
        SCROLLBAR_ATLAS_WIDTH,
        SCROLLBAR_ATLAS_HEIGHT,
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

fn scrollbar_component_bytes() -> Vec<u8> {
    paint_scrollbar_component_for_test(SCROLLBAR_ATLAS_WIDTH, SCROLLBAR_ATLAS_HEIGHT)
}

fn pixel_at(bytes: &[u8], x: u32, y: u32) -> [u8; 4] {
    let index = ((y as usize * SCROLLBAR_ATLAS_WIDTH as usize) + x as usize) * 4;
    [
        bytes[index],
        bytes[index + 1],
        bytes[index + 2],
        bytes[index + 3],
    ]
}

fn distinct_pixel_count(
    bytes: &[u8],
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    excluded_colors: &[[u8; 4]],
) -> usize {
    let mut changed = 0;
    for py in y..(y + height) {
        for px in x..(x + width) {
            let color = pixel_at(bytes, px, py);
            if !excluded_colors.contains(&color) {
                changed += 1;
            }
        }
    }
    changed
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
