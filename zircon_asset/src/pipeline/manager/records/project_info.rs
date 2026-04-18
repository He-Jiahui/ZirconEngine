use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub root_path: String,
    pub name: String,
    pub default_scene_uri: String,
    pub library_version: u32,
}
