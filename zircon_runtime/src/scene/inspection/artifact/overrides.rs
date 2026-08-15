use std::sync::Arc;

use super::super::WorldInspectionHierarchyRow;

pub(super) type HierarchyRowOverrides = PersistentIndexMap<WorldInspectionHierarchyRow>;
pub(super) type HierarchyChildHashOverrides = PersistentIndexMap<u64>;

/// Persistent sparse row replacements shared across inspection generations.
///
/// The Patricia-tree path is bounded by `usize::BITS`. Updating one row clones only the nodes on
/// that path, so publishing a new generation never copies all historical replacements.
#[derive(Clone, Debug)]
pub(super) struct PersistentIndexMap<T> {
    root: Option<Arc<PersistentIndexNode<T>>>,
    len: usize,
}

#[derive(Debug)]
enum PersistentIndexNode<T> {
    Leaf {
        index: usize,
        value: T,
    },
    Branch {
        branching_bit: usize,
        zero: Arc<PersistentIndexNode<T>>,
        one: Arc<PersistentIndexNode<T>>,
    },
}

impl<T> Default for PersistentIndexMap<T> {
    fn default() -> Self {
        Self { root: None, len: 0 }
    }
}

impl<T: PartialEq> PartialEq for PersistentIndexMap<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.len != other.len {
            return false;
        }
        let mut equal = true;
        self.for_each(|index, value| {
            equal &= other.get(&index) == Some(value);
        });
        equal
    }
}

impl<T> PersistentIndexMap<T> {
    pub(super) fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub(super) fn len(&self) -> usize {
        self.len
    }

    pub(super) fn get(&self, index: &usize) -> Option<&T> {
        let mut node = self.root.as_deref()?;
        loop {
            match node {
                PersistentIndexNode::Leaf {
                    index: stored_index,
                    value,
                } => return (*stored_index == *index).then_some(value),
                PersistentIndexNode::Branch {
                    branching_bit,
                    zero,
                    one,
                } => {
                    node = if follows_zero_branch(*index, *branching_bit) {
                        zero
                    } else {
                        one
                    };
                }
            }
        }
    }

    pub(super) fn insert(&mut self, index: usize, value: T) {
        let Some(root) = self.root.as_ref() else {
            self.root = Some(Arc::new(PersistentIndexNode::Leaf { index, value }));
            self.len = 1;
            return;
        };

        let existing_index = routed_leaf_index(root, index);
        if existing_index == index {
            self.root = Some(replace_value(root, index, value));
            return;
        }

        let Some(branching_bit) = highest_bit(existing_index ^ index) else {
            return;
        };
        let leaf = Arc::new(PersistentIndexNode::Leaf { index, value });
        self.root = Some(insert_branch(root.clone(), leaf, index, branching_bit));
        self.len = self.len.saturating_add(1);
    }

    pub(super) fn for_each(&self, mut visitor: impl FnMut(usize, &T)) {
        if let Some(root) = self.root.as_deref() {
            visit_rows(root, &mut visitor);
        }
    }
}

fn routed_leaf_index<T>(mut node: &PersistentIndexNode<T>, index: usize) -> usize {
    loop {
        match node {
            PersistentIndexNode::Leaf { index, .. } => return *index,
            PersistentIndexNode::Branch {
                branching_bit,
                zero,
                one,
            } => {
                node = if follows_zero_branch(index, *branching_bit) {
                    zero
                } else {
                    one
                };
            }
        }
    }
}

fn replace_value<T>(
    node: &Arc<PersistentIndexNode<T>>,
    index: usize,
    value: T,
) -> Arc<PersistentIndexNode<T>> {
    match node.as_ref() {
        PersistentIndexNode::Leaf {
            index: stored_index,
            ..
        } if *stored_index == index => Arc::new(PersistentIndexNode::Leaf { index, value }),
        PersistentIndexNode::Leaf { .. } => node.clone(),
        PersistentIndexNode::Branch {
            branching_bit,
            zero,
            one,
        } => {
            let (zero, one) = if follows_zero_branch(index, *branching_bit) {
                (replace_value(zero, index, value), one.clone())
            } else {
                (zero.clone(), replace_value(one, index, value))
            };
            Arc::new(PersistentIndexNode::Branch {
                branching_bit: *branching_bit,
                zero,
                one,
            })
        }
    }
}

fn insert_branch<T>(
    node: Arc<PersistentIndexNode<T>>,
    leaf: Arc<PersistentIndexNode<T>>,
    index: usize,
    branching_bit: usize,
) -> Arc<PersistentIndexNode<T>> {
    if let PersistentIndexNode::Branch {
        branching_bit: current_bit,
        zero,
        one,
    } = node.as_ref()
    {
        if *current_bit > branching_bit {
            let (zero, one) = if follows_zero_branch(index, *current_bit) {
                (
                    insert_branch(zero.clone(), leaf, index, branching_bit),
                    one.clone(),
                )
            } else {
                (
                    zero.clone(),
                    insert_branch(one.clone(), leaf, index, branching_bit),
                )
            };
            return Arc::new(PersistentIndexNode::Branch {
                branching_bit: *current_bit,
                zero,
                one,
            });
        }
    }

    let (zero, one) = if follows_zero_branch(index, branching_bit) {
        (leaf, node)
    } else {
        (node, leaf)
    };
    Arc::new(PersistentIndexNode::Branch {
        branching_bit,
        zero,
        one,
    })
}

fn follows_zero_branch(index: usize, branching_bit: usize) -> bool {
    index & branching_bit == 0
}

fn highest_bit(value: usize) -> Option<usize> {
    (value != 0).then(|| 1usize << (usize::BITS - value.leading_zeros() - 1))
}

fn visit_rows<T>(node: &PersistentIndexNode<T>, visitor: &mut impl FnMut(usize, &T)) {
    match node {
        PersistentIndexNode::Leaf { index, value } => visitor(*index, value),
        PersistentIndexNode::Branch { zero, one, .. } => {
            visit_rows(zero, visitor);
            visit_rows(one, visitor);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::HierarchyRowOverrides;
    use crate::scene::WorldInspectionHierarchyRow;

    #[test]
    fn cloned_index_preserves_prior_rows_and_visits_each_current_replacement() {
        let mut rows = HierarchyRowOverrides::default();
        for index in [0, 1, 7, 63, 64, 1_024, usize::MAX] {
            rows.insert(index, row(index, "initial"));
        }
        let snapshot = rows.clone();

        rows.insert(63, row(63, "updated"));
        rows.insert(255, row(255, "new"));

        assert_eq!(snapshot.len(), 7);
        assert_eq!(
            snapshot.get(&63).map(|row| row.display_name.as_str()),
            Some("initial")
        );
        assert!(snapshot.get(&255).is_none());
        assert_eq!(rows.len(), 8);
        assert_eq!(
            rows.get(&63).map(|row| row.display_name.as_str()),
            Some("updated")
        );
        assert_eq!(
            rows.get(&255).map(|row| row.display_name.as_str()),
            Some("new")
        );
        let mut visited = Vec::new();
        rows.for_each(|index, _| visited.push(index));
        visited.sort_unstable();
        assert_eq!(visited, vec![0, 1, 7, 63, 64, 255, 1_024, usize::MAX]);
    }

    fn row(index: usize, display_name: &str) -> WorldInspectionHierarchyRow {
        WorldInspectionHierarchyRow {
            entity: index as u64,
            parent: None,
            depth: 0,
            display_name: display_name.to_string(),
            kind: "Empty".to_string(),
            subtree_hash: index as u64,
            focused: false,
            active_in_hierarchy: true,
            has_children: false,
        }
    }
}
