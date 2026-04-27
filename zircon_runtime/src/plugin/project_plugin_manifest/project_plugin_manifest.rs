use serde::{Deserialize, Serialize};

use super::ProjectPluginSelection;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectPluginManifest {
    #[serde(default)]
    pub selections: Vec<ProjectPluginSelection>,
}
