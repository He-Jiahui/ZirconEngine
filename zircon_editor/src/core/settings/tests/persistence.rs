use super::*;

#[test]
fn invalid_keys_and_duplicate_definitions_are_rejected() {
    assert!(SettingsKey::parse("Editor.scene.grid_step").is_err());
    assert!(SettingsKey::parse("editor..grid_step").is_err());

    let definition = project_grid_setting();
    let mut registry = SettingsRegistry::default();
    registry.register(definition.clone()).unwrap();
    assert!(matches!(
        registry.register(definition),
        Err(SettingsError::DuplicateDefinition(_))
    ));
}

#[test]
fn definition_constructor_rejects_invalid_schema() {
    assert!(SettingDefinition::new(
        key("editor.invalid.float"),
        SettingsScope::User,
        SettingSchema::Float {
            minimum: f64::NAN,
            maximum: 1.0,
            step: 0.1,
        },
        SettingValue::Float(0.5),
        false,
        presentation(
            "settings.editor.invalid.float.label",
            "settings.editor.invalid.float.description",
            &["settings.category.editor"],
        ),
    )
    .is_err());

    assert!(SettingDefinition::new(
        key("editor.invalid.range"),
        SettingsScope::Project,
        SettingSchema::Int {
            minimum: 10,
            maximum: 1,
            step: 1,
        },
        SettingValue::Int(5),
        false,
        presentation(
            "settings.editor.invalid.range.label",
            "settings.editor.invalid.range.description",
            &["settings.category.editor"],
        ),
    )
    .is_err());

    assert!(SettingDefinition::new(
        key("editor.invalid.step"),
        SettingsScope::User,
        SettingSchema::Int {
            minimum: 0,
            maximum: 10,
            step: 0,
        },
        SettingValue::Int(5),
        false,
        presentation(
            "settings.editor.invalid.step.label",
            "settings.editor.invalid.step.description",
            &["settings.category.editor"],
        ),
    )
    .is_err());
}

#[test]
fn numeric_schema_steps_are_bounded_and_float_quantized() {
    let integer = SettingSchema::Int {
        minimum: i64::MIN + 1,
        maximum: i64::MAX - 1,
        step: 4,
    };
    assert_eq!(
        integer.stepped_numeric_value(
            &SettingValue::Int(i64::MAX - 2),
            SettingNumericStepDirection::Increment,
        ),
        Some(SettingValue::Int(i64::MAX - 1))
    );
    assert_eq!(
        integer.stepped_numeric_value(
            &SettingValue::Int(i64::MIN + 2),
            SettingNumericStepDirection::Decrement,
        ),
        Some(SettingValue::Int(i64::MIN + 1))
    );

    let float = SettingSchema::Float {
        minimum: 0.0,
        maximum: 0.3,
        step: 0.1,
    };
    assert_eq!(
        float.stepped_numeric_value(
            &SettingValue::Float(0.1),
            SettingNumericStepDirection::Increment,
        ),
        Some(SettingValue::Float(0.2))
    );
    assert_eq!(
        float.stepped_numeric_value(
            &SettingValue::Float(0.3),
            SettingNumericStepDirection::Increment,
        ),
        Some(SettingValue::Float(0.3))
    );
}

#[test]
fn color_schema_steps_one_typed_channel_with_saturating_bounds() {
    let color = SettingSchema::Color { channel_step: 16 };
    assert_eq!(
        color.stepped_color_value(
            &SettingValue::Color([250, 16, 32, 0]),
            SettingColorChannel::Red,
            SettingNumericStepDirection::Increment,
        ),
        Some(SettingValue::Color([255, 16, 32, 0]))
    );
    assert_eq!(
        color.stepped_color_value(
            &SettingValue::Color([250, 16, 32, 0]),
            SettingColorChannel::Alpha,
            SettingNumericStepDirection::Decrement,
        ),
        Some(SettingValue::Color([250, 16, 32, 0]))
    );
}

#[test]
fn color_schema_rejects_a_zero_channel_step() {
    assert!(SettingDefinition::new(
        key("editor.invalid.color_step"),
        SettingsScope::User,
        SettingSchema::Color { channel_step: 0 },
        SettingValue::Color([0, 0, 0, 255]),
        false,
        presentation(
            "settings.editor.invalid.color_step.label",
            "settings.editor.invalid.color_step.description",
            &["settings.category.editor"],
        ),
    )
    .is_err());
}

#[test]
fn scope_and_schema_boundaries_remain_explicit() {
    assert!(SettingsScope::User.allows_write(SettingsScope::User));
    assert!(SettingsScope::User.allows_write(SettingsScope::Session));
    assert!(!SettingsScope::User.allows_write(SettingsScope::Project));
    assert!(SettingsScope::Project.allows_write(SettingsScope::User));
    assert!(SettingsScope::Project.allows_write(SettingsScope::Project));
    assert!(SettingsScope::Project.allows_write(SettingsScope::Session));
    assert!(!SettingsScope::Session.allows_write(SettingsScope::User));
    assert!(!SettingsScope::Session.allows_write(SettingsScope::Project));
    assert!(SettingsScope::Session.allows_write(SettingsScope::Session));

    let string = SettingDefinition::new(
        key("editor.caption"),
        SettingsScope::User,
        SettingSchema::String { maximum_bytes: 3 },
        SettingValue::String("abc".to_string()),
        false,
        presentation(
            "settings.editor.caption.label",
            "settings.editor.caption.description",
            &["settings.category.editor"],
        ),
    )
    .unwrap();
    let chord = SettingDefinition::new(
        key("editor.shortcut"),
        SettingsScope::User,
        SettingSchema::Chord,
        SettingValue::Chord("Ctrl+S".parse::<EditorKeyChord>().unwrap()),
        false,
        presentation(
            "settings.editor.shortcut.label",
            "settings.editor.shortcut.description",
            &["settings.category.editor"],
        ),
    )
    .unwrap();
    let mut registry = SettingsRegistry::default();
    let string_key = string.key.clone();
    registry.register(string).unwrap();
    registry.register(chord).unwrap();

    assert!(matches!(
        registry.set(
            SettingsScope::User,
            &string_key,
            SettingValue::String("four".to_string())
        ),
        Err(SettingsError::InvalidValue { .. })
    ));
    assert!(matches!(
        registry.set(
            SettingsScope::User,
            &key("editor.shortcut"),
            SettingValue::Chord(EditorKeyChord::new("Ctrl"))
        ),
        Err(SettingsError::InvalidValue { .. })
    ));
    assert!(matches!(
        registry.clear(SettingsScope::User, &key("editor.unknown")),
        Err(SettingsError::UnknownKey(_))
    ));
}

#[test]
fn design_tokens_are_a_strongly_typed_user_setting() {
    let mut registry = settings_registry_with_defaults();
    let key = key(EDITOR_DESIGN_TOKENS_KEY);
    let mut tokens = EditorDesignTokens::workbench_dark();
    tokens.id = "zircon.editor.tests.custom".to_string();
    tokens.density.row_height = 31.0;

    registry
        .set(
            SettingsScope::User,
            &key,
            SettingValue::DesignTokens(tokens.clone()),
        )
        .unwrap();
    assert_eq!(
        registry.resolve(&key).unwrap(),
        &SettingValue::DesignTokens(tokens)
    );
    assert!(matches!(
        registry.set(
            SettingsScope::User,
            &key,
            SettingValue::String("wrong".to_string())
        ),
        Err(SettingsError::InvalidValue { .. })
    ));
}

#[test]
fn command_palette_mru_is_a_bounded_session_only_setting() {
    let mut registry = settings_registry_with_defaults();
    let key = key(EDITOR_COMMAND_PALETTE_MRU_KEY);
    let mru = EditorCommandPaletteMru::new([
        EditorOperationPath::parse("file.project.open")
            .expect("the built-in command id should be valid"),
        EditorOperationPath::parse("file.project.save")
            .expect("the built-in command id should be valid"),
    ])
    .expect("the bounded command history should be valid");

    assert!(matches!(
        registry.set(
            SettingsScope::User,
            &key,
            SettingValue::CommandPaletteMru(mru.clone()),
        ),
        Err(SettingsError::ScopeNotAllowed { .. })
    ));
    registry
        .set(
            SettingsScope::Session,
            &key,
            SettingValue::CommandPaletteMru(mru.clone()),
        )
        .expect("the Session layer should own command palette history");
    assert_eq!(
        registry
            .resolve(&key)
            .expect("the MRU setting should resolve"),
        &SettingValue::CommandPaletteMru(mru),
    );
}

#[test]
fn viewport_snap_steps_resolve_at_project_scope_and_round_trip_without_touching_project_sources() {
    let root = temporary_root("viewport-snap-steps");
    let project_root = root.join("project");
    let source_path = project_root
        .join("assets")
        .join("scenes")
        .join("main.zscene");
    fs::create_dir_all(source_path.parent().unwrap()).unwrap();
    fs::write(&source_path, "scene source stays outside editor settings\n").unwrap();
    let source_digest_before = blake3::hash(&fs::read(&source_path).unwrap());

    let store = SettingsStore::from_roots(&root, Some(&project_root));
    let translate_key = key(VIEWPORT_TRANSLATE_STEP_KEY);
    let rotate_key = key(VIEWPORT_ROTATE_STEP_DEGREES_KEY);
    let scale_key = key(VIEWPORT_SCALE_STEP_KEY);
    let mut source = settings_registry_with_defaults();
    source
        .set(
            SettingsScope::User,
            &translate_key,
            SettingValue::Float(0.5),
        )
        .unwrap();
    source
        .set(SettingsScope::User, &rotate_key, SettingValue::Float(30.0))
        .unwrap();
    source
        .set(
            SettingsScope::Project,
            &translate_key,
            SettingValue::Float(2.0),
        )
        .unwrap();
    source
        .set(
            SettingsScope::Project,
            &scale_key,
            SettingValue::Float(0.25),
        )
        .unwrap();
    store.save_from(SettingsScope::User, &source).unwrap();
    store.save_from(SettingsScope::Project, &source).unwrap();

    let encoded = fs::read_to_string(store.paths().project().unwrap()).unwrap();
    assert!(encoded.contains(VIEWPORT_TRANSLATE_STEP_KEY));
    assert!(encoded.contains(VIEWPORT_SCALE_STEP_KEY));

    let mut restored = settings_registry_with_defaults();
    store.load_into(SettingsScope::User, &mut restored).unwrap();
    store
        .load_into(SettingsScope::Project, &mut restored)
        .unwrap();
    assert_eq!(
        restored.resolve(&translate_key).unwrap(),
        &SettingValue::Float(2.0)
    );
    assert_eq!(
        restored.resolve(&rotate_key).unwrap(),
        &SettingValue::Float(30.0)
    );
    assert_eq!(
        restored.resolve(&scale_key).unwrap(),
        &SettingValue::Float(0.25)
    );

    restored
        .set(
            SettingsScope::Session,
            &translate_key,
            SettingValue::Float(4.0),
        )
        .unwrap();
    assert_eq!(
        restored.resolve(&translate_key).unwrap(),
        &SettingValue::Float(4.0)
    );

    assert_eq!(
        source_digest_before,
        blake3::hash(&fs::read(&source_path).unwrap())
    );
    remove_temporary_root(&root);
}

#[test]
fn settings_store_round_trips_current_shell_at_planned_user_and_project_paths() {
    let root = temporary_root("round-trip");
    let project_root = root.join("project");
    let store = SettingsStore::from_roots(&root, Some(&project_root));
    let expected_user_settings = root.join("settings.toml");
    let expected_project_settings = project_root.join(".zircon").join("settings.toml");
    assert_eq!(store.paths().user(), expected_user_settings.as_path());
    assert_eq!(
        store.paths().project(),
        Some(expected_project_settings.as_path())
    );

    let settings_key = key(EDITOR_DESIGN_TOKENS_KEY);
    let mut expected = EditorDesignTokens::workbench_dark();
    expected.id = "zircon.editor.tests.persisted".to_string();
    expected.controls.default_height = 37.0;
    let mut source_registry = settings_registry_with_defaults();
    source_registry
        .set(
            SettingsScope::User,
            &settings_key,
            SettingValue::DesignTokens(expected.clone()),
        )
        .unwrap();
    store
        .save_from(SettingsScope::User, &source_registry)
        .unwrap();

    let encoded = fs::read_to_string(store.paths().user()).unwrap();
    assert!(encoded.contains("\"$zircon\""));
    assert!(encoded.contains("\"schema_id\": \"zircon.editor.settings\""));
    assert!(encoded.ends_with('\n'));

    let mut restored = settings_registry_with_defaults();
    assert!(matches!(
        store.load_into(SettingsScope::User, &mut restored).unwrap(),
        SettingsLoad::Loaded { .. }
    ));
    assert_eq!(
        restored.resolve(&settings_key).unwrap(),
        &SettingValue::DesignTokens(expected)
    );

    let mut replacement = EditorDesignTokens::workbench_dark();
    replacement.id = "zircon.editor.tests.replaced".to_string();
    source_registry
        .set(
            SettingsScope::User,
            &settings_key,
            SettingValue::DesignTokens(replacement.clone()),
        )
        .unwrap();
    store
        .save_from(SettingsScope::User, &source_registry)
        .unwrap();
    let mut replaced = settings_registry_with_defaults();
    store.load_into(SettingsScope::User, &mut replaced).unwrap();
    assert_eq!(
        replaced.resolve(&settings_key).unwrap(),
        &SettingValue::DesignTokens(replacement)
    );
    remove_temporary_root(&root);
}

#[test]
fn settings_store_rejects_retired_formats_and_keeps_the_existing_layer_atomic() {
    let root = temporary_root("strict-load");
    let store = SettingsStore::from_roots(&root, None);
    let settings_key = key(EDITOR_DESIGN_TOKENS_KEY);
    fs::create_dir_all(root.as_path()).unwrap();
    fs::write(store.paths().user(), "active_profile = 'legacy'\n").unwrap();
    let mut registry = settings_registry_with_defaults();

    assert!(matches!(
        store.load_into(SettingsScope::User, &mut registry),
        Err(SettingsStoreError::Decode {
            source: SettingsDecodeError::LegacyPayload,
            ..
        })
    ));
    let default = registry.resolve(&settings_key).unwrap().clone();

    let mut custom_tokens = EditorDesignTokens::workbench_dark();
    custom_tokens.id = "zircon.editor.tests.rejected".to_string();
    let invalid_document = SettingsDocument {
        values: BTreeMap::from([
            (
                settings_key.clone(),
                SettingValue::DesignTokens(custom_tokens),
            ),
            (key("editor.unknown_setting"), SettingValue::Bool(true)),
        ]),
    };
    fs::write(
        store.paths().user(),
        write_versioned_text(&invalid_document).unwrap(),
    )
    .unwrap();
    assert!(matches!(
        store.load_into(SettingsScope::User, &mut registry),
        Err(SettingsStoreError::Apply {
            source: SettingsError::UnknownKey(_),
            ..
        })
    ));
    assert_eq!(registry.resolve(&settings_key).unwrap(), &default);

    let current = fs::read_to_string(store.paths().user()).unwrap();
    let mut retired = serde_json::from_str::<serde_json::Value>(&current).unwrap();
    retired["$zircon"]["header"]["schema_version"] = json!(0);
    fs::write(
        store.paths().user(),
        serde_json::to_string(&retired).unwrap(),
    )
    .unwrap();
    assert!(matches!(
        store.load_into(SettingsScope::User, &mut registry),
        Err(SettingsStoreError::Decode {
            source: SettingsDecodeError::Versioned(_),
            ..
        })
    ));

    let malformed_key = serde_json::json!({
        "$zircon": {
            "header": {
                "schema_id": "zircon.editor.settings",
                "schema_version": 1
            },
            "payload": {
                "values": {
                    "Editor.invalid": { "kind": "bool", "value": true }
                }
            }
        }
    });
    fs::write(
        store.paths().user(),
        serde_json::to_string(&malformed_key).unwrap(),
    )
    .unwrap();
    assert!(matches!(
        store.load_into(SettingsScope::User, &mut registry),
        Err(SettingsStoreError::Decode {
            source: SettingsDecodeError::Versioned(_),
            ..
        })
    ));
    remove_temporary_root(&root);
}

#[test]
fn user_environment_value_is_a_root_override_not_a_retired_file_override() {
    let root = temporary_root("env-root");
    let root_value = OsString::from(root.as_os_str());
    assert_eq!(
        SettingsPaths::user_root_from_env_value(Some(root_value)).unwrap(),
        root
    );
    let expected_user_settings = root.join("settings.toml");
    assert_eq!(
        SettingsPaths::from_roots(&root, None).user(),
        expected_user_settings.as_path()
    );

    fs::write(&root, "retired settings file").unwrap();
    assert!(matches!(
        SettingsPaths::user_root_from_env_value(Some(OsString::from(root.as_os_str()))),
        Err(SettingsStoreError::UserRootIsFile { .. })
    ));
    let _ = fs::remove_file(root);
}

#[test]
fn session_layer_cannot_be_persisted() {
    let root = temporary_root("session-only");
    let store = SettingsStore::from_roots(&root, None);
    let registry = settings_registry_with_defaults();
    assert!(matches!(
        store.save_from(SettingsScope::Session, &registry),
        Err(SettingsStoreError::NonPersistentScope(
            SettingsScope::Session
        ))
    ));
}
