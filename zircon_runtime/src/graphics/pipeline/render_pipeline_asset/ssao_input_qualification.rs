use crate::core::framework::render::{
    PostProcessGraphResourceNames, ProjectionMode, RenderFrameExtract, RenderPipelinePhase,
    RenderViewportRect,
};
use crate::core::math::UVec2;
use crate::graphics::feature::{
    BuiltinRenderFeature, RenderFeatureDescriptor, RenderFeatureResourceAccess,
    RenderFeatureResourceDescriptor, RenderFeatureResourceVersion,
};
use crate::graphics::pipeline::declarations::{
    AmbientOcclusionDepthConvention, AmbientOcclusionInputQualification,
    AmbientOcclusionInputSemantic, AmbientOcclusionProjectionClass, QualifiedAmbientOcclusionInput,
    RenderPassStage, RenderPipelineCompileOptions, RendererFeatureAsset,
};
use crate::graphics::visibility::HzbBuilder;
use crate::rhi::{TextureDesc, TextureDimension, TextureFormat, TextureUsage};

use super::resource_schema_catalog::RenderResourceSchemaCatalog;

const SSAO_EVALUATE_PASS_NAME: &str = "ssao-evaluate";
const SSAO_REQUIRED_INPUTS: [SsaoInputRequirement; 3] = [
    SsaoInputRequirement {
        resource_name: PostProcessGraphResourceNames::SCENE_DEPTH,
        semantic: AmbientOcclusionInputSemantic::StandardDeviceDepth,
        format: TextureFormat::Depth32Float,
        extent: SsaoInputExtent::SceneLinear,
    },
    SsaoInputRequirement {
        resource_name: PostProcessGraphResourceNames::GBUFFER_NORMAL,
        semantic: AmbientOcclusionInputSemantic::WorldNormalSignedUnorm,
        format: TextureFormat::Rgba8Unorm,
        extent: SsaoInputExtent::SceneLinear,
    },
    SsaoInputRequirement {
        resource_name: PostProcessGraphResourceNames::HZB_FURTHEST,
        semantic: AmbientOcclusionInputSemantic::StandardDeviceDepthMaxPyramid,
        format: TextureFormat::Rgba16Float,
        extent: SsaoInputExtent::HzbPyramid,
    },
];

pub(super) fn validate_ssao_input_qualification(
    extract: &RenderFrameExtract,
    options: &RenderPipelineCompileOptions,
    stages: &[RenderPassStage],
    enabled_features: &[RendererFeatureAsset],
    descriptors: &[RenderFeatureDescriptor],
) -> Result<Option<AmbientOcclusionInputQualification>, String> {
    if !enabled_features.iter().any(feature_requests_ssao) {
        return Ok(None);
    }

    let ssao_pass = descriptors
        .iter()
        .flat_map(|descriptor| descriptor.stage_passes.iter())
        .find(|pass| pass.pass_name == SSAO_EVALUATE_PASS_NAME)
        .ok_or_else(|| {
            format!(
                "screen_space_ambient_occlusion input qualification failed: enabled feature has no `{SSAO_EVALUATE_PASS_NAME}` pass"
            )
        })?;
    let ssao_stage_index = stage_index(stages, ssao_pass.stage).ok_or_else(|| {
        format!(
            "screen_space_ambient_occlusion input qualification failed: `{SSAO_EVALUATE_PASS_NAME}` stage {:?} is not enabled",
            ssao_pass.stage
        )
    })?;

    let view_family = extract.view.view_family_pipeline();
    let scene_target = view_family
        .output_target_for_phase(RenderPipelinePhase::SceneLinear)
        .ok_or_else(|| qualification_error("SceneLinear view-family target is unavailable"))?;
    let render_rect = scene_target.viewport();
    let allocation_extent = scene_target.allocation_extent();
    validate_render_rect(render_rect, allocation_extent)?;
    let projection_class = projection_class(extract)?;

    let catalog = RenderResourceSchemaCatalog::new(extract, options);
    let mut inputs = Vec::with_capacity(SSAO_REQUIRED_INPUTS.len());
    for requirement in SSAO_REQUIRED_INPUTS {
        inputs.push(qualify_input(
            requirement,
            stages,
            ssao_stage_index,
            descriptors,
            allocation_extent,
            &catalog,
        )?);
    }

    AmbientOcclusionInputQualification::new(
        projection_class,
        AmbientOcclusionDepthConvention::StandardZeroToOne,
        render_rect,
        allocation_extent,
        inputs,
    )
    .map(Some)
    .map_err(qualification_error)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SsaoInputExtent {
    SceneLinear,
    HzbPyramid,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SsaoInputRequirement {
    resource_name: &'static str,
    semantic: AmbientOcclusionInputSemantic,
    format: TextureFormat,
    extent: SsaoInputExtent,
}

fn projection_class(
    extract: &RenderFrameExtract,
) -> Result<AmbientOcclusionProjectionClass, String> {
    if extract.view.camera.projection_override.is_some() {
        return Err(qualification_error(
            "custom/oblique projection has no compiled AO reconstruction contract",
        ));
    }
    Ok(match extract.view.camera.projection_mode {
        ProjectionMode::Perspective => AmbientOcclusionProjectionClass::Perspective,
        ProjectionMode::Orthographic => AmbientOcclusionProjectionClass::Orthographic,
    })
}

fn validate_render_rect(
    render_rect: RenderViewportRect,
    allocation_extent: UVec2,
) -> Result<(), String> {
    if render_rect.physical_position != UVec2::ZERO {
        return Err(qualification_error(format!(
            "viewport origin {}x{} is unsupported until the AO kernel consumes the compiled render rect",
            render_rect.physical_position.x, render_rect.physical_position.y
        )));
    }
    if render_rect.physical_size != allocation_extent {
        return Err(qualification_error(format!(
            "viewport size {}x{} does not cover the {}x{} allocation; partial rect AO is unsupported until the kernel consumes the compiled render rect",
            render_rect.physical_size.x,
            render_rect.physical_size.y,
            allocation_extent.x,
            allocation_extent.y
        )));
    }
    let maximum = UVec2::new(
        render_rect
            .physical_position
            .x
            .saturating_add(render_rect.physical_size.x),
        render_rect
            .physical_position
            .y
            .saturating_add(render_rect.physical_size.y),
    );
    if render_rect.physical_size.x == 0
        || render_rect.physical_size.y == 0
        || maximum.x > allocation_extent.x
        || maximum.y > allocation_extent.y
    {
        return Err(qualification_error(format!(
            "viewport {}x{} at {}x{} exceeds the {}x{} SceneLinear allocation",
            render_rect.physical_size.x,
            render_rect.physical_size.y,
            render_rect.physical_position.x,
            render_rect.physical_position.y,
            allocation_extent.x,
            allocation_extent.y
        )));
    }
    if render_rect.depth_min != 0.0 || render_rect.depth_max != 1.0 {
        return Err(qualification_error(format!(
            "viewport depth range {}..{} does not match standard zero-to-one device depth",
            render_rect.depth_min, render_rect.depth_max
        )));
    }
    Ok(())
}

fn qualify_input(
    requirement: SsaoInputRequirement,
    stages: &[RenderPassStage],
    ssao_stage_index: usize,
    descriptors: &[RenderFeatureDescriptor],
    scene_allocation_extent: UVec2,
    catalog: &RenderResourceSchemaCatalog<'_>,
) -> Result<QualifiedAmbientOcclusionInput, String> {
    let writers = descriptors
        .iter()
        .flat_map(|descriptor| descriptor.stage_passes.iter())
        .filter(|pass| pass.pass_name != SSAO_EVALUATE_PASS_NAME)
        .filter(|pass| {
            stage_index(stages, pass.stage).is_some_and(|index| index <= ssao_stage_index)
        })
        .filter_map(|pass| {
            pass.resources
                .iter()
                .find(|resource| resource_is_writer(resource, requirement.resource_name))
                .map(|resource| (pass, resource))
        })
        .collect::<Vec<_>>();
    let [(producer, resource)] = writers.as_slice() else {
        return Err(if writers.is_empty() {
            qualification_error(format!(
                "`{}` has no enabled writer before `{SSAO_EVALUATE_PASS_NAME}`",
                requirement.resource_name
            ))
        } else {
            qualification_error(format!(
                "`{}` has {} ambiguous writers before `{SSAO_EVALUATE_PASS_NAME}`",
                requirement.resource_name,
                writers.len()
            ))
        });
    };
    if resource.kind != RenderFeatureResourceKind::Texture {
        return Err(qualification_error(format!(
            "`{}` writer `{}` declares {:?}, expected a texture",
            requirement.resource_name, producer.pass_name, resource.kind
        )));
    }

    let desc = catalog.texture_desc(requirement.resource_name, resource.schema)?;
    validate_input_desc(requirement, &desc, scene_allocation_extent)?;
    Ok(QualifiedAmbientOcclusionInput::new(
        requirement.semantic,
        RenderFeatureResourceVersion::new(
            requirement.resource_name,
            resource.kind,
            producer.pass_name.clone(),
        ),
        desc,
    ))
}

fn resource_is_writer(resource: &RenderFeatureResourceDescriptor, resource_name: &str) -> bool {
    resource.name == resource_name && resource.access == RenderFeatureResourceAccess::Write
}

fn validate_input_desc(
    requirement: SsaoInputRequirement,
    desc: &TextureDesc,
    scene_allocation_extent: UVec2,
) -> Result<(), String> {
    if desc.format != requirement.format {
        return Err(qualification_error(format!(
            "`{}` format {:?} does not match required {:?} {:?} contract",
            requirement.resource_name, desc.format, requirement.semantic, requirement.format
        )));
    }
    if desc.dimension != TextureDimension::D2 {
        return Err(qualification_error(format!(
            "`{}` dimension {:?} is not a 2D AO input",
            requirement.resource_name, desc.dimension
        )));
    }
    if desc.sample_count != 1 {
        return Err(qualification_error(format!(
            "`{}` sample count {} is unsupported; the current AO binding requires 1",
            requirement.resource_name, desc.sample_count
        )));
    }
    if !desc.usage.contains(TextureUsage::SAMPLED) {
        return Err(qualification_error(format!(
            "`{}` does not declare sampled-texture usage",
            requirement.resource_name
        )));
    }

    let (expected_extent, expected_mips) = match requirement.extent {
        SsaoInputExtent::SceneLinear => (scene_allocation_extent, 1),
        SsaoInputExtent::HzbPyramid => {
            let plan = HzbBuilder::new(scene_allocation_extent).build_plan();
            (plan.hzb_size, plan.mip_count)
        }
    };
    if (desc.width, desc.height, desc.mip_levels)
        != (expected_extent.x, expected_extent.y, expected_mips)
    {
        return Err(qualification_error(format!(
            "`{}` physical contract is {}x{} with {} mips; expected {}x{} with {} mips",
            requirement.resource_name,
            desc.width,
            desc.height,
            desc.mip_levels,
            expected_extent.x,
            expected_extent.y,
            expected_mips
        )));
    }
    Ok(())
}

fn qualification_error(message: impl std::fmt::Display) -> String {
    format!("screen_space_ambient_occlusion input qualification failed: {message}")
}

fn feature_requests_ssao(feature: &RendererFeatureAsset) -> bool {
    feature.is_builtin(BuiltinRenderFeature::ScreenSpaceAmbientOcclusion)
        || matches!(
            feature.feature_name().as_str(),
            "screen_space_ambient_occlusion" | "ssao"
        )
}

fn stage_index(stages: &[RenderPassStage], stage: RenderPassStage) -> Option<usize> {
    stages.iter().position(|candidate| *candidate == stage)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_rect_qualification_rejects_unconsumed_partial_rects() {
        let error = validate_render_rect(
            RenderViewportRect::new(UVec2::ZERO, UVec2::new(640, 360)),
            UVec2::new(1280, 720),
        )
        .unwrap_err();

        assert!(error.contains("partial rect AO is unsupported"));
    }
}
