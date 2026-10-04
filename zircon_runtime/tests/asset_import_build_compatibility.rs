use std::collections::BTreeMap;

#[path = "../build/asset_import_context/emission.rs"]
mod emission;
#[path = "../build/asset_import_context/fingerprint.rs"]
mod fingerprint;

use fingerprint::{canonical_configuration, fingerprint};

fn configuration() -> BTreeMap<String, String> {
    canonical_configuration([
        ("TARGET".into(), "x86_64-pc-windows-msvc".into()),
        ("HOST".into(), "x86_64-pc-windows-msvc".into()),
        ("PROFILE".into(), "release".into()),
        ("OPT_LEVEL".into(), "2".into()),
        ("DEBUG".into(), "false".into()),
        ("CARGO_PKG_VERSION".into(), "0.1.0".into()),
        ("CARGO_CFG_FEATURE".into(), "asset,graphics".into()),
        ("CARGO_CFG_TARGET_FEATURE".into(), "sse,sse2".into()),
        ("CARGO_ENCODED_RUSTFLAGS".into(), String::new()),
    ])
}

#[test]
fn import_build_emission_binds_generated_names_to_their_qualified_values() {
    let configuration = configuration();
    let compiler = b"rustc A commit:123";
    let directives = emission::directives(compiler, &configuration).unwrap();
    assert_eq!(
        directives,
        [
            "cargo:rustc-env=ZR_ASSET_IMPORT_BUILD_TARGET=x86_64-pc-windows-msvc".to_owned(),
            format!(
                "cargo:rustc-env=ZR_ASSET_IMPORT_BUILD_ID={}",
                fingerprint(compiler, &configuration)
            ),
        ]
    );
}

#[test]
fn import_build_emission_rejects_missing_or_invalid_identity_inputs() {
    let configuration = configuration();
    for required in [
        "TARGET",
        "HOST",
        "PROFILE",
        "OPT_LEVEL",
        "DEBUG",
        "CARGO_PKG_VERSION",
    ] {
        let mut missing = configuration.clone();
        missing.remove(required);
        assert!(
            emission::directives(b"rustc A", &missing).is_err(),
            "{required}"
        );
        missing.insert(required.to_owned(), String::new());
        assert!(
            emission::directives(b"rustc A", &missing).is_err(),
            "{required}"
        );
    }
    assert!(emission::directives(b"", &configuration).is_err());
    let mut invalid = configuration;
    invalid.insert(
        "TARGET".to_owned(),
        "target\ncargo:rustc-env=OTHER=value".to_owned(),
    );
    assert!(emission::directives(b"rustc A", &invalid).is_err());
}

#[test]
fn import_build_identity_distinguishes_compiler_target_features_and_codegen() {
    let baseline = configuration();
    let key = fingerprint(b"rustc A commit:123", &baseline);
    assert_ne!(key, fingerprint(b"rustc A commit:456", &baseline));

    for (name, value) in [
        ("TARGET", "x86_64-pc-windows-gnu"),
        ("HOST", "x86_64-unknown-linux-gnu"),
        ("OPT_LEVEL", "3"),
        ("DEBUG", "true"),
        ("CARGO_PKG_VERSION", "0.2.0"),
        ("CARGO_CFG_FEATURE", "asset"),
        ("CARGO_CFG_TARGET_FEATURE", "avx2,sse,sse2"),
        ("CARGO_ENCODED_RUSTFLAGS", "-C\u{1f}target-cpu=native"),
    ] {
        let mut changed = baseline.clone();
        changed.insert(name.to_owned(), value.to_owned());
        assert_ne!(key, fingerprint(b"rustc A commit:123", &changed), "{name}");
    }
}

#[test]
fn import_build_identity_ignores_environment_order_and_workspace_paths() {
    let baseline = configuration();
    let mut reordered: Vec<_> = baseline.clone().into_iter().rev().collect();
    reordered.push(("OUT_DIR".into(), "different-build-root".into()));
    reordered.push(("CARGO_MANIFEST_DIR".into(), "different-checkout".into()));
    for (name, value) in &mut reordered {
        if name == "CARGO_CFG_FEATURE" {
            *value = "graphics,asset".into();
        }
        if name == "CARGO_CFG_TARGET_FEATURE" {
            *value = "sse2,sse".into();
        }
    }
    assert_eq!(
        fingerprint(b"rustc A", &baseline),
        fingerprint(b"rustc A", &canonical_configuration(reordered))
    );
}

#[test]
fn import_build_identity_preserves_flag_order_and_field_boundaries() {
    let mut first = configuration();
    first.insert(
        "CARGO_ENCODED_RUSTFLAGS".into(),
        "-C\u{1f}opt-level=2\u{1f}-C\u{1f}opt-level=3".into(),
    );
    let mut second = first.clone();
    second.insert(
        "CARGO_ENCODED_RUSTFLAGS".into(),
        "-C\u{1f}opt-level=3\u{1f}-C\u{1f}opt-level=2".into(),
    );
    assert_ne!(
        fingerprint(b"rustc A", &first),
        fingerprint(b"rustc A", &second)
    );

    first.insert("HOST".into(), "ab".into());
    first.insert("TARGET".into(), "c".into());
    second = first.clone();
    second.insert("HOST".into(), "a".into());
    second.insert("TARGET".into(), "bc".into());
    assert_ne!(
        fingerprint(b"rustc A", &first),
        fingerprint(b"rustc A", &second)
    );
}
