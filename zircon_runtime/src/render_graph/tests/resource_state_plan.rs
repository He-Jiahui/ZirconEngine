use crate::render_graph::{
    PassFlags, QueueLane, RenderGraphBufferRange, RenderGraphBuilder,
    RenderGraphExternalResourceBinding, RenderGraphResource, RenderGraphResourceAccessIntent,
    RenderGraphResourceAccessKind, RenderGraphResourceAccessRange, RenderGraphResourceState,
    RenderGraphResourceUsageFlags, RenderGraphShaderStages, RenderGraphTextureAspect,
    RenderGraphTextureSubresourceRange,
};
use crate::rhi::{
    BufferDesc, BufferUsage, TextureDesc, TextureDimension, TextureFormat, TextureUsage,
};

fn texture_cell(
    mip_level: u32,
    array_layer: u32,
    aspect: RenderGraphTextureAspect,
) -> RenderGraphResourceAccessRange {
    RenderGraphResourceAccessRange::Texture(RenderGraphTextureSubresourceRange {
        base_mip_level: mip_level,
        mip_level_count: Some(1),
        base_array_layer: array_layer,
        array_layer_count: Some(1),
        aspect,
    })
}

fn texture_layers(
    mip_level: u32,
    base_array_layer: u32,
    array_layer_count: u32,
    aspect: RenderGraphTextureAspect,
) -> RenderGraphResourceAccessRange {
    RenderGraphResourceAccessRange::Texture(RenderGraphTextureSubresourceRange {
        base_mip_level: mip_level,
        mip_level_count: Some(1),
        base_array_layer,
        array_layer_count: Some(array_layer_count),
        aspect,
    })
}

#[test]
fn persistent_middle_layer_alias_keeps_enclosing_parent_writer() {
    assert_middle_layer_alias_culling(false);
}

#[test]
fn persistent_middle_layer_alias_culls_unrelated_parent_layer_writer() {
    assert_middle_layer_alias_culling(true);
}

fn assert_middle_layer_alias_culling(add_unrelated_writer: bool) {
    let mut builder = RenderGraphBuilder::new("middle-layer-alias-culling");
    let parent = builder.create_texture(
        TextureDesc::new(
            "parent",
            4,
            4,
            TextureFormat::Rgba8Unorm,
            TextureUsage::STORAGE,
        )
        .with_dimension(TextureDimension::D2Array)
        .with_array_layers(4),
    );
    let alias = builder
        .create_texture_view_alias(
            "middle-layer",
            parent,
            RenderGraphTextureSubresourceRange {
                base_array_layer: 1,
                array_layer_count: Some(1),
                ..RenderGraphTextureSubresourceRange::full()
            },
        )
        .unwrap();
    builder.mark_persistent(alias).unwrap();
    let parent_writer = builder.add_pass("write-parent", QueueLane::AsyncCompute);
    let intent =
        RenderGraphResourceAccessIntent::storage_texture_write(RenderGraphShaderStages::COMPUTE);
    builder
        .access_texture(
            parent_writer,
            parent,
            RenderGraphResourceAccessKind::Write,
            RenderGraphTextureSubresourceRange::full(),
            intent,
            None,
        )
        .unwrap();
    let unrelated_writer = add_unrelated_writer.then(|| {
        let pass = builder.add_pass("write-unrelated-layer", QueueLane::AsyncCompute);
        builder
            .access_texture(
                pass,
                parent,
                RenderGraphResourceAccessKind::Write,
                RenderGraphTextureSubresourceRange {
                    base_array_layer: 3,
                    array_layer_count: Some(1),
                    ..RenderGraphTextureSubresourceRange::full()
                },
                intent,
                None,
            )
            .unwrap();
        pass
    });
    let graph = builder.compile().unwrap();
    assert!(
        !graph
            .passes()
            .iter()
            .find(|pass| pass.id == parent_writer)
            .unwrap()
            .culled
    );
    if let Some(unrelated_writer) = unrelated_writer {
        assert!(
            graph
                .passes()
                .iter()
                .find(|pass| pass.id == unrelated_writer)
                .unwrap()
                .culled
        );
    }
}

#[test]
fn compiled_state_plan_tracks_exact_texture_state_transitions() {
    let mut builder = RenderGraphBuilder::new("texture-state-plan");
    let texture = builder.create_texture(TextureDesc::new(
        "scene-color",
        32,
        32,
        TextureFormat::Rgba16Float,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let draw = builder.add_pass("draw", QueueLane::Graphics);
    let sample = builder.add_pass("sample", QueueLane::Graphics);
    let range = RenderGraphTextureSubresourceRange::full();

    builder
        .access_texture(
            draw,
            texture,
            RenderGraphResourceAccessKind::Write,
            range,
            RenderGraphResourceAccessIntent::ColorAttachment,
            None,
        )
        .unwrap();
    builder
        .read_texture_with_access(
            sample,
            texture,
            range,
            RenderGraphResourceAccessIntent::sampled_texture(RenderGraphShaderStages::FRAGMENT),
        )
        .unwrap();
    builder
        .set_pass_flags(
            sample,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .unwrap();

    let graph = builder.compile().unwrap();
    let plan = graph.resource_state_plan();

    assert_eq!(plan.transitions().len(), 1);
    let transition = plan.transitions()[0];
    assert_eq!(
        transition.resource,
        RenderGraphResource::TransientTexture(texture)
    );
    assert_eq!(
        transition.from_state,
        RenderGraphResourceState::ColorAttachment
    );
    assert_eq!(
        transition.to_state,
        RenderGraphResourceState::SampledTexture
    );
    assert!(!transition.crosses_queue());
    assert_eq!(plan.queue_transition_count(), 0);
    assert_eq!(plan.legacy_access_count(), 0);
}

#[test]
fn compiled_state_plan_matches_overlapping_subresources_without_crossing_disjoint_mips() {
    let mut builder = RenderGraphBuilder::new("subresource-state-plan");
    let texture = builder.create_texture(
        TextureDesc::new(
            "history-pyramid",
            32,
            32,
            TextureFormat::Rgba16Float,
            TextureUsage::STORAGE | TextureUsage::SAMPLED,
        )
        .with_mip_levels(3),
    );
    let write_full = builder.add_pass("write-full", QueueLane::AsyncCompute);
    let read_mip_one = builder.add_pass("read-mip-one", QueueLane::Graphics);
    let write_mip_two = builder.add_pass("write-mip-two", QueueLane::Graphics);

    builder
        .access_texture(
            write_full,
            texture,
            RenderGraphResourceAccessKind::Write,
            RenderGraphTextureSubresourceRange::full(),
            RenderGraphResourceAccessIntent::storage_texture_write(
                RenderGraphShaderStages::COMPUTE,
            ),
            None,
        )
        .unwrap();
    builder
        .read_texture_with_access(
            read_mip_one,
            texture,
            RenderGraphTextureSubresourceRange::single_mip(1),
            RenderGraphResourceAccessIntent::sampled_texture(RenderGraphShaderStages::FRAGMENT),
        )
        .unwrap();
    builder
        .access_texture(
            write_mip_two,
            texture,
            RenderGraphResourceAccessKind::Write,
            RenderGraphTextureSubresourceRange::single_mip(2),
            RenderGraphResourceAccessIntent::storage_texture_write(
                RenderGraphShaderStages::COMPUTE,
            ),
            None,
        )
        .unwrap();
    builder
        .set_pass_flags(
            read_mip_one,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .unwrap();
    builder
        .set_pass_flags(
            write_mip_two,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .unwrap();

    let graph = builder.compile().unwrap();
    let transitions = graph.resource_state_plan().transitions();
    assert_eq!(transitions.len(), 2);
    assert_eq!(
        transitions[0].to_state,
        RenderGraphResourceState::SampledTexture
    );
    assert_eq!(
        transitions[1].to_state,
        RenderGraphResourceState::StorageTextureWrite
    );
    assert_eq!(
        transitions[0].range,
        texture_cell(1, 0, RenderGraphTextureAspect::Color)
    );
    assert_eq!(
        transitions[1].range,
        texture_cell(2, 0, RenderGraphTextureAspect::Color)
    );
}

#[test]
fn compiled_state_plan_emits_every_exact_partial_texture_predecessor() {
    let mut builder = RenderGraphBuilder::new("partial-texture-predecessors");
    let texture = builder.create_texture(
        TextureDesc::new(
            "layered-history",
            32,
            32,
            TextureFormat::Rgba16Float,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::STORAGE | TextureUsage::SAMPLED,
        )
        .with_dimension(TextureDimension::D2Array)
        .with_array_layers(2)
        .with_mip_levels(2),
    );
    let draw = builder.add_pass("draw-mip0", QueueLane::Graphics);
    let filter = builder.add_pass("filter-mip1", QueueLane::AsyncCompute);
    let sample = builder.add_pass("sample-both", QueueLane::Graphics);
    let mip_zero = RenderGraphTextureSubresourceRange::single_mip(0).with_array_layers(0, 2);
    let mip_one = RenderGraphTextureSubresourceRange::single_mip(1).with_array_layers(0, 2);

    builder
        .access_texture(
            draw,
            texture,
            RenderGraphResourceAccessKind::Write,
            mip_zero,
            RenderGraphResourceAccessIntent::ColorAttachment,
            None,
        )
        .unwrap();
    builder
        .access_texture(
            filter,
            texture,
            RenderGraphResourceAccessKind::Write,
            mip_one,
            RenderGraphResourceAccessIntent::storage_texture_write(
                RenderGraphShaderStages::COMPUTE,
            ),
            None,
        )
        .unwrap();
    builder
        .read_texture_with_access(
            sample,
            texture,
            RenderGraphTextureSubresourceRange::full(),
            RenderGraphResourceAccessIntent::sampled_texture(RenderGraphShaderStages::FRAGMENT),
        )
        .unwrap();
    builder
        .set_pass_flags(
            sample,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .unwrap();

    let graph = builder.compile().unwrap();
    let transitions = graph.resource_state_plan().transitions();
    assert_eq!(transitions.len(), 2);
    assert_eq!(
        transitions[0].range,
        texture_layers(0, 0, 2, RenderGraphTextureAspect::Color)
    );
    assert_eq!(
        transitions[1].range,
        texture_layers(1, 0, 2, RenderGraphTextureAspect::Color)
    );
    assert_eq!(
        transitions[0].from_state,
        RenderGraphResourceState::ColorAttachment
    );
    assert_eq!(
        transitions[1].from_state,
        RenderGraphResourceState::StorageTextureWrite
    );
    assert_eq!(transitions[1].from_queue, QueueLane::AsyncCompute);
}

#[test]
fn compiled_state_plan_preserves_both_depth_stencil_predecessors_for_all_aspects() {
    let mut builder = RenderGraphBuilder::new("depth-stencil-state-plan");
    let texture = builder.create_texture(TextureDesc::new(
        "depth-stencil",
        32,
        32,
        TextureFormat::Depth24PlusStencil8,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let depth = builder.add_pass("write-depth", QueueLane::Graphics);
    let stencil = builder.add_pass("write-stencil", QueueLane::Graphics);
    let sample = builder.add_pass("sample-both", QueueLane::Graphics);

    for (pass, aspect) in [
        (depth, RenderGraphTextureAspect::Depth),
        (stencil, RenderGraphTextureAspect::Stencil),
    ] {
        builder
            .access_texture(
                pass,
                texture,
                RenderGraphResourceAccessKind::Write,
                RenderGraphTextureSubresourceRange::full().with_aspect(aspect),
                RenderGraphResourceAccessIntent::DepthStencilAttachment,
                None,
            )
            .unwrap();
    }
    builder
        .read_texture_with_access(
            sample,
            texture,
            RenderGraphTextureSubresourceRange::full(),
            RenderGraphResourceAccessIntent::sampled_texture(RenderGraphShaderStages::FRAGMENT),
        )
        .unwrap();
    builder
        .set_pass_flags(
            sample,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .unwrap();

    let graph = builder.compile().unwrap();
    let sample_access = graph.access_id_at(sample, 0).unwrap();
    let transitions = graph.resource_state_plan().transitions();
    assert_eq!(transitions.len(), 2);
    for (pass, aspect) in [
        (depth, RenderGraphTextureAspect::Depth),
        (stencil, RenderGraphTextureAspect::Stencil),
    ] {
        let expected_range = texture_cell(0, 0, aspect);
        let transition = transitions
            .iter()
            .find(|transition| transition.range == expected_range)
            .expect("each plane has its own exact predecessor");
        assert_eq!(transition.from_access, graph.access_id_at(pass, 0).unwrap());
        assert_eq!(transition.to_access, sample_access);
        assert_eq!(
            transition.from_state,
            RenderGraphResourceState::DepthStencilAttachment
        );
        assert_eq!(
            transition.to_state,
            RenderGraphResourceState::SampledTexture
        );
    }
    let dependencies = &graph
        .passes()
        .iter()
        .find(|pass| pass.id == sample)
        .unwrap()
        .dependencies;
    assert_eq!(dependencies.len(), 2);
    assert!(dependencies.contains(&depth));
    assert!(dependencies.contains(&stencil));
}

#[test]
fn compiled_state_plan_emits_exact_adjacent_buffer_predecessors() {
    let mut builder = RenderGraphBuilder::new("partial-buffer-predecessors");
    let buffer = builder.create_buffer(BufferDesc::new(
        "staging",
        256,
        BufferUsage::STORAGE | BufferUsage::COPY_SRC | BufferUsage::COPY_DST,
    ));
    let first = builder.add_pass("write-low", QueueLane::AsyncCompute);
    let second = builder.add_pass("copy-high", QueueLane::AsyncCopy);
    let consume = builder.add_pass("consume-full", QueueLane::Graphics);

    builder
        .access_buffer(
            first,
            buffer,
            RenderGraphResourceAccessKind::Write,
            RenderGraphBufferRange::new(0, Some(128)),
            RenderGraphResourceAccessIntent::storage_buffer_read_write(
                RenderGraphShaderStages::COMPUTE,
            ),
        )
        .unwrap();
    builder
        .access_buffer(
            second,
            buffer,
            RenderGraphResourceAccessKind::Write,
            RenderGraphBufferRange::new(128, Some(128)),
            RenderGraphResourceAccessIntent::CopyDestination,
        )
        .unwrap();
    builder
        .access_buffer(
            consume,
            buffer,
            RenderGraphResourceAccessKind::Read,
            RenderGraphBufferRange::full(),
            RenderGraphResourceAccessIntent::CopySource,
        )
        .unwrap();
    builder
        .set_pass_flags(
            consume,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .unwrap();

    let graph = builder.compile().unwrap();
    let transitions = graph.resource_state_plan().transitions();
    assert_eq!(transitions.len(), 2);
    assert_eq!(
        transitions[0].range,
        RenderGraphResourceAccessRange::Buffer(RenderGraphBufferRange::new(0, Some(128)))
    );
    assert_eq!(
        transitions[1].range,
        RenderGraphResourceAccessRange::Buffer(RenderGraphBufferRange::new(128, Some(128)))
    );
    assert_eq!(
        transitions[0].from_state,
        RenderGraphResourceState::StorageBufferReadWrite
    );
    assert_eq!(
        transitions[1].from_state,
        RenderGraphResourceState::CopyDestination
    );
}

#[test]
fn compiled_state_plan_coalesces_external_alias_identity() {
    let mut builder = RenderGraphBuilder::new("external-alias-state-plan");
    let storage = builder.import_external_resource_with_usage_binding_and_alias_group(
        "environment.storage",
        RenderGraphResourceUsageFlags::persistent(),
        RenderGraphExternalResourceBinding::required_texture(),
        "environment-cube",
    );
    let sampled = builder.import_external_resource_with_usage_binding_and_alias_group(
        "environment.sampled",
        RenderGraphResourceUsageFlags::persistent(),
        RenderGraphExternalResourceBinding::required_texture(),
        "environment-cube",
    );
    let produce = builder.add_pass("produce", QueueLane::AsyncCompute);
    let consume = builder.add_pass("consume", QueueLane::Graphics);
    builder.write_storage_external(produce, storage).unwrap();
    builder.read_external(consume, sampled).unwrap();

    let graph = builder.compile().unwrap();
    let transitions = graph.resource_state_plan().transitions();
    assert_eq!(transitions.len(), 1);
    assert_eq!(
        transitions[0].resource,
        RenderGraphResource::External(storage)
    );
    assert!(transitions[0].crosses_queue());
}

#[test]
fn compiled_state_plan_disjoint_buffer_work_is_bounded_at_ten_thousand_ranges() {
    const RANGE_COUNT: usize = 10_000;
    const RANGE_SIZE: u64 = 16;
    let mut builder = RenderGraphBuilder::new("bounded-disjoint-buffer-state-plan");
    let buffer = builder.create_buffer(BufferDesc::new(
        "disjoint",
        RANGE_COUNT as u64 * RANGE_SIZE,
        BufferUsage::STORAGE,
    ));
    for index in 0..RANGE_COUNT {
        let pass = builder.add_pass(format!("write-{index}"), QueueLane::AsyncCompute);
        builder
            .access_buffer(
                pass,
                buffer,
                RenderGraphResourceAccessKind::Write,
                RenderGraphBufferRange::new(index as u64 * RANGE_SIZE, Some(RANGE_SIZE)),
                RenderGraphResourceAccessIntent::storage_buffer_read_write(
                    RenderGraphShaderStages::COMPUTE,
                ),
            )
            .unwrap();
        builder
            .set_pass_flags(
                pass,
                PassFlags {
                    has_side_effects: true,
                    ..PassFlags::default()
                },
            )
            .unwrap();
    }
    let graph = builder.compile().unwrap();
    assert!(graph.resource_state_plan().transitions().is_empty());
    let plan = graph.resource_state_plan();
    assert_eq!(plan.tracking_lookup_visits(), RANGE_COUNT * 4);
    assert_eq!(plan.tracking_split_visits(), RANGE_COUNT - 1);
    assert_eq!(plan.tracking_read_visits(), RANGE_COUNT);
    assert_eq!(plan.tracking_update_visits(), RANGE_COUNT);
    assert_eq!(plan.tracking_merge_visits(), RANGE_COUNT * 2 - 2);
    assert_eq!(plan.tracking_work_count(), RANGE_COUNT * 9 - 3);
}

#[test]
#[ignore = "release performance evidence"]
fn compiled_state_plan_disjoint_buffer_release_workload_evidence() {
    use std::hint::black_box;
    use std::time::Instant;

    assert!(
        !cfg!(debug_assertions),
        "run release evidence with --release"
    );
    const RANGE_COUNT: usize = 10_000;
    const RANGE_SIZE: u64 = 16;
    const SAMPLE_COUNT: usize = 21;
    let mut builder = RenderGraphBuilder::new("release-disjoint-buffer-state-plan");
    let buffer = builder.create_buffer(BufferDesc::new(
        "disjoint-release",
        RANGE_COUNT as u64 * RANGE_SIZE,
        BufferUsage::STORAGE,
    ));
    for index in 0..RANGE_COUNT {
        let pass = builder.add_pass(format!("write-{index}"), QueueLane::AsyncCompute);
        builder
            .access_buffer(
                pass,
                buffer,
                RenderGraphResourceAccessKind::Write,
                RenderGraphBufferRange::new(index as u64 * RANGE_SIZE, Some(RANGE_SIZE)),
                RenderGraphResourceAccessIntent::storage_buffer_read_write(
                    RenderGraphShaderStages::COMPUTE,
                ),
            )
            .unwrap();
        builder
            .set_pass_flags(
                pass,
                PassFlags {
                    has_side_effects: true,
                    ..PassFlags::default()
                },
            )
            .unwrap();
    }
    let consumer = builder.add_pass("read-all-ranges", QueueLane::Graphics);
    builder
        .access_buffer(
            consumer,
            buffer,
            RenderGraphResourceAccessKind::Read,
            RenderGraphBufferRange::full(),
            RenderGraphResourceAccessIntent::storage_buffer_read(RenderGraphShaderStages::FRAGMENT),
        )
        .unwrap();
    builder
        .set_pass_flags(
            consumer,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .unwrap();

    let run = |legacy: bool| {
        let input = builder.clone();
        let started = Instant::now();
        let graph = if legacy {
            input.compile_with_whole_map_coalescing_for_test()
        } else {
            input.compile()
        }
        .unwrap();
        black_box(&graph);
        (started.elapsed().as_nanos(), graph)
    };
    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..=SAMPLE_COUNT {
        let (legacy, optimized) = if sample % 2 == 0 {
            (run(true), run(false))
        } else {
            let optimized = run(false);
            (run(true), optimized)
        };
        let (legacy_ns, legacy_graph) = legacy;
        let (optimized_ns, optimized_graph) = optimized;
        assert_eq!(legacy_graph.passes(), optimized_graph.passes());
        assert_eq!(
            legacy_graph.resource_lifetimes(),
            optimized_graph.resource_lifetimes()
        );
        assert_eq!(
            legacy_graph.resource_state_plan().transitions(),
            optimized_graph.resource_state_plan().transitions()
        );
        assert_eq!(
            optimized_graph.resource_state_plan().transitions().len(),
            RANGE_COUNT
        );
        for pass in optimized_graph.passes() {
            for index in 0..pass.resources.len() {
                let access = optimized_graph.access_id_at(pass.id, index).unwrap();
                assert_eq!(
                    legacy_graph.access_metadata(access),
                    optimized_graph.access_metadata(access)
                );
                assert_eq!(
                    legacy_graph.resource_version_for_id(access),
                    optimized_graph.resource_version_for_id(access)
                );
                assert_eq!(
                    legacy_graph.input_version_for_id(access),
                    optimized_graph.input_version_for_id(access)
                );
            }
        }
        if sample > 0 {
            legacy_samples.push(legacy_ns);
            optimized_samples.push(optimized_ns);
        }
    }
    println!("RG_A4_STATE_PLAN_RELEASE_V4 ranges={RANGE_COUNT} sample_pairs={SAMPLE_COUNT} legacy_samples_ns={legacy_samples:?} optimized_samples_ns={optimized_samples:?}");
    legacy_samples.sort_unstable();
    optimized_samples.sort_unstable();
    let percentile =
        |samples: &[u128], rank: usize| samples[(samples.len() * rank).div_ceil(100) - 1];
    for rank in [50, 95, 99] {
        println!(
            "RG_A4_STATE_PLAN_RELEASE_V4 percentile={rank} legacy_ns={} optimized_ns={}",
            percentile(&legacy_samples, rank),
            percentile(&optimized_samples, rank)
        );
    }
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p95 = percentile(&optimized_samples, 95);
    assert!(
        optimized_p95 * 100 <= legacy_p95 * 80,
        "expected >=20% p95 compile improvement: legacy={legacy_p95} optimized={optimized_p95}"
    );
}

#[test]
fn compact_large_texture_mixed_ranges_match_whole_map_and_exact_predecessors() {
    const LAYERS: u32 = u32::MAX;
    const MIDDLE: u32 = LAYERS / 2;
    let mut builder = RenderGraphBuilder::new("large-texture-mixed-ranges");
    let texture = builder.create_texture(
        TextureDesc::new(
            "large-array",
            4,
            4,
            TextureFormat::Rgba8Unorm,
            TextureUsage::STORAGE | TextureUsage::SAMPLED,
        )
        .with_dimension(TextureDimension::D2Array)
        .with_array_layers(LAYERS),
    );
    let writes =
        RenderGraphResourceAccessIntent::storage_texture_write(RenderGraphShaderStages::COMPUTE);
    let reads = RenderGraphResourceAccessIntent::sampled_texture(RenderGraphShaderStages::FRAGMENT);
    let mut passes = Vec::new();
    for (name, start, count, kind, intent, queue) in [
        (
            "initial",
            0,
            LAYERS,
            RenderGraphResourceAccessKind::Write,
            writes,
            QueueLane::AsyncCompute,
        ),
        (
            "read-middle-left",
            1,
            MIDDLE - 1,
            RenderGraphResourceAccessKind::Read,
            reads,
            QueueLane::Graphics,
        ),
        (
            "write-adjacent-right",
            MIDDLE,
            LAYERS - MIDDLE - 1,
            RenderGraphResourceAccessKind::Write,
            writes,
            QueueLane::AsyncCompute,
        ),
        (
            "write-overlap",
            MIDDLE - 1,
            2,
            RenderGraphResourceAccessKind::Write,
            writes,
            QueueLane::AsyncCompute,
        ),
        (
            "read-all",
            0,
            LAYERS,
            RenderGraphResourceAccessKind::Read,
            reads,
            QueueLane::Graphics,
        ),
    ] {
        let pass = builder.add_pass(name, queue);
        builder
            .access_texture(
                pass,
                texture,
                kind,
                RenderGraphTextureSubresourceRange::single_mip(0).with_array_layers(start, count),
                intent,
                None,
            )
            .unwrap();
        builder
            .set_pass_flags(
                pass,
                PassFlags {
                    has_side_effects: true,
                    ..PassFlags::default()
                },
            )
            .unwrap();
        passes.push(pass);
    }
    let legacy = builder
        .clone()
        .compile_with_whole_map_coalescing_for_test()
        .unwrap();
    let optimized = builder.compile().unwrap();
    assert_eq!(legacy.passes(), optimized.passes());
    assert_eq!(legacy.resource_lifetimes(), optimized.resource_lifetimes());
    assert_eq!(
        legacy.resource_state_plan().transitions(),
        optimized.resource_state_plan().transitions()
    );
    for pass in &passes {
        let access = optimized.access_id_at(*pass, 0).unwrap();
        assert_eq!(
            legacy.access_metadata(access),
            optimized.access_metadata(access)
        );
        assert_eq!(
            legacy.resource_version_for_id(access),
            optimized.resource_version_for_id(access)
        );
        assert_eq!(
            legacy.input_version_for_id(access),
            optimized.input_version_for_id(access)
        );
    }
    let final_access = optimized.access_id_at(passes[4], 0).unwrap();
    let actual: Vec<_> = optimized
        .resource_state_plan()
        .transitions()
        .iter()
        .filter(|transition| transition.to_access == final_access)
        .map(|transition| (transition.from_access, transition.range))
        .collect();
    let expected: Vec<_> = [
        (passes[0], 0, 1),
        (passes[3], MIDDLE - 1, 2),
        (passes[2], MIDDLE + 1, LAYERS - MIDDLE - 2),
        (passes[0], LAYERS - 1, 1),
    ]
    .into_iter()
    .map(|(pass, start, count)| {
        (
            optimized.access_id_at(pass, 0).unwrap(),
            texture_layers(0, start, count, RenderGraphTextureAspect::Color),
        )
    })
    .collect();
    assert_eq!(actual, expected);
    let mut dependencies = optimized
        .passes()
        .iter()
        .find(|pass| pass.id == passes[4])
        .unwrap()
        .dependencies
        .clone();
    dependencies.sort_by_key(|pass| pass.index());
    assert_eq!(dependencies, vec![passes[0], passes[2], passes[3]]);
}

#[test]
fn compiled_state_plan_tracks_a_large_texture_array_with_cardinality_independent_work() {
    const ARRAY_LAYERS: u32 = u32::MAX;
    let mut builder = RenderGraphBuilder::new("bounded-large-texture-array-state-plan");
    let texture = builder.create_texture(
        TextureDesc::new(
            "max-array",
            4,
            4,
            TextureFormat::Rgba8Unorm,
            TextureUsage::STORAGE,
        )
        .with_dimension(TextureDimension::D2Array)
        .with_array_layers(ARRAY_LAYERS),
    );
    let pass = builder.add_pass("write-array", QueueLane::AsyncCompute);
    builder
        .access_texture(
            pass,
            texture,
            RenderGraphResourceAccessKind::Write,
            RenderGraphTextureSubresourceRange::full(),
            RenderGraphResourceAccessIntent::storage_texture_write(
                RenderGraphShaderStages::COMPUTE,
            ),
            None,
        )
        .unwrap();
    builder
        .set_pass_flags(
            pass,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .unwrap();

    let graph = builder.compile().unwrap();
    let plan = graph.resource_state_plan();
    assert_eq!(plan.tracking_lookup_visits(), 4);
    assert_eq!(plan.tracking_split_visits(), 0);
    assert_eq!(plan.tracking_read_visits(), 1);
    assert_eq!(plan.tracking_update_visits(), 1);
    assert_eq!(plan.tracking_merge_visits(), 0);
    assert_eq!(plan.tracking_plane_visits(), 2);
    assert_eq!(plan.tracking_work_count(), 8);
}

#[test]
fn compiled_state_plan_counts_texture_planes_independent_of_array_layer_count() {
    let mut tracking_work = Vec::new();

    for array_layers in [1, u32::MAX] {
        let mut builder = RenderGraphBuilder::new("bounded-depth-stencil-plane-work");
        let texture = builder.create_texture(
            TextureDesc::new(
                "depth-stencil-array",
                4,
                4,
                TextureFormat::Depth24PlusStencil8,
                TextureUsage::RENDER_ATTACHMENT,
            )
            .with_dimension(TextureDimension::D2Array)
            .with_array_layers(array_layers)
            .with_mip_levels(2),
        );
        let pass = builder.add_pass("write-all-planes", QueueLane::Graphics);
        builder
            .access_texture(
                pass,
                texture,
                RenderGraphResourceAccessKind::Write,
                RenderGraphTextureSubresourceRange::full(),
                RenderGraphResourceAccessIntent::DepthStencilAttachment,
                None,
            )
            .unwrap();
        builder
            .set_pass_flags(
                pass,
                PassFlags {
                    has_side_effects: true,
                    ..PassFlags::default()
                },
            )
            .unwrap();

        let graph = builder.compile().unwrap();
        let plan = graph.resource_state_plan();
        assert_eq!(plan.tracking_plane_visits(), 8);
        assert_eq!(plan.tracking_lookup_visits(), 16);
        assert_eq!(plan.tracking_split_visits(), 0);
        assert_eq!(plan.tracking_read_visits(), 4);
        assert_eq!(plan.tracking_update_visits(), 4);
        assert_eq!(plan.tracking_merge_visits(), 0);
        assert_eq!(plan.tracking_work_count(), 32);
        tracking_work.push(plan.tracking_work_count());
    }

    assert_eq!(tracking_work, [32, 32]);
}

#[test]
fn state_plan_prepares_texture_scopes_as_compact_ranges() {
    let source = include_str!("../builder/access_scope_tracker.rs");

    assert!(source.contains("Texture(TextureScope)"));
    assert!(source.contains("PreparedScopeKind::Texture(TextureScope::new(desc, range))"));
    assert!(source.contains("Texture(HashMap<TexturePlane, BufferScopeHistory>)"));
    assert!(!source.contains("Texture(Vec<TextureCell>)"));
    assert!(!source.contains("fn texture_cells("));
}

#[test]
fn compiled_state_plan_reports_queue_transition_and_legacy_fallback() {
    let mut builder = RenderGraphBuilder::new("queue-state-plan");
    let buffer = builder.create_buffer(BufferDesc::new(
        "visible-items",
        256,
        BufferUsage::STORAGE | BufferUsage::INDIRECT,
    ));
    let prepare = builder.add_pass("prepare", QueueLane::AsyncCompute);
    let draw = builder.add_pass("draw", QueueLane::Graphics);

    builder
        .access_buffer(
            prepare,
            buffer,
            RenderGraphResourceAccessKind::Write,
            RenderGraphBufferRange::full(),
            RenderGraphResourceAccessIntent::storage_buffer_read_write(
                RenderGraphShaderStages::COMPUTE,
            ),
        )
        .unwrap();
    builder
        .access_buffer(
            draw,
            buffer,
            RenderGraphResourceAccessKind::Read,
            RenderGraphBufferRange::full(),
            RenderGraphResourceAccessIntent::Indirect,
        )
        .unwrap();
    builder
        .set_pass_flags(
            draw,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .unwrap();

    let graph = builder.compile().unwrap();
    let plan = graph.resource_state_plan();
    assert_eq!(plan.transitions().len(), 1);
    assert!(plan.transitions()[0].crosses_queue());
    assert_eq!(plan.queue_transition_count(), 1);
    assert_eq!(plan.legacy_access_count(), 0);

    let mut legacy = RenderGraphBuilder::new("legacy-state-plan");
    let texture = legacy.create_texture(TextureDesc::new(
        "legacy-output",
        4,
        4,
        TextureFormat::Rgba8Unorm,
        TextureUsage::RENDER_ATTACHMENT,
    ));
    let pass = legacy.add_pass("legacy-write", QueueLane::Graphics);
    legacy.write_texture(pass, texture).unwrap();
    legacy
        .set_pass_flags(
            pass,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .unwrap();

    assert_eq!(
        legacy
            .compile()
            .unwrap()
            .resource_state_plan()
            .legacy_access_count(),
        1
    );
}
