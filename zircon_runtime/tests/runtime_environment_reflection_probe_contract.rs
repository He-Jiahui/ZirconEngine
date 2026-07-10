use zircon_runtime::core::framework::render::{
    reflection_probe_box_project_direction, reflection_probe_influence_weight,
    select_reflection_probe_blend, EnvironmentExtract, ProbeBakeTiming, ProbeInfluenceShape,
    ReflectionProbeData, ReflectionProbeValidationError, RenderLayerSet,
};
use zircon_runtime::core::math::{Quat, Vec3};
use zircon_runtime::core::resource::ResourceId;

const EPSILON: f32 = 0.0001;

#[test]
fn reflection_probe_box_weight_fades_linearly_inside_the_boundary() {
    let probe = box_probe(
        1,
        Vec3::ZERO,
        Quat::IDENTITY,
        Vec3::new(2.0, 3.0, 4.0),
        1.0,
        0,
        "probe-box-weight",
    );

    assert_close(reflection_probe_influence_weight(&probe, Vec3::ZERO), 1.0);
    assert_close(
        reflection_probe_influence_weight(&probe, Vec3::new(1.5, 0.0, 0.0)),
        0.5,
    );
    assert_close(
        reflection_probe_influence_weight(&probe, Vec3::new(2.0, 0.0, 0.0)),
        0.0,
    );
    assert_close(
        reflection_probe_influence_weight(&probe, Vec3::new(2.1, 0.0, 0.0)),
        0.0,
    );
}

#[test]
fn reflection_probe_sphere_weight_fades_linearly_inside_the_boundary() {
    let probe = sphere_probe(2, Vec3::ZERO, 4.0, 2.0, 0, "probe-sphere-weight");

    assert_close(reflection_probe_influence_weight(&probe, Vec3::ZERO), 1.0);
    assert_close(
        reflection_probe_influence_weight(&probe, Vec3::new(3.0, 0.0, 0.0)),
        0.5,
    );
    assert_close(
        reflection_probe_influence_weight(&probe, Vec3::new(4.0, 0.0, 0.0)),
        0.0,
    );
}

#[test]
fn reflection_probe_box_projection_hits_the_rotated_box_face() {
    let rotation = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
    let probe = box_probe(
        3,
        Vec3::new(10.0, 2.0, 3.0),
        rotation,
        Vec3::new(2.0, 3.0, 4.0),
        1.0,
        0,
        "probe-box-projection",
    );
    let local_reflection = Vec3::X;
    let world_reflection = rotation * local_reflection;

    let projected =
        reflection_probe_box_project_direction(&probe, Vec3::new(10.0, 2.0, 3.0), world_reflection);

    assert_vec3_close(projected, rotation * Vec3::new(2.0, 0.0, 0.0));
}

#[test]
fn reflection_probe_blend_selects_top_two_and_preserves_sky_fallback() {
    let probes = vec![
        sphere_probe(10, Vec3::new(0.6, 0.0, 0.0), 1.0, 1.0, 0, "probe-a"),
        sphere_probe(11, Vec3::new(0.7, 0.0, 0.0), 1.0, 1.0, 0, "probe-b"),
        sphere_probe(12, Vec3::new(0.8, 0.0, 0.0), 1.0, 1.0, 0, "probe-c"),
    ];

    let blend = select_reflection_probe_blend(&probes, Vec3::ZERO, &RenderLayerSet::default());

    let primary = blend.primary.expect("highest-weight probe");
    let secondary = blend.secondary.expect("second-highest probe");
    assert_eq!(primary.probe_index, 0);
    assert_eq!(secondary.probe_index, 1);
    assert_close(primary.weight, 0.4);
    assert_close(secondary.weight, 0.3);
    assert_close(blend.skybox_weight, 0.3);
    assert_close(primary.weight + secondary.weight + blend.skybox_weight, 1.0);
}

#[test]
fn reflection_probe_blend_uses_priority_for_equal_weights_and_filters_layers() {
    let default_layer = RenderLayerSet::default();
    let hidden_layer = RenderLayerSet::layer(7);
    let probes = vec![
        sphere_probe(20, Vec3::new(0.5, 0.0, 0.0), 1.0, 1.0, 1, "probe-low"),
        sphere_probe(21, Vec3::new(-0.5, 0.0, 0.0), 1.0, 1.0, 9, "probe-high"),
        sphere_probe(22, Vec3::ZERO, 1.0, 1.0, 100, "probe-hidden").with_layer_mask(hidden_layer),
    ];

    let blend = select_reflection_probe_blend(&probes, Vec3::ZERO, &default_layer);

    assert_eq!(blend.primary.expect("primary probe").probe_index, 1);
    assert_eq!(blend.secondary.expect("secondary probe").probe_index, 0);
}

#[test]
fn reflection_probe_contract_roundtrips_and_is_owned_by_environment_extract() {
    let probe = box_probe(
        30,
        Vec3::new(1.0, 2.0, 3.0),
        Quat::from_rotation_x(0.25),
        Vec3::new(4.0, 5.0, 6.0),
        0.75,
        4,
        "probe-roundtrip",
    )
    .with_bake_timing(ProbeBakeTiming::RuntimeManual);
    let encoded = serde_json::to_string(&probe).expect("serialize reflection probe");
    let decoded: ReflectionProbeData =
        serde_json::from_str(&encoded).expect("deserialize reflection probe");
    let environment = EnvironmentExtract::disabled().with_reflection_probes(vec![decoded.clone()]);

    assert_eq!(decoded, probe);
    assert_eq!(environment.reflection_probes(), &[probe]);
}

#[test]
fn reflection_probe_invalid_radius_is_a_typed_error() {
    let error = ProbeInfluenceShape::sphere(-1.0, 0.5)
        .expect_err("negative sphere radius must be rejected");

    assert_eq!(
        error,
        ReflectionProbeValidationError::InvalidSphereRadius { radius: -1.0 }
    );
}

fn box_probe(
    probe_id: u64,
    position: Vec3,
    rotation: Quat,
    half_extents: Vec3,
    blend_distance: f32,
    priority: i32,
    asset_label: &str,
) -> ReflectionProbeData {
    ReflectionProbeData::try_new(
        probe_id,
        position,
        rotation,
        ProbeInfluenceShape::box_shape(half_extents, blend_distance).expect("valid box influence"),
        half_extents,
    )
    .expect("valid reflection probe")
    .with_box_projection(true)
    .with_baked_cubemap(Some(ResourceId::from_stable_label(asset_label)))
    .with_priority(priority)
}

fn sphere_probe(
    probe_id: u64,
    position: Vec3,
    radius: f32,
    blend_distance: f32,
    priority: i32,
    asset_label: &str,
) -> ReflectionProbeData {
    ReflectionProbeData::try_new(
        probe_id,
        position,
        Quat::IDENTITY,
        ProbeInfluenceShape::sphere(radius, blend_distance).expect("valid sphere influence"),
        Vec3::splat(radius),
    )
    .expect("valid reflection probe")
    .with_baked_cubemap(Some(ResourceId::from_stable_label(asset_label)))
    .with_priority(priority)
}

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= EPSILON,
        "actual={actual}, expected={expected}"
    );
}

fn assert_vec3_close(actual: Vec3, expected: Vec3) {
    assert_close(actual.x, expected.x);
    assert_close(actual.y, expected.y);
    assert_close(actual.z, expected.z);
}
