use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::render_graph::{QueueLane, RenderGraphAttachmentOps};

use crate::graphics::pipeline::RenderPassStage;

use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;

pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor(
) -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "shadows",
        vec![
            "view".to_string(),
            "geometry".to_string(),
            "lighting".to_string(),
            "visibility".to_string(),
        ],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::Shadow,
            "shadow-atlas",
            QueueLane::Graphics,
        )
        .with_executor_id("shadow.atlas")
        .with_side_effects()
        .write_required_external_texture_with_ops(
            PostProcessGraphResourceNames::SHADOW_ATLAS,
            RenderGraphAttachmentOps::clear_store(),
        )],
    )
}

#[cfg(test)]
mod tests {
    use super::super::super::render_feature_pass_descriptor::{
        RenderFeatureResourceAccess, RenderFeatureResourceKind,
    };
    use super::*;
    use crate::render_graph::{RenderGraphAttachmentOps, RenderGraphExternalResourceBinding};

    #[test]
    fn shadow_atlas_pass_declares_required_external_texture_write() {
        let descriptor = descriptor();
        let pass = descriptor
            .stage_passes
            .iter()
            .find(|pass| pass.pass_name == "shadow-atlas")
            .expect("shadow atlas pass");

        let atlas = pass
            .resources
            .iter()
            .find(|resource| resource.name == PostProcessGraphResourceNames::SHADOW_ATLAS)
            .expect("shadow atlas resource");

        assert_eq!(atlas.kind, RenderFeatureResourceKind::External);
        assert_eq!(atlas.access, RenderFeatureResourceAccess::Write);
        assert_eq!(
            atlas.attachment_ops,
            Some(RenderGraphAttachmentOps::clear_store())
        );
        assert_eq!(
            atlas.external_binding,
            RenderGraphExternalResourceBinding::required_texture()
        );
    }
}
