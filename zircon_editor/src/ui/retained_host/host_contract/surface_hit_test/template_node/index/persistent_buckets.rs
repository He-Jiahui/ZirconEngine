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
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    #[test]
    fn updates_preserve_old_snapshot_and_delete_vacated_cells() {
        let original = PersistentCellBuckets::from_cells(HashMap::from([
            ((0, 0), vec![1, 2]),
            ((1, 0), vec![3]),
        ]));
        let updated = original.with_updates(BTreeMap::from([
            ((0, 0), Some(vec![4])),
            ((1, 0), None),
            ((2, 0), Some(vec![5])),
        ]));

        assert_eq!(original.get(&(0, 0)), Some(&vec![1, 2]));
        assert_eq!(original.get(&(1, 0)), Some(&vec![3]));
        assert_eq!(updated.get(&(0, 0)), Some(&vec![4]));
        assert_eq!(updated.get(&(1, 0)), None);
        assert_eq!(updated.get(&(2, 0)), Some(&vec![5]));
    }

    #[test]
    fn repeated_path_copy_updates_keep_lookup_depth_balanced() {
        let mut buckets = PersistentCellBuckets::default();
        for x in 0..512 {
            buckets = buckets.with_updates(BTreeMap::from([((x, 0), Some(vec![x as usize]))]));
        }

        assert!(buckets.height_for_test() <= 11);
        assert_eq!(buckets.get(&(511, 0)), Some(&vec![511]));
    }

    #[test]
    fn optimization_batch_gy_editor580_from_cells_sorts_one_dense_buffer() {
        let buckets = PersistentCellBuckets::from_cells(HashMap::from([
            ((3, 1), vec![31]),
            ((-2, 4), Vec::new()),
            ((0, 0), vec![1, 2]),
            ((-1, -1), vec![7]),
        ]));

        assert_eq!(buckets.len, 3);
        assert_eq!(buckets.get(&(3, 1)), Some(&vec![31]));
        assert_eq!(buckets.get(&(0, 0)), Some(&vec![1, 2]));
        assert_eq!(buckets.get(&(-1, -1)), Some(&vec![7]));
        assert_eq!(buckets.get(&(-2, 4)), None);

        let source = include_str!("persistent_buckets.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("persistent bucket production source");
        assert!(production.contains("sorted.sort_unstable_by_key"));
        assert!(!production.contains("collect::<BTreeMap<_, _>>()"));
    }

    #[test]
    #[ignore = "release performance evidence; run through the validation coordinator"]
    fn optimization_batch_gy_editor580_dense_sort_performance_evidence() {
        fn legacy_from_cells(cells: HashMap<Cell, Vec<usize>>) -> PersistentCellBuckets {
            let sorted = cells
                .into_iter()
                .filter(|(_, rows)| !rows.is_empty())
                .map(|(cell, rows)| (cell_key(cell), Arc::new(rows)))
                .collect::<BTreeMap<_, _>>()
                .into_iter()
                .collect::<Vec<_>>();
            PersistentCellBuckets {
                root: build_balanced(&sorted),
                len: sorted.len(),
            }
        }

        const CELL_COUNT: usize = 8_192;
        let cells = (0..CELL_COUNT)
            .map(|index| {
                let x = (index as i32).wrapping_mul(17);
                let y = (index as i32).wrapping_mul(-31);
                ((x, y), vec![index, index + 1])
            })
            .collect::<HashMap<_, _>>();
        let mut legacy_samples = Vec::with_capacity(17);
        let mut optimized_samples = Vec::with_capacity(17);
        for _ in 0..17 {
            let legacy_input = cells.clone();
            let started = Instant::now();
            black_box(legacy_from_cells(black_box(legacy_input)));
            legacy_samples.push(started.elapsed().as_nanos());

            let optimized_input = cells.clone();
            let started = Instant::now();
            black_box(PersistentCellBuckets::from_cells(black_box(
                optimized_input,
            )));
            optimized_samples.push(started.elapsed().as_nanos());
        }

        legacy_samples.sort_unstable();
        optimized_samples.sort_unstable();
        let legacy_p95 = legacy_samples[16];
        let optimized_p95 = optimized_samples[16];
        println!(
            "EDITOR580_PERSISTENT_BUCKET_DENSE_SORT_BENCH_V1 cells={CELL_COUNT} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} target_ratio_bp=7000"
        );
        assert!(
            optimized_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(7_000),
            "persistent bucket dense sort P95 {optimized_p95} ns exceeded 70% of legacy {legacy_p95} ns"
        );
    }
}
