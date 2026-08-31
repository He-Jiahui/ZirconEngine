use zircon_runtime_interface::ui::event_ui::UiNodeId;
use zircon_runtime_interface::ui::tree::{UiTree, UiTreeError};

pub trait UiRuntimeTreeRenderOrderExt {
    fn draw_order(&self) -> Vec<UiNodeId>;
    fn is_visible_in_tree(&self, node_id: UiNodeId) -> Result<bool, UiTreeError>;
}

impl UiRuntimeTreeRenderOrderExt for UiTree {
    fn draw_order(&self) -> Vec<UiNodeId> {
        let mut order: Vec<_> = self
            .nodes
            .values()
            .map(|node| (node.z_index, node.paint_order, node.node_id))
            .collect();
        order.sort_unstable_by_key(|entry| (entry.0, entry.1, entry.2));
        order.into_iter().map(|(_, _, node_id)| node_id).collect()
    }

    fn is_visible_in_tree(&self, node_id: UiNodeId) -> Result<bool, UiTreeError> {
        let mut current = Some(node_id);
        while let Some(id) = current {
            let node = self.nodes.get(&id).ok_or(UiTreeError::MissingNode(id))?;
            if !node.is_render_visible() {
                return Ok(false);
            }
            current = node.parent;
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use zircon_runtime_interface::ui::event_ui::{UiNodePath, UiTreeId};
    use zircon_runtime_interface::ui::tree::UiTreeNode;

    use super::*;

    const PERFORMANCE_NODE_COUNT: usize = 65_536;
    const PERFORMANCE_SAMPLE_COUNT: usize = 17;

    #[test]
    fn optimization_batch_cz_runtime403_node_id_tie_break_preserves_stable_draw_order() {
        let mut tree = UiTree::new(UiTreeId::new("render-order-ties"));
        for node_id in [3, 1, 2] {
            tree.insert_root(UiTreeNode::new(
                UiNodeId::new(node_id),
                UiNodePath::new(format!("root/{node_id}")),
            ));
        }
        for node_id in [1, 2, 3] {
            let node = tree.node_mut(UiNodeId::new(node_id)).expect("fixture node");
            node.z_index = 7;
            node.paint_order = 11;
        }

        assert_eq!(tree.draw_order(), legacy_draw_order(&tree));
        assert_eq!(
            tree.draw_order(),
            vec![UiNodeId::new(1), UiNodeId::new(2), UiNodeId::new(3)]
        );
    }

    #[test]
    fn optimization_batch_cz_runtime403_draw_order_uses_total_unstable_key() {
        let production = include_str!("render_order.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source");

        assert!(production.contains("order.sort_unstable_by_key"));
        assert!(production.contains("(entry.0, entry.1, entry.2)"));
        assert!(!production.contains("order.sort_by_key"));
    }

    #[test]
    #[ignore = "release performance evidence"]
    fn optimization_batch_cz_runtime403_draw_order_unstable_sort_performance_evidence() {
        let tree = render_tree_fixture();
        assert_eq!(tree.draw_order(), legacy_draw_order(&tree));

        let mut legacy_samples = Vec::with_capacity(PERFORMANCE_SAMPLE_COUNT);
        let mut unstable_samples = Vec::with_capacity(PERFORMANCE_SAMPLE_COUNT);
        for sample in 0..PERFORMANCE_SAMPLE_COUNT {
            if sample % 2 == 0 {
                legacy_samples.push(measure(|| legacy_draw_order(black_box(&tree))));
                unstable_samples.push(measure(|| black_box(&tree).draw_order()));
            } else {
                unstable_samples.push(measure(|| black_box(&tree).draw_order()));
                legacy_samples.push(measure(|| legacy_draw_order(black_box(&tree))));
            }
        }

        let legacy_p95 = percentile_95(&mut legacy_samples);
        let unstable_p95 = percentile_95(&mut unstable_samples);
        println!(
            "RUNTIME403_UI_DRAW_ORDER_UNSTABLE_SORT_BENCH_V1 nodes={PERFORMANCE_NODE_COUNT} \
             stable_sort=true total_unstable_key=true legacy_p95_ns={} unstable_p95_ns={}",
            legacy_p95.as_nanos(),
            unstable_p95.as_nanos(),
        );
        assert!(
            unstable_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 70,
            "unstable draw-order P95 {:?} exceeded 70% of stable P95 {:?}",
            unstable_p95,
            legacy_p95,
        );
    }

    fn render_tree_fixture() -> UiTree {
        let mut tree = UiTree::new(UiTreeId::new("render-order-performance"));
        for index in 0..PERFORMANCE_NODE_COUNT {
            let node_id = (index * 48_271) % PERFORMANCE_NODE_COUNT + 1;
            tree.insert_root(
                UiTreeNode::new(
                    UiNodeId::new(u64::try_from(node_id).expect("fixture node id fits u64")),
                    UiNodePath::new(format!("root/{node_id}")),
                )
                .with_z_index(((index * 257) % 4_096) as i32 - 2_048),
            );
        }
        tree
    }

    fn legacy_draw_order(tree: &UiTree) -> Vec<UiNodeId> {
        let mut order = tree
            .nodes
            .values()
            .map(|node| (node.z_index, node.paint_order, node.node_id))
            .collect::<Vec<_>>();
        order.sort_by_key(|entry| (entry.0, entry.1));
        order.into_iter().map(|(_, _, node_id)| node_id).collect()
    }

    fn measure<T>(run: impl FnOnce() -> T) -> Duration {
        let started = Instant::now();
        black_box(run());
        started.elapsed()
    }

    fn percentile_95(samples: &mut [Duration]) -> Duration {
        samples.sort_unstable();
        samples[(samples.len() - 1) * 95 / 100]
    }
}
