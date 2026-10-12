use super::wire::*;
use super::*;
use crate::core::framework::{platform::RuntimeTargetMode, project::ExportTargetPlatform};
use ring::signature::{Ed25519KeyPair, KeyPair};

const MANIFEST: &str = "id = \"weather\"\nversion = \"1.0.0\"\ndisplay_name = \"Weather\"\nsupported_targets = [\"client_runtime\"]\nsupported_platforms = [\"windows\"]\ncapabilities = [\"weather.read\"]\n[[modules]]\nname = \"WeatherRuntime\"\nkind = \"runtime\"\ncrate_name = \"weather_runtime\"\ncapabilities = [\"weather.write\"]\n";

fn time(value: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(value)
        .unwrap()
        .with_timezone(&Utc)
}
fn key(seed: u8) -> Ed25519KeyPair {
    Ed25519KeyPair::from_seed_unchecked(&[seed; 32]).unwrap()
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02X}")).collect()
}
fn key_policy(signer: &str) -> NativePackageKeyPolicy {
    NativePackageKeyPolicy {
        signer_id: signer.into(),
        not_before: time("2026-09-01T00:00:00Z"),
        not_after: time("2026-10-01T00:00:00Z"),
        revoked: false,
        allowed_package_ids: vec!["weather".into()],
    }
}
fn trust(
    signers: &[(&str, &Ed25519KeyPair, bool)],
    policies: Vec<NativePackageKeyPolicy>,
) -> NativePackageReceiptTrust {
    let issuers: Vec<_> = signers.iter().map(|(id, key, disabled)| serde_json::json!({"signer_id":id,"algorithm":"ed25519-v1","public_key_hex":hex(key.public_key().as_ref()),"disabled":disabled})).collect();
    let registry = serde_json::to_vec(&serde_json::json!({"schema_version":1,"trust_registry_kind":"zircon_product_receipt_trust_registry","issuers":issuers})).unwrap();
    NativePackageReceiptTrust::from_registry_json(
        &registry,
        policies,
        time("2026-09-06T00:00:00Z"),
        16_384,
    )
    .unwrap()
}
fn policy() -> NativePackageReceiptPolicy {
    NativePackageReceiptPolicy {
        plugin_id: "weather".into(),
        package_id: "weather".into(),
        package_version: "1.0.0".into(),
        sdk_api_version: "0.1.0".into(),
        build_set_id: "A".repeat(64),
        target: NativePluginArtifactTarget::new(
            RuntimeTargetMode::ClientRuntime,
            ExportTargetPlatform::Windows,
        ),
        target_triple: "x86_64-pc-windows-msvc".into(),
        manifest_logical_name: "weather-manifest".into(),
        manifest_relative_path: "plugins/weather/plugin.toml".into(),
        modules: vec![NativePackageModuleArtifact {
            module_kind: PluginModuleKind::Runtime,
            logical_name: "weather-runtime".into(),
            relative_path: "plugins/weather/weather_runtime.dll".into(),
            dependencies: vec![],
        }],
        required_capabilities: vec!["weather.read".into()],
        allowed_capabilities: vec!["weather.read".into(), "weather.write".into()],
        now: time("2026-09-05T12:00:00Z"),
        max_receipt_age_seconds: 86_400,
        max_receipt_bytes: 65_536,
        max_receipt_count: 16,
        max_manifest_bytes: 16_384,
    }
}
fn artifact(name: &str, path: &str, kind: &str, bytes: &[u8]) -> ReceiptArtifact {
    let digest = digest(bytes);
    ReceiptArtifact {
        logical_name: name.into(),
        relative_path: path.into(),
        kind: kind.into(),
        sha256: digest.sha256.to_ascii_uppercase(),
        byte_length: digest.byte_length,
    }
}
fn fixture() -> ProductReceipt {
    let mut toolchain = ToolchainSet {
        toolchain_set_id: String::new(),
        cargo_sha256: "B".repeat(64),
        rustc_sha256: "C".repeat(64),
        linker_sha256: Some("D".repeat(64)),
        sdk_fingerprint: "E".repeat(64),
        environment_digest: "F".repeat(64),
    };
    toolchain.toolchain_set_id = toolchain_id(&toolchain).unwrap();
    ProductReceipt {
        schema_version: 1,
        receipt_kind: "zircon_product_receipt".into(),
        receipt_id: String::new(),
        created_utc: "2026-09-05T08:00:00.0000000Z".into(),
        build_set_id: "A".repeat(64),
        toolchain,
        target_profile: TargetProfile {
            target_triple: "x86_64-pc-windows-msvc".into(),
            cargo_profile: "release".into(),
            codegen_flags_digest: "1".repeat(64),
            cargo_graph_digest: "2".repeat(64),
        },
        action: BuildAction {
            package: "zircon_app".into(),
            bin: Some("zircon_runtime".into()),
            features: vec!["target-client".into()],
        },
        producer: ProducerIdentity {
            tool: "cargo-zircon".into(),
            tool_version: "0.1.0".into(),
            worker_id: "windows-worker-01".into(),
            operation_id: "package-test".into(),
        },
        build_products: vec![artifact(
            "runtime-executable",
            "bin/zircon_runtime.exe",
            "executable",
            b"exe",
        )],
        runtime_dependencies: vec![
            artifact(
                "weather-manifest",
                "plugins/weather/plugin.toml",
                "resource",
                MANIFEST.as_bytes(),
            ),
            artifact(
                "weather-runtime",
                "plugins/weather/weather_runtime.dll",
                "dynamic_library",
                b"dll",
            ),
        ],
        symbols: vec![],
        sbom: None,
        attestation: ReceiptAttestation {
            signer_id: "worker-old".into(),
            algorithm: "ed25519-v1".into(),
            signature_hex: String::new(),
        },
    }
}
fn sign(mut receipt: ProductReceipt, signer: &str, key: &Ed25519KeyPair) -> Vec<u8> {
    receipt
        .runtime_dependencies
        .sort_by(|a, b| a.logical_name.cmp(&b.logical_name));
    receipt.attestation.signer_id = signer.into();
    receipt.receipt_id = receipt.canonical_id().unwrap();
    receipt.attestation.signature_hex =
        hex(key.sign(&receipt.attestation_bytes().unwrap()).as_ref());
    serde_json::to_vec(&receipt).unwrap()
}

#[test]
fn signed_product_receipt_authenticates_manifest_and_native_library_expectation() {
    let key = key(7);
    let bytes = sign(fixture(), "worker-old", &key);
    let trust = trust(
        &[("worker-old", &key, false)],
        vec![key_policy("worker-old")],
    );
    let proof =
        verify_native_package_receipts(&[&bytes], MANIFEST.as_bytes(), &trust, &policy()).unwrap();
    let (expectations, valid_until) = proof.into_parts();
    assert_eq!(valid_until, time("2026-09-06T00:00:00Z"));
    assert_eq!(expectations.len(), 1);
    assert_eq!(expectations[0].manifest_digest, digest(MANIFEST.as_bytes()));
    assert_eq!(expectations[0].library_digest, digest(b"dll"));
    assert_eq!(
        expectations[0].capabilities,
        vec!["weather.read", "weather.write"]
    );
    assert!(
        matches!(&expectations[0].trust,NativePluginArtifactTrust::SignedPackage{signer_id,..} if signer_id=="worker-old")
    );
}

#[test]
fn signed_receipt_rejects_altered_signature_manifest_and_dll_digest() {
    let key = key(7);
    let bytes = sign(fixture(), "worker-old", &key);
    let trust = trust(
        &[("worker-old", &key, false)],
        vec![key_policy("worker-old")],
    );
    let mut receipt: ProductReceipt = serde_json::from_slice(&bytes).unwrap();
    receipt.attestation.signature_hex.replace_range(0..2, "00");
    assert!(matches!(
        verify_native_package_receipts(
            &[&serde_json::to_vec(&receipt).unwrap()],
            MANIFEST.as_bytes(),
            &trust,
            &policy()
        ),
        Err(NativePackageReceiptError::Signature)
    ));
    assert!(matches!(
        verify_native_package_receipts(
            &[&bytes],
            MANIFEST.replace("weather.read", "weather.admin").as_bytes(),
            &trust,
            &policy()
        ),
        Err(NativePackageReceiptError::Artifact(_))
    ));
    let mut receipt: ProductReceipt = serde_json::from_slice(&bytes).unwrap();
    receipt.runtime_dependencies[1].sha256 = "0".repeat(64);
    assert!(verify_native_package_receipts(
        &[&serde_json::to_vec(&receipt).unwrap()],
        MANIFEST.as_bytes(),
        &trust,
        &policy()
    )
    .is_err());
}

#[test]
fn authenticated_proof_becomes_authority_and_expiry_is_rechecked_before_file_access() {
    use super::super::{
        NativePluginArtifactAdmissionError, NativePluginArtifactAuthority, NativePluginCandidate,
    };
    let key = key(7);
    let bytes = sign(fixture(), "worker-old", &key);
    let trust = trust(
        &[("worker-old", &key, false)],
        vec![key_policy("worker-old")],
    );
    let mut proof =
        verify_native_package_receipts(&[&bytes], MANIFEST.as_bytes(), &trust, &policy()).unwrap();
    let forged = proof.expectations.clone();
    assert!(matches!(
        NativePluginArtifactAuthority::from_expectations(forged),
        Err(NativePluginArtifactAdmissionError::SignatureVerificationUnavailable)
    ));
    // Model time passing after verification without waiting or opening a candidate file.
    proof.valid_until = Utc::now() - chrono::Duration::seconds(1);
    let authority = NativePluginArtifactAuthority::from_verified_package(proof).unwrap();
    let candidate = NativePluginCandidate {
        plugin_id: "weather".into(),
        package_manifest: toml::from_str(MANIFEST).unwrap(),
        manifest_path: "never-opened.toml".into(),
        library_path: "never-opened.dll".into(),
    };
    assert!(matches!(
        authority.admit(
            &candidate,
            &candidate.library_path,
            &[PluginModuleKind::Runtime]
        ),
        Err(NativePluginArtifactAdmissionError::AuthorityExpired { .. })
    ));
}

#[test]
fn verified_package_proofs_aggregate_with_the_earliest_deadline() {
    use super::super::{
        NativePluginArtifactAdmissionError, NativePluginArtifactAuthority,
        NativePluginArtifactDependency, NativePluginArtifactDigest,
        NativePluginArtifactExpectation, NativePluginArtifactTarget, NativePluginArtifactTrust,
        NativePluginCandidate, VerifiedNativePackageProof,
    };

    fn proof(
        plugin_id: &str,
        module_kind: PluginModuleKind,
        valid_until: DateTime<Utc>,
    ) -> VerifiedNativePackageProof {
        VerifiedNativePackageProof {
            expectations: vec![NativePluginArtifactExpectation {
                plugin_id: plugin_id.into(),
                package_id: format!("{plugin_id}-package"),
                manifest_digest: NativePluginArtifactDigest {
                    sha256: "a".repeat(64),
                    byte_length: 8,
                },
                library_digest: NativePluginArtifactDigest {
                    sha256: "b".repeat(64),
                    byte_length: 16,
                },
                trust: NativePluginArtifactTrust::SignedPackage {
                    signer_id: "trusted-signer".into(),
                    algorithm: "ed25519-v1".into(),
                },
                target: NativePluginArtifactTarget::new(
                    RuntimeTargetMode::ClientRuntime,
                    ExportTargetPlatform::Windows,
                ),
                module_kinds: vec![module_kind],
                capabilities: vec!["weather.read".into()],
                dependencies: Vec::<NativePluginArtifactDependency>::new(),
            }],
            valid_until,
        }
    }

    let expired = Utc::now() - chrono::Duration::seconds(1);
    let later = Utc::now() + chrono::Duration::minutes(5);
    let authority = NativePluginArtifactAuthority::from_verified_packages([
        proof("weather", PluginModuleKind::Runtime, expired),
        proof("cloud", PluginModuleKind::Editor, later),
    ])
    .unwrap();

    assert!(authority.expectation("weather").is_some());
    assert!(authority.expectation("cloud").is_some());
    let candidate = NativePluginCandidate {
        plugin_id: "cloud".into(),
        package_manifest: toml::from_str(&MANIFEST.replace("weather", "cloud")).unwrap(),
        manifest_path: "never-opened.toml".into(),
        library_path: "never-opened.dll".into(),
    };
    assert!(matches!(
        authority.admit(
            &candidate,
            &candidate.library_path,
            &[PluginModuleKind::Editor]
        ),
        Err(NativePluginArtifactAdmissionError::AuthorityExpired { plugin_id })
            if plugin_id == "cloud"
    ));
}

#[test]
fn verified_package_aggregation_rejects_duplicate_plugin_identity_across_packages() {
    use super::super::{
        NativePluginArtifactAdmissionError, NativePluginArtifactAuthority,
        NativePluginArtifactDependency, NativePluginArtifactDigest,
        NativePluginArtifactExpectation, NativePluginArtifactTarget, NativePluginArtifactTrust,
        VerifiedNativePackageProof,
    };

    let expectation = |module_kind| NativePluginArtifactExpectation {
        plugin_id: "weather".into(),
        package_id: "weather-package".into(),
        manifest_digest: NativePluginArtifactDigest {
            sha256: "a".repeat(64),
            byte_length: 8,
        },
        library_digest: NativePluginArtifactDigest {
            sha256: "b".repeat(64),
            byte_length: 16,
        },
        trust: NativePluginArtifactTrust::SignedPackage {
            signer_id: "trusted-signer".into(),
            algorithm: "ed25519-v1".into(),
        },
        target: NativePluginArtifactTarget::new(
            RuntimeTargetMode::ClientRuntime,
            ExportTargetPlatform::Windows,
        ),
        module_kinds: vec![module_kind],
        capabilities: vec!["weather.read".into()],
        dependencies: Vec::<NativePluginArtifactDependency>::new(),
    };
    let proof = |module_kind| VerifiedNativePackageProof {
        expectations: vec![expectation(module_kind)],
        valid_until: Utc::now() + chrono::Duration::minutes(5),
    };

    assert!(matches!(
        NativePluginArtifactAuthority::from_verified_packages([
            proof(PluginModuleKind::Runtime),
            proof(PluginModuleKind::Editor),
        ]),
        Err(NativePluginArtifactAdmissionError::DuplicateAuthorityIdentity { plugin_id })
            if plugin_id == "weather"
    ));
}

#[test]
fn signed_receipt_denies_identity_target_module_and_capability_escalation() {
    let key = key(7);
    let bytes = sign(fixture(), "worker-old", &key);
    let trust = trust(
        &[("worker-old", &key, false)],
        vec![key_policy("worker-old")],
    );
    for mutation in 0..9 {
        let mut policy = policy();
        match mutation {
            0 => policy.plugin_id = "other".into(),
            1 => policy.package_version = "2.0.0".into(),
            2 => policy.target_triple = "aarch64-unknown-linux-gnu".into(),
            3 => policy.target.runtime_mode = RuntimeTargetMode::ServerRuntime,
            4 => policy.target.platform = ExportTargetPlatform::Linux,
            5 => policy.required_capabilities.push("weather.admin".into()),
            6 => policy.allowed_capabilities.clear(),
            7 => policy.modules[0].logical_name = "other-dll".into(),
            _ => policy.modules.push(policy.modules[0].clone()),
        }
        assert!(
            verify_native_package_receipts(&[&bytes], MANIFEST.as_bytes(), &trust, &policy)
                .is_err(),
            "mutation {mutation}"
        );
    }
}

#[cfg(windows)]
#[test]
fn signed_real_dll_closure_admits_normal_and_rejects_tamper_before_entry() {
    use super::super::{
        native_artifact_staging::tests::DllFixture, NativePluginArtifactAuthority,
        NativePluginCandidate,
    };
    let dll = DllFixture::build();
    let manifest_path = dll.root.join("plugin.toml");
    let manifest_bytes = std::fs::read(&manifest_path).unwrap();
    let manifest: PluginPackageManifest =
        toml::from_str(std::str::from_utf8(&manifest_bytes).unwrap()).unwrap();
    let key = key(17);
    let now = Utc::now();
    let mut key_policy = key_policy("worker-signed-native");
    key_policy.not_before = now - chrono::Duration::hours(1);
    key_policy.not_after = now + chrono::Duration::hours(1);
    key_policy.allowed_package_ids = vec![manifest.package_id().to_string()];
    let registry = serde_json::to_vec(&serde_json::json!({"schema_version":1,"trust_registry_kind":"zircon_product_receipt_trust_registry","issuers":[{"signer_id":"worker-signed-native","algorithm":"ed25519-v1","public_key_hex":hex(key.public_key().as_ref()),"disabled":false}]})).unwrap();
    let trust = NativePackageReceiptTrust::from_registry_json(
        &registry,
        vec![key_policy],
        now + chrono::Duration::hours(1),
        16_384,
    )
    .unwrap();
    let mut policy = policy();
    policy.plugin_id = manifest.id.clone();
    policy.package_id = manifest.package_id().to_string();
    policy.package_version = manifest.version.to_string();
    policy.sdk_api_version = manifest.sdk_api_version.to_string();
    policy.now = now;
    policy.manifest_relative_path = "plugins/fixture/plugin.toml".into();
    policy.required_capabilities.clear();
    policy.allowed_capabilities.clear();
    policy.modules[0].relative_path = format!(
        "plugins/fixture/native/{}",
        dll.main.file_name().unwrap().to_str().unwrap()
    );
    let dependency_path = dll.main.parent().unwrap().join(&dll.dependency.file_name);
    let dependency_relative = format!("plugins/fixture/native/{}", dll.dependency.file_name);
    policy.modules[0].dependencies = vec![NativePackageDependencyArtifact {
        logical_name: "fixture-dependency".into(),
        relative_path: dependency_relative.clone(),
    }];
    let mut receipt = fixture();
    receipt.created_utc = (now - chrono::Duration::seconds(1)).to_rfc3339();
    receipt.runtime_dependencies = vec![
        artifact(
            &policy.manifest_logical_name,
            &policy.manifest_relative_path,
            "resource",
            &manifest_bytes,
        ),
        artifact(
            &policy.modules[0].logical_name,
            &policy.modules[0].relative_path,
            "dynamic_library",
            &std::fs::read(&dll.main).unwrap(),
        ),
        artifact(
            "fixture-dependency",
            &dependency_relative,
            "dynamic_library",
            &std::fs::read(&dependency_path).unwrap(),
        ),
    ];
    let receipt_bytes = sign(receipt, "worker-signed-native", &key);
    let proof = verify_native_package_receipts(&[&receipt_bytes], &manifest_bytes, &trust, &policy)
        .unwrap();
    let authority = NativePluginArtifactAuthority::from_verified_package(proof).unwrap();
    let candidate = NativePluginCandidate {
        plugin_id: manifest.id.clone(),
        package_manifest: manifest,
        manifest_path,
        library_path: dll.main.clone(),
    };
    let original_dependency = std::fs::read(&dependency_path).unwrap();
    std::fs::write(&dependency_path, b"modified signed dependency").unwrap();
    assert!(authority
        .admit(&candidate, &dll.main, &[PluginModuleKind::Runtime])
        .is_err());
    assert!(!dll.marker.exists());
    std::fs::write(&dependency_path, original_dependency).unwrap();
    let original_main = std::fs::read(&dll.main).unwrap();
    std::fs::write(&dll.main, b"modified signed main").unwrap();
    assert!(authority
        .admit(&candidate, &dll.main, &[PluginModuleKind::Runtime])
        .is_err());
    assert!(!dll.marker.exists());
    std::fs::write(&dll.main, original_main).unwrap();
    let admission = authority
        .admit(&candidate, &dll.main, &[PluginModuleKind::Runtime])
        .unwrap();
    assert!(!dll.marker.exists());
    let library = unsafe {
        libloading::os::windows::Library::load_with_flags(
            admission.admitted_library_path(),
            0x100 | 0x800,
        )
    }
    .unwrap();
    assert!(dll.marker.exists());
    drop(library);
    drop(admission);
}

#[test]
fn signed_receipt_rotation_and_revocation_use_host_key_policy() {
    let old = key(7);
    let new = key(8);
    let mut old_policy = key_policy("worker-old");
    old_policy.revoked = true;
    let trust = trust(
        &[("worker-old", &old, false), ("worker-new", &new, false)],
        vec![old_policy, key_policy("worker-new")],
    );
    assert!(matches!(
        verify_native_package_receipts(
            &[&sign(fixture(), "worker-old", &old)],
            MANIFEST.as_bytes(),
            &trust,
            &policy()
        ),
        Err(NativePackageReceiptError::UntrustedIssuer)
    ));
    assert!(verify_native_package_receipts(
        &[&sign(fixture(), "worker-new", &new)],
        MANIFEST.as_bytes(),
        &trust,
        &policy()
    )
    .is_ok());
    assert!(matches!(
        verify_native_package_receipts(
            &[&sign(fixture(), "unknown", &old)],
            MANIFEST.as_bytes(),
            &trust,
            &policy()
        ),
        Err(NativePackageReceiptError::UntrustedIssuer)
    ));
}

#[test]
fn signed_receipt_rejects_disabled_expired_future_and_wrong_package_keys() {
    let key = key(7);
    let bytes = sign(fixture(), "worker-old", &key);
    for mutation in 0..5 {
        let mut key_policy = key_policy("worker-old");
        match mutation {
            1 => key_policy.not_after = time("2026-09-04T00:00:00Z"),
            2 => key_policy.not_before = time("2026-09-06T00:00:00Z"),
            3 => key_policy.allowed_package_ids = vec!["other".into()],
            _ => {}
        }
        let trust = trust(&[("worker-old", &key, mutation == 0)], vec![key_policy]);
        let mut policy = policy();
        if mutation == 4 {
            policy.now = time("2026-09-06T00:00:00Z");
        }
        assert!(matches!(
            verify_native_package_receipts(&[&bytes], MANIFEST.as_bytes(), &trust, &policy),
            Err(NativePackageReceiptError::UntrustedIssuer)
        ));
    }
    let trust = trust(
        &[("worker-old", &key, false)],
        vec![key_policy("worker-old")],
    );
    let mut policy = policy();
    policy.max_receipt_age_seconds = 1;
    assert!(matches!(
        verify_native_package_receipts(&[&bytes], MANIFEST.as_bytes(), &trust, &policy),
        Err(NativePackageReceiptError::ReceiptTime)
    ));
    policy.now = time("2026-09-05T07:00:00Z");
    assert!(matches!(
        verify_native_package_receipts(&[&bytes], MANIFEST.as_bytes(), &trust, &policy),
        Err(NativePackageReceiptError::ReceiptTime)
    ));
}

#[test]
fn signed_receipt_accepts_wire_normalization_without_changing_identity() {
    let key = key(7);
    let bytes = sign(fixture(), "worker-old", &key);
    let mut receipt: ProductReceipt = serde_json::from_slice(&bytes).unwrap();
    receipt.runtime_dependencies.reverse();
    for artifact in &mut receipt.runtime_dependencies {
        artifact.sha256.make_ascii_lowercase();
    }
    let trust = trust(
        &[("worker-old", &key, false)],
        vec![key_policy("worker-old")],
    );
    assert!(verify_native_package_receipts(
        &[&serde_json::to_vec(&receipt).unwrap()],
        MANIFEST.as_bytes(),
        &trust,
        &policy()
    )
    .is_ok());
}

#[test]
fn signed_receipt_authenticates_multiple_modules_and_dependency_dlls() {
    let key = key(7);
    let manifest = format!("{MANIFEST}\n[[modules]]\nname = \"WeatherEditor\"\nkind = \"editor\"\ncrate_name = \"weather_editor\"\n");
    let mut receipt = fixture();
    receipt.runtime_dependencies[0] = artifact(
        "weather-manifest",
        "plugins/weather/plugin.toml",
        "resource",
        manifest.as_bytes(),
    );
    receipt.runtime_dependencies.push(artifact(
        "weather-editor",
        "plugins/weather/weather_editor.dll",
        "dynamic_library",
        b"editor-dll",
    ));
    receipt.runtime_dependencies.push(artifact(
        "weather-support",
        "plugins/weather/support.dll",
        "dynamic_library",
        b"support-dll",
    ));
    let bytes = sign(receipt, "worker-old", &key);
    let trust = trust(
        &[("worker-old", &key, false)],
        vec![key_policy("worker-old")],
    );
    let mut policy = policy();
    policy.modules[0]
        .dependencies
        .push(NativePackageDependencyArtifact {
            logical_name: "weather-support".into(),
            relative_path: "plugins/weather/support.dll".into(),
        });
    policy.modules.push(NativePackageModuleArtifact {
        module_kind: PluginModuleKind::Editor,
        logical_name: "weather-editor".into(),
        relative_path: "plugins/weather/weather_editor.dll".into(),
        dependencies: vec![],
    });
    let proof =
        verify_native_package_receipts(&[&bytes], manifest.as_bytes(), &trust, &policy).unwrap();
    let (expectations, _) = proof.into_parts();
    assert_eq!(expectations.len(), 2);
    assert_eq!(expectations[0].dependencies[0].file_name, "support.dll");
    assert_eq!(
        expectations[0].dependencies[0].digest,
        digest(b"support-dll")
    );
    assert_eq!(expectations[1].library_digest, digest(b"editor-dll"));
    policy.modules[0].dependencies[0].logical_name = "unsigned-support".into();
    assert!(matches!(
        verify_native_package_receipts(&[&bytes], manifest.as_bytes(), &trust, &policy),
        Err(NativePackageReceiptError::Artifact(_))
    ));
}

#[test]
fn signed_receipt_proof_deadline_keeps_key_and_receipt_expiry() {
    let key = key(7);
    let bytes = sign(fixture(), "worker-old", &key);
    let mut issuer = key_policy("worker-old");
    issuer.not_after = time("2026-09-05T13:00:00Z");
    let trust = trust(&[("worker-old", &key, false)], vec![issuer]);
    let (_, deadline) =
        verify_native_package_receipts(&[&bytes], MANIFEST.as_bytes(), &trust, &policy())
            .unwrap()
            .into_parts();
    assert_eq!(deadline, time("2026-09-05T13:00:00Z"));
    let mut policy = policy();
    policy.max_receipt_age_seconds = 4 * 60 * 60 + 1;
    let (_, deadline) =
        verify_native_package_receipts(&[&bytes], MANIFEST.as_bytes(), &trust, &policy)
            .unwrap()
            .into_parts();
    assert_eq!(deadline, time("2026-09-05T12:00:01Z"));
    policy.max_receipt_age_seconds = u64::MAX;
    assert!(matches!(
        verify_native_package_receipts(&[&bytes], MANIFEST.as_bytes(), &trust, &policy),
        Err(NativePackageReceiptError::Policy(_))
    ));
}

#[test]
fn runtime_ring_verifies_independent_product_receipt_wire_fixture() {
    let receipt_bytes = include_bytes!("../fixtures/receipt.json");
    let manifest: String =
        serde_json::from_slice(include_bytes!("../fixtures/manifest-bytes.json")).unwrap();
    let registry = include_bytes!("../fixtures/trust-registry.json");
    let trust = NativePackageReceiptTrust::from_registry_json(
        registry,
        vec![key_policy("worker-old")],
        time("2026-09-06T00:00:00Z"),
        16_384,
    )
    .unwrap();
    let mut receipt: ProductReceipt = serde_json::from_slice(receipt_bytes).unwrap();
    receipt.verify_integrity().unwrap();
    assert_eq!(
        receipt.receipt_id,
        "20C28D5AF8C448C86F30095D7A1BF0DA8BCC15EBA139FD729EF9EE47F54C0DCB"
    );
    assert_eq!(
        receipt.toolchain.toolchain_set_id,
        "1A15D846E8810546D756F418881A546812109B1163E60949D615A335727226BA"
    );
    assert_eq!(
        receipt.attestation_bytes().unwrap(),
        include_str!("../fixtures/canonical-attestation.json")
            .trim()
            .as_bytes()
    );
    let (expected, _) =
        verify_native_package_receipts(&[receipt_bytes], manifest.as_bytes(), &trust, &policy())
            .unwrap()
            .into_parts();
    assert_eq!(expected[0].library_digest, digest(b"dll"));
    receipt.attestation.signature_hex.replace_range(
        0..2,
        if &receipt.attestation.signature_hex[..2] == "00" {
            "01"
        } else {
            "00"
        },
    );
    let altered = serde_json::to_vec(&receipt).unwrap();
    assert!(matches!(
        verify_native_package_receipts(&[&altered], manifest.as_bytes(), &trust, &policy()),
        Err(NativePackageReceiptError::Signature)
    ));
}
