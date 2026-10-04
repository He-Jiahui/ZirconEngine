use thiserror::Error;
use zircon_runtime_interface::{ZrRuntimeEditorTransformPhaseV1, ZrRuntimeEditorTransformWriteV1};

use crate::core::math::Transform;
use crate::scene::{SceneError, World};

#[derive(Clone, Copy, Debug)]
struct ActiveEditorTransformInteraction {
    entity: u64,
    interaction_id: u64,
    sequence: u64,
    world_replacement_epoch: u64,
    initial: Transform,
    current: Transform,
}

#[derive(Default)]
pub(super) struct RuntimeEditorTransformState {
    active: Option<ActiveEditorTransformInteraction>,
}

impl RuntimeEditorTransformState {
    pub(super) fn handle(
        &mut self,
        world: &mut World,
        current_world_replacement_epoch: u64,
        request: ZrRuntimeEditorTransformWriteV1,
    ) -> Result<(), RuntimeEditorTransformWriteError> {
        if !request.validate_editor_transform_write() {
            return Err(RuntimeEditorTransformWriteError::InvalidRequest);
        }
        match request
            .phase()
            .expect("validated editor transform request has a known phase")
        {
            ZrRuntimeEditorTransformPhaseV1::Begin => {
                self.begin(world, current_world_replacement_epoch, request)
            }
            ZrRuntimeEditorTransformPhaseV1::Preview => {
                self.write_active(world, current_world_replacement_epoch, request, false)
            }
            ZrRuntimeEditorTransformPhaseV1::Commit => {
                self.write_active(world, current_world_replacement_epoch, request, true)
            }
            ZrRuntimeEditorTransformPhaseV1::Cancel => {
                self.cancel(world, current_world_replacement_epoch, request)
            }
            ZrRuntimeEditorTransformPhaseV1::Apply => {
                self.apply(world, current_world_replacement_epoch, request)
            }
        }
    }

    fn begin(
        &mut self,
        world: &World,
        current_world_replacement_epoch: u64,
        request: ZrRuntimeEditorTransformWriteV1,
    ) -> Result<(), RuntimeEditorTransformWriteError> {
        if self
            .active
            .is_some_and(|active| active.world_replacement_epoch != current_world_replacement_epoch)
        {
            self.active = None;
        }
        if self.active.is_some() {
            return Err(RuntimeEditorTransformWriteError::InteractionBusy);
        }
        ensure_world_epoch(
            current_world_replacement_epoch,
            request.world_replacement_epoch,
        )?;
        let current = local_transform(world, request.entity)?;
        let expected = request.expected_transform();
        if current != expected || request.target_transform() != expected {
            return Err(RuntimeEditorTransformWriteError::ExpectedTransformChanged);
        }
        self.active = Some(ActiveEditorTransformInteraction {
            entity: request.entity,
            interaction_id: request.interaction_id,
            sequence: request.sequence,
            world_replacement_epoch: current_world_replacement_epoch,
            initial: current,
            current,
        });
        Ok(())
    }

    // 预览与提交都要求连续序号和 compare-and-set 前值；外部写入或换世界会退休本次交互。
    fn write_active(
        &mut self,
        world: &mut World,
        current_world_replacement_epoch: u64,
        request: ZrRuntimeEditorTransformWriteV1,
        finish: bool,
    ) -> Result<(), RuntimeEditorTransformWriteError> {
        let active = self.active()?;
        self.validate_active(current_world_replacement_epoch, request, active)?;
        let current = local_transform(world, request.entity)?;
        if current != active.current || request.expected_transform() != active.current {
            self.active = None;
            return Err(RuntimeEditorTransformWriteError::ExpectedTransformChanged);
        }
        let target = request.target_transform();
        if let Err(error) = world.update_transform(request.entity, target) {
            self.active = None;
            return Err(RuntimeEditorTransformWriteError::Scene(error));
        }
        if finish {
            self.active = None;
        } else if let Some(active) = self.active.as_mut() {
            active.current = target;
            active.sequence = request.sequence;
        }
        Ok(())
    }

    fn cancel(
        &mut self,
        world: &mut World,
        current_world_replacement_epoch: u64,
        request: ZrRuntimeEditorTransformWriteV1,
    ) -> Result<(), RuntimeEditorTransformWriteError> {
        let active = self.active()?;
        self.validate_active(current_world_replacement_epoch, request, active)?;
        if request.expected_transform() != active.current
            || request.target_transform() != active.initial
            || local_transform(world, request.entity)? != active.current
        {
            self.active = None;
            return Err(RuntimeEditorTransformWriteError::ExpectedTransformChanged);
        }
        if let Err(error) = world.update_transform(request.entity, active.initial) {
            self.active = None;
            return Err(RuntimeEditorTransformWriteError::Scene(error));
        }
        self.active = None;
        Ok(())
    }

    fn apply(
        &mut self,
        world: &mut World,
        current_world_replacement_epoch: u64,
        request: ZrRuntimeEditorTransformWriteV1,
    ) -> Result<(), RuntimeEditorTransformWriteError> {
        if self.active.is_some() {
            return Err(RuntimeEditorTransformWriteError::InteractionBusy);
        }
        ensure_world_epoch(
            current_world_replacement_epoch,
            request.world_replacement_epoch,
        )?;
        if local_transform(world, request.entity)? != request.expected_transform() {
            return Err(RuntimeEditorTransformWriteError::ExpectedTransformChanged);
        }
        world
            .update_transform(request.entity, request.target_transform())
            .map_err(RuntimeEditorTransformWriteError::Scene)?;
        Ok(())
    }

    fn active(&self) -> Result<ActiveEditorTransformInteraction, RuntimeEditorTransformWriteError> {
        self.active
            .ok_or(RuntimeEditorTransformWriteError::InteractionMissing)
    }

    fn validate_active(
        &mut self,
        current_world_replacement_epoch: u64,
        request: ZrRuntimeEditorTransformWriteV1,
        active: ActiveEditorTransformInteraction,
    ) -> Result<(), RuntimeEditorTransformWriteError> {
        if active.world_replacement_epoch != current_world_replacement_epoch {
            self.active = None;
            return Err(RuntimeEditorTransformWriteError::WorldReplaced {
                expected: active.world_replacement_epoch,
                actual: current_world_replacement_epoch,
            });
        }
        ensure_world_epoch(
            current_world_replacement_epoch,
            request.world_replacement_epoch,
        )?;
        if active.entity != request.entity || active.interaction_id != request.interaction_id {
            return Err(RuntimeEditorTransformWriteError::InteractionMismatch);
        }
        let expected_sequence = active
            .sequence
            .checked_add(1)
            .ok_or(RuntimeEditorTransformWriteError::SequenceExhausted)?;
        if request.sequence != expected_sequence {
            return Err(RuntimeEditorTransformWriteError::SequenceMismatch {
                expected: expected_sequence,
                actual: request.sequence,
            });
        }
        Ok(())
    }
}

fn local_transform(
    world: &World,
    entity: u64,
) -> Result<Transform, RuntimeEditorTransformWriteError> {
    world
        .local_transform(entity)
        .ok_or(RuntimeEditorTransformWriteError::TargetMissing { entity })
}

fn ensure_world_epoch(actual: u64, expected: u64) -> Result<(), RuntimeEditorTransformWriteError> {
    if actual != expected {
        return Err(RuntimeEditorTransformWriteError::WorldReplaced { expected, actual });
    }
    Ok(())
}

#[derive(Debug, Error)]
pub(super) enum RuntimeEditorTransformWriteError {
    #[error("editor transform request is structurally invalid")]
    InvalidRequest,
    #[error("another editor transform interaction is active")]
    InteractionBusy,
    #[error("editor transform interaction is not active")]
    InteractionMissing,
    #[error("editor transform interaction identity does not match its owner")]
    InteractionMismatch,
    #[error("editor transform sequence mismatch: expected {expected}, got {actual}")]
    SequenceMismatch { expected: u64, actual: u64 },
    #[error("editor transform sequence space is exhausted")]
    SequenceExhausted,
    #[error("runtime world was replaced: expected epoch {expected}, got {actual}")]
    WorldReplaced { expected: u64, actual: u64 },
    #[error("editor transform target {entity} is missing")]
    TargetMissing { entity: u64 },
    #[error("editor transform expected value no longer matches the runtime world")]
    ExpectedTransformChanged,
    #[error("write editor transform: {0}")]
    Scene(SceneError),
}

#[cfg(test)]
#[path = "tests/editor_transform.rs"]
mod tests;
