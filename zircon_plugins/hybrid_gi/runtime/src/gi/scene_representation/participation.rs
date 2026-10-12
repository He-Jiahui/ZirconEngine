use std::collections::BTreeMap;

use zircon_runtime::core::framework::render::{
    LightmapConsumeContract, RenderDirectionalLightSnapshot, RenderHybridGiMode,
    RenderMeshSnapshot, RenderPointLightSnapshot, RenderSpotLightSnapshot,
};
use zircon_runtime::core::framework::scene::Mobility;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HybridGiSurfaceParticipation {
    BakedStatic,
    DynamicReceiver,
    HybridReceiver,
    Disabled,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct HybridGiParticipationState {
    surfaces: BTreeMap<u64, HybridGiSurfaceParticipation>,
    surface_signatures: BTreeMap<u64, u64>,
    light_signatures: BTreeMap<u64, u64>,
    light_set_generation: Option<u64>,
    participation_epoch: u64,
}

impl HybridGiParticipationState {
    pub(super) fn synchronize(
        &mut self,
        mode: RenderHybridGiMode,
        meshes: &[RenderMeshSnapshot],
        directional_lights: &[RenderDirectionalLightSnapshot],
        point_lights: &[RenderPointLightSnapshot],
        spot_lights: &[RenderSpotLightSnapshot],
        baked_lighting: Option<&LightmapConsumeContract>,
        has_baked_probe_grid: bool,
    ) {
        let light_set_generation = baked_lighting.map(|contract| contract.light_set_generation);
        let surfaces = meshes
            .iter()
            .map(|mesh| {
                (
                    mesh.stable_instance_key,
                    classify_surface(mode, mesh, baked_lighting, has_baked_probe_grid),
                )
            })
            .collect();
        let surface_signatures = meshes
            .iter()
            .map(|mesh| (mesh.stable_instance_key, mesh_invalidation_signature(mesh)))
            .collect();
        let light_signatures =
            light_invalidation_signatures(directional_lights, point_lights, spot_lights);

        if self.surfaces != surfaces
            || self.surface_signatures != surface_signatures
            || self.light_signatures != light_signatures
            || self.light_set_generation != light_set_generation
        {
            self.surfaces = surfaces;
            self.surface_signatures = surface_signatures;
            self.light_signatures = light_signatures;
            self.light_set_generation = light_set_generation;
            self.participation_epoch = self.participation_epoch.saturating_add(1).max(1);
        }
    }

    pub(super) fn light_set_generation(&self) -> Option<u64> {
        self.light_set_generation
    }

    pub(super) fn participation_epoch(&self) -> u64 {
        self.participation_epoch
    }

    pub(super) fn surface(&self, stable_instance_key: u64) -> Option<HybridGiSurfaceParticipation> {
        self.surfaces.get(&stable_instance_key).copied()
    }

    pub(super) fn surfaces(
        &self,
    ) -> impl Iterator<Item = (u64, HybridGiSurfaceParticipation)> + '_ {
        self.surfaces
            .iter()
            .map(|(stable_instance_key, participation)| (*stable_instance_key, *participation))
    }
}

fn mesh_invalidation_signature(mesh: &RenderMeshSnapshot) -> u64 {
    let mut signature = mix_signature(mesh.stable_instance_key, mesh.transform_revision);
    signature = mix_signature(signature, mesh.static_state.geometry_revision);
    signature = mix_signature(signature, mesh.static_state.material_revision);
    signature = mix_signature(signature, mesh.mobility as u64);
    signature = mix_resource_id(signature, mesh.model.id());
    signature = mix_resource_id(signature, mesh.material.id());
    if let Some(mesh_resource) = mesh.mesh {
        signature = mix_resource_id(signature, mesh_resource.id());
    }
    for component in mesh.tint.to_array() {
        signature = mix_signature(signature, u64::from(component.to_bits()));
    }
    signature
}

fn light_invalidation_signatures(
    directional_lights: &[RenderDirectionalLightSnapshot],
    point_lights: &[RenderPointLightSnapshot],
    spot_lights: &[RenderSpotLightSnapshot],
) -> BTreeMap<u64, u64> {
    let mut signatures = BTreeMap::new();
    for light in directional_lights {
        let mut signature = light_signature(light.light_id, light.mobility);
        signature = mix_vec3(signature, light.direction);
        signature = mix_vec3(signature, light.color);
        signature = mix_signature(signature, u64::from(light.intensity.to_bits()));
        signature = mix_shadow(signature, light.shadow);
        signatures.insert(light.light_id, signature);
    }
    for light in point_lights {
        let mut signature = light_signature(light.light_id, light.mobility);
        signature = mix_vec3(signature, light.position);
        signature = mix_vec3(signature, light.color);
        signature = mix_signature(signature, u64::from(light.intensity.to_bits()));
        signature = mix_signature(signature, u64::from(light.range.to_bits()));
        signature = mix_shadow(signature, light.shadow);
        signatures.insert(light.light_id, signature);
    }
    for light in spot_lights {
        let mut signature = light_signature(light.light_id, light.mobility);
        signature = mix_vec3(signature, light.position);
        signature = mix_vec3(signature, light.direction);
        signature = mix_vec3(signature, light.color);
        signature = mix_signature(signature, u64::from(light.intensity.to_bits()));
        signature = mix_signature(signature, u64::from(light.range.to_bits()));
        signature = mix_signature(signature, u64::from(light.inner_angle_radians.to_bits()));
        signature = mix_signature(signature, u64::from(light.outer_angle_radians.to_bits()));
        signature = mix_shadow(signature, light.shadow);
        signatures.insert(light.light_id, signature);
    }
    signatures
}

fn light_signature(light_id: u64, mobility: Mobility) -> u64 {
    mix_signature(light_id, mobility as u64)
}

fn mix_vec3(mut signature: u64, value: zircon_runtime::core::math::Vec3) -> u64 {
    for component in value.to_array() {
        signature = mix_signature(signature, u64::from(component.to_bits()));
    }
    signature
}

fn mix_resource_id(
    mut signature: u64,
    resource_id: zircon_runtime::core::resource::ResourceId,
) -> u64 {
    for byte in resource_id.to_string().bytes() {
        signature = mix_signature(signature, u64::from(byte));
    }
    signature
}

fn mix_shadow(
    mut signature: u64,
    shadow: Option<zircon_runtime::core::framework::render::LightShadowSettings>,
) -> u64 {
    let Some(shadow) = shadow else {
        return mix_signature(signature, 0);
    };
    signature = mix_signature(signature, 1);
    signature = mix_signature(signature, shadow.casts_shadow as u64);
    signature = mix_signature(signature, u64::from(shadow.depth_bias.to_bits()));
    signature = mix_signature(signature, u64::from(shadow.normal_bias.to_bits()));
    signature = mix_signature(signature, u64::from(shadow.strength.to_bits()));
    signature = mix_signature(signature, shadow.resolution_preference as u64);
    mix_signature(signature, shadow.pcf_quality as u64)
}

fn mix_signature(signature: u64, value: u64) -> u64 {
    signature
        .rotate_left(17)
        .wrapping_add(value ^ 0x9E37_79B9_7F4A_7C15)
        .wrapping_mul(0xBF58_476D_1CE4_E5B9)
}

fn classify_surface(
    mode: RenderHybridGiMode,
    mesh: &RenderMeshSnapshot,
    baked_lighting: Option<&LightmapConsumeContract>,
    has_baked_probe_grid: bool,
) -> HybridGiSurfaceParticipation {
    if mode == RenderHybridGiMode::DynamicOnly {
        return HybridGiSurfaceParticipation::DynamicReceiver;
    }

    let has_instance_lightmap = baked_lighting
        .and_then(|contract| contract.slot_for_instance(mesh.stable_instance_key))
        .is_some();
    match mesh.mobility {
        Mobility::Static if has_instance_lightmap || has_baked_probe_grid => {
            HybridGiSurfaceParticipation::BakedStatic
        }
        Mobility::Static => HybridGiSurfaceParticipation::DynamicReceiver,
        Mobility::Dynamic if baked_lighting.is_some() && has_baked_probe_grid => {
            HybridGiSurfaceParticipation::HybridReceiver
        }
        Mobility::Dynamic => HybridGiSurfaceParticipation::DynamicReceiver,
    }
}
