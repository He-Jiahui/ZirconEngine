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
fn render_perf_parallel_record_submission_order() {
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

#[test]
fn parallel_encoder_partition_filters_passes_without_losing_graph_layer_order() {
    let graph = compiled_graph_with_executable_chain(5);

    let encoder_set = ParallelEncoderSet::partition_filtered(&graph, 1, |_, pass| {
        matches!(pass.name.as_str(), "pass-1" | "pass-3")
    });

    assert_eq!(
        encoder_set
            .buckets()
            .iter()
            .map(|bucket| bucket.topology_layer_range())
            .collect::<Vec<_>>(),
        vec![1..=1, 3..=3]
    );
    assert_eq!(
        encoder_set
            .buckets()
            .iter()
            .flat_map(|bucket| bucket.pass_indices())
            .map(|pass_index| graph.passes()[*pass_index].name.as_str())
            .collect::<Vec<_>>(),
        vec!["pass-1", "pass-3"]
    );
}

#[test]
fn render_perf_parallel_encoder_outputs_return_in_topology_order() {
    let graph = compiled_graph_with_executable_chain(6);
    let encoder_set = ParallelEncoderSet::partition(&graph, 2);
    let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(2));
    let backend = crate::graphics::backend::RenderBackend::new_offscreen()
        .expect("parallel encoder output test backend");

    let recorded = encoder_set
        .record_parallel_with_outputs(&backend.device, &pool, |bucket, encoder| {
            encoder.insert_debug_marker("parallel-output-order");
            Ok::<_, ()>(vec![bucket.topology_order()])
        })
        .expect("parallel bucket recording");
    let (command_buffers, outputs): (Vec<_>, Vec<_>) = recorded
        .into_iter()
        .map(|bucket| bucket.into_parts())
        .unzip();

    assert_eq!(outputs, vec![vec![0], vec![1], vec![2]]);
    backend.queue.submit(command_buffers);
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
