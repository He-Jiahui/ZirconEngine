use crate::resource::{
    AssetReference, AssetUuid, ResourceId, ResourceKind, ResourceLocator, ResourceRecord,
    ResourceScheme,
};

#[test]
fn resource_contract_exposes_stable_identity_and_status_records() {
    let locator = ResourceLocator::parse("res://materials/hero.mat#surface").unwrap();
    let id = ResourceId::from_locator(&locator);
    let dependency_id = ResourceId::from_stable_label("res://textures/hero.png");
    let record = ResourceRecord::new(id, ResourceKind::Material, locator.clone())
        .with_source_hash("source-hash")
        .with_importer_version(2)
        .with_dependency_ids(vec![dependency_id]);

    assert_eq!(locator.to_string(), "res://materials/hero.mat#surface");
    assert_eq!(record.id(), id);
    assert_eq!(record.kind, ResourceKind::Material);
    assert_eq!(record.primary_locator(), &locator);
    assert_eq!(record.source_hash, "source-hash");
    assert_eq!(record.importer_version, 2);
    assert_eq!(record.dependency_ids, vec![dependency_id]);
}

#[test]
fn resource_contract_parses_package_locators_and_asset_reference_urls() {
    let locator =
        ResourceLocator::parse("package://com.zircon.navigation/nav/agent.znav#mesh").unwrap();
    let uuid = AssetUuid::from_stable_label("package://com.zircon.navigation/nav/agent.znav#mesh");
    let reference = AssetReference::new(uuid, locator.clone());
    let json = serde_json::to_string(&reference).unwrap();
    let decoded: AssetReference = serde_json::from_str(&json).unwrap();

    assert_eq!(locator.scheme(), ResourceScheme::Package);
    assert_eq!(locator.package_id(), Some("com.zircon.navigation"));
    assert_eq!(locator.package_path(), Some("nav/agent.znav"));
    assert_eq!(locator.label(), Some("mesh"));
    assert!(json.contains("\"url\""));
    assert!(!json.contains("\"locator\""));
    assert_eq!(decoded.uuid, uuid);
    assert_eq!(decoded.locator, locator);
}
