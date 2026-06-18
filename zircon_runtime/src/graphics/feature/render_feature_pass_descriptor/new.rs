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
}
