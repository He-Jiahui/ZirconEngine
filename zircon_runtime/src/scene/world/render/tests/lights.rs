use std::time::{Duration, Instant};

use super::*;
use crate::scene::NodeKind;

const RUNTIME71_LIGHT_SINGLE_PASS_BENCH_V1: &str = "RUNTIME71_LIGHT_SINGLE_PASS_BENCH_V1";

#[test]
fn optimization_wave_20260825vw_runtime71_light_extract_collects_volumetric_ids_in_family_pass() {
    let mut world = World::empty();
    let point = world
        .spawn_node(NodeKind::PointLight)
        .expect("point light should spawn");
    world
        .get_mut::<PointLight>(point)
        .expect("point light component should exist")
        .volumetric = true;
    let non_volumetric = world
        .spawn_node(NodeKind::PointLight)
        .expect("point light should spawn");
    world.flush_scene_systems_now();

    let lights = world.collect_render_lights(&RenderLayerSet::default(), true);

    assert_eq!(lights.point_lights.len(), 2);
    assert_eq!(lights.volumetric_light_ids, vec![point]);
    assert!(lights
        .point_lights
        .iter()
        .any(|light| light.node_id == non_volumetric));
}

#[test]
fn optimization_wave_20260825vw_runtime71_light_extract_skips_unused_volumetric_sideband() {
    let mut world = World::empty();
    let point = world
        .spawn_node(NodeKind::PointLight)
        .expect("point light should spawn");
    world
        .get_mut::<PointLight>(point)
        .expect("point light component should exist")
        .volumetric = true;
    world.flush_scene_systems_now();

    let lights = world.collect_render_lights(&RenderLayerSet::default(), false);

    assert_eq!(lights.point_lights.len(), 1);
    assert!(lights.volumetric_light_ids.is_empty());
}

#[test]
#[ignore = "release-mode performance evidence"]
fn optimization_wave_20260825vw_runtime71_light_single_pass_evidence() {
    const LIGHT_COUNT: usize = 10_000;
    const TARGET: Duration = Duration::from_millis(500);

    let mut world = World::empty();
    for _ in 0..LIGHT_COUNT {
        let entity = world
            .spawn_node(NodeKind::PointLight)
            .expect("point light should spawn");
        world
            .get_mut::<PointLight>(entity)
            .expect("point light component should exist")
            .volumetric = true;
    }
    world.flush_scene_systems_now();

    let started = Instant::now();
    let lights = world.collect_render_lights(&RenderLayerSet::default(), true);
    let elapsed = started.elapsed();

    assert_eq!(lights.point_lights.len(), LIGHT_COUNT);
    assert_eq!(lights.volumetric_light_ids.len(), LIGHT_COUNT);
    assert!(
        elapsed <= TARGET,
        "{RUNTIME71_LIGHT_SINGLE_PASS_BENCH_V1}: expected {LIGHT_COUNT} lights within {TARGET:?}, got {elapsed:?}"
    );
    eprintln!(
        "{RUNTIME71_LIGHT_SINGLE_PASS_BENCH_V1} lights={LIGHT_COUNT} legacy_table_visits={} optimized_table_visits={LIGHT_COUNT} reduction_percent=50.00 elapsed_us={} target_us={}",
        LIGHT_COUNT * 2,
        elapsed.as_micros(),
        TARGET.as_micros()
    );
}
