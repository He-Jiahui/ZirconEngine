use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

pub type ProjectMetadataMap = BTreeMap<String, ProjectMetadata>;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectMetadata {
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub engine_id: Option<String>,
    #[serde(default)]
    pub last_selected_template: Option<String>,
}

impl ProjectMetadata {
    pub fn is_empty(&self) -> bool {
        !self.pinned && self.engine_id.is_none() && self.last_selected_template.is_none()
    }
}

pub fn project_metadata_key(path: impl AsRef<Path>) -> String {
    let mut text = path.as_ref().to_string_lossy().replace('\\', "/");
    while text.ends_with('/') && text.len() > 1 {
        text.pop();
    }
    if cfg!(target_os = "windows") || looks_like_windows_drive_path(&text) {
        text.to_ascii_lowercase()
    } else {
        text
    }
}

pub fn project_filesystem_path_key(path: impl AsRef<Path>) -> String {
    let resolved = path
        .as_ref()
        .canonicalize()
        .unwrap_or_else(|_| path.as_ref().to_path_buf());
    project_metadata_key(resolved)
}

fn looks_like_windows_drive_path(path: &str) -> bool {
    let bytes = path.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

pub fn project_paths_match(left: impl AsRef<Path>, right: impl AsRef<Path>) -> bool {
    project_metadata_key(left) == project_metadata_key(right)
}

pub fn metadata_for_path<'a>(
    metadata: &'a ProjectMetadataMap,
    path: impl AsRef<Path>,
) -> Option<&'a ProjectMetadata> {
    metadata.get(&project_metadata_key(path))
}

pub fn metadata_for_path_mut<'a>(
    metadata: &'a mut ProjectMetadataMap,
    path: impl AsRef<Path>,
) -> &'a mut ProjectMetadata {
    let key = project_metadata_key(path);
    metadata.entry(key).or_default()
}

pub fn prune_empty_metadata(metadata: &mut ProjectMetadataMap) {
    metadata.retain(|_, value| !value.is_empty());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_key_normalizes_separators_and_trailing_slashes() {
        let key = project_metadata_key("E:\\Projects\\Game\\");

        assert_eq!(key, "e:/projects/game");
    }

    #[test]
    fn project_paths_match_uses_metadata_key_normalization() {
        assert!(project_paths_match(
            "E:\\Projects\\Game\\",
            "E:/Projects/Game"
        ));
        assert!(project_paths_match("E:/Projects/Game", "e:/projects/game/"));
    }

    #[test]
    fn filesystem_path_key_canonicalizes_when_possible() {
        let root = std::env::temp_dir().join(format!(
            "zircon-hub-path-key-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let project = root.join("Project");
        std::fs::create_dir_all(&project).unwrap();

        assert_eq!(
            project_filesystem_path_key(project.join(".")),
            project_filesystem_path_key(&project)
        );

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn empty_metadata_can_be_pruned() {
        let mut metadata = ProjectMetadataMap::new();
        metadata.insert("empty".to_string(), ProjectMetadata::default());
        metadata.insert(
            "pinned".to_string(),
            ProjectMetadata {
                pinned: true,
                ..ProjectMetadata::default()
            },
        );

        prune_empty_metadata(&mut metadata);

        assert!(!metadata.contains_key("empty"));
        assert!(metadata.contains_key("pinned"));
    }
}
