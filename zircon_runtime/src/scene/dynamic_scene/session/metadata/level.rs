use crate::scene::LevelMetadata;

use super::model::RuntimeSessionMetadata;

impl RuntimeSessionMetadata {
    pub fn from_level_metadata(metadata: LevelMetadata) -> Self {
        Self {
            project_root: metadata.project_root,
            asset_uri: metadata.asset_uri,
            display_name: metadata.display_name,
            ..Self::default()
        }
    }

    pub fn to_level_metadata(&self) -> LevelMetadata {
        LevelMetadata {
            project_root: self.project_root.clone(),
            asset_uri: self.asset_uri.clone(),
            display_name: self.display_name.clone(),
        }
    }
}
