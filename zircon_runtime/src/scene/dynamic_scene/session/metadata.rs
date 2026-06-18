use serde::{Deserialize, Serialize};

use crate::scene::LevelMetadata;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeSessionMetadata {
    #[serde(default)]
    pub project_root: Option<String>,
    #[serde(default)]
    pub asset_uri: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub created_at_unix_millis: Option<u64>,
    #[serde(default)]
    pub updated_at_unix_millis: Option<u64>,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl RuntimeSessionMetadata {
    pub fn from_level_metadata(metadata: LevelMetadata) -> Self {
        Self {
            project_root: metadata.project_root,
            asset_uri: metadata.asset_uri,
            display_name: metadata.display_name,
            ..Self::default()
        }
    }

    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }

    pub fn with_updated_at_unix_millis(mut self, updated_at_unix_millis: u64) -> Self {
        self.updated_at_unix_millis = Some(updated_at_unix_millis);
        self
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self.normalize();
        self
    }

    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn normalize(&mut self) {
        normalize_metadata_tags(&mut self.tags);
    }

    pub fn to_level_metadata(&self) -> LevelMetadata {
        LevelMetadata {
            project_root: self.project_root.clone(),
            asset_uri: self.asset_uri.clone(),
            display_name: self.display_name.clone(),
        }
    }
}

fn normalize_metadata_tags(tags: &mut Vec<String>) {
    for tag in tags.iter_mut() {
        *tag = tag.trim().to_string();
    }
    tags.retain(|tag| !tag.is_empty());
    tags.sort();
    tags.dedup();
}
