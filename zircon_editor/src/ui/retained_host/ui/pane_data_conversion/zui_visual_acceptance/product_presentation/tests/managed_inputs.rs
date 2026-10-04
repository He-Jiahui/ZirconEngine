use super::*;

#[test]
fn validation_rejects_swapped_sources_before_reading_files() {
    let root = std::env::current_dir().expect("test cwd");
    let value = ManagedSceneFingerprint {
        project_root: root.to_string_lossy().into_owned(),
        sources: vec![
            ManagedInputFingerprint {
                source_path: EMPTY_SCENE_SOURCE_PATH.into(),
                sha256: "a".repeat(64),
            },
            ManagedInputFingerprint {
                source_path: MAIN_SCENE_SOURCE_PATH.into(),
                sha256: "b".repeat(64),
            },
            ManagedInputFingerprint {
                source_path: PROJECT_MANIFEST_SOURCE_PATH.into(),
                sha256: "c".repeat(64),
            },
        ],
    };
    let error = validate_fingerprint(&value, &root).expect_err("swapped inputs must fail");
    assert!(error.contains("source 0 must be zircon-project.toml"));
}

#[test]
fn validation_rejects_changes_to_every_managed_input() {
    let root = temporary_project_root();
    let fingerprint = capture(&root).expect("capture managed project inputs");
    let value = serde_json::to_value(&fingerprint).expect("serialize fingerprint");
    validate_value(&value, &root).expect("fresh managed inputs must validate");

    for source in [
        PROJECT_MANIFEST_SOURCE_PATH,
        MAIN_SCENE_SOURCE_PATH,
        EMPTY_SCENE_SOURCE_PATH,
    ] {
        std::fs::write(root.join(source), format!("changed: {source}"))
            .expect("change managed input");
        let error = validate_value(&value, &root).expect_err("stale input must fail");
        assert!(error.contains(&format!(
            "managedSceneFingerprint source is stale: {source}"
        )));
        std::fs::write(root.join(source), format!("fixture: {source}"))
            .expect("restore managed input");
    }

    let approved_root = test_artifact_root();
    let physical_root = std::fs::canonicalize(&root).expect("canonical temporary project");
    assert_eq!(physical_root.parent(), Some(approved_root.as_path()));
    assert!(same_physical_path(&physical_root, &root));
    std::fs::remove_dir_all(&root).expect("remove temporary project");
}

fn temporary_project_root() -> std::path::PathBuf {
    use std::time::{SystemTime, UNIX_EPOCH};

    let approved_root = test_artifact_root();
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let root = approved_root.join(format!(
        "zircon-managed-inputs-{}-{nonce}",
        std::process::id()
    ));
    assert!(!root.exists(), "unique managed test directory must be new");
    for source in [
        PROJECT_MANIFEST_SOURCE_PATH,
        MAIN_SCENE_SOURCE_PATH,
        EMPTY_SCENE_SOURCE_PATH,
    ] {
        let path = root.join(source);
        std::fs::create_dir_all(path.parent().expect("source parent"))
            .expect("create source parent");
        std::fs::write(path, format!("fixture: {source}")).expect("write managed input fixture");
    }
    root
}

fn test_artifact_root() -> std::path::PathBuf {
    let configured = std::env::var_os("ZIRCON_MANAGED_INPUTS_TEST_ROOT")
        .expect("set ZIRCON_MANAGED_INPUTS_TEST_ROOT below approved D/E/F cargo-targets");
    let configured = std::path::PathBuf::from(configured);
    assert!(
        configured.is_absolute(),
        "test artifact root must be absolute"
    );
    let physical = std::fs::canonicalize(&configured).expect("canonical test artifact root");
    assert!(physical.is_dir(), "test artifact root must be a directory");
    assert!(
        same_physical_path(&physical, &configured),
        "test artifact root must not use aliases"
    );
    let normalized = normalize_physical(&physical);
    assert!(
        [
            "d:\\cargo-targets",
            "e:\\cargo-targets",
            "f:\\cargo-targets"
        ]
        .iter()
        .any(|root| normalized == *root || normalized.starts_with(&format!("{root}\\"))),
        "managed input tests may only write below approved D/E/F cargo-targets roots"
    );
    physical
}
