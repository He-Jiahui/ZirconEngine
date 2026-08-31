use std::collections::HashMap;

use crate::core::framework::render::{
    CastShadowsMode, GEOMETRY_SOURCE_ID_MORPHED_MESH, GEOMETRY_SOURCE_ID_SKINNED_MESH,
    GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH, GEOMETRY_SOURCE_ID_STATIC_MESH, GeometrySourceId,
    RenderViewportPickPolicy, ShaderQualityTier,
};
use crate::core::resource::ResourceId;
use crate::graphics::scene::resources::{
    MaterialDisabledPasses, MaterialDrawGenerationSelection, MaterialRuntime, PipelineKey,
    ResourceStreamer, default_pipeline_key,
};
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshPassPipelineKind;
use crate::graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::PipelineCreationTarget;
use crate::graphics::scene::scene_renderer::mesh::{
    MaterialPipelineRequirement, MaterialPipelineRequirementSet,
};

use super::super::super::mesh_draw::{MeshDrawQueuePhase, MeshDrawQueueProfile};
use super::geometry_source_selection::pending_mesh_geometry_source;
use super::material_draw_selection::MaterialDrawSelection;
use super::pending_mesh_draw::PendingMeshDraw;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct MaterialPipelineCensusOwner {
    material_id: ResourceId,
    generation: u64,
}

#[derive(Default)]
pub(crate) struct MaterialPipelineRequirementCensus {
    rows: HashMap<MaterialPipelineCensusOwner, MaterialPipelineRequirementRow>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct MaterialPipelineDrawContext {
    geometry: MaterialPipelineGeometryContext,
    velocity_history_eligible: bool,
    shadow: MaterialPipelineShadowContext,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
enum MaterialPipelineGeometryContext {
    Static = 0,
    Skinned = 1,
    Morphed = 2,
    SkinnedMorphed = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
enum MaterialPipelineShadowContext {
    Disabled = 0,
    OneSided = 1,
    ForcedTwoSided = 2,
}

impl MaterialPipelineShadowContext {
    const fn from_modes(renderer_mode: CastShadowsMode, material_casts_shadows: bool) -> Self {
        if !material_casts_shadows || matches!(renderer_mode, CastShadowsMode::Off) {
            Self::Disabled
        } else if matches!(renderer_mode, CastShadowsMode::TwoSided) {
            Self::ForcedTwoSided
        } else {
            Self::OneSided
        }
    }

    const fn casts_shadows(self) -> bool {
        !matches!(self, Self::Disabled)
    }

    fn effective_shadow_pipeline_key(self, source: &PipelineKey) -> PipelineKey {
        let mut pipeline_key = source.clone();
        if matches!(self, Self::ForcedTwoSided) {
            pipeline_key.double_sided = true;
        }
        pipeline_key
    }
}

impl MaterialPipelineGeometryContext {
    fn from_geometry_source_id(geometry_source: GeometrySourceId) -> Self {
        if geometry_source == GEOMETRY_SOURCE_ID_STATIC_MESH {
            Self::Static
        } else if geometry_source == GEOMETRY_SOURCE_ID_SKINNED_MESH {
            Self::Skinned
        } else if geometry_source == GEOMETRY_SOURCE_ID_MORPHED_MESH {
            Self::Morphed
        } else if geometry_source == GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH {
            Self::SkinnedMorphed
        } else {
            unreachable!("pending mesh draw produced an unknown shader geometry source")
        }
    }

    const fn geometry_source_id(self) -> GeometrySourceId {
        match self {
            Self::Static => GEOMETRY_SOURCE_ID_STATIC_MESH,
            Self::Skinned => GEOMETRY_SOURCE_ID_SKINNED_MESH,
            Self::Morphed => GEOMETRY_SOURCE_ID_MORPHED_MESH,
            Self::SkinnedMorphed => GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH,
        }
    }
}

#[derive(Default)]
struct MaterialPipelineObservedContexts {
    bits: u32,
}

impl MaterialPipelineObservedContexts {
    fn insert(&mut self, context: MaterialPipelineDrawContext) -> bool {
        let bit = 1u32 << context.bit_index();
        let inserted = self.bits & bit == 0;
        self.bits |= bit;
        inserted
    }

    fn len(&self) -> usize {
        self.bits.count_ones() as usize
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct MaterialPipelineRequirementCensusStats {
    observed_draw_count: usize,
    unique_context_count: usize,
    candidate_resolution_count: usize,
}

#[derive(Default)]
pub(super) struct PublishedMaterialPipelineRequirementCollector {
    census: MaterialPipelineRequirementCensus,
}

struct MaterialPipelineRequirementRow {
    candidate: MaterialPipelineCandidate,
    observed_contexts: MaterialPipelineObservedContexts,
    requirements: MaterialPipelineRequirementSet,
}

impl MaterialPipelineRequirementRow {
    fn new(candidate: MaterialPipelineCandidate) -> Self {
        Self {
            candidate,
            observed_contexts: MaterialPipelineObservedContexts::default(),
            requirements: MaterialPipelineRequirementSet::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct MaterialPipelineFeatureSet {
    depth_prepass: bool,
    deferred_gbuffer: bool,
    base_opaque: bool,
    base_alpha_mask: bool,
    base_transparent: bool,
    advanced_pbr_opaque: bool,
    transmission: bool,
    shadow: bool,
    velocity: bool,
    taa_reactive: bool,
    oit: bool,
    hit_proxy: bool,
    hit_proxy_translucent: bool,
    hit_proxy_backfaces: bool,
    reverse_view_raster_winding: bool,
}

#[derive(Clone)]
struct MaterialPipelineInputs {
    pipeline_key: PipelineKey,
    disabled_passes: MaterialDisabledPasses,
    taa_reactive_mask_strength: f32,
}

#[derive(Clone)]
struct MaterialPipelineCandidate {
    inputs: MaterialPipelineInputs,
    cast_shadows: bool,
}

impl MaterialPipelineCandidate {
    fn from_runtime(runtime: &MaterialRuntime) -> Self {
        Self {
            inputs: MaterialPipelineInputs {
                pipeline_key: runtime.pipeline_key.clone(),
                disabled_passes: runtime.disabled_passes,
                taa_reactive_mask_strength: runtime.taa_reactive_mask_strength,
            },
            cast_shadows: runtime.cast_shadows,
        }
    }

    fn error_proxy() -> Self {
        Self {
            inputs: MaterialPipelineInputs {
                pipeline_key: default_pipeline_key(),
                disabled_passes: MaterialDisabledPasses::default(),
                taa_reactive_mask_strength: 0.0,
            },
            cast_shadows: true,
        }
    }

    fn from_published_draw(pending_draw: &PendingMeshDraw) -> Self {
        Self {
            inputs: MaterialPipelineInputs {
                pipeline_key: pending_draw.material.pipeline_key.clone(),
                disabled_passes: pending_draw.material.disabled_passes,
                taa_reactive_mask_strength: pending_draw.material.taa_reactive_mask_strength,
            },
            cast_shadows: pending_draw.material.common.cast_shadows != CastShadowsMode::Off,
        }
    }
}

impl MaterialPipelineRequirementCensus {
    fn new() -> Self {
        Self::default()
    }

    pub(crate) fn len(&self) -> usize {
        self.rows.len()
    }

    pub(crate) fn remove(
        &mut self,
        material_id: ResourceId,
        generation: u64,
    ) -> Option<MaterialPipelineRequirementSet> {
        self.rows
            .remove(&MaterialPipelineCensusOwner {
                material_id,
                generation,
            })
            .map(|row| row.requirements)
    }

    pub(crate) fn retain(
        &mut self,
        mut retain: impl FnMut(&ResourceId, u64, &MaterialPipelineRequirementSet) -> bool,
    ) {
        self.rows
            .retain(|owner, row| retain(&owner.material_id, owner.generation, &row.requirements));
    }

    pub(crate) fn into_requirements(
        self,
    ) -> impl Iterator<Item = (ResourceId, u64, MaterialPipelineRequirementSet)> {
        self.rows
            .into_iter()
            .map(|(owner, row)| (owner.material_id, owner.generation, row.requirements))
    }
}

impl MaterialPipelineDrawContext {
    const fn bit_index(self) -> u32 {
        (self.geometry as u32) * 6
            + (self.velocity_history_eligible as u32) * 3
            + self.shadow as u32
    }

    fn from_pending_draw(
        pending_draw: &PendingMeshDraw,
        pipeline_key: &PipelineKey,
        material_casts_shadows: bool,
    ) -> Self {
        let skinned_gpu_skinning_enabled =
            pipeline_key.uses_fallback_shader() && pending_draw.skinned_gpu_source.is_some();
        let geometry_source = pending_mesh_geometry_source(
            &pending_draw.mesh,
            pending_draw.skinned_gpu_source.as_ref(),
            skinned_gpu_skinning_enabled,
            pending_draw.morph_payload.is_some(),
        );
        let phase = MeshDrawQueuePhase::from_pipeline_flags(
            pipeline_key.is_transparent(),
            pipeline_key.is_alpha_mask(),
        );
        let queue_profile = MeshDrawQueueProfile::new(
            phase,
            geometry_source,
            pending_draw.mobility,
            pending_draw.indirect_draw_ref.is_some(),
            skinned_gpu_skinning_enabled,
            pending_draw.mesh_lod.is_some(),
        );
        Self {
            geometry: MaterialPipelineGeometryContext::from_geometry_source_id(
                queue_profile.shader_geometry_source_id(),
            ),
            velocity_history_eligible: queue_profile.velocity_history_eligible(),
            shadow: MaterialPipelineShadowContext::from_modes(
                pending_draw.material.renderer_cast_shadows,
                material_casts_shadows,
            ),
        }
    }
}

impl PublishedMaterialPipelineRequirementCollector {
    pub(super) fn observe_published_draw(
        &mut self,
        pending_draw: &PendingMeshDraw,
        features: MaterialPipelineFeatureSet,
        shader_quality: ShaderQualityTier,
        volumetric_fog_enabled: bool,
    ) {
        let Some(draw_generation) = pending_draw.material.draw_generation else {
            return;
        };
        let material_id = pending_draw.material.resource_id;
        let context = MaterialPipelineDrawContext::from_pending_draw(
            pending_draw,
            &pending_draw.material.pipeline_key,
            pending_draw.material.common.cast_shadows != CastShadowsMode::Off,
        );
        let owner = MaterialPipelineCensusOwner {
            material_id,
            generation: draw_generation,
        };
        let row = self.census.rows.entry(owner).or_insert_with(|| {
            MaterialPipelineRequirementRow::new(MaterialPipelineCandidate::from_published_draw(
                pending_draw,
            ))
        });
        if !row.observed_contexts.insert(context) {
            return;
        }

        insert_material_pipeline_requirements(
            &mut row.requirements,
            context,
            &row.candidate.inputs,
            features,
            shader_quality,
            volumetric_fog_enabled,
        );
    }

    pub(super) fn finish(self, observed_draw_count: usize) -> MaterialPipelineRequirementCensus {
        let unique_context_count = self
            .census
            .rows
            .values()
            .map(|row| row.observed_contexts.len())
            .sum::<usize>();
        let candidate_resolution_count = self.census.rows.len();
        crate::profile_counter!(
            "render",
            "material_current_requirement_observed_draw_count",
            observed_draw_count
        );
        crate::profile_counter!(
            "render",
            "material_current_requirement_unique_context_count",
            unique_context_count
        );
        crate::profile_counter!(
            "render",
            "material_current_requirement_candidate_resolution_count",
            candidate_resolution_count
        );
        let _ = (
            observed_draw_count,
            unique_context_count,
            candidate_resolution_count,
        );
        self.census
    }
}

impl MaterialPipelineFeatureSet {
    pub(crate) fn direct(shadow: bool) -> Self {
        Self {
            base_opaque: true,
            base_alpha_mask: true,
            base_transparent: true,
            advanced_pbr_opaque: true,
            transmission: true,
            shadow,
            ..Self::default()
        }
    }

    pub(crate) fn environment_capture() -> Self {
        Self {
            base_opaque: true,
            base_alpha_mask: true,
            advanced_pbr_opaque: true,
            reverse_view_raster_winding: true,
            ..Self::default()
        }
    }

    pub(crate) const fn reverses_view_raster_winding(self) -> bool {
        self.reverse_view_raster_winding
    }

    pub(crate) fn hit_proxy(policy: RenderViewportPickPolicy) -> Self {
        Self {
            hit_proxy: true,
            hit_proxy_translucent: policy.includes_translucent(),
            hit_proxy_backfaces: policy.includes_backfaces(),
            ..Self::default()
        }
    }

    pub(crate) fn from_executor_ids<'a>(
        executor_ids: impl IntoIterator<Item = Option<&'a str>>,
    ) -> Self {
        let mut features = Self::default();
        for executor_id in executor_ids.into_iter().flatten() {
            match executor_id {
                "mesh.depth-prepass" | "deferred.depth-prepass" => {
                    features.depth_prepass = true;
                }
                "deferred.gbuffer" => features.deferred_gbuffer = true,
                "mesh.opaque" => features.base_opaque = true,
                "mesh.alpha-mask" => features.base_alpha_mask = true,
                "mesh.transparent" | "mesh.halfres-transparent" => {
                    features.base_transparent = true;
                }
                "mesh.advanced-pbr-opaque" => features.advanced_pbr_opaque = true,
                "shadow.atlas" => features.shadow = true,
                "temporal.velocity-object" => features.velocity = true,
                "temporal.taa-reactive-mask-mesh" => features.taa_reactive = true,
                "oit.fragment_store" => features.oit = true,
                _ if crate::graphics::pipeline::transmission_mesh_step_index(executor_id)
                    .is_some() =>
                {
                    features.transmission = true;
                }
                _ => {}
            }
        }
        features
    }

    fn base_enabled(self, pipeline_key: &PipelineKey, phase: MeshDrawQueuePhase) -> bool {
        if pipeline_key.pbr_transmission {
            return self.transmission;
        }
        if pipeline_key.requires_forward_path()
            && matches!(
                phase,
                MeshDrawQueuePhase::Opaque | MeshDrawQueuePhase::AlphaMask
            )
        {
            return self.advanced_pbr_opaque;
        }
        match phase {
            MeshDrawQueuePhase::Opaque => self.base_opaque,
            MeshDrawQueuePhase::AlphaMask => self.base_alpha_mask,
            MeshDrawQueuePhase::Transparent => self.base_transparent,
        }
    }
}

pub(super) fn collect_material_pipeline_requirements(
    pending_draws: &[PendingMeshDraw],
    streamer: &ResourceStreamer,
    features: MaterialPipelineFeatureSet,
    shader_quality: ShaderQualityTier,
    volumetric_fog_enabled: bool,
) -> MaterialPipelineRequirementCensus {
    crate::profile_scope!("render", "material", "staged_requirement_census");
    if !streamer.has_active_staged_material_candidates() {
        profile_material_pipeline_census_stats(
            MaterialPipelineRequirementCensusKind::Staged,
            MaterialPipelineRequirementCensusStats::default(),
        );
        return MaterialPipelineRequirementCensus::new();
    }
    collect_material_pipeline_requirements_for(
        pending_draws,
        |material_id| {
            let runtime = streamer.staged_material_candidate(material_id)?;
            let generation = streamer.staged_material_draw_generation(material_id)?;
            Some((generation, MaterialPipelineCandidate::from_runtime(runtime)))
        },
        features,
        shader_quality,
        volumetric_fog_enabled,
        MaterialPipelineRequirementCensusKind::Staged,
    )
}

pub(super) fn collect_previous_context_pipeline_requirements(
    pending_draws: &[PendingMeshDraw],
    streamer: &ResourceStreamer,
    selection: &MaterialDrawSelection,
    features: MaterialPipelineFeatureSet,
    shader_quality: ShaderQualityTier,
    volumetric_fog_enabled: bool,
) -> MaterialPipelineRequirementCensus {
    crate::profile_scope!("render", "material", "previous_requirement_census");
    if !selection.has_previous_proxies() {
        profile_material_pipeline_census_stats(
            MaterialPipelineRequirementCensusKind::Previous,
            MaterialPipelineRequirementCensusStats::default(),
        );
        return MaterialPipelineRequirementCensus::new();
    }
    collect_material_pipeline_requirements_for(
        pending_draws,
        |material_id| {
            if selection.selection_for(material_id)
                != MaterialDrawGenerationSelection::PreviousPublished
            {
                return None;
            }
            let proxy = selection.proxy(streamer, material_id);
            Some((
                proxy.draw_generation()?,
                MaterialPipelineCandidate::from_runtime(proxy.runtime()?),
            ))
        },
        features,
        shader_quality,
        volumetric_fog_enabled,
        MaterialPipelineRequirementCensusKind::Previous,
    )
}

pub(super) fn collect_error_proxy_context_pipeline_requirements(
    pending_draws: &[PendingMeshDraw],
    streamer: &ResourceStreamer,
    selection: &MaterialDrawSelection,
    features: MaterialPipelineFeatureSet,
    shader_quality: ShaderQualityTier,
    volumetric_fog_enabled: bool,
) -> MaterialPipelineRequirementSet {
    crate::profile_scope!("render", "material", "error_proxy_requirement_census");
    let mut requirements = MaterialPipelineRequirementSet::default();
    if !selection.has_error_proxies() && !streamer.has_active_staged_material_candidates() {
        profile_error_proxy_pipeline_census_stats(MaterialPipelineRequirementCensusStats::default());
        return requirements;
    }
    let error_proxy = MaterialPipelineCandidate::error_proxy();
    let mut material_uses_error_proxy = HashMap::new();
    let mut observed_contexts = MaterialPipelineObservedContexts::default();
    for pending_draw in pending_draws {
        let material_id = pending_draw.material.resource_id;
        let uses_error_proxy = *material_uses_error_proxy
            .entry(material_id)
            .or_insert_with(|| selection.proxy(streamer, &material_id).runtime().is_none());
        if !uses_error_proxy {
            continue;
        }
        let context = MaterialPipelineDrawContext::from_pending_draw(
            pending_draw,
            &error_proxy.inputs.pipeline_key,
            error_proxy.cast_shadows,
        );
        if !observed_contexts.insert(context) {
            continue;
        }
        insert_material_pipeline_requirements(
            &mut requirements,
            context,
            &error_proxy.inputs,
            features,
            shader_quality,
            volumetric_fog_enabled,
        );
    }
    profile_error_proxy_pipeline_census_stats(MaterialPipelineRequirementCensusStats {
        observed_draw_count: pending_draws.len(),
        unique_context_count: observed_contexts.len(),
        candidate_resolution_count: material_uses_error_proxy.len(),
    });
    requirements
}

#[derive(Clone, Copy)]
enum MaterialPipelineRequirementCensusKind {
    Staged,
    Previous,
}

fn profile_material_pipeline_census_stats(
    kind: MaterialPipelineRequirementCensusKind,
    stats: MaterialPipelineRequirementCensusStats,
) {
    match kind {
        MaterialPipelineRequirementCensusKind::Staged => {
            crate::profile_counter!(
                "render",
                "material_staged_requirement_observed_draw_count",
                stats.observed_draw_count
            );
            crate::profile_counter!(
                "render",
                "material_staged_requirement_unique_context_count",
                stats.unique_context_count
            );
            crate::profile_counter!(
                "render",
                "material_staged_requirement_candidate_resolution_count",
                stats.candidate_resolution_count
            );
        }
        MaterialPipelineRequirementCensusKind::Previous => {
            crate::profile_counter!(
                "render",
                "material_previous_requirement_observed_draw_count",
                stats.observed_draw_count
            );
            crate::profile_counter!(
                "render",
                "material_previous_requirement_unique_context_count",
                stats.unique_context_count
            );
            crate::profile_counter!(
                "render",
                "material_previous_requirement_candidate_resolution_count",
                stats.candidate_resolution_count
            );
        }
    }
    let _ = stats;
}

fn profile_error_proxy_pipeline_census_stats(stats: MaterialPipelineRequirementCensusStats) {
    crate::profile_counter!(
        "render",
        "material_error_proxy_requirement_observed_draw_count",
        stats.observed_draw_count
    );
    crate::profile_counter!(
        "render",
        "material_error_proxy_requirement_unique_context_count",
        stats.unique_context_count
    );
    crate::profile_counter!(
        "render",
        "material_error_proxy_requirement_candidate_resolution_count",
        stats.candidate_resolution_count
    );
    let _ = stats;
}

fn collect_material_pipeline_requirements_for(
    pending_draws: &[PendingMeshDraw],
    material_for_id: impl Fn(&ResourceId) -> Option<(u64, MaterialPipelineCandidate)>,
    features: MaterialPipelineFeatureSet,
    shader_quality: ShaderQualityTier,
    volumetric_fog_enabled: bool,
    kind: MaterialPipelineRequirementCensusKind,
) -> MaterialPipelineRequirementCensus {
    let mut census = MaterialPipelineRequirementCensus::new();
    let mut candidate_owners = HashMap::<ResourceId, Option<MaterialPipelineCensusOwner>>::new();
    for pending_draw in pending_draws {
        let material_id = pending_draw.material.resource_id;
        let owner = match candidate_owners.entry(material_id) {
            std::collections::hash_map::Entry::Occupied(entry) => *entry.get(),
            std::collections::hash_map::Entry::Vacant(entry) => {
                let resolved = material_for_id(&material_id).map(|(generation, candidate)| {
                    let owner = MaterialPipelineCensusOwner {
                        material_id,
                        generation,
                    };
                    census
                        .rows
                        .insert(owner, MaterialPipelineRequirementRow::new(candidate));
                    owner
                });
                entry.insert(resolved);
                resolved
            }
        };
        let Some(owner) = owner else {
            continue;
        };
        let row = census
            .rows
            .get_mut(&owner)
            .expect("resolved material census owner must retain its row");
        let context = MaterialPipelineDrawContext::from_pending_draw(
            pending_draw,
            &row.candidate.inputs.pipeline_key,
            row.candidate.cast_shadows,
        );
        if !row.observed_contexts.insert(context) {
            continue;
        }
        insert_material_pipeline_requirements(
            &mut row.requirements,
            context,
            &row.candidate.inputs,
            features,
            shader_quality,
            volumetric_fog_enabled,
        );
    }
    let unique_context_count = census
        .rows
        .values()
        .map(|row| row.observed_contexts.len())
        .sum();
    let candidate_resolution_count = candidate_owners.len();
    profile_material_pipeline_census_stats(
        kind,
        MaterialPipelineRequirementCensusStats {
            observed_draw_count: pending_draws.len(),
            unique_context_count,
            candidate_resolution_count,
        },
    );
    census
}

fn insert_material_pipeline_requirements(
    requirements: &mut MaterialPipelineRequirementSet,
    context: MaterialPipelineDrawContext,
    candidate: &MaterialPipelineInputs,
    features: MaterialPipelineFeatureSet,
    shader_quality: ShaderQualityTier,
    volumetric_fog_enabled: bool,
) {
    let phase = MeshDrawQueuePhase::from_pipeline_flags(
        candidate.pipeline_key.is_transparent(),
        candidate.pipeline_key.is_alpha_mask(),
    );
    let mut pipeline_key = candidate.pipeline_key.clone();
    pipeline_key.volumetric_fog = volumetric_fog_enabled;

    if features.hit_proxy
        && (!matches!(phase, MeshDrawQueuePhase::Transparent) || features.hit_proxy_translucent)
    {
        let mut hit_proxy_pipeline_key = pipeline_key.clone();
        if features.hit_proxy_backfaces {
            hit_proxy_pipeline_key.double_sided = true;
        }
        insert_requirement(
            requirements,
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::HitProxy),
            &hit_proxy_pipeline_key,
            context.geometry.geometry_source_id(),
            shader_quality,
        );
    }

    if !candidate.disabled_passes.disables_base() && features.base_enabled(&pipeline_key, phase) {
        insert_requirement(
            requirements,
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base),
            &pipeline_key,
            context.geometry.geometry_source_id(),
            shader_quality,
        );
    }
    if features.deferred_gbuffer
        && !candidate.disabled_passes.disables_base()
        && !pipeline_key.requires_forward_path()
        && !matches!(phase, MeshDrawQueuePhase::Transparent)
    {
        insert_requirement(
            requirements,
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::GBuffer),
            &pipeline_key,
            context.geometry.geometry_source_id(),
            shader_quality,
        );
    }
    if features.depth_prepass
        && !candidate.disabled_passes.disables_depth_prepass()
        && phase.casts_shadow()
    {
        insert_requirement(
            requirements,
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::DepthPrepass),
            &pipeline_key,
            context.geometry.geometry_source_id(),
            shader_quality,
        );
    }
    if features.shadow
        && context.shadow.casts_shadows()
        && !candidate.disabled_passes.disables_shadow()
    {
        let shadow_kind = match phase {
            MeshDrawQueuePhase::Opaque => Some(MeshPassPipelineKind::ShadowDepth),
            MeshDrawQueuePhase::AlphaMask => Some(MeshPassPipelineKind::ShadowDepthAlphaMask),
            MeshDrawQueuePhase::Transparent => None,
        };
        if let Some(shadow_kind) = shadow_kind {
            let shadow_pipeline_key = context.shadow.effective_shadow_pipeline_key(&pipeline_key);
            insert_requirement(
                requirements,
                PipelineCreationTarget::MeshPass(shadow_kind),
                &shadow_pipeline_key,
                context.geometry.geometry_source_id(),
                shader_quality,
            );
        }
    }
    if features.velocity
        && !candidate.disabled_passes.disables_velocity()
        && phase.casts_shadow()
        && context.velocity_history_eligible
    {
        insert_requirement(
            requirements,
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Velocity),
            &pipeline_key,
            context.geometry.geometry_source_id(),
            shader_quality,
        );
    }
    if features.taa_reactive && !candidate.disabled_passes.disables_taa_reactive_mask() {
        let taa_kind = match phase {
            MeshDrawQueuePhase::Transparent => Some(MeshPassPipelineKind::TaaReactiveMask),
            MeshDrawQueuePhase::Opaque | MeshDrawQueuePhase::AlphaMask
                if candidate.taa_reactive_mask_strength > f32::EPSILON =>
            {
                Some(MeshPassPipelineKind::TaaReactiveMaterialMask)
            }
            _ => None,
        };
        if let Some(taa_kind) = taa_kind {
            insert_requirement(
                requirements,
                PipelineCreationTarget::MeshPass(taa_kind),
                &pipeline_key,
                context.geometry.geometry_source_id(),
                shader_quality,
            );
        }
    }
    if features.oit
        && !candidate.disabled_passes.disables_base()
        && matches!(phase, MeshDrawQueuePhase::Transparent)
    {
        insert_requirement(
            requirements,
            PipelineCreationTarget::Oit,
            &pipeline_key,
            context.geometry.geometry_source_id(),
            shader_quality,
        );
    }
}

fn insert_requirement(
    requirements: &mut MaterialPipelineRequirementSet,
    target: PipelineCreationTarget,
    pipeline_key: &crate::graphics::scene::resources::PipelineKey,
    geometry_source: GeometrySourceId,
    shader_quality: ShaderQualityTier,
) {
    requirements.insert(MaterialPipelineRequirement::new(
        target,
        pipeline_key.clone(),
        geometry_source,
        shader_quality,
    ));
}

#[cfg(test)]
#[path = "material_pipeline_requirements/tests.rs"]
mod tests;
