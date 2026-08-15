use zircon_runtime::ui::v2::UiV2AssetLoader;
use zircon_runtime_interface::ui::design_tokens::{EditorChromeTokens, EditorDesignTokens};

const EDITOR_TOKENS_ASSET: &str =
    include_str!("../../../../assets/ui/editor/theme/editor_tokens.zui");
const WORKBENCH_SKELETON_ASSET: &str =
    include_str!("../../../../assets/ui/editor/components/workbench/shell/workbench_skeleton.zui");
const WORKBENCH_STATUS_BAR_ASSET: &str = include_str!(
    "../../../../assets/ui/editor/components/workbench/shell/workbench_status_bar.zui"
);
const WORKBENCH_TOP_TOOLBAR_ASSET: &str = include_str!(
    "../../../../assets/ui/editor/components/workbench/shell/workbench_top_toolbar.zui"
);
const WORKBENCH_COMPONENT_DRAWER_ASSET: &str = include_str!(
    "../../../../assets/ui/editor/components/workbench/shell/workbench_component_drawer.zui"
);
const WORKBENCH_MAIN_BAND_ASSET: &str =
    include_str!("../../../../assets/ui/editor/components/workbench/shell/workbench_main_band.zui");
const WORKBENCH_SCENE_TREE_PANEL_ASSET: &str = include_str!(
    "../../../../assets/ui/editor/components/workbench/shell/workbench_scene_tree_panel.zui"
);
const WORKBENCH_INSPECTOR_PANEL_ASSET: &str = include_str!(
    "../../../../assets/ui/editor/components/workbench/shell/workbench_inspector_panel.zui"
);
const COMMAND_PALETTE_ASSET: &str = include_str!(
    "../../../../assets/ui/editor/components/workbench/floating/workbench_command_palette.zui"
);
const PREFERENCES_ASSET: &str = include_str!(
    "../../../../assets/ui/editor/components/workbench/floating/workbench_preferences.zui"
);
const WORKBENCH_VIEWPORT_PANEL_ASSET: &str = include_str!(
    "../../../../assets/ui/editor/components/workbench/shell/workbench_viewport_panel.zui"
);
const UI_ASSET_EDITOR_ASSET: &str =
    include_str!("../../../../assets/ui/editor/ui_asset_editor.zui");
const WORKBENCH_UI_ASSET_ACTION_BAR_ASSET: &str = include_str!(
    "../../../../assets/ui/editor/components/workbench/composites/chrome/workbench_ui_asset_action_bar.zui"
);
const WORKBENCH_MENU_POPUP_ASSET: &str =
    include_str!("../../../../assets/ui/editor/workbench_menu_popup.zui");
const WORKBENCH_DOCK_HEADER_ASSET: &str =
    include_str!("../../../../assets/ui/editor/workbench_dock_header.zui");
const WELCOME_ASSET: &str = include_str!("../../../../assets/ui/editor/welcome.zui");

mod ui_asset_editor;

#[test]
fn layout_shell_and_floating_assets_reference_editor_tokens_instead_of_hex_colors() {
    assert!(EDITOR_TOKENS_ASSET.contains("editor.surface.0"));
    assert!(EDITOR_TOKENS_ASSET.contains("editor.control.height.default"));
    assert!(EDITOR_TOKENS_ASSET.contains("editor.control.radius.pill"));
    assert!(
        EDITOR_TOKENS_ASSET.contains("editor.density.toolbar_action_width")
            && EDITOR_TOKENS_ASSET.contains("editor.density.toolbar_wide_action_width"),
        "editor token asset must name both toolbar action width tokens"
    );
    assert!(EDITOR_TOKENS_ASSET.contains("--left-drawer-width"));
    let typography_tokens = EDITOR_TOKENS_ASSET
        .split("[typography]")
        .nth(1)
        .and_then(|section| section.split("[controls]").next())
        .expect("editor token asset must contain typography and controls sections");
    assert!(
        typography_tokens.contains("font_smoothing = \"grayscale\""),
        "the typography asset must mirror the central font smoothing token"
    );

    for (asset_name, asset_source, required_tokens) in [
        ("workbench_skeleton.zui", WORKBENCH_SKELETON_ASSET, &[][..]),
        (
            "workbench_command_palette.zui",
            COMMAND_PALETTE_ASSET,
            &[
                "$editor.control.border_width",
                "$editor.control.radius.large",
            ][..],
        ),
        (
            "workbench_preferences.zui",
            PREFERENCES_ASSET,
            &[
                "$editor.control.border_width",
                "$editor.control.radius.large",
            ][..],
        ),
        (
            "workbench_status_bar.zui",
            WORKBENCH_STATUS_BAR_ASSET,
            &[
                "$editor.text.secondary",
                "$editor.semantic.success",
                "$editor.semantic.warning",
                "$editor.semantic.info",
                "$editor.density.gap.medium",
                "$editor.control.height.default",
                "$editor.control.height.dense",
            ][..],
        ),
        (
            "workbench_top_toolbar.zui",
            WORKBENCH_TOP_TOOLBAR_ASSET,
            &[
                "$editor.density.gap.small",
                "$editor.control.height.compact",
                "$editor.control.height.default",
                "$editor.control.border_width",
            ][..],
        ),
        (
            "workbench_component_drawer.zui",
            WORKBENCH_COMPONENT_DRAWER_ASSET,
            &[
                "$editor.accent",
                "$editor.accent.soft",
                "$editor.surface.recessed",
                "$editor.surface.selected",
                "$editor.text.primary",
                "$editor.text.secondary",
                "$editor.text.disabled",
                "$editor.semantic.error",
                "$editor.separator.soft",
                "$editor.density.gap.small",
                "$editor.density.gap.medium",
                "$editor.density.gap.large",
                "$editor.control.height.default",
                "$editor.control.height.dense",
                "$editor.control.radius.small",
                "$editor.control.border_width",
            ][..],
        ),
        (
            "workbench_inspector_panel.zui",
            WORKBENCH_INSPECTOR_PANEL_ASSET,
            &[
                "$editor.density.right_drawer_width",
                "$editor.density.gap.small",
                "$editor.density.gap.large",
                "$editor.typography.body.size",
                "$editor.typography.caption.size",
                "$editor.text.disabled",
                "$editor.text.secondary",
                "$editor.text.primary",
                "$editor.surface.0",
                "$editor.separator.soft",
                "$editor.control.height.default",
                "$editor.control.height.dense",
            ][..],
        ),
        (
            "workbench_scene_tree_panel.zui",
            WORKBENCH_SCENE_TREE_PANEL_ASSET,
            &[
                "$editor.density.left_drawer_width",
                "$editor.density.gap.medium",
                "$editor.control.height.default",
            ][..],
        ),
        (
            "workbench_main_band.zui",
            WORKBENCH_MAIN_BAND_ASSET,
            &[
                "$editor.density.left_drawer_width",
                "$editor.density.right_drawer_width",
            ][..],
        ),
    ] {
        assert!(
            asset_source.contains("res://ui/editor/theme/editor_tokens.zui"),
            "{asset_name} must import the editor token asset"
        );
        assert!(
            asset_source.contains("$editor.surface.")
                || asset_source.contains("$editor.text.")
                || asset_source.contains("$editor.border")
                || asset_source.contains("$editor.density."),
            "{asset_name} must use the canonical $editor token reference grammar"
        );
        for token in required_tokens {
            assert!(
                asset_source.contains(token),
                "{asset_name} must reference {token} instead of a raw control metric"
            );
        }
        assert!(
            !contains_hex_color(asset_source),
            "{asset_name} must not reintroduce naked hex colors"
        );
    }
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
fn workbench_dock_header_uses_the_shared_chrome_and_text_tokens() {
    assert!(
        WORKBENCH_DOCK_HEADER_ASSET.contains("res://ui/editor/theme/editor_tokens.zui"),
        "Workbench dock header must import the editor token asset"
    );
    for token in [
        "$editor.chrome.document_header.height",
        "$editor.control.height.compact",
        "$editor.control.radius.small",
        "$editor.typography.caption.size",
        "$editor.typography.body.size",
    ] {
        assert!(
            WORKBENCH_DOCK_HEADER_ASSET.contains(token),
            "Workbench dock header must use Workbench token {token}"
        );
    }
    for raw_metric in [
        "radius = 4.0",
        "font_size = 10.666667",
        "font_size = 13.333333",
        "min = 31.0, preferred = 31.0, max = 31.0",
        "min = 30.0, preferred = 30.0, max = 30.0",
    ] {
        assert!(
            !WORKBENCH_DOCK_HEADER_ASSET.contains(raw_metric),
            "Workbench dock header must not retain raw shared chrome metric {raw_metric}"
        );
    }

    assert!(
        WORKBENCH_DOCK_HEADER_ASSET
            .contains("surface_variant = \"panel\", radius = 0.0, border_width = 0.0"),
        "dock header bar must intentionally remain a square, borderless chrome band"
    );
    assert_eq!(
        WORKBENCH_DOCK_HEADER_ASSET
            .matches("border_width = 0.0")
            .count(),
        3,
        "dock header may use zero border width only for its borderless bar and two close icons"
    );
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
fn shell_drawer_assets_use_canonical_design_tokens_instead_of_inline_drawer_widths() {
    for (asset_name, asset_source, token_name, removed_width) in [
        (
            "workbench_main_band.zui",
            WORKBENCH_MAIN_BAND_ASSET,
            "$editor.density.left_drawer_width",
            "332.0",
        ),
        (
            "workbench_main_band.zui",
            WORKBENCH_MAIN_BAND_ASSET,
            "$editor.density.right_drawer_width",
            "404.0",
        ),
        (
            "workbench_scene_tree_panel.zui",
            WORKBENCH_SCENE_TREE_PANEL_ASSET,
            "$editor.density.left_drawer_width",
            "332.0",
        ),
        (
            "workbench_inspector_panel.zui",
            WORKBENCH_INSPECTOR_PANEL_ASSET,
            "$editor.density.right_drawer_width",
            "404.0",
        ),
    ] {
        assert!(
            asset_source.contains("res://ui/editor/theme/editor_tokens.zui"),
            "{asset_name} must import the editor token asset"
        );
        assert!(
            asset_source.contains(token_name),
            "{asset_name} must reference {token_name}"
        );
        assert!(
            !asset_source.contains(removed_width),
            "{asset_name} must not keep the old inline drawer width {removed_width}"
        );
    }

    assert!(
        !WORKBENCH_MAIN_BAND_ASSET.contains("$--"),
        "workbench_main_band.zui must not use the retired $-- token grammar"
    );
    assert!(
        !WORKBENCH_SCENE_TREE_PANEL_ASSET.contains("$--"),
        "workbench_scene_tree_panel.zui must not use the retired $-- token grammar"
    );
    assert!(
        !WORKBENCH_INSPECTOR_PANEL_ASSET.contains("$--"),
        "workbench_inspector_panel.zui must not use the retired $-- token grammar"
    );
}

#[test]
fn viewport_overlay_text_uses_central_typography_tokens() {
    assert!(
        WORKBENCH_VIEWPORT_PANEL_ASSET.contains("res://ui/editor/theme/editor_tokens.zui"),
        "workbench_viewport_panel.zui must import the editor token asset"
    );
    for token in [
        "$editor.typography.overlay.size",
        "$editor.typography.emphasis.weight",
    ] {
        assert!(
            WORKBENCH_VIEWPORT_PANEL_ASSET.contains(token),
            "workbench_viewport_panel.zui must reference {token}"
        );
    }
    assert!(
        !WORKBENCH_VIEWPORT_PANEL_ASSET.contains("font_size = 12.0"),
        "viewport overlay labels must not keep a local font size"
    );
    assert!(
        !WORKBENCH_VIEWPORT_PANEL_ASSET.contains("font_weight = 700"),
        "viewport overlay labels must not keep a local font weight"
    );
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
