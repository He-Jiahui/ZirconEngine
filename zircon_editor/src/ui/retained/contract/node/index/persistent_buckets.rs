use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

pub(super) type Cell = (i32, i32);

#[derive(Clone, Debug, Default)]
pub(super) struct PersistentCellBuckets {
    root: Option<Arc<BucketNode>>,
    len: usize,
}

#[derive(Debug)]
struct BucketNode {
    key: u64,
    rows: Arc<Vec<usize>>,
    height: u8,
    left: Option<Arc<BucketNode>>,
    right: Option<Arc<BucketNode>>,
}

impl PersistentCellBuckets {
    pub(super) fn from_cells(cells: HashMap<Cell, Vec<usize>>) -> Self {
        let mut sorted = cells
            .into_iter()
            .filter(|(_, rows)| !rows.is_empty())
            .map(|(cell, rows)| (cell_key(cell), Arc::new(rows)))
            .collect::<Vec<_>>();
        sorted.sort_unstable_by_key(|(key, _)| *key);
        Self {
            root: build_balanced(&sorted),
            len: sorted.len(),
        }
    }

    pub(super) fn get(&self, cell: &Cell) -> Option<&Vec<usize>> {
        find_node(self.root.as_deref(), cell_key(*cell)).map(|node| node.rows.as_ref())
    }

    pub(super) fn with_updates(&self, updates: BTreeMap<Cell, Option<Vec<usize>>>) -> Self {
        let mut root = self.root.clone();
        let mut len = self.len;
        for (cell, rows) in updates {
            let key = cell_key(cell);
            let existed = find_node(root.as_deref(), key).is_some();
            match rows.filter(|rows| !rows.is_empty()) {
                Some(rows) => {
                    root = Some(insert(root, key, Arc::new(rows)));
                    if !existed {
                        len += 1;
                    }
                }
                None => {
                    root = remove(root, key);
                    if existed {
                        len -= 1;
                    }
                }
            }
        }
        Self { root, len }
    }

    #[cfg(test)]
    fn height_for_test(&self) -> u8 {
        height(&self.root)
    }
}

fn cell_key((x, y): Cell) -> u64 {
    (u64::from(x as u32) << 32) | u64::from(y as u32)
}

fn find_node(mut node: Option<&BucketNode>, key: u64) -> Option<&BucketNode> {
    while let Some(current) = node {
        if key < current.key {
            node = current.left.as_deref();
        } else if key > current.key {
            node = current.right.as_deref();
        } else {
            return Some(current);
        }
    }
    None
}

fn build_balanced(sorted: &[(u64, Arc<Vec<usize>>)]) -> Option<Arc<BucketNode>> {
    if sorted.is_empty() {
        return None;
    }
    let middle = sorted.len() / 2;
    Some(make_node(
        sorted[middle].0,
        Arc::clone(&sorted[middle].1),
        build_balanced(&sorted[..middle]),
        build_balanced(&sorted[middle + 1..]),
    ))
}

fn insert(node: Option<Arc<BucketNode>>, key: u64, rows: Arc<Vec<usize>>) -> Arc<BucketNode> {
    let Some(node) = node else {
        return make_node(key, rows, None, None);
    };
    if key < node.key {
        balance(
            node.key,
            Arc::clone(&node.rows),
            Some(insert(node.left.clone(), key, rows)),
            node.right.clone(),
        )
    } else if key > node.key {
        balance(
            node.key,
            Arc::clone(&node.rows),
            node.left.clone(),
            Some(insert(node.right.clone(), key, rows)),
        )
    } else {
        make_node(key, rows, node.left.clone(), node.right.clone())
    }
}

fn remove(node: Option<Arc<BucketNode>>, key: u64) -> Option<Arc<BucketNode>> {
    let node = node?;
    if key < node.key {
        return Some(balance(
            node.key,
            Arc::clone(&node.rows),
            remove(node.left.clone(), key),
            node.right.clone(),
        ));
    }
    if key > node.key {
        return Some(balance(
            node.key,
            Arc::clone(&node.rows),
            node.left.clone(),
            remove(node.right.clone(), key),
        ));
    }
    match (&node.left, &node.right) {
        (None, None) => None,
        (Some(left), None) => Some(Arc::clone(left)),
        (None, Some(right)) => Some(Arc::clone(right)),
        (Some(_), Some(right)) => {
            let successor = minimum(right);
            Some(balance(
                successor.key,
                Arc::clone(&successor.rows),
                node.left.clone(),
                remove(node.right.clone(), successor.key),
            ))
        }
    }
}

fn minimum(mut node: &Arc<BucketNode>) -> &BucketNode {
    while let Some(left) = &node.left {
        node = left;
    }
    node
}

fn balance(
    key: u64,
    rows: Arc<Vec<usize>>,
    left: Option<Arc<BucketNode>>,
    right: Option<Arc<BucketNode>>,
) -> Arc<BucketNode> {
    let factor = i16::from(height(&left)) - i16::from(height(&right));
    if factor > 1 {
        let left_node = left.as_ref().expect("left-heavy bucket node");
        if height(&left_node.left) < height(&left_node.right) {
            let rotated_left = rotate_left(Arc::clone(left_node));
            return rotate_right(make_node(key, rows, Some(rotated_left), right));
        }
        return rotate_right(make_node(key, rows, left, right));
    }
    if factor < -1 {
        let right_node = right.as_ref().expect("right-heavy bucket node");
        if height(&right_node.right) < height(&right_node.left) {
            let rotated_right = rotate_right(Arc::clone(right_node));
            return rotate_left(make_node(key, rows, left, Some(rotated_right)));
        }
        return rotate_left(make_node(key, rows, left, right));
    }
    make_node(key, rows, left, right)
}

fn rotate_left(node: Arc<BucketNode>) -> Arc<BucketNode> {
    let right = node
        .right
        .as_ref()
        .expect("left rotation requires right node");
    let left = make_node(
        node.key,
        Arc::clone(&node.rows),
        node.left.clone(),
        right.left.clone(),
    );
    make_node(
        right.key,
        Arc::clone(&right.rows),
        Some(left),
        right.right.clone(),
    )
}

fn rotate_right(node: Arc<BucketNode>) -> Arc<BucketNode> {
    let left = node
        .left
        .as_ref()
        .expect("right rotation requires left node");
    let right = make_node(
        node.key,
        Arc::clone(&node.rows),
        left.right.clone(),
        node.right.clone(),
    );
    make_node(
        left.key,
        Arc::clone(&left.rows),
        left.left.clone(),
        Some(right),
    )
}

fn make_node(
    key: u64,
    rows: Arc<Vec<usize>>,
    left: Option<Arc<BucketNode>>,
    right: Option<Arc<BucketNode>>,
) -> Arc<BucketNode> {
    Arc::new(BucketNode {
        key,
        rows,
        height: height(&left).max(height(&right)).saturating_add(1),
        left,
        right,
    })
}

fn height(node: &Option<Arc<BucketNode>>) -> u8 {
    node.as_ref().map_or(0, |node| node.height)
}

#[cfg(test)]
#[path = "tests/persistent_buckets.rs"]
mod tests;
