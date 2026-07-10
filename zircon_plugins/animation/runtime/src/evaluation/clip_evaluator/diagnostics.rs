use zircon_runtime::scene::EntityId;

use crate::AnimationEvaluationDiagnostic;

use super::{AnimationAssetRevision, AnimationClipEvaluator, AnimationEvaluationError};

impl AnimationClipEvaluator {
    pub fn record_diagnostic(
        &mut self,
        entity: EntityId,
        skeleton: AnimationAssetRevision,
        clip: AnimationAssetRevision,
        error: AnimationEvaluationError,
    ) {
        let key = (
            skeleton.id(),
            skeleton.revision(),
            clip.id(),
            clip.revision(),
            error.to_string(),
        );
        if self.reported_diagnostics.insert(key.clone()) {
            self.diagnostic_order.push_back(key);
            self.pending_diagnostics
                .push(AnimationEvaluationDiagnostic {
                    entity,
                    skeleton,
                    clip,
                    error,
                });
            self.enforce_diagnostic_limit();
        }
    }

    pub fn drain_diagnostics(&mut self) -> Vec<AnimationEvaluationDiagnostic> {
        std::mem::take(&mut self.pending_diagnostics)
    }
}
