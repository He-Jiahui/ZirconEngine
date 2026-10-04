use std::time::Instant;

use crate::core::math::Vec3;
use crate::scene::components::{NodeKind, NodeRecord};
use crate::scene::ecs::InternalSceneSystem;
use crate::scene::World;

use super::Transform;

const NODE_COUNTS: [usize; 4] = [1, 1_000, 100_000, 1_000_000];
const CHAIN_DEPTHS: [usize; 3] = [1, 32, 1_024];
const SAMPLE_COUNT: usize = 31;
const FIRST_ENTITY: u64 = 2_000_000;

#[derive(Clone, Copy)]
enum HierarchyShape {
    Star,
    Chain,
}

impl HierarchyShape {
    const fn label(self) -> &'static str {
        match self {
            Self::Star => "star",
            Self::Chain => "chain",
        }
    }
}

#[derive(Clone, Copy)]
struct PropagationSample {
    elapsed_ns: u64,
    visited: u64,
    written: u64,
}

#[test]
#[ignore = "managed Windows Release Runtime62 star 1/1K/100K/1M and chain 1/32/1024 samples"]
fn runtime62_derived_propagation_scale_profile() {
    assert!(
        !cfg!(debug_assertions),
        "run this ignored profile with a Release build"
    );

    for node_count in NODE_COUNTS {
        let mut world = hierarchy_world(node_count, HierarchyShape::Star);
        let matrix_samples = matrix_propagation_samples(&mut world, node_count);
        report_samples("star", "world_matrix", node_count, &matrix_samples);

        let active_samples = active_propagation_samples(&mut world, node_count);
        report_samples("star", "active_in_hierarchy", node_count, &active_samples);
    }
    // The branching-ancestor stack must not make a long single-child path
    // slower or allocate depth-sized pending frames.
    for depth in CHAIN_DEPTHS {
        let mut world = hierarchy_world(depth, HierarchyShape::Chain);
        let matrix_samples = matrix_propagation_samples(&mut world, depth);
        report_samples("chain", "world_matrix", depth, &matrix_samples);

        let active_samples = active_propagation_samples(&mut world, depth);
        report_samples("chain", "active_in_hierarchy", depth, &active_samples);
    }
}

fn hierarchy_world(node_count: usize, shape: HierarchyShape) -> World {
    let mut template_world = World::empty();
    let template_entity = template_world
        .spawn_node(NodeKind::Empty)
        .expect("scale profile template should spawn");
    let template = template_world
        .node_record(template_entity)
        .expect("scale profile template should have a node record");

    let mut records = Vec::with_capacity(node_count);
    for index in 0..node_count {
        let mut record: NodeRecord = template.clone();
        record.id = FIRST_ENTITY + index as u64;
        record.name = format!("Propagation scale profile {index}");
        record.parent = if index == 0 {
            None
        } else {
            Some(match shape {
                HierarchyShape::Star => FIRST_ENTITY,
                HierarchyShape::Chain => FIRST_ENTITY + index as u64 - 1,
            })
        };
        records.push(record);
    }

    let mut world = World::empty();
    world
        .insert_owned_node_records(records)
        .expect("scale profile records should publish");
    world.flush_pending_scene_systems();
    assert!(!world.has_pending_scene_systems());
    world
}

fn matrix_propagation_samples(world: &mut World, node_count: usize) -> Vec<PropagationSample> {
    let mut samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        world.reset_ecs_frame_performance_diagnostics();
        world
            .update_transform(
                FIRST_ENTITY,
                Transform::from_translation(Vec3::new(sample as f32 + 1.0, 0.0, 0.0)),
            )
            .expect("root transform should change");

        let started = Instant::now();
        world.run_internal_scene_system(InternalSceneSystem::WorldTransform);
        let elapsed_ns = started.elapsed().as_nanos() as u64;
        let diagnostics = world.ecs_frame_performance_diagnostics().derived_state;
        let sample = PropagationSample {
            elapsed_ns,
            visited: diagnostics.world_matrix_propagation_entities,
            written: diagnostics.world_matrix_propagation_written_entities,
        };
        assert_eq!(sample.visited, node_count as u64);
        assert_eq!(sample.written, node_count as u64);
        samples.push(sample);
        // Publish render dirtiness outside the timer so the next sample does
        // not measure an ever-growing journal from previous root changes.
        world.flush_pending_scene_systems();
    }
    samples
}

fn active_propagation_samples(world: &mut World, node_count: usize) -> Vec<PropagationSample> {
    let mut samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        world.reset_ecs_frame_performance_diagnostics();
        world
            .set_active_self(FIRST_ENTITY, sample % 2 == 1)
            .expect("root active state should change");

        let started = Instant::now();
        world.run_internal_scene_system(InternalSceneSystem::ActiveHierarchy);
        let elapsed_ns = started.elapsed().as_nanos() as u64;
        let diagnostics = world.ecs_frame_performance_diagnostics().derived_state;
        let sample = PropagationSample {
            elapsed_ns,
            visited: diagnostics.active_propagation_entities,
            written: diagnostics.active_propagation_written_entities,
        };
        assert_eq!(sample.visited, node_count as u64);
        assert_eq!(sample.written, node_count as u64);
        samples.push(sample);
        world.flush_pending_scene_systems();
    }
    samples
}

fn report_samples(shape: &str, kind: &str, node_count: usize, samples: &[PropagationSample]) {
    let elapsed_ns = samples
        .iter()
        .map(|sample| sample.elapsed_ns)
        .collect::<Vec<_>>();
    let visited = samples
        .iter()
        .map(|sample| sample.visited)
        .collect::<Vec<_>>();
    let written = samples
        .iter()
        .map(|sample| sample.written)
        .collect::<Vec<_>>();
    println!(
        "RUNTIME62_DERIVED_PROPAGATION_SCALE_PROFILE_V1 shape={shape} kind={kind} node_count={node_count} samples={SAMPLE_COUNT} elapsed_ns_p50={} elapsed_ns_p95={} elapsed_ns_p99={} elapsed_ns_raw={elapsed_ns:?} visited_raw={visited:?} written_raw={written:?}",
        nearest_rank_percentile(&elapsed_ns, 50),
        nearest_rank_percentile(&elapsed_ns, 95),
        nearest_rank_percentile(&elapsed_ns, 99),
    );
}

fn nearest_rank_percentile(samples: &[u64], percentile: usize) -> u64 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = (ordered.len() * percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}
