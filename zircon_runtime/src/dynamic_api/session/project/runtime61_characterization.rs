use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime_interface::project::{render_project_template, ProjectTemplateId};
use zircon_runtime_interface::ZrByteSlice;

use crate::asset::project::ProjectPaths;
use crate::scene::DynamicScene;

use super::super::{RuntimeDynamicSession, RuntimeDynamicSessionProfile};
use super::RuntimeProjectConfig;

const EMPTY_PLAY_SCENE: &str = ".zircon/play/instance/empty-play.zrscene.json";

#[test]
#[ignore = "Runtime61 M0 RED: exact Play restore contracts are not implemented"]
fn empty_versioned_play_snapshot_restores_an_exact_empty_runtime_world() {
    let fixture = EmptyPlaySnapshotProject::create();
    let root = fixture.root.to_string_lossy().into_owned();
    let config = RuntimeProjectConfig::from_abi_startup_config(
        byte_slice(root.as_bytes()),
        byte_slice(EMPTY_PLAY_SCENE.as_bytes()),
        ZrByteSlice::empty(),
    )
    .expect("empty Play snapshot startup config should be valid")
    .expect("project-backed startup should produce a project config");

    let session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, Some(config))
        .expect("headless product startup should load the empty Play snapshot");
    session.level.with_world(|world| {
        assert!(
            world.nodes().is_empty(),
            "exact Play restore must not inject entities absent from the snapshot; restored nodes: {:?}",
            world.nodes()
        );
    });

    drop(session);
    fixture.assert_removable();
}

fn byte_slice(bytes: &[u8]) -> ZrByteSlice {
    ZrByteSlice {
        data: bytes.as_ptr(),
        len: bytes.len(),
    }
}

struct EmptyPlaySnapshotProject {
    root: PathBuf,
}

impl EmptyPlaySnapshotProject {
    fn create() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after Unix epoch")
            .as_nanos();
        let root = fixture_root().join(format!(
            "empty-play-snapshot-{}_{}_{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed),
            unique
        ));
        write_template_project(&root);
        write_empty_play_snapshot(&root);
        Self { root }
    }

    fn assert_removable(&self) {
        std::fs::remove_dir_all(&self.root)
            .expect("Play snapshot fixture must not retain runtime-owned file handles");
        assert!(!self.root.exists());
    }
}

impl Drop for EmptyPlaySnapshotProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn fixture_root() -> PathBuf {
    let executable = std::env::current_exe().expect("locate test executable");
    let binary_directory = executable.parent().expect("test executable parent");
    ProjectPaths::resolve_existing(binary_directory)
        .expect("resolve test binary directory")
        .operation_path()
        .join("zircon-runtime61-characterization")
}

fn write_template_project(root: &Path) {
    let rendered =
        render_project_template(ProjectTemplateId::RenderableEmpty, "Runtime61EmptyPlay")
            .expect("render Runtime61 product fixture");
    for entry in rendered.entries {
        let destination = entry.path.join_to(root);
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent).expect("create template directory");
        }
        std::fs::write(destination, entry.bytes).expect("write template entry");
    }
    ProjectPaths::from_root(root)
        .expect("Runtime61 project paths")
        .ensure_derived_layout()
        .expect("Runtime61 project derived layout");
}

fn write_empty_play_snapshot(root: &Path) {
    let path = root.join(EMPTY_PLAY_SCENE);
    std::fs::create_dir_all(path.parent().expect("Play snapshot parent"))
        .expect("create Play snapshot directory");
    let document = DynamicScene::empty()
        .to_versioned_json_pretty()
        .expect("encode empty versioned Play snapshot");
    std::fs::write(path, document).expect("write empty Play snapshot");
}
