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

#[derive(Clone, Copy)]
enum PatchCursorDirection {
    Forward,
    Reverse,
}

const PATCH_CURSOR_CAPACITY: usize = usize::BITS as usize;

struct PendingPatchStack<'a, T> {
    entries: [Option<&'a PersistentRowPatchNode<T>>; PATCH_CURSOR_CAPACITY],
    occupied: usize,
}

impl<'a, T> PendingPatchStack<'a, T> {
    fn new() -> Self {
        Self {
            entries: std::array::from_fn(|_| None),
            occupied: 0,
        }
    }

    fn push(&mut self, depth: u32, node: &'a PersistentRowPatchNode<T>) {
        let bit = 1usize
            .checked_shl(depth)
            .expect("patch cursor depth fits the usize trie");
        assert_eq!(
            self.occupied & bit,
            0,
            "a depth-first trie frontier owns at most one sibling per level"
        );
        let slot = self
            .entries
            .get_mut(depth as usize)
            .expect("patch cursor stack capacity covers every usize trie level");
        *slot = Some(node);
        self.occupied |= bit;
    }

    fn pop(&mut self) -> Option<(u32, &'a PersistentRowPatchNode<T>)> {
        if self.occupied == 0 {
            return None;
        }
        let depth = self.occupied.trailing_zeros();
        self.occupied &= !(1usize << depth);
        self.entries[depth as usize]
            .take()
            .map(|node| (depth, node))
    }
}

pub(super) struct PersistentRowPatchCursor<'a, T> {
    direction: PatchCursorDirection,
    pending: PendingPatchStack<'a, T>,
    current: Option<(usize, &'a Rc<T>)>,
    #[cfg(test)]
    node_visits: usize,
}

impl<'a, T> PersistentRowPatchCursor<'a, T> {
    fn new(map: &'a PersistentRowPatchMap<T>, direction: PatchCursorDirection) -> Self {
        let mut cursor = Self {
            direction,
            pending: PendingPatchStack::new(),
            current: None,
            #[cfg(test)]
            node_visits: 0,
        };
        if let Some(root) = map.root.as_deref() {
            cursor.descend(root, map.depth, 0);
        }
        cursor
    }

    pub(super) fn value_at(&mut self, row: usize) -> Option<&'a Rc<T>> {
        while self
            .current
            .is_some_and(|(current_row, _)| match self.direction {
                PatchCursorDirection::Forward => current_row < row,
                PatchCursorDirection::Reverse => current_row > row,
            })
        {
            self.advance();
        }
        self.current
            .filter(|(current_row, _)| *current_row == row)
            .map(|(_, value)| value)
    }

    fn advance(&mut self) {
        let Some(previous_row) = self.current.map(|(row, _)| row) else {
            return;
        };
        self.current = None;
        let Some((depth, node)) = self.pending.pop() else {
            return;
        };
        let lower_bit_count = depth + 1;
        let high_prefix = if lower_bit_count >= usize::BITS {
            0
        } else {
            previous_row & (usize::MAX << lower_bit_count)
        };
        let row_prefix = match self.direction {
            PatchCursorDirection::Forward => high_prefix | (1usize << depth),
            PatchCursorDirection::Reverse => high_prefix,
        };
        self.descend(node, depth, row_prefix);
    }

    fn descend(
        &mut self,
        mut node: &'a PersistentRowPatchNode<T>,
        mut depth: u32,
        mut row_prefix: usize,
    ) {
        loop {
            #[cfg(test)]
            {
                self.node_visits = self.node_visits.saturating_add(1);
            }
            match node {
                PersistentRowPatchNode::Leaf(value) if depth == 0 => {
                    self.current = Some((row_prefix, value));
                    return;
                }
                PersistentRowPatchNode::Branch { zero, one } if depth > 0 => {
                    let child_depth = depth - 1;
                    match self.direction {
                        PatchCursorDirection::Forward => match (zero.as_deref(), one.as_deref()) {
                            (Some(zero), one) => {
                                if let Some(one) = one {
                                    self.pending.push(child_depth, one);
                                }
                                node = zero;
                            }
                            (None, Some(one)) => {
                                row_prefix |= 1usize << child_depth;
                                node = one;
                            }
                            (None, None) => return,
                        },
                        PatchCursorDirection::Reverse => match (zero.as_deref(), one.as_deref()) {
                            (zero, Some(one)) => {
                                if let Some(zero) = zero {
                                    self.pending.push(child_depth, zero);
                                }
                                row_prefix |= 1usize << child_depth;
                                node = one;
                            }
                            (Some(zero), None) => {
                                node = zero;
                            }
                            (None, None) => return,
                        },
                    }
                    depth = child_depth;
                }
                PersistentRowPatchNode::Leaf(_) | PersistentRowPatchNode::Branch { .. } => return,
            }
        }
    }

    #[cfg(test)]
    fn node_visits(&self) -> usize {
        self.node_visits
    }
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

    pub(super) fn forward_cursor(&self) -> PersistentRowPatchCursor<'_, T> {
        PersistentRowPatchCursor::new(self, PatchCursorDirection::Forward)
    }

    pub(super) fn reverse_cursor(&self) -> PersistentRowPatchCursor<'_, T> {
        PersistentRowPatchCursor::new(self, PatchCursorDirection::Reverse)
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
#[path = "tests/persistent_row_patch_map.rs"]
mod tests;
