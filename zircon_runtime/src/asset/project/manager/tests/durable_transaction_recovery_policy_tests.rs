use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime_interface::{project::RelPath, resource::ResourceId};

use super::{ProjectPaths, ProjectRecoveryPolicy};

static NEXT_TEST_ROOT: AtomicU64 = AtomicU64::new(1);

#[test]
fn physical_asset_target_does_not_authorize_an_outside_raw_directory_entry() {
    let fixture = unique_test_root("raw-entry-containment");
    let project_root = fixture.join("project");
    let outside_root = fixture.join("outside");
    fs::create_dir_all(&project_root).unwrap();
    fs::create_dir_all(&outside_root).unwrap();
    let asset_root = RelPath::parse("assets").unwrap();
    let paths = ProjectPaths::from_root(&project_root).unwrap();
    paths
        .ensure_layout(std::slice::from_ref(&asset_root))
        .unwrap();
    let policy = ProjectRecoveryPolicy::new(&paths, &[asset_root]).unwrap();
    let physical_target =
        ProjectPaths::resolve_identity(project_root.join("assets/panel.zui.zmeta")).unwrap();
    let outside_entry = outside_root.join("panel.zui.zmeta");

    assert!(policy.is_relocatable_project_file(&physical_target));
    assert!(!policy
        .is_relocatable_project_entry(&outside_entry, &physical_target)
        .unwrap());

    fs::remove_dir_all(fixture).unwrap();
}

#[test]
fn canonical_publication_target_does_not_authorize_a_different_raw_leaf_layout() {
    let fixture = unique_test_root("raw-leaf-layout");
    let project_root = fixture.join("project");
    fs::create_dir_all(&project_root).unwrap();
    let asset_root = RelPath::parse("assets").unwrap();
    let paths = ProjectPaths::from_root(&project_root).unwrap();
    paths
        .ensure_layout(std::slice::from_ref(&asset_root))
        .unwrap();
    let policy = ProjectRecoveryPolicy::new(&paths, &[asset_root]).unwrap();

    let import_target =
        ProjectPaths::resolve_identity(project_root.join("assets/models/source.obj")).unwrap();
    assert!(policy.is_import_source_plan_target(&import_target));
    assert!(!policy
        .is_import_source_plan_entry(&project_root.join("assets/arbitrary.txt"), &import_target,)
        .unwrap());

    let artifact_name = format!(
        "{}.zasset",
        ResourceId::from_stable_label("raw-leaf-layout-artifact")
    );
    let artifact_target = ProjectPaths::resolve_identity(
        paths
            .asset_artifact_root()
            .join("models")
            .join(artifact_name),
    )
    .unwrap();
    assert!(policy.is_artifact_manifest(&artifact_target));
    assert!(!policy
        .is_artifact_manifest_entry(
            &paths.asset_artifact_root().join("models/arbitrary.cache"),
            &artifact_target,
        )
        .unwrap());

    fs::remove_dir_all(fixture).unwrap();
}

fn unique_test_root(label: &str) -> PathBuf {
    let root = test_output_root().join(format!(
        "zircon-project-recovery-{label}-{}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after Unix epoch")
            .as_nanos(),
        NEXT_TEST_ROOT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&root).unwrap();
    root
}

fn test_output_root() -> PathBuf {
    std::env::var_os("ZIRCON_TEST_OUTPUT_ROOT")
        .or_else(|| std::env::var_os("CARGO_TARGET_DIR"))
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::current_dir()
                .expect("resolve current workspace for project recovery test output")
                .join("target")
        })
        .join("zircon-test-output")
}
