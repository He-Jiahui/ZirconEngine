#[test]
fn editor_review_theme_matches_the_product_settings_authority() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets/ui/editor/theme/editor_tokens.zui");
    let source: Value = toml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    verify(&source, &EditorDesignTokens::workbench_dark()).unwrap();
}

#[test]
fn editor_review_theme_rejects_silent_font_and_control_drift() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets/ui/editor/theme/editor_tokens.zui");
    let source: Value = toml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let mut tokens = EditorDesignTokens::workbench_dark();
    tokens.typography.body_size += 1.0;
    assert!(verify(&source, &tokens)
        .unwrap_err()
        .contains("editor.typography.body.size"));
    let mut tokens = EditorDesignTokens::workbench_dark();
    tokens.controls.control_radius += 1.0;
    assert!(verify(&source, &tokens)
        .unwrap_err()
        .contains("editor.control.radius.control"));
}

#[test]
fn review_theme_compares_hex_color_values_without_ignoring_actual_drift() {
    assert!(matches_value(&Value::String("#0070E0".into()), &Value::String("#0070e0".into())));
    assert!(!matches_value(&Value::String("#0070E0".into()), &Value::String("#0071e0".into())));
    assert!(!matches_value(&Value::String("#00000073".into()), &Value::String("#00000074".into())));
    assert!(!matches_value(&Value::String("Roboto".into()), &Value::String("roboto".into())));
}
