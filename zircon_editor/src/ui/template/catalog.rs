use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;
use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    layout::UiSlotKind,
    template::{UiAssetError, UiTemplateError},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EditorComponentTier {
    Primitive,
    #[default]
    Composite,
    RegionPanel,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditorSlotContract {
    pub name: String,
    pub kind: UiSlotKind,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub multiple: bool,
    #[serde(default)]
    pub accepts: BTreeSet<String>,
}

impl EditorSlotContract {
    pub fn new(name: impl Into<String>, kind: UiSlotKind) -> Self {
        Self {
            name: name.into(),
            kind,
            required: false,
            multiple: false,
            accepts: BTreeSet::new(),
        }
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn multiple(mut self, multiple: bool) -> Self {
        self.multiple = multiple;
        self
    }

    pub fn accepts<I, S>(mut self, components: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.accepts = components.into_iter().map(Into::into).collect();
        self
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EditorPropLiteral {
    Text(String),
    Boolean(bool),
    Integer(i64),
    Float(f64),
    TextList(Vec<String>),
}

impl From<&str> for EditorPropLiteral {
    fn from(value: &str) -> Self {
        Self::Text(value.to_string())
    }
}

impl From<String> for EditorPropLiteral {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<bool> for EditorPropLiteral {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<i64> for EditorPropLiteral {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

impl From<f64> for EditorPropLiteral {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl From<Vec<String>> for EditorPropLiteral {
    fn from(value: Vec<String>) -> Self {
        Self::TextList(value)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EditorPropDefault {
    None,
    Literal(EditorPropLiteral),
    Token(String),
}

impl Default for EditorPropDefault {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct EditorPropContract {
    pub name: String,
    pub value_type: String,
    #[serde(default)]
    pub default: EditorPropDefault,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_token: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EditorPropContractWire {
    name: String,
    value_type: String,
    #[serde(default)]
    default: Option<EditorPropDefault>,
    #[serde(default)]
    default_token: Option<String>,
}

impl<'de> Deserialize<'de> for EditorPropContract {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = EditorPropContractWire::deserialize(deserializer)?;
        let default = wire.default.unwrap_or_else(|| {
            wire.default_token
                .as_ref()
                .map(|token| EditorPropDefault::Token(token.clone()))
                .unwrap_or_default()
        });
        let default_token = match &default {
            EditorPropDefault::Token(token) => Some(token.clone()),
            EditorPropDefault::None | EditorPropDefault::Literal(_) => None,
        };

        Ok(Self {
            name: wire.name,
            value_type: wire.value_type,
            default,
            default_token,
        })
    }
}

impl EditorPropContract {
    pub fn literal_default(
        name: impl Into<String>,
        value_type: impl Into<String>,
        default_value: impl Into<EditorPropLiteral>,
    ) -> Self {
        Self {
            name: name.into(),
            value_type: value_type.into(),
            default: EditorPropDefault::Literal(default_value.into()),
            default_token: None,
        }
    }

    pub fn token_default(
        name: impl Into<String>,
        value_type: impl Into<String>,
        default_token: impl Into<String>,
    ) -> Self {
        let default_token = default_token.into();
        Self {
            name: name.into(),
            value_type: value_type.into(),
            default: EditorPropDefault::Token(default_token.clone()),
            default_token: Some(default_token),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditorComponentDescriptor {
    pub component_id: String,
    pub document_id: String,
    pub binding_namespace: String,
    #[serde(default)]
    pub tier: EditorComponentTier,
    #[serde(default)]
    pub slots: Vec<EditorSlotContract>,
    #[serde(default)]
    pub props: Vec<EditorPropContract>,
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
            tier: EditorComponentTier::default(),
            slots: Vec::new(),
            props: Vec::new(),
        }
    }

    pub fn with_tier(mut self, tier: EditorComponentTier) -> Self {
        self.tier = tier;
        self
    }

    pub fn with_slot(mut self, slot: EditorSlotContract) -> Self {
        self.slots.push(slot);
        self
    }

    pub fn with_prop(mut self, prop: EditorPropContract) -> Self {
        self.props.push(prop);
        self
    }

    pub fn with_literal_prop(
        self,
        name: impl Into<String>,
        value_type: impl Into<String>,
        default_value: impl Into<EditorPropLiteral>,
    ) -> Self {
        self.with_prop(EditorPropContract::literal_default(
            name,
            value_type,
            default_value,
        ))
    }

    pub fn with_token_prop(
        self,
        name: impl Into<String>,
        value_type: impl Into<String>,
        default_token: impl Into<String>,
    ) -> Self {
        self.with_prop(EditorPropContract::token_default(
            name,
            value_type,
            default_token,
        ))
    }
}

pub const EDITOR_COMPONENT_CATALOG_MANIFEST_FORMAT_VERSION: u32 = 1;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EditorComponentCatalogManifest {
    version: u32,
    #[serde(default)]
    components: Vec<EditorComponentDescriptor>,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum EditorComponentCatalogManifestError {
    #[error("editor component catalog manifest could not be parsed: {detail}")]
    Parse { detail: String },
    #[error(
        "editor component catalog format version {version} is unsupported; expected {expected}"
    )]
    UnsupportedVersion { version: u32, expected: u32 },
    #[error("editor component catalog manifest must contain at least one component")]
    Empty,
    #[error("editor component catalog contains duplicate component {component_id}")]
    DuplicateComponent { component_id: String },
    #[error("editor component catalog component {component_id} contains duplicate slot {slot}")]
    DuplicateSlot { component_id: String, slot: String },
    #[error(
        "editor component catalog component {component_id} contains duplicate property {property}"
    )]
    DuplicateProperty {
        component_id: String,
        property: String,
    },
    #[error(
        "editor component catalog manifest component {component_id} must reference a packaged res://ui/editor/*.zui UI asset, got {document_id}"
    )]
    InvalidDocumentReference {
        component_id: String,
        document_id: String,
    },
    #[error(
        "editor component catalog manifest component {component_id} property {property} has an invalid token default {token}"
    )]
    InvalidTokenDefault {
        component_id: String,
        property: String,
        token: String,
    },
}

/// Parses the typed editor component catalog manifest before descriptors are registered.
///
/// Keeping this metadata boundary makes component identity, composition contracts, and
/// author-facing document paths one source of truth rather than parallel Rust tables. It is
/// intentionally not a `.zui` descriptor because it does not declare a renderable UI tree.
pub fn parse_editor_component_catalog_manifest(
    source: &str,
) -> Result<Vec<EditorComponentDescriptor>, EditorComponentCatalogManifestError> {
    let catalog: EditorComponentCatalogManifest =
        toml::from_str(source).map_err(|error| EditorComponentCatalogManifestError::Parse {
            detail: error.to_string(),
        })?;
    if catalog.version != EDITOR_COMPONENT_CATALOG_MANIFEST_FORMAT_VERSION {
        return Err(EditorComponentCatalogManifestError::UnsupportedVersion {
            version: catalog.version,
            expected: EDITOR_COMPONENT_CATALOG_MANIFEST_FORMAT_VERSION,
        });
    }
    if catalog.components.is_empty() {
        return Err(EditorComponentCatalogManifestError::Empty);
    }

    let mut component_ids = BTreeSet::new();
    for descriptor in &catalog.components {
        if !is_builtin_editor_component_document_id(&descriptor.document_id) {
            return Err(
                EditorComponentCatalogManifestError::InvalidDocumentReference {
                    component_id: descriptor.component_id.clone(),
                    document_id: descriptor.document_id.clone(),
                },
            );
        }
        let mut slot_names = BTreeSet::new();
        for slot in &descriptor.slots {
            if !slot_names.insert(slot.name.as_str()) {
                return Err(EditorComponentCatalogManifestError::DuplicateSlot {
                    component_id: descriptor.component_id.clone(),
                    slot: slot.name.clone(),
                });
            }
        }
        let mut property_names = BTreeSet::new();
        for property in &descriptor.props {
            if !property_names.insert(property.name.as_str()) {
                return Err(EditorComponentCatalogManifestError::DuplicateProperty {
                    component_id: descriptor.component_id.clone(),
                    property: property.name.clone(),
                });
            }
            let EditorPropDefault::Token(token) = &property.default else {
                continue;
            };
            if !is_token_reference(token) {
                return Err(EditorComponentCatalogManifestError::InvalidTokenDefault {
                    component_id: descriptor.component_id.clone(),
                    property: property.name.clone(),
                    token: token.clone(),
                });
            }
        }
        if !component_ids.insert(descriptor.component_id.as_str()) {
            return Err(EditorComponentCatalogManifestError::DuplicateComponent {
                component_id: descriptor.component_id.clone(),
            });
        }
    }
    Ok(catalog.components)
}

fn is_builtin_editor_component_document_id(document_id: &str) -> bool {
    document_id.starts_with("res://ui/editor/") && document_id.ends_with(".zui")
}

fn is_token_reference(token: &str) -> bool {
    token
        .strip_prefix('$')
        .is_some_and(|token_name| !token_name.is_empty())
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

    pub fn descriptors(&self) -> Vec<&EditorComponentDescriptor> {
        self.descriptors.values().collect()
    }
}
