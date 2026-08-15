const WORKBENCH_COMPONENT_ATLAS_SCREENSHOT: &str =
    "editor-components-workbench-slate-atlas-900x620.png";

mod primitives;

use super::*;
use primitives::*;

#[test]
#[ignore = "writes local workbench component visual atlas for bottom-up style review"]
fn capture_workbench_component_slate_atlas_visual_artifact() {
    let width = 900;
    let height = 620;
    let bytes = paint_template_nodes_for_test_with_background(
        width,
        height,
        [17, 20, 22, 255],
        crate::ui::layouts::common::model_rc(workbench_component_atlas_nodes()),
    );
    let output_path = visual_layout_output_path(WORKBENCH_COMPONENT_ATLAS_SCREENSHOT);

    image::save_buffer_with_format(
        &output_path,
        &bytes,
        width,
        height,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("workbench component atlas screenshot should be written as PNG");

    assert!(
        output_path.exists(),
        "expected visual screenshot at {}",
        output_path.display()
    );
}

fn workbench_component_atlas_nodes() -> Vec<TemplatePaneNodeData> {
    let mut nodes = vec![
        atlas_surface("AtlasRoot", "shell", 0.0, 0.0, 900.0, 620.0),
        atlas_label(
            "AtlasTitle",
            "Workbench Component Style Atlas",
            22.0,
            20.0,
            360.0,
            22.0,
            13.0,
            "",
        ),
        atlas_label(
            "AtlasSubtitle",
            "Buttons, text, inputs, image containers, rows, tables, bars and popups",
            22.0,
            42.0,
            560.0,
            18.0,
            10.0,
            "muted",
        ),
        atlas_surface("AtlasButtonsPanel", "panel", 18.0, 78.0, 272.0, 190.0),
        atlas_surface("AtlasInputsPanel", "panel", 306.0, 78.0, 276.0, 190.0),
        atlas_surface("AtlasRowsPanel", "panel", 598.0, 78.0, 284.0, 190.0),
        atlas_surface("AtlasComplexPanel", "panel", 18.0, 286.0, 420.0, 266.0),
        atlas_surface("AtlasContainersPanel", "panel", 454.0, 286.0, 428.0, 266.0),
        atlas_surface("AtlasStatusBar", "inset", 0.0, 578.0, 900.0, 42.0),
    ];

    nodes.extend([
        atlas_label(
            "AtlasButtonsTitle",
            "Buttons",
            34.0,
            96.0,
            220.0,
            18.0,
            11.0,
            "",
        ),
        atlas_button(
            "WorkbenchPrimaryButton",
            "Primary",
            "primary",
            34.0,
            128.0,
            112.0,
            26.0,
        ),
        atlas_button(
            "WorkbenchSecondaryButton",
            "Secondary",
            "secondary",
            156.0,
            128.0,
            112.0,
            26.0,
        ),
        atlas_button(
            "WorkbenchTertiaryButton",
            "Tertiary",
            "tertiary",
            34.0,
            162.0,
            112.0,
            26.0,
        ),
        atlas_button(
            "WorkbenchDangerButton",
            "Danger",
            "danger",
            156.0,
            162.0,
            112.0,
            26.0,
        ),
        atlas_button_state(
            "WorkbenchHoverButton",
            "Hover",
            "secondary",
            34.0,
            204.0,
            70.0,
            24.0,
            "hover",
        ),
        atlas_button_state(
            "WorkbenchPressedButton",
            "Pressed",
            "secondary",
            112.0,
            204.0,
            76.0,
            24.0,
            "pressed",
        ),
        atlas_button_state(
            "WorkbenchDisabledButton",
            "Disabled",
            "secondary",
            196.0,
            204.0,
            78.0,
            24.0,
            "disabled",
        ),
    ]);

    nodes.extend([
        atlas_label(
            "AtlasInputsTitle",
            "Inputs And Selection",
            322.0,
            96.0,
            220.0,
            18.0,
            11.0,
            "",
        ),
        atlas_field(
            "WorkbenchInputSearch",
            "Search assets...",
            322.0,
            128.0,
            238.0,
            28.0,
            "",
        ),
        atlas_field(
            "WorkbenchInputFocused",
            "Focused field",
            322.0,
            164.0,
            238.0,
            28.0,
            "focus",
        ),
        atlas_dropdown(
            "WorkbenchDropdownAtlas",
            "Kind: Mesh",
            322.0,
            200.0,
            116.0,
            28.0,
            "",
        ),
        atlas_selection(
            "WorkbenchCheckboxAtlas",
            "Checkbox",
            454.0,
            199.0,
            104.0,
            28.0,
            "checkbox",
            true,
        ),
        atlas_selection(
            "WorkbenchRadioAtlas",
            "Radio",
            322.0,
            234.0,
            96.0,
            26.0,
            "radio",
            true,
        ),
        atlas_selection(
            "WorkbenchToggleAtlas",
            "Snap",
            448.0,
            234.0,
            112.0,
            26.0,
            "toggle",
            true,
        ),
    ]);

    nodes.extend([
        atlas_label(
            "AtlasRowsTitle",
            "Rows And Lists",
            614.0,
            96.0,
            220.0,
            18.0,
            11.0,
            "",
        ),
        atlas_list_row(
            "WorkbenchListAsset0",
            "Neutral list row",
            614.0,
            126.0,
            246.0,
            30.0,
            "",
        ),
        atlas_list_row(
            "WorkbenchListAsset1",
            "Selected list row",
            614.0,
            160.0,
            246.0,
            30.0,
            "selected",
        ),
        atlas_tree_row(
            "WorkbenchSceneAssetItem",
            "Scene tree row",
            614.0,
            202.0,
            246.0,
            28.0,
            1,
            true,
        ),
        atlas_tree_row(
            "WorkbenchSceneLightItem",
            "Child row hover",
            614.0,
            234.0,
            246.0,
            28.0,
            2,
            false,
        ),
    ]);

    nodes.extend([
        atlas_label(
            "AtlasComplexTitle",
            "Complex Content",
            34.0,
            304.0,
            240.0,
            18.0,
            11.0,
            "",
        ),
        atlas_segmented(
            "WorkbenchSegmentedAtlas",
            &["All", "Selected", "Recent"],
            "Selected",
            34.0,
            338.0,
            252.0,
            34.0,
        ),
        atlas_table_row(
            "WorkbenchTableHeader",
            &["Name", "Type", "Size", "Modified"],
            34.0,
            390.0,
            370.0,
            28.0,
            false,
        ),
        atlas_table_row(
            "WorkbenchTableSelected",
            &["Box_01.mesh", "Mesh", "2.4 MB", "2m ago"],
            34.0,
            418.0,
            370.0,
            30.0,
            true,
        ),
        atlas_table_row(
            "WorkbenchTableRowAsset",
            &["M_Metal.zmat", "Material", "512 KB", "10m ago"],
            34.0,
            448.0,
            370.0,
            30.0,
            false,
        ),
        atlas_label(
            "AtlasProgressLabel",
            "Progress",
            34.0,
            492.0,
            70.0,
            16.0,
            10.0,
            "muted",
        ),
        atlas_progress("WorkbenchProgressAtlas", 0.64, 112.0, 496.0, 292.0, 12.0),
    ]);

    nodes.extend([
        atlas_label(
            "AtlasContainersTitle",
            "Containers, Images And Overlays",
            470.0,
            304.0,
            280.0,
            18.0,
            11.0,
            "",
        ),
        atlas_surface("AtlasImageCard", "inset", 470.0, 338.0, 132.0, 96.0),
        atlas_surface(
            "AtlasImagePreview",
            "asset-preview-visual",
            486.0,
            354.0,
            100.0,
            48.0,
        ),
        atlas_label(
            "AtlasImageLabel",
            "Image preview",
            492.0,
            414.0,
            94.0,
            18.0,
            10.0,
            "muted",
        ),
        atlas_surface("AtlasPopup", "popup", 624.0, 338.0, 220.0, 96.0),
        atlas_label(
            "AtlasPopupTitle",
            "Popup / Picker",
            640.0,
            354.0,
            160.0,
            18.0,
            11.0,
            "",
        ),
        atlas_field(
            "WorkbenchInputPopupFilter",
            "Filter rows...",
            640.0,
            380.0,
            184.0,
            28.0,
            "",
        ),
        atlas_list_row(
            "WorkbenchListPopupSelected",
            "Interactive option",
            640.0,
            414.0,
            184.0,
            28.0,
            "selected",
        ),
        atlas_tooltip(
            "WorkbenchTooltipAtlas",
            "Tooltip",
            "Host route",
            470.0,
            448.0,
            132.0,
            70.0,
        ),
        atlas_dialog(
            "WorkbenchDialogAtlas",
            "Unsaved asset",
            "Apply material changes?",
            624.0,
            446.0,
            220.0,
            104.0,
            "warning",
        ),
    ]);

    nodes.extend([
        atlas_status_signal("WorkbenchStatusReady", "Ready", 2.0, 578.0, 104.0, 42.0),
        atlas_status_signal(
            "WorkbenchStatusWarnings",
            "2 Warnings",
            110.0,
            578.0,
            128.0,
            42.0,
        ),
        atlas_status_signal(
            "WorkbenchStatusMessages",
            "0 Messages",
            238.0,
            578.0,
            132.0,
            42.0,
        ),
        atlas_status_chip(
            "WorkbenchStatusGrid",
            "Grid: 10 cm",
            634.0,
            585.0,
            92.0,
            28.0,
        ),
        atlas_status_chip("WorkbenchStatusSnap", "Snap: On", 734.0, 585.0, 82.0, 28.0),
        atlas_status_chip("WorkbenchStatusZoom", "100%", 824.0, 585.0, 52.0, 28.0),
    ]);

    nodes
}
