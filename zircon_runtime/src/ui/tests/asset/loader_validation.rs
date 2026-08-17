use super::*;

#[test]
fn ui_asset_loader_rejects_duplicate_stable_style_rule_ids() {
    const STYLE_WITH_DUPLICATE_RULE_IDS: &str = r##"
[asset]
kind = "style"
id = "ui.theme.duplicate_rule_ids"
version = 1
display_name = "Duplicate Rule Ids"

[[stylesheets]]
id = "first_sheet"

[[stylesheets.rules]]
id = "primary_button"
selector = "Button.primary"
set = { self = { text = "Primary" } }

[[stylesheets]]
id = "second_sheet"

[[stylesheets.rules]]
id = "primary_button"
selector = "Button.primary:hover"
set = { self = { text = "Hover" } }
"##;

    let error = UiAssetLoader::load_toml_str(STYLE_WITH_DUPLICATE_RULE_IDS)
        .expect_err("duplicate stable style rule ids should be rejected");

    assert!(
        matches!(error, UiAssetError::InvalidDocument { .. }),
        "unexpected error: {error:?}"
    );
    assert!(
        error
            .to_string()
            .contains("duplicate style rule id primary_button"),
        "unexpected error: {error:?}"
    );
}

#[test]
fn ui_asset_loader_rejects_duplicate_stable_stylesheet_ids() {
    const STYLE_WITH_DUPLICATE_STYLESHEET_IDS: &str = r##"
[asset]
kind = "style"
id = "ui.theme.duplicate_stylesheet_ids"
version = 1
display_name = "Duplicate Stylesheet Ids"

[[stylesheets]]
id = "editor_base"

[[stylesheets.rules]]
selector = "Button.primary"
set = { self = { text = "Primary" } }

[[stylesheets]]
id = "editor_base"

[[stylesheets.rules]]
selector = "Button.primary:hover"
set = { self = { text = "Hover" } }
"##;

    let error = UiAssetLoader::load_toml_str(STYLE_WITH_DUPLICATE_STYLESHEET_IDS)
        .expect_err("duplicate stable stylesheet ids should be rejected");

    assert!(
        matches!(error, UiAssetError::InvalidDocument { .. }),
        "unexpected error: {error:?}"
    );
    assert!(
        error
            .to_string()
            .contains("duplicate stylesheet id editor_base"),
        "unexpected error: {error:?}"
    );
}

#[test]
fn ui_asset_loader_rejects_blank_stable_stylesheet_ids() {
    const STYLE_WITH_BLANK_STYLESHEET_ID: &str = r##"
[asset]
kind = "style"
id = "ui.theme.blank_stylesheet_id"
version = 1
display_name = "Blank Stylesheet Id"

[[stylesheets]]
id = " "

[[stylesheets.rules]]
selector = "Button.primary"
set = { self = { text = "Primary" } }
"##;

    let error = UiAssetLoader::load_toml_str(STYLE_WITH_BLANK_STYLESHEET_ID)
        .expect_err("blank stable stylesheet ids should be rejected");

    assert!(
        matches!(error, UiAssetError::InvalidDocument { .. }),
        "unexpected error: {error:?}"
    );
    assert!(
        error.to_string().contains("stylesheet id cannot be empty"),
        "unexpected error: {error:?}"
    );
}

#[test]
fn ui_asset_loader_rejects_invalid_style_rule_selectors() {
    const STYLE_WITH_INVALID_SELECTOR: &str = r##"
[asset]
kind = "style"
id = "ui.theme.invalid_selector"
version = 1
display_name = "Invalid Selector"

[[stylesheets]]
id = "editor_base"

[[stylesheets.rules]]
id = "bad_rule"
selector = "Button#"
set = { self = { text = "Bad" } }
"##;

    let error = UiAssetLoader::load_toml_str(STYLE_WITH_INVALID_SELECTOR)
        .expect_err("invalid style rule selectors should be rejected");

    assert!(
        matches!(error, UiAssetError::InvalidSelector(_)),
        "unexpected error: {error:?}"
    );
    assert!(
        error.to_string().contains("Button#"),
        "unexpected error: {error:?}"
    );
}

#[test]
fn ui_asset_loader_rejects_action_refs_with_both_command_and_route_targets() {
    const AMBIGUOUS_ACTION_TARGET_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.action_policy.ambiguous_target"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Button"
control_id = "AmbiguousTarget"

[[root.bindings]]
id = "Project/OpenOrRun"
event = "Click"

[root.bindings.action]
route = "Project.Open"
action = "runtime.play_mode.enter"
"##;

    let error = UiAssetLoader::load_toml_str(AMBIGUOUS_ACTION_TARGET_LAYOUT)
        .expect_err("an action ref with both targets should be rejected");

    assert!(matches!(error, UiAssetError::InvalidDocument { .. }));
    assert!(error.to_string().contains("Project/OpenOrRun"));
    assert!(error.to_string().contains("exactly one action target"));
}

#[test]
fn ui_asset_loader_rejects_command_actions_with_payload() {
    const COMMAND_ACTION_WITH_PAYLOAD_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.action_policy.command_payload"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Button"
control_id = "RunMode"

[[root.bindings]]
id = "Runtime/EnterPlayMode"
event = "Click"

[root.bindings.action]
action = "runtime.play_mode.enter"

[root.bindings.action.payload]
mode = "selected_viewport"
"##;

    let error = UiAssetLoader::load_toml_str(COMMAND_ACTION_WITH_PAYLOAD_LAYOUT)
        .expect_err("a command action with route payload should be rejected");

    assert!(matches!(error, UiAssetError::InvalidDocument { .. }));
    assert!(error.to_string().contains("Runtime/EnterPlayMode"));
    assert!(error.to_string().contains("must not carry payload"));
}

#[test]
fn ui_asset_loader_rejects_action_refs_without_a_non_empty_target() {
    const MISSING_ACTION_TARGET_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.action_policy.missing_target"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Button"
control_id = "MissingTarget"

[[root.bindings]]
id = "Project/MissingTarget"
event = "Click"

[root.bindings.action]
route = " "

[root.bindings.action.payload]
path = "samples/minimal"
"##;

    let error = UiAssetLoader::load_toml_str(MISSING_ACTION_TARGET_LAYOUT)
        .expect_err("an action ref without one non-empty target should be rejected");

    assert!(matches!(error, UiAssetError::InvalidDocument { .. }));
    assert!(error.to_string().contains("Project/MissingTarget"));
    assert!(error.to_string().contains("exactly one non-empty target"));
}

#[test]
fn ui_asset_loader_allows_route_actions_with_payload() {
    const ROUTE_ACTION_WITH_PAYLOAD_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.action_policy.route_payload"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Button"
control_id = "OpenProject"

[[root.bindings]]
id = "Project/Open"
event = "Click"

[root.bindings.action]
route = "Project.Open"

[root.bindings.action.payload]
path = "samples/minimal"
"##;

    let document = UiAssetLoader::load_toml_str(ROUTE_ACTION_WITH_PAYLOAD_LAYOUT)
        .expect("a route action may carry typed payload");
    let action = document.root.as_ref().unwrap().bindings[0]
        .action
        .as_ref()
        .expect("route action should remain authored");

    assert_eq!(action.route.as_deref(), Some("Project.Open"));
    assert!(action.action.is_none());
    assert_eq!(
        action.payload.get("path").and_then(toml::Value::as_str),
        Some("samples/minimal")
    );
}
