use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime::core::framework::render::{
    RenderDirectionalLightSnapshot, RenderHybridGiDebugView, RenderHybridGiExtract,
    RenderHybridGiQuality, RenderMeshSnapshot, RenderPointLightSnapshot, RenderSpotLightSnapshot,
};
use zircon_runtime::core::framework::scene::Mobility;
use zircon_runtime::core::math::{Transform, Vec4};
use zircon_runtime::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};
use zircon_runtime::graphics::hybrid_gi_extract_sources::{
    hybrid_gi_extract_probe_records, hybrid_gi_extract_trace_region_records,
};

use super::input_set::HybridGiInputSet;
use super::radiance_cache_state::HybridGiRadianceCacheState;
use super::screen_probe_state::HybridGiScreenProbeState;
use super::surface_cache_state::HybridGiSurfaceCacheState;
use super::voxel_scene_state::HybridGiVoxelSceneState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct HybridGiSceneRepresentationSettings {
    enabled: bool,
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
}

impl Default for HybridGiSceneRepresentationSettings {
    fn default() -> Self {
        Self {
            enabled: false,
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
    directional_lights: Vec<RenderDirectionalLightSnapshot>,
    point_lights: Vec<RenderPointLightSnapshot>,
    spot_lights: Vec<RenderSpotLightSnapshot>,
    fixture_probe_count: usize,
    fixture_trace_region_count: usize,
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
            directional_lights: Vec::new(),
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
            fixture_probe_count: 0,
            fixture_trace_region_count: 0,
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

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn from_extract(extract: &RenderHybridGiExtract) -> Self {
        let mut representation = Self::default();
        representation.apply_extract(extract);
        representation
    }

    pub(crate) fn apply_extract(&mut self, extract: &RenderHybridGiExtract) {
        self.settings = HybridGiSceneRepresentationSettings {
            enabled: extract.enabled,
            quality: extract.quality,
            trace_budget: extract.trace_budget,
            card_budget: extract.card_budget,
            voxel_budget: extract.voxel_budget,
            debug_view: extract.debug_view,
        };
        self.inputs = HybridGiInputSet::deferred();
        if !extract.enabled {
            self.fixture_probe_count = 0;
            self.fixture_trace_region_count = 0;
            return;
        }
        self.fixture_probe_count = hybrid_gi_extract_probe_records(extract).len();
        self.fixture_trace_region_count = hybrid_gi_extract_trace_region_records(extract).len();
    }

    pub(crate) fn synchronize_scene(
        &mut self,
        meshes: &[RenderMeshSnapshot],
        directional_lights: &[RenderDirectionalLightSnapshot],
        point_lights: &[RenderPointLightSnapshot],
        spot_lights: &[RenderSpotLightSnapshot],
    ) {
        let cards = build_card_descriptors(meshes);
        let directional_lights = sorted_directional_lights(directional_lights);
        let point_lights = sorted_point_lights(point_lights);
        let spot_lights = sorted_spot_lights(spot_lights);
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
    pub(crate) fn fixture_probe_count(&self) -> usize {
        self.fixture_probe_count
    }

    #[cfg(test)]
    pub(crate) fn fixture_trace_region_count(&self) -> usize {
        self.fixture_trace_region_count
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
    let mut cards = BTreeMap::new();
    for mesh in meshes {
        cards.insert(
            mesh.node_id as u32,
            HybridGiCardDescriptor::new(
                mesh.node_id as u32,
                mesh.clone(),
                mesh.transform.translation,
                card_bounds_radius(mesh),
            ),
        );
    }
    cards.into_values().collect()
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
    RenderMeshSnapshot {
        node_id: card_id as u64,
        transform: Transform::identity(),
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(&format!(
            "builtin://hybrid-gi/card/{card_id}/model"
        ))),
        material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(&format!(
            "builtin://hybrid-gi/card/{card_id}/material"
        ))),
        tint: Vec4::ONE,
        mobility: Mobility::Static,
        render_layer_mask: u32::MAX,
    }
}
