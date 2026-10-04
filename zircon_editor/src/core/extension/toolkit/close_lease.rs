use crate::core::editor_message::DocumentId;

use super::{
    DocumentToolkitDescriptor, DocumentToolkitRegistry, ToolkitInstanceId, ToolkitRegistryError,
};

/// 文档关闭的待提交许可；未提交时 Drop 撤销 closing 标记。UI 关闭视图路径在布局成功后提交。
pub struct DocumentCloseLease<'a, Host> {
    registry: &'a DocumentToolkitRegistry<Host>,
    document: DocumentId,
    instance: ToolkitInstanceId,
    committed: bool,
}

impl<'a, Host> DocumentCloseLease<'a, Host> {
    pub(super) fn new(
        registry: &'a DocumentToolkitRegistry<Host>,
        document: DocumentId,
        instance: ToolkitInstanceId,
    ) -> Self {
        Self {
            registry,
            document,
            instance,
            committed: false,
        }
    }

    pub const fn document_id(&self) -> DocumentId {
        self.document
    }

    pub fn instance_id(&self) -> &ToolkitInstanceId {
        &self.instance
    }

    /// 移除本租约对应的注册并发布新目录快照；调用方负责先完成所需的宿主关闭步骤。
    pub fn commit(mut self) -> Result<DocumentToolkitDescriptor, ToolkitRegistryError> {
        let descriptor = self.registry.commit_close(self.document, &self.instance)?;
        self.committed = true;
        Ok(descriptor)
    }
}

impl<Host> Drop for DocumentCloseLease<'_, Host> {
    fn drop(&mut self) {
        if !self.committed {
            self.registry.rollback_close(self.document, &self.instance);
        }
    }
}
