use super::*;

#[tokio::test]
async fn helper_response_is_bounded_and_never_relays_unsanitized_errors() {
    let unknown = b"{\"status\":\"failed\",\"error\":\"secret path or token\"}\n";
    assert!(matches!(
        read_response(&mut &unknown[..]).await,
        Err(AccountError::PackageUnavailable)
    ));
    for bytes in [
        vec![b'x'; MAX_REPLY + 1],
        b"{\"status\":\"ready\"}".to_vec(),
    ] {
        assert!(read_response(&mut &bytes[..]).await.is_err());
    }
    let exact = format!(
        "{{\"status\":\"ready\",\"data\":\"{}\"}}\n",
        "x".repeat(MAX_REPLY - 29)
    );
    assert_eq!(exact.len(), MAX_REPLY);
    assert!(read_response(&mut exact.as_bytes()).await.is_ok());
}

#[test]
fn policy_selection_errors_remain_actionable_and_precommit() {
    assert!(matches!(
        helper_error(Some("package_policy_unconfigured")),
        AccountError::PackagePolicyUnconfigured
    ));
    assert!(matches!(
        helper_error(Some("package_target_unconfigured")),
        AccountError::PackageTargetUnconfigured
    ));
    assert!(matches!(
        helper_error(Some("package_trust_denied")),
        AccountError::PackageTrust
    ));
}

#[test]
fn helper_receives_the_verified_policy_index_digest_with_its_path() {
    let digest = "a".repeat(64);
    let args = helper_policy_arguments(
        std::path::Path::new(r"C:\Host\zircon_native_plugin_policy_index.json"),
        &digest,
    );
    assert_eq!(args[0], r"C:\Host\zircon_native_plugin_policy_index.json");
    assert_eq!(args[1].as_os_str(), std::ffi::OsStr::new(&digest));
}
