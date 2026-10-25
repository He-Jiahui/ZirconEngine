use zircon_runtime::ui::v2::UiV2AssetLoader;
use zircon_runtime_interface::ui::design_tokens::{EditorChromeTokens, EditorDesignTokens};

const EDITOR_TOKENS_ASSET: &str =
    include_str!("../../../../assets/ui/editor/theme/editor_tokens.zui");
const WORKBENCH_SKELETON_ASSET: &str =
    include_str!("../../../../assets/ui/editor/workbench/primitives/feedback/skeleton.zui");
const UNREAL_LEVEL_EDITOR_ASSET: &str =
    include_str!("../../../../assets/ui/editor/unreal/level_editor_window.zui");
const UNREAL_EDITOR_SHELL_ASSET: &str =
    include_str!("../../../../assets/ui/editor/unreal/editor_shell.zui");
const UNREAL_STYLE_ASSET: &str = include_str!("../../../../assets/ui/editor/unreal/starship.zui");
const UNREAL_TOOLBAR_ASSET: &str =
    include_str!("../../../../assets/ui/editor/unreal/components/toolbar.zui");
const UNREAL_STATUS_BAR_ASSET: &str =
    include_str!("../../../../assets/ui/editor/unreal/components/status_bar.zui");
const UNREAL_CONTENT_DRAWER_ASSET: &str =
    include_str!("../../../../assets/ui/editor/unreal/components/content_drawer.zui");
const UNREAL_VIEWPORT_ASSET: &str =
    include_str!("../../../../assets/ui/editor/unreal/components/viewport.zui");
const UNREAL_OUTLINER_ASSET: &str =
    include_str!("../../../../assets/ui/editor/unreal/components/outliner.zui");
const UNREAL_DETAILS_ASSET: &str =
    include_str!("../../../../assets/ui/editor/unreal/components/details.zui");
const COMMAND_PALETTE_ASSET: &str =
    include_str!("../../../../assets/ui/editor/components/workbench/floating/command_palette.zui");
const PREFERENCES_ASSET: &str =
    include_str!("../../../../assets/ui/editor/components/workbench/floating/preferences.zui");
const UI_ASSET_EDITOR_ASSET: &str =
    include_str!("../../../../assets/ui/editor/ui_asset_editor.zui");
const WORKBENCH_UI_ASSET_ACTION_BAR_ASSET: &str =
    include_str!("../../../../assets/ui/editor/workbench/chrome/ui_asset_action_bar.zui");
const WORKBENCH_MENU_POPUP_ASSET: &str =
    include_str!("../../../../assets/ui/editor/workbench_menu_popup.zui");
const UNREAL_DOCK_HEADER_ASSET: &str =
    include_str!("../../../../assets/ui/editor/unreal/components/dock_header.zui");
const WELCOME_ASSET: &str = include_str!("../../../../assets/ui/editor/welcome.zui");

#[path = "../../layout/editor_shell_asset_contracts/ui_asset_editor.rs"]
mod ui_asset_editor;

#[test]
fn unreal_default_assets_mount_real_shell_regions_and_starship() {
    assert!(WORKBENCH_SKELETON_ASSET.contains("component = \"Skeleton\""));
    assert!(UNREAL_LEVEL_EDITOR_ASSET.contains("control_id = \"WorkbenchWindowRoot\""));
    assert!(UNREAL_LEVEL_EDITOR_ASSET.contains("res://ui/editor/unreal/starship.zui"));
    assert!(UNREAL_EDITOR_SHELL_ASSET.contains("component = \"UiHostWindow\""));
    assert!(UNREAL_EDITOR_SHELL_ASSET.contains("component = \"DocumentHost\""));
    assert!(UNREAL_EDITOR_SHELL_ASSET.contains("component = \"PaneSurface\""));
    for source in [
        UNREAL_TOOLBAR_ASSET,
        UNREAL_STATUS_BAR_ASSET,
        UNREAL_CONTENT_DRAWER_ASSET,
        UNREAL_VIEWPORT_ASSET,
        UNREAL_OUTLINER_ASSET,
        UNREAL_DETAILS_ASSET,
    ] {
        assert!(
            source.contains("res://ui/editor/unreal/starship.zui")
                || source.contains("res://ui/editor/theme/editor_tokens.zui"),
            "every Unreal default surface must import the approved Starship/editor style"
        );
    }
    assert!(!UNREAL_LEVEL_EDITOR_ASSET.contains("activity_rail"));
    assert!(!UNREAL_LEVEL_EDITOR_ASSET.contains("component_drawer"));
}

#[test]
fn chrome_token_asset_matches_the_runtime_interface_contract() {
    let chrome = EditorChromeTokens::workbench_dense();
    let chrome_values = token_asset_section(EDITOR_TOKENS_ASSET, "chrome");
    let chrome_names = token_asset_section(EDITOR_TOKENS_ASSET, "names.chrome");

    assert!(
        chrome_values.contains("top_bar_height"),
        "editor token asset must define the chrome token section"
    );
    assert!(
        chrome_names.contains("top_bar_height"),
        "editor token asset must name the chrome token section"
    );

    for (field, name, value) in [
        (
            "top_bar_height",
            "editor.chrome.top_bar.height",
            chrome.top_bar_height,
        ),
        (
            "host_bar_height",
            "editor.chrome.host_bar.height",
            chrome.host_bar_height,
        ),
        (
            "workbench_toolbar_height",
            "editor.chrome.workbench_toolbar.height",
            chrome.workbench_toolbar_height,
        ),
        (
            "workbench_toolbar_command_row_height",
            "editor.chrome.workbench_toolbar.command_row.height",
            chrome.workbench_toolbar_command_row_height,
        ),
        (
            "workbench_toolbar_popup_command_offset_y",
            "editor.chrome.workbench_toolbar.popup.command_offset_y",
            chrome.workbench_toolbar_popup_command_offset_y,
        ),
        (
            "workbench_toolbar_popup_module_offset_y",
            "editor.chrome.workbench_toolbar.popup.module_offset_y",
            chrome.workbench_toolbar_popup_module_offset_y,
        ),
        (
            "status_bar_height",
            "editor.chrome.status_bar.height",
            chrome.status_bar_height,
        ),
        (
            "panel_header_height",
            "editor.chrome.panel_header.height",
            chrome.panel_header_height,
        ),
        (
            "document_header_height",
            "editor.chrome.document_header.height",
            chrome.document_header_height,
        ),
        (
            "viewport_toolbar_height",
            "editor.chrome.viewport_toolbar.height",
            chrome.viewport_toolbar_height,
        ),
        (
            "activity_rail_width",
            "editor.chrome.activity_rail.width",
            chrome.activity_rail_width,
        ),
        (
            "separator_thickness",
            "editor.chrome.separator.thickness",
            chrome.separator_thickness,
        ),
        (
            "splitter_hit_size",
            "editor.chrome.splitter.hit_size",
            chrome.splitter_hit_size,
        ),
    ] {
        assert!(
            chrome_values.contains(&format!("{field} = {value:.1}")),
            "editor token asset must preserve the logical {field} value"
        );
        assert!(
            chrome_names.contains(&format!("{field} = \"{name}\"")),
            "editor token asset must name {field} as {name}"
        );
    }
}

#[test]
fn ui_asset_editor_uses_the_workbench_visual_token_contract() {
    let asset = UiV2AssetLoader::load_toml_str(UI_ASSET_EDITOR_ASSET)
        .expect("UI Asset Editor visual asset should parse as V2");
    assert_eq!(
        asset.imports.styles,
        vec!["res://ui/editor/theme/editor_tokens.zui"],
        "UI Asset Editor must use the Workbench token asset as its sole style import"
    );
    assert!(
        !UI_ASSET_EDITOR_ASSET.contains("editor_material.zui"),
        "UI Asset Editor must not reintroduce the legacy Material style sheet"
    );
    for primitive in [
        "workbench_button.zui#WorkbenchButton",
        "workbench_icon_button.zui#WorkbenchIconButton",
        "workbench_search_input.zui#WorkbenchSearchInput",
        "workbench_tree_row.zui#WorkbenchTreeRow",
        "workbench_caption.zui#WorkbenchCaption",
        "workbench_label.zui#WorkbenchLabel",
        "workbench_section_title.zui#WorkbenchSectionTitle",
    ] {
        assert!(
            UI_ASSET_EDITOR_ASSET.contains(primitive),
            "UI Asset Editor must compose its controls from canonical primitive {primitive}"
        );
    }
    assert!(
        !asset
            .imports
            .widgets
            .iter()
            .any(|widget| widget.contains("workbench_sample_grid.zui")),
        "the UI Asset Editor must reserve its canvas for the real preview projection, not a Blend Space grid primitive"
    );
    assert!(
        !UI_ASSET_EDITOR_ASSET.contains("$material_"),
        "UI Asset Editor must not retain a parallel Material visual token vocabulary"
    );
    for token in [
        "$editor.surface.0",
        "$editor.surface.1",
        "$editor.surface.recessed",
        "$editor.border",
        "$editor.text.primary",
        "$editor.text.secondary",
        "$editor.accent",
        "$editor.control.radius.control",
        "$editor.control.border_width",
        "$editor.typography.caption.size",
        "$editor.density.gap.tight",
        "$editor.density.gap.xsmall",
        "$editor.density.gap.small",
        "$editor.density.gap.regular",
        "$editor.density.gap.medium",
        "$editor.control.height.dense",
        "$editor.control.height.compact",
    ] {
        assert!(
            UI_ASSET_EDITOR_ASSET.contains(token),
            "UI Asset Editor must use Workbench token {token}"
        );
    }
    for raw_metric in [
        "radius = 6.0",
        "radius = 5.0",
        "radius = 4.0",
        "corner_radius = 6.0",
        "corner_radius = 5.0",
        "corner_radius = 4.0",
        "font_size = 11.0",
        "gap = 8.0",
        "gap = 6.0",
        "gap = 4.0",
        "gap = 3.0",
    ] {
        assert!(
            !UI_ASSET_EDITOR_ASSET.contains(raw_metric),
            "UI Asset Editor must not retain raw common chrome metric {raw_metric}"
        );
    }

    let density = EditorDesignTokens::workbench_dark().density;
    let density_values = token_asset_section(EDITOR_TOKENS_ASSET, "density");
    let density_names = token_asset_section(EDITOR_TOKENS_ASSET, "names.density");
    for (field, name, value) in [
        ("gap_tight", "editor.density.gap.tight", density.gap_tight),
        (
            "gap_regular",
            "editor.density.gap.regular",
            density.gap_regular,
        ),
    ] {
        assert!(
            density_values.contains(&format!("{field} = {value:.1}")),
            "editor token asset must preserve the {field} density value"
        );
        assert!(
            density_names.contains(&format!("{field} = \"{name}\"")),
            "editor token asset must name {field} as {name}"
        );
    }
}

#[test]
fn editor_token_asset_keeps_focus_distinct_from_persistent_selection() {
    let state_roles = token_asset_section(EDITOR_TOKENS_ASSET, "state_roles");

    assert!(
        state_roles.contains("focused = \"surface_1\""),
        "focused state must retain the normal surface and use a focus border"
    );
    assert!(
        state_roles.contains("selected = \"surface_selected\""),
        "selected state must retain the persistent selected surface"
    );
}

#[test]
fn workbench_menu_popup_uses_the_shared_chrome_and_text_tokens() {
    assert!(
        WORKBENCH_MENU_POPUP_ASSET.contains("res://ui/editor/theme/editor_tokens.zui"),
        "Workbench menu popup must import the editor token asset"
    );
    for token in [
        "$editor.control.radius.panel",
        "$editor.control.radius.small",
        "$editor.control.border_width",
        "$editor.typography.overlay.size",
        "$editor.typography.caption.size",
    ] {
        assert!(
            WORKBENCH_MENU_POPUP_ASSET.contains(token),
            "Workbench menu popup must use Workbench token {token}"
        );
    }
    for raw_metric in [
        "radius = 8.0",
        "radius = 4.0",
        "border_width = 1.0",
        "font_size = 12.0",
        "font_size = 11.0",
    ] {
        assert!(
            !WORKBENCH_MENU_POPUP_ASSET.contains(raw_metric),
            "Workbench menu popup must not retain raw common chrome metric {raw_metric}"
        );
    }
}

#[test]
fn unreal_dock_header_uses_starship_styles_and_keeps_tab_controls() {
    assert!(UNREAL_DOCK_HEADER_ASSET.contains("res://ui/editor/unreal/starship.zui"));
    for required in [
        "control_id = \"DockHeaderRoot\"",
        "control_id = \"DockHeaderBar\"",
        "control_id = \"DockTabStrip\"",
        "control_id = \"DockTab0\"",
        "control_id = \"DockTabClose0\"",
        "control_id = \"DockTab1\"",
        "control_id = \"DockTabClose1\"",
        "control_id = \"DockTab2\"",
    ] {
        assert!(UNREAL_DOCK_HEADER_ASSET.contains(required));
    }
    for token in [
        "$unreal_font_family",
        "$unreal_font_size",
        "$unreal_font_small_size",
        "$unreal_foreground_disabled",
        "$unreal_dropdown",
        "$unreal_recessed",
        "$unreal_foreground_hover",
        "$unreal_foreground_header",
    ] {
        assert!(UNREAL_DOCK_HEADER_ASSET.contains(token));
    }
    for selector in [
        "selector = \".unreal-dock-tab:hovered\"",
        "selector = \".unreal-dock-tab:selected\"",
    ] {
        assert!(UNREAL_DOCK_HEADER_ASSET.contains(selector));
    }
}

#[test]
fn welcome_controls_keep_the_compatibility_chain_but_use_workbench_states() {
    for style_asset in [
        "res://ui/theme/editor_base.zui",
        "res://ui/theme/editor_material.zui",
        "res://ui/editor/theme/editor_tokens.zui",
    ] {
        assert!(
            WELCOME_ASSET.contains(style_asset),
            "Welcome must retain the required style asset {style_asset}"
        );
    }
    assert!(
        !WELCOME_ASSET.contains("$material_"),
        "Welcome control visuals must resolve through Workbench tokens rather than Material token values"
    );
    for class_name in [
        "welcome-workbench-control",
        "welcome-workbench-field",
        "welcome-workbench-secondary-action",
        "welcome-workbench-primary-action",
    ] {
        assert!(
            WELCOME_ASSET.contains(class_name),
            "Welcome must assign and define the Workbench control class {class_name}"
        );
    }
    for selector in [
        ".welcome-workbench-control\"",
        ".welcome-workbench-control:hover",
        ".welcome-workbench-control:pressed",
        ".welcome-workbench-control:focus",
        ".welcome-workbench-control:disabled",
        ".welcome-workbench-primary-action\"",
        ".welcome-workbench-primary-action:hover",
        ".welcome-workbench-primary-action:pressed",
    ] {
        assert!(
            WELCOME_ASSET.contains(selector),
            "Welcome must define Workbench state selector {selector}"
        );
    }
    for token in [
        "$editor.surface.1",
        "$editor.surface.hover",
        "$editor.surface.3",
        "$editor.surface.disabled",
        "$editor.text.primary",
        "$editor.text.disabled",
        "$editor.accent",
        "$editor.accent.soft",
        "$editor.border",
        "$editor.border.disabled",
        "$editor.control.border_width",
        "$editor.control.radius.control",
        "$editor.typography.body.size",
    ] {
        assert!(
            WELCOME_ASSET.contains(token),
            "Welcome Workbench control rules must use {token}"
        );
    }
}

#[test]
fn unreal_main_band_keeps_the_75_25_dock_split_and_native_regions() {
    for required in [
        "control_id = \"WorkbenchWindowMainBandRegion\"",
        "linear_size = { rule = \"Stretch\", value = 1.0 }",
        "linear_size = { rule = \"Stretch\", value = 0.3333333333333333 }",
        "control_id = \"WorkbenchMainBandViewportPanel\"",
        "control_id = \"WorkbenchMainBandSceneTreePanel\"",
        "control_id = \"WorkbenchMainBandInspectorPanel\"",
    ] {
        assert!(
            UNREAL_LEVEL_EDITOR_ASSET.contains(required),
            "Unreal main band must retain `{required}`"
        );
    }
    assert!(!UNREAL_LEVEL_EDITOR_ASSET.contains("min = 0.0, preferred = 0.0, max = 0.0"));
}

#[test]
fn unreal_viewport_preserves_native_identity_and_interactive_surface() {
    for required in [
        "[components.UnrealLevelViewport]",
        "control_id = \"UnrealLevelViewport\"",
        "component = \"SceneViewViewport\"",
        "control_id = \"WorkbenchViewportSurface\"",
        "input_interactive = true",
        "input_focusable = true",
        "workbench.viewport.set_projection_mode",
    ] {
        assert!(
            UNREAL_VIEWPORT_ASSET.contains(required),
            "native viewport must retain `{required}`"
        );
    }
}

fn contains_hex_color(source: &str) -> bool {
    source
        .as_bytes()
        .windows(7)
        .any(|window| window[0] == b'#' && window[1..].iter().all(u8::is_ascii_hexdigit))
}

fn token_asset_section<'a>(asset: &'a str, section_name: &str) -> &'a str {
    let section_marker = format!("[{section_name}]");
    asset
        .split_once(&section_marker)
        .and_then(|(_, section)| section.split("\n[").next())
        .expect("editor token asset must contain the requested section")
}

fn stylesheet_rule_sources<'a>(asset: &'a str, selector: &str) -> Vec<&'a str> {
    let selector_marker = format!("selector = \"{selector}\"");
    asset
        .split("[[stylesheets.rules]]")
        .filter(|rule| rule.contains(&selector_marker))
        .collect()
}

fn node_children_source<'a>(asset: &'a str, node_name: &str) -> &'a str {
    let node_marker = format!("[nodes.{node_name}]");
    asset
        .split_once(&node_marker)
        .and_then(|(_, node)| node.split_once("children = ["))
        .and_then(|(_, children)| children.split("\n]").next())
        .expect("editor asset must contain the requested node children")
}

fn node_definition_source<'a>(asset: &'a str, node_name: &str) -> &'a str {
    let marker = format!("[nodes.{node_name}]");
    let after_node = asset
        .split_once(marker.as_str())
        .map(|(_, source)| source)
        .unwrap_or_else(|| panic!("missing node definition `{node_name}`"));
    after_node.split("\n[nodes.").next().unwrap_or(after_node)
}
