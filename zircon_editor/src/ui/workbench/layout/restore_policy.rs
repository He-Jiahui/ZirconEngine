use serde::{Deserialize, Serialize};

use super::WorkbenchLayout;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RestorePolicy {
    ProjectThenGlobal,
    PresetThenProjectThenGlobal { preset: Option<WorkbenchLayout> },
}
