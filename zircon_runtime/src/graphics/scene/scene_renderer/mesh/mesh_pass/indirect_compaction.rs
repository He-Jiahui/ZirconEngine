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
mod tests {
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use super::*;

    const PERFORMANCE_ARG_COUNT: usize = 65_536;
    const PERFORMANCE_BATCH_SIZE: usize = 32;
    const PERFORMANCE_SAMPLE_COUNT: usize = 17;

    #[test]
    fn indirect_compaction_metadata_preserves_source_spans_and_prefixes_output_capacity() {
        let args = [
            indirect_args(36, 3, 10),
            indirect_args(18, 0, 20),
            indirect_args(12, 2, 40),
        ];

        let plan = IndirectCompactionPlan::try_from_args(&args).expect("compaction plan");

        assert_eq!(INDIRECT_COMPACTION_METADATA_STRIDE_BYTES, 24);
        assert_eq!(INDIRECT_VISIBLE_INSTANCE_INDEX_STRIDE_BYTES, 4);
        assert_eq!(plan.metadata_count(), 3);
        assert_eq!(plan.visible_instance_capacity(), 5);
        assert_eq!(plan.draw_count_count(), 1);
        assert_eq!(plan.metadata_buffer_byte_size(), 72);
        assert_eq!(plan.visible_instance_index_buffer_byte_size(), 20);
        assert_eq!(plan.draw_count_buffer_byte_size(), 4);
        assert_eq!(
            plan.metadata(),
            &[
                IndirectCompactionBatchMetadata {
                    source_arg_index: 0,
                    visible_instance_base: 0,
                    source_first_instance: 10,
                    source_instance_count: 3,
                    output_arg_base: 0,
                    draw_count_index: 0,
                },
                IndirectCompactionBatchMetadata {
                    source_arg_index: 1,
                    visible_instance_base: 3,
                    source_first_instance: 20,
                    source_instance_count: 0,
                    output_arg_base: 0,
                    draw_count_index: 0,
                },
                IndirectCompactionBatchMetadata {
                    source_arg_index: 2,
                    visible_instance_base: 3,
                    source_first_instance: 40,
                    source_instance_count: 2,
                    output_arg_base: 0,
                    draw_count_index: 0,
                },
            ]
        );
    }

    #[test]
    fn indirect_compaction_simulation_rewrites_args_to_visible_instance_remap() {
        let args = [indirect_args(36, 3, 10), indirect_args(18, 2, 40)];
        let plan = IndirectCompactionPlan::try_from_args(&args).expect("compaction plan");

        let simulation =
            plan.compact_args_for_test(&args, |instance| matches!(instance, 10 | 12 | 41));

        assert_eq!(simulation.compacted_args[0].first_instance, 0);
        assert_eq!(simulation.compacted_args[0].instance_count, 2);
        assert_eq!(simulation.compacted_args[1].first_instance, 3);
        assert_eq!(simulation.compacted_args[1].instance_count, 1);
        assert_eq!(simulation.draw_counts, vec![2]);
        assert_eq!(
            simulation.visible_instance_indices,
            vec![
                10,
                12,
                INDIRECT_COMPACTION_UNUSED_INSTANCE_INDEX,
                41,
                INDIRECT_COMPACTION_UNUSED_INSTANCE_INDEX,
            ]
        );
    }

    #[test]
    fn indirect_compaction_simulation_keeps_counts_and_outputs_per_draw_batch() {
        let args = [
            indirect_args(36, 1, 10),
            indirect_args(18, 1, 20),
            indirect_args(12, 1, 30),
        ];
        let plan = IndirectCompactionPlan::try_from_args_and_batches_for_test(
            &args,
            &[
                IndirectCompactionBatchRange::new(0, 2, 0),
                IndirectCompactionBatchRange::new(2, 1, 1),
            ],
        )
        .expect("compaction plan");

        let simulation = plan.compact_args_for_test(&args, |instance| matches!(instance, 20 | 30));

        assert_eq!(plan.draw_count_count(), 2);
        assert_eq!(plan.draw_count_buffer_byte_size(), 8);
        assert_eq!(simulation.draw_counts, vec![1, 1]);
        assert_eq!(simulation.compacted_args[0].first_instance, 1);
        assert_eq!(simulation.compacted_args[0].instance_count, 1);
        assert_eq!(simulation.compacted_args[1].instance_count, 0);
        assert_eq!(simulation.compacted_args[2].first_instance, 2);
        assert_eq!(simulation.compacted_args[2].instance_count, 1);
    }

    #[test]
    fn indirect_compaction_ordered_ranges_match_the_general_plan() {
        let args = [
            indirect_args(36, 2, 10),
            indirect_args(18, 1, 20),
            indirect_args(12, 3, 30),
        ];
        let batches = [
            IndirectCompactionBatchRange::new(0, 2, 0),
            IndirectCompactionBatchRange::new(2, 1, 1),
        ];

        let ordered =
            IndirectCompactionPlan::try_from_ordered_batch_ranges(&args, batches.iter().copied())
                .expect("ordered compaction plan");
        let general = IndirectCompactionPlan::try_from_args_and_batches_for_test(&args, &batches)
            .expect("general compaction plan");

        assert_eq!(ordered, general);
    }

    #[test]
    fn optimization_batch_cy_runtime400_direct_index_preserves_reordered_batch_metadata() {
        let args = [
            indirect_args(36, 2, 10),
            indirect_args(18, 1, 20),
            indirect_args(12, 3, 30),
            indirect_args(6, 4, 40),
        ];
        let batches = [
            IndirectCompactionBatchRange::new(2, 2, 1),
            IndirectCompactionBatchRange::new(0, 2, 0),
        ];

        let legacy = legacy_general_plan(&args, &batches).expect("legacy compaction plan");
        let optimized = IndirectCompactionPlan::try_from_args_and_batches_for_test(&args, &batches)
            .expect("direct-index compaction plan");

        assert_eq!(optimized, legacy);
        assert_eq!(
            optimized
                .metadata()
                .iter()
                .map(|metadata| metadata.source_arg_index)
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3]
        );
    }

    #[test]
    fn optimization_batch_cy_runtime400_general_plan_writes_metadata_by_source_index() {
        let production = include_str!("indirect_compaction.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source");

        assert!(production.contains("let mut metadata = vec!["));
        assert!(production.contains("metadata[source_arg_index] ="));
        assert!(!production.contains("metadata.sort_by_key"));
    }

    #[test]
    #[ignore = "release performance evidence"]
    fn optimization_batch_cy_runtime400_direct_index_performance_evidence() {
        let args = (0..PERFORMANCE_ARG_COUNT)
            .map(|index| {
                indirect_args(
                    3,
                    (index % 5) as u32,
                    u32::try_from(index).expect("fixture index fits u32"),
                )
            })
            .collect::<Vec<_>>();
        let mut batches = (0..PERFORMANCE_ARG_COUNT)
            .step_by(PERFORMANCE_BATCH_SIZE)
            .enumerate()
            .map(|(draw_count_index, first_arg)| {
                IndirectCompactionBatchRange::new(
                    u32::try_from(first_arg).expect("fixture first arg fits u32"),
                    PERFORMANCE_BATCH_SIZE as u32,
                    u32::try_from(draw_count_index).expect("fixture draw count fits u32"),
                )
            })
            .collect::<Vec<_>>();
        batches.reverse();
        assert_eq!(
            legacy_general_plan(&args, &batches),
            IndirectCompactionPlan::try_from_args_and_batches_for_test(&args, &batches)
        );

        let mut legacy_samples = Vec::with_capacity(PERFORMANCE_SAMPLE_COUNT);
        let mut direct_samples = Vec::with_capacity(PERFORMANCE_SAMPLE_COUNT);
        for sample in 0..PERFORMANCE_SAMPLE_COUNT {
            if sample % 2 == 0 {
                legacy_samples.push(measure(|| {
                    black_box(legacy_general_plan(black_box(&args), black_box(&batches)))
                }));
                direct_samples.push(measure(|| {
                    black_box(IndirectCompactionPlan::try_from_args_and_batches_for_test(
                        black_box(&args),
                        black_box(&batches),
                    ))
                }));
            } else {
                direct_samples.push(measure(|| {
                    black_box(IndirectCompactionPlan::try_from_args_and_batches_for_test(
                        black_box(&args),
                        black_box(&batches),
                    ))
                }));
                legacy_samples.push(measure(|| {
                    black_box(legacy_general_plan(black_box(&args), black_box(&batches)))
                }));
            }
        }

        let legacy_p95 = percentile_95(&mut legacy_samples);
        let direct_p95 = percentile_95(&mut direct_samples);
        println!(
            "RUNTIME400_INDIRECT_DIRECT_INDEX_BENCH_V1 args={PERFORMANCE_ARG_COUNT} \
             batches={} legacy_sort=true direct_index=true legacy_p95_ns={} direct_p95_ns={}",
            batches.len(),
            legacy_p95.as_nanos(),
            direct_p95.as_nanos(),
        );
        assert!(
            direct_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 70,
            "direct-index P95 {:?} exceeded 70% of sort-based P95 {:?}",
            direct_p95,
            legacy_p95,
        );
    }

    #[test]
    fn indirect_compaction_rejects_visible_instance_capacity_overflow() {
        let args = [indirect_args(36, u32::MAX, 0), indirect_args(18, 1, 7)];

        assert!(IndirectCompactionPlan::try_from_args(&args).is_none());
    }

    fn legacy_general_plan(
        args: &[IndexedIndirectArgs],
        batches: &[IndirectCompactionBatchRange],
    ) -> Option<IndirectCompactionPlan> {
        let mut metadata = Vec::with_capacity(args.len());
        let mut visible_instance_capacity = 0u32;
        let mut draw_count_count = 0u32;
        let mut covered = vec![false; args.len()];

        for batch in batches {
            let first_arg = batch.first_arg as usize;
            let end_arg = first_arg.checked_add(batch.arg_count as usize)?;
            if end_arg > args.len() {
                return None;
            }
            draw_count_count = draw_count_count.max(batch.draw_count_index.checked_add(1)?);
            for source_arg_index in first_arg..end_arg {
                if std::mem::replace(&mut covered[source_arg_index], true) {
                    return None;
                }
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
        }
        if covered.iter().any(|covered| !covered) {
            return None;
        }
        metadata.sort_by_key(|metadata| metadata.source_arg_index);
        Some(IndirectCompactionPlan {
            metadata,
            visible_instance_capacity,
            draw_count_count,
        })
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

    fn indirect_args(
        index_count: u32,
        instance_count: u32,
        first_instance: u32,
    ) -> IndexedIndirectArgs {
        IndexedIndirectArgs {
            index_count,
            instance_count,
            first_index: 0,
            base_vertex: 0,
            first_instance,
        }
    }
}
