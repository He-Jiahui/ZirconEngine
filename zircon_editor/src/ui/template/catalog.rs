use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    template::{UiAssetError, UiTemplateError},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorComponentDescriptor {
    pub component_id: String,
    pub document_id: String,
    pub binding_namespace: String,
}

impl EditorComponentDescriptor {
    pub fn new(
        component_id: impl Into<String>,
        document_id: impl Into<String>,
        binding_namespace: impl Into<String>,
    ) -> Self {
        Self {
            component_id: component_id.into(),
            document_id: document_id.into(),
            binding_namespace: binding_namespace.into(),
        }
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum EditorTemplateError {
    #[error("editor component {component_id} already registered")]
    DuplicateComponent { component_id: String },
    #[error("editor template document {document_id} already registered")]
    DuplicateDocument { document_id: String },
    #[error("editor template binding {binding_id} already registered")]
    DuplicateBinding { binding_id: String },
    #[error("editor template document {document_id} is not registered")]
    MissingDocument { document_id: String },
    #[error("editor template binding {binding_id} is not registered")]
    MissingBinding { binding_id: String },
    #[error(
        "editor template binding {binding_id} expected event {expected:?} but found {actual:?}"
    )]
    BindingEventMismatch {
        binding_id: String,
        expected: UiEventKind,
        actual: UiEventKind,
    },
    #[error(transparent)]
    Template(#[from] UiTemplateError),
    #[error(transparent)]
    Asset(#[from] UiAssetError),
}

#[derive(Default)]
pub struct EditorComponentCatalog {
    descriptors: BTreeMap<String, EditorComponentDescriptor>,
}

impl EditorComponentCatalog {
    pub fn register(
        &mut self,
        descriptor: EditorComponentDescriptor,
    ) -> Result<(), EditorTemplateError> {
        if self.descriptors.contains_key(&descriptor.component_id) {
            return Err(EditorTemplateError::DuplicateComponent {
                component_id: descriptor.component_id,
            });
        }
        self.descriptors
            .insert(descriptor.component_id.clone(), descriptor);
        Ok(())
    }

    pub fn descriptor(&self, component_id: &str) -> Option<&EditorComponentDescriptor> {
        self.descriptors.get(component_id)
    }

    #[allow(dead_code)]
    pub fn descriptors(&self) -> Vec<&EditorComponentDescriptor> {
        self.descriptors.values().collect()
    }
}
