use std::collections::BTreeSet;

use serde::Serialize;
use zircon_runtime_interface::runtime_build_set::{
    current_runtime_payload_schema_set_digest, ZrRuntimeArtifactIdentityV1, ZrRuntimeBuildSetId,
    ZrRuntimeInterfaceSpecV1, ZR_RUNTIME_ARTIFACT_MANIFEST_SCHEMA_V1,
};

use super::*;

const PLACEHOLDER_BUILD_SET: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

#[test]
fn valid_staged_build_recomputes_every_required_artifact_digest() {
    let fixture = Fixture::new("valid");
    fixture.write("debug");

    let qualified = validate_staged_editor_runtime_build(
        &fixture.staged_engine_dir,
        &fixture.source_dir,
        "debug",
    )
    .expect("fixture should qualify");

    assert_eq!(qualified.build_set_id.len(), 64);
    fixture.remove();
}

#[test]
fn staged_build_rejects_an_editor_replaced_after_manifest_creation() {
    let fixture = Fixture::new("tampered-editor");
    fixture.write("debug");
    fs::write(
        fixture
            .staged_engine_dir
            .join(platform_editor_executable_name()),
        b"replaced editor",
    )
    .unwrap();

    let error = validate_staged_editor_runtime_build(
        &fixture.staged_engine_dir,
        &fixture.source_dir,
        "debug",
    )
    .err()
    .expect("tampered editor should be rejected");

    assert!(error
        .to_string()
        .contains("does not match its staged SHA-256"));
    fixture.remove();
}

#[test]
fn staged_build_rejects_a_manifest_for_another_profile() {
    let fixture = Fixture::new("wrong-profile");
    fixture.write("release");

    let error = validate_staged_editor_runtime_build(
        &fixture.staged_engine_dir,
        &fixture.source_dir,
        "debug",
    )
    .err()
    .expect("wrong profile should be rejected");

    assert!(error
        .to_string()
        .contains("does not match requested profile debug"));
    fixture.remove();
}

#[test]
fn qualified_staged_build_replaces_the_active_directory_atomically() {
    let fixture = Fixture::new("atomic-activation");
    fixture.write("debug");
    let active_engine_dir = fixture.root.join("out").join("ZirconEngine-active");
    fs::create_dir_all(&active_engine_dir).unwrap();
    fs::write(active_engine_dir.join("old-build.marker"), b"old").unwrap();

    activate_staged_engine(&fixture.staged_engine_dir, &active_engine_dir)
        .expect("qualified staging should become the active BuildSet");

    assert!(active_engine_dir
        .join(platform_editor_executable_name())
        .is_file());
    assert!(!active_engine_dir.join("old-build.marker").exists());
    assert!(!fixture.staged_engine_dir.exists());
    fixture.remove();
}

pub(super) fn write_valid_fixture(staged_engine_dir: &Path, source_dir: &Path, profile: &str) {
    write_fixture(staged_engine_dir, source_dir, profile);
}

struct Fixture {
    root: PathBuf,
    source_dir: PathBuf,
    staged_engine_dir: PathBuf,
}

impl Fixture {
    fn new(label: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "zircon-hub-staged-build-{label}-{}-{}",
            std::process::id(),
            crate::projects::now_unix_ms()
        ));
        let _ = fs::remove_dir_all(&root);
        let source_dir = root.join("source");
        let staged_engine_dir = root.join("out").join("ZirconEngine");
        fs::create_dir_all(&source_dir).unwrap();
        Self {
            root,
            source_dir,
            staged_engine_dir,
        }
    }

    fn write(&self, profile: &str) {
        write_fixture(&self.staged_engine_dir, &self.source_dir, profile);
    }

    fn remove(&self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}

fn write_fixture(staged_engine_dir: &Path, source_dir: &Path, profile: &str) {
    fs::create_dir_all(staged_engine_dir).unwrap();
    let runtime_library_name = platform_runtime_library_name();
    let editor_name = platform_editor_executable_name();
    let runtime_name = platform_runtime_executable_name();
    for (name, bytes) in [
        (runtime_library_name, b"runtime-library".as_slice()),
        (editor_name, b"editor-executable".as_slice()),
        (runtime_name, b"runtime-executable".as_slice()),
    ] {
        fs::write(staged_engine_dir.join(name), bytes).unwrap();
    }

    let interface_spec = ZrRuntimeInterfaceSpecV1::current().unwrap();
    let mut runtime_manifest = ZrRuntimeArtifactManifestV1 {
        schema_version: ZR_RUNTIME_ARTIFACT_MANIFEST_SCHEMA_V1,
        build_set_id: ZrRuntimeBuildSetId::parse(PLACEHOLDER_BUILD_SET).unwrap(),
        build_mode: build_mode(profile).unwrap(),
        runtime_features: BTreeSet::from(["target-client".to_string()]),
        interface_spec_digest: interface_spec.digest().unwrap(),
        interface_spec,
        payload_schema_digest: current_runtime_payload_schema_set_digest(),
        target: ZrRuntimeTargetModelV1::current(),
        artifact: identity_for(staged_engine_dir, runtime_library_name),
        host_artifacts: vec![
            identity_for(staged_engine_dir, editor_name),
            identity_for(staged_engine_dir, runtime_name),
        ],
        capabilities: BTreeSet::new(),
    };
    runtime_manifest.build_set_id = runtime_manifest.derived_build_set_id().unwrap();
    let runtime_manifest_name = format!("{runtime_library_name}.manifest.json");
    let runtime_manifest_path = staged_engine_dir.join(&runtime_manifest_name);
    fs::write(
        &runtime_manifest_path,
        serde_json::to_vec_pretty(&runtime_manifest).unwrap(),
    )
    .unwrap();

    let artifacts = [
        artifact_entry(staged_engine_dir, editor_name, "editor.executable"),
        artifact_entry(staged_engine_dir, runtime_name, "runtime.executable"),
        artifact_entry(staged_engine_dir, runtime_library_name, "runtime.library"),
        artifact_entry(
            staged_engine_dir,
            &runtime_manifest_name,
            "runtime.library.manifest",
        ),
    ];
    let staging = TestStagingManifest {
        schema_version: STAGING_MANIFEST_SCHEMA_V1,
        source_repository: source_dir.canonicalize().unwrap(),
        build: TestStagingBuild {
            mode: profile,
            targets: ["editor", "runtime"],
            runtime_features: ["target-client"],
        },
        artifacts,
    };
    fs::write(
        staged_engine_dir.join(STAGING_MANIFEST_FILE_NAME),
        serde_json::to_vec_pretty(&staging).unwrap(),
    )
    .unwrap();
}

fn identity_for(staged_engine_dir: &Path, name: &str) -> ZrRuntimeArtifactIdentityV1 {
    ZrRuntimeArtifactIdentityV1::new(
        name,
        hash_regular_file(&staged_engine_dir.join(name)).unwrap(),
    )
    .unwrap()
}

fn artifact_entry(
    staged_engine_dir: &Path,
    target_path: &str,
    logical_artifact: &str,
) -> TestStagingArtifact {
    TestStagingArtifact {
        logical_artifact,
        source: TestStagingSource {
            kind: "build_artifact",
            path: target_path,
        },
        target_path,
        sha256: hash_regular_file(&staged_engine_dir.join(target_path)).unwrap(),
    }
}

#[derive(Serialize)]
struct TestStagingManifest<'a> {
    schema_version: u32,
    source_repository: PathBuf,
    build: TestStagingBuild<'a>,
    artifacts: [TestStagingArtifact<'a>; 4],
}

#[derive(Serialize)]
struct TestStagingBuild<'a> {
    mode: &'a str,
    targets: [&'a str; 2],
    runtime_features: [&'a str; 1],
}

#[derive(Serialize)]
struct TestStagingArtifact<'a> {
    logical_artifact: &'a str,
    source: TestStagingSource<'a>,
    target_path: &'a str,
    sha256: ZrRuntimeDigestV1,
}

#[derive(Serialize)]
struct TestStagingSource<'a> {
    kind: &'a str,
    path: &'a str,
}
