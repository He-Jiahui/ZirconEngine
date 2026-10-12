use std::collections::{BTreeSet, HashMap};

use crate::scene::{EntityId, SceneError, SceneResult};

#[derive(Clone, Copy)]
enum ParentChainState {
    Visiting,
    Complete { reaches_requested_root: bool },
}

/// Validate the union of reachable parent chains before collapsing covered roots.
/// Completed chains are shared across the batch, including densely requested chains.
pub(super) fn normalize_roots(
    roots: &BTreeSet<EntityId>,
    mut parent_of: impl FnMut(EntityId) -> Option<EntityId>,
) -> SceneResult<Vec<EntityId>> {
    let mut states = HashMap::with_capacity(roots.len());
    let mut path = Vec::new();
    let mut normalized = Vec::new();
    for start in roots.iter().copied() {
        if states.contains_key(&start) {
            continue;
        }
        let mut cursor = Some(start);
        let mut reaches_requested_root = loop {
            let Some(entity) = cursor else {
                break false;
            };
            match states.get(&entity).copied() {
                Some(ParentChainState::Complete {
                    reaches_requested_root,
                }) => break reaches_requested_root,
                Some(ParentChainState::Visiting) => {
                    return Err(SceneError::HierarchyParentChainCycle {
                        start,
                        repeated: entity,
                    });
                }
                None => {}
            }
            states.insert(entity, ParentChainState::Visiting);
            path.push(entity);
            cursor = parent_of(entity);
        };
        // A requested ancestor is a coverage boundary only after its entire chain
        // is known to terminate. Stopping there earlier would conceal a raw cycle.
        for entity in path.drain(..).rev() {
            let requested = roots.contains(&entity);
            if requested && !reaches_requested_root {
                normalized.push(entity);
            }
            reaches_requested_root |= requested;
            states.insert(
                entity,
                ParentChainState::Complete {
                    reaches_requested_root,
                },
            );
        }
    }
    Ok(normalized)
}
