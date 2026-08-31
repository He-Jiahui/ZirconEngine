use std::hint::black_box;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::graphics::RenderFeatureCapabilityRequirement;
use crate::render_graph::{QueueLane, RenderGraphBuilder};

use super::runtime_metadata::CompiledRenderPipelineRuntimeMetadata;

const RUNTIME89_LOOKUP_PASS_COUNT: usize = 2_048;
const RUNTIME89_LOOKUP_SAMPLE_PAIRS: usize = 21;

#[test]
fn execution_packet_indexes_stage_metadata_by_compiled_pass_identity() {
    let mut builder = RenderGraphBuilder::new("execution-packet-pass-identity");
    let depth = builder.add_pass("depth", QueueLane::Graphics);
    let post = builder.add_pass("post", QueueLane::AsyncCompute);
    builder
        .set_pass_flags(
            post,
            crate::render_graph::PassFlags {
                has_side_effects: true,
                ..crate::render_graph::PassFlags::default()
            },
        )
        .expect("packet fixture cull root");
    let graph = builder.compile().expect("packet fixture graph");

    let packet = super::RenderGraphExecutionPacket::new(
        graph,
        vec![
            super::RenderGraphExecutionPassMetadata::new(
                depth,
                super::RenderPassStage::DepthPrepass,
            ),
            super::RenderGraphExecutionPassMetadata::new(post, super::RenderPassStage::PostProcess),
        ],
    )
    .expect("packet should index every compiled pass exactly once");

    assert_eq!(
        packet
            .passes_for_stage(super::RenderPassStage::PostProcess)
            .map(|pass| packet.graph().passes()[pass.graph_pass_index].id)
            .collect::<Vec<_>>(),
        vec![post]
    );
    let depth_index = packet
        .graph()
        .indexed_pass(depth)
        .expect("depth identity should resolve in the compiled graph")
        .0;
    assert_eq!(
        packet.execution_pass_at(depth_index).map(|pass| pass.stage),
        Some(super::RenderPassStage::DepthPrepass)
    );
}

#[test]
fn execution_packet_carries_exact_access_ids_in_compiled_order() {
    let mut builder = RenderGraphBuilder::new("execution-packet-access-identities");
    let color = builder.import_present_external_resource("viewport-color");
    let depth = builder.import_present_external_resource("viewport-depth");
    let consumer = builder.add_pass("consumer", QueueLane::Graphics);
    let producer = builder.add_pass("producer", QueueLane::Graphics);
    builder
        .write_external(producer, color)
        .expect("producer writes the external color");
    builder
        .write_external(producer, depth)
        .expect("producer writes the external depth");
    builder
        .read_external(consumer, color)
        .expect("consumer reads the producer color");
    builder
        .read_external(consumer, depth)
        .expect("consumer reads the producer depth");
    builder
        .add_dependency(producer, consumer)
        .expect("producer must precede the later-authored consumer");
    builder
        .set_pass_flags(
            consumer,
            crate::render_graph::PassFlags {
                has_side_effects: true,
                ..crate::render_graph::PassFlags::default()
            },
        )
        .expect("packet fixture cull root");
    let graph = builder.compile().expect("packet fixture graph");
    let producer_color_access = graph
        .access_id_at(producer, 0)
        .expect("producer color compiled access ID");
    let producer_depth_access = graph
        .access_id_at(producer, 1)
        .expect("producer depth compiled access ID");
    let consumer_color_access = graph
        .access_id_at(consumer, 0)
        .expect("consumer color compiled access ID");
    let consumer_depth_access = graph
        .access_id_at(consumer, 1)
        .expect("consumer depth compiled access ID");
    let packet = super::RenderGraphExecutionPacket::new(
        graph,
        vec![
            super::RenderGraphExecutionPassMetadata::new(
                consumer,
                super::RenderPassStage::PostProcess,
            ),
            super::RenderGraphExecutionPassMetadata::new(
                producer,
                super::RenderPassStage::DepthPrepass,
            ),
        ],
    )
    .expect("packet should preserve every compiled access identity");
    let producer_index = packet
        .graph()
        .indexed_pass(producer)
        .expect("producer compiled index")
        .0;
    let consumer_index = packet
        .graph()
        .indexed_pass(consumer)
        .expect("consumer compiled index")
        .0;

    assert_eq!(
        packet.access_ids_for_pass(producer_index),
        Some([producer_color_access, producer_depth_access].as_slice())
    );
    assert_eq!(
        packet.access_ids_for_pass(consumer_index),
        Some([consumer_color_access, consumer_depth_access].as_slice())
    );
}

#[test]
fn execution_packet_rejects_duplicate_compiled_pass_metadata() {
    let mut builder = RenderGraphBuilder::new("execution-packet-validation");
    let first = builder.add_pass("first", QueueLane::Graphics);
    let second = builder.add_pass("second", QueueLane::Graphics);
    builder
        .set_pass_flags(
            second,
            crate::render_graph::PassFlags {
                has_side_effects: true,
                ..crate::render_graph::PassFlags::default()
            },
        )
        .expect("packet fixture cull root");
    let graph = builder.compile().expect("packet fixture graph");

    let error = super::RenderGraphExecutionPacket::new(
        graph,
        vec![
            super::RenderGraphExecutionPassMetadata::new(
                first,
                super::RenderPassStage::DepthPrepass,
            ),
            super::RenderGraphExecutionPassMetadata::new(
                first,
                super::RenderPassStage::PostProcess,
            ),
        ],
    )
    .expect_err("duplicate metadata must be rejected before execution");

    assert!(error.contains("duplicates compiled pass identity"));
}

#[test]
fn execution_packet_rejects_missing_compiled_pass_metadata() {
    let mut builder = RenderGraphBuilder::new("execution-packet-missing-metadata");
    let first = builder.add_pass("first", QueueLane::Graphics);
    let second = builder.add_pass("second", QueueLane::Graphics);
    builder
        .set_pass_flags(
            second,
            crate::render_graph::PassFlags {
                has_side_effects: true,
                ..crate::render_graph::PassFlags::default()
            },
        )
        .expect("packet fixture cull root");
    let graph = builder.compile().expect("packet fixture graph");

    let error = super::RenderGraphExecutionPacket::new(
        graph,
        vec![super::RenderGraphExecutionPassMetadata::new(
            first,
            super::RenderPassStage::DepthPrepass,
        )],
    )
    .expect_err("missing metadata must be rejected before execution");

    assert!(error.contains("missing stage metadata"));
}

#[test]
fn execution_packet_rejects_a_pass_identity_from_another_graph() {
    let mut foreign_builder = RenderGraphBuilder::new("execution-packet-foreign-identity");
    let foreign_pass = foreign_builder.add_pass("foreign", QueueLane::Graphics);

    let mut builder = RenderGraphBuilder::new("execution-packet-target-identity");
    let target_pass = builder.add_pass("target", QueueLane::Graphics);
    builder
        .set_pass_flags(
            target_pass,
            crate::render_graph::PassFlags {
                has_side_effects: true,
                ..crate::render_graph::PassFlags::default()
            },
        )
        .expect("packet fixture cull root");
    let graph = builder.compile().expect("packet fixture graph");

    let error = super::RenderGraphExecutionPacket::new(
        graph,
        vec![super::RenderGraphExecutionPassMetadata::new(
            foreign_pass,
            super::RenderPassStage::DepthPrepass,
        )],
    )
    .expect_err("a pass identity must belong to the compiled graph");

    assert!(error.contains("references missing compiled pass identity"));
}

#[test]
fn execution_packet_metadata_completion_stays_on_the_fallible_error_path() {
    let source = include_str!("execution_packet.rs");

    assert!(!source.contains(".expect("));
    assert!(!source.contains(".unwrap("));
}

#[test]
fn execution_packet_stage_ranges_keep_compiled_graph_order() {
    let mut builder = RenderGraphBuilder::new("execution-packet-stage-ranges");
    let consumer_depth = builder.add_pass("consumer-depth", QueueLane::Graphics);
    let producer_depth = builder.add_pass("producer-depth", QueueLane::Graphics);
    let post = builder.add_pass("post", QueueLane::Graphics);
    let second_depth = builder.add_pass("second-depth", QueueLane::Graphics);
    builder
        .add_dependency(producer_depth, consumer_depth)
        .expect("packet fixture reordering dependency");
    builder
        .add_dependency(consumer_depth, post)
        .expect("packet fixture post dependency");
    builder
        .add_dependency(post, second_depth)
        .expect("packet fixture second depth dependency");
    for pass in [consumer_depth, producer_depth, post, second_depth] {
        builder
            .set_pass_flags(
                pass,
                crate::render_graph::PassFlags {
                    has_side_effects: true,
                    ..crate::render_graph::PassFlags::default()
                },
            )
            .expect("packet fixture cull root");
    }
    let graph = builder.compile().expect("packet fixture graph");
    let packet = super::RenderGraphExecutionPacket::new(
        graph,
        vec![
            super::RenderGraphExecutionPassMetadata::new(
                consumer_depth,
                super::RenderPassStage::DepthPrepass,
            ),
            super::RenderGraphExecutionPassMetadata::new(
                producer_depth,
                super::RenderPassStage::DepthPrepass,
            ),
            super::RenderGraphExecutionPassMetadata::new(post, super::RenderPassStage::PostProcess),
            super::RenderGraphExecutionPassMetadata::new(
                second_depth,
                super::RenderPassStage::DepthPrepass,
            ),
        ],
    )
    .expect("packet should compile stage ranges");

    assert_eq!(
        packet
            .passes_for_stage(super::RenderPassStage::DepthPrepass)
            .map(|execution_pass| packet.graph().passes()[execution_pass.graph_pass_index].id)
            .collect::<Vec<_>>(),
        vec![producer_depth, consumer_depth, second_depth]
    );
}

#[test]
fn execution_packet_cursor_rejects_out_of_order_live_passes() {
    let mut builder = RenderGraphBuilder::new("execution-packet-cursor-order");
    let first = builder.add_pass("first", QueueLane::Graphics);
    let second = builder.add_pass("second", QueueLane::Graphics);
    for pass in [first, second] {
        builder
            .set_pass_flags(
                pass,
                crate::render_graph::PassFlags {
                    has_side_effects: true,
                    ..crate::render_graph::PassFlags::default()
                },
            )
            .expect("cursor fixture cull root");
    }
    let graph = builder.compile().expect("cursor fixture graph");
    let packet = super::RenderGraphExecutionPacket::new(
        graph,
        vec![
            super::RenderGraphExecutionPassMetadata::new(
                first,
                super::RenderPassStage::DepthPrepass,
            ),
            super::RenderGraphExecutionPassMetadata::new(
                second,
                super::RenderPassStage::PostProcess,
            ),
        ],
    )
    .expect("cursor fixture packet");
    let second_index = packet.graph().indexed_pass(second).expect("second pass").0;
    let mut cursor = packet.begin_execution();

    let error = packet
        .admit_execution_pass(&mut cursor, second_index)
        .expect_err("stage routing may not execute a later graph pass first");

    assert!(error.contains("expected compiled graph pass"));
    assert!(error.contains("first"));
    assert!(error.contains("second"));
}

#[test]
fn execution_packet_cursor_skips_culled_passes_without_execution_slots() {
    let mut builder = RenderGraphBuilder::new("execution-packet-cursor-culling");
    let culled = builder.add_pass("culled", QueueLane::Graphics);
    let live = builder.add_pass("live", QueueLane::Graphics);
    builder
        .set_pass_flags(
            live,
            crate::render_graph::PassFlags {
                has_side_effects: true,
                ..crate::render_graph::PassFlags::default()
            },
        )
        .expect("cursor fixture live root");
    let graph = builder.compile().expect("cursor fixture graph");
    let packet = super::RenderGraphExecutionPacket::new(
        graph,
        vec![
            super::RenderGraphExecutionPassMetadata::new(
                culled,
                super::RenderPassStage::DepthPrepass,
            ),
            super::RenderGraphExecutionPassMetadata::new(live, super::RenderPassStage::PostProcess),
        ],
    )
    .expect("cursor fixture packet");
    let live_index = packet.graph().indexed_pass(live).expect("live pass").0;
    let mut cursor = packet.begin_execution();

    packet
        .admit_execution_pass(&mut cursor, live_index)
        .expect("the first live pass must be admitted after skipped culled passes");
    packet
        .finish_execution(cursor)
        .expect("culled passes must not leave the packet incomplete");
}

#[test]
fn execution_packet_cursor_rejects_missing_live_passes_at_frame_tail() {
    let mut builder = RenderGraphBuilder::new("execution-packet-cursor-completion");
    let first = builder.add_pass("first", QueueLane::Graphics);
    let missing = builder.add_pass("missing", QueueLane::Graphics);
    for pass in [first, missing] {
        builder
            .set_pass_flags(
                pass,
                crate::render_graph::PassFlags {
                    has_side_effects: true,
                    ..crate::render_graph::PassFlags::default()
                },
            )
            .expect("cursor fixture cull root");
    }
    let graph = builder.compile().expect("cursor fixture graph");
    let packet = super::RenderGraphExecutionPacket::new(
        graph,
        vec![
            super::RenderGraphExecutionPassMetadata::new(
                first,
                super::RenderPassStage::DepthPrepass,
            ),
            super::RenderGraphExecutionPassMetadata::new(
                missing,
                super::RenderPassStage::PostProcess,
            ),
        ],
    )
    .expect("cursor fixture packet");
    let first_index = packet.graph().indexed_pass(first).expect("first pass").0;
    let mut cursor = packet.begin_execution();
    packet
        .admit_execution_pass(&mut cursor, first_index)
        .expect("first live pass must be admitted");

    let error = packet
        .finish_execution(cursor)
        .expect_err("a live pass omitted by stage routing must fail before submission");

    assert!(error.contains("did not execute compiled graph pass"));
    assert!(error.contains("missing"));
}

#[test]
fn execution_packet_batches_preserve_graph_order_and_queue_boundaries() {
    let mut builder = RenderGraphBuilder::new("execution-packet-batches");
    let graphics_a = builder.add_pass("graphics-a", QueueLane::Graphics);
    let graphics_b = builder.add_pass("graphics-b", QueueLane::Graphics);
    let compute = builder.add_pass("compute", QueueLane::AsyncCompute);
    let graphics_c = builder.add_pass("graphics-c", QueueLane::Graphics);
    for pass in [graphics_a, graphics_b, compute, graphics_c] {
        builder
            .set_pass_flags(
                pass,
                crate::render_graph::PassFlags {
                    has_side_effects: true,
                    ..crate::render_graph::PassFlags::default()
                },
            )
            .expect("batch fixture cull root");
    }
    builder
        .add_dependency(graphics_a, graphics_b)
        .expect("graphics dependency");
    builder
        .add_dependency(graphics_b, compute)
        .expect("compute dependency");
    builder
        .add_dependency(compute, graphics_c)
        .expect("final graphics dependency");
    let graph = builder.compile().expect("batch fixture graph");
    let packet = super::RenderGraphExecutionPacket::new(
        graph,
        [
            (graphics_a, super::RenderPassStage::DepthPrepass),
            (graphics_b, super::RenderPassStage::Shadow),
            (compute, super::RenderPassStage::PostProcess),
            (graphics_c, super::RenderPassStage::Present),
        ]
        .into_iter()
        .map(|(pass_id, stage)| super::RenderGraphExecutionPassMetadata::new(pass_id, stage))
        .collect(),
    )
    .expect("batch fixture packet");

    let batches = packet
        .execution_batches()
        .map(|batch| {
            (
                batch.queue(),
                packet
                    .passes_for_batch(batch)
                    .map(|pass| packet.graph().passes()[pass.graph_pass_index].name.as_str())
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        batches,
        vec![
            (QueueLane::Graphics, vec!["graphics-a", "graphics-b"]),
            (QueueLane::AsyncCompute, vec!["compute"]),
            (QueueLane::Graphics, vec!["graphics-c"]),
        ]
    );
    assert_eq!(
        packet.execution_batch_report(),
        crate::core::framework::render::RenderGraphExecutionBatchReport::new(3, 4, 2, 1, 0, 2, 2)
    );
    let depth_batches = packet
        .execution_batches_for_stage(super::RenderPassStage::DepthPrepass)
        .map(|batch| {
            packet
                .passes_for_batch(batch)
                .map(|pass| packet.graph().passes()[pass.graph_pass_index].name.as_str())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(depth_batches, vec![vec!["graphics-a", "graphics-b"]]);
    let depth_batch_indices = packet
        .execution_batches_with_indices_for_stage(super::RenderPassStage::DepthPrepass)
        .map(|(index, batch)| (index, batch.graph_pass_range().collect::<Vec<_>>()))
        .collect::<Vec<_>>();
    assert_eq!(depth_batch_indices, vec![(0, vec![0, 1])]);
    assert_eq!(
        packet.execution_stages_in_graph_order().collect::<Vec<_>>(),
        vec![
            super::RenderPassStage::DepthPrepass,
            super::RenderPassStage::Shadow,
            super::RenderPassStage::PostProcess,
            super::RenderPassStage::Present,
        ]
    );
}

#[test]
fn execution_packet_batches_split_around_culled_graph_passes() {
    let mut builder = RenderGraphBuilder::new("execution-packet-batches-culling");
    let first = builder.add_pass("first", QueueLane::Graphics);
    let culled = builder.add_pass("culled", QueueLane::Graphics);
    let last = builder.add_pass("last", QueueLane::Graphics);
    for pass in [first, last] {
        builder
            .set_pass_flags(
                pass,
                crate::render_graph::PassFlags {
                    has_side_effects: true,
                    ..crate::render_graph::PassFlags::default()
                },
            )
            .expect("live batch fixture cull root");
    }
    builder
        .add_dependency(first, last)
        .expect("live passes must retain graph order");
    let graph = builder.compile().expect("culling batch fixture graph");
    assert!(
        graph
            .passes()
            .iter()
            .find(|pass| pass.id == culled)
            .expect("culled pass identity")
            .culled
    );
    let packet = super::RenderGraphExecutionPacket::new(
        graph,
        [
            (first, super::RenderPassStage::DepthPrepass),
            (culled, super::RenderPassStage::Shadow),
            (last, super::RenderPassStage::Present),
        ]
        .into_iter()
        .map(|(pass_id, stage)| super::RenderGraphExecutionPassMetadata::new(pass_id, stage))
        .collect(),
    )
    .expect("culling batch fixture packet");

    let ranges = packet
        .execution_batches()
        .map(super::RenderGraphExecutionBatch::graph_pass_range)
        .collect::<Vec<_>>();
    assert_eq!(ranges.len(), 2);
    assert!(ranges.iter().all(|range| range.len() == 1));
    assert!(packet.execution_batches().all(|batch| {
        packet
            .passes_for_batch(batch)
            .all(|pass| !packet.graph().passes()[pass.graph_pass_index].culled)
    }));
    assert_eq!(
        packet.execution_batch_report(),
        crate::core::framework::render::RenderGraphExecutionBatchReport::new(2, 2, 2, 0, 0, 1, 0)
    );
    assert_eq!(packet.execution_batch_index_for_pass(0), Some(0));
    assert_eq!(packet.execution_batch_index_for_pass(1), None);
    assert_eq!(packet.execution_batch_index_for_pass(2), Some(1));
}

#[test]
fn compiled_pipeline_forwards_packet_stage_batch_index() {
    let source = include_str!("../compiled_render_pipeline.rs");
    assert!(source.contains("pub(in crate::graphics) fn execution_batches_for_stage("));
    assert!(source.contains("self.execution_packet.execution_batches_for_stage(stage)"));
    assert!(source.contains("execution_batches_with_indices_for_stage("));
    assert!(source.contains("pub(in crate::graphics) fn execution_stages_in_graph_order("));
    assert!(source.contains("self.execution_packet.execution_stages_in_graph_order()"));
    assert!(source.contains("execution_batch_index_for_pass("));
    assert!(source.contains("pub(in crate::graphics) const fn begin_execution("));
    assert!(source.contains(".admit_execution_pass(cursor, graph_pass_index)"));
    assert!(source.contains("self.execution_packet.finish_execution(cursor)"));
}

#[test]
fn render01_compiled_pipeline_runtime_metadata_freezes_descriptor_capability_flags() {
    let graph = RenderGraphBuilder::new("runtime-metadata-capability-flags")
        .compile()
        .unwrap();
    let metadata = CompiledRenderPipelineRuntimeMetadata::from_compiled_inputs(
        &[],
        &[RenderFeatureCapabilityRequirement::ScreenSpaceAntiAlias],
        &graph,
    );

    assert!(
        metadata
            .runtime_feature_flags()
            .screen_space_anti_alias_capability_enabled
    );
}

#[test]
#[ignore = "release performance gate"]
fn runtime89_compiled_pass_identity_lookup_beats_name_scan_p95() {
    let mut builder = RenderGraphBuilder::new("runtime89-pass-identity-lookup");
    let mut execution_pass_metadata = Vec::with_capacity(RUNTIME89_LOOKUP_PASS_COUNT);
    let mut final_pass = None;
    for index in 0..RUNTIME89_LOOKUP_PASS_COUNT {
        let pass_name = format!("runtime89-pass-{index:04}");
        let pass_id = builder.add_pass(pass_name.clone(), QueueLane::Graphics);
        execution_pass_metadata.push(super::RenderGraphExecutionPassMetadata::new(
            pass_id,
            super::RenderPassStage::PostProcess,
        ));
        final_pass = Some(pass_id);
    }
    builder
        .set_pass_flags(
            final_pass.expect("benchmark pass"),
            crate::render_graph::PassFlags {
                has_side_effects: true,
                ..crate::render_graph::PassFlags::default()
            },
        )
        .expect("benchmark cull root");
    let graph = builder.compile().expect("benchmark graph should compile");
    let packet =
        super::RenderGraphExecutionPacket::new(graph.clone(), execution_pass_metadata.clone())
            .expect("benchmark packet should preserve every compiled pass");

    let _ = legacy_name_lookup_sample(&graph, &execution_pass_metadata);
    let _ = direct_packet_lookup_sample(&packet);
    let mut legacy_samples = Vec::with_capacity(RUNTIME89_LOOKUP_SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(RUNTIME89_LOOKUP_SAMPLE_PAIRS);
    let mut legacy_comparisons = 0_usize;
    let mut optimized_lookups = 0_usize;
    for pair in 0..RUNTIME89_LOOKUP_SAMPLE_PAIRS {
        if pair % 2 == 0 {
            let (elapsed, work) = legacy_name_lookup_sample(&graph, &execution_pass_metadata);
            legacy_samples.push(elapsed);
            legacy_comparisons = work;
            let (elapsed, work) = direct_packet_lookup_sample(&packet);
            optimized_samples.push(elapsed);
            optimized_lookups = work;
        } else {
            let (elapsed, work) = direct_packet_lookup_sample(&packet);
            optimized_samples.push(elapsed);
            optimized_lookups = work;
            let (elapsed, work) = legacy_name_lookup_sample(&graph, &execution_pass_metadata);
            legacy_samples.push(elapsed);
            legacy_comparisons = work;
        }
    }

    let legacy_p95_ns = nearest_rank_duration(&legacy_samples, 95).as_nanos();
    let optimized_p95_ns = nearest_rank_duration(&optimized_samples, 95).as_nanos();
    let ratio_pct = optimized_p95_ns.saturating_mul(100) / legacy_p95_ns.max(1);
    println!(
        "RUNTIME89_COMPILED_PASS_IDENTITY_TIME workload=compiled_pass_identity_lookup pass_count={} sample_pairs={} alternation=legacy_first_even_pair legacy_first_pairs=11 optimized_first_pairs=10 legacy_p95_ns={} optimized_p95_ns={} legacy_name_comparisons={} optimized_id_lookups={} ratio_pct={} legacy_ns={} optimized_ns={}",
        RUNTIME89_LOOKUP_PASS_COUNT,
        RUNTIME89_LOOKUP_SAMPLE_PAIRS,
        legacy_p95_ns,
        optimized_p95_ns,
        legacy_comparisons,
        optimized_lookups,
        ratio_pct,
        join_duration_ns(&legacy_samples),
        join_duration_ns(&optimized_samples)
    );

    assert_eq!(
        legacy_comparisons,
        RUNTIME89_LOOKUP_PASS_COUNT * (RUNTIME89_LOOKUP_PASS_COUNT + 1) / 2
    );
    assert_eq!(optimized_lookups, RUNTIME89_LOOKUP_PASS_COUNT);
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(25),
        "direct pass identity lookup P95 must be at least 75% faster: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn legacy_name_lookup_sample(
    graph: &crate::render_graph::CompiledRenderGraph,
    execution_pass_metadata: &[super::RenderGraphExecutionPassMetadata],
) -> (Duration, usize) {
    let started = Instant::now();
    let mut comparisons = 0_usize;
    let mut checksum = 0_usize;
    for metadata in black_box(execution_pass_metadata) {
        let (_, target) = graph
            .indexed_pass(metadata.pass_id)
            .expect("compiled pass identity");
        for pass in black_box(graph.passes()) {
            comparisons += 1;
            if pass.name == target.name {
                checksum ^= pass.id.index();
                break;
            }
        }
    }
    black_box(checksum);
    (started.elapsed(), comparisons)
}

fn direct_packet_lookup_sample(packet: &super::RenderGraphExecutionPacket) -> (Duration, usize) {
    let started = Instant::now();
    let mut checksum = 0_usize;
    for execution_pass in black_box(packet.execution_passes_in_graph_order()) {
        let pass = &packet.graph().passes()[execution_pass.graph_pass_index];
        checksum ^= pass.id.index();
    }
    black_box(checksum);
    (started.elapsed(), packet.graph().passes().len())
}

fn nearest_rank_duration(samples: &[Duration], percentile: usize) -> Duration {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1).min(sorted.len().saturating_sub(1))]
}

fn join_duration_ns(samples: &[Duration]) -> String {
    samples
        .iter()
        .map(|sample| sample.as_nanos().to_string())
        .collect::<Vec<_>>()
        .join(",")
}

#[test]
fn render01_compiled_pipeline_runtime_metadata_builds_resource_write_index_once_for_scaled_graphs()
{
    for pass_count in [10_usize, 100, 500] {
        let mut builder = RenderGraphBuilder::new(format!("runtime-metadata-{pass_count}"));
        let mut written_resources = Vec::with_capacity(pass_count);
        for index in 0..pass_count {
            let resource_name = format!("runtime-metadata-output-{index}");
            let pass = builder.add_pass_with_executor(
                format!("runtime-metadata-pass-{index}"),
                QueueLane::Graphics,
                Some("runtime-metadata.executor"),
            );
            let resource = builder.import_present_external_resource(resource_name.clone());
            builder.write_external(pass, resource).unwrap();
            written_resources.push(resource_name);
        }
        let graph = builder.compile().unwrap();
        let metadata =
            CompiledRenderPipelineRuntimeMetadata::from_compiled_inputs(&[], &[], &graph);
        let equivalent =
            CompiledRenderPipelineRuntimeMetadata::from_compiled_inputs(&[], &[], &graph);
        let build_stats = metadata.build_stats();
        let storage_before = metadata.resource_write_storage_snapshot();

        assert_ne!(
            metadata.validation_generation(),
            equivalent.validation_generation()
        );
        assert_eq!(
            metadata, equivalent,
            "validation identity must not change public compiled-pipeline equality"
        );
        assert_eq!(build_stats.0, pass_count);
        assert_eq!(build_stats.1, pass_count);
        for _ in 0..128 {
            for resource_name in &written_resources {
                assert!(metadata.writes_resource(resource_name));
            }
            assert!(!metadata.writes_resource("runtime-metadata-missing"));
        }

        assert_eq!(metadata.build_stats(), build_stats);
        assert_eq!(metadata.resource_write_storage_snapshot(), storage_before);
    }
}

#[test]
fn render01_compiled_pipeline_runtime_metadata_lazily_shares_graph_dump_per_generation() {
    let mut builder = RenderGraphBuilder::new("runtime-metadata-graph-dump");
    let pass = builder.add_pass_with_executor(
        "runtime-metadata-graph-dump-pass",
        QueueLane::Graphics,
        Some("runtime-metadata.executor"),
    );
    let output = builder.import_present_external_resource("runtime-metadata-graph-dump-output");
    builder.write_external(pass, output).unwrap();
    let graph = builder.compile().unwrap();
    let metadata = CompiledRenderPipelineRuntimeMetadata::from_compiled_inputs(&[], &[], &graph);

    let first = metadata.graph_dump_text(&graph);
    let second = metadata.graph_dump_text(&graph);

    assert!(Arc::ptr_eq(&first, &second));
    assert!(first.contains("runtime-metadata-graph-dump-pass"));
}
