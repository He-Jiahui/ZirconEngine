use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::graphics::pipeline::RenderPassStage;
use crate::render_graph::{
    QueueLane, RenderGraphAttachmentOps, RenderGraphResourceAccessIntent, RenderGraphShaderStages,
    RenderGraphTextureSubresourceRange, RenderResourceSchema, RenderTextureExtentPolicy,
    RenderTextureSchema,
};
use crate::rhi::{TextureFormat, TextureUsage};

use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;

pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor()
-> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "temporal",
        vec!["view".to_string(), "post_process".to_string()],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "velocity-camera",
                QueueLane::Graphics,
            )
            .with_executor_id("temporal.velocity-camera")
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .write_texture_with_ops(
                PostProcessGraphResourceNames::SCENE_VELOCITY,
                RenderGraphAttachmentOps::clear_store(),
            ),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "velocity-object",
                QueueLane::Graphics,
            )
            .with_executor_id("temporal.velocity-object")
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .write_texture_with_ops(
                PostProcessGraphResourceNames::SCENE_VELOCITY,
                RenderGraphAttachmentOps::load_store(),
            ),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "taa-reactive-mask-mesh",
                QueueLane::Graphics,
            )
            .with_executor_id("temporal.taa-reactive-mask-mesh")
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .write_texture_with_ops(
                PostProcessGraphResourceNames::TAA_REACTIVE_MASK,
                RenderGraphAttachmentOps::clear_store(),
            ),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "taa-resolve",
                QueueLane::Graphics,
            )
            .with_executor_id("temporal.taa-resolve")
            .read_texture(PostProcessGraphResourceNames::SCENE_COLOR)
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .read_texture(PostProcessGraphResourceNames::SCENE_VELOCITY)
            .read_persistent_external_texture_with_schema_and_access(
                PostProcessGraphResourceNames::TAA_HISTORY_PREVIOUS,
                taa_history_schema(),
                RenderGraphTextureSubresourceRange::full(),
                RenderGraphResourceAccessIntent::sampled_texture(RenderGraphShaderStages::FRAGMENT),
            )
            .read_texture(PostProcessGraphResourceNames::TAA_REACTIVE_MASK)
            .write_persistent_external_texture_with_schema_and_access(
                PostProcessGraphResourceNames::TAA_HISTORY_CURRENT,
                taa_history_schema(),
                RenderGraphTextureSubresourceRange::full(),
                RenderGraphResourceAccessIntent::ColorAttachment,
            )
            .write_texture_with_ops(
                PostProcessGraphResourceNames::TAA_OUTPUT,
                RenderGraphAttachmentOps::clear_store(),
            ),
        ],
    )
}

fn taa_history_schema() -> RenderResourceSchema {
    RenderResourceSchema::texture(
        RenderTextureSchema::new(
            TextureFormat::Rgba16Float,
            TextureUsage::SAMPLED | TextureUsage::RENDER_ATTACHMENT,
        )
        .with_extent(RenderTextureExtentPolicy::View),
    )
}

#[cfg(test)]
mod tests {
    use super::{descriptor, taa_history_schema};
    use crate::core::framework::render::PostProcessGraphResourceNames;
    use crate::render_graph::{
        RenderGraphAttachmentOps, RenderGraphResourceAccessIntent,
        RenderGraphResourceAccessMetadata, RenderGraphResourceAccessRange, RenderGraphShaderStages,
        RenderGraphTextureSubresourceRange,
    };

    #[test]
    fn temporal_velocity_composes_camera_before_object_motion() {
        let descriptor = descriptor();
        let camera_index = descriptor
            .stage_passes
            .iter()
            .position(|pass| pass.pass_name == "velocity-camera")
            .expect("camera velocity pass");
        let object_index = descriptor
            .stage_passes
            .iter()
            .position(|pass| pass.pass_name == "velocity-object")
            .expect("object velocity pass");

        assert!(camera_index < object_index);
        assert_eq!(
            velocity_attachment_ops(&descriptor.stage_passes[camera_index]),
            RenderGraphAttachmentOps::clear_store()
        );
        assert_eq!(
            velocity_attachment_ops(&descriptor.stage_passes[object_index]),
            RenderGraphAttachmentOps::load_store()
        );
    }

    #[test]
    fn taa_history_slots_declare_exact_external_texture_contracts() {
        let descriptor = descriptor();
        let pass = descriptor
            .stage_passes
            .iter()
            .find(|pass| pass.pass_name == "taa-resolve")
            .expect("TAA resolve pass");
        let previous = pass
            .resources
            .iter()
            .find(|resource| resource.name == PostProcessGraphResourceNames::TAA_HISTORY_PREVIOUS)
            .expect("previous TAA history resource");
        let current = pass
            .resources
            .iter()
            .find(|resource| resource.name == PostProcessGraphResourceNames::TAA_HISTORY_CURRENT)
            .expect("current TAA history resource");
        let full_range =
            RenderGraphResourceAccessRange::Texture(RenderGraphTextureSubresourceRange::full());

        assert_eq!(previous.schema, Some(taa_history_schema()));
        assert_eq!(current.schema, Some(taa_history_schema()));
        assert_eq!(
            previous.access_metadata,
            Some(RenderGraphResourceAccessMetadata::new(
                full_range,
                RenderGraphResourceAccessIntent::sampled_texture(RenderGraphShaderStages::FRAGMENT,),
            ))
        );
        assert_eq!(
            current.access_metadata,
            Some(RenderGraphResourceAccessMetadata::new(
                full_range,
                RenderGraphResourceAccessIntent::ColorAttachment,
            ))
        );
    }

    fn velocity_attachment_ops(
        pass: &crate::graphics::feature::render_feature_pass_descriptor::RenderFeaturePassDescriptor,
    ) -> RenderGraphAttachmentOps {
        pass.resources
            .iter()
            .find(|resource| resource.name == PostProcessGraphResourceNames::SCENE_VELOCITY)
            .and_then(|resource| resource.attachment_ops)
            .expect("scene velocity attachment ops")
    }
}
