use std::path::{Path, PathBuf};

use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::{
    paint_template_nodes_for_test_with_background, TemplateNodeFrameData, TemplatePaneMenuItemData,
    TemplatePaneNodeData, TemplatePaneOptionData,
};

const POPUP_SELECTION_COMPONENT_SCREENSHOT: &str =
    "editor-components-popup-selection-list-900x360.png";
const POPUP_SELECTION_ATLAS_WIDTH: u32 = 900;
const POPUP_SELECTION_ATLAS_HEIGHT: u32 = 360;
const POPUP_SELECTION_ATLAS_BACKGROUND: [u8; 4] = [17, 20, 22, 255];

#[test]
fn popup_selection_list_component_visual_paints_menu_options_shortcuts_and_states() {
    let bytes = popup_selection_component_bytes();

    let menu_row_surface = pixel_at(&bytes, 44, 132);
    assert!(
        distinct_pixel_count(
            &bytes,
            38,
            127,
            116,
            16,
            &[POPUP_SELECTION_ATLAS_BACKGROUND, menu_row_surface],
        ) > 0,
        "menu popup should paint runtime label text above row surface"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            166,
            127,
            56,
            16,
            &[POPUP_SELECTION_ATLAS_BACKGROUND, menu_row_surface],
        ) > 0,
        "menu popup should paint right-aligned shortcut text"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            244,
            126,
            18,
            18,
            &[POPUP_SELECTION_ATLAS_BACKGROUND, menu_row_surface],
        ) > 0,
        "menu popup should paint right-side icon/check/chevron adornments"
    );

    assert!(
        distinct_pixel_count(
            &bytes,
            38,
            210,
            212,
            4,
            &[POPUP_SELECTION_ATLAS_BACKGROUND, menu_row_surface],
        ) > 0,
        "menu popup should paint separator as its own retained row layer"
    );

    let option_row_surface = pixel_at(&bytes, 334, 134);
    assert!(
        distinct_pixel_count(
            &bytes,
            340,
            127,
            126,
            16,
            &[POPUP_SELECTION_ATLAS_BACKGROUND, option_row_surface],
        ) > 0,
        "option popup should paint retained option labels"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            542,
            126,
            22,
            18,
            &[POPUP_SELECTION_ATLAS_BACKGROUND, option_row_surface],
        ) > 0,
        "selected option row should paint a right-side check adornment"
    );

    let trigger_surface = pixel_at(&bytes, 632, 148);
    assert!(
        distinct_pixel_count(
            &bytes,
            642,
            140,
            110,
            20,
            &[POPUP_SELECTION_ATLAS_BACKGROUND, trigger_surface],
        ) > 0,
        "anchored dropdown trigger should paint current value text"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            622,
            166,
            176,
            90,
            &[POPUP_SELECTION_ATLAS_BACKGROUND, trigger_surface],
        ) > 0,
        "anchored dropdown should paint projected popup rows below the trigger"
    );
}

#[test]
#[ignore = "writes local popup/selection-list component screenshot artifact for visual review"]
fn capture_popup_selection_component_visual_artifact() {
    let bytes = popup_selection_component_bytes();
    let output_path = visual_layout_output_path(POPUP_SELECTION_COMPONENT_SCREENSHOT);

    image::save_buffer_with_format(
        &output_path,
        &bytes,
        POPUP_SELECTION_ATLAS_WIDTH,
        POPUP_SELECTION_ATLAS_HEIGHT,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("popup selection component screenshot should be written as PNG");

    assert!(
        output_path.exists(),
        "expected visual screenshot at {}",
        output_path.display()
    );
}

fn popup_selection_component_bytes() -> Vec<u8> {
    paint_template_nodes_for_test_with_background(
        POPUP_SELECTION_ATLAS_WIDTH,
        POPUP_SELECTION_ATLAS_HEIGHT,
        POPUP_SELECTION_ATLAS_BACKGROUND,
        model_rc(popup_selection_component_nodes()),
    )
}

fn popup_selection_component_nodes() -> Vec<TemplatePaneNodeData> {
    vec![
        surface("PopupSelectionRoot", "shell", 0.0, 0.0, 900.0, 360.0),
        label(
            "PopupSelectionTitle",
            "Popup Selection Lists",
            22.0,
            20.0,
            300.0,
            22.0,
            13.0,
            "",
        ),
        label(
            "PopupSelectionSubtitle",
            "Menu rows, option rows, shortcuts, separators and anchored dropdown popups",
            22.0,
            42.0,
            720.0,
            18.0,
            10.0,
            "muted",
        ),
        surface("PopupMenuPanel", "panel", 18.0, 78.0, 266.0, 214.0),
        surface("PopupOptionsPanel", "panel", 318.0, 78.0, 266.0, 214.0),
        surface("PopupDropdownPanel", "inset", 618.0, 78.0, 264.0, 214.0),
        label(
            "PopupMenuTitle",
            "Menu Rows",
            36.0,
            96.0,
            120.0,
            18.0,
            11.0,
            "",
        ),
        label(
            "PopupOptionsTitle",
            "Option Rows",
            336.0,
            96.0,
            120.0,
            18.0,
            11.0,
            "",
        ),
        label(
            "PopupDropdownTitle",
            "Anchored Select",
            636.0,
            96.0,
            140.0,
            18.0,
            11.0,
            "",
        ),
        popup_menu(
            "WorkbenchPopupSelectionMenu",
            32.0,
            120.0,
            238.0,
            156.0,
            &[
                menu_item("New Asset|icon=plus", "Ctrl+N", false, false, false, false),
                menu_item(
                    "Open Project|icon=folder",
                    "Ctrl+O",
                    false,
                    false,
                    true,
                    false,
                ),
                menu_item("Save All|icon=save", "Ctrl+S", true, false, false, false),
                menu_item("", "", false, true, false, false),
                menu_item(
                    "Delete Selected|danger,icon=trash",
                    "Del",
                    false,
                    false,
                    false,
                    true,
                ),
                menu_item("More Tools|submenu", "", false, false, true, false),
            ],
        ),
        popup_options(
            "WorkbenchPopupSelectionOptions",
            332.0,
            120.0,
            238.0,
            156.0,
            "DropdownPopup",
            "dropdown-popup",
            &[
                option("Selected View", true, false, false, false, false),
                option("Hovered Choice", false, true, false, false, false),
                option("Focused Choice", false, false, true, false, false),
                option("Disabled Choice", false, false, false, true, false),
                option("Loading Choice", false, false, false, false, true),
            ],
        ),
        dropdown(
            "WorkbenchInputDropdownPopupSelection",
            "Lighting",
            632.0,
            130.0,
            176.0,
            30.0,
            &[
                option("Lighting", true, false, false, false, false),
                option("Environment", false, true, false, false, false),
                option("Editor Only", false, false, false, false, false),
            ],
        ),
        label(
            "PopupDropdownCopy",
            "Trigger + projected option list use the same popup row painter",
            636.0,
            268.0,
            218.0,
            18.0,
            10.0,
            "muted",
        ),
    ]
}

fn popup_menu(
    control_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    items: &[TemplatePaneMenuItemData],
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Menu".into(),
        component_role: "menu".into(),
        popup_open: true,
        structured_menu_items: model_rc(items.to_vec()),
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn popup_options(
    control_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    role: &str,
    component_role: &str,
    options: &[TemplatePaneOptionData],
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: role.into(),
        component_role: component_role.into(),
        popup_open: true,
        structured_options: model_rc(options.to_vec()),
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn dropdown(
    control_id: &str,
    value: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    options: &[TemplatePaneOptionData],
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Dropdown".into(),
        component_role: "dropdown".into(),
        component_variant: "workbench-dropdown".into(),
        value_text: value.into(),
        popup_open: true,
        focused: true,
        structured_options: model_rc(options.to_vec()),
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn menu_item(
    raw: &str,
    shortcut: &str,
    checked: bool,
    separator: bool,
    hovered: bool,
    disabled: bool,
) -> TemplatePaneMenuItemData {
    let label = raw.split('|').next().unwrap_or_default();
    TemplatePaneMenuItemData {
        raw: raw.into(),
        action_id: label.into(),
        label: label.into(),
        shortcut: shortcut.into(),
        checked,
        separator,
        disabled: disabled || separator,
        hovered,
        ..TemplatePaneMenuItemData::default()
    }
}

fn option(
    label: &str,
    selected: bool,
    hovered: bool,
    focused: bool,
    disabled: bool,
    loading: bool,
) -> TemplatePaneOptionData {
    TemplatePaneOptionData {
        id: label.into(),
        label: label.into(),
        selected,
        hovered,
        focused,
        disabled,
        loading,
        ..TemplatePaneOptionData::default()
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
    let index = ((y as usize * POPUP_SELECTION_ATLAS_WIDTH as usize) + x as usize) * 4;
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
            let index = ((py as usize * POPUP_SELECTION_ATLAS_WIDTH as usize) + px as usize) * 4;
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
