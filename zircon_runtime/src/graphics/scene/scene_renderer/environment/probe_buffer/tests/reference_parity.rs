use crate::core::framework::render::{
    reflection_probe_box_project_direction, reflection_probe_influence_weight, ProbeInfluenceShape,
    ReflectionProbeData,
};
use crate::core::math::{Quat, Vec3};

use super::super::gpu_layout::GpuReflectionProbe;

#[test]
fn render_probe_gpu_weight_matches_cpu_reference_for_box_and_sphere() {
    let probes = [
        probe(
            ProbeInfluenceShape::box_shape(Vec3::new(3.0, 2.0, 4.0), 1.0).expect("box influence"),
            Quat::from_rotation_y(0.45),
        ),
        probe(
            ProbeInfluenceShape::sphere(4.0, 1.5).expect("sphere influence"),
            Quat::from_rotation_y(0.85),
        ),
    ];
    let positions = [
        Vec3::new(1.0, 0.5, -0.25),
        Vec3::new(2.8, 0.0, 0.0),
        Vec3::new(8.0, 0.0, 0.0),
    ];

    for probe in &probes {
        let gpu = GpuReflectionProbe::from_probe(probe, 0, 8);
        for position in positions {
            assert_approx(
                gpu_equivalent_weight(gpu, position),
                reflection_probe_influence_weight(probe, position),
            );
        }
    }
}

#[test]
fn render_probe_upload_normalizes_finite_scaled_box_rotation() {
    let scaled_rotation = Quat::from_rotation_y(0.45) * 2.75;
    let probe = probe(
        ProbeInfluenceShape::box_shape(Vec3::new(3.0, 2.0, 4.0), 1.0).expect("box influence"),
        scaled_rotation,
    );
    let gpu = GpuReflectionProbe::from_probe(&probe, 0, 8);

    assert_approx(probe.rotation().length(), 1.0);
    assert_approx(Quat::from_array(gpu.rotation).length(), 1.0);
    let expected_rotation = scaled_rotation.normalize();
    assert!(
        probe.rotation().dot(expected_rotation).abs() > 0.99999,
        "probe construction must preserve the scaled input rotation"
    );
    assert!(
        Quat::from_array(gpu.rotation).dot(expected_rotation).abs() > 0.99999,
        "GPU upload must preserve the normalized input rotation"
    );
    for position in [Vec3::new(1.0, 0.5, -0.25), Vec3::new(2.8, 0.0, 0.0)] {
        assert_approx(
            gpu_equivalent_weight(gpu, position),
            reflection_probe_influence_weight(&probe, position),
        );
    }
}

#[test]
fn render_probe_gpu_box_projection_matches_cpu_reference() {
    let probe = probe(
        ProbeInfluenceShape::box_shape(Vec3::new(5.0, 4.0, 3.0), 1.0).expect("box influence"),
        Quat::from_rotation_y(0.7),
    )
    .with_box_projection(true);
    let gpu = GpuReflectionProbe::from_probe(&probe, 0, 8);
    let world_position = Vec3::new(1.0, 0.5, -0.5);
    let reflection_direction = Vec3::new(0.8, 0.2, -0.4).normalize();

    let expected =
        reflection_probe_box_project_direction(&probe, world_position, reflection_direction);
    let actual = gpu_equivalent_box_project(gpu, world_position, reflection_direction);

    assert!(
        (actual - expected).length() < 0.0001,
        "{actual:?} != {expected:?}"
    );
}

fn probe(shape: ProbeInfluenceShape, rotation: Quat) -> ReflectionProbeData {
    ReflectionProbeData::try_new(1, Vec3::ZERO, rotation, shape, Vec3::new(5.0, 4.0, 3.0))
        .expect("reflection probe")
}

fn gpu_equivalent_weight(probe: GpuReflectionProbe, world_position: Vec3) -> f32 {
    let center = Vec3::from_array(probe.position_blend[..3].try_into().expect("center"));
    let position_delta = world_position - center;
    let half_extents = Vec3::from_array(probe.box_max[..3].try_into().expect("half extents"));
    let edge_distance = if probe.box_max[3] >= 0.5 {
        half_extents.x - position_delta.length()
    } else {
        let rotation = Quat::from_array(probe.rotation);
        let local = rotation.conjugate() * position_delta;
        (half_extents - local.abs()).min_element()
    };
    if edge_distance <= 0.0 {
        return 0.0;
    }
    let blend_distance = probe.position_blend[3];
    if blend_distance <= 0.000001 {
        return 1.0;
    }
    (edge_distance / blend_distance).clamp(0.0, 1.0)
}

fn gpu_equivalent_box_project(
    probe: GpuReflectionProbe,
    world_position: Vec3,
    reflection_direction: Vec3,
) -> Vec3 {
    let rotation = Quat::from_array(probe.rotation);
    let center = Vec3::from_array(probe.position_blend[..3].try_into().expect("center"));
    let extent = Vec3::from_array(probe.proj_params[..3].try_into().expect("extent"));
    let local_position = rotation.conjugate() * (world_position - center);
    let local_direction = rotation.conjugate() * reflection_direction;
    let mut distance = f32::INFINITY;
    for axis in 0..3 {
        if local_direction[axis].abs() <= 0.000001 {
            continue;
        }
        let plane = if local_direction[axis] > 0.0 {
            extent[axis]
        } else {
            -extent[axis]
        };
        let axis_distance = (plane - local_position[axis]) / local_direction[axis];
        if axis_distance >= 0.0 {
            distance = distance.min(axis_distance);
        }
    }
    rotation * (local_position + local_direction * distance)
}

fn assert_approx(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 0.0001, "{actual} != {expected}");
}
