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
    if cfg!(target_os = "windows") {
        text.to_ascii_lowercase()
    } else {
        text
    }
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

        if cfg!(target_os = "windows") {
            assert_eq!(key, "e:/projects/game");
        } else {
            assert_eq!(key, "E:/Projects/Game");
        }
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
