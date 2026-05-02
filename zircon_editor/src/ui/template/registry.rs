use std::collections::BTreeMap;

use zircon_runtime::ui::template::{UiCompiledDocument, UiDocumentCompiler, UiTemplateInstance};
use zircon_runtime_interface::ui::template::UiAssetDocument;

use crate::ui::template::EditorTemplateError;

#[derive(Default)]
pub struct EditorTemplateRegistry {
    documents: BTreeMap<String, UiCompiledDocument>,
}

impl EditorTemplateRegistry {
    pub fn register_asset_document(
        &mut self,
        document_id: impl Into<String>,
        document: UiAssetDocument,
    ) -> Result<(), EditorTemplateError> {
        let compiled = UiDocumentCompiler::default().compile(&document)?;
        self.register_compiled_document(document_id, compiled)
    }

    pub fn register_compiled_document(
        &mut self,
        document_id: impl Into<String>,
        document: UiCompiledDocument,
    ) -> Result<(), EditorTemplateError> {
        let document_id = document_id.into();
        if self.documents.contains_key(&document_id) {
            return Err(EditorTemplateError::DuplicateDocument { document_id });
        }
        self.documents.insert(document_id, document);
        Ok(())
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
        Ok(document.clone().into_template_instance())
    }
}
