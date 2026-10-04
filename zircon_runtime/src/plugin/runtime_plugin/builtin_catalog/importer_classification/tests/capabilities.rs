use super::primary_importer_capability;

#[test]
fn fallback_importer_capability_writes_slug_into_single_output() {
    assert_eq!(
        primary_importer_capability("custom_mesh_importer"),
        "runtime.asset.importer.custom.mesh"
    );
    assert_eq!(
        primary_importer_capability("procedural_cache"),
        "runtime.asset.importer.procedural.cache"
    );
    assert_eq!(
        primary_importer_capability("gltf_importer"),
        "runtime.asset.importer.model.gltf"
    );
}
