use bytemuck::{Pod, Zeroable};

use crate::graphics::scene::scene_renderer::mesh::build_mesh_draws::IndexedIndirectArgs;

pub(crate) const INDIRECT_COMPACTION_METADATA_STRIDE_BYTES: wgpu::BufferAddress =
    std::mem::size_of::<IndirectCompactionBatchMetadata>() as wgpu::BufferAddress;
pub(crate) const INDIRECT_VISIBLE_INSTANCE_INDEX_STRIDE_BYTES: wgpu::BufferAddress =
    std::mem::size_of::<u32>() as wgpu::BufferAddress;
pub(crate) const INDIRECT_DRAW_COUNT_BUFFER_SIZE_BYTES: wgpu::BufferAddress =
    std::mem::size_of::<u32>() as wgpu::BufferAddress;
pub(crate) const INDIRECT_COMPACTION_UNUSED_INSTANCE_INDEX: u32 = u32::MAX;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Pod, Zeroable)]
pub(crate) struct IndirectCompactionBatchMetadata {
    pub(crate) source_arg_index: u32,
    pub(crate) visible_instance_base: u32,
    pub(crate) source_first_instance: u32,
    pub(crate) source_instance_count: u32,
    pub(crate) output_arg_base: u32,
    pub(crate) draw_count_index: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct IndirectCompactionBatchRange {
    pub(crate) first_arg: u32,
    pub(crate) arg_count: u32,
    pub(crate) draw_count_index: u32,
}

impl IndirectCompactionBatchRange {
    pub(crate) const fn new(first_arg: u32, arg_count: u32, draw_count_index: u32) -> Self {
        Self {
            first_arg,
            arg_count,
            draw_count_index,
        }
    }
}

/// 将按批次排列的间接参数映射为遮挡剔除后的可见实例容量与计数。
/// 构造失败时调用方须退回原命令路径，不能消费部分 metadata。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct IndirectCompactionPlan {
    metadata: Vec<IndirectCompactionBatchMetadata>,
    visible_instance_capacity: u32,
    draw_count_count: u32,
}

impl IndirectCompactionPlan {
    pub(crate) fn try_from_args(args: &[IndexedIndirectArgs]) -> Option<Self> {
        if args.is_empty() {
            return Some(Self::default());
        }
        let arg_count = u32::try_from(args.len()).ok()?;
        Self::try_from_ordered_batch_ranges(
            args,
            [IndirectCompactionBatchRange::new(0, arg_count, 0)],
        )
    }

    /// 批次须按参数顺序连续覆盖整个 args；缺口、重叠或容量溢出返回 None，调用方应保留原绘制路径。
    pub(crate) fn try_from_ordered_batch_ranges(
        args: &[IndexedIndirectArgs],
        batch_ranges: impl IntoIterator<Item = IndirectCompactionBatchRange>,
    ) -> Option<Self> {
        let mut metadata = Vec::with_capacity(args.len());
        let mut visible_instance_capacity = 0u32;
        let mut draw_count_count = 0u32;
        let mut expected_first_arg = 0usize;

        for batch in batch_ranges {
            let first_arg = batch.first_arg as usize;
            let arg_count = batch.arg_count as usize;
            let end_arg = first_arg.checked_add(arg_count)?;
            if first_arg != expected_first_arg || end_arg > args.len() {
                return None;
            }
            draw_count_count = draw_count_count.max(batch.draw_count_index.checked_add(1)?);

            for source_arg_index in first_arg..end_arg {
                let args = &args[source_arg_index];
                let source_arg_index = u32::try_from(source_arg_index).ok()?;
                metadata.push(IndirectCompactionBatchMetadata {
                    source_arg_index,
                    visible_instance_base: visible_instance_capacity,
                    source_first_instance: args.first_instance,
                    source_instance_count: args.instance_count,
                    output_arg_base: batch.first_arg,
                    draw_count_index: batch.draw_count_index,
                });
                visible_instance_capacity =
                    visible_instance_capacity.checked_add(args.instance_count)?;
            }
            expected_first_arg = end_arg;
        }

        (expected_first_arg == args.len()).then_some(Self {
            metadata,
            visible_instance_capacity,
            draw_count_count,
        })
    }

    pub(crate) fn try_from_args_and_batch_ranges(
        args: &[IndexedIndirectArgs],
        batch_ranges: impl IntoIterator<Item = IndirectCompactionBatchRange>,
    ) -> Option<Self> {
        let mut metadata = vec![IndirectCompactionBatchMetadata::default(); args.len()];
        let mut visible_instance_capacity = 0u32;
        let mut draw_count_count = 0u32;
        let mut covered = vec![false; args.len()];

        for batch in batch_ranges {
            let first_arg = batch.first_arg as usize;
            let arg_count = batch.arg_count as usize;
            let end_arg = first_arg.checked_add(arg_count)?;
            if end_arg > args.len() {
                return None;
            }
            draw_count_count = draw_count_count.max(batch.draw_count_index.checked_add(1)?);

            for source_arg_index in first_arg..end_arg {
                if std::mem::replace(&mut covered[source_arg_index], true) {
                    return None;
                }
                let source_args = &args[source_arg_index];
                metadata[source_arg_index] = IndirectCompactionBatchMetadata {
                    source_arg_index: u32::try_from(source_arg_index).ok()?,
                    visible_instance_base: visible_instance_capacity,
                    source_first_instance: source_args.first_instance,
                    source_instance_count: source_args.instance_count,
                    output_arg_base: batch.first_arg,
                    draw_count_index: batch.draw_count_index,
                };
                visible_instance_capacity =
                    visible_instance_capacity.checked_add(source_args.instance_count)?;
            }
        }

        if covered.iter().any(|covered| !covered) {
            return None;
        }

        Some(Self {
            metadata,
            visible_instance_capacity,
            draw_count_count,
        })
    }

    #[cfg(test)]
    fn try_from_args_and_batches_for_test(
        args: &[IndexedIndirectArgs],
        batches: &[IndirectCompactionBatchRange],
    ) -> Option<Self> {
        Self::try_from_args_and_batch_ranges(args, batches.iter().copied())
    }

    pub(crate) fn metadata(&self) -> &[IndirectCompactionBatchMetadata] {
        &self.metadata
    }

    pub(crate) fn metadata_count(&self) -> u32 {
        self.metadata.len().min(u32::MAX as usize) as u32
    }

    pub(crate) const fn visible_instance_capacity(&self) -> u32 {
        self.visible_instance_capacity
    }

    pub(crate) const fn draw_count_count(&self) -> u32 {
        self.draw_count_count
    }

    pub(crate) fn metadata_buffer_byte_size(&self) -> wgpu::BufferAddress {
        u64::from(self.metadata_count()) * INDIRECT_COMPACTION_METADATA_STRIDE_BYTES
    }

    pub(crate) fn visible_instance_index_buffer_byte_size(&self) -> wgpu::BufferAddress {
        u64::from(self.visible_instance_capacity) * INDIRECT_VISIBLE_INSTANCE_INDEX_STRIDE_BYTES
    }

    pub(crate) fn draw_count_buffer_byte_size(&self) -> wgpu::BufferAddress {
        u64::from(self.draw_count_count) * INDIRECT_DRAW_COUNT_BUFFER_SIZE_BYTES
    }

    #[cfg(test)]
    fn compact_args_for_test(
        &self,
        args: &[IndexedIndirectArgs],
        mut is_visible: impl FnMut(u32) -> bool,
    ) -> IndirectCompactionSimulation {
        assert_eq!(args.len(), self.metadata.len());
        let mut compacted_args = vec![IndexedIndirectArgs::zeroed(); args.len()];
        let mut draw_counts = vec![0u32; self.draw_count_count as usize];
        let mut visible_instance_indices = vec![
            INDIRECT_COMPACTION_UNUSED_INSTANCE_INDEX;
            self.visible_instance_capacity as usize
        ];

        for source in &self.metadata {
            let args = args[source.source_arg_index as usize];
            let mut visible_count = 0u32;
            for offset in 0..source.source_instance_count {
                let source_instance = source.source_first_instance.saturating_add(offset);
                if !is_visible(source_instance) {
                    continue;
                }
                let remap_index = source
                    .visible_instance_base
                    .checked_add(visible_count)
                    .expect("test compaction remap index overflowed");
                visible_instance_indices[remap_index as usize] = source_instance;
                visible_count += 1;
            }
            if visible_count == 0 {
                continue;
            }

            let draw_count = &mut draw_counts[source.draw_count_index as usize];
            let output_arg_index = source
                .output_arg_base
                .checked_add(*draw_count)
                .expect("test compaction output arg index overflowed");
            *draw_count += 1;
            let mut output = args;
            output.first_instance = source.visible_instance_base;
            output.instance_count = visible_count;
            compacted_args[output_arg_index as usize] = output;
        }

        IndirectCompactionSimulation {
            compacted_args,
            draw_counts,
            visible_instance_indices,
        }
    }
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq)]
struct IndirectCompactionSimulation {
    compacted_args: Vec<IndexedIndirectArgs>,
    draw_counts: Vec<u32>,
    visible_instance_indices: Vec<u32>,
}

#[cfg(test)]
#[path = "tests/indirect_compaction.rs"]
mod tests;
