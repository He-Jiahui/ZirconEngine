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
