use super::*;

#[test]
fn ui_asset_stylesheet_rules_preserve_stable_rule_ids() {
    let document = UiAssetLoader::load_toml_str(STYLE_WITH_RULE_IDS).unwrap();
    let rules = &document.stylesheets[0].rules;
    assert_eq!(rules[0].id.as_deref(), Some("primary_button_hover"));
    assert_eq!(rules[1].id, None);

    let roundtrip = toml::to_string_pretty(&document).unwrap();
    assert!(roundtrip.contains("id = \"primary_button_hover\""));
    let reparsed = UiAssetLoader::load_toml_str(&roundtrip).unwrap();
    assert_eq!(
        reparsed.stylesheets[0].rules[0].id.as_deref(),
        Some("primary_button_hover")
    );
}

#[test]
fn ui_asset_stylesheet_rules_can_be_found_and_edited_by_stable_id() {
    let mut document = UiAssetLoader::load_toml_str(STYLE_WITH_RULE_IDS).unwrap();

    assert_eq!(
        document
            .style_rule("primary_button_hover")
            .map(|rule| rule.selector.as_str()),
        Some("Button.primary:hover")
    );
    assert!(document.style_rule("missing_rule").is_none());

    document
        .style_rule_mut("primary_button_hover")
        .unwrap()
        .selector = "Button.primary:pressed".to_string();

    assert_eq!(
        document
            .style_rule("primary_button_hover")
            .map(|rule| rule.selector.as_str()),
        Some("Button.primary:pressed")
    );
}

#[test]
fn ui_asset_stylesheets_can_be_found_and_edited_by_stable_id() {
    let mut document = UiAssetLoader::load_toml_str(STYLE_WITH_RULE_IDS).unwrap();

    assert_eq!(
        document
            .style_sheet("rule_id_sheet")
            .map(|stylesheet| stylesheet.rules.len()),
        Some(2)
    );
    assert!(document.style_sheet("missing_sheet").is_none());

    document.style_sheet_mut("rule_id_sheet").unwrap().id = "renamed_rule_id_sheet".to_string();

    assert!(document.style_sheet("rule_id_sheet").is_none());
    assert_eq!(
        document
            .style_sheet("renamed_rule_id_sheet")
            .map(|stylesheet| stylesheet.rules.len()),
        Some(2)
    );
}

#[test]
fn ui_asset_stylesheet_rules_can_be_renamed_without_breaking_id_uniqueness() {
    let mut document = UiAssetLoader::load_toml_str(STYLE_WITH_RULE_IDS).unwrap();

    assert!(document
        .rename_style_rule("primary_button_hover", "primary_button_pressed")
        .unwrap());
    assert!(document.style_rule("primary_button_hover").is_none());
    assert_eq!(
        document
            .style_rule("primary_button_pressed")
            .map(|rule| rule.selector.as_str()),
        Some("Button.primary:hover")
    );
    assert!(!document
        .rename_style_rule("missing_rule", "new_rule")
        .unwrap());

    let duplicate_error = document
        .rename_style_rule("primary_button_pressed", "secondary_label_rule")
        .expect_err("renaming to a duplicate style rule id should fail");
    assert!(
        duplicate_error
            .to_string()
            .contains("duplicate style rule id secondary_label_rule"),
        "unexpected error: {duplicate_error:?}"
    );

    let empty_error = document
        .rename_style_rule("primary_button_pressed", " ")
        .expect_err("renaming to an empty style rule id should fail");
    assert!(
        empty_error
            .to_string()
            .contains("style rule id cannot be empty"),
        "unexpected error: {empty_error:?}"
    );
}

#[test]
fn ui_asset_stylesheets_can_be_renamed_without_breaking_id_uniqueness() {
    let mut document = UiAssetLoader::load_toml_str(STYLE_WITH_RULE_IDS).unwrap();

    assert!(document
        .rename_style_sheet("rule_id_sheet", "renamed_rule_id_sheet")
        .unwrap());
    assert!(document.style_sheet("rule_id_sheet").is_none());
    assert_eq!(
        document
            .style_sheet("renamed_rule_id_sheet")
            .map(|stylesheet| stylesheet.rules.len()),
        Some(2)
    );
    assert!(!document
        .rename_style_sheet("missing_sheet", "new_sheet")
        .unwrap());

    let duplicate_error = document
        .rename_style_sheet("renamed_rule_id_sheet", "secondary_sheet")
        .expect_err("renaming to a duplicate stylesheet id should fail");
    assert!(
        duplicate_error
            .to_string()
            .contains("duplicate stylesheet id secondary_sheet"),
        "unexpected error: {duplicate_error:?}"
    );

    let empty_error = document
        .rename_style_sheet("renamed_rule_id_sheet", " ")
        .expect_err("renaming to an empty stylesheet id should fail");
    assert!(
        empty_error
            .to_string()
            .contains("stylesheet id cannot be empty"),
        "unexpected error: {empty_error:?}"
    );
}

#[test]
fn ui_asset_stylesheet_rules_can_be_removed_by_stable_id_for_editor_undo() {
    let mut document = UiAssetLoader::load_toml_str(STYLE_WITH_RULE_IDS).unwrap();

    let removed = document
        .remove_style_rule("primary_button_hover")
        .expect("style rule should be removed");
    assert_eq!(removed.id.as_deref(), Some("primary_button_hover"));
    assert_eq!(removed.selector, "Button.primary:hover");
    assert!(document.style_rule("primary_button_hover").is_none());
    assert_eq!(document.stylesheets[0].rules.len(), 1);

    assert!(document.remove_style_rule("missing_rule").is_none());
}

#[test]
fn ui_asset_stylesheets_can_be_removed_by_stable_id_for_editor_undo() {
    let mut document = UiAssetLoader::load_toml_str(STYLE_WITH_RULE_IDS).unwrap();

    let removed = document
        .remove_style_sheet("secondary_sheet")
        .expect("stylesheet should be removed");
    assert_eq!(removed.id, "secondary_sheet");
    assert_eq!(removed.rules.len(), 1);
    assert!(document.style_sheet("secondary_sheet").is_none());
    assert_eq!(document.stylesheets.len(), 1);

    assert!(document.remove_style_sheet("missing_sheet").is_none());
}

#[test]
fn ui_asset_stylesheet_rules_can_be_inserted_at_stable_editor_positions() {
    let mut document = UiAssetLoader::load_toml_str(STYLE_WITH_RULE_IDS).unwrap();
    let removed = document
        .remove_style_rule("primary_button_hover")
        .expect("style rule should be removed");

    assert!(document
        .insert_style_rule("rule_id_sheet", 0, removed)
        .unwrap());
    assert_eq!(
        document.stylesheets[0].rules[0].id.as_deref(),
        Some("primary_button_hover")
    );
    assert!(!document
        .insert_style_rule(
            "missing_sheet",
            0,
            document.style_rule("primary_button_hover").unwrap().clone()
        )
        .unwrap());

    let duplicate_error = document
        .insert_style_rule(
            "rule_id_sheet",
            0,
            document.style_rule("primary_button_hover").unwrap().clone(),
        )
        .expect_err("inserting a duplicate style rule id should fail");
    assert!(
        duplicate_error
            .to_string()
            .contains("duplicate style rule id primary_button_hover"),
        "unexpected error: {duplicate_error:?}"
    );
}

#[test]
fn ui_asset_stylesheet_rules_can_be_replaced_atomically_for_editor_edits() {
    let mut document = UiAssetLoader::load_toml_str(STYLE_WITH_RULE_IDS).unwrap();
    let mut replacement = document
        .style_rule("primary_button_hover")
        .expect("style rule")
        .clone();
    replacement.selector = "Button.primary:focus".to_string();

    let previous = document
        .replace_style_rule("primary_button_hover", replacement)
        .unwrap()
        .expect("style rule should be replaced");
    assert_eq!(previous.selector, "Button.primary:hover");
    assert_eq!(
        document
            .style_rule("primary_button_hover")
            .map(|rule| rule.selector.as_str()),
        Some("Button.primary:focus")
    );
    assert!(document
        .replace_style_rule(
            "missing_rule",
            document.style_rule("primary_button_hover").unwrap().clone()
        )
        .unwrap()
        .is_none());

    let duplicate_replacement = document
        .style_rule("secondary_label_rule")
        .expect("secondary rule")
        .clone();
    let duplicate_error = document
        .replace_style_rule("primary_button_hover", duplicate_replacement)
        .expect_err("replacing with a duplicate rule id should fail");
    assert!(
        duplicate_error
            .to_string()
            .contains("duplicate style rule id secondary_label_rule"),
        "unexpected error: {duplicate_error:?}"
    );
    assert_eq!(
        document
            .style_rule("primary_button_hover")
            .map(|rule| rule.selector.as_str()),
        Some("Button.primary:focus")
    );
}
