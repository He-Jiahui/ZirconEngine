use crate::core::framework::render::{
    ComputeDispatchPlan, FullscreenPassPlan, ShaderNamedResourceBinding, ShaderResourceAccess,
    ShaderResourceKind,
};
use crate::graphics::scene::RenderPassExecutorId;
use crate::render_graph::{
    QueueLane, RenderGraphAttachmentOps, RenderGraphComputeWorkload,
    RenderGraphExternalResourceBinding,
};

use crate::graphics::pipeline::RenderPassStage;

use super::render_feature_pass_descriptor::{
    RenderFeaturePassDescriptor, RenderFeatureResourceAccess, RenderFeatureResourceDescriptor,
    RenderFeatureResourceKind, RenderFeatureResourceWriteMode,
};

impl RenderFeaturePassDescriptor {
    pub fn new(stage: RenderPassStage, pass_name: impl Into<String>, queue: QueueLane) -> Self {
        let pass_name = pass_name.into();
        Self {
            stage,
            executor_id: RenderPassExecutorId::new(pass_name.clone()),
            pass_name,
            queue,
            flags: Default::default(),
            compute_workload: None,
            resources: Vec::new(),
        }
    }

    pub fn with_executor_id(mut self, executor_id: impl Into<RenderPassExecutorId>) -> Self {
        self.executor_id = executor_id.into();
        self
    }

    pub fn with_side_effects(mut self) -> Self {
        self.flags.has_side_effects = true;
        self
    }

    pub fn with_compute_workload(mut self, workload: RenderGraphComputeWorkload) -> Self {
        self.compute_workload = Some(workload);
        self
    }

    pub fn with_compute_dispatch_plan(mut self, plan: &ComputeDispatchPlan) -> Self {
        self.compute_workload = Some(RenderGraphComputeWorkload::from_shader_dispatch(plan));
        self.push_shader_resource_bindings(&plan.resources);
        self
    }

    pub fn with_fullscreen_pass_plan(mut self, plan: &FullscreenPassPlan) -> Self {
        self.push_shader_resource_bindings(&plan.resources);
        self
    }

    pub fn read_texture(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::Texture,
            RenderFeatureResourceAccess::Read,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only(),
        )
    }

    pub fn write_texture(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::Texture,
            RenderFeatureResourceAccess::Write,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only(),
        )
    }

    pub fn write_storage_texture(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::Texture,
            RenderFeatureResourceAccess::Write,
            None,
            RenderFeatureResourceWriteMode::Storage,
            RenderGraphExternalResourceBinding::report_only(),
        )
    }

    pub fn write_texture_with_ops(
        self,
        name: impl Into<String>,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::Texture,
            RenderFeatureResourceAccess::Write,
            Some(attachment_ops),
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only(),
        )
    }

    pub fn read_buffer(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::Buffer,
            RenderFeatureResourceAccess::Read,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only(),
        )
    }

    pub fn write_buffer(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::Buffer,
            RenderFeatureResourceAccess::Write,
            None,
            RenderFeatureResourceWriteMode::Storage,
            RenderGraphExternalResourceBinding::report_only(),
        )
    }

    pub fn read_external(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Read,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only(),
        )
    }

    pub fn read_external_texture(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Read,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only_texture(),
        )
    }

    pub fn read_external_buffer(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Read,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only_buffer(),
        )
    }

    pub fn write_external(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Write,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only(),
        )
    }

    pub fn write_external_texture(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Write,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only_texture(),
        )
    }

    pub fn write_external_buffer(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Write,
            None,
            RenderFeatureResourceWriteMode::Storage,
            RenderGraphExternalResourceBinding::report_only_buffer(),
        )
    }

    pub fn write_storage_external(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Write,
            None,
            RenderFeatureResourceWriteMode::Storage,
            RenderGraphExternalResourceBinding::report_only(),
        )
    }

    pub fn write_storage_external_texture(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Write,
            None,
            RenderFeatureResourceWriteMode::Storage,
            RenderGraphExternalResourceBinding::report_only_texture(),
        )
    }

    pub fn write_storage_external_buffer(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Write,
            None,
            RenderFeatureResourceWriteMode::Storage,
            RenderGraphExternalResourceBinding::report_only_buffer(),
        )
    }

    pub fn read_required_external_buffer(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Read,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::required_buffer(),
        )
    }

    pub fn read_required_external_texture(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Read,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::required_texture(),
        )
    }

    pub fn write_required_external_buffer(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Write,
            None,
            RenderFeatureResourceWriteMode::Storage,
            RenderGraphExternalResourceBinding::required_buffer(),
        )
    }

    pub fn write_required_external_texture(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Write,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::required_texture(),
        )
    }

    pub fn write_required_external_texture_with_ops(
        self,
        name: impl Into<String>,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Write,
            Some(attachment_ops),
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::required_texture(),
        )
    }

    pub fn write_required_storage_external_texture(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Write,
            None,
            RenderFeatureResourceWriteMode::Storage,
            RenderGraphExternalResourceBinding::required_texture(),
        )
    }

    pub fn write_external_with_ops(
        self,
        name: impl Into<String>,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Write,
            Some(attachment_ops),
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only(),
        )
    }

    pub fn write_external_texture_with_ops(
        self,
        name: impl Into<String>,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Write,
            Some(attachment_ops),
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only_texture(),
        )
    }

    fn with_resource(
        mut self,
        name: impl Into<String>,
        kind: RenderFeatureResourceKind,
        access: RenderFeatureResourceAccess,
        attachment_ops: Option<RenderGraphAttachmentOps>,
        write_mode: RenderFeatureResourceWriteMode,
        external_binding: RenderGraphExternalResourceBinding,
    ) -> Self {
        self.resources.push(RenderFeatureResourceDescriptor {
            name: name.into(),
            kind,
            access,
            attachment_ops,
            write_mode,
            external_binding,
        });
        self
    }

    fn push_shader_resource_bindings(&mut self, bindings: &[ShaderNamedResourceBinding]) {
        self.resources.extend(
            bindings
                .iter()
                .filter_map(render_feature_resource_for_shader_binding),
        );
    }
}

fn render_feature_resource_for_shader_binding(
    binding: &ShaderNamedResourceBinding,
) -> Option<RenderFeatureResourceDescriptor> {
    let kind = match binding.kind {
        ShaderResourceKind::UniformBuffer | ShaderResourceKind::StorageBuffer => {
            RenderFeatureResourceKind::Buffer
        }
        ShaderResourceKind::Texture | ShaderResourceKind::StorageTexture => {
            RenderFeatureResourceKind::Texture
        }
        ShaderResourceKind::Sampler => return None,
    };
    let access = match binding.access {
        ShaderResourceAccess::Read => RenderFeatureResourceAccess::Read,
        ShaderResourceAccess::ReadWrite | ShaderResourceAccess::Write => {
            RenderFeatureResourceAccess::Write
        }
    };
    let write_mode = if matches!(access, RenderFeatureResourceAccess::Write)
        || matches!(
            binding.kind,
            ShaderResourceKind::StorageBuffer | ShaderResourceKind::StorageTexture
        ) {
        RenderFeatureResourceWriteMode::Storage
    } else {
        RenderFeatureResourceWriteMode::Attachment
    };

    Some(RenderFeatureResourceDescriptor {
        name: binding.name.clone(),
        kind,
        access,
        attachment_ops: None,
        write_mode,
        external_binding: RenderGraphExternalResourceBinding::report_only(),
    })
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::resource::{AssetReference, ResourceLocator};

    use super::*;
    use crate::core::framework::render::{
        ComputeDispatchBuilder, ComputeKernelRef, RenderShaderEntryPointDescriptor,
        RenderShaderStage, ShaderAssetKind, ShaderDispatchExtent, ShaderResourceDescriptor,
    };

    #[test]
    fn feature_pass_descriptor_consumes_shader_compute_dispatch_plan_resources() {
        let shader = AssetReference::from_locator(
            ResourceLocator::parse("builtin://shaders/compute/clustered_lighting").unwrap(),
        );
        let mut dispatch = ComputeDispatchBuilder::new(ComputeKernelRef::new(shader, "cs_main"));
        dispatch
            .with_pipeline_label("zircon-cluster-pipeline")
            .with_workgroup_size([8, 8, 1])
            .bind_storage_write("light-list")
            .bind_sampler("linear_sampler")
            .dispatch_extent(ShaderDispatchExtent::ClusterGrid);
        let dispatch = dispatch
            .build(
                ShaderAssetKind::Compute,
                &[RenderShaderEntryPointDescriptor {
                    name: "cs_main".to_string(),
                    stage: RenderShaderStage::Compute,
                }],
                &[
                    ShaderResourceDescriptor {
                        name: "light-list".to_string(),
                        kind: ShaderResourceKind::StorageBuffer,
                        access: Some(ShaderResourceAccess::Write),
                    },
                    ShaderResourceDescriptor {
                        name: "linear_sampler".to_string(),
                        kind: ShaderResourceKind::Sampler,
                        access: Some(ShaderResourceAccess::Read),
                    },
                ],
            )
            .unwrap();

        let pass = RenderFeaturePassDescriptor::new(
            RenderPassStage::Lighting,
            "light-grid-build",
            QueueLane::AsyncCompute,
        )
        .with_compute_dispatch_plan(&dispatch);

        assert_eq!(
            pass.compute_workload.as_ref().unwrap().pipeline_label,
            "zircon-cluster-pipeline"
        );
        assert_eq!(pass.resources.len(), 1);
        assert_eq!(pass.resources[0].name, "light-list");
        assert_eq!(pass.resources[0].kind, RenderFeatureResourceKind::Buffer);
        assert_eq!(pass.resources[0].access, RenderFeatureResourceAccess::Write);
        assert_eq!(
            pass.resources[0].write_mode,
            RenderFeatureResourceWriteMode::Storage
        );
    }
}
