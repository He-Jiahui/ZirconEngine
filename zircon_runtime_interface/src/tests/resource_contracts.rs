use crate::resource::{ResourceId, ResourceKind, ResourceLocator, ResourceRecord};

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
