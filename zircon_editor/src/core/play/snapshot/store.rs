use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use super::PlaySceneSource;

#[derive(Debug, Default)]
pub struct PlaySnapshotStore {
    next_sequence: AtomicU64,
}

#[derive(Debug)]
pub struct MaterializedPlayScene {
    instance_id: String,
    path: PathBuf,
    owned_root: Option<PathBuf>,
}

impl PlaySnapshotStore {
    pub fn materialize(
        &self,
        project_root: &Path,
        source: &PlaySceneSource,
    ) -> Result<MaterializedPlayScene, String> {
        let instance_id = self.next_instance_id();
        match source {
            PlaySceneSource::Persisted(path) => {
                let path = if path.is_absolute() {
                    path.clone()
                } else {
                    project_root.join(path)
                };
                if !path.is_file() {
                    return Err(format!(
                        "persisted play scene does not exist: {}",
                        path.display()
                    ));
                }
                Ok(MaterializedPlayScene {
                    instance_id,
                    path,
                    owned_root: None,
                })
            }
            PlaySceneSource::Snapshot(document) => {
                let root = project_root.join(".zircon").join("play").join(&instance_id);
                fs::create_dir_all(&root).map_err(|error| {
                    format!("failed to create play snapshot {}: {error}", root.display())
                })?;
                let final_path = root.join("play-scene.zrscene.json");
                let temporary_path = root.join("play-scene.zrscene.json.tmp");
                let write_result = write_atomic_snapshot(&temporary_path, &final_path, document);
                if let Err(error) = write_result {
                    let _ = fs::remove_dir_all(&root);
                    return Err(error);
                }
                Ok(MaterializedPlayScene {
                    instance_id,
                    path: final_path,
                    owned_root: Some(root),
                })
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

impl MaterializedPlayScene {
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn cleanup(&mut self) -> Result<(), String> {
        let Some(root) = self.owned_root.as_ref() else {
            return Ok(());
        };
        fs::remove_dir_all(root).map_err(|error| {
            format!("failed to clean play snapshot {}: {error}", root.display())
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

fn write_atomic_snapshot(temporary: &Path, target: &Path, document: &str) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(temporary)
        .map_err(|error| {
            format!(
                "failed to create play snapshot temporary file {}: {error}",
                temporary.display()
            )
        })?;
    file.write_all(document.as_bytes()).map_err(|error| {
        format!(
            "failed to write play snapshot temporary file {}: {error}",
            temporary.display()
        )
    })?;
    file.sync_all().map_err(|error| {
        format!(
            "failed to flush play snapshot temporary file {}: {error}",
            temporary.display()
        )
    })?;
    fs::rename(temporary, target).map_err(|error| {
        format!(
            "failed to publish play snapshot {} -> {}: {error}",
            temporary.display(),
            target.display()
        )
    })
}
