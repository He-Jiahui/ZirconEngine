use std::collections::BTreeMap;

use crate::graphics::feature::{
    RenderFeatureDescriptor, RenderFeatureResourceAccess, RenderFeatureResourceDescriptor,
    RenderFeatureResourceKind, RenderFeatureResourceVersion, RenderFeatureResourceWriteMode,
};
use crate::graphics::pipeline::declarations::RenderPassStage;
use crate::render_graph::{RenderGraphAttachmentLoadOp, RenderGraphAttachmentOps};

use super::pass_authoring::ordered_render_feature_passes;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ResourceIdentity {
    name: String,
    kind: RenderFeatureResourceKind,
}

impl ResourceIdentity {
    fn from_resource(resource: &RenderFeatureResourceDescriptor) -> Self {
        Self {
            name: resource.name.clone(),
            kind: resource.kind,
        }
    }
}

/// Resolves the legacy implicit attachment initialization rule before graph authoring.
///
/// The resulting descriptor carries concrete load/store operations. Internal texture
/// loads also carry the producer token that supplied their previous contents. Physical
/// version-to-view resolution remains the responsibility of the compiled binding table.
pub(super) fn normalize_attachment_initialization(
    stages: &[RenderPassStage],
    descriptors: &mut [RenderFeatureDescriptor],
) -> Result<(), String> {
    let pass_locations = pass_locations(descriptors)?;
    let ordered_passes = ordered_render_feature_passes(stages, descriptors)?;
    let mut latest_producers = BTreeMap::<ResourceIdentity, RenderFeatureResourceVersion>::new();

    for ordered_pass in ordered_passes {
        let (descriptor_index, pass_index) = pass_locations
            .get(ordered_pass.pass_name.as_str())
            .copied()
            .ok_or_else(|| {
                format!(
                    "normalized render pass `{}` is absent from the feature descriptors",
                    ordered_pass.pass_name
                )
            })?;
        let pass = &mut descriptors[descriptor_index].stage_passes[pass_index];
        let pass_name = pass.pass_name.clone();

        for resource in &mut pass.resources {
            let identity = ResourceIdentity::from_resource(resource);
            let previous_producer = latest_producers.get(&identity).cloned();
            if resource.access == RenderFeatureResourceAccess::Write
                && resource.write_mode == RenderFeatureResourceWriteMode::Attachment
                && matches!(
                    resource.kind,
                    RenderFeatureResourceKind::Texture | RenderFeatureResourceKind::External
                )
            {
                normalize_attachment_write(&pass_name, resource, previous_producer.as_ref())?;
            }

            if resource.access == RenderFeatureResourceAccess::Write {
                latest_producers.insert(
                    identity,
                    RenderFeatureResourceVersion::new(
                        resource.name.clone(),
                        resource.kind,
                        pass_name.clone(),
                    ),
                );
            }
        }
    }

    Ok(())
}

fn pass_locations(
    descriptors: &[RenderFeatureDescriptor],
) -> Result<BTreeMap<String, (usize, usize)>, String> {
    let mut locations = BTreeMap::new();
    for (descriptor_index, descriptor) in descriptors.iter().enumerate() {
        for (pass_index, pass) in descriptor.stage_passes.iter().enumerate() {
            if locations
                .insert(pass.pass_name.clone(), (descriptor_index, pass_index))
                .is_some()
            {
                return Err(format!(
                    "duplicate render graph pass name `{}` prevents attachment initialization normalization",
                    pass.pass_name
                ));
            }
        }
    }
    Ok(locations)
}

fn normalize_attachment_write(
    pass_name: &str,
    resource: &mut RenderFeatureResourceDescriptor,
    previous_producer: Option<&RenderFeatureResourceVersion>,
) -> Result<(), String> {
    match resource.attachment_ops {
        None if resource.input_version.is_some() => {
            return Err(format!(
                "render pass `{pass_name}` declares a producer version for attachment `{}` without a Load operation",
                resource.name
            ));
        }
        None if resource.kind == RenderFeatureResourceKind::External => {
            resource.attachment_ops = Some(RenderGraphAttachmentOps::load_store());
            normalize_attachment_load(pass_name, resource, previous_producer)?;
        }
        None => match previous_producer {
            Some(producer) => {
                resource.attachment_ops = Some(RenderGraphAttachmentOps::load_store());
                resource.input_version = Some(producer.clone());
            }
            None => resource.attachment_ops = Some(RenderGraphAttachmentOps::clear_store()),
        },
        Some(ops) if ops.load == RenderGraphAttachmentLoadOp::Load => {
            normalize_attachment_load(pass_name, resource, previous_producer)?;
        }
        Some(_) if resource.input_version.is_some() => {
            return Err(format!(
                "render pass `{pass_name}` clears attachment `{}` but declares a producer version",
                resource.name
            ));
        }
        Some(_) => {}
    }
    Ok(())
}

fn normalize_attachment_load(
    pass_name: &str,
    resource: &mut RenderFeatureResourceDescriptor,
    previous_producer: Option<&RenderFeatureResourceVersion>,
) -> Result<(), String> {
    let external_initial_contents = resource.kind == RenderFeatureResourceKind::External
        && previous_producer.is_none()
        && resource.input_version.is_none();
    if external_initial_contents {
        return Ok(());
    }

    let producer = previous_producer.ok_or_else(|| {
        format!(
            "render pass `{pass_name}` loads attachment `{}` with no prior producer",
            resource.name
        )
    })?;
    match &resource.input_version {
        Some(input_version) if input_version == producer => Ok(()),
        Some(input_version) => Err(format!(
            "render pass `{pass_name}` loads attachment `{}` from producer `{}`, but the normalized prior producer is `{}`",
            resource.name,
            input_version.producer_pass_name(),
            producer.producer_pass_name(),
        )),
        None => {
            resource.input_version = Some(producer.clone());
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::graphics::feature::RenderFeaturePassDescriptor;
    use crate::render_graph::QueueLane;

    use super::*;

    #[test]
    fn normalizes_sequential_attachment_writes_to_clear_then_load() {
        let mut descriptors = vec![RenderFeatureDescriptor::new(
            "attachment-initialization",
            Vec::new(),
            Vec::new(),
            vec![
                RenderFeaturePassDescriptor::new(
                    RenderPassStage::PostProcess,
                    "attachment-seed",
                    QueueLane::Graphics,
                )
                .with_executor_id("test.attachment-seed")
                .write_texture("attachment-color"),
                RenderFeaturePassDescriptor::new(
                    RenderPassStage::PostProcess,
                    "attachment-compose",
                    QueueLane::Graphics,
                )
                .with_executor_id("test.attachment-compose")
                .write_texture("attachment-color"),
            ],
        )];

        normalize_attachment_initialization(&[RenderPassStage::PostProcess], &mut descriptors)
            .expect("sequential attachment writes normalize");

        let seed = &descriptors[0].stage_passes[0].resources[0];
        let compose = &descriptors[0].stage_passes[1].resources[0];
        assert_eq!(
            seed.attachment_ops,
            Some(RenderGraphAttachmentOps::clear_store())
        );
        assert_eq!(
            compose.attachment_ops,
            Some(RenderGraphAttachmentOps::load_store())
        );
        assert_eq!(
            compose
                .input_version
                .as_ref()
                .map(|version| version.producer_pass_name()),
            Some("attachment-seed")
        );
    }

    #[test]
    fn preserves_external_attachment_initial_contents_as_load() {
        let mut descriptors = vec![RenderFeatureDescriptor::new(
            "external-attachment-initialization",
            Vec::new(),
            Vec::new(),
            vec![
                RenderFeaturePassDescriptor::new(
                    RenderPassStage::PostProcess,
                    "external-compose",
                    QueueLane::Graphics,
                )
                .with_executor_id("test.external-compose")
                .write_external_texture("external-color"),
            ],
        )];

        normalize_attachment_initialization(&[RenderPassStage::PostProcess], &mut descriptors)
            .expect("external attachment writes normalize");

        let attachment = &descriptors[0].stage_passes[0].resources[0];
        assert_eq!(
            attachment.attachment_ops,
            Some(RenderGraphAttachmentOps::load_store())
        );
        assert!(attachment.input_version.is_none());
    }

    #[test]
    fn rejects_a_producer_token_without_an_attachment_load_operation() {
        let mut descriptors = vec![RenderFeatureDescriptor::new(
            "attachment-token-without-load",
            Vec::new(),
            Vec::new(),
            vec![
                RenderFeaturePassDescriptor::new(
                    RenderPassStage::PostProcess,
                    "attachment-token-consumer",
                    QueueLane::Graphics,
                )
                .with_executor_id("test.attachment-token-consumer")
                .write_texture("attachment-color"),
            ],
        )];
        descriptors[0].stage_passes[0].resources[0].input_version =
            Some(RenderFeatureResourceVersion::new(
                "attachment-color",
                RenderFeatureResourceKind::Texture,
                "attachment-seed",
            ));

        let error =
            normalize_attachment_initialization(&[RenderPassStage::PostProcess], &mut descriptors)
                .expect_err("a producer token requires an attachment load operation");

        assert!(error.contains("without a Load operation"), "{error}");
    }
}
