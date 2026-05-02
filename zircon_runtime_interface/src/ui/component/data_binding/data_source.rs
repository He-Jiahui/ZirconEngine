use serde::{Deserialize, Serialize};

use crate::ui::component::{UiValidationLevel, UiValueKind};

/// Describes an editor/runtime data source that component events can target.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiComponentDataSourceDescriptor {
    pub domain: String,
    pub source_name: String,
    pub display_name: String,
    pub kind: UiComponentDataSourceKind,
    pub subject: Option<String>,
    pub path_prefix: Option<String>,
    pub writable: bool,
    pub value_kinds: Vec<UiValueKind>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<UiComponentDataSourceFieldDescriptor>,
}

impl UiComponentDataSourceDescriptor {
    pub fn new(
        domain: impl Into<String>,
        source_name: impl Into<String>,
        display_name: impl Into<String>,
        kind: UiComponentDataSourceKind,
    ) -> Self {
        Self {
            domain: domain.into(),
            source_name: source_name.into(),
            display_name: display_name.into(),
            kind,
            subject: None,
            path_prefix: None,
            writable: false,
            value_kinds: Vec::new(),
            fields: Vec::new(),
        }
    }

    pub fn with_subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    pub fn with_path_prefix(mut self, path_prefix: impl Into<String>) -> Self {
        self.path_prefix = Some(path_prefix.into());
        self
    }

    pub fn writable(mut self, writable: bool) -> Self {
        self.writable = writable;
        self
    }

    pub fn with_value_kinds(mut self, value_kinds: impl IntoIterator<Item = UiValueKind>) -> Self {
        self.value_kinds = value_kinds.into_iter().collect();
        self
    }

    pub fn with_field(mut self, field: UiComponentDataSourceFieldDescriptor) -> Self {
        self.fields.push(field);
        self
    }

    pub fn with_fields(
        mut self,
        fields: impl IntoIterator<Item = UiComponentDataSourceFieldDescriptor>,
    ) -> Self {
        self.fields = fields.into_iter().collect();
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiComponentDataSourceKind {
    Inspector,
    Reflection,
    AssetEditor,
    Showcase,
    Custom(String),
}

/// Describes one bindable property row resolved from a data source.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiComponentDataSourceFieldDescriptor {
    pub path: String,
    pub display_name: String,
    pub value_kind: UiValueKind,
    pub writable: bool,
    pub group: Option<String>,
    pub collapsed: bool,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
    pub options: Vec<UiComponentDataSourceFieldOption>,
    pub element_kind: Option<UiValueKind>,
    pub key_kind: Option<UiValueKind>,
    pub map_value_kind: Option<UiValueKind>,
    pub reference_kind: Option<String>,
    pub validation_level: Option<UiValidationLevel>,
    pub validation_message: Option<String>,
}

impl UiComponentDataSourceFieldDescriptor {
    pub fn new(
        path: impl Into<String>,
        display_name: impl Into<String>,
        value_kind: UiValueKind,
    ) -> Self {
        Self {
            path: path.into(),
            display_name: display_name.into(),
            value_kind,
            writable: false,
            group: None,
            collapsed: false,
            min: None,
            max: None,
            step: None,
            options: Vec::new(),
            element_kind: None,
            key_kind: None,
            map_value_kind: None,
            reference_kind: None,
            validation_level: None,
            validation_message: None,
        }
    }

    pub fn writable(mut self, writable: bool) -> Self {
        self.writable = writable;
        self
    }

    pub fn group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
        self
    }

    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }

    pub fn range(mut self, min: f64, max: f64) -> Self {
        self.min = Some(min);
        self.max = Some(max);
        self
    }

    pub fn step(mut self, step: f64) -> Self {
        self.step = Some(step);
        self
    }

    pub fn options(
        mut self,
        options: impl IntoIterator<Item = UiComponentDataSourceFieldOption>,
    ) -> Self {
        self.options = options.into_iter().collect();
        self
    }

    pub fn array_element_kind(mut self, element_kind: UiValueKind) -> Self {
        self.element_kind = Some(element_kind);
        self
    }

    pub fn map_kinds(mut self, key_kind: UiValueKind, value_kind: UiValueKind) -> Self {
        self.key_kind = Some(key_kind);
        self.map_value_kind = Some(value_kind);
        self
    }

    pub fn reference_kind(mut self, reference_kind: impl Into<String>) -> Self {
        self.reference_kind = Some(reference_kind.into());
        self
    }

    pub fn validation(mut self, level: UiValidationLevel, message: impl Into<String>) -> Self {
        self.validation_level = Some(level);
        self.validation_message = Some(message.into());
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiComponentDataSourceFieldOption {
    pub id: String,
    pub display_name: String,
    pub disabled: bool,
}

impl UiComponentDataSourceFieldOption {
    pub fn new(id: impl Into<String>, display_name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}
