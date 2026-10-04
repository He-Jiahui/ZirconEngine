use crate::text::document::{PreparedTextDocumentStoreEdit, TextDocumentStoreEditCommit};

use super::property_transaction::{
    PreparedUiEditableTextPropertyTransaction, UiEditableTextPropertyTransactionError,
    UiEditableTextPropertyTransactionReceipt,
};

/// 同时准备 surface 属性投影与文档存储编辑，避免任一准备失败留下半次提交。
/// 提交先执行仍可能拒绝的属性事务，成功后消费已准备且不会再失败的文档编辑。
#[must_use = "a prepared editable document transaction must be committed or explicitly discarded"]
pub(in crate::ui) struct PreparedUiEditableTextDocumentTransaction<'surface, 'documents> {
    properties: PreparedUiEditableTextPropertyTransaction<'surface>,
    document: PreparedTextDocumentStoreEdit<'documents>,
}

#[derive(Debug)]
pub(in crate::ui) struct UiEditableTextDocumentTransactionReceipt {
    pub(in crate::ui) properties: UiEditableTextPropertyTransactionReceipt,
    pub(in crate::ui) document: TextDocumentStoreEditCommit,
}

impl<'surface, 'documents> PreparedUiEditableTextDocumentTransaction<'surface, 'documents> {
    pub(in crate::ui) const fn new(
        properties: PreparedUiEditableTextPropertyTransaction<'surface>,
        document: PreparedTextDocumentStoreEdit<'documents>,
    ) -> Self {
        Self {
            properties,
            document,
        }
    }

    pub(in crate::ui) fn commit(
        self,
    ) -> Result<UiEditableTextDocumentTransactionReceipt, UiEditableTextPropertyTransactionError>
    {
        let properties = self.properties.commit()?;
        let document = self.document.commit();
        Ok(UiEditableTextDocumentTransactionReceipt {
            properties,
            document,
        })
    }
}
