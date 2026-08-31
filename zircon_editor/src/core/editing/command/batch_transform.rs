use serde::{Deserialize, Serialize};
use zircon_runtime::scene::{NodeId, Scene};

use super::{
    applied, execute_scene_write, external_error, journal_payload, unchanged,
    CommandExecutionError, CommandJournalPayload, CommandJournalUnavailable, EditCommandError,
    NodeEditState, SceneWriteCompletion,
};
use crate::core::editing::context::CoreEditContext;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct BatchTransformTarget {
    pub(crate) node_id: NodeId,
    pub(crate) before: NodeEditState,
    pub(crate) after: NodeEditState,
}

impl BatchTransformTarget {
    pub(crate) fn new(
        node_id: NodeId,
        before: NodeEditState,
        after: NodeEditState,
    ) -> Option<Self> {
        (before.transform != after.transform).then_some(Self {
            node_id,
            before,
            after,
        })
    }
}

#[derive(Clone, Debug)]
pub(crate) struct BatchTransformCommand {
    targets: Vec<BatchTransformTarget>,
    already_applied: bool,
    expected_applied_world_generation: Option<u64>,
}

impl BatchTransformCommand {
    pub(crate) fn applied(
        targets: Vec<BatchTransformTarget>,
        expected_world_generation: u64,
    ) -> Option<Self> {
        (!targets.is_empty()).then_some(Self {
            targets,
            already_applied: true,
            expected_applied_world_generation: Some(expected_world_generation),
        })
    }

    pub(crate) fn from_journal(
        targets: Vec<BatchTransformTarget>,
    ) -> Result<Self, EditCommandError> {
        validate_targets(&targets)?;
        Ok(Self {
            targets,
            already_applied: false,
            expected_applied_world_generation: None,
        })
    }

    pub(crate) const fn target_count(&self) -> usize {
        self.targets.len()
    }

    pub(super) fn apply(&mut self, context: &CoreEditContext) -> Result<(), CommandExecutionError> {
        if self.already_applied {
            let expected_world_generation =
                self.expected_applied_world_generation.ok_or_else(|| {
                    unchanged(EditCommandError::InvariantViolation {
                        invariant: "applied batch transform requires its preview world generation",
                    })
                })?;
            context
                .with_scene(|scene| {
                    validate_applied_targets(scene, &self.targets, expected_world_generation)
                })
                .map_err(unchanged)?
                .map_err(unchanged)?;
            self.already_applied = false;
            self.expected_applied_world_generation = None;
            return Ok(());
        }
        apply_targets(context, &self.targets, false)
    }

    pub(super) fn revert(
        &mut self,
        context: &CoreEditContext,
    ) -> Result<(), CommandExecutionError> {
        apply_targets(context, &self.targets, true)
    }

    pub(super) fn journal_payload(
        &self,
    ) -> Result<CommandJournalPayload, CommandJournalUnavailable> {
        journal_payload(
            "zircon.editor.scene.batch_transform",
            &BatchTransformJournalPayload {
                targets: self.targets.clone(),
            },
        )
    }
}

fn validate_applied_targets(
    scene: &Scene,
    targets: &[BatchTransformTarget],
    expected_world_generation: u64,
) -> Result<(), EditCommandError> {
    let actual_world_generation = scene.world_generation();
    if actual_world_generation != expected_world_generation {
        return Err(external_error(format!(
            "batch transform preview world generation changed from {expected_world_generation} to {actual_world_generation} before commit"
        )));
    }
    for target in targets {
        let current = NodeEditState::capture(scene, target.node_id)?;
        if current != target.after {
            return Err(external_error(format!(
                "batch transform preview for entity {} changed before commit",
                target.node_id
            )));
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct BatchTransformJournalPayload {
    pub(crate) targets: Vec<BatchTransformTarget>,
}

fn validate_targets(targets: &[BatchTransformTarget]) -> Result<(), EditCommandError> {
    if targets.is_empty() {
        return Err(EditCommandError::InvariantViolation {
            invariant: "batch transform journal requires at least one target",
        });
    }
    let mut identities = targets
        .iter()
        .map(|target| target.node_id)
        .collect::<Vec<_>>();
    identities.sort_unstable();
    identities.dedup();
    if identities.len() != targets.len() {
        return Err(EditCommandError::InvariantViolation {
            invariant: "batch transform journal target identities must be unique",
        });
    }
    if targets.iter().any(|target| {
        target.before.name != target.after.name
            || target.before.parent != target.after.parent
            || target.before.transform == target.after.transform
    }) {
        return Err(EditCommandError::InvariantViolation {
            invariant: "batch transform journal may only change transforms",
        });
    }
    Ok(())
}

fn apply_targets(
    context: &CoreEditContext,
    targets: &[BatchTransformTarget],
    reverse: bool,
) -> Result<(), CommandExecutionError> {
    match execute_scene_write(context, |scene| write_targets(scene, targets, reverse))? {
        SceneWriteCompletion::Completed(()) => Ok(()),
        SceneWriteCompletion::AppliedThenGatewayFailed { error, .. } => Err(applied(error)),
    }
}

fn write_targets(
    scene: &mut Scene,
    targets: &[BatchTransformTarget],
    reverse: bool,
) -> Result<(), EditCommandError> {
    let mut applied_count = 0;
    for target in targets {
        let transform = if reverse {
            target.before.transform
        } else {
            target.after.transform
        };
        if let Err(error) = scene.update_transform(target.node_id, transform) {
            let rollback = rollback_prefix(scene, &targets[..applied_count], reverse);
            return Err(external_error(match rollback {
                Ok(()) => format!("batch transform target {} failed: {error}", target.node_id),
                Err(rollback) => format!(
                    "batch transform target {} failed: {error}; rollback failed: {rollback}",
                    target.node_id
                ),
            }));
        }
        applied_count += 1;
    }
    Ok(())
}

fn rollback_prefix(
    scene: &mut Scene,
    applied: &[BatchTransformTarget],
    reverse: bool,
) -> Result<(), String> {
    for target in applied.iter().rev() {
        let transform = if reverse {
            target.after.transform
        } else {
            target.before.transform
        };
        scene
            .update_transform(target.node_id, transform)
            .map_err(|error| format!("entity {}: {error}", target.node_id))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use zircon_runtime::scene::components::NodeKind;
    use zircon_runtime_interface::math::{Transform, Vec3};

    use super::*;

    #[test]
    fn applied_batch_validation_rejects_world_changes_before_commit() {
        let mut scene = Scene::empty();
        let node_id = scene.spawn_node(NodeKind::Cube).unwrap();
        let before = NodeEditState::capture(&scene, node_id).unwrap();
        let mut after = before.clone();
        after.transform = Transform::from_translation(Vec3::new(2.0, 0.0, 0.0));
        scene.update_transform(node_id, after.transform).unwrap();
        let expected_world_generation = scene.world_generation();
        let targets = vec![BatchTransformTarget::new(node_id, before, after).unwrap()];

        validate_applied_targets(&scene, &targets, expected_world_generation).unwrap();
        scene.rename_node(node_id, "Externally changed").unwrap();
        assert!(
            validate_applied_targets(&scene, &targets, expected_world_generation).is_err(),
            "an externally advanced world must not commit stale already-applied snapshots"
        );
    }
}
