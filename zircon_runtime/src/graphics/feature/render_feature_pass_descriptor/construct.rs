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

use super::super::compute_pass_descriptor::ComputePassDescriptor;

use super::render_feature_pass_descriptor::{
    RenderFeaturePassDescriptor, RenderFeatureResourceAccess, RenderFeatureResourceDescriptor,
    RenderFeatureResourceKind, RenderFeatureResourceVersion, RenderFeatureResourceWriteMode,
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
            compute_pass: None,
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
        self.compute_pass = None;
        self
    }

    pub fn with_compute_pass(self, compute_pass: ComputePassDescriptor) -> Self {
        compute_pass.lower_into(self)
    }

    pub fn with_compute_dispatch_plan(mut self, plan: &ComputeDispatchPlan) -> Self {
        self.compute_workload = Some(RenderGraphComputeWorkload::from_shader_dispatch(plan));
        self.compute_pass = None;
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

    pub fn read_texture_from(
        self,
        name: impl Into<String>,
        producer_pass_name: impl Into<String>,
    ) -> Self {
        self.with_resource_from_producer(
            name,
            RenderFeatureResourceKind::Texture,
            RenderFeatureResourceAccess::Read,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only(),
            producer_pass_name,
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

    pub fn read_buffer_from(
        self,
        name: impl Into<String>,
        producer_pass_name: impl Into<String>,
    ) -> Self {
        self.with_resource_from_producer(
            name,
            RenderFeatureResourceKind::Buffer,
            RenderFeatureResourceAccess::Read,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only(),
            producer_pass_name,
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

    pub fn write_buffer_with_minimum_size(
        self,
        name: impl Into<String>,
        minimum_size_bytes: u64,
    ) -> Self {
        self.with_resource_minimum_size(
            name,
            RenderFeatureResourceKind::Buffer,
            RenderFeatureResourceAccess::Write,
            RenderFeatureResourceWriteMode::Storage,
            minimum_size_bytes,
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

    pub fn read_external_from(
        self,
        name: impl Into<String>,
        producer_pass_name: impl Into<String>,
    ) -> Self {
        self.with_resource_from_producer(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Read,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only(),
            producer_pass_name,
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
        self.with_resource_with_input_version(
            name,
            kind,
            access,
            attachment_ops,
            write_mode,
            external_binding,
            None,
        )
    }

    fn with_resource_from_producer(
        self,
        name: impl Into<String>,
        kind: RenderFeatureResourceKind,
        access: RenderFeatureResourceAccess,
        attachment_ops: Option<RenderGraphAttachmentOps>,
        write_mode: RenderFeatureResourceWriteMode,
        external_binding: RenderGraphExternalResourceBinding,
        producer_pass_name: impl Into<String>,
    ) -> Self {
        let name = name.into();
        self.with_resource_with_input_version(
            name.clone(),
            kind,
            access,
            attachment_ops,
            write_mode,
            external_binding,
            Some(RenderFeatureResourceVersion::new(
                name,
                kind,
                producer_pass_name,
            )),
        )
    }

    fn with_resource_with_input_version(
        mut self,
        name: impl Into<String>,
        kind: RenderFeatureResourceKind,
        access: RenderFeatureResourceAccess,
        attachment_ops: Option<RenderGraphAttachmentOps>,
        write_mode: RenderFeatureResourceWriteMode,
        external_binding: RenderGraphExternalResourceBinding,
        input_version: Option<RenderFeatureResourceVersion>,
    ) -> Self {
        self.resources.push(RenderFeatureResourceDescriptor {
            name: name.into(),
            kind,
            access,
            input_version,
            minimum_size_bytes: None,
            attachment_ops,
            write_mode,
            external_binding,
        });
        self
    }

    fn with_resource_minimum_size(
        mut self,
        name: impl Into<String>,
        kind: RenderFeatureResourceKind,
        access: RenderFeatureResourceAccess,
        write_mode: RenderFeatureResourceWriteMode,
        minimum_size_bytes: u64,
    ) -> Self {
        self.resources.push(RenderFeatureResourceDescriptor {
            name: name.into(),
            kind,
            access,
            input_version: None,
            minimum_size_bytes: Some(minimum_size_bytes),
            attachment_ops: None,
            write_mode,
            external_binding: RenderGraphExternalResourceBinding::report_only(),
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
        input_version: None,
        minimum_size_bytes: None,
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
        ComputeDispatchBuilder, ComputeKernelRef, FullscreenPassBuilder, FullscreenShaderRef,
        RenderShaderEntryPointDescriptor, RenderShaderStage, ShaderAssetKind,
        ShaderDispatchBuildDiagnostic, ShaderDispatchExtent, ShaderResourceDescriptor,
    };

    #[test]
    fn feature_pass_descriptor_consumes_shader_compute_dispatch_plan_resources() {
        let shader = AssetReference::from_locator(
            ResourceLocator::parse("builtin://shaders/compute/clustered_lighting").unwrap(),
        );
        let dispatch = ComputeDispatchBuilder::new(ComputeKernelRef::new(shader, "cs_main"))
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

    #[test]
    fn feature_pass_descriptor_compute_and_fullscreen_contracts_report_named_resource_errors() {
        let compute_shader = AssetReference::from_locator(
            ResourceLocator::parse("res://shaders/simulation.zshader").unwrap(),
        );
        let compute = ComputeDispatchBuilder::new(ComputeKernelRef::new(compute_shader, "cs_main"))
            .bind_texture("particle_state")
            .dispatch_groups([1, 1, 1]);
        let compute_diagnostics = compute
            .build(
                ShaderAssetKind::Compute,
                &[RenderShaderEntryPointDescriptor {
                    name: "cs_main".to_string(),
                    stage: RenderShaderStage::Compute,
                }],
                &[ShaderResourceDescriptor {
                    name: "particle_state".to_string(),
                    kind: ShaderResourceKind::StorageBuffer,
                    access: Some(ShaderResourceAccess::ReadWrite),
                }],
            )
            .expect_err("compute resource type mismatch should be diagnosed");
        assert!(compute_diagnostics.contains(
            &ShaderDispatchBuildDiagnostic::ResourceKindMismatch {
                name: "particle_state".to_string(),
                expected: ShaderResourceKind::StorageBuffer,
                actual: ShaderResourceKind::Texture,
            }
        ));

        let fullscreen_shader = AssetReference::from_locator(
            ResourceLocator::parse("res://shaders/postprocess.zshader").unwrap(),
        );
        let fullscreen =
            FullscreenPassBuilder::new(FullscreenShaderRef::new(fullscreen_shader, "fs_main"))
                .bind_texture("scene_color");
        let fullscreen_plan = fullscreen
            .build(
                ShaderAssetKind::Fullscreen,
                &[RenderShaderEntryPointDescriptor {
                    name: "fs_main".to_string(),
                    stage: RenderShaderStage::Fragment,
                }],
                &[ShaderResourceDescriptor {
                    name: "scene_color".to_string(),
                    kind: ShaderResourceKind::Texture,
                    access: Some(ShaderResourceAccess::Read),
                }],
            )
            .expect("fullscreen resources should match the authored contract");
        let pass = RenderFeaturePassDescriptor::new(
            RenderPassStage::PostProcess,
            "authoring-fullscreen",
            QueueLane::Graphics,
        )
        .with_fullscreen_pass_plan(&fullscreen_plan);

        assert_eq!(pass.resources.len(), 1);
        assert_eq!(pass.resources[0].name, "scene_color");
        assert_eq!(pass.resources[0].kind, RenderFeatureResourceKind::Texture);
        assert_eq!(pass.resources[0].access, RenderFeatureResourceAccess::Read);
    }
}
