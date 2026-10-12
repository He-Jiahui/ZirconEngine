use std::collections::HashMap;

use crate::scene::{EntityId, SceneError, SceneResult};

#[derive(Clone, Copy)]
enum ParentChainState {
    Visiting(usize),
    Complete,
}

/// Check only parent chains reachable from restored rows in the prospective
/// union of stored batch parents and authoritative live Hierarchy rows.
// 批次父边覆盖同 ID 的现场父边，只检查从恢复行可达的联合父链；无关现场环留给原有层级修复阶段处理。
pub(super) fn validate_restore_parent_chains(
    batch_parents: &HashMap<EntityId, Option<EntityId>>,
    starts: impl IntoIterator<Item = EntityId>,
    mut live_parent_of: impl FnMut(EntityId) -> Option<EntityId>,
) -> SceneResult<()> {
    let mut states = HashMap::with_capacity(batch_parents.len());
    let mut path = Vec::new();
    for start in starts {
        if matches!(states.get(&start), Some(ParentChainState::Complete)) {
            continue;
        }
        let mut cursor = Some(start);
        while let Some(entity) = cursor {
            match states.get(&entity).copied() {
                Some(ParentChainState::Complete) => break,
                Some(ParentChainState::Visiting(cycle_start)) => {
                    if let Some(child) = path[cycle_start..]
                        .iter()
                        .copied()
                        .find(|candidate| batch_parents.contains_key(candidate))
                    {
                        // A batch edge in this cycle is introduced by restoring the
                        // union. A live-only cycle was already present before restore.
                        let parent = batch_parents[&child]
                            .expect("an entity on a parent cycle must have a parent");
                        return Err(SceneError::HierarchyCycle { child, parent });
                    }
                    return Err(SceneError::HierarchyParentChainCycle {
                        start,
                        repeated: entity,
                    });
                }
                None => {}
            }
            states.insert(entity, ParentChainState::Visiting(path.len()));
            path.push(entity);
            cursor = match batch_parents.get(&entity) {
                Some(parent) => *parent,
                None => live_parent_of(entity),
            };
        }
        for entity in path.drain(..) {
            states.insert(entity, ParentChainState::Complete);
        }
    }
    Ok(())
}
