use crate::scene::ecs::{Command, DeferredCommandOperation, DeferredEntityRef};

pub(crate) trait QueuedStructuralCommand: Command {
    fn structural_metadata(&self) -> DeferredStructuralMetadata;

    fn stage_into_batch(
        self,
        batch: &mut crate::scene::world::DeferredStructuralBatch,
        world: &mut crate::scene::World,
    );
}

pub(crate) trait ErasedQueuedStructuralCommand: Send {
    fn structural_metadata(&self) -> DeferredStructuralMetadata;

    fn stage_boxed(
        self: Box<Self>,
        batch: &mut crate::scene::world::DeferredStructuralBatch,
        world: &mut crate::scene::World,
    );
}

impl<C> ErasedQueuedStructuralCommand for C
where
    C: QueuedStructuralCommand,
{
    fn structural_metadata(&self) -> DeferredStructuralMetadata {
        QueuedStructuralCommand::structural_metadata(self)
    }

    fn stage_boxed(
        self: Box<Self>,
        batch: &mut crate::scene::world::DeferredStructuralBatch,
        world: &mut crate::scene::World,
    ) {
        (*self).stage_into_batch(batch, world);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DeferredStructuralKind {
    SpawnEmpty,
    SpawnBundle,
    InsertBundle,
    RemoveComponent,
    Despawn,
}

#[derive(Clone, Debug)]
pub(crate) struct DeferredStructuralMetadata {
    target: DeferredEntityRef,
    kind: DeferredStructuralKind,
    operation: DeferredCommandOperation,
}

impl DeferredStructuralMetadata {
    pub(crate) fn new(
        target: DeferredEntityRef,
        kind: DeferredStructuralKind,
        operation: DeferredCommandOperation,
    ) -> Self {
        Self {
            target,
            kind,
            operation,
        }
    }

    pub(crate) fn target(&self) -> &DeferredEntityRef {
        &self.target
    }

    pub(crate) const fn kind(&self) -> DeferredStructuralKind {
        self.kind
    }

    pub(crate) const fn operation(&self) -> DeferredCommandOperation {
        self.operation
    }
}
