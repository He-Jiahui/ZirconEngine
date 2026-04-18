use std::collections::BTreeMap;

use zircon_ui::{
    UiAssetDocument, UiCompiledDocument, UiDocumentCompiler, UiTemplateDocument, UiTemplateInstance,
};

use crate::EditorTemplateError;

enum EditorTemplateSource {
    Template(UiTemplateDocument),
    Compiled(UiCompiledDocument),
}

#[derive(Default)]
pub struct EditorTemplateRegistry {
    documents: BTreeMap<String, EditorTemplateSource>,
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
        self.documents
            .insert(document_id, EditorTemplateSource::Template(document));
        Ok(())
    }

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
        self.documents
            .insert(document_id, EditorTemplateSource::Compiled(document));
        Ok(())
    }

    pub fn document(&self, document_id: &str) -> Option<&UiTemplateDocument> {
        match self.documents.get(document_id) {
            Some(EditorTemplateSource::Template(document)) => Some(document),
            Some(EditorTemplateSource::Compiled(_)) | None => None,
        }
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
        match document {
            EditorTemplateSource::Template(document) => {
                UiTemplateInstance::from_document(document).map_err(EditorTemplateError::from)
            }
            EditorTemplateSource::Compiled(document) => {
                Ok(document.clone().into_template_instance())
            }
        }
    }
}
