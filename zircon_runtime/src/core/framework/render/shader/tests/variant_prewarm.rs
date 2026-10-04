use super::{
    ShaderVariantPrewarmManifest, ShaderVariantPrewarmManifestIntegrityError,
    ShaderVariantPrewarmOutcome, ShaderVariantPrewarmSource,
    ShaderVariantPrewarmSourceProvenanceSummary,
};

#[test]
fn source_provenance_aggregates_repeated_source_outcomes() {
    let source = ShaderVariantPrewarmSource::new(
        "res://shared.wgsl",
        "fn main() {}",
        vec!["include-a".to_string()],
        "template-r1",
        "naga-r1",
        "wgpu-r1",
    );
    let mut summary = ShaderVariantPrewarmSourceProvenanceSummary::default();

    summary.record(&source, ShaderVariantPrewarmOutcome::Written);
    summary.record(&source, ShaderVariantPrewarmOutcome::Failed);

    assert_eq!(summary.source_count, 1);
    assert_eq!(summary.variant_count, 2);
    let entry = summary
        .sources
        .get(source.id.as_str())
        .expect("shared source should have one provenance entry");
    assert_eq!(entry.source_hash, source.source_hash());
    assert_eq!(entry.requested_count, 2);
    assert_eq!(entry.written_count, 1);
    assert_eq!(entry.failed_count, 1);
}

#[test]
fn manifest_integrity_reports_duplicate_source_id_as_typed_error() {
    let source = ShaderVariantPrewarmSource::new(
        "res://shared.wgsl",
        "fn main() {}",
        Vec::new(),
        "template-r1",
        "naga-r1",
        "wgpu-r1",
    );

    let error = ShaderVariantPrewarmManifest::new(vec![source.clone(), source], Vec::new())
        .validate_integrity()
        .expect_err("duplicate source ids must fail manifest integrity validation");
    assert!(matches!(
        error,
        ShaderVariantPrewarmManifestIntegrityError::DuplicateSourceId { .. }
    ));
}
