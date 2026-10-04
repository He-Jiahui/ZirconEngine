use super::PersistedAssetReference;

#[test]
fn serde_rejects_non_builtin_locator_for_builtin_variant() {
    let error = serde_json::from_str::<PersistedAssetReference>(
        r#"{"kind":"builtin","locator":"res://materials/hero.zmaterial"}"#,
    )
    .expect_err("builtin variant must reject project locator");
    assert!(error.to_string().contains("requires builtin://"));
}

#[test]
fn serde_rejects_locator_payload_for_project_variant() {
    serde_json::from_str::<PersistedAssetReference>(
        r#"{"kind":"project","locator":"builtin://shader/pbr.wgsl"}"#,
    )
    .expect_err("project variant must contain AssetRef fields only");
}
