use std::collections::BTreeMap;
use std::rc::Rc;

/// Persistent row-index map with path-copy updates bounded by the model's index width.
///
/// A sparse presentation update shares every untouched trie branch with the previous
/// presentation. Update cost therefore depends on `log2(row_count)`, never on the number of
/// patches accumulated by earlier frames.
pub(super) struct PersistentRowPatchMap<T> {
    root: Option<Rc<PersistentRowPatchNode<T>>>,
    row_count: usize,
    depth: u32,
    len: usize,
}

impl<T> Clone for PersistentRowPatchMap<T> {
    fn clone(&self) -> Self {
        Self {
            root: self.root.clone(),
            row_count: self.row_count,
            depth: self.depth,
            len: self.len,
        }
    }
}

enum PersistentRowPatchNode<T> {
    Leaf(Rc<T>),
    Branch {
        zero: Option<Rc<PersistentRowPatchNode<T>>>,
        one: Option<Rc<PersistentRowPatchNode<T>>>,
    },
}

impl<T> PersistentRowPatchMap<T> {
    pub(super) fn from_shared_rows(row_count: usize, rows: &BTreeMap<usize, Rc<T>>) -> Self {
        Self::empty(row_count).with_updates(
            rows.iter()
                .map(|(row, value)| (*row, Rc::clone(value)))
                .collect(),
        )
    }

    pub(super) fn with_updates(&self, rows: BTreeMap<usize, Rc<T>>) -> Self {
        let mut updated = self.clone();
        for (row, value) in rows {
            if row >= updated.row_count {
                continue;
            }
            let (root, replaced) = insert_node(updated.root.as_ref(), updated.depth, row, value);
            updated.root = Some(root);
            if !replaced {
                updated.len = updated.len.saturating_add(1);
            }
        }
        updated
    }

    pub(super) fn get(&self, row: usize) -> Option<&Rc<T>> {
        if row >= self.row_count {
            return None;
        }
        let mut node = self.root.as_deref()?;
        let mut depth = self.depth;
        while depth > 0 {
            let PersistentRowPatchNode::Branch { zero, one } = node else {
                return None;
            };
            depth -= 1;
            let child = if row & (1usize << depth) == 0 {
                zero
            } else {
                one
            };
            node = child.as_deref()?;
        }
        match node {
            PersistentRowPatchNode::Leaf(value) => Some(value),
            PersistentRowPatchNode::Branch { .. } => None,
        }
    }

    pub(super) fn len(&self) -> usize {
        self.len
    }

    pub(super) fn shares_storage_with(&self, other: &Self) -> bool {
        match (&self.root, &other.root) {
            (Some(left), Some(right)) => Rc::ptr_eq(left, right),
            (None, None) => true,
            _ => false,
        }
    }

    pub(super) fn empty(row_count: usize) -> Self {
        let depth = usize::BITS - row_count.saturating_sub(1).leading_zeros();
        Self {
            root: None,
            row_count,
            depth,
            len: 0,
        }
    }
}

fn insert_node<T>(
    node: Option<&Rc<PersistentRowPatchNode<T>>>,
    depth: u32,
    row: usize,
    value: Rc<T>,
) -> (Rc<PersistentRowPatchNode<T>>, bool) {
    if depth == 0 {
        return (Rc::new(PersistentRowPatchNode::Leaf(value)), node.is_some());
    }

    let (mut zero, mut one) = match node.map(Rc::as_ref) {
        Some(PersistentRowPatchNode::Branch { zero, one }) => (zero.clone(), one.clone()),
        Some(PersistentRowPatchNode::Leaf(_)) | None => (None, None),
    };
    let child_depth = depth - 1;
    let child = if row & (1usize << child_depth) == 0 {
        &mut zero
    } else {
        &mut one
    };
    let (updated_child, replaced) = insert_node(child.as_ref(), child_depth, row, value);
    *child = Some(updated_child);
    (
        Rc::new(PersistentRowPatchNode::Branch { zero, one }),
        replaced,
    )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::rc::Rc;

    use super::PersistentRowPatchMap;

    #[test]
    fn repeated_single_row_updates_keep_history_out_of_the_update_cost() {
        let mut patches = PersistentRowPatchMap::empty(10_000);
        let first_version = patches.with_updates(BTreeMap::from([(0, Rc::new(1usize))]));
        patches = first_version.clone();
        for row in 1..10_000 {
            patches = patches.with_updates(BTreeMap::from([(row, Rc::new(row + 1))]));
        }

        assert_eq!(patches.depth, 14);
        assert_eq!(patches.len(), 10_000);
        assert_eq!(patches.get(0).map(|value| **value), Some(1));
        assert_eq!(patches.get(9_999).map(|value| **value), Some(10_000));
        assert!(first_version.get(9_999).is_none());
    }

    #[test]
    fn replacing_a_row_keeps_the_patch_cardinality_stable() {
        let patches =
            PersistentRowPatchMap::empty(16).with_updates(BTreeMap::from([(3, Rc::new("old"))]));
        let replaced = patches.with_updates(BTreeMap::from([(3, Rc::new("new"))]));

        assert_eq!(replaced.len(), 1);
        assert_eq!(replaced.get(3).map(|value| **value), Some("new"));
        assert_eq!(patches.get(3).map(|value| **value), Some("old"));
    }
}
