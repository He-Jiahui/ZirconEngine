#[test]
fn asset_derived_writer_uses_runtime_atomic_publication() {
    let source = include_str!("../ibl_bake_artifact_asset_derived.rs");
    let writer = source
        .split("pub fn write_asset_derived_blob(")
        .nth(1)
        .and_then(|writer| {
            writer
                .split("pub fn write_source_cubemap_asset_derived_artifact(")
                .next()
        })
        .expect("asset-derived store must retain its writer");

    assert!(source.contains("core::resource::io::atomic_write"));
    assert!(writer.contains("atomic_write("));
    assert!(!writer.contains("fs::write("));
}
