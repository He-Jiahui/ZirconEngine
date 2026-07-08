use std::path::{Path, PathBuf};

use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::primitives::SharedString;
use crate::ui::retained_host::{
    paint_template_nodes_for_test_with_background, TemplateNodeFrameData, TemplatePaneNodeData,
};

const ROW_COMPONENT_SCREENSHOT: &str = "editor-components-rows-900x360.png";
const ROW_ATLAS_WIDTH: u32 = 900;
const ROW_ATLAS_HEIGHT: u32 = 360;
const ROW_ATLAS_BACKGROUND: [u8; 4] = [17, 20, 22, 255];

#[test]
fn row_component_visual_paints_list_tree_table_states_and_actions() {
    let bytes = row_component_bytes();

    let list_surface = pixel_at(&bytes, 46, 132);
    assert_ne!(
        list_surface, ROW_ATLAS_BACKGROUND,
        "selected list row should paint a visible row surface"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            54,
            126,
            126,
            22,
            &[ROW_ATLAS_BACKGROUND, list_surface],
        ) > 0,
        "list rows should paint retained text above their row surface"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            170,
            126,
            24,
            22,
            &[ROW_ATLAS_BACKGROUND, list_surface],
        ) > 0,
        "list rows should paint trailing selection/navigation adornments"
    );

    let tree_surface = pixel_at(&bytes, 258, 132);
    assert_ne!(
        tree_surface, ROW_ATLAS_BACKGROUND,
        "selected tree row should paint its muted selected surface"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            274,
            126,
            44,
            22,
            &[ROW_ATLAS_BACKGROUND, tree_surface],
        ) > 0,
        "tree rows should paint disclosure and object icon glyphs"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            430,
            126,
            42,
            22,
            &[ROW_ATLAS_BACKGROUND, tree_surface],
        ) > 0,
        "tree rows should paint action button slots and shell icon pixels"
    );

    let table_header_surface = pixel_at(&bytes, 502, 120);
    assert_ne!(
        table_header_surface, ROW_ATLAS_BACKGROUND,
        "table header should paint a recessed header surface"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            514,
            114,
            210,
            18,
            &[ROW_ATLAS_BACKGROUND, table_header_surface],
        ) > 0,
        "table header should paint column text and header action"
    );

    let table_selected_surface = pixel_at(&bytes, 502, 150);
    assert_ne!(
        table_selected_surface, ROW_ATLAS_BACKGROUND,
        "selected table row should paint a selected row surface"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            514,
            144,
            300,
            18,
            &[ROW_ATLAS_BACKGROUND, table_selected_surface],
        ) > 0,
        "table row should paint normalized cells and row action"
    );

    let table_hovered_surface = pixel_at(&bytes, 502, 180);
    assert!(
        distinct_pixel_count(
            &bytes,
            824,
            174,
            24,
            20,
            &[ROW_ATLAS_BACKGROUND, table_hovered_surface],
        ) > 0,
        "hovered table row should expose its trailing action slot"
    );
}

#[test]
#[ignore = "writes local row/list component screenshot artifact for visual review"]
fn capture_row_component_visual_artifact() {
    let bytes = row_component_bytes();
    let output_path = visual_layout_output_path(ROW_COMPONENT_SCREENSHOT);

    image::save_buffer_with_format(
        &output_path,
        &bytes,
        ROW_ATLAS_WIDTH,
        ROW_ATLAS_HEIGHT,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("row component screenshot should be written as PNG");

    assert!(
        output_path.exists(),
        "expected visual screenshot at {}",
        output_path.display()
    );
}

fn row_component_bytes() -> Vec<u8> {
    paint_template_nodes_for_test_with_background(
        ROW_ATLAS_WIDTH,
        ROW_ATLAS_HEIGHT,
        ROW_ATLAS_BACKGROUND,
        model_rc(row_component_nodes()),
    )
}

fn row_component_nodes() -> Vec<TemplatePaneNodeData> {
    vec![
        surface("RowRoot", "shell", 0.0, 0.0, 900.0, 360.0),
        label(
            "RowTitle",
            "Rows and Lists",
            22.0,
            20.0,
            260.0,
            22.0,
            13.0,
            "",
        ),
        label(
            "RowSubtitle",
            "List, scene tree and asset table rows use retained row painters",
            22.0,
            42.0,
            620.0,
            18.0,
            10.0,
            "muted",
        ),
        surface("RowListPanel", "panel", 18.0, 78.0, 188.0, 214.0),
        surface("RowTreePanel", "panel", 230.0, 78.0, 244.0, 214.0),
        surface("RowTablePanel", "inset", 498.0, 78.0, 384.0, 214.0),
        label(
            "RowListTitle",
            "List Rows",
            36.0,
            96.0,
            140.0,
            18.0,
            11.0,
            "",
        ),
        label(
            "RowTreeTitle",
            "Tree Rows",
            248.0,
            96.0,
            140.0,
            18.0,
            11.0,
            "",
        ),
        label(
            "RowTableTitle",
            "Table Rows",
            516.0,
            96.0,
            140.0,
            18.0,
            11.0,
            "",
        ),
        list_row(
            "WorkbenchListChecked",
            "Selected material",
            RowState::Selected,
            38.0,
            122.0,
            148.0,
            30.0,
        ),
        list_row(
            "WorkbenchListNavigation",
            "Navigate to asset",
            RowState::Normal,
            38.0,
            160.0,
            148.0,
            30.0,
        ),
        list_row(
            "WorkbenchListDisabled",
            "Disabled source",
            RowState::Disabled,
            38.0,
            198.0,
            148.0,
            30.0,
        ),
        tree_row(
            "WorkbenchScenePropsItem",
            "Props",
            "zircon_editor_shell/scene/props.svg",
            2,
            true,
            250.0,
            122.0,
            204.0,
            30.0,
        ),
        tree_row(
            "WorkbenchSceneEnvironmentItem",
            "Environment",
            "zircon_editor_shell/scene/sky.svg",
            1,
            false,
            250.0,
            160.0,
            204.0,
            30.0,
        ),
        tree_row(
            "WorkbenchScenePlayerStartItem",
            "PlayerStart",
            "zircon_editor_shell/scene/player-start.svg",
            0,
            false,
            250.0,
            198.0,
            204.0,
            30.0,
        ),
        table_row(
            "WorkbenchTableHeader",
            &["Name", "Type", "Size", "Revision"],
            RowState::Normal,
            518.0,
            112.0,
            344.0,
            28.0,
        ),
        table_row(
            "WorkbenchTableSelected",
            &["SM_Chair", "Mesh", "512K", "r41"],
            RowState::Selected,
            518.0,
            142.0,
            344.0,
            28.0,
        ),
        table_row(
            "WorkbenchTableHovered",
            &["M_Wood", "Mat", "128K", "r37"],
            RowState::Hovered,
            518.0,
            172.0,
            344.0,
            28.0,
        ),
        table_row(
            "WorkbenchTableTail",
            &["", "", "", ""],
            RowState::Normal,
            518.0,
            202.0,
            344.0,
            28.0,
        ),
        label(
            "RowTableCopy",
            "Header, selected, hovered and tail rows stay on one recessed table surface",
            516.0,
            244.0,
            336.0,
            30.0,
            10.0,
            "muted",
        ),
    ]
}

#[derive(Clone, Copy)]
enum RowState {
    Normal,
    Hovered,
    Selected,
    Disabled,
}

fn list_row(
    control_id: &str,
    text: &str,
    state: RowState,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    let selected = matches!(state, RowState::Selected);
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "ListRow".into(),
        component_role: "list-row".into(),
        text: text.into(),
        selected,
        checked: selected,
        hovered: matches!(state, RowState::Hovered),
        disabled: matches!(state, RowState::Disabled),
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn tree_row(
    control_id: &str,
    text: &str,
    icon_name: &str,
    depth: i32,
    selected: bool,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "TreeRow".into(),
        component_role: "tree-row".into(),
        text: text.into(),
        icon_name: icon_name.into(),
        tree_depth: depth,
        tree_indent_px: if selected { 40.0 } else { 0.0 },
        selected,
        checked: selected,
        expanded: !text.contains("Player"),
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn table_row(
    control_id: &str,
    cells: &[&str],
    state: RowState,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    let selected = matches!(state, RowState::Selected);
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Table".into(),
        component_role: "table-row".into(),
        component_variant: "workbench-table-row".into(),
        options: model_rc(cells.iter().map(|cell| SharedString::from(*cell)).collect()),
        selected,
        checked: selected,
        hovered: matches!(state, RowState::Hovered),
        disabled: matches!(state, RowState::Disabled),
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn surface(
    control_id: &str,
    variant: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Panel".into(),
        surface_variant: variant.into(),
        border_width: 1.0,
        corner_radius: 6.0,
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn label(
    control_id: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    font_size: f32,
    tone: &str,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Label".into(),
        text: text.into(),
        font_size,
        text_tone: tone.into(),
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
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
    let index = ((y as usize * ROW_ATLAS_WIDTH as usize) + x as usize) * 4;
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
            let index = ((py as usize * ROW_ATLAS_WIDTH as usize) + px as usize) * 4;
            let color = [
                bytes[index],
                bytes[index + 1],
                bytes[index + 2],
                bytes[index + 3],
            ];
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
