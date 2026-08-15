use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::render_graph::RenderGraphResourceAccessKind;

use super::super::RenderPassExecutionContext;

pub(super) fn output_transfer_output_resource(
    _context: &RenderPassExecutionContext<'_>,
) -> &'static str {
    PostProcessGraphResourceNames::FINAL_COLOR
}

pub(super) fn output_transfer_input_resource(
    context: &RenderPassExecutionContext<'_>,
) -> &'static str {
    if context.declares_resource_name_access(
        PostProcessGraphResourceNames::UPSCALED,
        RenderGraphResourceAccessKind::Read,
    ) {
        PostProcessGraphResourceNames::UPSCALED
    } else if context.declares_resource_name_access(
        PostProcessGraphResourceNames::FINAL_COMPOSITED,
        RenderGraphResourceAccessKind::Read,
    ) {
        PostProcessGraphResourceNames::FINAL_COMPOSITED
    } else {
        PostProcessGraphResourceNames::TONEMAPPED
    }
}

pub(super) fn upscale_input_resource(context: &RenderPassExecutionContext<'_>) -> &'static str {
    if context.declares_resource_name_access(
        PostProcessGraphResourceNames::FINAL_COMPOSITED,
        RenderGraphResourceAccessKind::Read,
    ) {
        PostProcessGraphResourceNames::FINAL_COMPOSITED
    } else {
        PostProcessGraphResourceNames::TONEMAPPED
    }
}

pub(super) fn terminal_anti_alias_input_resource(
    context: &RenderPassExecutionContext<'_>,
) -> &'static str {
    if context.declares_resource_name_access(
        PostProcessGraphResourceNames::TONEMAPPED,
        RenderGraphResourceAccessKind::Read,
    ) {
        PostProcessGraphResourceNames::TONEMAPPED
    } else {
        PostProcessGraphResourceNames::FINAL_COMPOSITED
    }
}

pub(super) fn bloom_input_resource(context: &RenderPassExecutionContext<'_>) -> &'static str {
    if context.declares_resource_name_access(
        PostProcessGraphResourceNames::MOTION_BLURRED,
        RenderGraphResourceAccessKind::Read,
    ) {
        PostProcessGraphResourceNames::MOTION_BLURRED
    } else if context.declares_resource_name_access(
        PostProcessGraphResourceNames::DEPTH_OF_FIELDED,
        RenderGraphResourceAccessKind::Read,
    ) {
        PostProcessGraphResourceNames::DEPTH_OF_FIELDED
    } else if context.declares_resource_name_access(
        PostProcessGraphResourceNames::TAA_OUTPUT,
        RenderGraphResourceAccessKind::Read,
    ) {
        PostProcessGraphResourceNames::TAA_OUTPUT
    } else {
        PostProcessGraphResourceNames::SCENE_COLOR
    }
}

#[cfg(test)]
pub(super) fn uber_input_resource(context: &RenderPassExecutionContext<'_>) -> &'static str {
    if context.declares_resource_name_access(
        PostProcessGraphResourceNames::BLURRED,
        RenderGraphResourceAccessKind::Read,
    ) {
        PostProcessGraphResourceNames::BLURRED
    } else if context.declares_resource_name_access(
        PostProcessGraphResourceNames::SCENE_COMPOSITED,
        RenderGraphResourceAccessKind::Read,
    ) {
        PostProcessGraphResourceNames::SCENE_COMPOSITED
    } else if context.declares_resource_name_access(
        PostProcessGraphResourceNames::MOTION_BLURRED,
        RenderGraphResourceAccessKind::Read,
    ) {
        PostProcessGraphResourceNames::MOTION_BLURRED
    } else if context.declares_resource_name_access(
        PostProcessGraphResourceNames::DEPTH_OF_FIELDED,
        RenderGraphResourceAccessKind::Read,
    ) {
        PostProcessGraphResourceNames::DEPTH_OF_FIELDED
    } else if context.declares_resource_name_access(
        PostProcessGraphResourceNames::TAA_OUTPUT,
        RenderGraphResourceAccessKind::Read,
    ) {
        PostProcessGraphResourceNames::TAA_OUTPUT
    } else {
        PostProcessGraphResourceNames::SCENE_COLOR
    }
}

#[cfg(test)]
mod tests {
    use super::{
        bloom_input_resource, output_transfer_input_resource, output_transfer_output_resource,
        terminal_anti_alias_input_resource, uber_input_resource, upscale_input_resource,
    };
    use crate::core::framework::render::PostProcessGraphResourceNames;
    use crate::graphics::RenderPassExecutorId;
    use crate::render_graph::{
        PassFlags, QueueLane, RenderGraphPassResourceAccess, RenderGraphResourceAccessKind,
        RenderGraphResourceKind,
    };

    use super::super::super::RenderPassExecutionContext;

    #[test]
    fn output_transfer_executor_always_targets_final_color() {
        let context = RenderPassExecutionContext::with_graph_metadata_and_resources(
            "output-transfer",
            RenderPassExecutorId::new("post.output-transfer"),
            QueueLane::Graphics,
            PassFlags::default(),
            vec![RenderGraphPassResourceAccess {
                name: PostProcessGraphResourceNames::FINAL_COMPOSITED.to_string(),
                kind: RenderGraphResourceKind::TransientTexture,
                access: RenderGraphResourceAccessKind::Read,
                attachment_ops: None,
            }],
        );

        assert_eq!(
            output_transfer_output_resource(&context),
            PostProcessGraphResourceNames::FINAL_COLOR
        );
    }

    #[test]
    fn output_transfer_executor_defaults_to_final_color() {
        let context = RenderPassExecutionContext::with_graph_metadata_and_resources(
            "output-transfer",
            RenderPassExecutorId::new("post.output-transfer"),
            QueueLane::Graphics,
            PassFlags::default(),
            Vec::new(),
        );

        assert_eq!(
            output_transfer_output_resource(&context),
            PostProcessGraphResourceNames::FINAL_COLOR
        );
    }

    #[test]
    fn output_transfer_executor_reads_upscaled_input_when_declared() {
        let context = RenderPassExecutionContext::with_graph_metadata_and_resources(
            "output-transfer",
            RenderPassExecutorId::new("post.output-transfer"),
            QueueLane::Graphics,
            PassFlags::default(),
            vec![RenderGraphPassResourceAccess {
                name: PostProcessGraphResourceNames::UPSCALED.to_string(),
                kind: RenderGraphResourceKind::TransientTexture,
                access: RenderGraphResourceAccessKind::Read,
                attachment_ops: None,
            }],
        );

        assert_eq!(
            output_transfer_input_resource(&context),
            PostProcessGraphResourceNames::UPSCALED
        );
    }

    #[test]
    fn output_transfer_executor_reads_terminal_anti_alias_result_when_declared() {
        let context = RenderPassExecutionContext::with_graph_metadata_and_resources(
            "output-transfer",
            RenderPassExecutorId::new("post.output-transfer"),
            QueueLane::Graphics,
            PassFlags::default(),
            vec![RenderGraphPassResourceAccess {
                name: PostProcessGraphResourceNames::FINAL_COMPOSITED.to_string(),
                kind: RenderGraphResourceKind::TransientTexture,
                access: RenderGraphResourceAccessKind::Read,
                attachment_ops: None,
            }],
        );

        assert_eq!(
            output_transfer_input_resource(&context),
            PostProcessGraphResourceNames::FINAL_COMPOSITED
        );
    }

    #[test]
    fn upscale_executor_reads_terminal_anti_alias_result_when_declared() {
        let context = RenderPassExecutionContext::with_graph_metadata_and_resources(
            "upscale",
            RenderPassExecutorId::new("post.upscale"),
            QueueLane::Graphics,
            PassFlags::default(),
            vec![RenderGraphPassResourceAccess {
                name: PostProcessGraphResourceNames::FINAL_COMPOSITED.to_string(),
                kind: RenderGraphResourceKind::TransientTexture,
                access: RenderGraphResourceAccessKind::Read,
                attachment_ops: None,
            }],
        );

        assert_eq!(
            upscale_input_resource(&context),
            PostProcessGraphResourceNames::FINAL_COMPOSITED
        );
    }

    #[test]
    fn terminal_anti_alias_executor_reads_tonemapped_input_when_declared() {
        let context = RenderPassExecutionContext::with_graph_metadata_and_resources(
            "fxaa",
            RenderPassExecutorId::new("post.fxaa"),
            QueueLane::Graphics,
            PassFlags::default(),
            vec![RenderGraphPassResourceAccess {
                name: PostProcessGraphResourceNames::TONEMAPPED.to_string(),
                kind: RenderGraphResourceKind::TransientTexture,
                access: RenderGraphResourceAccessKind::Read,
                attachment_ops: None,
            }],
        );

        assert_eq!(
            terminal_anti_alias_input_resource(&context),
            PostProcessGraphResourceNames::TONEMAPPED
        );
    }

    #[test]
    fn output_transfer_executor_defaults_to_tonemapped_input() {
        let context = RenderPassExecutionContext::with_graph_metadata_and_resources(
            "output-transfer",
            RenderPassExecutorId::new("post.output-transfer"),
            QueueLane::Graphics,
            PassFlags::default(),
            Vec::new(),
        );

        assert_eq!(
            output_transfer_input_resource(&context),
            PostProcessGraphResourceNames::TONEMAPPED
        );
    }

    #[test]
    fn bloom_executor_reads_motion_blurred_source_when_declared() {
        let context = RenderPassExecutionContext::with_graph_metadata_and_resources(
            "bloom-extract",
            RenderPassExecutorId::new("post.bloom-extract"),
            QueueLane::Graphics,
            PassFlags::default(),
            vec![RenderGraphPassResourceAccess {
                name: PostProcessGraphResourceNames::MOTION_BLURRED.to_string(),
                kind: RenderGraphResourceKind::TransientTexture,
                access: RenderGraphResourceAccessKind::Read,
                attachment_ops: None,
            }],
        );

        assert_eq!(
            bloom_input_resource(&context),
            PostProcessGraphResourceNames::MOTION_BLURRED
        );
    }

    #[test]
    fn bloom_executor_falls_back_to_scene_color_input() {
        let context = RenderPassExecutionContext::with_graph_metadata_and_resources(
            "bloom-extract",
            RenderPassExecutorId::new("post.bloom-extract"),
            QueueLane::Graphics,
            PassFlags::default(),
            Vec::new(),
        );

        assert_eq!(
            bloom_input_resource(&context),
            PostProcessGraphResourceNames::SCENE_COLOR
        );
    }

    #[test]
    fn uber_executor_reads_scene_composited_source_when_declared() {
        let context = RenderPassExecutionContext::with_graph_metadata_and_resources(
            "uber",
            RenderPassExecutorId::new("post.uber"),
            QueueLane::Graphics,
            PassFlags::default(),
            vec![RenderGraphPassResourceAccess {
                name: PostProcessGraphResourceNames::SCENE_COMPOSITED.to_string(),
                kind: RenderGraphResourceKind::TransientTexture,
                access: RenderGraphResourceAccessKind::Read,
                attachment_ops: None,
            }],
        );

        assert_eq!(
            uber_input_resource(&context),
            PostProcessGraphResourceNames::SCENE_COMPOSITED
        );
    }

    #[test]
    fn uber_executor_reads_blurred_source_when_declared() {
        let context = RenderPassExecutionContext::with_graph_metadata_and_resources(
            "uber",
            RenderPassExecutorId::new("post.uber"),
            QueueLane::Graphics,
            PassFlags::default(),
            vec![RenderGraphPassResourceAccess {
                name: PostProcessGraphResourceNames::BLURRED.to_string(),
                kind: RenderGraphResourceKind::TransientTexture,
                access: RenderGraphResourceAccessKind::Read,
                attachment_ops: None,
            }],
        );

        assert_eq!(
            uber_input_resource(&context),
            PostProcessGraphResourceNames::BLURRED
        );
    }
}
