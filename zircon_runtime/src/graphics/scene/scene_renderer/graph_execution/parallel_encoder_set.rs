use std::collections::HashMap;

use rayon::prelude::*;

use crate::core::TaskPool;
use crate::render_graph::CompiledRenderGraph;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EncoderBucket {
    topology_order: usize,
    first_topology_layer: usize,
    last_topology_layer: usize,
    pass_indices: Vec<usize>,
}

impl EncoderBucket {
    pub(crate) fn topology_order(&self) -> usize {
        self.topology_order
    }

    pub(crate) fn pass_indices(&self) -> &[usize] {
        &self.pass_indices
    }

    pub(crate) fn topology_layer_range(&self) -> std::ops::RangeInclusive<usize> {
        self.first_topology_layer..=self.last_topology_layer
    }

    pub(crate) fn pass_count(&self) -> usize {
        self.pass_indices.len()
    }
}

/// Owns contiguous executable-pass buckets in compiled graph topology order.
///
/// Parallel recorders must capture only immutable prepared inputs. Mutable render-graph resource
/// owners remain on the serial preparation path until they expose independently owned pass data.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ParallelEncoderSet {
    buckets: Vec<EncoderBucket>,
}

impl ParallelEncoderSet {
    pub(crate) fn partition(compiled: &CompiledRenderGraph, min_passes_per_bucket: usize) -> Self {
        Self {
            buckets: topology_buckets(executable_topology_layers(compiled), min_passes_per_bucket),
        }
    }

    pub(crate) fn buckets(&self) -> &[EncoderBucket] {
        &self.buckets
    }

    pub(crate) fn should_record_parallel(&self, parallel_record: bool, pool: &TaskPool) -> bool {
        parallel_record && pool.parallelism() > 1 && self.buckets.len() > 1
    }

    /// Records one command buffer per profitable bucket and returns them in graph topology order.
    /// Disabled or undersized workloads record all buckets into one command buffer.
    pub(crate) fn record_parallel<F>(
        self,
        device: &wgpu::Device,
        pool: &TaskPool,
        parallel_record: bool,
        record_bucket: F,
    ) -> Vec<wgpu::CommandBuffer>
    where
        F: Fn(&EncoderBucket, &mut wgpu::CommandEncoder) + Send + Sync,
    {
        if self.buckets.is_empty() {
            return Vec::new();
        }
        if !self.should_record_parallel(parallel_record, pool) {
            let mut encoder = create_bucket_encoder(device);
            for bucket in &self.buckets {
                record_bucket(bucket, &mut encoder);
            }
            return vec![encoder.finish()];
        }
        record_buckets_ordered(&self.buckets, pool, |bucket| {
            let mut encoder = create_bucket_encoder(device);
            record_bucket(bucket, &mut encoder);
            encoder.finish()
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
struct TopologyLayer {
    order: usize,
    pass_indices: Vec<usize>,
}

fn executable_topology_layers(compiled: &CompiledRenderGraph) -> Vec<TopologyLayer> {
    let passes = compiled.passes();
    let mut layer_by_pass_id = HashMap::with_capacity(passes.len());
    let mut pass_indices_by_layer = Vec::<Vec<usize>>::new();
    for (pass_index, pass) in passes.iter().enumerate() {
        let layer = pass
            .dependencies
            .iter()
            .filter_map(|dependency| layer_by_pass_id.get(dependency).copied())
            .map(|dependency_layer| dependency_layer.saturating_add(1))
            .max()
            .unwrap_or(0);
        layer_by_pass_id.insert(pass.id, layer);
        if pass.culled {
            continue;
        }
        while pass_indices_by_layer.len() <= layer {
            pass_indices_by_layer.push(Vec::new());
        }
        if let Some(pass_indices) = pass_indices_by_layer.get_mut(layer) {
            pass_indices.push(pass_index);
        }
    }
    pass_indices_by_layer
        .into_iter()
        .enumerate()
        .filter_map(|(order, pass_indices)| {
            (!pass_indices.is_empty()).then_some(TopologyLayer {
                order,
                pass_indices,
            })
        })
        .collect()
}

fn topology_buckets(
    layers: Vec<TopologyLayer>,
    min_passes_per_bucket: usize,
) -> Vec<EncoderBucket> {
    let min_passes_per_bucket = min_passes_per_bucket.max(1);
    let mut buckets = Vec::<EncoderBucket>::new();
    let mut pending = None::<EncoderBucket>;
    for layer in layers {
        let bucket = pending.get_or_insert_with(|| EncoderBucket {
            topology_order: buckets.len(),
            first_topology_layer: layer.order,
            last_topology_layer: layer.order,
            pass_indices: Vec::new(),
        });
        bucket.last_topology_layer = layer.order;
        bucket.pass_indices.extend(layer.pass_indices);
        if bucket.pass_count() >= min_passes_per_bucket {
            if let Some(bucket) = pending.take() {
                buckets.push(bucket);
            }
        }
    }
    if let Some(mut tail) = pending {
        if let Some(last) = buckets.last_mut() {
            last.last_topology_layer = tail.last_topology_layer;
            last.pass_indices.append(&mut tail.pass_indices);
        } else {
            buckets.push(tail);
        }
    }
    buckets
}

fn create_bucket_encoder(device: &wgpu::Device) -> wgpu::CommandEncoder {
    device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-parallel-render-graph-bucket"),
    })
}

fn record_buckets_ordered<T, F>(
    buckets: &[EncoderBucket],
    pool: &TaskPool,
    record_bucket: F,
) -> Vec<T>
where
    T: Send,
    F: Fn(&EncoderBucket) -> T + Send + Sync,
{
    pool.install(|| buckets.par_iter().map(record_bucket).collect())
}

#[cfg(test)]
mod tests {
    use crate::core::{TaskPool, TaskPoolDescriptor};
    use crate::render_graph::{PassFlags, QueueLane, RenderGraphBuilder};

    use super::{record_buckets_ordered, ParallelEncoderSet};

    #[test]
    fn parallel_encoder_partition_respects_topology_layers_and_skips_culled_passes() {
        let graph = compiled_graph_with_executable_chain(7);

        let encoder_set = ParallelEncoderSet::partition(&graph, 2);

        assert_eq!(
            encoder_set
                .buckets()
                .iter()
                .map(|bucket| bucket.pass_count())
                .collect::<Vec<_>>(),
            vec![2, 2, 3]
        );
        assert_eq!(
            encoder_set
                .buckets()
                .iter()
                .map(|bucket| bucket.topology_layer_range())
                .collect::<Vec<_>>(),
            vec![0..=1, 2..=3, 4..=6]
        );
        assert_eq!(
            encoder_set
                .buckets()
                .iter()
                .flat_map(|bucket| bucket.pass_indices())
                .map(|pass_index| graph.passes()[*pass_index].name.clone())
                .collect::<Vec<_>>(),
            (0..7)
                .map(|index| format!("pass-{index}"))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn parallel_bucket_results_preserve_topology_order() {
        let graph = compiled_graph_with_executable_chain(8);
        let encoder_set = ParallelEncoderSet::partition(&graph, 2);
        let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(2));

        let recorded = record_buckets_ordered(encoder_set.buckets(), &pool, |bucket| {
            bucket.topology_order()
        });

        assert_eq!(recorded, vec![0, 1, 2, 3]);
        assert!(encoder_set.should_record_parallel(true, &pool));
        assert!(!encoder_set.should_record_parallel(false, &pool));
    }

    #[test]
    fn parallel_encoder_partition_never_splits_a_topology_layer() {
        let graph = compiled_graph_with_wide_root_layer(4);

        let encoder_set = ParallelEncoderSet::partition(&graph, 1);

        assert_eq!(
            encoder_set
                .buckets()
                .iter()
                .map(|bucket| bucket.pass_count())
                .collect::<Vec<_>>(),
            vec![4, 1]
        );
        assert_eq!(
            encoder_set
                .buckets()
                .iter()
                .map(|bucket| bucket.topology_layer_range())
                .collect::<Vec<_>>(),
            vec![0..=0, 1..=1]
        );
    }

    #[test]
    fn parallel_encoder_set_falls_back_when_only_one_bucket_is_profitable() {
        let graph = compiled_graph_with_executable_chain(3);
        let encoder_set = ParallelEncoderSet::partition(&graph, 2);
        let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(2));

        assert_eq!(encoder_set.buckets().len(), 1);
        assert!(!encoder_set.should_record_parallel(true, &pool));
    }

    fn compiled_graph_with_executable_chain(
        pass_count: usize,
    ) -> crate::render_graph::CompiledRenderGraph {
        let mut builder = RenderGraphBuilder::new("parallel-encoder-set");
        builder.add_pass("culled", QueueLane::Graphics);
        let mut previous = None;
        let mut last = None;
        for index in 0..pass_count {
            let pass = builder.add_pass(format!("pass-{index}"), QueueLane::Graphics);
            if let Some(previous) = previous {
                builder.add_dependency(previous, pass).unwrap();
            }
            previous = Some(pass);
            last = Some(pass);
        }
        if let Some(last) = last {
            builder
                .set_pass_flags(
                    last,
                    PassFlags {
                        has_side_effects: true,
                        ..PassFlags::default()
                    },
                )
                .unwrap();
        }
        builder.compile().unwrap()
    }

    fn compiled_graph_with_wide_root_layer(
        root_pass_count: usize,
    ) -> crate::render_graph::CompiledRenderGraph {
        let mut builder = RenderGraphBuilder::new("parallel-wide-root-layer");
        let roots = (0..root_pass_count)
            .map(|index| builder.add_pass(format!("root-{index}"), QueueLane::Graphics))
            .collect::<Vec<_>>();
        let final_pass = builder.add_pass("final", QueueLane::Graphics);
        for root in roots {
            builder.add_dependency(root, final_pass).unwrap();
        }
        builder
            .set_pass_flags(
                final_pass,
                PassFlags {
                    has_side_effects: true,
                    ..PassFlags::default()
                },
            )
            .unwrap();
        builder.compile().unwrap()
    }
}
