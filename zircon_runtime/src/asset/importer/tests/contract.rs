use super::*;
use crate::asset::importer::AssetImportRecipeValue;

#[test]
fn source_str_borrows_the_context_utf8_buffer() {
    let context = AssetImportContext::new(
        PathBuf::from("assets/shaders/main.wgsl"),
        AssetUri::parse("res://shaders/main.wgsl").unwrap(),
        b"@compute @workgroup_size(1) fn main() {}".to_vec(),
        toml::Table::new(),
    );

    let source = context.source_str().unwrap();

    assert_eq!(source, "@compute @workgroup_size(1) fn main() {}");
    assert_eq!(source.as_ptr(), context.source_bytes.as_ptr());
    assert_eq!(source.len(), context.source_bytes.len());
}

fn context_with_settings(import_settings: toml::Table) -> AssetImportContext {
    AssetImportContext::new(
        PathBuf::from("assets/models/mesh.obj"),
        AssetUri::parse("res://models/mesh.obj").unwrap(),
        Vec::new(),
        import_settings,
    )
}

#[test]
fn context_exposes_disabled_virtual_geometry_request_by_default() {
    assert_eq!(
        context_with_settings(toml::Table::new())
            .virtual_geometry_cook_request()
            .unwrap(),
        VirtualGeometryCookRequest::Disabled
    );
}

#[test]
fn context_rejects_malformed_virtual_geometry_request() {
    let settings = toml::from_str(
        r#"
                [virtual_geometry]
                enabled = "yes"
            "#,
    )
    .unwrap();

    assert!(matches!(
        context_with_settings(settings).virtual_geometry_cook_request(),
        Err(AssetImportError::Parse(message)) if message.contains("virtual_geometry.enabled")
    ));
}

#[test]
fn context_exposes_the_recipe_and_qualified_build_identity() {
    let mut settings = toml::Table::new();
    settings.insert("quality".to_string(), toml::Value::Integer(4));
    let recipe = AssetImportRecipe::from_legacy_settings(settings.clone());
    let build_context = AssetImportBuildContext::new(
        "windows-x86_64",
        "project_import",
        8,
        "zircon-runtime@test;model-importer@3",
    );
    let identity = AssetImportBuildIdentity::new(
        "model-plugin:model-importer",
        recipe,
        "blake3:input",
        build_context.clone(),
    );
    let action_key = identity.action_key().to_string();

    let context = context_with_settings(settings).with_build_identity(identity);

    assert_eq!(
        context.import_settings().get("quality"),
        Some(&toml::Value::Integer(4))
    );
    assert_eq!(context.import_recipe().schema_version(), 1);
    assert_eq!(
        context.import_recipe().setting("quality"),
        Some(&AssetImportRecipeValue::Integer(4))
    );
    assert_eq!(context.build_context(), Some(&build_context));
    assert_eq!(context.build_action_key(), Some(action_key.as_str()));
}

#[test]
#[should_panic(expected = "asset import build identity recipe must match the context settings")]
fn context_rejects_a_build_identity_for_different_settings() {
    let mut context_settings = toml::Table::new();
    context_settings.insert("quality".to_string(), toml::Value::Integer(4));
    let mut identity_settings = toml::Table::new();
    identity_settings.insert("quality".to_string(), toml::Value::Integer(5));
    let identity = AssetImportBuildIdentity::new(
        "model-importer",
        AssetImportRecipe::from_legacy_settings(identity_settings),
        "blake3:input",
        AssetImportBuildContext::new(
            "windows-x86_64",
            "project_import",
            8,
            "zircon-runtime@test;model-importer@3",
        ),
    );

    let _ = context_with_settings(context_settings).with_build_identity(identity);
}
