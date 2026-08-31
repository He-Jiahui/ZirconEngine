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
mod tests {
    use super::*;
    use crate::core::math::{Quat, Vec3};
    use crate::scene::components::NodeKind;

    fn transform(x: f32) -> Transform {
        Transform {
            translation: Vec3::new(x, 0.0, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    fn request(
        entity: u64,
        interaction_id: u64,
        sequence: u64,
        epoch: u64,
        phase: ZrRuntimeEditorTransformPhaseV1,
        expected: Transform,
        target: Transform,
    ) -> ZrRuntimeEditorTransformWriteV1 {
        ZrRuntimeEditorTransformWriteV1::new(
            entity,
            interaction_id,
            sequence,
            epoch,
            phase,
            expected,
            target,
        )
    }

    #[test]
    fn preview_commit_is_ordered_and_compare_and_set() {
        let mut world = World::empty();
        let entity = world.spawn_node(NodeKind::Empty).unwrap();
        let initial = world.local_transform(entity).unwrap();
        let preview = transform(2.0);
        let committed = transform(3.0);
        let mut state = RuntimeEditorTransformState::default();

        state
            .handle(
                &mut world,
                7,
                request(
                    entity,
                    11,
                    1,
                    7,
                    ZrRuntimeEditorTransformPhaseV1::Begin,
                    initial,
                    initial,
                ),
            )
            .unwrap();
        assert_eq!(world.local_transform(entity), Some(initial));
        state
            .handle(
                &mut world,
                7,
                request(
                    entity,
                    11,
                    2,
                    7,
                    ZrRuntimeEditorTransformPhaseV1::Preview,
                    initial,
                    preview,
                ),
            )
            .unwrap();
        assert_eq!(world.local_transform(entity), Some(preview));
        assert!(matches!(
            state.handle(
                &mut world,
                7,
                request(
                    entity,
                    11,
                    4,
                    7,
                    ZrRuntimeEditorTransformPhaseV1::Commit,
                    preview,
                    committed,
                ),
            ),
            Err(RuntimeEditorTransformWriteError::SequenceMismatch { .. })
        ));
        state
            .handle(
                &mut world,
                7,
                request(
                    entity,
                    11,
                    3,
                    7,
                    ZrRuntimeEditorTransformPhaseV1::Commit,
                    preview,
                    committed,
                ),
            )
            .unwrap();
        assert_eq!(world.local_transform(entity), Some(committed));
    }

    #[test]
    fn cancel_restores_initial_and_world_replacement_retires_owner() {
        let mut world = World::empty();
        let entity = world.spawn_node(NodeKind::Empty).unwrap();
        let initial = world.local_transform(entity).unwrap();
        let preview = transform(2.0);
        let mut state = RuntimeEditorTransformState::default();
        state
            .handle(
                &mut world,
                7,
                request(
                    entity,
                    11,
                    1,
                    7,
                    ZrRuntimeEditorTransformPhaseV1::Begin,
                    initial,
                    initial,
                ),
            )
            .unwrap();
        state
            .handle(
                &mut world,
                7,
                request(
                    entity,
                    11,
                    2,
                    7,
                    ZrRuntimeEditorTransformPhaseV1::Preview,
                    initial,
                    preview,
                ),
            )
            .unwrap();
        state
            .handle(
                &mut world,
                7,
                request(
                    entity,
                    11,
                    3,
                    7,
                    ZrRuntimeEditorTransformPhaseV1::Cancel,
                    preview,
                    initial,
                ),
            )
            .unwrap();
        assert_eq!(world.local_transform(entity), Some(initial));

        state
            .handle(
                &mut world,
                7,
                request(
                    entity,
                    13,
                    1,
                    7,
                    ZrRuntimeEditorTransformPhaseV1::Begin,
                    initial,
                    initial,
                ),
            )
            .unwrap();
        assert!(matches!(
            state.handle(
                &mut world,
                8,
                request(
                    entity,
                    13,
                    2,
                    7,
                    ZrRuntimeEditorTransformPhaseV1::Preview,
                    initial,
                    preview,
                ),
            ),
            Err(RuntimeEditorTransformWriteError::WorldReplaced { .. })
        ));
        assert!(matches!(
            state.handle(
                &mut world,
                8,
                request(
                    entity,
                    13,
                    3,
                    8,
                    ZrRuntimeEditorTransformPhaseV1::Cancel,
                    initial,
                    initial,
                ),
            ),
            Err(RuntimeEditorTransformWriteError::InteractionMissing)
        ));
        state
            .handle(
                &mut world,
                8,
                request(
                    entity,
                    17,
                    1,
                    8,
                    ZrRuntimeEditorTransformPhaseV1::Begin,
                    initial,
                    initial,
                ),
            )
            .unwrap();
    }
}
