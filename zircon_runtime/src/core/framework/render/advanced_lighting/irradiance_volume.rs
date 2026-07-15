use serde::{Deserialize, Serialize};

use crate::core::framework::render::RenderLayerSet;
use crate::core::math::{Mat4, Real, Vec3};
use crate::core::resource::ResourceId as AssetId;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IrradianceVolumeData {
    pub volume_id: u64,
    pub transform: Mat4,
    pub voxels: AssetId,
    pub intensity: Real,
    pub affects_lightmapped_meshes: bool,
    pub priority: i32,
    #[serde(default)]
    pub layer_mask: RenderLayerSet,
}

impl IrradianceVolumeData {
    /// Maps a world-space position into the volume's normalized texture domain.
    /// Authored local volume bounds are `[-0.5, 0.5]` on every axis.
    pub fn world_to_uvw(&self, world_position: Vec3) -> Vec3 {
        self.transform.transform_point3(world_position) + Vec3::splat(0.5)
    }

    pub fn contains_world_position(&self, world_position: Vec3) -> bool {
        let uvw = self.world_to_uvw(world_position);
        uvw.is_finite() && uvw.cmpge(Vec3::ZERO).all() && uvw.cmple(Vec3::ONE).all()
    }
}

pub fn select_irradiance_volume<'a>(
    volumes: &'a [IrradianceVolumeData],
    world_position: Vec3,
    render_layers: &RenderLayerSet,
) -> Option<&'a IrradianceVolumeData> {
    volumes
        .iter()
        .filter(|volume| {
            volume.intensity > 0.0
                && volume.layer_mask.intersects(render_layers)
                && volume.contains_world_position(world_position)
        })
        .max_by(|left, right| {
            left.priority
                .cmp(&right.priority)
                .then_with(|| right.volume_id.cmp(&left.volume_id))
        })
}

pub fn select_irradiance_volume_for_view<'a>(
    volumes: &'a [IrradianceVolumeData],
    render_layers: &RenderLayerSet,
    visible_world_positions: &[Vec3],
) -> Option<&'a IrradianceVolumeData> {
    volumes
        .iter()
        .filter(|volume| {
            volume.intensity > 0.0
                && volume.layer_mask.intersects(render_layers)
                && volume.transform.is_finite()
                && volume.transform.determinant().abs() > Real::EPSILON
                && (visible_world_positions.is_empty()
                    || visible_world_positions
                        .iter()
                        .copied()
                        .any(|position| volume.contains_world_position(position)))
        })
        .max_by(|left, right| {
            left.priority
                .cmp(&right.priority)
                .then_with(|| right.volume_id.cmp(&left.volume_id))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::resource::ResourceId;

    const EPSILON: f32 = 1.0e-5;

    #[test]
    fn render_irrvol_world_to_uvw_roundtrip() {
        let world_from_volume = Mat4::from_scale_rotation_translation(
            Vec3::new(4.0, 2.0, 8.0),
            crate::core::math::Quat::from_rotation_y(0.4),
            Vec3::new(10.0, 3.0, -6.0),
        );
        let volume = volume(1, 0, world_from_volume.inverse());
        let local = Vec3::new(0.25, -0.125, 0.4);
        let world = world_from_volume.transform_point3(local);
        let uvw = volume.world_to_uvw(world);

        assert_vec3_near(uvw, local + Vec3::splat(0.5));
        assert!(volume.contains_world_position(world));
        assert!(!volume.contains_world_position(
            world_from_volume.transform_point3(Vec3::new(0.51, 0.0, 0.0))
        ));
    }

    #[test]
    fn render_irrvol_selection_prefers_priority_inside() {
        let layers = RenderLayerSet::layer(2);
        let volumes = vec![
            volume(9, 1, Mat4::IDENTITY),
            volume(7, 5, Mat4::IDENTITY),
            volume(3, 5, Mat4::IDENTITY),
            IrradianceVolumeData {
                layer_mask: RenderLayerSet::layer(4),
                ..volume(1, 99, Mat4::IDENTITY)
            },
        ];

        let selected = select_irradiance_volume(&volumes, Vec3::ZERO, &layers)
            .expect("an intersecting volume should be selected");
        assert_eq!(selected.volume_id, 3);
        assert!(select_irradiance_volume(&volumes, Vec3::splat(0.75), &layers).is_none());
    }

    #[test]
    fn render_irrvol_view_selection_does_not_require_camera_containment() {
        let layers = RenderLayerSet::layer(2);
        let volumes = vec![volume(7, 4, Mat4::IDENTITY), volume(3, 4, Mat4::IDENTITY)];

        let selected = select_irradiance_volume_for_view(&volumes, &layers, &[])
            .expect("a layer-compatible volume should be selected for per-pixel containment");

        assert_eq!(selected.volume_id, 3);
    }

    #[test]
    fn render_irrvol_view_selection_ignores_unrelated_higher_priority_volume() {
        let layers = RenderLayerSet::layer(2);
        let visible = volume(7, 4, Mat4::IDENTITY);
        let unrelated = volume(3, 99, Mat4::from_translation(Vec3::new(-100.0, 0.0, 0.0)));
        let volumes = [visible, unrelated];

        let selected = select_irradiance_volume_for_view(&volumes, &layers, &[Vec3::ZERO])
            .expect("the volume containing visible scene content should be selected");

        assert_eq!(selected.volume_id, 7);
    }

    fn volume(volume_id: u64, priority: i32, transform: Mat4) -> IrradianceVolumeData {
        IrradianceVolumeData {
            volume_id,
            transform,
            voxels: ResourceId::from_stable_label(&format!(
                "runtime://irradiance-volume/{volume_id}"
            )),
            intensity: 1.0,
            affects_lightmapped_meshes: false,
            priority,
            layer_mask: RenderLayerSet::layer(2),
        }
    }

    fn assert_vec3_near(actual: Vec3, expected: Vec3) {
        assert!((actual.x - expected.x).abs() <= EPSILON);
        assert!((actual.y - expected.y).abs() <= EPSILON);
        assert!((actual.z - expected.z).abs() <= EPSILON);
    }
}
