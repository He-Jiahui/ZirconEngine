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
