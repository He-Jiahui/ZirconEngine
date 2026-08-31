use std::fs;

use zircon_runtime::ui::v2::UiZuiAssetLoader;

use super::super::support::*;

#[test]
fn material_component_lab_shell_keeps_material_style_contract() {
    let lab_path = editor_asset("assets/ui/editor/material_component_lab.zui");
    let lab = UiZuiAssetLoader::load_zui_file(&lab_path).unwrap_or_else(|error| {
        panic!(
            "Material Component Lab should load as runtime UI v2 from {}: {error}",
            lab_path.display()
        )
    });
    let source = fs::read_to_string(&lab_path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", lab_path.display()));

    assert!(
        lab.imports
            .styles
            .iter()
            .any(|style| style == "res://ui/theme/editor_material.zui"),
        "Material Lab should import the shared dark Material v2 theme"
    );
    assert_node_class(&lab, "material_lab_root", "material-lab-shell");
    for node_id in [
        "appbar",
        "drawer",
        "side_panel",
        "data_display_title",
        "feedback_title",
        "inputs_title",
        "layout_title",
        "mui_x_title",
        "navigation_title",
        "surfaces_title",
        "utils_lab_title",
    ] {
        assert_node_class(&lab, node_id, "material-lab-card");
    }

    for token in [
        "selector = \".material-lab-shell\"",
        "selector = \".material-lab-card\"",
        "selector = \".material-lab-appbar-title\"",
        "selector = \".material-lab-appbar-chip\"",
        "selector = \".material-lab-appbar-primary\"",
        "selector = \".material-lab-appbar-status\"",
        "selector = \".material-lab-section-header\"",
        "selector = \".material-lab-section-title\"",
        "selector = \".material-lab-section-chip\"",
        "selector = \".material-lab-section-status\"",
        "selector = \".material-lab-side-panel\"",
        "selector = \".material-lab-side-title\"",
        "selector = \".material-lab-side-caption\"",
        "selector = \".material-lab-side-row\"",
        "selector = \".material-lab-side-chip\"",
        "selector = \".material-lab-side-info\"",
        "selector = \".material-lab-side-success\"",
        "selector = \".material-lab-side-error\"",
        "selector = \".material-lab-content-surface\"",
        "selector = \".material-lab-drawer-title\"",
        "selector = \".material-lab-nav-item\"",
        "selector = \".material-lab-nav-label\"",
        "selector = \".material-lab-nav-count\"",
        "selector = \".material-lab-nav-count-active\"",
        "selector = \".material-lab-nav-count-hover\"",
        "selector = \".material-lab-nav-active\"",
        "selector = \".material-lab-nav-hover\"",
        "selector = \".material-lab-meta-strip\"",
        "selector = \".material-lab-meta-chip\"",
        "selector = \".material-lab-meta-response\"",
        "selector = \".material-lab-meta-variant\"",
        "selector = \".material-lab-meta-layout\"",
        "selector = \".material-lab-state-strip\"",
        "selector = \".material-lab-state-pill\"",
        "selector = \".material-lab-state-hover\"",
        "selector = \".material-lab-state-pressed\"",
        "selector = \".material-lab-state-focus\"",
        "selector = \".material-lab-state-disabled\"",
        "selector = \".material-lab-state-selected\"",
        "selector = \".material-lab-state-open\"",
        "selector = \".material-lab-state-error\"",
        "background_color = \"#101418\"",
        "background_color = \"#202830\"",
        "background_color = \"#0b4050\"",
        "background_color = \"#1e3540\"",
        "background_color = \"#141b20\"",
        "background_color = \"#182128\"",
        "background_color = \"#173126\"",
        "background_color = \"#301b1e\"",
        "background_color = \"#151c22\"",
        "background_color = \"#121b20\"",
        "background_color = \"#142229\"",
        "foreground_color = \"#8fd7e8\"",
        "foreground_color = \"#e6f1f4\"",
        "foreground_color = \"#c8d7dd\"",
        "foreground_color = \"#d8f7fb\"",
        "foreground_color = \"#b9eaf2\"",
        "foreground_color = \"#ffb4ab\"",
        "foreground_color = \"#9de7be\"",
        "foreground_color = \"#ffe08a\"",
        "border_color = \"#4b626d\"",
        "border_color = \"#2aaac0\"",
        "border_color = \"#3f7080\"",
        "border_color = \"#344954\"",
        "border_color = \"#33764f\"",
        "border_color = \"#2c6575\"",
        "border_color = \"#7f6820\"",
        "border_color = \"#2d5b68\"",
        "border_color = \"#36c7d9\"",
        "border_color = \"#e56767\"",
        "radius = 12.0",
        "radius = 10.0",
        "radius = 999.0",
    ] {
        assert!(
            source.contains(token),
            "Material Lab style contract should keep `{token}`"
        );
    }
}

#[test]
fn material_component_lab_profile_capture_scenarios_open_the_lab_window() {
    let script_path = workspace_root().join("tools/ui-profile-capture.ps1");
    let source = fs::read_to_string(&script_path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", script_path.display()));

    for scenario in [
        "material_lab_startup",
        "material_lab_hover",
        "material_lab_click",
    ] {
        assert!(
            source.contains(scenario),
            "profile capture script should define `{scenario}`"
        );
    }
    assert!(source.contains("--builtin-view"));
    assert!(source.contains("editor.material_component_lab"));
    assert!(source.contains("Expand-CaptureScenarioNames"));
    assert!(source.contains("$name -split \",\""));
    assert!(source.contains("Resolve-InteractionScenarioName"));
    assert!(source.contains("UI scenario evidence ($evidenceScenario):"));
    assert!(source.contains("has no hover redraw batch"));
    assert!(source.contains("dependency-bound"));
    assert!(
        source.contains("$templateControlsOnly = $normalizedScenario -eq \"material_lab_click\"")
    );
    assert!(source.contains("-TemplateControlsOnly:$templateControlsOnly"));
}
