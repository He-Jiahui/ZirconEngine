use super::*;
use crate::plugin::PluginPackageManifest;

#[test]
fn deny_all_authority_rejects_before_artifact_access() {
    let candidate = NativePluginCandidate {
        plugin_id: "weather".to_string(),
        package_manifest: PluginPackageManifest::new("weather", "Weather"),
        manifest_path: PathBuf::from("missing-plugin.toml"),
        library_path: PathBuf::from("missing-plugin.dll"),
    };
    let error = NativePluginArtifactAuthority::deny_all()
        .admit(
            &candidate,
            &candidate.library_path,
            &[PluginModuleKind::Runtime],
        )
        .expect_err("authority-free native execution must fail closed");
    assert!(matches!(
        error,
        NativePluginArtifactAdmissionError::MissingAuthority { .. }
    ));
}

#[test]
fn isolated_artifact_cannot_enter_in_process_loader() {
    let digest = NativePluginArtifactDigest {
        sha256: "0".repeat(64),
        byte_length: 0,
    };
    let mut expectation = NativePluginArtifactExpectation::trusted_local_first_party(
        "weather",
        "weather",
        digest.clone(),
        digest,
        "local-test-authority",
        NativePluginArtifactTarget::new(
            RuntimeTargetMode::ClientRuntime,
            ExportTargetPlatform::Windows,
        ),
        [PluginModuleKind::Runtime],
        [],
    );
    expectation.trust = NativePluginArtifactTrust::IsolatedUntrusted {
        reason: "third-party signer is not trusted".to_string(),
    };
    assert!(matches!(
        NativePluginArtifactAuthority::from_expectations([expectation]),
        Err(NativePluginArtifactAdmissionError::IsolationRequired { .. })
    ));
}

#[test]
fn admission_rejects_non_product_role_even_with_matching_authority() {
    let digest = NativePluginArtifactDigest {
        sha256: "0".repeat(64),
        byte_length: 0,
    };
    let expectation = NativePluginArtifactExpectation::trusted_local_first_party(
        "fixture",
        "fixture",
        digest.clone(),
        digest,
        "test-authority",
        NativePluginArtifactTarget::new(
            RuntimeTargetMode::ClientRuntime,
            ExportTargetPlatform::Windows,
        ),
        [PluginModuleKind::Runtime],
        [],
    );
    let authority = NativePluginArtifactAuthority::from_expectations([expectation])
        .expect("fixture authority should be structurally valid");
    let candidate = NativePluginCandidate {
        plugin_id: "fixture".to_string(),
        package_manifest: PluginPackageManifest::new("fixture", "Fixture")
            .with_package_role(crate::plugin::PluginPackageRole::TestFixture),
        manifest_path: PathBuf::from("missing-plugin.toml"),
        library_path: PathBuf::from("missing-plugin.dll"),
    };

    let error = authority
        .admit(
            &candidate,
            &candidate.library_path,
            &[PluginModuleKind::Runtime],
        )
        .expect_err("test carrier must be rejected before artifact access");
    assert!(matches!(
        error,
        NativePluginArtifactAdmissionError::IneligiblePackageRole {
            plugin_id,
            role: crate::plugin::PluginPackageRole::TestFixture,
        } if plugin_id == "fixture"
    ));
}

#[test]
fn local_authority_rejects_non_product_package_roles_before_artifact_capture() {
    let root = std::env::temp_dir().join(format!(
        "zircon-native-authority-role-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).expect("create role fixture root");
    let manifest = root.join("plugin.toml");
    std::fs::write(
        &manifest,
        "id = \"fixture\"\nversion = \"0.1.0\"\ndisplay_name = \"Fixture\"\npackage_role = \"test_fixture\"\n\n[[modules]]\nname = \"fixture.runtime\"\nkind = \"runtime\"\ncrate_name = \"fixture_native\"\n",
    )
    .expect("write role fixture manifest");

    let error = NativePluginArtifactAuthority::capture_trusted_local_package(
        "fixture",
        &manifest,
        "test-authority",
        NativePluginArtifactTarget::new(
            RuntimeTargetMode::ClientRuntime,
            ExportTargetPlatform::Windows,
        ),
        [PluginModuleKind::Runtime],
    )
    .expect_err("test fixture must not mint local native authority");
    assert!(matches!(
        error,
        NativePluginArtifactAdmissionError::InvalidAuthority { reason }
            if reason.contains("cannot capture package fixture")
    ));
    debug_assert!(root.starts_with(std::env::temp_dir()));
    std::fs::remove_dir_all(root).expect("remove role fixture root");
}
