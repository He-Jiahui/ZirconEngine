use zircon_runtime_interface::ui::event_ui::UiNodeId;
use zircon_runtime_interface::ui::tree::{UiTree, UiTreeError};

const COMMON_BUBBLE_ROUTE_DEPTH: usize = 16;

pub trait UiRuntimeTreeRoutingExt {
    fn bubble_route(&self, node_id: UiNodeId) -> Result<Vec<UiNodeId>, UiTreeError>;
}

impl UiRuntimeTreeRoutingExt for UiTree {
    fn bubble_route(&self, node_id: UiNodeId) -> Result<Vec<UiNodeId>, UiTreeError> {
        let mut route = Vec::with_capacity(route_initial_capacity(self.nodes.len()));
        let mut current = Some(node_id);
        while let Some(id) = current {
            let node = self.nodes.get(&id).ok_or(UiTreeError::MissingNode(id))?;
            route.push(id);
            current = node.parent;
        }
        Ok(route)
    }
}

fn route_initial_capacity(node_count: usize) -> usize {
    node_count.min(COMMON_BUBBLE_ROUTE_DEPTH)
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use zircon_runtime_interface::ui::event_ui::{UiNodePath, UiTreeId};
    use zircon_runtime_interface::ui::tree::UiTreeNode;

    use super::*;

    const ROUTE_DEPTH: usize = 16;
    const ROUTES_PER_SAMPLE: usize = 32_768;
    const SAMPLE_PAIRS: usize = 17;

    #[test]
    fn optimization_batch_fq_runtime473_reserves_common_bubble_route_depth() {
        let tree = deep_tree(ROUTE_DEPTH);
        let route = tree
            .bubble_route(UiNodeId::new(ROUTE_DEPTH as u64))
            .expect("deep route");

        assert_eq!(route.len(), ROUTE_DEPTH);
        assert_eq!(route.first(), Some(&UiNodeId::new(ROUTE_DEPTH as u64)));
        assert_eq!(route.last(), Some(&UiNodeId::new(1)));
        assert_eq!(route_initial_capacity(tree.nodes.len()), ROUTE_DEPTH);
        assert!(route.capacity() >= ROUTE_DEPTH);
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_fq_runtime473_reserved_bubble_route_benchmark() {
        let route = (1..=ROUTE_DEPTH)
            .rev()
            .map(|id| UiNodeId::new(id as u64))
            .collect::<Vec<_>>();
        for _ in 0..4 {
            black_box(measure_routes(&route, false));
            black_box(measure_routes(&route, true));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_routes(&route, false));
                optimized_samples.push(measure_routes(&route, true));
            } else {
                optimized_samples.push(measure_routes(&route, true));
                legacy_samples.push(measure_routes(&route, false));
            }
        }

        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "RUNTIME473_RESERVED_BUBBLE_ROUTE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} route_depth={ROUTE_DEPTH} routes_per_sample={ROUTES_PER_SAMPLE} legacy_growth_allocations_per_route=3 optimized_growth_allocations_per_route=0 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=40",
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(optimized_p95 <= legacy_p95 * 60 / 100);
    }

    fn deep_tree(depth: usize) -> UiTree {
        let mut tree = UiTree::new(UiTreeId::new("routing-capacity"));
        tree.insert_root(UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root")));
        for id in 2..=depth {
            tree.insert_child(
                UiNodeId::new((id - 1) as u64),
                UiTreeNode::new(
                    UiNodeId::new(id as u64),
                    UiNodePath::new(format!("node-{id}")),
                ),
            )
            .expect("parent exists");
        }
        tree
    }

    fn measure_routes(route: &[UiNodeId], optimized: bool) -> u128 {
        let started = Instant::now();
        for _ in 0..ROUTES_PER_SAMPLE {
            let mut collected = if optimized {
                Vec::with_capacity(route_initial_capacity(route.len()))
            } else {
                Vec::new()
            };
            for node_id in black_box(route).iter().copied() {
                collected.push(node_id);
            }
            black_box(collected);
        }
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
