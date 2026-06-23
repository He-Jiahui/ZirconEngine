use std::fs;
use std::path::Path;

use toml::{Table, Value};
use zircon_editor::ui::workbench::{
    FloatingLayer, FloatingWindow, FloatingWindowContentLayout, FloatingWindowInteractionMode,
    FloatingWindowKind, FloatingWindowPlacement, FLOATING_WINDOW_DESIGN_CONTRACTS,
};

fn asset_source(relative: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

fn asset_document(source: &str) -> Value {
    let table = source
        .parse::<Table>()
        .unwrap_or_else(|error| panic!("asset should parse as TOML: {error}"))
        .into_iter()
        .collect();
    Value::Table(table)
}

fn value_at<'a>(value: &'a Value, path: &[&str]) -> &'a Value {
    let mut current = value;
    for key in path {
        current = current
            .get(*key)
            .unwrap_or_else(|| panic!("missing TOML path `{}`", path.join(".")));
    }
    current
}

fn string_at<'a>(value: &'a Value, path: &[&str]) -> &'a str {
    value_at(value, path)
        .as_str()
        .unwrap_or_else(|| panic!("TOML path `{}` should be a string", path.join(".")))
}

fn number_at(value: &Value, path: &[&str]) -> f64 {
    let value = value_at(value, path);
    value
        .as_float()
        .or_else(|| value.as_integer().map(|number| number as f64))
        .unwrap_or_else(|| panic!("TOML path `{}` should be numeric", path.join(".")))
}

fn child_nodes<'a>(value: &'a Value, node: &str) -> Vec<&'a str> {
    value_at(value, &["nodes", node, "children"])
        .as_array()
        .unwrap_or_else(|| panic!("node `{node}` should have children"))
        .iter()
        .map(|child| string_at(child, &["node"]))
        .collect()
}

fn assert_low_chrome(value: &Value, node: &str) {
    assert_eq!(
        number_at(value, &["nodes", node, "style", "self", "border", "width"]),
        1.0,
        "{node} must use the 1px workbench border contract"
    );
    assert!(
        number_at(value, &["nodes", node, "style", "self", "border", "radius"]) <= 8.0,
        "{node} must keep low workbench corner radius"
    );
    assert!(
        string_at(
            value,
            &["nodes", node, "style", "self", "background", "color"]
        )
        .starts_with("editor.surface."),
        "{node} must use editor surface tokens"
    );
    assert_eq!(
        string_at(value, &["nodes", node, "style", "self", "border", "color"]),
        "editor.border",
        "{node} must use the editor border token"
    );
}

fn contains_hex_color(source: &str) -> bool {
    source
        .as_bytes()
        .windows(7)
        .any(|window| window[0] == b'#' && window[1..].iter().all(u8::is_ascii_hexdigit))
}

#[test]
fn floating_window_design_contracts_cover_the_reference_window_roles() {
    assert_eq!(FLOATING_WINDOW_DESIGN_CONTRACTS.len(), 3);

    let command = FloatingWindow::command_palette();
    let command_contract = command.design_contract();
    assert_eq!(command_contract.kind, FloatingWindowKind::CommandPalette);
    assert_eq!(command_contract.layer, FloatingLayer::TopOverlay);
    assert_eq!(
        command_contract.placement,
        FloatingWindowPlacement::TopCenter
    );
    assert_eq!(
        command_contract.content_layout,
        FloatingWindowContentLayout::CommandPalette
    );
    assert_eq!(
        command_contract.interaction_mode,
        FloatingWindowInteractionMode::KeyboardDriven
    );
    assert!(!command_contract.modal);

    let preferences = FloatingWindow::preferences();
    let preferences_contract = preferences.design_contract();
    assert_eq!(preferences_contract.kind, FloatingWindowKind::Preferences);
    assert_eq!(preferences_contract.layer, FloatingLayer::ModalOverlay);
    assert_eq!(
        preferences_contract.placement,
        FloatingWindowPlacement::WorkbenchCenter
    );
    assert_eq!(
        preferences_contract.content_layout,
        FloatingWindowContentLayout::NavigationContent
    );
    assert!(preferences_contract.modal);

    let detached = FloatingWindow::detached_editor(
        "res://ui/editor/components/workbench/modules/core/scene/workbench_scene_workspace.zui",
    );
    let detached_contract = detached.design_contract();
    assert_eq!(detached_contract.kind, FloatingWindowKind::DetachedEditor);
    assert_eq!(detached_contract.layer, FloatingLayer::NativeDetached);
    assert_eq!(
        detached_contract.content_layout,
        FloatingWindowContentLayout::PageTemplate
    );
    assert_eq!(
        detached_contract.interaction_mode,
        FloatingWindowInteractionMode::DetachedEditorPage
    );
    assert!(!detached_contract.modal);
}

#[test]
fn floating_assets_use_tokenized_flat_workbench_chrome() {
    for (name, source) in [
        (
            "command_palette.zui",
            asset_source("assets/ui/editor/components/workbench/floating/command_palette.zui"),
        ),
        (
            "preferences.zui",
            asset_source("assets/ui/editor/components/workbench/floating/preferences.zui"),
        ),
    ] {
        assert!(
            source.contains("res://ui/editor/theme/editor_tokens.v2.ui.toml"),
            "{name} must import the editor token asset"
        );
        assert!(
            !contains_hex_color(&source),
            "{name} must not contain naked hex colors"
        );
        for forbidden in ["gradient", "shadow", "glow", "blur"] {
            assert!(
                !source.to_ascii_lowercase().contains(forbidden),
                "{name} must not use decorative `{forbidden}` effects"
            );
        }
    }
}

#[test]
fn command_palette_matches_top_center_keyboard_overlay_contract() {
    let source = asset_source("assets/ui/editor/components/workbench/floating/command_palette.zui");
    let document = asset_document(&source);

    assert_eq!(
        string_at(
            &document,
            &["components", "WorkbenchCommandPalette", "root"]
        ),
        "palette"
    );
    assert_low_chrome(&document, "palette");
    assert_eq!(
        string_at(
            &document,
            &["nodes", "palette", "layout", "container", "kind"]
        ),
        "VerticalBox"
    );
    assert_eq!(
        child_nodes(&document, "palette"),
        vec!["search", "first_result"]
    );
    assert_eq!(
        string_at(&document, &["nodes", "search", "component"]),
        "WorkbenchSearchInput"
    );
    assert_eq!(
        number_at(
            &document,
            &["nodes", "search", "layout", "height", "preferred"]
        ),
        32.0
    );
    assert_eq!(
        number_at(
            &document,
            &["nodes", "first_result", "layout", "height", "preferred"]
        ),
        28.0
    );
    assert!(
        number_at(
            &document,
            &["nodes", "palette", "layout", "width", "preferred"]
        ) >= 640.0
    );
    assert!(number_at(&document, &["nodes", "palette", "layout", "height", "max"]) <= 520.0);
}

#[test]
fn preferences_matches_modal_navigation_content_contract() {
    let source = asset_source("assets/ui/editor/components/workbench/floating/preferences.zui");
    let document = asset_document(&source);

    assert_eq!(
        string_at(&document, &["components", "WorkbenchPreferences", "root"]),
        "preferences"
    );
    assert_low_chrome(&document, "preferences");
    assert_low_chrome(&document, "navigation");
    assert_low_chrome(&document, "content");
    assert_eq!(
        string_at(
            &document,
            &["nodes", "preferences", "layout", "container", "kind"]
        ),
        "HorizontalBox"
    );
    assert_eq!(
        child_nodes(&document, "preferences"),
        vec!["navigation", "content"]
    );
    assert!(
        number_at(
            &document,
            &["nodes", "navigation", "layout", "width", "preferred"]
        ) >= 224.0
    );
    assert_eq!(
        string_at(
            &document,
            &["nodes", "content", "layout", "width", "stretch"]
        ),
        "Stretch"
    );
    assert_eq!(
        string_at(&document, &["nodes", "title", "props", "text"]),
        "Preferences"
    );
}
