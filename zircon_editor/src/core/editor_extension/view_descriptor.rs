use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::component::{UiComponentProjectionPatch, UiValue};

use crate::core::commands::DocumentKind;
use crate::core::editor_operation::{EditorOperationPath, EditorOperationPathError};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewDescriptor {
    id: String,
    display_name: String,
    category: String,
    document_kind: Option<DocumentKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ui_template_id: Option<String>,
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
            ui_template_id: None,
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

    pub fn with_ui_template_id(mut self, template_id: impl Into<String>) -> Self {
        self.ui_template_id = Some(template_id.into());
        self
    }

    pub fn ui_template_id(&self) -> Option<&str> {
        self.ui_template_id.as_deref()
    }

    pub(crate) fn bind_ui_template_id(&mut self, template_id: impl Into<String>) {
        self.ui_template_id = Some(template_id.into());
    }

    pub fn open_operation_path(&self) -> Result<EditorOperationPath, EditorOperationPathError> {
        EditorOperationPath::parse(format!("view.{}.open", self.id))
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EditorUiTemplatePaneDataSnapshot {
    values: BTreeMap<String, UiValue>,
    component_patches: Vec<UiComponentProjectionPatch>,
}

impl EditorUiTemplatePaneDataSnapshot {
    pub fn new(values: BTreeMap<String, UiValue>) -> Self {
        Self {
            values,
            component_patches: Vec::new(),
        }
    }

    pub fn values(&self) -> &BTreeMap<String, UiValue> {
        &self.values
    }

    pub fn with_component_patch(mut self, patch: UiComponentProjectionPatch) -> Self {
        self.component_patches.push(patch);
        self
    }

    pub fn component_patches(&self) -> &[UiComponentProjectionPatch] {
        &self.component_patches
    }
}

pub trait EditorUiTemplatePaneDataSource: Send + Sync {
    fn snapshot(&self) -> EditorUiTemplatePaneDataSnapshot;
}
