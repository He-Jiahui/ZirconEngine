use std::collections::BTreeMap;

use zircon_runtime_interface::ui::{event_ui::UiNodeId, tree::UiTree};

use super::slot::UiLayoutSlotIndex;

#[derive(Clone, Debug, Default)]
pub(super) struct UiMaterializedVirtualListLayoutIndex {
    owners: BTreeMap<UiNodeId, UiMaterializedVirtualListLayout>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct UiMaterializedVirtualListLayout {
    logical_count: usize,
    logical_indices_by_child: BTreeMap<UiNodeId, usize>,
}

impl UiLayoutSlotIndex {
    pub(crate) fn replace_materialized_virtual_list(
        &self,
        owner_id: UiNodeId,
        logical_count: usize,
        assignments: impl IntoIterator<Item = (UiNodeId, usize)>,
    ) {
        let logical_indices_by_child = assignments
            .into_iter()
            .filter(|(_, logical_index)| *logical_index < logical_count)
            .collect();
        self.virtual_lists.borrow_mut().owners.insert(
            owner_id,
            UiMaterializedVirtualListLayout {
                logical_count,
                logical_indices_by_child,
            },
        );
    }

    pub(crate) fn clear_materialized_virtual_list(&self, owner_id: UiNodeId) {
        self.virtual_lists.borrow_mut().owners.remove(&owner_id);
    }

    pub(crate) fn prune_materialized_virtual_lists(&self, tree: &UiTree) -> usize {
        let mut index = self.virtual_lists.borrow_mut();
        let previous_count = index.owners.len();
        index
            .owners
            .retain(|owner_id, _| tree.nodes.contains_key(owner_id));
        previous_count - index.owners.len()
    }

    pub(super) fn with_materialized_virtual_list<T>(
        &self,
        owner_id: UiNodeId,
        read: impl FnOnce(Option<&UiMaterializedVirtualListLayout>) -> T,
    ) -> T {
        let index = self.virtual_lists.borrow();
        read(index.owners.get(&owner_id))
    }
}

impl UiMaterializedVirtualListLayout {
    pub(super) fn logical_count(&self) -> usize {
        self.logical_count
    }

    pub(super) fn logical_index_for_child(&self, child_id: UiNodeId) -> Option<usize> {
        self.logical_indices_by_child.get(&child_id).copied()
    }

    #[cfg(test)]
    pub(super) fn assignment_count(&self) -> usize {
        self.logical_indices_by_child.len()
    }
}
