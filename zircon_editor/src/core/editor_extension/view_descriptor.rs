use serde::{Deserialize, Serialize};

use crate::core::commands::DocumentKind;
use crate::core::editor_operation::{EditorOperationPath, EditorOperationPathError};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewDescriptor {
    id: String,
    display_name: String,
    category: String,
    document_kind: Option<DocumentKind>,
}

impl ViewDescriptor {
    pub fn new(
        id: impl Into<String>,
        display_name: impl Into<String>,
        category: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
            category: category.into(),
            document_kind: None,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn category(&self) -> &str {
        &self.category
    }

    pub fn with_document_kind(mut self, document_kind: DocumentKind) -> Self {
        self.document_kind = Some(document_kind);
        self
    }

    pub fn document_kind(&self) -> Option<&DocumentKind> {
        self.document_kind.as_ref()
    }

    pub fn open_operation_path(&self) -> Result<EditorOperationPath, EditorOperationPathError> {
        EditorOperationPath::parse(format!("view.{}.open", self.id))
    }
}
