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
    Target: Send + 'static,
{
    fn label(&self) -> &str {
        &self.label
    }

    fn apply(&mut self, context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        if self.inverse_delta.is_some() {
            return Err(CommandExecutionError::unchanged(
                EditCommandError::InvariantViolation {
                    invariant: "graph delta command must be reverted before it can be applied again",
                },
            ));
        }
        let context = graph_context_mut::<Model, Context, Target>(context)?;
        let inverse = require_applied(
            context
                .graph_model_mut(&self.target)
                .map_err(CommandExecutionError::unchanged)?
                .apply(self.forward_delta.clone())
                .map_err(transaction_failure)?,
            "graph delta command must not enter history without an applied mutation",
        )?;
        self.inverse_delta = Some(inverse);
        Ok(())
    }

    fn revert(&mut self, context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        let context = graph_context_mut::<Model, Context, Target>(context)?;
        let model = context
            .graph_model_mut(&self.target)
            .map_err(CommandExecutionError::unchanged)?;
        let inverse = self.inverse_delta.take().ok_or_else(|| {
            CommandExecutionError::unchanged(EditCommandError::InvariantViolation {
                invariant: "graph delta command must be applied before it can be reverted",
            })
        })?;
        let forward = match model.apply(inverse.clone()) {
            Ok(effect) => require_applied(
                effect,
                "graph inverse delta must apply during transaction rollback",
            ),
            Err(error) => Err(transaction_failure(error)),
        };
        match forward {
            Ok(forward) => {
                self.forward_delta = forward;
                Ok(())
            }
            Err(error) => {
                self.inverse_delta = Some(inverse);
                Err(error)
            }
        }
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
