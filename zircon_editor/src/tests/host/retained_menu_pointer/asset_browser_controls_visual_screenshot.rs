use std::path::{Path, PathBuf};

use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::{
    TemplateNodeFrameData, TemplatePaneNodeData, paint_template_nodes_for_test_with_background,
};

const ASSET_BROWSER_CONTROLS_SCREENSHOT: &str =
    "editor-components-asset-browser-controls-900x360.png";
const ASSET_BROWSER_CONTROLS_WIDTH: u32 = 900;
const ASSET_BROWSER_CONTROLS_HEIGHT: u32 = 360;
const ASSET_BROWSER_CONTROLS_BACKGROUND: [u8; 4] = [17, 20, 22, 255];

#[test]
fn asset_browser_controls_visual_paints_toolbar_chips_tabs_and_search_state_split() {
    let nodes = asset_browser_controls_nodes();
    assert!(
        nodes.iter().any(|node| {
            node.control_id.as_str() == "AssetBrowserKindTextureChip"
                && node.selected
                && node.checked
                && !node.focused
        }),
        "selected kind chip must not impersonate keyboard focus"
    );
    assert!(
        nodes.iter().any(|node| {
            node.control_id.as_str() == "AssetBrowserPreviewTabButton"
                && node.selected
                && node.checked
                && !node.focused
        }),
        "selected utility tab should carry selected state without focused=true"
    );
    assert!(
        nodes.iter().any(|node| {
            node.control_id.as_str() == "AssetBrowserMetadataTabButton"
                && node.focused
                && !node.selected
                && !node.checked
        }),
        "focused-only utility tab fixture must stay separate from selected state"
    );
    for (control_id, icon_name, selected) in [
        ("AssetBrowserViewModeListButton", "list-outline", false),
        ("AssetBrowserViewModeThumbButton", "grid-outline", true),
        (
            "LocateSelectedAsset",
            "editor_pages/asset_browser/navigation/search.svg",
            false,
        ),
    ] {
        assert!(
            nodes.iter().any(|node| {
                node.control_id.as_str() == control_id
                    && node.role.as_str() == "IconButton"
                    && node.icon_name.as_str() == icon_name
                    && node.icon_placement.as_str() == "icon_only"
                    && node.frame.width == 30.0
                    && node.frame.height == 30.0
                    && node.selected == selected
            }),
            "{control_id} must model the production icon-only direct toolbar action"
        );
    }

    let bytes = asset_browser_controls_bytes_from_nodes(nodes);
    let toolbar_panel = pixel_at(&bytes, 34, 94);

    assert!(
        distinct_pixel_count(
            &bytes,
            70,
            107,
            26,
            18,
            &[ASSET_BROWSER_CONTROLS_BACKGROUND, toolbar_panel],
        ) > 0,
        "search field should paint shell search icon pixels in the compact toolbar"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            100,
            107,
            126,
            18,
            &[ASSET_BROWSER_CONTROLS_BACKGROUND, toolbar_panel],
        ) > 0,
        "search field should paint placeholder text after the search icon"
    );

    let selected_tab_indicator = pixel_at(&bytes, 92, 209);
    let selected_chip_surface = pixel_at(&bytes, 360, 114);
    assert_ne!(
        selected_chip_surface, toolbar_panel,
        "selected toolbar chip should paint a framed selected surface"
    );
    assert_ne!(
        pixel_at(&bytes, 360, 137),
        selected_tab_indicator,
        "selected toolbar chip should not reuse the utility tab underline"
    );
    assert_ne!(
        pixel_at(&bytes, 330, 106),
        selected_tab_indicator,
        "selected toolbar chip should not paint a keyboard-focus frame"
    );

    assert_ne!(
        selected_tab_indicator, ASSET_BROWSER_CONTROLS_BACKGROUND,
        "selected utility tab should paint the Slate-like bottom indicator"
    );
    assert_ne!(
        pixel_at(&bytes, 270, 209),
        selected_tab_indicator,
        "focused-only utility tab should not paint the selected bottom indicator"
    );
    assert_ne!(
        pixel_at(&bytes, 270, 184),
        selected_tab_indicator,
        "focused-only utility tab should stay visually quiet instead of drawing a focus slab"
    );

    let import_surface = pixel_at(&bytes, 652, 114);
    assert!(
        distinct_pixel_count(
            &bytes,
            660,
            106,
            64,
            20,
            &[ASSET_BROWSER_CONTROLS_BACKGROUND, import_surface],
        ) > 0,
        "Quick Import command should paint readable retained button text"
    );
}

#[test]
#[ignore = "writes local Asset Browser controls screenshot artifact for visual review"]
fn capture_asset_browser_controls_visual_artifact() {
    let bytes = asset_browser_controls_bytes();
    let output_path = visual_layout_output_path(ASSET_BROWSER_CONTROLS_SCREENSHOT);

    image::save_buffer_with_format(
        &output_path,
        &bytes,
        ASSET_BROWSER_CONTROLS_WIDTH,
        ASSET_BROWSER_CONTROLS_HEIGHT,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("Asset Browser controls screenshot should be written as PNG");

    assert!(
        output_path.exists(),
        "expected visual screenshot at {}",
        output_path.display()
    );
}

fn asset_browser_controls_bytes() -> Vec<u8> {
    asset_browser_controls_bytes_from_nodes(asset_browser_controls_nodes())
}

fn asset_browser_controls_bytes_from_nodes(nodes: Vec<TemplatePaneNodeData>) -> Vec<u8> {
    paint_template_nodes_for_test_with_background(
        ASSET_BROWSER_CONTROLS_WIDTH,
        ASSET_BROWSER_CONTROLS_HEIGHT,
        ASSET_BROWSER_CONTROLS_BACKGROUND,
        model_rc(nodes),
    )
}

fn asset_browser_controls_nodes() -> Vec<TemplatePaneNodeData> {
    vec![
        surface("AssetBrowserControlsRoot", "shell", 0.0, 0.0, 900.0, 360.0),
        label(
            "AssetBrowserControlsTitle",
            "Asset Browser Controls",
            22.0,
            20.0,
            260.0,
            22.0,
            13.0,
            "",
        ),
        label(
            "AssetBrowserControlsSubtitle",
            "Search, toolbar chips, view modes, utility tabs and Quick Import use retained painters",
            22.0,
            42.0,
            710.0,
            18.0,
            10.0,
            "muted",
        ),
        surface("AssetBrowserToolbarPanel", "panel", 18.0, 76.0, 864.0, 78.0),
        surface(
            "AssetBrowserUtilityPanel",
            "panel",
            18.0,
            168.0,
            864.0,
            86.0,
        ),
        surface(
            "AssetBrowserPreviewPanel",
            "content-panel",
            18.0,
            268.0,
            864.0,
            46.0,
        ),
        field(
            "SearchEdited",
            "",
            "Search Assets",
            52.0,
            100.0,
            196.0,
            30.0,
        ),
        button(
            "AssetBrowserKindAllChip",
            "All",
            "text",
            "workbench.asset.kind_filter.set",
            264.0,
            100.0,
            54.0,
            30.0,
            ButtonState::Normal,
        ),
        button(
            "AssetBrowserKindTextureChip",
            "Texture",
            "text",
            "workbench.asset.kind_filter.set",
            326.0,
            100.0,
            82.0,
            30.0,
            ButtonState::Selected,
        ),
        button(
            "AssetBrowserKindMaterialChip",
            "Material",
            "text",
            "workbench.asset.kind_filter.set",
            416.0,
            100.0,
            88.0,
            30.0,
            ButtonState::Hovered,
        ),
        icon_button(
            "AssetBrowserViewModeListButton",
            "list-outline",
            "workbench.asset.view_mode.set",
            536.0,
            100.0,
            30.0,
            30.0,
            ButtonState::Normal,
        ),
        icon_button(
            "AssetBrowserViewModeThumbButton",
            "grid-outline",
            "workbench.asset.view_mode.set",
            570.0,
            100.0,
            30.0,
            30.0,
            ButtonState::Selected,
        ),
        icon_button(
            "LocateSelectedAsset",
            "editor_pages/asset_browser/navigation/search.svg",
            "workbench.asset.locate_selected_asset",
            604.0,
            100.0,
            30.0,
            30.0,
            ButtonState::Normal,
        ),
        button(
            "ImportModel",
            "Quick Import",
            "primary",
            "workbench.asset.import_model",
            640.0,
            100.0,
            116.0,
            30.0,
            ButtonState::Normal,
        ),
        button(
            "AssetBrowserPreviewTabButton",
            "Preview",
            "text",
            "workbench.asset.utility_tab.set",
            52.0,
            182.0,
            84.0,
            28.0,
            ButtonState::Selected,
        ),
        button(
            "AssetBrowserReferencesTabButton",
            "References",
            "text",
            "workbench.asset.utility_tab.set",
            146.0,
            182.0,
            112.0,
            28.0,
            ButtonState::Normal,
        ),
        button(
            "AssetBrowserMetadataTabButton",
            "Metadata",
            "text",
            "workbench.asset.utility_tab.set",
            268.0,
            182.0,
            104.0,
            28.0,
            ButtonState::Focused,
        ),
        button(
            "AssetBrowserPluginsTabButton",
            "Plugins",
            "text",
            "workbench.asset.utility_tab.set",
            382.0,
            182.0,
            88.0,
            28.0,
            ButtonState::Hovered,
        ),
        label(
            "AssetBrowserControlsLegendA",
            "selected chip: framed, no focus ring",
            52.0,
            282.0,
            240.0,
            18.0,
            10.0,
            "muted",
        ),
        label(
            "AssetBrowserControlsLegendB",
            "selected tab: underline; focused tab: no underline",
            336.0,
            282.0,
            330.0,
            18.0,
            10.0,
            "muted",
        ),
        label(
            "AssetBrowserControlsLegendC",
            "relative toolbar spacing stays within one compact row",
            674.0,
            282.0,
            174.0,
            18.0,
            10.0,
            "muted",
        ),
    ]
}

#[derive(Clone, Copy)]
enum ButtonState {
    Normal,
    Hovered,
    Focused,
    Selected,
}

fn button(
    control_id: &str,
    text: &str,
    variant: &str,
    action_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    state: ButtonState,
) -> TemplatePaneNodeData {
    let selected = matches!(state, ButtonState::Selected);
    TemplatePaneNodeData {
        control_id: control_id.into(),
        action_id: action_id.into(),
        role: "Button".into(),
        component_role: "button".into(),
        button_variant: variant.into(),
        text: text.into(),
        selected,
        checked: selected,
        hovered: matches!(state, ButtonState::Hovered),
        focused: matches!(state, ButtonState::Focused),
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn icon_button(
    control_id: &str,
    icon_name: &str,
    action_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    state: ButtonState,
) -> TemplatePaneNodeData {
    let selected = matches!(state, ButtonState::Selected);
    TemplatePaneNodeData {
        control_id: control_id.into(),
        action_id: action_id.into(),
        role: "IconButton".into(),
        component_role: "icon-button".into(),
        icon_name: icon_name.into(),
        icon_placement: "icon_only".into(),
        selected,
        checked: selected,
        hovered: matches!(state, ButtonState::Hovered),
        focused: matches!(state, ButtonState::Focused),
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn field(
    control_id: &str,
    value: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "InputField".into(),
        component_role: "search-field".into(),
        component_variant: "workbench-field".into(),
        value_text: value.into(),
        text: text.into(),
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
    let index = ((y as usize * ASSET_BROWSER_CONTROLS_WIDTH as usize) + x as usize) * 4;
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
            let index = ((py as usize * ASSET_BROWSER_CONTROLS_WIDTH as usize) + px as usize) * 4;
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
