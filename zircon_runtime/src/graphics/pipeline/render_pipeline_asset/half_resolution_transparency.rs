use crate::core::framework::render::{PostProcessGraphResourceNames, RenderFrameExtract};
use crate::graphics::feature::{
    RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderFeatureResourceAccess,
};
use crate::graphics::pipeline::{RenderPassStage, RenderPipelineCompileOptions};
use crate::graphics::scene::{
    HALF_RES_TRANSPARENCY_COMPOSITE_EXECUTOR_ID, HALF_RES_TRANSPARENCY_COMPOSITE_PASS_NAME,
    HALF_RES_TRANSPARENCY_DEPTH_DOWNSAMPLE_EXECUTOR_ID,
    HALF_RES_TRANSPARENCY_DEPTH_DOWNSAMPLE_PASS_NAME, HALF_RES_TRANSPARENCY_MESH_EXECUTOR_ID,
    HALF_RES_TRANSPARENCY_MESH_PASS_NAME, HALF_RES_TRANSPARENCY_PARTICLE_EXECUTOR_ID,
    half_resolution_transparency_supported,
};
use crate::render_graph::{QueueLane, RenderGraphAttachmentOps};

pub(super) fn maybe_insert_half_resolution_transparency_passes(
    extract: &RenderFrameExtract,
    options: &RenderPipelineCompileOptions,
    declared_stages: &[RenderPassStage],
    descriptors: &mut Vec<RenderFeatureDescriptor>,
) -> Result<(), String> {
    if !half_resolution_transparency_enabled(extract, options, declared_stages) {
        return Ok(());
    }
    let has_half_resolution_particle_pass = descriptors.iter().any(|descriptor| {
        descriptor
            .stage_passes
            .iter()
            .any(|pass| pass.executor_id.as_str() == HALF_RES_TRANSPARENCY_PARTICLE_EXECUTOR_ID)
    });
    let has_half_resolution_mesh_pass =
        replace_with_half_resolution_transparent_mesh_pass(descriptors)?;
    if !has_half_resolution_particle_pass && !has_half_resolution_mesh_pass {
        return Ok(());
    }

    descriptors.insert(
        0,
        RenderFeatureDescriptor::new(
            "half-resolution-transparency-depth",
            vec!["view".to_string()],
            Vec::new(),
            vec![
                RenderFeaturePassDescriptor::new(
                    RenderPassStage::Transparent3d,
                    HALF_RES_TRANSPARENCY_DEPTH_DOWNSAMPLE_PASS_NAME,
                    QueueLane::Graphics,
                )
                .with_executor_id(HALF_RES_TRANSPARENCY_DEPTH_DOWNSAMPLE_EXECUTOR_ID)
                .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
                .write_texture_with_ops(
                    PostProcessGraphResourceNames::HALF_RES_TRANSPARENCY_COLOR,
                    RenderGraphAttachmentOps::clear_store(),
                )
                .write_texture_with_ops(
                    PostProcessGraphResourceNames::HALF_RES_TRANSPARENCY_DEPTH,
                    RenderGraphAttachmentOps::clear_store(),
                ),
            ],
        ),
    );
    descriptors.push(RenderFeatureDescriptor::new(
        "half-resolution-transparency-composite",
        vec!["view".to_string()],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Transparent3d,
                HALF_RES_TRANSPARENCY_COMPOSITE_PASS_NAME,
                QueueLane::Graphics,
            )
            .with_executor_id(HALF_RES_TRANSPARENCY_COMPOSITE_EXECUTOR_ID)
            .read_texture(PostProcessGraphResourceNames::HALF_RES_TRANSPARENCY_COLOR)
            .read_texture(PostProcessGraphResourceNames::HALF_RES_TRANSPARENCY_DEPTH)
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .write_texture_with_ops(
                PostProcessGraphResourceNames::SCENE_COLOR,
                RenderGraphAttachmentOps::load_store(),
            ),
        ],
    ));
    Ok(())
}

pub(super) fn half_resolution_transparency_enabled(
    extract: &RenderFrameExtract,
    options: &RenderPipelineCompileOptions,
    declared_stages: &[RenderPassStage],
) -> bool {
    options.enable_half_resolution_transparency
        && declared_stages.contains(&RenderPassStage::Transparent3d)
        && half_resolution_transparency_supported(
            options.graph_msaa_sample_count(extract.view.camera.msaa_samples),
        )
}

fn replace_with_half_resolution_transparent_mesh_pass(
    descriptors: &mut [RenderFeatureDescriptor],
) -> Result<bool, String> {
    let (descriptor_index, pass_index, mut half_resolution_pass) = {
        let mut owners =
            descriptors
                .iter()
                .enumerate()
                .filter_map(|(descriptor_index, descriptor)| {
                    descriptor
                        .stage_passes
                        .iter()
                        .enumerate()
                        .find(|(_, pass)| pass.executor_id.as_str() == "mesh.transparent")
                        .map(|(pass_index, pass)| (descriptor_index, pass_index, pass))
                });
        let Some((descriptor_index, pass_index, transparent_template)) = owners.next() else {
            return Ok(false);
        };
        if owners.next().is_some() {
            return Err(
                "half-resolution transparency requires exactly one mesh.transparent pass owner"
                    .to_string(),
            );
        }
        (descriptor_index, pass_index, transparent_template.clone())
    };

    half_resolution_pass.pass_name = HALF_RES_TRANSPARENCY_MESH_PASS_NAME.to_string();
    half_resolution_pass.executor_id = HALF_RES_TRANSPARENCY_MESH_EXECUTOR_ID.into();
    let mut mapped_color = false;
    let mut mapped_color_write = false;
    let mut mapped_depth = false;
    for resource in &mut half_resolution_pass.resources {
        match resource.name.as_str() {
            PostProcessGraphResourceNames::SCENE_COLOR => {
                resource.name =
                    PostProcessGraphResourceNames::HALF_RES_TRANSPARENCY_COLOR.to_string();
                mapped_color = true;
                if resource.access == RenderFeatureResourceAccess::Write {
                    mapped_color_write = true;
                    resource.attachment_ops = Some(RenderGraphAttachmentOps::load_store());
                }
            }
            PostProcessGraphResourceNames::SCENE_DEPTH => {
                resource.name =
                    PostProcessGraphResourceNames::HALF_RES_TRANSPARENCY_DEPTH.to_string();
                mapped_depth = true;
            }
            _ => {}
        }
    }
    if !mapped_color || !mapped_color_write || !mapped_depth {
        return Ok(false);
    }

    descriptors[descriptor_index].stage_passes[pass_index] = half_resolution_pass;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn streamed_half_resolution_owner_selection_preserves_cardinality_contract() {
        let mut empty = Vec::new();
        assert!(!replace_with_half_resolution_transparent_mesh_pass(&mut empty).unwrap());

        let mut unique = vec![transparent_owner("unique")];
        assert!(replace_with_half_resolution_transparent_mesh_pass(&mut unique).unwrap());
        assert_eq!(
            unique[0].stage_passes[0].executor_id.as_str(),
            HALF_RES_TRANSPARENCY_MESH_EXECUTOR_ID
        );

        let mut duplicate = vec![transparent_owner("first"), transparent_owner("second")];
        let duplicate_error =
            replace_with_half_resolution_transparent_mesh_pass(&mut duplicate).unwrap_err();
        assert!(duplicate_error.contains("exactly one"));
    }

    fn transparent_owner(name: &str) -> RenderFeatureDescriptor {
        RenderFeatureDescriptor::new(
            name,
            Vec::new(),
            Vec::new(),
            vec![
                RenderFeaturePassDescriptor::new(
                    RenderPassStage::Transparent3d,
                    "transparent-mesh",
                    QueueLane::Graphics,
                )
                .with_executor_id("mesh.transparent")
                .write_texture_with_ops(
                    PostProcessGraphResourceNames::SCENE_COLOR,
                    RenderGraphAttachmentOps::load_store(),
                )
                .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH),
            ],
        )
    }
}
