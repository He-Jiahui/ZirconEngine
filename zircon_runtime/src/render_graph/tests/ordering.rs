use crate::render_graph::{PassFlags, QueueLane, RenderGraphBuilder};
use crate::rhi::{TextureDesc, TextureFormat, TextureUsage};

#[test]
fn compile_orders_passes_by_declared_dependencies() {
    let mut builder = RenderGraphBuilder::new("frame");
    let depth = builder.add_pass("depth-prepass", QueueLane::Graphics);
    let shadow = builder.add_pass("shadow", QueueLane::Graphics);
    let lighting = builder.add_pass("lighting", QueueLane::Graphics);
    builder.add_dependency(depth, lighting).unwrap();
    builder.add_dependency(shadow, lighting).unwrap();
    builder
        .set_pass_flags(
            lighting,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .unwrap();

    let graph = builder.compile().unwrap();
    let ordered = graph
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(ordered, vec!["depth-prepass", "shadow", "lighting"]);
}

#[test]
fn compile_preserves_declared_dependencies_on_compiled_passes() {
    let mut builder = RenderGraphBuilder::new("frame");
    let depth = builder.add_pass("depth-prepass", QueueLane::Graphics);
    let shadow = builder.add_pass("shadow", QueueLane::Graphics);
    let lighting = builder.add_pass("lighting", QueueLane::Graphics);
    builder.add_dependency(depth, lighting).unwrap();
    builder.add_dependency(shadow, lighting).unwrap();
    builder
        .set_pass_flags(
            lighting,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .unwrap();

    let graph = builder.compile().unwrap();
    let lighting_pass = graph
        .passes()
        .iter()
        .find(|pass| pass.name == "lighting")
        .unwrap();

    assert_eq!(lighting_pass.dependencies, vec![depth, shadow]);
}

#[test]
fn compile_exposes_inferred_resource_dependencies_on_compiled_passes() {
    let mut builder = RenderGraphBuilder::new("frame");
    let color = builder.create_texture(TextureDesc::new(
        "scene-color",
        64,
        64,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let opaque = builder.add_pass("opaque", QueueLane::Graphics);
    let final_blit = builder.add_pass("final-blit", QueueLane::Graphics);
    builder.write_texture(opaque, color).unwrap();
    builder.read_texture(final_blit, color).unwrap();
    builder
        .set_pass_flags(
            final_blit,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .unwrap();

    let graph = builder.compile().unwrap();
    let final_blit_pass = graph
        .passes()
        .iter()
        .find(|pass| pass.name == "final-blit")
        .unwrap();

    assert_eq!(final_blit_pass.dependencies, vec![opaque]);
    assert_eq!(graph.stats().total_dependency_count, 1);
}

#[test]
fn compile_accepts_transitively_ordered_resource_writers() {
    let mut builder = RenderGraphBuilder::new("transitive-writers");
    let color = builder.create_texture(TextureDesc::new(
        "scene-color",
        64,
        64,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT,
    ));
    let first_writer = builder.add_pass("first-writer", QueueLane::Graphics);
    let ordering_bridge = builder.add_pass("ordering-bridge", QueueLane::Graphics);
    let second_writer = builder.add_pass("second-writer", QueueLane::Graphics);
    builder.write_texture(first_writer, color).unwrap();
    builder.write_texture(second_writer, color).unwrap();
    builder
        .add_dependency(first_writer, ordering_bridge)
        .unwrap();
    builder
        .add_dependency(ordering_bridge, second_writer)
        .unwrap();
    builder
        .set_pass_flags(
            second_writer,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .unwrap();

    let graph = builder.compile().unwrap();

    assert_eq!(
        graph
            .passes()
            .iter()
            .map(|pass| pass.name.as_str())
            .collect::<Vec<_>>(),
        vec!["first-writer", "ordering-bridge", "second-writer"]
    );
}

#[test]
fn compile_accepts_transitive_writer_chains_across_reachability_words() {
    for pass_count in [65, 129] {
        let mut builder = RenderGraphBuilder::new("wide-transitive-writers");
        let color = builder.create_texture(TextureDesc::new(
            "scene-color",
            64,
            64,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::RENDER_ATTACHMENT,
        ));
        let first_writer = builder.add_pass("first-writer", QueueLane::Graphics);
        let mut previous = first_writer;
        for index in 1..pass_count {
            let bridge = builder.add_pass(format!("ordering-bridge-{index}"), QueueLane::Graphics);
            builder.add_dependency(previous, bridge).unwrap();
            previous = bridge;
        }
        builder.write_texture(first_writer, color).unwrap();
        builder.write_texture(previous, color).unwrap();
        builder
            .set_pass_flags(
                previous,
                PassFlags {
                    has_side_effects: true,
                    ..PassFlags::default()
                },
            )
            .unwrap();

        let graph = builder.compile().unwrap();

        assert_eq!(graph.passes().len(), pass_count);
        assert!(graph.passes().iter().all(|pass| !pass.culled));
        assert_eq!(
            graph.passes().first().map(|pass| pass.id),
            Some(first_writer)
        );
        assert_eq!(graph.passes().last().map(|pass| pass.id), Some(previous));
    }
}
