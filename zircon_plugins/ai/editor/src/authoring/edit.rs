use zircon_runtime::core::framework::ai::{
    AiBehaviorAbortPolicy, AiBehaviorNodeDescriptor, AiBehaviorNodeParameter,
    AiBehaviorTreeDescriptor,
};

use super::topology;
use super::AiBehaviorTreeEditError;

#[derive(Clone, Debug, PartialEq)]
pub enum AiBehaviorTreeEdit {
    SetTreeName {
        display_name: String,
    },
    SetNodeName {
        node_id: String,
        display_name: String,
    },
    SetParameter {
        node_id: String,
        parameter: AiBehaviorNodeParameter,
    },
    RemoveParameter {
        node_id: String,
        key: String,
    },
    SetAbortPolicy {
        node_id: String,
        policy: AiBehaviorAbortPolicy,
    },
    InsertChild {
        parent_id: String,
        index: usize,
        node: AiBehaviorNodeDescriptor,
    },
    /// Index in the destination children AFTER removal from the old parent.
    Reparent {
        node_id: String,
        parent_id: String,
        index: usize,
    },
    WrapDecorator {
        child_id: String,
        decorator: AiBehaviorNodeDescriptor,
    },
    UnwrapDecorator {
        node_id: String,
    },
    DeleteSubtree {
        node_id: String,
    },
}

impl AiBehaviorTreeEdit {
    pub(super) fn apply(
        &self,
        descriptor: &mut AiBehaviorTreeDescriptor,
    ) -> Result<(), AiBehaviorTreeEditError> {
        match self {
            Self::SetTreeName { display_name } => descriptor.display_name.clone_from(display_name),
            Self::SetNodeName {
                node_id,
                display_name,
            } => {
                topology::node_mut(descriptor, node_id)?
                    .display_name
                    .clone_from(display_name);
            }
            Self::SetParameter { node_id, parameter } => {
                let node = topology::node_mut(descriptor, node_id)?;
                if let Some(old) = node
                    .parameters
                    .iter_mut()
                    .find(|old| old.key == parameter.key)
                {
                    old.value.clone_from(&parameter.value);
                } else {
                    node.parameters.push(parameter.clone());
                }
            }
            Self::SetAbortPolicy { node_id, policy } => {
                topology::node_mut(descriptor, node_id)?.abort_policy = *policy;
            }
            Self::RemoveParameter { node_id, key } => {
                topology::node_mut(descriptor, node_id)?
                    .parameters
                    .retain(|old| old.key != *key);
            }
            Self::InsertChild {
                parent_id,
                index,
                node,
            } => {
                topology::insert(descriptor, parent_id, *index, node)?;
            }
            Self::Reparent {
                node_id,
                parent_id,
                index,
            } => {
                topology::reparent(descriptor, node_id, parent_id, *index)?;
            }
            Self::WrapDecorator {
                child_id,
                decorator,
            } => {
                topology::wrap(descriptor, child_id, decorator)?;
            }
            Self::UnwrapDecorator { node_id } => topology::unwrap(descriptor, node_id)?,
            Self::DeleteSubtree { node_id } => topology::delete(descriptor, node_id)?,
        }
        Ok(())
    }
}
