use zircon_runtime::core::framework::ai::{
    AiBehaviorAbortPolicy, AiBehaviorNodeParameterValue, AiManagerError,
};

use crate::behavior_tree::CompiledBehaviorTree;

use super::{BlackboardLayout, BlackboardSlot};

const BLACKBOARD_KEY_PARAMETER: &str = "blackboard_key";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BlackboardObserver {
    pub(crate) node_index: u32,
    pub(crate) policy: AiBehaviorAbortPolicy,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct BlackboardObserverSet {
    schema_id: String,
    observers_by_slot: Box<[Box<[BlackboardObserver]>]>,
    slots_by_node: Box<[Option<BlackboardSlot>]>,
}

impl BlackboardObserverSet {
    pub(crate) fn resolve(
        tree: &CompiledBehaviorTree,
        layout: &BlackboardLayout,
    ) -> Result<Self, AiManagerError> {
        let mut observers_by_slot = std::iter::repeat_with(Vec::new)
            .take(layout.key_count())
            .collect::<Vec<_>>();
        let mut slots_by_node = vec![None; tree.nodes().len()];
        for (node_index, node) in tree.nodes().iter().enumerate() {
            let key = node.parameters().iter().find_map(|parameter| {
                (parameter.key == BLACKBOARD_KEY_PARAMETER)
                    .then_some(&parameter.value)
                    .and_then(AiBehaviorNodeParameterValue::as_string)
            });
            if node.abort_policy() != AiBehaviorAbortPolicy::None && key.is_none() {
                return Err(AiManagerError::BehaviorObserverMissingBlackboardKey {
                    tree_id: tree.id().to_string(),
                    node_id: node.id().to_string(),
                });
            }
            let Some(key) = key else {
                continue;
            };
            let slot = layout.resolve(key);
            if node.abort_policy() != AiBehaviorAbortPolicy::None && slot.is_none() {
                return Err(AiManagerError::BehaviorObserverUnknownBlackboardKey {
                    tree_id: tree.id().to_string(),
                    node_id: node.id().to_string(),
                    schema_id: layout.schema_id().to_string(),
                    key: key.to_string(),
                });
            }
            let Some(slot) = slot else {
                continue;
            };
            slots_by_node[node_index] = Some(slot);
            if node.abort_policy() == AiBehaviorAbortPolicy::None {
                continue;
            }
            observers_by_slot[slot.generation_index() as usize].push(BlackboardObserver {
                node_index: node_index as u32,
                policy: node.abort_policy(),
            });
        }
        Ok(Self {
            schema_id: layout.schema_id().to_string(),
            observers_by_slot: observers_by_slot
                .into_iter()
                .map(Vec::into_boxed_slice)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            slots_by_node: slots_by_node.into_boxed_slice(),
        })
    }

    pub(crate) fn schema_id(&self) -> &str {
        &self.schema_id
    }

    pub(crate) fn slot_for_node(&self, node_index: u32) -> Option<BlackboardSlot> {
        self.slots_by_node
            .get(node_index as usize)
            .copied()
            .flatten()
    }

    pub(crate) fn matching(&self, changed_slots: &[BlackboardSlot]) -> Vec<BlackboardObserver> {
        changed_slots
            .iter()
            .flat_map(|slot| {
                self.observers_by_slot
                    .get(slot.generation_index() as usize)
                    .into_iter()
                    .flat_map(|observers| observers.iter().copied())
            })
            .collect()
    }
}
