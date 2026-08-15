use crate::graphics::feature::{
    RenderFeaturePassDescriptor, RenderFeatureResourceAccess, RenderFeatureResourceDescriptor,
    RenderFeatureResourceKind, RenderFeatureResourceWriteMode,
};
use crate::graphics::scene::RenderPassExecutorId;
use crate::render_graph::{
    ComputeBindingKind, RenderGraphComputeDispatchExtent, RenderGraphComputeWorkload,
    RenderGraphExternalResourceBinding,
};

use super::ComputePassDescriptor;

pub const COMPUTE_GENERIC_EXECUTOR_ID: &str = "compute.generic";

impl ComputePassDescriptor {
    pub fn into_feature_pass(self) -> RenderFeaturePassDescriptor {
        let pass = RenderFeaturePassDescriptor::new(self.stage, self.pass_name.clone(), self.queue);
        self.lower_into(pass)
    }

    pub(crate) fn lower_into(
        self,
        mut pass: RenderFeaturePassDescriptor,
    ) -> RenderFeaturePassDescriptor {
        debug_assert_eq!(pass.stage, self.stage);
        debug_assert_eq!(pass.pass_name, self.pass_name);
        debug_assert_eq!(pass.queue, self.queue);
        pass.executor_id = RenderPassExecutorId::new(COMPUTE_GENERIC_EXECUTOR_ID);
        pass.flags = self.flags;
        pass.compute_workload = Some(RenderGraphComputeWorkload::new(
            self.shader.pipeline_label(),
            self.workgroup_size,
            self.dispatch.clone(),
        ));
        extend_missing_resource_declarations(&mut pass.resources, lower_bindings(&self.bindings));
        match &self.dispatch {
            RenderGraphComputeDispatchExtent::FromBuffer { buffer, .. }
                if !pass.resources.iter().any(|resource| {
                    resource.name == buffer.as_str()
                        && resource.access == RenderFeatureResourceAccess::Read
                }) =>
            {
                pass.resources.push(resource_descriptor(
                    buffer.clone(),
                    RenderFeatureResourceKind::Buffer,
                    RenderFeatureResourceAccess::Read,
                    RenderFeatureResourceWriteMode::Attachment,
                ));
            }
            RenderGraphComputeDispatchExtent::PerPixel { target, .. }
                if !pass
                    .resources
                    .iter()
                    .any(|resource| resource.name == target.as_str()) =>
            {
                pass.resources.push(resource_descriptor(
                    target.clone(),
                    RenderFeatureResourceKind::Texture,
                    RenderFeatureResourceAccess::Read,
                    RenderFeatureResourceWriteMode::Attachment,
                ));
            }
            _ => {}
        }
        pass.compute_pass = Some(self);
        pass
    }
}

fn extend_missing_resource_declarations(
    resources: &mut Vec<RenderFeatureResourceDescriptor>,
    lowered_resources: Vec<RenderFeatureResourceDescriptor>,
) {
    for mut resource in lowered_resources {
        let existing_resource = resources
            .iter()
            .find(|existing| existing.name == resource.name);
        if existing_resource.is_some_and(|existing| existing.access == resource.access) {
            continue;
        }
        if let Some(existing) = existing_resource {
            resource.kind = existing.kind;
            resource.external_binding = existing.external_binding;
        }
        resources.push(resource);
    }
}

fn lower_bindings(
    bindings: &[crate::render_graph::BindingSchemaEntry],
) -> Vec<RenderFeatureResourceDescriptor> {
    let mut resources = Vec::with_capacity(bindings.len());
    for binding in bindings {
        match binding.kind {
            ComputeBindingKind::UniformBuffer | ComputeBindingKind::StorageBufferRead => {
                resources.push(resource_descriptor(
                    binding.resource.clone(),
                    RenderFeatureResourceKind::Buffer,
                    RenderFeatureResourceAccess::Read,
                    RenderFeatureResourceWriteMode::Attachment,
                ));
            }
            ComputeBindingKind::StorageBufferReadWrite => {
                resources.push(resource_descriptor(
                    binding.resource.clone(),
                    RenderFeatureResourceKind::Buffer,
                    RenderFeatureResourceAccess::Read,
                    RenderFeatureResourceWriteMode::Attachment,
                ));
                resources.push(resource_descriptor(
                    binding.resource.clone(),
                    RenderFeatureResourceKind::Buffer,
                    RenderFeatureResourceAccess::Write,
                    RenderFeatureResourceWriteMode::Storage,
                ));
            }
            ComputeBindingKind::SampledTexture => resources.push(resource_descriptor(
                binding.resource.clone(),
                RenderFeatureResourceKind::Texture,
                RenderFeatureResourceAccess::Read,
                RenderFeatureResourceWriteMode::Attachment,
            )),
            ComputeBindingKind::StorageTextureWrite => resources.push(resource_descriptor(
                binding.resource.clone(),
                RenderFeatureResourceKind::Texture,
                RenderFeatureResourceAccess::Write,
                RenderFeatureResourceWriteMode::Storage,
            )),
        }
    }
    resources
}

fn resource_descriptor(
    name: String,
    kind: RenderFeatureResourceKind,
    access: RenderFeatureResourceAccess,
    write_mode: RenderFeatureResourceWriteMode,
) -> RenderFeatureResourceDescriptor {
    let external_binding = match kind {
        RenderFeatureResourceKind::Buffer => RenderGraphExternalResourceBinding::required_buffer(),
        RenderFeatureResourceKind::Texture => {
            RenderGraphExternalResourceBinding::required_texture()
        }
        RenderFeatureResourceKind::External => RenderGraphExternalResourceBinding::report_only(),
    };
    RenderFeatureResourceDescriptor {
        name,
        kind,
        access,
        input_version: None,
        minimum_size_bytes: None,
        attachment_ops: None,
        write_mode,
        external_binding,
    }
}
