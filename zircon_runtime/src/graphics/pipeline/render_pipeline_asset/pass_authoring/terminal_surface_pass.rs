use crate::core::framework::render::{
    PostProcessGraphResourceNames, RenderCameraTarget, RenderFrameExtract,
};
use crate::graphics::pipeline::declarations::{
    OUTPUT_TARGET_DIRECT_IMPORT_EXECUTOR_ID, OUTPUT_TARGET_DIRECT_IMPORT_PASS_NAME,
    OUTPUT_TARGET_TEXTURE_RESOURCE_NAME, OUTPUT_TARGET_WRITEBACK_EXECUTOR_ID,
    OUTPUT_TARGET_WRITEBACK_PASS_NAME, RenderGraphExecutionPassMetadata, RenderPassStage,
    SURFACE_PRESENT_EXECUTOR_ID, SURFACE_PRESENT_PASS_NAME,
};
use crate::graphics::pipeline::{
    RenderGraphCompileCameraTargetFingerprint, RenderGraphCompileTextureTargetFormat,
};
use crate::render_graph::{
    PassFlags, QueueLane, RenderGraphAttachmentOps, RenderGraphBuilder,
    RenderGraphExternalResourceBinding, RenderGraphResourceAccessIntent,
    RenderGraphResourceAccessKind, RenderGraphResourceAccessRange, RenderGraphShaderStages,
    RenderGraphTextureSubresourceRange,
};
use crate::rhi::{TextureDesc, TextureUsage};

use super::AuthoredGraphResources;

pub(super) fn author_terminal_surface_pass(
    graph: &mut RenderGraphBuilder,
    mut execution_pass_metadata: Vec<RenderGraphExecutionPassMetadata>,
    resources: &AuthoredGraphResources,
    extract: &RenderFrameExtract,
    camera_target: RenderGraphCompileCameraTargetFingerprint,
) -> Result<Vec<RenderGraphExecutionPassMetadata>, String> {
    let selected_camera = extract
        .view
        .selected_camera_descriptor()
        .ok_or_else(|| "surface-present authoring requires a selected camera".to_string())?;
    let Some(source_name) = [
        PostProcessGraphResourceNames::VIEWPORT_OUTPUT,
        PostProcessGraphResourceNames::FINAL_COLOR,
    ]
    .into_iter()
    .find(|name| resources.external_resources.contains_key(*name)) else {
        return Ok(execution_pass_metadata);
    };
    match (&selected_camera.target, camera_target) {
        (
            RenderCameraTarget::PrimarySurface,
            RenderGraphCompileCameraTargetFingerprint::PrimarySurface,
        ) => author_surface_present_pass(graph, execution_pass_metadata, resources, source_name),
        (
            RenderCameraTarget::Texture(_),
            RenderGraphCompileCameraTargetFingerprint::Texture {
                width,
                height,
                format,
                ..
            },
        ) => match format {
            RenderGraphCompileTextureTargetFormat::Rgba8UnormSrgb => {
                author_output_target_direct_import_pass(
                    graph,
                    execution_pass_metadata,
                    resources,
                    source_name,
                )
            }
            RenderGraphCompileTextureTargetFormat::Rgba8Unorm => {
                author_output_target_writeback_pass(
                    graph,
                    execution_pass_metadata,
                    resources,
                    source_name,
                    width,
                    height,
                )
            }
        },
        (
            RenderCameraTarget::Headless { .. },
            RenderGraphCompileCameraTargetFingerprint::Headless { .. },
        ) => Ok(execution_pass_metadata),
        _ => Err(
            "resolved camera target fingerprint does not match the selected camera target"
                .to_string(),
        ),
    }
}

fn author_output_target_direct_import_pass(
    graph: &mut RenderGraphBuilder,
    mut execution_pass_metadata: Vec<RenderGraphExecutionPassMetadata>,
    resources: &AuthoredGraphResources,
    source_name: &str,
) -> Result<Vec<RenderGraphExecutionPassMetadata>, String> {
    let pass = graph.add_pass_with_executor_and_declared_queue(
        OUTPUT_TARGET_DIRECT_IMPORT_PASS_NAME,
        QueueLane::Graphics,
        QueueLane::Graphics,
        Some(OUTPUT_TARGET_DIRECT_IMPORT_EXECUTOR_ID),
    );
    graph
        .set_pass_flags(
            pass,
            PassFlags {
                allow_culling: false,
                has_side_effects: true,
            },
        )
        .map_err(|error| error.to_string())?;
    graph
        .read_external(pass, resources.external_resources[source_name])
        .map_err(|error| error.to_string())?;
    execution_pass_metadata.push(RenderGraphExecutionPassMetadata::new(
        pass,
        RenderPassStage::Present,
    ));
    Ok(execution_pass_metadata)
}

fn author_surface_present_pass(
    graph: &mut RenderGraphBuilder,
    mut execution_pass_metadata: Vec<RenderGraphExecutionPassMetadata>,
    resources: &AuthoredGraphResources,
    source_name: &str,
) -> Result<Vec<RenderGraphExecutionPassMetadata>, String> {
    let pass = graph.add_pass_with_executor_and_declared_queue(
        SURFACE_PRESENT_PASS_NAME,
        QueueLane::Graphics,
        QueueLane::Graphics,
        Some(SURFACE_PRESENT_EXECUTOR_ID),
    );
    graph
        .set_pass_flags(
            pass,
            PassFlags {
                allow_culling: false,
                has_side_effects: true,
            },
        )
        .map_err(|error| error.to_string())?;
    graph
        .read_external(pass, resources.external_resources[source_name])
        .map_err(|error| error.to_string())?;
    execution_pass_metadata.push(RenderGraphExecutionPassMetadata::new(
        pass,
        RenderPassStage::Present,
    ));
    Ok(execution_pass_metadata)
}

fn author_output_target_writeback_pass(
    graph: &mut RenderGraphBuilder,
    mut execution_pass_metadata: Vec<RenderGraphExecutionPassMetadata>,
    resources: &AuthoredGraphResources,
    source_name: &str,
    width: u32,
    height: u32,
) -> Result<Vec<RenderGraphExecutionPassMetadata>, String> {
    let output_target = graph.import_present_external_texture_with_binding(
        OUTPUT_TARGET_TEXTURE_RESOURCE_NAME,
        TextureDesc::new(
            OUTPUT_TARGET_TEXTURE_RESOURCE_NAME,
            width.max(1),
            height.max(1),
            RenderGraphCompileTextureTargetFormat::Rgba8Unorm.as_rhi_format(),
            TextureUsage::RENDER_ATTACHMENT,
        ),
        RenderGraphExternalResourceBinding::report_only_texture(),
    );
    let pass = graph.add_pass_with_executor_and_declared_queue(
        OUTPUT_TARGET_WRITEBACK_PASS_NAME,
        QueueLane::Graphics,
        QueueLane::Graphics,
        Some(OUTPUT_TARGET_WRITEBACK_EXECUTOR_ID),
    );
    graph
        .set_pass_flags(
            pass,
            PassFlags {
                allow_culling: false,
                has_side_effects: true,
            },
        )
        .map_err(|error| error.to_string())?;
    graph
        .access_external(
            pass,
            resources.external_resources[source_name],
            RenderGraphResourceAccessKind::Read,
            RenderGraphResourceAccessRange::Texture(RenderGraphTextureSubresourceRange::full()),
            RenderGraphResourceAccessIntent::sampled_texture(RenderGraphShaderStages::FRAGMENT),
            None,
        )
        .map_err(|error| error.to_string())?;
    graph
        .access_external(
            pass,
            output_target,
            RenderGraphResourceAccessKind::Write,
            RenderGraphResourceAccessRange::Texture(RenderGraphTextureSubresourceRange::full()),
            RenderGraphResourceAccessIntent::ColorAttachment,
            Some(RenderGraphAttachmentOps::clear_store()),
        )
        .map_err(|error| error.to_string())?;
    execution_pass_metadata.push(RenderGraphExecutionPassMetadata::new(
        pass,
        RenderPassStage::Present,
    ));
    Ok(execution_pass_metadata)
}

#[cfg(test)]
mod tests {
    #[test]
    fn terminal_outputs_are_non_cullable_graph_side_effects() {
        let source = include_str!("terminal_surface_pass.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("surface-present test boundary");

        assert!(source.contains("RenderCameraTarget::PrimarySurface"));
        assert!(source.contains("RenderPassStage::Present"));
        assert!(source.contains("allow_culling: false"));
        assert!(source.contains("has_side_effects: true"));
        assert!(source.contains("graph.read_external("));
        assert!(source.contains("author_output_target_direct_import_pass("));
        assert!(source.contains("OUTPUT_TARGET_DIRECT_IMPORT_EXECUTOR_ID"));
        assert!(source.contains("import_present_external_texture_with_binding"));
        assert!(source.contains("RenderGraphResourceAccessIntent::sampled_texture"));
        assert!(source.contains("RenderGraphResourceAccessIntent::ColorAttachment"));
        assert!(!source.contains("RenderGraphResourceAccessIntent::CopySource"));
        assert!(!source.contains("RenderGraphResourceAccessIntent::CopyDestination"));
    }
}
