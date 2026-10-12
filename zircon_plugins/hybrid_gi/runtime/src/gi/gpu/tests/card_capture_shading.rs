use zircon_runtime::core::framework::render::{
    render_mesh_stable_instance_key, render_mesh_transform_revision,
    RenderDirectionalLightSnapshot, RenderLayerSet, RenderMeshStaticState,
    RenderPointLightSnapshot, RenderSpotLightSnapshot, RendererCommon,
};
use zircon_runtime::core::framework::scene::Mobility;
use zircon_runtime::core::math::Transform;
use zircon_runtime::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};

use super::*;

fn directional_light(direction: Vec3) -> RenderDirectionalLightSnapshot {
    RenderDirectionalLightSnapshot {
        node_id: 1,
        light_id: 1,
        layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
        direction,
        color: Vec3::new(1.0, 0.1, 0.05),
        intensity: 2.0,
        mobility: zircon_runtime::core::framework::scene::Mobility::Dynamic,
        shadow: None,
    }
}

fn point_light(position: Vec3, range: f32) -> RenderPointLightSnapshot {
    RenderPointLightSnapshot {
        node_id: 2,
        light_id: 2,
        layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
        position,
        color: Vec3::new(0.05, 1.0, 0.1),
        intensity: 4.0,
        range,
        mobility: zircon_runtime::core::framework::scene::Mobility::Dynamic,
        shadow: None,
    }
}

fn spot_light(direction: Vec3) -> RenderSpotLightSnapshot {
    RenderSpotLightSnapshot {
        node_id: 3,
        light_id: 3,
        layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
        position: Vec3::new(0.0, 0.0, 2.0),
        direction,
        color: Vec3::new(0.05, 0.1, 1.0),
        intensity: 5.0,
        range: 5.0,
        inner_angle_radians: 0.2,
        outer_angle_radians: 0.7,
        mobility: zircon_runtime::core::framework::scene::Mobility::Dynamic,
        shadow: None,
    }
}

#[test]
fn directional_capture_respects_surface_orientation() {
    let front_lit = directional_light_contribution(Vec3::Z, false, &directional_light(Vec3::NEG_Z));
    let back_lit = directional_light_contribution(Vec3::Z, false, &directional_light(Vec3::Z));

    assert!(front_lit.x > 0.0);
    assert_eq!(back_lit, Vec3::ZERO);
}

#[test]
fn point_capture_respects_distance_and_range() {
    let near = point_light_contribution(
        Vec3::ZERO,
        Vec3::Z,
        false,
        &point_light(Vec3::new(0.0, 0.0, 1.0), 5.0),
    );
    let far = point_light_contribution(
        Vec3::ZERO,
        Vec3::Z,
        false,
        &point_light(Vec3::new(0.0, 0.0, 4.0), 5.0),
    );
    let outside = point_light_contribution(
        Vec3::ZERO,
        Vec3::Z,
        false,
        &point_light(Vec3::new(0.0, 0.0, 5.0), 5.0),
    );

    assert!(near.y > far.y);
    assert!(far.y > 0.0);
    assert_eq!(outside, Vec3::ZERO);
}

#[test]
fn spot_capture_respects_cone_direction() {
    let inside = spot_light_contribution(Vec3::ZERO, Vec3::Z, false, &spot_light(Vec3::NEG_Z));
    let outside = spot_light_contribution(Vec3::ZERO, Vec3::Z, false, &spot_light(Vec3::X));

    assert!(inside.z > 0.0);
    assert_eq!(outside, Vec3::ZERO);
}

#[test]
fn material_occlusion_applies_gltf_strength() {
    assert_eq!(material_occlusion(0.2, 0.0), 1.0);
    assert!((material_occlusion(0.2, 0.25) - 0.8).abs() <= f32::EPSILON);
    assert!((material_occlusion(0.2, 1.0) - 0.2).abs() <= f32::EPSILON);
    assert_eq!(material_occlusion(f32::NAN, 1.0), 1.0);
    assert!((material_occlusion(0.2, f32::NAN) - 0.2).abs() <= f32::EPSILON);
}

#[test]
fn normal_texture_scale_only_scales_tangent_xy_before_normalization() {
    let unscaled = decode_normal_texture(Vec3::new(0.75, 0.5, 1.0), 1.0);
    let flattened = decode_normal_texture(Vec3::new(0.75, 0.5, 1.0), 0.0);
    let non_finite = decode_normal_texture(Vec3::new(0.75, 0.5, 1.0), f32::NAN);

    assert!(unscaled.x > 0.0);
    assert!(unscaled.z < 1.0);
    assert_eq!(flattened, Vec3::Z);
    assert_eq!(non_finite, unscaled);
}

#[test]
fn card_capture_applies_gltf_occlusion_strength_to_indirect_light() {
    let no_occlusion = capture_ambient_with_occlusion_strength(0.0);
    let quarter_strength = capture_ambient_with_occlusion_strength(0.25);
    let full_strength = capture_ambient_with_occlusion_strength(1.0);

    assert!((no_occlusion.x - 0.08).abs() <= 0.000_001);
    assert!((quarter_strength.x - 0.064).abs() <= 0.000_001);
    assert!((full_strength.x - 0.016).abs() <= 0.000_001);
    assert!(no_occlusion.x > quarter_strength.x);
    assert!(quarter_strength.x > full_strength.x);
}

struct OcclusionCaptureSource {
    seed: HybridGiMaterialCaptureSeed,
    occlusion_texture: HybridGiMaterialCaptureTextureKey,
}

impl HybridGiMaterialCaptureSource for OcclusionCaptureSource {
    fn material_capture_seed(&self, _id: &ResourceId) -> Option<HybridGiMaterialCaptureSeed> {
        Some(self.seed)
    }

    fn sample_texture_rgba(
        &self,
        texture: Option<HybridGiMaterialCaptureTextureKey>,
        _uv: [f32; 2],
    ) -> Option<Vec4> {
        (texture == Some(self.occlusion_texture)).then_some(Vec4::splat(0.2))
    }
}

fn capture_ambient_with_occlusion_strength(occlusion_strength: f32) -> Vec3 {
    let material_id = ResourceId::from_stable_label("res://materials/occlusion.zmaterial");
    let occlusion_texture = HybridGiMaterialCaptureTextureKey::new(
        ResourceId::from_stable_label("res://textures/occlusion.png"),
        1,
    );
    let source = OcclusionCaptureSource {
        seed: HybridGiMaterialCaptureSeed {
            base_color: Vec4::ONE,
            emissive: Vec3::ZERO,
            metallic: 0.0,
            roughness: 1.0,
            occlusion_strength,
            normal_scale: 1.0,
            double_sided: false,
            alpha_blend: false,
            alpha_cutoff: None,
            cast_shadows: true,
            base_color_texture: None,
            normal_texture: None,
            metallic_roughness_texture: None,
            occlusion_texture: Some(occlusion_texture),
            emissive_texture: None,
        },
        occlusion_texture,
    };
    let transform = Transform::from_translation(Vec3::ZERO);
    let mesh = RenderMeshSnapshot {
        node_id: 1,
        stable_instance_key: render_mesh_stable_instance_key(1, 0),
        transform_revision: render_mesh_transform_revision(&transform),
        transform,
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label("builtin://cube")),
        mesh: None,
        material: ResourceHandle::<MaterialMarker>::new(material_id),
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility: Mobility::Static,
        static_state: RenderMeshStaticState::from_transform_static(true),
        common: RendererCommon {
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
            is_static: true,
            ..RendererCommon::default()
        },
    };

    mesh_capture_radiance(
        &mesh,
        Vec3::ZERO,
        &source,
        &HybridGiPrepareExecutionInputs::default(),
    )
}
