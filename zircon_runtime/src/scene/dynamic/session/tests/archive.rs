use super::RuntimeSessionArchive;

#[test]
fn invalid_runtime_session_archive_generation_caches_its_seal_rejection() {
    let archive = RuntimeSessionArchive::from_payload(u32::MAX, Vec::new());

    let first = archive
        .sealed_artifact()
        .expect_err("unsupported format must reject the generation");
    let second = archive
        .sealed_artifact()
        .expect_err("deterministic validation rejection must be cached");

    assert_eq!(first.to_string(), second.to_string());
    let diagnostics = archive.artifact_diagnostics();
    assert_eq!(diagnostics.validate_count, 0);
    assert_eq!(diagnostics.serialize_count, 0);
}
