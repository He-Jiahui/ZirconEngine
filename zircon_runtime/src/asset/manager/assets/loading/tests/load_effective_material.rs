use crate::asset::{AssetReference, AssetUri, MaterialAsset, ProjectAssetManager};
use crate::core::framework::render::RenderMaterialValidationError;
use crate::core::resource::{ResourceId, ResourceKind, ResourceRecord};

fn material() -> MaterialAsset {
    MaterialAsset::from_toml_str(
        r#"
version = 2

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "builtin://shader/pbr.wgsl"
"#,
    )
    .expect("material document")
}

fn insert_material(
    manager: &ProjectAssetManager,
    uri: &str,
    material: MaterialAsset,
) -> (ResourceId, AssetUri) {
    let uri = AssetUri::parse(uri).expect("material uri");
    let id = ResourceId::from_locator(&uri);
    manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(id, ResourceKind::Material, uri.clone()),
            material,
        )
        .expect("material insert");
    (id, uri)
}

#[test]
fn effective_material_inherits_parent_values_once_for_all_consumers() {
    let manager = ProjectAssetManager::default();
    let mut parent = material();
    parent
        .property_values
        .insert("roughness".to_string(), toml::Value::Float(0.23));
    let (_, parent_uri) = insert_material(
        &manager,
        "res://materials/effective-parent.zmaterial",
        parent,
    );
    let mut child = material();
    child.parent = Some(AssetReference::from_locator(parent_uri));
    let (child_id, _) =
        insert_material(&manager, "res://materials/effective-child.zmaterial", child);

    let (effective, diagnostics) = manager
        .load_effective_material_asset(child_id)
        .expect("effective child material");

    assert_eq!(effective.roughness, 0.23);
    assert!(effective.parent.is_none());
    assert!(diagnostics.is_empty());
}

#[test]
fn missing_parent_keeps_the_renderable_child_and_reports_one_diagnostic() {
    let manager = ProjectAssetManager::default();
    let mut child = material();
    child.roughness = 0.41;
    child.parent = Some(AssetReference::from_locator(
        AssetUri::parse("res://materials/missing-parent.zmaterial").expect("missing parent uri"),
    ));
    let (child_id, _) = insert_material(
        &manager,
        "res://materials/missing-parent-child.zmaterial",
        child,
    );

    let (effective, diagnostics) = manager
        .load_effective_material_asset(child_id)
        .expect("renderable child material");

    assert_eq!(effective.roughness, 0.41);
    assert!(effective.parent.is_none());
    assert!(matches!(
        diagnostics.as_slice(),
        [RenderMaterialValidationError::InvalidMaterialParent { .. }]
    ));
}
