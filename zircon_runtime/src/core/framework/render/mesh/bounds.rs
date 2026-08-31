use serde::{Deserialize, Serialize};

use crate::core::math::{Mat4, Transform, Vec3};

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
        self.transformed_by_affine(transform.matrix())
    }

    pub(crate) fn transformed_by_affine(self, world_from_local: Mat4) -> Self {
        let local_min = Vec3::from_array(self.min);
        let local_max = Vec3::from_array(self.max);
        let local_center = (local_min + local_max) * 0.5;
        let local_half_extent = (local_max - local_min).abs() * 0.5;
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
    use crate::core::math::{Mat4, Quat, Transform, Vec3, Vec4};

    #[test]
    fn render_mesh_bounds_transform_preserves_local_center_rotation_and_non_uniform_scale() {
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

    #[test]
    fn render_mesh_bounds_affine_matrix_projection_preserves_shear() {
        let local = RenderMeshBounds::from_min_max([-1.0, -2.0, -0.5], [3.0, 2.0, 0.5]);
        let world_from_local = Mat4::from_cols(
            Vec4::new(2.0, 0.0, 0.0, 0.0),
            Vec4::new(1.0, 1.0, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 3.0, 0.0),
            Vec4::new(10.0, 20.0, 30.0, 1.0),
        );

        let world = local.transformed_by_affine(world_from_local);

        assert_vec3_close(world.center, [12.0, 20.0, 30.0]);
        assert_vec3_close(world.min, [6.0, 18.0, 28.5]);
        assert_vec3_close(world.max, [18.0, 22.0, 31.5]);
        assert!((world.radius - 6.5).abs() <= 1.0e-5);
    }

    #[test]
    fn render_mesh_bounds_transform_rebuilds_derived_metadata_from_min_max() {
        let stale = RenderMeshBounds {
            min: [-1.0; 3],
            max: [1.0; 3],
            center: [100.0; 3],
            radius: 0.0,
        };

        let canonical = stale.transformed_by_affine(Mat4::IDENTITY);

        assert_eq!(canonical.min, [-1.0; 3]);
        assert_eq!(canonical.max, [1.0; 3]);
        assert_eq!(canonical.center, [0.0; 3]);
        assert!((canonical.radius - 3.0_f32.sqrt()).abs() <= 1.0e-5);
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
