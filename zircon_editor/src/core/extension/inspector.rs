use std::collections::HashSet;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::core::editor_operation::EditorOperationPath;

mod field_editor;

pub use field_editor::{
    FieldEditorContainer, FieldEditorDefinition, FieldEditorFactory, FieldEditorInit,
    FieldEditorInstance, FieldEditorKind,
};

/// Reflection type identity used to select a whole-inspector customization.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct InspectTargetType(String);

impl InspectTargetType {
    pub fn new(type_name: impl Into<String>) -> Result<Self, InspectorRegistrationError> {
        let type_name = type_name.into();
        if !is_stable_type_name(&type_name) {
            return Err(InspectorRegistrationError::InvalidTypeName(type_name));
        }
        Ok(Self(type_name))
    }

    pub fn type_name(&self) -> &str {
        &self.0
    }
}

/// The selected object identity supplied to a class-level customization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InspectTarget {
    target_type: InspectTargetType,
    object_id: String,
}

impl InspectTarget {
    pub fn new(
        target_type: InspectTargetType,
        object_id: impl Into<String>,
    ) -> Result<Self, InspectorRegistrationError> {
        let object_id = object_id.into();
        if object_id.trim().is_empty() || object_id.trim() != object_id {
            return Err(InspectorRegistrationError::InvalidTargetId(object_id));
        }
        Ok(Self {
            target_type,
            object_id,
        })
    }

    pub fn target_type(&self) -> &InspectTargetType {
        &self.target_type
    }

    pub fn object_id(&self) -> &str {
        &self.object_id
    }
}

/// One reflected field before a type-level editor resolves its presentation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InspectorField {
    id: String,
    label: String,
    type_name: String,
    value: String,
    editable: bool,
}

impl InspectorField {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        type_name: impl Into<String>,
        value: impl Into<String>,
        editable: bool,
    ) -> Result<Self, InspectorRegistrationError> {
        let id = id.into();
        let label = label.into();
        let type_name = type_name.into();
        if !is_stable_type_name(&type_name) {
            return Err(InspectorRegistrationError::InvalidTypeName(type_name));
        }
        if id.trim().is_empty() || id.trim() != id {
            return Err(InspectorRegistrationError::InvalidFieldId(id));
        }
        if label.trim().is_empty() {
            return Err(InspectorRegistrationError::InvalidFieldLabel(label));
        }
        Ok(Self {
            id,
            label,
            type_name,
            value: value.into(),
            editable,
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub const fn editable(&self) -> bool {
        self.editable
    }
}

/// Declarative retained-UI surface supplied by a class-level customization.
///
/// The surface only describes the plugin-owned UI contract. The customization itself remains the
/// authoritative owner of target matching and layout construction.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InspectorCustomizationSurface {
    ui_document: String,
    controller: String,
    template_id: Option<String>,
    data_root: Option<String>,
    bindings: Vec<String>,
}

impl InspectorCustomizationSurface {
    pub fn new(ui_document: impl Into<String>, controller: impl Into<String>) -> Self {
        Self {
            ui_document: ui_document.into(),
            controller: controller.into(),
            template_id: None,
            data_root: None,
            bindings: Vec::new(),
        }
    }

    pub fn with_template_id(mut self, template_id: impl Into<String>) -> Self {
        self.template_id = Some(template_id.into());
        self
    }

    pub fn with_data_root(mut self, data_root: impl Into<String>) -> Self {
        self.data_root = Some(data_root.into());
        self
    }

    pub fn with_binding(mut self, binding: impl Into<String>) -> Self {
        self.bindings.push(binding.into());
        self.bindings.sort();
        self.bindings.dedup();
        self
    }

    pub fn ui_document(&self) -> &str {
        &self.ui_document
    }

    pub fn controller(&self) -> &str {
        &self.controller
    }

    pub fn template_id(&self) -> Option<&str> {
        self.template_id.as_deref()
    }

    pub fn data_root(&self) -> Option<&str> {
        self.data_root.as_deref()
    }

    pub fn bindings(&self) -> &[String] {
        &self.bindings
    }

    /// Validates a retained inspector surface before it enters any contribution registry.
    pub(crate) fn validate(&self) -> Result<(), InspectorRegistrationError> {
        if !is_valid_zui_document(&self.ui_document) {
            return Err(InspectorRegistrationError::InvalidCustomizationUiDocument(
                self.ui_document.clone(),
            ));
        }
        if !is_stable_type_name(&self.controller) {
            return Err(InspectorRegistrationError::InvalidCustomizationController(
                self.controller.clone(),
            ));
        }
        if self
            .template_id
            .as_deref()
            .is_some_and(|template_id| !is_trimmed_non_empty(template_id))
        {
            return Err(InspectorRegistrationError::InvalidCustomizationTemplateId(
                self.template_id.clone().unwrap_or_default(),
            ));
        }
        if self
            .data_root
            .as_deref()
            .is_some_and(|data_root| !is_trimmed_non_empty(data_root))
        {
            return Err(InspectorRegistrationError::InvalidCustomizationDataRoot(
                self.data_root.clone().unwrap_or_default(),
            ));
        }
        for binding in &self.bindings {
            if EditorOperationPath::parse(binding.clone()).is_err() {
                return Err(InspectorRegistrationError::InvalidCustomizationBinding(
                    binding.clone(),
                ));
            }
        }
        Ok(())
    }
}

/// Class-level customization with first-match responsibility-chain semantics.
pub trait InspectorCustomization: Send + Sync {
    fn id(&self) -> &str;

    /// Rejects invalid declarative registrations before they reach an inspector chain or store.
    fn validate(&self) -> Result<(), InspectorRegistrationError> {
        if !is_stable_type_name(self.id()) {
            return Err(InspectorRegistrationError::InvalidCustomizationId(
                self.id().to_string(),
            ));
        }
        Ok(())
    }

    fn can_handle(&self, target: &InspectTargetType) -> bool;

    fn build(&self, target: &InspectTarget, layout: &mut InspectorLayoutBuilder);

    /// A customization may expose a retained-UI surface after it wins the responsibility chain.
    fn surface(&self) -> Option<&InspectorCustomizationSurface> {
        None
    }
}

/// Canonical plugin-facing class customization declaration.
///
/// Its target type is explicit and it is consumed through [`InspectorCustomization`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InspectorCustomizationDescriptor {
    id: String,
    target_type: String,
    surface: InspectorCustomizationSurface,
}

impl InspectorCustomizationDescriptor {
    pub fn new(
        target_type: impl Into<String>,
        ui_document: impl Into<String>,
        controller: impl Into<String>,
    ) -> Self {
        let target_type = target_type.into();
        Self {
            id: target_type.clone(),
            target_type,
            surface: InspectorCustomizationSurface::new(ui_document, controller),
        }
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    pub fn with_template_id(mut self, template_id: impl Into<String>) -> Self {
        self.surface = self.surface.with_template_id(template_id);
        self
    }

    pub fn with_data_root(mut self, data_root: impl Into<String>) -> Self {
        self.surface = self.surface.with_data_root(data_root);
        self
    }

    pub fn with_binding(mut self, binding: impl Into<String>) -> Self {
        self.surface = self.surface.with_binding(binding);
        self
    }

    pub fn target_type(&self) -> &str {
        &self.target_type
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn surface(&self) -> &InspectorCustomizationSurface {
        &self.surface
    }

    pub fn validate(&self) -> Result<(), InspectorRegistrationError> {
        if !is_stable_type_name(&self.id) {
            return Err(InspectorRegistrationError::InvalidCustomizationId(
                self.id.clone(),
            ));
        }
        InspectTargetType::new(self.target_type.clone())?;
        self.surface.validate()?;
        Ok(())
    }
}

impl InspectorCustomization for InspectorCustomizationDescriptor {
    fn id(&self) -> &str {
        &self.id
    }

    fn validate(&self) -> Result<(), InspectorRegistrationError> {
        InspectorCustomizationDescriptor::validate(self)
    }

    fn can_handle(&self, target: &InspectTargetType) -> bool {
        target.type_name() == self.target_type
    }

    fn build(&self, _target: &InspectTarget, layout: &mut InspectorLayoutBuilder) {
        layout.add_custom_row(self.id(), self.surface.controller());
    }

    fn surface(&self) -> Option<&InspectorCustomizationSurface> {
        Some(&self.surface)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InspectorLayoutRow {
    Field {
        field: InspectorField,
        editor: FieldEditorInstance,
    },
    Custom {
        id: String,
        label: String,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InspectorLayoutBuilder {
    rows: Vec<InspectorLayoutRow>,
}

impl InspectorLayoutBuilder {
    pub fn add_auto_field(&mut self, field: InspectorField, editors: &FieldEditorContainer) {
        let editor = editors.resolve(field.clone());
        self.rows.push(InspectorLayoutRow::Field { field, editor });
    }

    pub fn add_custom_row(&mut self, id: impl Into<String>, label: impl Into<String>) {
        self.rows.push(InspectorLayoutRow::Custom {
            id: id.into(),
            label: label.into(),
        });
    }

    pub fn rows(&self) -> &[InspectorLayoutRow] {
        &self.rows
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InspectorLayout {
    customization_id: Option<String>,
    rows: Arc<[InspectorLayoutRow]>,
}

impl InspectorLayout {
    pub fn customization_id(&self) -> Option<&str> {
        self.customization_id.as_deref()
    }

    pub fn rows(&self) -> &[InspectorLayoutRow] {
        &self.rows
    }
}

#[derive(Default)]
/// 按注册顺序执行首匹配的 Inspector 定制链；从贡献快照构造时顺序由定制 ID 的键序决定，命中后不自动追加反射字段。
pub struct InspectorCustomizationChain {
    customizations: Vec<Arc<dyn InspectorCustomization>>,
    ids: HashSet<String>,
}

impl InspectorCustomizationChain {
    /// 构建 UI 前校验自定义项和 retained UI 声明，防止无效贡献进入匹配链。
    pub fn register(
        &mut self,
        customization: Arc<dyn InspectorCustomization>,
    ) -> Result<(), InspectorRegistrationError> {
        customization.validate()?;
        if let Some(surface) = customization.surface() {
            surface.validate()?;
        }
        let id = customization.id().to_string();
        if !self.ids.insert(id.clone()) {
            return Err(InspectorRegistrationError::DuplicateCustomization(id));
        }
        self.customizations.push(customization);
        Ok(())
    }

    /// 传入当前快照对应的字段编辑器容器；无人接管时才走自动字段布局。
    pub fn build(
        &self,
        target: &InspectTarget,
        fields: impl IntoIterator<Item = InspectorField>,
        editors: &FieldEditorContainer,
    ) -> InspectorLayout {
        let mut layout = InspectorLayoutBuilder::default();
        if let Some(customization) = self.matching(target) {
            customization.build(target, &mut layout);
            return InspectorLayout {
                customization_id: Some(customization.id().to_string()),
                rows: layout.rows.into(),
            };
        }
        for field in fields {
            layout.add_auto_field(field, editors);
        }
        InspectorLayout {
            customization_id: None,
            rows: layout.rows.into(),
        }
    }

    pub fn matching(&self, target: &InspectTarget) -> Option<&dyn InspectorCustomization> {
        self.customizations
            .iter()
            .find(|customization| customization.can_handle(target.target_type()))
            .map(|customization| customization.as_ref())
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum InspectorRegistrationError {
    #[error("inspector type name must be a stable non-empty identifier: {0:?}")]
    InvalidTypeName(String),
    #[error("inspector target id must be non-empty and trimmed: {0:?}")]
    InvalidTargetId(String),
    #[error("inspector field id must be non-empty and trimmed: {0:?}")]
    InvalidFieldId(String),
    #[error("inspector field label must be non-empty: {0:?}")]
    InvalidFieldLabel(String),
    #[error("inspector field editor already exists for type {0:?}")]
    DuplicateFieldEditor(String),
    #[error("inspector field editor type must use its canonical lookup key: {0:?}")]
    NonCanonicalFieldEditorType(String),
    #[error("inspector customization id must be a stable non-empty identifier: {0:?}")]
    InvalidCustomizationId(String),
    #[error("inspector customization already exists: {0:?}")]
    DuplicateCustomization(String),
    #[error("inspector customization UI document must be a trimmed .zui path: {0:?}")]
    InvalidCustomizationUiDocument(String),
    #[error("inspector customization controller must be a stable non-empty identifier: {0:?}")]
    InvalidCustomizationController(String),
    #[error("inspector customization template id must be non-empty and trimmed: {0:?}")]
    InvalidCustomizationTemplateId(String),
    #[error("inspector customization data root must be non-empty and trimmed: {0:?}")]
    InvalidCustomizationDataRoot(String),
    #[error("inspector customization binding must be a valid editor operation: {0:?}")]
    InvalidCustomizationBinding(String),
}

fn is_stable_type_name(value: &str) -> bool {
    !value.trim().is_empty()
        && value.trim() == value
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | ':' | '.')
        })
}

fn is_valid_zui_document(value: &str) -> bool {
    is_trimmed_non_empty(value) && value.ends_with(".zui")
}

fn is_trimmed_non_empty(value: &str) -> bool {
    !value.trim().is_empty() && value.trim() == value
}

#[cfg(test)]
#[path = "inspector/tests/hash_customization_tests.rs"]
mod hash_customization_tests;

#[cfg(test)]
#[path = "tests/inspector.rs"]
mod tests;
