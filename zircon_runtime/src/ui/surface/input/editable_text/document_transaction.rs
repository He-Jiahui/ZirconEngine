use crate::text::document::{PreparedTextDocumentStoreEdit, TextDocumentStoreEditCommit};

use super::property_transaction::{
    PreparedUiEditableTextPropertyTransaction, UiEditableTextPropertyTransactionError,
    UiEditableTextPropertyTransactionReceipt,
};

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
