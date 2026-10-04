use super::*;

fn value(raw: &str) -> Value {
    Value::String(raw.to_owned())
}

#[test]
fn ended_rule_restores_baseline_and_removes_introduced_metadata() {
    let base = BTreeMap::from([("background".to_owned(), value("base"))]);
    let overrides = base.clone();
    let tokens = BTreeMap::from([("background".to_owned(), "token.base".to_owned())]);
    let rules = BTreeMap::new();
    let rule_tokens = BTreeMap::new();
    let mut metadata = UiTemplateNodeMetadata {
        attributes: BTreeMap::from([
            ("background".to_owned(), value("hover")),
            ("outline".to_owned(), value("introduced")),
            ("hovered".to_owned(), Value::Boolean(true)),
        ]),
        style_overrides: BTreeMap::from([
            ("background".to_owned(), value("hover")),
            ("outline".to_owned(), value("introduced")),
        ]),
        style_tokens: BTreeMap::from([
            ("background".to_owned(), "token.hover".to_owned()),
            ("outline".to_owned(), "token.outline".to_owned()),
        ]),
        ..Default::default()
    };
    let layers = || RuntimeStyleLayers {
        base_attributes: &base,
        base_overrides: Some(&overrides),
        base_tokens: Some(&tokens),
        rule_values: &rules,
        rule_tokens: &rule_tokens,
        active_states: &[],
    };

    let dirty = patch_runtime_style_attributes(&mut metadata, layers()).unwrap();
    assert_eq!(metadata.attributes, base);
    assert_eq!(metadata.style_overrides, overrides);
    assert_eq!(metadata.style_tokens, tokens);
    assert!(dirty.render);
    assert!(!dirty.text && !dirty.style);
    assert_eq!(
        patch_runtime_style_attributes(&mut metadata, layers()),
        None
    );
}

#[test]
fn retained_state_aliases_are_absent_and_only_active_canonical_keys_remain() {
    let base = BTreeMap::from([
        ("hover".to_owned(), Value::Boolean(true)),
        ("enabled".to_owned(), Value::Boolean(true)),
        ("custom".to_owned(), Value::Boolean(true)),
    ]);
    let rules = BTreeMap::from([
        ("focused".to_owned(), Value::Boolean(false)),
        ("open".to_owned(), Value::Boolean(true)),
    ]);
    let rule_tokens = BTreeMap::new();
    let active = ["hovered", "focus_visible", "popup_open"].map(str::to_owned);
    let mut metadata = UiTemplateNodeMetadata::default();

    let dirty = patch_runtime_style_attributes(
        &mut metadata,
        RuntimeStyleLayers {
            base_attributes: &base,
            base_overrides: None,
            base_tokens: None,
            rule_values: &rules,
            rule_tokens: &rule_tokens,
            active_states: &active,
        },
    )
    .unwrap();

    assert_eq!(
        metadata.attributes,
        ["custom", "hovered", "focus_visible", "popup_open"]
            .map(|key| (key.to_owned(), Value::Boolean(true)))
            .into_iter()
            .collect()
    );
    assert!(dirty.render);
    assert!(dirty.style); // The ordinary custom attribute was introduced.
    assert!(!dirty.text);
}

#[test]
fn protected_inline_override_and_nested_token_boundaries_keep_their_sources() {
    let base = BTreeMap::from([("background".to_owned(), value("base"))]);
    let overrides = BTreeMap::from([("background".to_owned(), value("inline"))]);
    let tokens = BTreeMap::from([
        ("background".to_owned(), "token.base".to_owned()),
        ("background.color".to_owned(), "token.nested".to_owned()),
        ("background[0]".to_owned(), "token.array".to_owned()),
        ("background_extra".to_owned(), "token.extra".to_owned()),
        ("slot.background".to_owned(), "token.slot".to_owned()),
    ]);
    let rules = BTreeMap::from([("background".to_owned(), value("hover"))]);
    let rule_tokens = BTreeMap::from([("background".to_owned(), "token.hover".to_owned())]);
    let mut metadata = UiTemplateNodeMetadata {
        attributes: base.clone(),
        style_overrides: overrides.clone(),
        style_tokens: tokens.clone(),
        ..Default::default()
    };

    patch_runtime_style_attributes(
        &mut metadata,
        RuntimeStyleLayers {
            base_attributes: &base,
            base_overrides: Some(&overrides),
            base_tokens: Some(&tokens),
            rule_values: &rules,
            rule_tokens: &rule_tokens,
            active_states: &[],
        },
    )
    .unwrap();

    assert_eq!(metadata.attributes["background"], value("hover"));
    assert_eq!(metadata.style_overrides["background"], value("inline"));
    assert_eq!(
        metadata.style_tokens,
        BTreeMap::from([
            ("background".to_owned(), "token.hover".to_owned()),
            ("background_extra".to_owned(), "token.extra".to_owned()),
            ("slot.background".to_owned(), "token.slot".to_owned()),
        ])
    );
}

#[test]
fn metadata_only_changes_are_render_only() {
    let base = BTreeMap::from([("label".to_owned(), value("unchanged"))]);
    let rules = BTreeMap::new();
    let rule_tokens = BTreeMap::new();
    let mut metadata = UiTemplateNodeMetadata {
        attributes: base.clone(),
        style_tokens: BTreeMap::from([("label".to_owned(), "stale.source".to_owned())]),
        ..Default::default()
    };

    let dirty = patch_runtime_style_attributes(
        &mut metadata,
        RuntimeStyleLayers {
            base_attributes: &base,
            base_overrides: None,
            base_tokens: None,
            rule_values: &rules,
            rule_tokens: &rule_tokens,
            active_states: &[],
        },
    )
    .unwrap();
    assert!(metadata.style_tokens.is_empty());
    assert!(dirty.render);
    assert!(!dirty.style && !dirty.text && !dirty.layout);
}
