use super::EditorAssetCatalogGeneration;

#[test]
fn catalog_identity_advances_revision_and_publish_epoch_together() {
    let generation = EditorAssetCatalogGeneration::default();

    assert_eq!(generation.next_catalog_identity(), (1, 1));
}

#[test]
#[should_panic(expected = "editor asset catalog revision exhausted")]
fn catalog_revision_exhaustion_never_reuses_an_identity() {
    let mut generation = EditorAssetCatalogGeneration::default();
    generation.catalog_revision = u64::MAX;

    let _ = generation.next_catalog_identity();
}

#[test]
#[should_panic(expected = "editor asset catalog publish epoch exhausted")]
fn publish_epoch_exhaustion_never_reuses_an_identity() {
    let mut generation = EditorAssetCatalogGeneration::default();
    generation.publish_epoch = u64::MAX;

    let _ = generation.next_publish_epoch();
}
