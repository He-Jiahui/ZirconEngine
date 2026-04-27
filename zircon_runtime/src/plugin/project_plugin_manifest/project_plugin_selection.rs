use serde::{Deserialize, Serialize};

use crate::{ExportPackagingStrategy, RuntimeTargetMode};

use super::default_packaging::default_packaging;
use super::default_true::default_true;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectPluginSelection {
    pub id: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub target_modes: Vec<RuntimeTargetMode>,
    #[serde(default = "default_packaging")]
    pub packaging: ExportPackagingStrategy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_crate: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub editor_crate: Option<String>,
}
