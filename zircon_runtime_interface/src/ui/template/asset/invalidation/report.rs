use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{UiAssetChange, UiInvalidationDiagnostic, UiInvalidationImpact, UiInvalidationStage};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiInvalidationReport {
    pub changes: Vec<UiAssetChange>,
    pub stages: BTreeSet<UiInvalidationStage>,
    pub impact: UiInvalidationImpact,
    pub diagnostics: Vec<UiInvalidationDiagnostic>,
}

impl UiInvalidationReport {
    pub fn cache_hit() -> Self {
        Self::default()
    }

    pub fn from_stages(
        changes: Vec<UiAssetChange>,
        stages: BTreeSet<UiInvalidationStage>,
        diagnostics: Vec<UiInvalidationDiagnostic>,
    ) -> Self {
        let impact = UiInvalidationImpact::from_stages(&stages);
        Self {
            changes,
            stages,
            impact,
            diagnostics,
        }
    }
}
