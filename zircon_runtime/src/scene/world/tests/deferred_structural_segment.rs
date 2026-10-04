use super::*;
use crate::scene::ecs::{DeferredStructuralKind, DeferredStructuralMetadata};

#[test]
fn sequence_exhaustion_is_reported_without_publishing_the_barrier() {
    let mut world = World::empty();
    let mut batch = DeferredStructuralBatch::new();
    batch.next_sequence = usize::MAX;

    batch.stage_despawn(
        &mut world,
        DeferredStructuralMetadata::new(
            DeferredEntityRef::existing(7),
            DeferredStructuralKind::Despawn,
            DeferredCommandOperation::Despawn,
        ),
    );

    let errors = batch.finish(&mut world);
    assert!(!world.contains_entity(7));
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].operation(), DeferredCommandOperation::Despawn);
    assert_eq!(errors[0].target(), &DeferredCommandTarget::resolved(7));
    assert_eq!(
        errors[0].error(),
        &SceneError::DeferredCommandSequenceExhausted
    );
}
