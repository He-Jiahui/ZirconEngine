use super::*;

#[test]
fn ui_asset_stylesheet_rule_write_apis_reject_invalid_selectors_atomically() {
    let mut document = UiAssetLoader::load_toml_str(STYLE_WITH_RULE_IDS).unwrap();

    let mut invalid_replacement = document
        .style_rule("primary_button_hover")
        .expect("style rule")
        .clone();
    invalid_replacement.selector = "Button#".to_string();

    let replace_error = document
        .replace_style_rule("primary_button_hover", invalid_replacement)
        .expect_err("replacing with an invalid selector should fail");
    assert!(
        matches!(replace_error, UiAssetError::InvalidSelector(_)),
        "unexpected error: {replace_error:?}"
    );
    assert_eq!(
        document
            .style_rule("primary_button_hover")
            .map(|rule| rule.selector.as_str()),
        Some("Button.primary:hover")
    );

    let mut invalid_insert = document
        .remove_style_rule("primary_button_hover")
        .expect("style rule should be removed before reinserting");
    invalid_insert.selector = "Button#".to_string();
    let rule_count = document.stylesheets[0].rules.len();

    let insert_error = document
        .insert_style_rule("rule_id_sheet", 0, invalid_insert)
        .expect_err("inserting an invalid selector should fail");
    assert!(
        matches!(insert_error, UiAssetError::InvalidSelector(_)),
        "unexpected error: {insert_error:?}"
    );
    assert_eq!(document.stylesheets[0].rules.len(), rule_count);
    assert!(document.style_rule("primary_button_hover").is_none());
}

#[test]
fn ui_asset_stylesheets_can_be_inserted_at_stable_editor_positions() {
    let mut document = UiAssetLoader::load_toml_str(STYLE_WITH_RULE_IDS).unwrap();
    let removed = document
        .remove_style_sheet("secondary_sheet")
        .expect("stylesheet should be removed");

    let inserted_index = document.insert_style_sheet(0, removed).unwrap();
    assert_eq!(inserted_index, 0);
    assert_eq!(document.stylesheets[0].id, "secondary_sheet");

    let duplicate_error = document
        .insert_style_sheet(0, document.style_sheet("secondary_sheet").unwrap().clone())
        .expect_err("inserting a duplicate stylesheet id should fail");
    assert!(
        duplicate_error
            .to_string()
            .contains("duplicate stylesheet id secondary_sheet"),
        "unexpected error: {duplicate_error:?}"
    );
}

#[test]
fn ui_asset_stylesheets_can_be_replaced_atomically_for_editor_edits() {
    let mut document = UiAssetLoader::load_toml_str(STYLE_WITH_RULE_IDS).unwrap();
    let mut replacement = document
        .style_sheet("rule_id_sheet")
        .expect("stylesheet")
        .clone();
    replacement.rules.clear();

    let previous = document
        .replace_style_sheet("rule_id_sheet", replacement)
        .unwrap()
        .expect("stylesheet should be replaced");
    assert_eq!(previous.rules.len(), 2);
    assert_eq!(
        document
            .style_sheet("rule_id_sheet")
            .map(|stylesheet| stylesheet.rules.len()),
        Some(0)
    );
    assert!(document
        .replace_style_sheet(
            "missing_sheet",
            document.style_sheet("rule_id_sheet").unwrap().clone()
        )
        .unwrap()
        .is_none());

    let duplicate_replacement = document
        .style_sheet("secondary_sheet")
        .expect("secondary sheet")
        .clone();
    let duplicate_error = document
        .replace_style_sheet("rule_id_sheet", duplicate_replacement)
        .expect_err("replacing with a duplicate stylesheet id should fail");
    assert!(
        duplicate_error
            .to_string()
            .contains("duplicate stylesheet id secondary_sheet"),
        "unexpected error: {duplicate_error:?}"
    );
    assert_eq!(
        document
            .style_sheet("rule_id_sheet")
            .map(|stylesheet| stylesheet.rules.len()),
        Some(0)
    );
}

#[test]
fn ui_asset_stylesheet_write_apis_reject_invalid_selectors_atomically() {
    let mut document = UiAssetLoader::load_toml_str(STYLE_WITH_RULE_IDS).unwrap();

    let mut invalid_replacement = document
        .style_sheet("rule_id_sheet")
        .expect("stylesheet")
        .clone();
    invalid_replacement.rules[0].selector = "Button#".to_string();

    let replace_error = document
        .replace_style_sheet("rule_id_sheet", invalid_replacement)
        .expect_err("replacing a stylesheet with an invalid selector should fail");
    assert!(
        matches!(replace_error, UiAssetError::InvalidSelector(_)),
        "unexpected error: {replace_error:?}"
    );
    assert_eq!(
        document.stylesheets[0].rules[0].selector,
        "Button.primary:hover"
    );

    let mut invalid_insert = document
        .remove_style_sheet("secondary_sheet")
        .expect("stylesheet should be removed before reinserting");
    invalid_insert.rules[0].selector = "Button#".to_string();
    let stylesheet_count = document.stylesheets.len();

    let insert_error = document
        .insert_style_sheet(0, invalid_insert)
        .expect_err("inserting a stylesheet with an invalid selector should fail");
    assert!(
        matches!(insert_error, UiAssetError::InvalidSelector(_)),
        "unexpected error: {insert_error:?}"
    );
    assert_eq!(document.stylesheets.len(), stylesheet_count);
    assert!(document.style_sheet("secondary_sheet").is_none());
}

#[test]
fn ui_asset_stylesheets_can_be_replaced_atomically_for_editor_replay() {
    let mut document = UiAssetLoader::load_toml_str(STYLE_WITH_RULE_IDS).unwrap();
    let original_stylesheets = document.stylesheets.clone();
    let mut replacement = original_stylesheets.clone();
    replacement[0].rules[0].selector = "Button#".to_string();

    let replace_error = document
        .set_style_sheets(replacement)
        .expect_err("setting invalid stylesheets should fail");
    assert!(
        matches!(replace_error, UiAssetError::InvalidSelector(_)),
        "unexpected error: {replace_error:?}"
    );
    assert_eq!(document.stylesheets, original_stylesheets);

    let mut replacement = original_stylesheets.clone();
    replacement[0].rules.clear();
    assert!(document
        .set_style_sheets(replacement.clone())
        .expect("valid stylesheets should be accepted"));
    assert_eq!(document.stylesheets, replacement);
    assert!(!document
        .set_style_sheets(replacement)
        .expect("unchanged stylesheets should be a no-op"));
}

#[test]
fn ui_asset_stylesheets_replacement_rejects_duplicate_ids_atomically() {
    let mut document = UiAssetLoader::load_toml_str(STYLE_WITH_RULE_IDS).unwrap();
    let original_stylesheets = document.stylesheets.clone();
    let mut replacement = original_stylesheets.clone();
    replacement[1].id = replacement[0].id.clone();

    let replace_error = document
        .set_style_sheets(replacement)
        .expect_err("setting duplicate stylesheet ids should fail");
    assert!(
        replace_error
            .to_string()
            .contains("duplicate stylesheet id"),
        "unexpected error: {replace_error:?}"
    );
    assert_eq!(document.stylesheets, original_stylesheets);
}

#[test]
fn ui_asset_stylesheet_rules_can_be_moved_to_stable_editor_positions() {
    let mut document = UiAssetLoader::load_toml_str(STYLE_WITH_RULE_IDS).unwrap();

    assert!(document
        .move_style_rule("secondary_label_rule", "rule_id_sheet", 1)
        .unwrap());
    assert_eq!(document.stylesheets[0].rules.len(), 3);
    assert_eq!(
        document.stylesheets[0].rules[1].id.as_deref(),
        Some("secondary_label_rule")
    );
    assert!(document.stylesheets[1].rules.is_empty());

    assert!(!document
        .move_style_rule("missing_rule", "rule_id_sheet", 0)
        .unwrap());
    assert!(!document
        .move_style_rule("secondary_label_rule", "missing_sheet", 0)
        .unwrap());
}

#[test]
fn ui_asset_stylesheets_can_be_moved_to_stable_editor_positions() {
    let mut document = UiAssetLoader::load_toml_str(STYLE_WITH_RULE_IDS).unwrap();

    assert_eq!(document.move_style_sheet("secondary_sheet", 0), Some(0));
    assert_eq!(document.stylesheets[0].id, "secondary_sheet");
    assert_eq!(document.stylesheets[1].id, "rule_id_sheet");

    assert_eq!(document.move_style_sheet("secondary_sheet", 99), Some(1));
    assert_eq!(document.stylesheets[1].id, "secondary_sheet");
    assert_eq!(document.move_style_sheet("missing_sheet", 0), None);
}

#[test]
fn ui_asset_style_positions_follow_editor_reorder_operations() {
    let mut document = UiAssetLoader::load_toml_str(STYLE_WITH_RULE_IDS).unwrap();

    assert_eq!(document.style_sheet_index("rule_id_sheet"), Some(0));
    assert_eq!(document.style_sheet_index("secondary_sheet"), Some(1));
    let position = document
        .style_rule_position("secondary_label_rule")
        .expect("style rule position");
    assert_eq!(position.stylesheet_id, "secondary_sheet");
    assert_eq!(position.stylesheet_index, 1);
    assert_eq!(position.rule_index, 0);

    document
        .move_style_rule("secondary_label_rule", "rule_id_sheet", 1)
        .unwrap();
    document.move_style_sheet("secondary_sheet", 0);

    assert_eq!(document.style_sheet_index("rule_id_sheet"), Some(1));
    let position = document
        .style_rule_position("secondary_label_rule")
        .expect("style rule position after move");
    assert_eq!(position.stylesheet_id, "rule_id_sheet");
    assert_eq!(position.stylesheet_index, 1);
    assert_eq!(position.rule_index, 1);
    assert!(document.style_rule_position("missing_rule").is_none());
}
