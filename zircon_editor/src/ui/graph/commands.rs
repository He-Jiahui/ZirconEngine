use std::any::type_name;
use std::collections::BTreeSet;
use std::error::Error;
use std::marker::PhantomData;

use crate::core::editing::engine::{
    CommandExecutionError, EditCommand, EditCommandError, EditContext,
};

use super::{
    GraphModel, GraphMutationEffect, GraphNodeMove, GraphNodeView, GraphPoint, GraphSelection,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphAlignment {
    Top,
    Middle,
    Bottom,
    Left,
    Center,
    Right,
}

/// Computes layout moves from the outer bounds of the selected group, matching the six alignment
/// commands shared by mature graph editors. The domain still turns these moves into its own delta.
pub fn aligned_node_moves<NodeId>(
    nodes: &[GraphNodeView<NodeId>],
    selection: &GraphSelection<NodeId>,
    alignment: GraphAlignment,
) -> Vec<GraphNodeMove<NodeId>>
where
    NodeId: Clone + Ord,
{
    let mut selected = nodes
        .iter()
        .filter(|node| selection.contains(&node.id))
        .collect::<Vec<_>>();
    if selected.len() < 2 {
        return Vec::new();
    }
    selected.sort_by(|left, right| left.id.cmp(&right.id));

    let left = selected
        .iter()
        .map(|node| node.bounds.origin.x)
        .fold(f32::INFINITY, f32::min);
    let right = selected
        .iter()
        .map(|node| node.bounds.right())
        .fold(f32::NEG_INFINITY, f32::max);
    let top = selected
        .iter()
        .map(|node| node.bounds.origin.y)
        .fold(f32::INFINITY, f32::min);
    let bottom = selected
        .iter()
        .map(|node| node.bounds.bottom())
        .fold(f32::NEG_INFINITY, f32::max);
    let horizontal_center = (left + right) * 0.5;
    let vertical_center = (top + bottom) * 0.5;

    selected
        .into_iter()
        .filter_map(|node| {
            let mut position = node.bounds.origin;
            match alignment {
                GraphAlignment::Top => position.y = top,
                GraphAlignment::Middle => position.y = vertical_center - node.bounds.size.y * 0.5,
                GraphAlignment::Bottom => position.y = bottom - node.bounds.size.y,
                GraphAlignment::Left => position.x = left,
                GraphAlignment::Center => position.x = horizontal_center - node.bounds.size.x * 0.5,
                GraphAlignment::Right => position.x = right - node.bounds.size.x,
            }
            (position != node.bounds.origin).then_some(GraphNodeMove {
                node_id: node.id.clone(),
                position,
            })
        })
        .collect()
}

/// A domain-owned subgraph clipboard protocol. Serialization and remapping remain with the graph
/// asset owner; the foundation only supplies the common selection and paste-anchor vocabulary.
pub trait GraphClipboardModel: GraphModel {
    type Clipboard: Clone + Send + Sync + 'static;

    fn serialize_subgraph(
        &self,
        selected_nodes: &BTreeSet<Self::NodeId>,
    ) -> Result<Self::Clipboard, Self::Error>;

    fn paste_subgraph_delta(
        &self,
        clipboard: &Self::Clipboard,
        anchor: GraphPoint,
    ) -> Result<Self::Delta, Self::Error>;
}

/// Context bridge implemented by the owner of concrete graph documents.
pub trait GraphEditContext<Model, Target>: EditContext
where
    Model: GraphModel,
{
    fn graph_model(&self, target: &Target) -> Result<&Model, EditCommandError>;
    fn graph_model_mut(&mut self, target: &Target) -> Result<&mut Model, EditCommandError>;
}

/// A reusable 03 transaction command that retains only the inverse delta between undo/redo.
pub struct GraphDeltaCommand<Model, Context, Target>
where
    Model: GraphModel,
{
    label: String,
    target: Target,
    forward_delta: Model::Delta,
    inverse_delta: Option<Model::Delta>,
    prepared: Option<super::model::PreparedGraphDeltaV1<Model::Delta>>,
    marker: PhantomData<fn() -> Context>,
}

impl<Model, Context, Target> GraphDeltaCommand<Model, Context, Target>
where
    Model: GraphModel,
{
    pub fn new(label: impl Into<String>, target: Target, forward_delta: Model::Delta) -> Self {
        Self {
            label: label.into(),
            target,
            forward_delta,
            inverse_delta: None,
            prepared: None,
            marker: PhantomData,
        }
    }
}

impl<Model, Context, Target> EditCommand for GraphDeltaCommand<Model, Context, Target>
where
    Model: GraphModel + 'static,
    Model::Delta: Send + 'static,
    Model::Error: Error + Send + Sync + 'static,
    Context: GraphEditContext<Model, Target> + 'static,
    Target: zircon_runtime_interface::retained_memory::ImmutableRetainedOwnerV1 + Send + 'static,
{
    fn label(&self) -> &str {
        &self.label
    }

    fn retention_plan_v1(
        &self,
        context: &dyn EditContext,
        phase: crate::core::editing::engine::RetentionPhaseV1,
        model: zircon_runtime_interface::retained_memory::RetainedAccountingModelV1,
        limit: u64,
    ) -> Result<crate::core::editing::engine::CommandRetentionPlanV1, EditCommandError> {
        use crate::core::editing::engine::*;
        use zircon_runtime_interface::retained_memory::*;
        // Charge the complete label/target/delta owner before consulting a model.
        let mut plan = CommandRetentionPlanV1::for_owner(self, context, phase, model, limit)?;
        let context = context.as_any().downcast_ref::<Context>().ok_or(
            EditCommandError::ContextTypeMismatch {
                expected: type_name::<Context>(),
            },
        )?;
        let graph = context.graph_model(&self.target)?;
        let delta = if matches!(phase, RetentionPhaseV1::Undo | RetentionPhaseV1::Rollback) {
            self.inverse_delta
                .as_ref()
                .ok_or(EditCommandError::InvariantViolation {
                    invariant: "graph undo requires its retained inverse",
                })?
        } else {
            &self.forward_delta
        };
        let mut meter = RetainedBoundBuilderV1::with_limit(model, RetainedBytesV1::new(limit));
        let proof = graph.delta_retention_v1(delta, &mut meter)?;
        if proof.revision != graph.retention_revision_v1() {
            return Err(RetentionContractError::Stale.into());
        }
        plan.add_retained(proof.retained_pair_bytes)?;
        plan.add_peak(proof.materialization_peak_bytes)?;
        plan.owner_revision = Some(proof.revision);
        Ok(plan)
    }
    fn prepare_retention_v1(
        &mut self,
        context: &mut dyn EditContext,
        phase: crate::core::editing::engine::RetentionPhaseV1,
        permit: &mut crate::core::editing::engine::RetentionMaterializationV1,
    ) -> Result<(), CommandExecutionError> {
        use crate::core::editing::engine::RetentionPhaseV1;
        permit
            .consume(self, phase)
            .map_err(CommandExecutionError::unchanged)?;
        let context = graph_context_mut::<Model, Context, Target>(context)?;
        let graph = context
            .graph_model(&self.target)
            .map_err(CommandExecutionError::unchanged)?;
        if permit.plan().owner_revision != Some(graph.retention_revision_v1()) {
            return Err(CommandExecutionError::unchanged(
                zircon_runtime_interface::retained_memory::RetentionContractError::Stale.into(),
            ));
        }
        let delta = if matches!(phase, RetentionPhaseV1::Undo | RetentionPhaseV1::Rollback) {
            self.inverse_delta.as_ref().ok_or_else(|| {
                CommandExecutionError::unchanged(EditCommandError::InvariantViolation {
                    invariant: "graph undo requires inverse",
                })
            })?
        } else {
            &self.forward_delta
        };
        let prepared = graph.prepare_delta_v1(delta).map_err(transaction_failure)?;
        permit
            .check_prepared(&prepared)
            .map_err(CommandExecutionError::unchanged)?;
        if prepared.revision != graph.retention_revision_v1() {
            return Err(CommandExecutionError::unchanged(
                zircon_runtime_interface::retained_memory::RetentionContractError::Stale.into(),
            ));
        }
        self.prepared = Some(prepared);
        Ok(())
    }
    fn apply(&mut self, context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        if self.inverse_delta.is_some() {
            return Err(CommandExecutionError::unchanged(
                EditCommandError::InvariantViolation {
                    invariant:
                        "graph delta command must be reverted before it can be applied again",
                },
            ));
        }
        self.execute_prepared(context, false)
    }
    fn revert(&mut self, context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        if self.inverse_delta.is_none() {
            return Err(CommandExecutionError::unchanged(
                EditCommandError::InvariantViolation {
                    invariant: "graph delta command must be applied before it can be reverted",
                },
            ));
        }
        self.execute_prepared(context, true)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

fn graph_context_mut<'a, Model, Context, Target>(
    context: &'a mut dyn EditContext,
) -> Result<&'a mut Context, CommandExecutionError>
where
    Model: GraphModel,
    Context: GraphEditContext<Model, Target> + 'static,
{
    context
        .as_any_mut()
        .downcast_mut::<Context>()
        .ok_or_else(|| {
            CommandExecutionError::unchanged(EditCommandError::ContextTypeMismatch {
                expected: type_name::<Context>(),
            })
        })
}

fn require_applied<Delta>(
    effect: GraphMutationEffect<Delta>,
    invariant: &'static str,
) -> Result<Delta, CommandExecutionError> {
    match effect {
        GraphMutationEffect::Applied { inverse } => Ok(inverse),
        GraphMutationEffect::Unchanged => Err(CommandExecutionError::unchanged(
            EditCommandError::InvariantViolation { invariant },
        )),
    }
}

fn transaction_failure(error: impl Error + Send + Sync + 'static) -> CommandExecutionError {
    CommandExecutionError::unchanged(EditCommandError::ExternalEffect {
        source: Box::new(error),
    })
}

impl<Model, Context, Target> zircon_runtime_interface::retained_memory::RetainedOwnerV1
    for GraphDeltaCommand<Model, Context, Target>
where
    Model: GraphModel,
    Target: zircon_runtime_interface::retained_memory::ImmutableRetainedOwnerV1,
{
    fn retained_bound_v1(
        &self,
        m: &mut zircon_runtime_interface::retained_memory::RetainedBoundBuilderV1,
    ) -> Result<(), zircon_runtime_interface::retained_memory::RetentionContractError> {
        use zircon_runtime_interface::retained_memory::RetainedOwnerV1;
        self.label.retained_bound_v1(m)?;
        self.target.retained_bound_v1(m)?;
        self.forward_delta.retained_bound_v1(m)?;
        self.inverse_delta.retained_bound_v1(m)?;
        self.prepared.retained_bound_v1(m)
    }
}
impl<Model, Context, Target> GraphDeltaCommand<Model, Context, Target>
where
    Model: GraphModel + 'static,
    Model::Delta: Send + 'static,
    Model::Error: Error + Send + Sync + 'static,
    Context: GraphEditContext<Model, Target> + 'static,
    Target: zircon_runtime_interface::retained_memory::ImmutableRetainedOwnerV1 + Send + 'static,
{
    fn execute_prepared(
        &mut self,
        context: &mut dyn EditContext,
        undo: bool,
    ) -> Result<(), CommandExecutionError> {
        // Direct execution has the same domain preparation semantics but makes no
        // history-budget claim. The engine always supplies its charged preparation.
        if self.prepared.is_none() {
            let graph = graph_context_mut::<Model, Context, Target>(context)?
                .graph_model(&self.target)
                .map_err(CommandExecutionError::unchanged)?;
            let delta = if undo {
                self.inverse_delta.as_ref().unwrap()
            } else {
                &self.forward_delta
            };
            self.prepared = Some(graph.prepare_delta_v1(delta).map_err(transaction_failure)?);
        }
        let graph = graph_context_mut::<Model, Context, Target>(context)?
            .graph_model_mut(&self.target)
            .map_err(CommandExecutionError::unchanged)?;
        let prepared = self.prepared.take().unwrap();
        if prepared.revision != graph.retention_revision_v1() {
            return Err(CommandExecutionError::unchanged(
                zircon_runtime_interface::retained_memory::RetentionContractError::Stale.into(),
            ));
        }
        if let Err(error) = graph.apply_prepared_delta_v1(&prepared) {
            // Failure leaves both command directions intact for a retry.
            return Err(transaction_failure(error));
        }
        if undo {
            self.forward_delta = prepared.inverse;
            self.inverse_delta = None;
        } else {
            self.forward_delta = prepared.forward;
            self.inverse_delta = Some(prepared.inverse);
        }
        Ok(())
    }
}
