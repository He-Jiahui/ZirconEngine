use serde::{Deserialize, Serialize};

use crate::core::math::{Transform, Vec3};

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RenderMeshBounds {
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub center: [f32; 3],
    pub radius: f32,
}

impl RenderMeshBounds {
    pub fn from_min_max(min: [f32; 3], max: [f32; 3]) -> Self {
        let center = [
            (min[0] + max[0]) * 0.5,
            (min[1] + max[1]) * 0.5,
            (min[2] + max[2]) * 0.5,
        ];
        Self {
            min,
            max,
            center,
            radius: max_distance_from_center([min, max], center),
        }
    }

    pub fn from_positions(positions: impl IntoIterator<Item = [f32; 3]>) -> Self {
        let mut iter = positions.into_iter();
        let Some(first) = iter.next() else {
            return Self::default();
        };

        let mut min = first;
        let mut max = first;
        for position in iter {
            for axis in 0..3 {
                min[axis] = min[axis].min(position[axis]);
                max[axis] = max[axis].max(position[axis]);
            }
        }

        Self::from_min_max(min, max)
    }

    pub fn transformed(self, transform: Transform) -> Self {
        let local_center = Vec3::from_array(self.center);
        let local_half_extent =
            (Vec3::from_array(self.max) - Vec3::from_array(self.min)).abs() * 0.5;
        let world_from_local = transform.matrix();
        let world_center = world_from_local.transform_point3(local_center);
        let world_half_extent = world_from_local
            .transform_vector3(Vec3::new(local_half_extent.x, 0.0, 0.0))
            .abs()
            + world_from_local
                .transform_vector3(Vec3::new(0.0, local_half_extent.y, 0.0))
                .abs()
            + world_from_local
                .transform_vector3(Vec3::new(0.0, 0.0, local_half_extent.z))
                .abs();
        Self::from_min_max(
            (world_center - world_half_extent).to_array(),
            (world_center + world_half_extent).to_array(),
        )
    }
}

fn max_distance_from_center(points: [[f32; 3]; 2], center: [f32; 3]) -> f32 {
    points
        .into_iter()
        .map(|point| {
            let dx = point[0] - center[0];
            let dy = point[1] - center[1];
            let dz = point[2] - center[2];
            (dx * dx + dy * dy + dz * dz).sqrt()
        })
        .fold(0.0, f32::max)
}

#[cfg(test)]
mod tests {
    use super::RenderMeshBounds;
    use crate::core::math::{Quat, Transform, Vec3};

    #[test]
    fn transformed_bounds_preserve_local_center_rotation_and_non_uniform_scale() {
        let local = RenderMeshBounds::from_min_max([-1.0, -2.0, -0.5], [3.0, 2.0, 0.5]);
        let transform = Transform::from_translation(Vec3::new(10.0, 20.0, 30.0))
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::new(2.0, 3.0, 4.0));

        let world = local.transformed(transform);

        assert_vec3_close(world.center, [10.0, 22.0, 30.0]);
        assert_vec3_close(world.min, [4.0, 18.0, 28.0]);
        assert_vec3_close(world.max, [16.0, 26.0, 32.0]);
        assert!((world.radius - 56.0_f32.sqrt()).abs() <= 1.0e-5);
    }

    fn assert_vec3_close(actual: [f32; 3], expected: [f32; 3]) {
        for axis in 0..3 {
            assert!(
                (actual[axis] - expected[axis]).abs() <= 1.0e-5,
                "axis {axis}: expected {}, got {}",
                expected[axis],
                actual[axis]
            );
        }
    }
}
