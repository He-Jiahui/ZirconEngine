use super::*;

#[test]
fn trusted_host_consumer_rejects_missing_or_malformed_identity() {
    assert_eq!(
        require_trusted_host_build_set_id(None),
        Err(ZrRuntimeTrustedHostBuildSetError::Missing)
    );
    assert!(matches!(
        require_trusted_host_build_set_id(Some("A")),
        Err(ZrRuntimeTrustedHostBuildSetError::Invalid { .. })
    ));
}

#[test]
fn trusted_host_consumer_accepts_only_the_canonical_lowercase_identity() {
    let value = "a".repeat(64);
    assert_eq!(
        require_trusted_host_build_set_id(Some(&value)),
        Ok(value.as_str())
    );
}
