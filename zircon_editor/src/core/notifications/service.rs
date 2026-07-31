use std::sync::OnceLock;

use super::{DecisionCenterConfig, DecisionNotificationCenter, DecisionNotificationError};

/// Context-owned notification authority. Leaf consumers resolve immutable receipts;
/// callbacks and producer-specific mutations remain outside this service.
#[derive(Default)]
pub struct EditorNotificationService {
    decisions: OnceLock<DecisionNotificationCenter>,
}

impl EditorNotificationService {
    pub fn decisions(&self) -> Result<&DecisionNotificationCenter, DecisionNotificationError> {
        self.decisions
            .get_or_try_init(|| DecisionNotificationCenter::new(DecisionCenterConfig::default()))
    }
}
