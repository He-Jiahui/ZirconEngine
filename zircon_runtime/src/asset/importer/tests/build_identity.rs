use super::*;

#[test]
fn project_import_context_consumes_the_generated_build_identity() {
    let context = AssetImportBuildContext::project_import("test-importer@7");
    assert_eq!(
        context.target_platform(),
        env!("ZR_ASSET_IMPORT_BUILD_TARGET")
    );
    assert_eq!(
        context.toolchain_identity(),
        format!("test-importer@7;build={}", env!("ZR_ASSET_IMPORT_BUILD_ID"))
    );
    assert_eq!(context.build_profile(), "project_import");
    assert_eq!(
        context.engine_abi_version(),
        zircon_runtime_interface::ZIRCON_RUNTIME_API_VERSION_V8
    );
}

fn context() -> AssetImportBuildContext {
    AssetImportBuildContext::new(
        "windows-x86_64",
        "project_import",
        8,
        "zircon-runtime@0.1.0;test-importer@7",
    )
}

fn recipe_with_quality(quality: i64) -> AssetImportRecipe {
    let mut settings = toml::Table::new();
    settings.insert("quality".to_string(), toml::Value::Integer(quality));
    AssetImportRecipe::from_legacy_settings(settings)
}

#[test]
fn legacy_settings_migrate_to_the_current_recipe_schema_with_empty_defaults() {
    let recipe = AssetImportRecipe::default();

    assert_eq!(recipe.schema_version(), ASSET_IMPORT_RECIPE_SCHEMA_VERSION);
    assert!(recipe.settings().is_empty());
    assert_eq!(
        recipe,
        AssetImportRecipe::from_legacy_settings(toml::Table::new())
    );
}

#[test]
fn canonical_recipe_encoding_is_independent_of_table_insertion_order() {
    let mut first = toml::Table::new();
    first.insert("zeta".to_string(), toml::Value::Integer(3));
    first.insert("alpha".to_string(), toml::Value::String("x".to_string()));
    let mut second = toml::Table::new();
    second.insert("alpha".to_string(), toml::Value::String("x".to_string()));
    second.insert("zeta".to_string(), toml::Value::Integer(3));

    assert_eq!(
        AssetImportRecipe::from_legacy_settings(first).canonical_bytes(),
        AssetImportRecipe::from_legacy_settings(second).canonical_bytes()
    );
}

#[test]
fn legacy_recipe_migration_preserves_typed_value_domains() {
    let settings = toml::from_str(
        r#"
name = "mesh"
quality = 3
scale = 1.5
enabled = true
timestamp = 2026-09-07T00:00:00Z
channels = ["normal", "tangent"]

[nested]
mode = "strict"
"#,
    )
    .unwrap();
    let recipe = AssetImportRecipe::from_legacy_settings(settings);

    assert!(matches!(
        recipe.setting("name"),
        Some(AssetImportRecipeValue::String(value)) if value == "mesh"
    ));
    assert_eq!(
        recipe.setting("quality"),
        Some(&AssetImportRecipeValue::Integer(3))
    );
    assert_eq!(
        recipe.setting("scale"),
        Some(&AssetImportRecipeValue::Float(1.5))
    );
    assert_eq!(
        recipe.setting("enabled"),
        Some(&AssetImportRecipeValue::Boolean(true))
    );
    assert!(matches!(
        recipe.setting("timestamp"),
        Some(AssetImportRecipeValue::Datetime(value)) if value == "2026-09-07T00:00:00Z"
    ));
    assert!(matches!(
        recipe.setting("channels"),
        Some(AssetImportRecipeValue::Array(values)) if values.len() == 2
    ));
    assert!(matches!(
        recipe.setting("nested"),
        Some(AssetImportRecipeValue::Table(values))
            if values.get("mode")
                == Some(&AssetImportRecipeValue::String("strict".to_string()))
    ));
}

#[test]
fn action_key_is_cryptographic_and_covers_every_qualified_input_domain() {
    let baseline = AssetImportBuildIdentity::new(
        "plugin:model",
        recipe_with_quality(2),
        "blake3:source-a",
        context(),
    );

    assert!(baseline.action_key().starts_with("blake3:"));
    assert_eq!(baseline.action_key().len(), "blake3:".len() + 64);
    for candidate in [
        AssetImportBuildIdentity::new(
            "plugin:model-v2",
            recipe_with_quality(2),
            "blake3:source-a",
            context(),
        ),
        AssetImportBuildIdentity::new(
            "plugin:model",
            recipe_with_quality(3),
            "blake3:source-a",
            context(),
        ),
        AssetImportBuildIdentity::new(
            "plugin:model",
            recipe_with_quality(2),
            "blake3:source-b",
            context(),
        ),
        AssetImportBuildIdentity::new(
            "plugin:model",
            recipe_with_quality(2),
            "blake3:source-a",
            AssetImportBuildContext::new(
                "linux-x86_64",
                "project_import",
                8,
                "zircon-runtime@0.1.0;test-importer@7",
            ),
        ),
        AssetImportBuildIdentity::new(
            "plugin:model",
            recipe_with_quality(2),
            "blake3:source-a",
            AssetImportBuildContext::new(
                "windows-x86_64",
                "shipping",
                8,
                "zircon-runtime@0.1.0;test-importer@7",
            ),
        ),
        AssetImportBuildIdentity::new(
            "plugin:model",
            recipe_with_quality(2),
            "blake3:source-a",
            AssetImportBuildContext::new(
                "windows-x86_64",
                "project_import",
                9,
                "zircon-runtime@0.1.0;test-importer@7",
            ),
        ),
        AssetImportBuildIdentity::new(
            "plugin:model",
            recipe_with_quality(2),
            "blake3:source-a",
            AssetImportBuildContext::new(
                "windows-x86_64",
                "project_import",
                8,
                "zircon-runtime@0.1.0;test-importer@8",
            ),
        ),
    ] {
        assert_ne!(candidate.action_key(), baseline.action_key());
    }
}

#[test]
fn canonical_input_digest_covers_primary_auxiliary_path_and_bytes() {
    let root = PathBuf::from("assets");
    let mut snapshots = BTreeMap::new();
    snapshots.insert(root.join("models/mesh.bin"), vec![1, 2, 3]);
    let baseline =
        canonical_import_input_digest("res://models/mesh.gltf", b"model", &snapshots, Some(&root));

    assert!(baseline.starts_with("blake3:"));
    assert_eq!(baseline.len(), "blake3:".len() + 64);
    assert_ne!(
        baseline,
        canonical_import_input_digest("res://models/moved.gltf", b"model", &snapshots, Some(&root))
    );
    assert_ne!(
        baseline,
        canonical_import_input_digest(
            "res://models/mesh.gltf",
            b"changed",
            &snapshots,
            Some(&root)
        )
    );
    snapshots.insert(root.join("models/mesh.bin"), vec![4, 5, 6]);
    assert_ne!(
        baseline,
        canonical_import_input_digest("res://models/mesh.gltf", b"model", &snapshots, Some(&root))
    );
    snapshots.insert(root.join("models/mesh.bin"), vec![1, 2, 3]);
    let moved = snapshots
        .remove(&root.join("models/mesh.bin"))
        .expect("snapshot");
    snapshots.insert(root.join("models/renamed.bin"), moved);
    assert_ne!(
        baseline,
        canonical_import_input_digest("res://models/mesh.gltf", b"model", &snapshots, Some(&root))
    );
}
