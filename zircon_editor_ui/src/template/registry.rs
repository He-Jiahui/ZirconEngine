use std::collections::BTreeMap;

use zircon_ui::{UiTemplateDocument, UiTemplateInstance};

use crate::EditorTemplateError;

#[derive(Default)]
pub struct EditorTemplateRegistry {
    documents: BTreeMap<String, UiTemplateDocument>,
}

impl EditorTemplateRegistry {
    pub fn register_document(
        &mut self,
        document_id: impl Into<String>,
        document: UiTemplateDocument,
    ) -> Result<(), EditorTemplateError> {
        let document_id = document_id.into();
        if self.documents.contains_key(&document_id) {
            return Err(EditorTemplateError::DuplicateDocument { document_id });
        }
        self.documents.insert(document_id, document);
        Ok(())
    }

    pub fn document(&self, document_id: &str) -> Option<&UiTemplateDocument> {
        self.documents.get(document_id)
    }

    pub fn instantiate(
        &self,
        document_id: &str,
    ) -> Result<UiTemplateInstance, EditorTemplateError> {
        let document = self.documents.get(document_id).ok_or_else(|| {
            EditorTemplateError::MissingDocument {
                document_id: document_id.to_string(),
            }
        })?;
        UiTemplateInstance::from_document(document).map_err(EditorTemplateError::from)
    }
}
