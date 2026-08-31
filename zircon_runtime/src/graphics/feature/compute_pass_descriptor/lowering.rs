use crate::graphics::feature::{
    RenderFeaturePassDescriptor, RenderFeatureResourceAccess, RenderFeatureResourceDescriptor,
    RenderFeatureResourceKind, RenderFeatureResourceWriteMode,
};
use crate::graphics::scene::RenderPassExecutorId;
use crate::render_graph::{
    BindingSchemaEntry, ComputeBindingKind, RenderGraphComputeDispatchExtent,
    RenderGraphComputeWorkload, RenderGraphExternalResourceBinding, RenderGraphResourceAccessKind,
    RenderGraphResourceAccessMetadata, RenderGraphResourceUsageFlags,
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
        pass.compute_workload = Some(
            RenderGraphComputeWorkload::new(
                self.shader.pipeline_label(),
                self.workgroup_size,
                self.dispatch.clone(),
            )
            .with_pipeline_fallback_policy(self.pipeline_fallback_policy.clone()),
        );
        extend_missing_resource_declarations(
            &mut pass.resources,
            lower_bindings(&self.bindings, &self),
        );
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
                    self.resource_schema(buffer),
                    None,
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
                    self.resource_schema(target),
                    None,
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
        let matching_resource = resources.iter_mut().find(|existing| {
            existing.name == resource.name
                && existing.access == resource.access
                && existing.access_metadata == resource.access_metadata
        });
        if let Some(existing) = matching_resource {
            if existing.schema.is_none() {
                existing.schema = resource.schema;
            }
            continue;
        }

        let legacy_resource = resources.iter_mut().find(|existing| {
            existing.name == resource.name
                && existing.access == resource.access
                && existing.access_metadata.is_none()
        });
        if let Some(existing) = legacy_resource {
            if existing.schema.is_none() {
                existing.schema = resource.schema;
            }
            if existing.kind != RenderFeatureResourceKind::External {
                existing.access_metadata = resource.access_metadata;
            }
            continue;
        }

        if let Some(existing) = resources
            .iter()
            .find(|existing| existing.name == resource.name)
        {
            resource.kind = existing.kind;
            resource.external_binding = existing.external_binding;
            if resource.schema.is_none() {
                resource.schema = existing.schema;
            }
            if resource.kind == RenderFeatureResourceKind::External {
                resource.access_metadata = None;
            }
        }
        resources.push(resource);
    }
}

fn lower_bindings(
    bindings: &[BindingSchemaEntry],
    descriptor: &ComputePassDescriptor,
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
                    descriptor.resource_schema(&binding.resource),
                    binding_access_metadata(binding, RenderFeatureResourceAccess::Read),
                ));
            }
            ComputeBindingKind::StorageBufferReadWrite => {
                resources.push(resource_descriptor(
                    binding.resource.clone(),
                    RenderFeatureResourceKind::Buffer,
                    RenderFeatureResourceAccess::Read,
                    RenderFeatureResourceWriteMode::Attachment,
                    descriptor.resource_schema(&binding.resource),
                    binding_access_metadata(binding, RenderFeatureResourceAccess::Read),
                ));
                resources.push(resource_descriptor(
                    binding.resource.clone(),
                    RenderFeatureResourceKind::Buffer,
                    RenderFeatureResourceAccess::Write,
                    RenderFeatureResourceWriteMode::Storage,
                    descriptor.resource_schema(&binding.resource),
                    binding_access_metadata(binding, RenderFeatureResourceAccess::Write),
                ));
            }
            ComputeBindingKind::SampledTexture => resources.push(resource_descriptor(
                binding.resource.clone(),
                RenderFeatureResourceKind::Texture,
                RenderFeatureResourceAccess::Read,
                RenderFeatureResourceWriteMode::Attachment,
                descriptor.resource_schema(&binding.resource),
                binding_access_metadata(binding, RenderFeatureResourceAccess::Read),
            )),
            ComputeBindingKind::StorageTextureWrite => resources.push(resource_descriptor(
                binding.resource.clone(),
                RenderFeatureResourceKind::Texture,
                RenderFeatureResourceAccess::Write,
                RenderFeatureResourceWriteMode::Storage,
                descriptor.resource_schema(&binding.resource),
                binding_access_metadata(binding, RenderFeatureResourceAccess::Write),
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
    schema: Option<crate::graphics::feature::RenderResourceSchema>,
    access_metadata: Option<RenderGraphResourceAccessMetadata>,
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
        access_metadata,
        external_binding,
        texture_view_alias: None,
        schema,
        usage: RenderGraphResourceUsageFlags::default(),
    }
}

fn binding_access_metadata(
    binding: &BindingSchemaEntry,
    access: RenderFeatureResourceAccess,
) -> Option<RenderGraphResourceAccessMetadata> {
    let access = match access {
        RenderFeatureResourceAccess::Read => RenderGraphResourceAccessKind::Read,
        RenderFeatureResourceAccess::Write => RenderGraphResourceAccessKind::Write,
    };
    binding.compute_access_metadata(access)
}
