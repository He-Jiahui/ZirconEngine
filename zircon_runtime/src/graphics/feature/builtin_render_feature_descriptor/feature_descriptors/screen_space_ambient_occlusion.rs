use bytemuck::{Pod, Zeroable};

use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::core::math::UVec2;
use crate::graphics::pipeline::{
    AO_SHADER_INTERFACE_VERSION, AmbientOcclusionInputSemantic, CompiledAoProfile, RenderPassStage,
};
use crate::graphics::{
    ComputePassDescriptor, ComputeShaderSource, RenderBufferSchema, RenderResourceSchema,
    RenderTextureExtentPolicy, RenderTextureExtentReference, RenderTextureExtentRounding,
    RenderTextureSchema,
};
use crate::render_graph::{
    BindingSchemaEntry, ComputeBindingKind, PassFlags, QueueLane, RenderGraphBufferRange,
    RenderGraphComputeDispatchExtent, RenderGraphExternalResourceBinding,
    RenderGraphResourceAccessIntent, RenderGraphResourceUsageFlags, RenderGraphShaderStages,
};
use crate::rhi::{BufferUsage, TextureFormat, TextureUsage};

use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::super::render_feature_pass_descriptor::{
    RenderFeaturePassDescriptor, RenderFeatureResourceAccess, RenderFeatureResourceKind,
};
use super::compute_workload::SSAO_WORKGROUP_SIZE;

const SSAO_EVALUATE_PIPELINE_FAMILY: &str = "ambient-occlusion.evaluate";
const SSAO_SPATIAL_PIPELINE_FAMILY: &str = "ambient-occlusion.spatial-denoise";
const SSAO_UPSAMPLE_PIPELINE_FAMILY: &str = "ambient-occlusion.bilateral-upsample";
const SSAO_EVALUATE_PASS_NAME: &str = "ssao-evaluate";
const SSAO_SPATIAL_PASS_NAME: &str = "ssao-spatial-denoise";
const SSAO_UPSAMPLE_PASS_NAME: &str = "ssao-bilateral-upsample";

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(crate) struct SsaoParams {
    extent_and_sample_counts: [u32; 4],
    input_extent_and_resolution: [u32; 4],
    world_radius_thickness_bias_falloff: [f32; 4],
    intensity_and_limits: [f32; 4],
}

impl SsaoParams {
    pub(crate) fn from_compiled_profile(
        profile: &CompiledAoProfile,
        runtime_extent: UVec2,
    ) -> Result<Self, String> {
        let qualification = profile.input_qualification();
        if profile.pipeline_generation() == 0 {
            return Err("SSAO params require a generated compiled AO profile".to_string());
        }
        if qualification.allocation_extent() != runtime_extent {
            return Err(format!(
                "SSAO runtime extent {}x{} does not match compiled allocation {}x{}",
                runtime_extent.x,
                runtime_extent.y,
                qualification.allocation_extent().x,
                qualification.allocation_extent().y
            ));
        }
        let hzb = qualification
            .input(AmbientOcclusionInputSemantic::StandardDeviceDepthMaxPyramid)
            .ok_or_else(|| "SSAO params require the compiled HZB receipt".to_string())?;
        let work_plan = profile.work_plan();
        let source = profile.source();
        let work_extent = profile.work_extent();
        let resolution_divisor = u32::from(profile.resolution_divisor());
        if !matches!(resolution_divisor, 1 | 2) {
            return Err(format!(
                "SSAO params do not support resolution divisor {resolution_divisor}"
            ));
        }
        Ok(Self {
            extent_and_sample_counts: [
                work_extent.x,
                work_extent.y,
                u32::from(work_plan.slice_count()),
                u32::from(work_plan.samples_per_slice_side()),
            ],
            input_extent_and_resolution: [
                runtime_extent.x,
                runtime_extent.y,
                resolution_divisor,
                0,
            ],
            world_radius_thickness_bias_falloff: [
                source.radius_meters(),
                source.thickness_meters(),
                source.depth_bias_meters(),
                source.falloff_start_meters(),
            ],
            intensity_and_limits: [
                source.intensity(),
                hzb.texture().mip_levels.saturating_sub(1) as f32,
                128.0 / resolution_divisor as f32,
                0.03,
            ],
        })
    }

    #[cfg(test)]
    pub(crate) const fn extent_and_sample_counts(self) -> [u32; 4] {
        self.extent_and_sample_counts
    }

    #[cfg(test)]
    pub(crate) const fn input_extent_and_resolution(self) -> [u32; 4] {
        self.input_extent_and_resolution
    }

    #[cfg(test)]
    pub(crate) const fn world_radius_thickness_bias_falloff(self) -> [f32; 4] {
        self.world_radius_thickness_bias_falloff
    }

    #[cfg(test)]
    pub(crate) const fn intensity_and_limits(self) -> [f32; 4] {
        self.intensity_and_limits
    }
}

pub(super) fn descriptor() -> RenderFeatureDescriptor {
    let evaluate = ComputePassDescriptor::new(
        "ssao-evaluate",
        RenderPassStage::AmbientOcclusion,
        QueueLane::AsyncCompute,
        ComputeShaderSource::builtin_wgsl(
            "zircon-ssao-pipeline",
            include_str!("../../../scene/scene_renderer/post_process/shaders/ssao.wgsl"),
        ),
        "cs_main",
        SSAO_WORKGROUP_SIZE,
        vec![
            BindingSchemaEntry::new(
                0,
                PostProcessGraphResourceNames::SCENE_DEPTH,
                ComputeBindingKind::SampledTexture,
            ),
            BindingSchemaEntry::new(
                1,
                PostProcessGraphResourceNames::GBUFFER_NORMAL,
                ComputeBindingKind::SampledTexture,
            ),
            BindingSchemaEntry::new(
                2,
                PostProcessGraphResourceNames::SSAO_PARAMS,
                ComputeBindingKind::UniformBuffer,
            ),
            BindingSchemaEntry::new(
                3,
                PostProcessGraphResourceNames::AMBIENT_OCCLUSION_RAW,
                ComputeBindingKind::StorageTextureWrite,
            ),
            BindingSchemaEntry::new(
                4,
                PostProcessGraphResourceNames::HZB_FURTHEST,
                ComputeBindingKind::SampledTexture,
            )
            .with_texture_full_mip_chain(),
        ],
        RenderGraphComputeDispatchExtent::PerPixel {
            target: PostProcessGraphResourceNames::AMBIENT_OCCLUSION_RAW.to_string(),
            local_size: [SSAO_WORKGROUP_SIZE[0], SSAO_WORKGROUP_SIZE[1]],
        },
        PassFlags::default(),
    )
    .with_last_good_pipeline(
        SSAO_EVALUATE_PIPELINE_FAMILY,
        u64::from(AO_SHADER_INTERFACE_VERSION),
    )
    .with_resource_schema(
        PostProcessGraphResourceNames::SSAO_PARAMS,
        ssao_params_schema(),
    )
    .with_resource_schema(
        PostProcessGraphResourceNames::AMBIENT_OCCLUSION_RAW,
        ambient_occlusion_schema(),
    );

    let spatial_denoise = ComputePassDescriptor::new(
        "ssao-spatial-denoise",
        RenderPassStage::AmbientOcclusion,
        QueueLane::AsyncCompute,
        ComputeShaderSource::builtin_wgsl(
            "zircon-ssao-spatial-denoise-pipeline",
            include_str!(
                "../../../scene/scene_renderer/post_process/shaders/ssao_spatial_denoise.wgsl"
            ),
        ),
        "cs_main",
        SSAO_WORKGROUP_SIZE,
        vec![
            BindingSchemaEntry::new(
                0,
                PostProcessGraphResourceNames::AMBIENT_OCCLUSION_RAW,
                ComputeBindingKind::SampledTexture,
            ),
            BindingSchemaEntry::new(
                1,
                PostProcessGraphResourceNames::SCENE_DEPTH,
                ComputeBindingKind::SampledTexture,
            ),
            BindingSchemaEntry::new(
                2,
                PostProcessGraphResourceNames::GBUFFER_NORMAL,
                ComputeBindingKind::SampledTexture,
            ),
            BindingSchemaEntry::new(
                3,
                PostProcessGraphResourceNames::SSAO_PARAMS,
                ComputeBindingKind::UniformBuffer,
            ),
            BindingSchemaEntry::new(
                4,
                PostProcessGraphResourceNames::AMBIENT_OCCLUSION,
                ComputeBindingKind::StorageTextureWrite,
            ),
        ],
        RenderGraphComputeDispatchExtent::PerPixel {
            target: PostProcessGraphResourceNames::AMBIENT_OCCLUSION.to_string(),
            local_size: [SSAO_WORKGROUP_SIZE[0], SSAO_WORKGROUP_SIZE[1]],
        },
        PassFlags::default(),
    )
    .with_last_good_pipeline(
        SSAO_SPATIAL_PIPELINE_FAMILY,
        u64::from(AO_SHADER_INTERFACE_VERSION),
    )
    .with_resource_schema(
        PostProcessGraphResourceNames::SSAO_PARAMS,
        ssao_params_schema(),
    )
    .with_resource_schema(
        PostProcessGraphResourceNames::AMBIENT_OCCLUSION_RAW,
        ambient_occlusion_schema(),
    )
    .with_resource_schema(
        PostProcessGraphResourceNames::AMBIENT_OCCLUSION,
        ambient_occlusion_schema(),
    );

    RenderFeatureDescriptor::new(
        "screen_space_ambient_occlusion",
        vec![
            "view".to_string(),
            "geometry".to_string(),
            "visibility".to_string(),
        ],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::AmbientOcclusion,
                "ssao-evaluate",
                QueueLane::AsyncCompute,
            )
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .read_texture(PostProcessGraphResourceNames::GBUFFER_NORMAL)
            .read_texture(PostProcessGraphResourceNames::HZB_FURTHEST)
            .read_external_buffer_with_schema_and_access(
                PostProcessGraphResourceNames::SSAO_PARAMS,
                ssao_params_schema(),
                RenderGraphBufferRange::full(),
                RenderGraphResourceAccessIntent::UniformBuffer {
                    stages: RenderGraphShaderStages::COMPUTE,
                },
            )
            .write_storage_texture(PostProcessGraphResourceNames::AMBIENT_OCCLUSION_RAW)
            .with_compute_pass(evaluate),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::AmbientOcclusion,
                "ssao-spatial-denoise",
                QueueLane::AsyncCompute,
            )
            .read_texture_from(
                PostProcessGraphResourceNames::AMBIENT_OCCLUSION_RAW,
                "ssao-evaluate",
            )
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .read_texture(PostProcessGraphResourceNames::GBUFFER_NORMAL)
            .read_external_buffer_with_schema_and_access(
                PostProcessGraphResourceNames::SSAO_PARAMS,
                ssao_params_schema(),
                RenderGraphBufferRange::full(),
                RenderGraphResourceAccessIntent::UniformBuffer {
                    stages: RenderGraphShaderStages::COMPUTE,
                },
            )
            .write_persistent_storage_external_texture(
                PostProcessGraphResourceNames::AMBIENT_OCCLUSION,
            )
            .with_compute_pass(spatial_denoise),
        ],
    )
}

pub(super) fn configure_for_profile(
    descriptors: &mut [RenderFeatureDescriptor],
    profile: &CompiledAoProfile,
) -> Result<(), String> {
    let owners = descriptors
        .iter()
        .enumerate()
        .filter(|(_, descriptor)| descriptor.name == "screen_space_ambient_occlusion")
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    let [owner_index] = owners.as_slice() else {
        return Err(format!(
            "compiled AO profile requires exactly one screen_space_ambient_occlusion descriptor, found {}",
            owners.len()
        ));
    };
    configure_descriptor_for_resolution(
        &mut descriptors[*owner_index],
        profile.resolution_divisor(),
    )
}

fn configure_descriptor_for_resolution(
    descriptor: &mut RenderFeatureDescriptor,
    resolution_divisor: u8,
) -> Result<(), String> {
    match resolution_divisor {
        1 => return Ok(()),
        2 => {}
        unsupported => {
            return Err(format!(
                "AO descriptor does not support resolution divisor {unsupported}"
            ));
        }
    }
    if descriptor
        .stage_passes
        .iter()
        .any(|pass| pass.pass_name == SSAO_UPSAMPLE_PASS_NAME)
    {
        return Err("AO half-resolution descriptor was configured more than once".to_string());
    }

    let evaluate_index = unique_pass_index(descriptor, SSAO_EVALUATE_PASS_NAME)?;
    let spatial_index = unique_pass_index(descriptor, SSAO_SPATIAL_PASS_NAME)?;
    let half_schema = half_resolution_ambient_occlusion_schema();
    configure_evaluate_half_resolution(&mut descriptor.stage_passes[evaluate_index], half_schema)?;
    configure_spatial_half_resolution(&mut descriptor.stage_passes[spatial_index], half_schema)?;
    descriptor
        .stage_passes
        .insert(spatial_index + 1, bilateral_upsample_pass(half_schema));
    Ok(())
}

fn unique_pass_index(
    descriptor: &RenderFeatureDescriptor,
    pass_name: &str,
) -> Result<usize, String> {
    let indices = descriptor
        .stage_passes
        .iter()
        .enumerate()
        .filter(|(_, pass)| pass.pass_name == pass_name)
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    let [index] = indices.as_slice() else {
        return Err(format!(
            "AO descriptor requires exactly one `{pass_name}` pass, found {}",
            indices.len()
        ));
    };
    Ok(*index)
}

fn configure_evaluate_half_resolution(
    pass: &mut RenderFeaturePassDescriptor,
    half_schema: RenderResourceSchema,
) -> Result<(), String> {
    let raw_write_count = pass
        .resources
        .iter()
        .filter(|resource| {
            resource.name == PostProcessGraphResourceNames::AMBIENT_OCCLUSION_RAW
                && resource.access == RenderFeatureResourceAccess::Write
        })
        .count();
    if raw_write_count != 1 {
        return Err(format!(
            "`{SSAO_EVALUATE_PASS_NAME}` requires exactly one raw AO write, found {}",
            raw_write_count
        ));
    }
    let raw_write = pass
        .resources
        .iter_mut()
        .find(|resource| {
            resource.name == PostProcessGraphResourceNames::AMBIENT_OCCLUSION_RAW
                && resource.access == RenderFeatureResourceAccess::Write
        })
        .ok_or_else(|| format!("`{SSAO_EVALUATE_PASS_NAME}` lost its raw AO write"))?;
    raw_write.schema = Some(half_schema);
    let compute = pass
        .compute_pass
        .take()
        .ok_or_else(|| format!("`{SSAO_EVALUATE_PASS_NAME}` has no compute descriptor"))?;
    pass.compute_pass = Some(compute.with_resource_schema(
        PostProcessGraphResourceNames::AMBIENT_OCCLUSION_RAW,
        half_schema,
    ));
    Ok(())
}

fn configure_spatial_half_resolution(
    pass: &mut RenderFeaturePassDescriptor,
    half_schema: RenderResourceSchema,
) -> Result<(), String> {
    for resource in &mut pass.resources {
        if resource.name == PostProcessGraphResourceNames::AMBIENT_OCCLUSION_RAW {
            resource.schema = Some(half_schema);
        }
    }
    let final_write_count = pass
        .resources
        .iter()
        .filter(|resource| {
            resource.name == PostProcessGraphResourceNames::AMBIENT_OCCLUSION
                && resource.access == RenderFeatureResourceAccess::Write
        })
        .count();
    if final_write_count != 1 {
        return Err(format!(
            "`{SSAO_SPATIAL_PASS_NAME}` requires exactly one final AO write, found {}",
            final_write_count
        ));
    }
    let final_write = pass
        .resources
        .iter_mut()
        .find(|resource| {
            resource.name == PostProcessGraphResourceNames::AMBIENT_OCCLUSION
                && resource.access == RenderFeatureResourceAccess::Write
        })
        .ok_or_else(|| format!("`{SSAO_SPATIAL_PASS_NAME}` lost its final AO write"))?;
    final_write.name = PostProcessGraphResourceNames::AMBIENT_OCCLUSION_SPATIAL.to_string();
    final_write.kind = RenderFeatureResourceKind::Texture;
    final_write.external_binding = RenderGraphExternalResourceBinding::report_only();
    final_write.schema = Some(half_schema);
    final_write.usage = RenderGraphResourceUsageFlags::default();

    let mut compute = pass
        .compute_pass
        .take()
        .ok_or_else(|| format!("`{SSAO_SPATIAL_PASS_NAME}` has no compute descriptor"))?;
    let output_binding_count = compute
        .bindings
        .iter()
        .filter(|binding| {
            binding.resource == PostProcessGraphResourceNames::AMBIENT_OCCLUSION
                && binding.kind == ComputeBindingKind::StorageTextureWrite
        })
        .count();
    if output_binding_count != 1 {
        return Err(format!(
            "`{SSAO_SPATIAL_PASS_NAME}` requires exactly one final AO storage binding, found {}",
            output_binding_count
        ));
    }
    let output_binding = compute
        .bindings
        .iter_mut()
        .find(|binding| {
            binding.resource == PostProcessGraphResourceNames::AMBIENT_OCCLUSION
                && binding.kind == ComputeBindingKind::StorageTextureWrite
        })
        .ok_or_else(|| format!("`{SSAO_SPATIAL_PASS_NAME}` lost its output storage binding"))?;
    output_binding.resource = PostProcessGraphResourceNames::AMBIENT_OCCLUSION_SPATIAL.to_string();
    compute.dispatch = RenderGraphComputeDispatchExtent::PerPixel {
        target: PostProcessGraphResourceNames::AMBIENT_OCCLUSION_SPATIAL.to_string(),
        local_size: [SSAO_WORKGROUP_SIZE[0], SSAO_WORKGROUP_SIZE[1]],
    };
    compute = compute
        .with_resource_schema(
            PostProcessGraphResourceNames::AMBIENT_OCCLUSION_RAW,
            half_schema,
        )
        .with_resource_schema(
            PostProcessGraphResourceNames::AMBIENT_OCCLUSION_SPATIAL,
            half_schema,
        );
    let mut rebuilt_pass = pass.clone();
    rebuilt_pass.compute_workload = None;
    rebuilt_pass.compute_pass = None;
    *pass = rebuilt_pass.with_compute_pass(compute);
    Ok(())
}

fn bilateral_upsample_pass(half_schema: RenderResourceSchema) -> RenderFeaturePassDescriptor {
    let compute = ComputePassDescriptor::new(
        SSAO_UPSAMPLE_PASS_NAME,
        RenderPassStage::AmbientOcclusion,
        QueueLane::AsyncCompute,
        ComputeShaderSource::builtin_wgsl(
            "zircon-ssao-bilateral-upsample-pipeline",
            include_str!(
                "../../../scene/scene_renderer/post_process/shaders/ssao_bilateral_upsample.wgsl"
            ),
        ),
        "cs_main",
        SSAO_WORKGROUP_SIZE,
        vec![
            BindingSchemaEntry::new(
                0,
                PostProcessGraphResourceNames::AMBIENT_OCCLUSION_SPATIAL,
                ComputeBindingKind::SampledTexture,
            ),
            BindingSchemaEntry::new(
                1,
                PostProcessGraphResourceNames::SCENE_DEPTH,
                ComputeBindingKind::SampledTexture,
            ),
            BindingSchemaEntry::new(
                2,
                PostProcessGraphResourceNames::GBUFFER_NORMAL,
                ComputeBindingKind::SampledTexture,
            ),
            BindingSchemaEntry::new(
                3,
                PostProcessGraphResourceNames::SSAO_PARAMS,
                ComputeBindingKind::UniformBuffer,
            ),
            BindingSchemaEntry::new(
                4,
                PostProcessGraphResourceNames::AMBIENT_OCCLUSION,
                ComputeBindingKind::StorageTextureWrite,
            ),
        ],
        RenderGraphComputeDispatchExtent::PerPixel {
            target: PostProcessGraphResourceNames::AMBIENT_OCCLUSION.to_string(),
            local_size: [SSAO_WORKGROUP_SIZE[0], SSAO_WORKGROUP_SIZE[1]],
        },
        PassFlags::default(),
    )
    .with_last_good_pipeline(
        SSAO_UPSAMPLE_PIPELINE_FAMILY,
        u64::from(AO_SHADER_INTERFACE_VERSION),
    )
    .with_resource_schema(
        PostProcessGraphResourceNames::SSAO_PARAMS,
        ssao_params_schema(),
    )
    .with_resource_schema(
        PostProcessGraphResourceNames::AMBIENT_OCCLUSION_SPATIAL,
        half_schema,
    )
    .with_resource_schema(
        PostProcessGraphResourceNames::AMBIENT_OCCLUSION,
        ambient_occlusion_schema(),
    );

    RenderFeaturePassDescriptor::new(
        RenderPassStage::AmbientOcclusion,
        SSAO_UPSAMPLE_PASS_NAME,
        QueueLane::AsyncCompute,
    )
    .read_texture_from(
        PostProcessGraphResourceNames::AMBIENT_OCCLUSION_SPATIAL,
        SSAO_SPATIAL_PASS_NAME,
    )
    .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
    .read_texture(PostProcessGraphResourceNames::GBUFFER_NORMAL)
    .read_external_buffer_with_schema_and_access(
        PostProcessGraphResourceNames::SSAO_PARAMS,
        ssao_params_schema(),
        RenderGraphBufferRange::full(),
        RenderGraphResourceAccessIntent::UniformBuffer {
            stages: RenderGraphShaderStages::COMPUTE,
        },
    )
    .write_persistent_storage_external_texture(PostProcessGraphResourceNames::AMBIENT_OCCLUSION)
    .with_compute_pass(compute)
}

fn ssao_params_schema() -> RenderResourceSchema {
    RenderResourceSchema::buffer(RenderBufferSchema::new(
        std::mem::size_of::<SsaoParams>() as u64,
        BufferUsage::UNIFORM | BufferUsage::COPY_DST,
    ))
}

fn ambient_occlusion_schema() -> RenderResourceSchema {
    RenderResourceSchema::texture(RenderTextureSchema::new(
        TextureFormat::Rgba8Unorm,
        TextureUsage::SAMPLED | TextureUsage::STORAGE,
    ))
}

fn half_resolution_ambient_occlusion_schema() -> RenderResourceSchema {
    RenderResourceSchema::texture(
        RenderTextureSchema::new(
            TextureFormat::Rgba8Unorm,
            TextureUsage::SAMPLED | TextureUsage::STORAGE,
        )
        .with_extent(RenderTextureExtentPolicy::Relative {
            reference: RenderTextureExtentReference::Render,
            numerator: 1,
            denominator: 2,
            rounding: RenderTextureExtentRounding::Ceil,
        }),
    )
}

#[cfg(test)]
mod tests;
