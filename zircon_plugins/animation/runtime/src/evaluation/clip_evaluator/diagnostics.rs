use std::collections::BTreeSet;

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

    pub fn drain_diagnostics_excluding(
        &mut self,
        deferred_entities: &BTreeSet<EntityId>,
    ) -> Vec<AnimationEvaluationDiagnostic> {
        if deferred_entities.is_empty() {
            return self.drain_diagnostics();
        }
        let (retained, admitted) = std::mem::take(&mut self.pending_diagnostics)
            .into_iter()
            .partition(|diagnostic| deferred_entities.contains(&diagnostic.entity));
        self.pending_diagnostics = retained;
        admitted
    }

    pub(crate) fn reset_diagnostics(&mut self) {
        self.pending_diagnostics.clear();
        self.reported_diagnostics.clear();
        self.diagnostic_order.clear();
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use zircon_runtime::core::resource::ResourceId;

    use super::{AnimationAssetRevision, AnimationClipEvaluator, AnimationEvaluationError};

    #[test]
    fn deferred_entity_diagnostic_remains_pending_until_admission() {
        let skeleton = AnimationAssetRevision::new(
            ResourceId::from_stable_label("animation.skeleton.diagnostic-retention"),
            1,
        );
        let clip_a = AnimationAssetRevision::new(
            ResourceId::from_stable_label("animation.clip.diagnostic-retention.a"),
            1,
        );
        let clip_b = AnimationAssetRevision::new(
            ResourceId::from_stable_label("animation.clip.diagnostic-retention.b"),
            1,
        );
        let mut evaluator = AnimationClipEvaluator::default();
        evaluator.record_diagnostic(
            17,
            skeleton,
            clip_a,
            AnimationEvaluationError::MissingPreparedClip {
                skeleton: skeleton.id(),
                clip: clip_a.id(),
            },
        );
        evaluator.record_diagnostic(
            18,
            skeleton,
            clip_b,
            AnimationEvaluationError::MissingPreparedClip {
                skeleton: skeleton.id(),
                clip: clip_b.id(),
            },
        );

        let admitted = evaluator.drain_diagnostics_excluding(&BTreeSet::from([18]));
        assert_eq!(admitted.len(), 1);
        assert_eq!(admitted[0].entity, 17);

        let retried = evaluator.drain_diagnostics_excluding(&BTreeSet::new());
        assert_eq!(retried.len(), 1);
        assert_eq!(retried[0].entity, 18);
    }
}
