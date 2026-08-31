use serde::{Deserialize, Serialize};

use super::update::UiBindingDirtyDomain;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiBindingMutationOutcome {
    Committed,
    RolledBack,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiBindingMutationReceipt {
    pub base_generation: u64,
    #[serde(default)]
    pub revision: u64,
    pub target_count: usize,
    pub applied_target_count: usize,
    #[serde(default)]
    pub unchanged_target_count: usize,
    #[serde(default)]
    pub impact: Vec<UiBindingDirtyDomain>,
    pub outcome: UiBindingMutationOutcome,
}

impl UiBindingMutationReceipt {
    pub fn committed(
        base_generation: u64,
        revision: u64,
        target_count: usize,
        applied_target_count: usize,
        unchanged_target_count: usize,
        impact: Vec<UiBindingDirtyDomain>,
    ) -> Self {
        Self {
            base_generation,
            revision,
            target_count,
            applied_target_count,
            unchanged_target_count,
            impact,
            outcome: UiBindingMutationOutcome::Committed,
        }
    }

    pub fn rolled_back(base_generation: u64, target_count: usize) -> Self {
        Self {
            base_generation,
            revision: base_generation,
            target_count,
            applied_target_count: 0,
            unchanged_target_count: 0,
            impact: Vec::new(),
            outcome: UiBindingMutationOutcome::RolledBack,
        }
    }
}
