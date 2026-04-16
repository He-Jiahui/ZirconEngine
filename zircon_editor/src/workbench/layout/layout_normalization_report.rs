use serde::{Deserialize, Serialize};

use crate::ViewInstanceId;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LayoutNormalizationReport {
    pub placeholders: Vec<ViewInstanceId>,
    pub removed_missing_active_tabs: usize,
}
