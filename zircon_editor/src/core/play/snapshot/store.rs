use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::project::ProjectPaths;
use zircon_runtime_interface::project::RelPath;

use super::PlaySceneSource;

#[derive(Debug, Default)]
pub struct PlaySnapshotStore {
    next_sequence: AtomicU64,
}

#[derive(Debug)]
pub struct MaterializedPlayScene {
    instance_id: String,
    path: PathBuf,
    relative_path: RelPath,
    owned_root: Option<PathBuf>,
}

#[derive(Debug)]
pub struct PlaySnapshotMaterializationFailure {
    message: String,
    cleanup_owner: Option<MaterializedPlayScene>,
}

impl PlaySnapshotStore {
    pub fn materialize(
        &self,
        project_root: &Path,
        source: &PlaySceneSource,
    ) -> Result<MaterializedPlayScene, PlaySnapshotMaterializationFailure> {
        let paths = ProjectPaths::from_root(project_root).map_err(|error| {
            PlaySnapshotMaterializationFailure::new(play_snapshot_path_error(
                "failed to resolve project root",
                project_root,
                error,
            ))
        })?;
        let instance_id = self.next_instance_id();
        match source {
            PlaySceneSource::Persisted(relative_path) => {
                let path = relative_path.join_to(paths.root());
                if !path.is_file() {
                    return Err(PlaySnapshotMaterializationFailure::new(format!(
                        "persisted play scene does not exist: {}",
                        display_play_snapshot_path(&path).display()
                    )));
                }
                Ok(MaterializedPlayScene {
                    instance_id,
                    path,
                    relative_path: relative_path.clone(),
                    owned_root: None,
                })
            }
            PlaySceneSource::Snapshot(document) => {
                let root = paths.play_root().join(&instance_id);
                fs::create_dir_all(&root).map_err(|error| {
                    PlaySnapshotMaterializationFailure::new(play_snapshot_path_error(
                        "failed to create play snapshot",
                        &root,
                        error,
                    ))
                })?;
                let final_path = root.join("play-scene.zrscene.json");
                let relative_path = RelPath::parse(format!(
                    ".zircon/play/{instance_id}/play-scene.zrscene.json"
                ))
                .expect("generated Play snapshot paths are project-relative");
                let temporary_path = root.join("play-scene.zrscene.json.tmp");
                let write_result = write_atomic_snapshot(&temporary_path, &final_path, document);
                let scene = MaterializedPlayScene {
                    instance_id,
                    path: final_path,
                    relative_path,
                    owned_root: Some(root),
                };
                complete_snapshot_write(scene, write_result)
            }
        }
    }

    fn next_instance_id(&self) -> String {
        let sequence = self.next_sequence.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos());
        format!("{}-{nanos}-{sequence}", std::process::id())
    }
}

impl PlaySnapshotMaterializationFailure {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            cleanup_owner: None,
        }
    }

    fn cleanup_pending(message: impl Into<String>, cleanup_owner: MaterializedPlayScene) -> Self {
        Self {
            message: message.into(),
            cleanup_owner: Some(cleanup_owner),
        }
    }

    pub fn into_parts(self) -> (Option<MaterializedPlayScene>, String) {
        (self.cleanup_owner, self.message)
    }
}

impl std::fmt::Display for PlaySnapshotMaterializationFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for PlaySnapshotMaterializationFailure {}

impl MaterializedPlayScene {
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn relative_path(&self) -> &RelPath {
        &self.relative_path
    }

    pub fn cleanup(&mut self) -> Result<(), String> {
        let Some(root) = self.owned_root.as_ref() else {
            return Ok(());
        };
        fs::remove_dir_all(root).map_err(|error| {
            play_snapshot_path_error("failed to clean play snapshot", root, error)
        })?;
        self.owned_root = None;
        Ok(())
    }
}

impl Drop for MaterializedPlayScene {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

fn complete_snapshot_write(
    mut scene: MaterializedPlayScene,
    write_result: Result<(), String>,
) -> Result<MaterializedPlayScene, PlaySnapshotMaterializationFailure> {
    match write_result {
        Ok(()) => Ok(scene),
        Err(write_error) => match scene.cleanup() {
            Ok(()) => Err(PlaySnapshotMaterializationFailure::new(write_error)),
            Err(cleanup_error) => Err(PlaySnapshotMaterializationFailure::cleanup_pending(
                format!("{write_error}; snapshot cleanup remains pending: {cleanup_error}"),
                scene,
            )),
        },
    }
}

fn write_atomic_snapshot(temporary: &Path, target: &Path, document: &str) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(temporary)
        .map_err(|error| {
            play_snapshot_path_error(
                "failed to create play snapshot temporary file",
                temporary,
                error,
            )
        })?;
    file.write_all(document.as_bytes()).map_err(|error| {
        play_snapshot_path_error(
            "failed to write play snapshot temporary file",
            temporary,
            error,
        )
    })?;
    file.sync_all().map_err(|error| {
        play_snapshot_path_error(
            "failed to flush play snapshot temporary file",
            temporary,
            error,
        )
    })?;
    fs::rename(temporary, target).map_err(|error| {
        format!(
            "failed to publish play snapshot {} -> {}: {error}",
            display_play_snapshot_path(temporary).display(),
            display_play_snapshot_path(target).display()
        )
    })
}

fn display_play_snapshot_path(path: &Path) -> PathBuf {
    ProjectPaths::display_path(path)
}

fn play_snapshot_path_error(action: &str, path: &Path, error: impl std::fmt::Display) -> String {
    format!(
        "{action} {}: {error}",
        display_play_snapshot_path(path).display()
    )
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use zircon_runtime_interface::project::RelPath;

    use super::{complete_snapshot_write, play_snapshot_path_error, MaterializedPlayScene};

    fn test_output_root(name: &str) -> PathBuf {
        let base = std::env::var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target-test-output")
            });
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos());
        base.join(format!(
            "zircon-editor-play-{name}-{}-{nanos}",
            std::process::id()
        ))
    }

    #[test]
    fn failed_snapshot_write_retains_cleanup_owner_until_retry_succeeds() {
        let owned_root = test_output_root("materialize-cleanup-retry");
        if let Some(parent) = owned_root.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&owned_root, "forces remove_dir_all to fail").unwrap();
        let scene = MaterializedPlayScene {
            instance_id: "fault-injected".to_string(),
            path: owned_root.join("play-scene.zrscene.json"),
            relative_path: RelPath::parse(".zircon/play/fault-injected/play-scene.zrscene.json")
                .unwrap(),
            owned_root: Some(owned_root.clone()),
        };

        let failure = complete_snapshot_write(scene, Err("fault-injected write failure".into()))
            .expect_err("failed write and failed cleanup must retain the snapshot owner");
        let (pending_scene, message) = failure.into_parts();
        assert!(message.contains("fault-injected write failure"));
        assert!(message.contains("snapshot cleanup remains pending"));
        let mut pending_scene = pending_scene.expect("cleanup owner must remain retryable");

        fs::remove_file(&owned_root).unwrap();
        fs::create_dir_all(&owned_root).unwrap();
        pending_scene.cleanup().unwrap();
        assert!(!owned_root.exists());
    }

    #[cfg(windows)]
    #[test]
    fn snapshot_error_messages_hide_windows_verbatim_operation_paths() {
        assert_eq!(
            play_snapshot_path_error(
                "failed to create play snapshot",
                Path::new(r"\\?\C:\projects\forest\.zircon\play\instance"),
                "access denied",
            ),
            "failed to create play snapshot C:\\projects\\forest\\.zircon\\play\\instance: access denied"
        );
    }
}
