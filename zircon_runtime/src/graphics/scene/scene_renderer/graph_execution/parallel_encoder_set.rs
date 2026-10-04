use std::collections::HashMap;

use crate::core::runtime::tasks::{parallel_map_indices, TaskPool};
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
        Self::partition_filtered(compiled, min_passes_per_bucket, |_, _| true)
    }

    pub(crate) fn partition_filtered(
        compiled: &CompiledRenderGraph,
        min_passes_per_bucket: usize,
        mut include_pass: impl FnMut(usize, &crate::render_graph::CompiledRenderPass) -> bool,
    ) -> Self {
        Self {
            buckets: topology_buckets(
                executable_topology_layers(compiled, |pass_index, pass| {
                    include_pass(pass_index, pass)
                }),
                min_passes_per_bucket,
            ),
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

    pub(crate) fn record_parallel_with_outputs<T, E, F>(
        &self,
        device: &wgpu::Device,
        pool: &TaskPool,
        record_bucket: F,
    ) -> Result<Vec<RecordedEncoderBucket<T>>, E>
    where
        T: Send,
        E: Send,
        F: Fn(&EncoderBucket, &mut wgpu::CommandEncoder) -> Result<Vec<T>, E> + Send + Sync,
    {
        record_buckets_ordered(&self.buckets, pool, |bucket| {
            let mut encoder = create_bucket_encoder(device);
            let outputs = record_bucket(bucket, &mut encoder)?;
            Ok(RecordedEncoderBucket {
                command_buffer: encoder.finish(),
                outputs,
            })
        })
        .into_iter()
        .collect()
    }
}

pub(crate) struct RecordedEncoderBucket<T> {
    command_buffer: wgpu::CommandBuffer,
    outputs: Vec<T>,
}

impl<T> RecordedEncoderBucket<T> {
    pub(crate) fn into_parts(self) -> (wgpu::CommandBuffer, Vec<T>) {
        (self.command_buffer, self.outputs)
    }
}

#[derive(Debug, PartialEq, Eq)]
struct TopologyLayer {
    order: usize,
    pass_indices: Vec<usize>,
}

fn executable_topology_layers(
    compiled: &CompiledRenderGraph,
    mut include_pass: impl FnMut(usize, &crate::render_graph::CompiledRenderPass) -> bool,
) -> Vec<TopologyLayer> {
    let passes = compiled.passes();
    let mut layer_by_pass_id = HashMap::<_, usize>::with_capacity(passes.len());
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
        if pass.culled || !include_pass(pass_index, pass) {
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
    parallel_map_indices(pool, buckets.len(), |index| record_bucket(&buckets[index]))
}

#[cfg(test)]
#[path = "tests/parallel_encoder_set.rs"]
mod tests;
