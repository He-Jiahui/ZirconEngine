use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime::core::framework::render::{
    render_mesh_stable_instance_key, render_mesh_transform_revision, LightmapConsumeContract,
    RenderDirectionalLightSnapshot, RenderHybridGiCompositePolicy, RenderHybridGiDebugView,
    RenderHybridGiExtract, RenderHybridGiFallbackReason, RenderHybridGiMode, RenderHybridGiProfile,
    RenderHybridGiQuality, RenderHybridGiResolvedSettings, RenderLayerSet, RenderMeshSnapshot,
    RenderMeshStaticState, RenderPointLightSnapshot, RenderSpotLightSnapshot, RendererCommon,
};
use zircon_runtime::core::framework::scene::Mobility;
use zircon_runtime::core::math::{Transform, Vec3, Vec4};
use zircon_runtime::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};

use super::input_set::HybridGiInputSet;
use super::participation::{HybridGiParticipationState, HybridGiSurfaceParticipation};
use super::radiance_cache_state::HybridGiRadianceCacheState;
use super::screen_probe_state::HybridGiScreenProbeState;
use super::source_ledger::HybridGiSourceLedger;
use super::surface_cache_state::HybridGiSurfaceCacheState;
use super::voxel_scene_state::HybridGiVoxelSceneState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct HybridGiSceneRepresentationSettings {
    enabled: bool,
    mode: RenderHybridGiMode,
    effective_mode: RenderHybridGiMode,
    profile: RenderHybridGiProfile,
    fallback_reason: Option<RenderHybridGiFallbackReason>,
    quality: RenderHybridGiQuality,
    trace_budget: u32,
    card_budget: u32,
    voxel_budget: u32,
    debug_view: RenderHybridGiDebugView,
}

impl HybridGiSceneRepresentationSettings {
    pub(crate) fn trace_budget(&self) -> u32 {
        self.trace_budget
    }

    pub(crate) fn card_budget(&self) -> u32 {
        self.card_budget
    }

    pub(crate) fn voxel_budget(&self) -> u32 {
        self.voxel_budget
    }

    pub(crate) fn mode(&self) -> RenderHybridGiMode {
        self.effective_mode
    }

    pub(crate) fn profile(&self) -> RenderHybridGiProfile {
        self.profile
    }

    pub(crate) fn fallback_reason(&self) -> Option<RenderHybridGiFallbackReason> {
        self.fallback_reason
    }
}

impl Default for HybridGiSceneRepresentationSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: RenderHybridGiMode::DynamicOnly,
            effective_mode: RenderHybridGiMode::DynamicOnly,
            profile: RenderHybridGiProfile::Custom,
            fallback_reason: None,
            quality: RenderHybridGiQuality::Medium,
            trace_budget: 0,
            card_budget: 0,
            voxel_budget: 0,
            debug_view: RenderHybridGiDebugView::None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct HybridGiCardDescriptor {
    card_id: u32,
    stable_instance_key: u64,
    mesh: RenderMeshSnapshot,
    bounds_center: zircon_runtime::core::math::Vec3,
    bounds_radius: f32,
}

impl HybridGiCardDescriptor {
    pub(in crate::hybrid_gi::scene_representation) fn new(
        card_id: u32,
        mesh: RenderMeshSnapshot,
        bounds_center: zircon_runtime::core::math::Vec3,
        bounds_radius: f32,
    ) -> Self {
        Self {
            card_id,
            stable_instance_key: mesh.stable_instance_key,
            mesh,
            bounds_center,
            bounds_radius,
        }
    }

    pub(super) fn card_id(&self) -> u32 {
        self.card_id
    }

    pub(super) fn mesh(&self) -> &RenderMeshSnapshot {
        &self.mesh
    }

    pub(super) fn stable_instance_key(&self) -> u64 {
        self.stable_instance_key
    }

    pub(super) fn bounds_center(&self) -> zircon_runtime::core::math::Vec3 {
        self.bounds_center
    }

    pub(super) fn bounds_radius(&self) -> f32 {
        self.bounds_radius
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::hybrid_gi) struct HybridGiCardCaptureRequest {
    card_id: u32,
    page_id: u32,
    atlas_slot_id: u32,
    capture_slot_id: u32,
    bounds_center: zircon_runtime::core::math::Vec3,
    bounds_radius: f32,
}

impl HybridGiCardCaptureRequest {
    pub(in crate::hybrid_gi) fn card_id(&self) -> u32 {
        self.card_id
    }

    pub(in crate::hybrid_gi) fn page_id(&self) -> u32 {
        self.page_id
    }

    pub(in crate::hybrid_gi) fn atlas_slot_id(&self) -> u32 {
        self.atlas_slot_id
    }

    pub(in crate::hybrid_gi) fn capture_slot_id(&self) -> u32 {
        self.capture_slot_id
    }

    pub(in crate::hybrid_gi) fn bounds_center(&self) -> zircon_runtime::core::math::Vec3 {
        self.bounds_center
    }

    pub(in crate::hybrid_gi) fn bounds_radius(&self) -> f32 {
        self.bounds_radius
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::hybrid_gi) struct HybridGiSceneScreenProbeRuntimeDescriptor {
    probe_id: u32,
    slot: u32,
    stable_instance_key: u64,
    source_mask: u32,
    dynamic_weight_q8: u8,
    bounds_center: Vec3,
    bounds_radius: f32,
    ray_budget: u32,
    irradiance_rgb: [u8; 3],
}

impl HybridGiSceneScreenProbeRuntimeDescriptor {
    pub(in crate::hybrid_gi) fn probe_id(&self) -> u32 {
        self.probe_id
    }

    pub(in crate::hybrid_gi) fn slot(&self) -> u32 {
        self.slot
    }

    pub(in crate::hybrid_gi) fn stable_instance_key(&self) -> u64 {
        self.stable_instance_key
    }

    pub(in crate::hybrid_gi) fn source_mask(&self) -> u32 {
        self.source_mask
    }

    pub(in crate::hybrid_gi) fn dynamic_weight_q8(&self) -> u8 {
        self.dynamic_weight_q8
    }

    pub(in crate::hybrid_gi) fn bounds_center(&self) -> Vec3 {
        self.bounds_center
    }

    pub(in crate::hybrid_gi) fn bounds_radius(&self) -> f32 {
        self.bounds_radius
    }

    pub(in crate::hybrid_gi) fn ray_budget(&self) -> u32 {
        self.ray_budget
    }

    pub(in crate::hybrid_gi) fn irradiance_rgb(&self) -> [u8; 3] {
        self.irradiance_rgb
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HybridGiSceneRepresentation {
    settings: HybridGiSceneRepresentationSettings,
    cards: Vec<HybridGiCardDescriptor>,
    card_capture_requests: Vec<HybridGiCardCaptureRequest>,
    surface_cache: HybridGiSurfaceCacheState,
    screen_probes: HybridGiScreenProbeState,
    radiance_cache: HybridGiRadianceCacheState,
    voxel_scene: HybridGiVoxelSceneState,
    inputs: HybridGiInputSet,
    participation: HybridGiParticipationState,
    source_ledger: HybridGiSourceLedger,
    directional_lights: Vec<RenderDirectionalLightSnapshot>,
    point_lights: Vec<RenderPointLightSnapshot>,
    spot_lights: Vec<RenderSpotLightSnapshot>,
}

impl Default for HybridGiSceneRepresentation {
    fn default() -> Self {
        Self {
            settings: HybridGiSceneRepresentationSettings::default(),
            cards: Vec::new(),
            card_capture_requests: Vec::new(),
            surface_cache: HybridGiSurfaceCacheState::default(),
            screen_probes: HybridGiScreenProbeState::default(),
            radiance_cache: HybridGiRadianceCacheState::default(),
            voxel_scene: HybridGiVoxelSceneState::default(),
            inputs: HybridGiInputSet::deferred(),
            participation: HybridGiParticipationState::default(),
            source_ledger: HybridGiSourceLedger::default(),
            directional_lights: Vec::new(),
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
        }
    }
}

impl HybridGiSceneRepresentation {
    pub(crate) fn settings(&self) -> HybridGiSceneRepresentationSettings {
        self.settings
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn inputs(&self) -> &HybridGiInputSet {
        &self.inputs
    }

    pub(crate) fn surface_cache(&self) -> &HybridGiSurfaceCacheState {
        &self.surface_cache
    }

    pub(crate) fn screen_probe_count(&self) -> usize {
        self.screen_probes.probe_count()
    }

    pub(crate) fn radiance_cache_entry_count(&self) -> usize {
        self.radiance_cache.entry_count()
    }

    pub(in crate::hybrid_gi) fn surface_cache_mut(&mut self) -> &mut HybridGiSurfaceCacheState {
        &mut self.surface_cache
    }

    pub(crate) fn voxel_scene(&self) -> &HybridGiVoxelSceneState {
        &self.voxel_scene
    }

    pub(in crate::hybrid_gi) fn voxel_scene_mut(&mut self) -> &mut HybridGiVoxelSceneState {
        &mut self.voxel_scene
    }

    pub(in crate::hybrid_gi) fn card_bounds_by_id(
        &self,
    ) -> BTreeMap<u32, (zircon_runtime::core::math::Vec3, f32)> {
        self.cards
            .iter()
            .map(|card| (card.card_id, (card.bounds_center, card.bounds_radius)))
            .collect()
    }

    pub(in crate::hybrid_gi) fn card_capture_request_descriptors(
        &self,
    ) -> &[HybridGiCardCaptureRequest] {
        &self.card_capture_requests
    }

    pub(in crate::hybrid_gi) fn screen_probe_runtime_descriptors(
        &self,
    ) -> Vec<HybridGiSceneScreenProbeRuntimeDescriptor> {
        self.screen_probes
            .descriptors()
            .iter()
            .enumerate()
            .map(|(slot, probe)| HybridGiSceneScreenProbeRuntimeDescriptor {
                probe_id: probe.probe_id(),
                slot: slot as u32,
                stable_instance_key: probe.stable_instance_key(),
                source_mask: self
                    .source_ledger
                    .surface_source_mask(probe.stable_instance_key())
                    .unwrap_or_default(),
                dynamic_weight_q8: self
                    .source_ledger
                    .surface_dynamic_weight_q8(probe.stable_instance_key()),
                bounds_center: probe.bounds_center(),
                bounds_radius: probe.bounds_radius(),
                ray_budget: probe.ray_budget(),
                irradiance_rgb: self
                    .radiance_cache
                    .radiance_rgb(probe.probe_id())
                    .unwrap_or([0, 0, 0]),
            })
            .collect()
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn from_extract(extract: &RenderHybridGiExtract) -> Self {
        let mut representation = Self::default();
        representation.apply_extract(extract);
        representation
    }

    pub(crate) fn apply_extract(&mut self, extract: &RenderHybridGiExtract) {
        let resolved = extract.resolved_settings(true);
        self.settings = HybridGiSceneRepresentationSettings {
            enabled: extract.enabled,
            mode: resolved.mode,
            effective_mode: resolved.mode,
            profile: resolved.profile,
            fallback_reason: resolved.fallback_reason,
            quality: resolved.quality,
            trace_budget: resolved.trace_budget,
            card_budget: resolved.card_budget,
            voxel_budget: resolved.voxel_budget,
            debug_view: extract.debug_view,
        };
        self.inputs = HybridGiInputSet::deferred();
    }

    pub(crate) fn synchronize_scene(
        &mut self,
        meshes: &[RenderMeshSnapshot],
        directional_lights: &[RenderDirectionalLightSnapshot],
        point_lights: &[RenderPointLightSnapshot],
        spot_lights: &[RenderSpotLightSnapshot],
    ) {
        self.synchronize_scene_with_baked(
            meshes,
            directional_lights,
            point_lights,
            spot_lights,
            None,
            false,
        );
    }

    pub(crate) fn synchronize_scene_with_baked(
        &mut self,
        meshes: &[RenderMeshSnapshot],
        directional_lights: &[RenderDirectionalLightSnapshot],
        point_lights: &[RenderPointLightSnapshot],
        spot_lights: &[RenderSpotLightSnapshot],
        baked_lighting: Option<&LightmapConsumeContract>,
        has_baked_probe_grid: bool,
    ) {
        self.settings.effective_mode = if self.settings.mode
            == RenderHybridGiMode::BakedStaticDynamic
            && baked_lighting.is_none()
        {
            self.settings.fallback_reason =
                Some(RenderHybridGiFallbackReason::BakedLightingUnavailable);
            RenderHybridGiMode::DynamicOnly
        } else {
            self.settings.fallback_reason = None;
            self.settings.mode
        };
        self.participation.synchronize(
            self.settings.effective_mode,
            meshes,
            directional_lights,
            point_lights,
            spot_lights,
            baked_lighting,
            has_baked_probe_grid,
        );
        self.source_ledger
            .synchronize(self.settings.effective_mode, &self.participation);
        let cards = build_card_descriptors(meshes);
        let dynamic_delta_only = self.settings.effective_mode
            == RenderHybridGiMode::BakedStaticDynamic
            && baked_lighting.is_some();
        let directional_lights = sorted_directional_lights(directional_lights)
            .into_iter()
            .filter(|light| !dynamic_delta_only || light.mobility == Mobility::Dynamic)
            .collect::<Vec<_>>();
        let point_lights = sorted_point_lights(point_lights)
            .into_iter()
            .filter(|light| !dynamic_delta_only || light.mobility == Mobility::Dynamic)
            .collect::<Vec<_>>();
        let spot_lights = sorted_spot_lights(spot_lights)
            .into_iter()
            .filter(|light| !dynamic_delta_only || light.mobility == Mobility::Dynamic)
            .collect::<Vec<_>>();
        let cards_changed = self.cards != cards;
        let lights_changed = self.directional_lights != directional_lights
            || self.point_lights != point_lights
            || self.spot_lights != spot_lights;
        let changed_card_ids = changed_card_ids(&self.cards, &cards);
        let active_card_ids = cards.iter().map(|card| card.card_id).collect::<Vec<_>>();

        self.surface_cache
            .synchronize(&active_card_ids, self.settings.card_budget as usize);
        self.surface_cache.mark_dirty_owner_cards(changed_card_ids);
        if lights_changed {
            self.surface_cache.mark_all_resident_pages_dirty();
        }
        let dirty_page_ids = self.surface_cache.dirty_page_ids_snapshot();
        let surface_cache_page_contents = self.surface_cache.page_contents_snapshot();
        self.voxel_scene.synchronize(
            &cards,
            &directional_lights,
            &point_lights,
            &spot_lights,
            &surface_cache_page_contents,
            &dirty_page_ids,
            self.settings.voxel_budget as usize,
            cards_changed || lights_changed,
        );
        self.screen_probes.synchronize(
            &cards,
            &self.surface_cache,
            self.settings.trace_budget as usize,
        );
        self.radiance_cache.synchronize(
            self.screen_probes.descriptors(),
            &self.surface_cache,
            &self.voxel_scene,
        );

        self.card_capture_requests = build_card_capture_requests(&cards, &self.surface_cache);
        self.cards = cards;
        self.directional_lights = directional_lights;
        self.point_lights = point_lights;
        self.spot_lights = spot_lights;
    }

    #[cfg(test)]
    pub(crate) fn participation_epoch(&self) -> u64 {
        self.participation.participation_epoch()
    }

    #[cfg(test)]
    pub(crate) fn baked_light_set_generation(&self) -> Option<u64> {
        self.participation.light_set_generation()
    }

    #[cfg(test)]
    pub(crate) fn surface_participation(
        &self,
        stable_instance_key: u64,
    ) -> Option<HybridGiSurfaceParticipation> {
        self.participation.surface(stable_instance_key)
    }

    pub(crate) fn composite_policy(&self) -> RenderHybridGiCompositePolicy {
        self.source_ledger.composite_policy()
    }

    pub(crate) fn resolved_settings(&self) -> RenderHybridGiResolvedSettings {
        RenderHybridGiResolvedSettings {
            mode: self.settings.effective_mode,
            profile: self.settings.profile,
            quality: self.settings.quality,
            trace_budget: self.settings.trace_budget,
            card_budget: self.settings.card_budget,
            voxel_budget: self.settings.voxel_budget,
            fallback_reason: self.settings.fallback_reason,
        }
    }

    #[cfg(test)]
    pub(crate) fn surface_source_mask(&self, stable_instance_key: u64) -> Option<u32> {
        self.source_ledger.surface_source_mask(stable_instance_key)
    }

    #[cfg(test)]
    pub(crate) fn directional_light_ids(&self) -> Vec<u64> {
        self.directional_lights
            .iter()
            .map(|light| light.light_id)
            .collect()
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn synchronize_cards(&mut self, card_ids: impl IntoIterator<Item = u32>) {
        let meshes = card_ids
            .into_iter()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(placeholder_mesh)
            .collect::<Vec<_>>();
        self.synchronize_scene(&meshes, &[], &[], &[]);
    }

    #[cfg(test)]
    pub(crate) fn card_ids(&self) -> Vec<u32> {
        self.cards.iter().map(|card| card.card_id).collect()
    }

    pub(crate) fn card_count(&self) -> usize {
        self.cards.len()
    }

    pub(crate) fn card_capture_request_count(&self) -> usize {
        self.card_capture_requests.len()
    }

    #[cfg(test)]
    pub(crate) fn screen_probe_descriptors(
        &self,
    ) -> Vec<(u32, u32, Option<u32>, [f32; 3], f32, u32)> {
        self.screen_probes
            .descriptors()
            .iter()
            .map(|probe| {
                (
                    probe.probe_id(),
                    probe.card_id(),
                    probe.surface_page_id(),
                    [
                        probe.bounds_center().x,
                        probe.bounds_center().y,
                        probe.bounds_center().z,
                    ],
                    probe.bounds_radius(),
                    probe.ray_budget(),
                )
            })
            .collect()
    }

    #[cfg(test)]
    pub(crate) fn radiance_cache_entries(
        &self,
    ) -> Vec<(u32, u32, Option<u32>, [u8; 3], u8, &'static str)> {
        self.radiance_cache.entries()
    }

    #[cfg(test)]
    pub(crate) fn radiance_cache_clipmap_topology(&self) -> Vec<(u32, u32, f32)> {
        self.radiance_cache.clipmap_topology()
    }

    #[cfg(test)]
    pub(crate) fn radiance_cache_probe_demands(&self) -> Vec<(u32, [i32; 3])> {
        self.radiance_cache.probe_demands()
    }

    #[cfg(test)]
    pub(crate) fn card_capture_requests(&self) -> Vec<(u32, u32, u32, u32, [f32; 3], f32)> {
        self.card_capture_requests
            .iter()
            .map(|request| {
                (
                    request.card_id,
                    request.page_id,
                    request.atlas_slot_id,
                    request.capture_slot_id,
                    [
                        request.bounds_center.x,
                        request.bounds_center.y,
                        request.bounds_center.z,
                    ],
                    request.bounds_radius,
                )
            })
            .collect()
    }
}

fn build_card_descriptors(meshes: &[RenderMeshSnapshot]) -> Vec<HybridGiCardDescriptor> {
    let mut meshes = meshes.to_vec();
    meshes.sort_by_key(|mesh| mesh.stable_instance_key);
    let mut used_card_ids = BTreeSet::new();
    meshes
        .into_iter()
        .map(|mesh| {
            let card_id = unique_card_id(&mesh, &used_card_ids);
            used_card_ids.insert(card_id);
            HybridGiCardDescriptor::new(
                card_id,
                mesh.clone(),
                mesh.transform.translation,
                card_bounds_radius(&mesh),
            )
        })
        .collect()
}

fn unique_card_id(mesh: &RenderMeshSnapshot, used_card_ids: &BTreeSet<u32>) -> u32 {
    let preferred = mesh.node_id as u32;
    if !used_card_ids.contains(&preferred) {
        return preferred;
    }

    let mut candidate = (mesh.stable_instance_key as u32)
        ^ ((mesh.stable_instance_key >> 32) as u32).rotate_left(13)
        ^ 0x8000_0000;
    while used_card_ids.contains(&candidate) {
        candidate = candidate.wrapping_add(0x9E37_79B9);
    }
    candidate
}

fn card_bounds_radius(mesh: &RenderMeshSnapshot) -> f32 {
    (mesh.transform.scale.abs().max_element() * 0.5).max(0.5)
}

fn changed_card_ids(
    previous_cards: &[HybridGiCardDescriptor],
    next_cards: &[HybridGiCardDescriptor],
) -> Vec<u32> {
    let previous_cards_by_id = previous_cards
        .iter()
        .map(|card| (card.card_id, &card.mesh))
        .collect::<BTreeMap<_, _>>();
    next_cards
        .iter()
        .filter_map(|card| match previous_cards_by_id.get(&card.card_id) {
            Some(previous_mesh) if **previous_mesh == card.mesh => None,
            _ => Some(card.card_id),
        })
        .collect()
}

fn build_card_capture_requests(
    cards: &[HybridGiCardDescriptor],
    surface_cache: &HybridGiSurfaceCacheState,
) -> Vec<HybridGiCardCaptureRequest> {
    let cards_by_id = cards
        .iter()
        .map(|card| (card.card_id, card))
        .collect::<BTreeMap<_, _>>();
    let atlas_slots_by_page_id = surface_cache
        .page_table_entries_snapshot()
        .into_iter()
        .collect::<BTreeMap<_, _>>();
    let owner_card_ids_by_page_id = surface_cache
        .owner_card_ids_by_page_id_snapshot()
        .into_iter()
        .collect::<BTreeMap<_, _>>();

    surface_cache
        .capture_atlas_entries_snapshot()
        .into_iter()
        .filter_map(|(page_id, capture_slot_id)| {
            let atlas_slot_id = atlas_slots_by_page_id.get(&page_id).copied()?;
            let owner_card_id = owner_card_ids_by_page_id.get(&page_id).copied()?;
            let card = cards_by_id.get(&owner_card_id)?;
            Some(HybridGiCardCaptureRequest {
                card_id: card.card_id,
                page_id,
                atlas_slot_id,
                capture_slot_id,
                bounds_center: card.bounds_center,
                bounds_radius: card.bounds_radius,
            })
        })
        .collect()
}

fn sorted_directional_lights(
    lights: &[RenderDirectionalLightSnapshot],
) -> Vec<RenderDirectionalLightSnapshot> {
    let mut lights = lights.to_vec();
    lights.sort_by_key(|light| light.node_id);
    lights
}

fn sorted_point_lights(lights: &[RenderPointLightSnapshot]) -> Vec<RenderPointLightSnapshot> {
    let mut lights = lights.to_vec();
    lights.sort_by_key(|light| light.node_id);
    lights
}

fn sorted_spot_lights(lights: &[RenderSpotLightSnapshot]) -> Vec<RenderSpotLightSnapshot> {
    let mut lights = lights.to_vec();
    lights.sort_by_key(|light| light.node_id);
    lights
}

fn placeholder_mesh(card_id: u32) -> RenderMeshSnapshot {
    let node_id = u64::from(card_id);
    let transform = Transform::identity();
    RenderMeshSnapshot {
        node_id,
        stable_instance_key: render_mesh_stable_instance_key(node_id, 0),
        transform_revision: render_mesh_transform_revision(&transform),
        transform,
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(&format!(
            "builtin://hybrid-gi/card/{card_id}/model"
        ))),
        mesh: None,
        material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(&format!(
            "builtin://hybrid-gi/card/{card_id}/material"
        ))),
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
    }
}
