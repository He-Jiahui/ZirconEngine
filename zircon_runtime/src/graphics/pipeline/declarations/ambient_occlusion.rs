use crate::core::framework::render::{
    AoQualityTier, AoSourceSettings, AoSourceSettingsKey, PostProcessGraphResourceNames,
    RenderPipelineHandle, RenderViewportRect,
};
use crate::core::math::UVec2;
use crate::graphics::feature::{RenderFeatureResourceKind, RenderFeatureResourceVersion};
use crate::render_graph::{
    CompiledRenderGraph, RenderGraphResourceAccessKind, RenderGraphResourceDesc,
};
use crate::rhi::{TextureDesc, TextureFormat};

pub const COMPILED_AO_PROFILE_VERSION: u32 = 2;
pub const AO_PROFILE_COMPILER_VERSION: u32 = 2;
pub const AO_SHADER_INTERFACE_VERSION: u32 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AmbientOcclusionMethod {
    Gtao,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CompiledAoWorkPlan {
    slice_count: u8,
    samples_per_slice_side: u8,
    sample_count: u16,
}

impl CompiledAoWorkPlan {
    const fn for_quality(quality: AoQualityTier) -> Self {
        let (slice_count, samples_per_slice_side) = match quality {
            AoQualityTier::Low => (1, 2),
            AoQualityTier::Medium => (2, 2),
            AoQualityTier::High => (3, 3),
            AoQualityTier::Ultra => (9, 3),
        };
        Self {
            slice_count,
            samples_per_slice_side,
            sample_count: slice_count as u16 * samples_per_slice_side as u16 * 2,
        }
    }

    pub const fn slice_count(self) -> u8 {
        self.slice_count
    }

    pub const fn samples_per_slice_side(self) -> u8 {
        self.samples_per_slice_side
    }

    pub const fn sample_count(self) -> u16 {
        self.sample_count
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AmbientOcclusionProjectionClass {
    Perspective,
    Orthographic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AmbientOcclusionDepthConvention {
    StandardZeroToOne,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AmbientOcclusionInputSemantic {
    StandardDeviceDepth,
    WorldNormalSignedUnorm,
    StandardDeviceDepthMaxPyramid,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AmbientOcclusionRenderRectKey {
    pub position_x: u32,
    pub position_y: u32,
    pub width: u32,
    pub height: u32,
    pub depth_min_bits: u32,
    pub depth_max_bits: u32,
}

impl From<RenderViewportRect> for AmbientOcclusionRenderRectKey {
    fn from(value: RenderViewportRect) -> Self {
        Self {
            position_x: value.physical_position.x,
            position_y: value.physical_position.y,
            width: value.physical_size.x,
            height: value.physical_size.y,
            depth_min_bits: value.depth_min.to_bits(),
            depth_max_bits: value.depth_max.to_bits(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QualifiedAmbientOcclusionInput {
    semantic: AmbientOcclusionInputSemantic,
    producer_version: RenderFeatureResourceVersion,
    producer_generation: u64,
    texture: TextureDesc,
}

impl QualifiedAmbientOcclusionInput {
    pub(crate) fn new(
        semantic: AmbientOcclusionInputSemantic,
        producer_version: RenderFeatureResourceVersion,
        texture: TextureDesc,
    ) -> Self {
        Self {
            semantic,
            producer_version,
            producer_generation: 0,
            texture,
        }
    }

    pub const fn semantic(&self) -> AmbientOcclusionInputSemantic {
        self.semantic
    }

    pub fn producer_version(&self) -> &RenderFeatureResourceVersion {
        &self.producer_version
    }

    pub const fn producer_generation(&self) -> u64 {
        self.producer_generation
    }

    pub const fn texture(&self) -> &TextureDesc {
        &self.texture
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AmbientOcclusionInputQualification {
    projection_class: AmbientOcclusionProjectionClass,
    depth_convention: AmbientOcclusionDepthConvention,
    render_rect: AmbientOcclusionRenderRectKey,
    allocation_extent: UVec2,
    inputs: Vec<QualifiedAmbientOcclusionInput>,
}

impl AmbientOcclusionInputQualification {
    pub(crate) fn new(
        projection_class: AmbientOcclusionProjectionClass,
        depth_convention: AmbientOcclusionDepthConvention,
        render_rect: RenderViewportRect,
        allocation_extent: UVec2,
        inputs: Vec<QualifiedAmbientOcclusionInput>,
    ) -> Result<Self, String> {
        let qualification = Self {
            projection_class,
            depth_convention,
            render_rect: render_rect.into(),
            allocation_extent,
            inputs,
        };
        qualification.validate()?;
        Ok(qualification)
    }

    pub const fn projection_class(&self) -> AmbientOcclusionProjectionClass {
        self.projection_class
    }

    pub const fn depth_convention(&self) -> AmbientOcclusionDepthConvention {
        self.depth_convention
    }

    pub const fn render_rect(&self) -> AmbientOcclusionRenderRectKey {
        self.render_rect
    }

    pub const fn allocation_extent(&self) -> UVec2 {
        self.allocation_extent
    }

    pub fn inputs(&self) -> &[QualifiedAmbientOcclusionInput] {
        &self.inputs
    }

    pub fn input(
        &self,
        semantic: AmbientOcclusionInputSemantic,
    ) -> Option<&QualifiedAmbientOcclusionInput> {
        self.inputs.iter().find(|input| input.semantic == semantic)
    }

    fn validate(&self) -> Result<(), String> {
        if self.render_rect.width == 0
            || self.render_rect.height == 0
            || self
                .render_rect
                .position_x
                .saturating_add(self.render_rect.width)
                > self.allocation_extent.x
            || self
                .render_rect
                .position_y
                .saturating_add(self.render_rect.height)
                > self.allocation_extent.y
        {
            return Err("AO qualification render rect exceeds its allocation".to_string());
        }
        for semantic in [
            AmbientOcclusionInputSemantic::StandardDeviceDepth,
            AmbientOcclusionInputSemantic::WorldNormalSignedUnorm,
            AmbientOcclusionInputSemantic::StandardDeviceDepthMaxPyramid,
        ] {
            let count = self
                .inputs
                .iter()
                .filter(|input| input.semantic == semantic)
                .count();
            if count != 1 {
                return Err(format!(
                    "AO qualification requires exactly one {semantic:?} input, found {count}"
                ));
            }
        }
        for input in &self.inputs {
            let expected_name = match input.semantic {
                AmbientOcclusionInputSemantic::StandardDeviceDepth => "scene-depth",
                AmbientOcclusionInputSemantic::WorldNormalSignedUnorm => "gbuffer-normal",
                AmbientOcclusionInputSemantic::StandardDeviceDepthMaxPyramid => "hzb-furthest",
            };
            if input.producer_version.resource_name() != expected_name
                || input.producer_version.resource_kind() != RenderFeatureResourceKind::Texture
                || input
                    .producer_version
                    .producer_pass_name()
                    .trim()
                    .is_empty()
                || input.texture.label.as_deref() != Some(expected_name)
            {
                return Err(format!(
                    "AO qualification lost the typed producer identity for {expected_name}"
                ));
            }
        }
        Ok(())
    }

    fn with_producer_generation(mut self, generation: u64) -> Self {
        for input in &mut self.inputs {
            input.producer_generation = generation;
        }
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledAoProfile {
    artifact_version: u32,
    compiler_version: u32,
    shader_interface_version: u32,
    pipeline_generation: u64,
    method: AmbientOcclusionMethod,
    source: AoSourceSettingsKey,
    resolution_divisor: u8,
    work_plan: CompiledAoWorkPlan,
    input_qualification: AmbientOcclusionInputQualification,
}

impl CompiledAoProfile {
    pub(crate) fn compile(
        settings: AoSourceSettings,
        input_qualification: AmbientOcclusionInputQualification,
    ) -> Result<Self, String> {
        validate_source_settings(settings)?;
        input_qualification.validate()?;
        Ok(Self {
            artifact_version: COMPILED_AO_PROFILE_VERSION,
            compiler_version: AO_PROFILE_COMPILER_VERSION,
            shader_interface_version: AO_SHADER_INTERFACE_VERSION,
            pipeline_generation: 0,
            method: AmbientOcclusionMethod::Gtao,
            source: settings.into(),
            resolution_divisor: if settings.half_resolution { 2 } else { 1 },
            work_plan: CompiledAoWorkPlan::for_quality(settings.quality),
            input_qualification,
        })
    }

    pub const fn artifact_version(&self) -> u32 {
        self.artifact_version
    }

    pub const fn compiler_version(&self) -> u32 {
        self.compiler_version
    }

    pub const fn shader_interface_version(&self) -> u32 {
        self.shader_interface_version
    }

    pub const fn pipeline_generation(&self) -> u64 {
        self.pipeline_generation
    }

    pub const fn method(&self) -> AmbientOcclusionMethod {
        self.method
    }

    pub const fn source(&self) -> AoSourceSettingsKey {
        self.source
    }

    pub const fn quality(&self) -> AoQualityTier {
        self.source.quality()
    }

    pub const fn resolution_divisor(&self) -> u8 {
        self.resolution_divisor
    }

    pub fn work_extent(&self) -> UVec2 {
        let divisor = u32::from(self.resolution_divisor.max(1));
        let allocation = self.input_qualification.allocation_extent();
        UVec2::new(
            allocation.x.div_ceil(divisor).max(1),
            allocation.y.div_ceil(divisor).max(1),
        )
    }

    pub const fn work_plan(&self) -> CompiledAoWorkPlan {
        self.work_plan
    }

    pub const fn input_qualification(&self) -> &AmbientOcclusionInputQualification {
        &self.input_qualification
    }

    pub(crate) fn with_pipeline_generation(mut self, generation: u64) -> Self {
        self.pipeline_generation = generation;
        self.input_qualification = self
            .input_qualification
            .with_producer_generation(generation);
        self
    }
}

fn validate_source_settings(settings: AoSourceSettings) -> Result<(), String> {
    for (name, value) in [
        ("intensity", settings.intensity),
        ("radius_meters", settings.radius_meters),
        ("thickness_meters", settings.thickness_meters),
        ("depth_bias_meters", settings.depth_bias_meters),
        ("falloff_start_meters", settings.falloff_start_meters),
    ] {
        if !value.is_finite() {
            return Err(format!("AO source setting `{name}` must be finite"));
        }
    }
    if !(0.0..=1.0).contains(&settings.intensity) {
        return Err("AO intensity must be in the closed range 0..1".to_string());
    }
    if settings.radius_meters <= 0.0 {
        return Err("AO radius_meters must be greater than zero".to_string());
    }
    if settings.thickness_meters <= 0.0 || settings.thickness_meters > settings.radius_meters {
        return Err(
            "AO thickness_meters must be greater than zero and no larger than radius_meters"
                .to_string(),
        );
    }
    if settings.depth_bias_meters < 0.0 || settings.depth_bias_meters >= settings.thickness_meters {
        return Err(
            "AO depth_bias_meters must be non-negative and smaller than thickness_meters"
                .to_string(),
        );
    }
    if settings.falloff_start_meters < 0.0 || settings.falloff_start_meters > settings.radius_meters
    {
        return Err("AO falloff_start_meters must lie inside the search radius".to_string());
    }
    if settings.temporal {
        return Err(
            "AO temporal accumulation is unavailable until motion/depth/normal history qualification is compiled"
                .to_string(),
        );
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AmbientOcclusionOutputs {
    indirect_diffuse_ao: RenderFeatureResourceVersion,
    specular_occlusion: Option<RenderFeatureResourceVersion>,
    bent_normal: Option<RenderFeatureResourceVersion>,
    confidence: Option<RenderFeatureResourceVersion>,
    format: TextureFormat,
    extent: UVec2,
    valid_rect: AmbientOcclusionRenderRectKey,
    producer_generation: u64,
}

impl AmbientOcclusionOutputs {
    pub(crate) fn from_compiled_graph(
        profile: &CompiledAoProfile,
        graph: &CompiledRenderGraph,
    ) -> Result<Self, String> {
        let resource_name = PostProcessGraphResourceNames::AMBIENT_OCCLUSION;
        let writers = graph
            .passes()
            .iter()
            .filter(|pass| !pass.culled)
            .filter(|pass| {
                pass.resources.iter().any(|access| {
                    access.name == resource_name
                        && access.access == RenderGraphResourceAccessKind::Write
                })
            })
            .collect::<Vec<_>>();
        let [writer] = writers.as_slice() else {
            return Err(format!(
                "compiled AO output requires exactly one executable `{resource_name}` writer, found {}",
                writers.len()
            ));
        };
        let lifetime = graph
            .resource_lifetime_by_name(resource_name)
            .ok_or_else(|| "compiled AO output has no graph lifetime".to_string())?;
        let texture = lifetime
            .external_texture_desc
            .as_ref()
            .or_else(|| match &lifetime.desc {
                RenderGraphResourceDesc::Texture(texture) => Some(texture),
                _ => None,
            })
            .ok_or_else(|| "compiled AO output has no physical texture contract".to_string())?;
        let allocation = profile.input_qualification().allocation_extent();
        if texture.width != allocation.x
            || texture.height != allocation.y
            || texture.mip_levels != 1
            || texture.sample_count != 1
        {
            return Err(format!(
                "compiled AO output physical contract is {}x{} with {} mips and {} samples; expected {}x{} with one mip and one sample",
                texture.width,
                texture.height,
                texture.mip_levels,
                texture.sample_count,
                allocation.x,
                allocation.y
            ));
        }
        Self::new(
            RenderFeatureResourceVersion::new(
                resource_name,
                RenderFeatureResourceKind::Texture,
                writer.name.clone(),
            ),
            None,
            None,
            None,
            texture.format,
            allocation,
            profile.input_qualification().render_rect(),
            profile.pipeline_generation(),
        )
    }

    pub(crate) fn new(
        indirect_diffuse_ao: RenderFeatureResourceVersion,
        specular_occlusion: Option<RenderFeatureResourceVersion>,
        bent_normal: Option<RenderFeatureResourceVersion>,
        confidence: Option<RenderFeatureResourceVersion>,
        format: TextureFormat,
        extent: UVec2,
        valid_rect: AmbientOcclusionRenderRectKey,
        producer_generation: u64,
    ) -> Result<Self, String> {
        if indirect_diffuse_ao.resource_name() != PostProcessGraphResourceNames::AMBIENT_OCCLUSION
            || indirect_diffuse_ao.resource_kind() != RenderFeatureResourceKind::Texture
            || indirect_diffuse_ao.producer_pass_name().trim().is_empty()
        {
            return Err(
                "AO outputs require a typed ambient-occlusion texture producer".to_string(),
            );
        }
        if extent.x == 0 || extent.y == 0 || producer_generation == 0 {
            return Err(
                "AO outputs require a non-empty extent and producer generation".to_string(),
            );
        }
        if valid_rect.width == 0
            || valid_rect.height == 0
            || valid_rect.position_x.saturating_add(valid_rect.width) > extent.x
            || valid_rect.position_y.saturating_add(valid_rect.height) > extent.y
        {
            return Err("AO output valid rect exceeds its physical extent".to_string());
        }
        Ok(Self {
            indirect_diffuse_ao,
            specular_occlusion,
            bent_normal,
            confidence,
            format,
            extent,
            valid_rect,
            producer_generation,
        })
    }

    pub fn indirect_diffuse_ao(&self) -> &RenderFeatureResourceVersion {
        &self.indirect_diffuse_ao
    }

    pub fn specular_occlusion(&self) -> Option<&RenderFeatureResourceVersion> {
        self.specular_occlusion.as_ref()
    }

    pub fn bent_normal(&self) -> Option<&RenderFeatureResourceVersion> {
        self.bent_normal.as_ref()
    }

    pub fn confidence(&self) -> Option<&RenderFeatureResourceVersion> {
        self.confidence.as_ref()
    }

    pub const fn format(&self) -> TextureFormat {
        self.format
    }

    pub const fn extent(&self) -> UVec2 {
        self.extent
    }

    pub const fn valid_rect(&self) -> AmbientOcclusionRenderRectKey {
        self.valid_rect
    }

    pub const fn producer_generation(&self) -> u64 {
        self.producer_generation
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct AoHistoryProducerIdentity {
    resource_name: String,
    producer_pass_name: String,
    generation: u64,
}

impl AoHistoryProducerIdentity {
    fn new(version: &RenderFeatureResourceVersion, generation: u64) -> Self {
        Self {
            resource_name: version.resource_name().to_string(),
            producer_pass_name: version.producer_pass_name().to_string(),
            generation,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AoHistoryKey {
    view_identity: u64,
    world_origin_epoch: u64,
    pipeline: RenderPipelineHandle,
    pipeline_generation: u64,
    method: AmbientOcclusionMethod,
    source: AoSourceSettingsKey,
    projection_class: AmbientOcclusionProjectionClass,
    depth_convention: AmbientOcclusionDepthConvention,
    render_rect: AmbientOcclusionRenderRectKey,
    allocation_width: u32,
    allocation_height: u32,
    depth: AoHistoryProducerIdentity,
    normal: AoHistoryProducerIdentity,
    motion: AoHistoryProducerIdentity,
    output_format_id: u8,
    output_width: u32,
    output_height: u32,
    output_generation: u64,
}

impl AoHistoryKey {
    pub(crate) fn new(
        view_identity: u64,
        world_origin_epoch: u64,
        pipeline: RenderPipelineHandle,
        profile: &CompiledAoProfile,
        motion_version: &RenderFeatureResourceVersion,
        motion_generation: u64,
        outputs: &AmbientOcclusionOutputs,
    ) -> Result<Self, String> {
        let qualification = profile.input_qualification();
        let depth = qualification
            .input(AmbientOcclusionInputSemantic::StandardDeviceDepth)
            .ok_or_else(|| "AO history key requires a qualified depth producer".to_string())?;
        let normal = qualification
            .input(AmbientOcclusionInputSemantic::WorldNormalSignedUnorm)
            .ok_or_else(|| "AO history key requires a qualified normal producer".to_string())?;
        if motion_version.resource_name() != PostProcessGraphResourceNames::SCENE_VELOCITY
            || motion_version.resource_kind() != RenderFeatureResourceKind::Texture
            || motion_version.producer_pass_name().trim().is_empty()
        {
            return Err(
                "AO history key requires a typed scene-velocity texture producer".to_string(),
            );
        }
        if profile.pipeline_generation() == 0
            || outputs.producer_generation() == 0
            || motion_generation == 0
        {
            return Err(
                "AO history key requires non-zero compiled producer generations".to_string(),
            );
        }
        if outputs.producer_generation() != profile.pipeline_generation() {
            return Err("AO output generation does not match its compiled profile".to_string());
        }
        if outputs.valid_rect() != qualification.render_rect() {
            return Err(
                "AO output valid rect does not match its compiled qualification".to_string(),
            );
        }
        let output_format_id = match outputs.format() {
            TextureFormat::R8Unorm => 1,
            TextureFormat::Rgba8Unorm => 2,
            other => {
                return Err(format!(
                    "AO history key does not support output format {other:?}"
                ));
            }
        };
        Ok(Self {
            view_identity,
            world_origin_epoch,
            pipeline,
            pipeline_generation: profile.pipeline_generation(),
            method: profile.method(),
            source: profile.source(),
            projection_class: qualification.projection_class(),
            depth_convention: qualification.depth_convention(),
            render_rect: qualification.render_rect(),
            allocation_width: qualification.allocation_extent().x,
            allocation_height: qualification.allocation_extent().y,
            depth: AoHistoryProducerIdentity::new(
                depth.producer_version(),
                depth.producer_generation(),
            ),
            normal: AoHistoryProducerIdentity::new(
                normal.producer_version(),
                normal.producer_generation(),
            ),
            motion: AoHistoryProducerIdentity::new(motion_version, motion_generation),
            output_format_id,
            output_width: outputs.extent().x,
            output_height: outputs.extent().y,
            output_generation: outputs.producer_generation(),
        })
    }
}

#[cfg(test)]
mod tests;
