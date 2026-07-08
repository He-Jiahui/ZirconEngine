use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::ui::retained_host::primitives::{ModelRc, VecModel};
use crate::ui::retained_host::{
    paint_template_nodes_for_test_with_background, TemplateNodeFrameData, TemplatePaneNodeData,
    TemplatePaneOptionData,
};

const COMMAND_PALETTE_COMPONENT_SCREENSHOT: &str = "editor-components-command-palette-900x360.png";
const COMMAND_PALETTE_ATLAS_WIDTH: u32 = 900;
const COMMAND_PALETTE_ATLAS_HEIGHT: u32 = 360;
const COMMAND_PALETTE_ATLAS_BACKGROUND: [u8; 4] = [17, 20, 22, 255];

#[test]
fn command_palette_component_visual_paints_search_rows_details_and_empty_state() {
    let bytes = command_palette_component_bytes();

    let search_surface = pixel_at(&bytes, 96, 80);
    assert!(
        distinct_pixel_count(
            &bytes,
            96,
            76,
            20,
            20,
            &[COMMAND_PALETTE_ATLAS_BACKGROUND, search_surface],
        ) > 0,
        "command palette search field should paint the retained search icon"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            126,
            76,
            96,
            18,
            &[COMMAND_PALETTE_ATLAS_BACKGROUND, search_surface],
        ) > 0,
        "command palette search field should paint runtime query text"
    );

    let selected_row_surface = pixel_at(&bytes, 100, 112);
    assert!(
        distinct_pixel_count(
            &bytes,
            112,
            106,
            146,
            20,
            &[COMMAND_PALETTE_ATLAS_BACKGROUND, selected_row_surface],
        ) > 0,
        "selected command row should paint the command label"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            466,
            106,
            62,
            20,
            &[COMMAND_PALETTE_ATLAS_BACKGROUND, selected_row_surface],
        ) > 0,
        "selected command row should paint right-side shortcut text"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            92,
            108,
            7,
            18,
            &[COMMAND_PALETTE_ATLAS_BACKGROUND, selected_row_surface],
        ) > 0,
        "matched command row should paint the accent match indicator"
    );

    let hovered_row_surface = pixel_at(&bytes, 100, 162);
    assert!(
        distinct_pixel_count(
            &bytes,
            112,
            156,
            146,
            20,
            &[COMMAND_PALETTE_ATLAS_BACKGROUND, hovered_row_surface],
        ) > 0,
        "hot command row should still paint a readable label"
    );
    assert_ne!(
        hovered_row_surface, selected_row_surface,
        "hovered row surface should stay visually distinct from selected row surface"
    );

    let empty_search_surface = pixel_at(&bytes, 630, 80);
    assert!(
        distinct_pixel_count(
            &bytes,
            662,
            76,
            136,
            18,
            &[COMMAND_PALETTE_ATLAS_BACKGROUND, empty_search_surface],
        ) > 0,
        "empty command palette should paint placeholder search text"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            630,
            126,
            142,
            20,
            &[COMMAND_PALETTE_ATLAS_BACKGROUND, empty_search_surface],
        ) > 0,
        "empty command palette should paint the no-results message"
    );
}

#[test]
#[ignore = "writes local command palette component screenshot artifact for visual review"]
fn capture_command_palette_component_visual_artifact() {
    let bytes = command_palette_component_bytes();
    let output_path = visual_layout_output_path(COMMAND_PALETTE_COMPONENT_SCREENSHOT);

    image::save_buffer_with_format(
        &output_path,
        &bytes,
        COMMAND_PALETTE_ATLAS_WIDTH,
        COMMAND_PALETTE_ATLAS_HEIGHT,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("command palette component screenshot should be written as PNG");

    assert!(
        output_path.exists(),
        "expected visual screenshot at {}",
        output_path.display()
    );
}

fn command_palette_component_bytes() -> Vec<u8> {
    paint_template_nodes_for_test_with_background(
        COMMAND_PALETTE_ATLAS_WIDTH,
        COMMAND_PALETTE_ATLAS_HEIGHT,
        COMMAND_PALETTE_ATLAS_BACKGROUND,
        model_rc(command_palette_component_nodes()),
    )
}

fn command_palette_component_nodes() -> Vec<TemplatePaneNodeData> {
    vec![
        command_palette(
            "WorkbenchCommandPaletteVisual",
            "build",
            70.0,
            54.0,
            520.0,
            244.0,
            &[
                option(
                    "build.project",
                    "Build Project",
                    "Ctrl+B",
                    true,
                    false,
                    true,
                    false,
                    false,
                ),
                option(
                    "scene.open",
                    "Open Scene Editor",
                    "Ctrl+1",
                    false,
                    false,
                    false,
                    false,
                    true,
                ),
                option(
                    "asset.import",
                    "Run Asset Import",
                    "Ctrl+I",
                    false,
                    false,
                    false,
                    true,
                    false,
                ),
                option(
                    "content.sync",
                    "Sync Content Cache",
                    "Alt+S",
                    false,
                    true,
                    false,
                    false,
                    false,
                ),
            ],
        ),
        command_palette(
            "WorkbenchCommandPaletteEmptyVisual",
            "",
            616.0,
            54.0,
            224.0,
            244.0,
            &[],
        ),
    ]
}

fn command_palette(
    control_id: &str,
    query: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    options: &[TemplatePaneOptionData],
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        node_id: format!("{control_id}.node").into(),
        role: "CommandPalette".into(),
        component_role: "command-palette".into(),
        popup_open: true,
        focused: true,
        search_query: query.into(),
        structured_options: model_rc(options.to_vec()),
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn option(
    id: &str,
    label: &str,
    shortcut: &str,
    selected: bool,
    disabled: bool,
    focused: bool,
    hovered: bool,
    special: bool,
) -> TemplatePaneOptionData {
    TemplatePaneOptionData {
        id: id.into(),
        label: label.into(),
        description: shortcut.into(),
        selected,
        disabled,
        focused,
        hovered,
        special,
        matched: true,
        ..TemplatePaneOptionData::default()
    }
}

fn model_rc<T: Clone + 'static>(items: Vec<T>) -> ModelRc<T> {
    ModelRc::from(Rc::new(VecModel::from(items)))
}

fn frame(x: f32, y: f32, width: f32, height: f32) -> TemplateNodeFrameData {
    TemplateNodeFrameData {
        x,
        y,
        width,
        height,
    }
}

fn pixel_at(bytes: &[u8], x: u32, y: u32) -> [u8; 4] {
    let index = ((y as usize * COMMAND_PALETTE_ATLAS_WIDTH as usize) + x as usize) * 4;
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
