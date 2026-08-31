use std::{
    collections::{btree_map::Entry, BTreeMap},
    fmt,
};

use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use thiserror::Error;

use super::{
    super::{UiModelProviderKey, UiModelSchemaKey},
    UiBindingValue,
};

pub const UI_BINDING_VALUE_MAX_DEPTH: usize = 64;
pub const UI_BINDING_VALUE_MAX_NODES: usize = 1_024;
pub const UI_BINDING_VALUE_MAX_STRING_BYTES: usize = 16 * 1_024;
pub const UI_BINDING_VALUE_MAX_COLLECTION_ENTRIES: usize = 256;
pub const UI_BINDING_VALUE_IDENTITY_MAX_BYTES: usize = 256;
pub const UI_BINDING_COLLECTION_VIEW_MAX_LENGTH: u32 = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiBindingValueBudget {
    pub max_depth: usize,
    pub max_nodes: usize,
    pub max_string_bytes: usize,
    pub max_collection_entries: usize,
}

impl UiBindingValueBudget {
    pub const STANDARD: Self = Self::new(
        UI_BINDING_VALUE_MAX_DEPTH,
        UI_BINDING_VALUE_MAX_NODES,
        UI_BINDING_VALUE_MAX_STRING_BYTES,
        UI_BINDING_VALUE_MAX_COLLECTION_ENTRIES,
    );

    pub const fn new(
        max_depth: usize,
        max_nodes: usize,
        max_string_bytes: usize,
        max_collection_entries: usize,
    ) -> Self {
        Self {
            max_depth,
            max_nodes,
            max_string_bytes,
            max_collection_entries,
        }
    }
}

impl Default for UiBindingValueBudget {
    fn default() -> Self {
        Self::STANDARD
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiBindingValueIdentityKind {
    RecordField,
    EnumType,
    EnumVariant,
    AssetLocator,
    EntityGeneration,
    CollectionRevision,
}

impl fmt::Display for UiBindingValueIdentityKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::RecordField => "record field",
            Self::EnumType => "enum type",
            Self::EnumVariant => "enum variant",
            Self::AssetLocator => "asset locator",
            Self::EntityGeneration => "entity generation",
            Self::CollectionRevision => "collection revision",
        })
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum UiBindingValueValidationError {
    #[error("{kind} cannot be empty")]
    EmptyIdentity { kind: UiBindingValueIdentityKind },
    #[error("{kind} uses {actual_bytes} bytes, exceeding the {maximum_bytes}-byte limit")]
    IdentityTooLong {
        kind: UiBindingValueIdentityKind,
        actual_bytes: usize,
        maximum_bytes: usize,
    },
    #[error("{kind} must be non-zero")]
    ZeroGeneration { kind: UiBindingValueIdentityKind },
    #[error("binding value contains a non-finite float")]
    NonFiniteFloat,
    #[error("binding value depth {actual} exceeds the configured limit {maximum}")]
    DepthExceeded { actual: usize, maximum: usize },
    #[error("binding value node count {actual} exceeds the configured limit {maximum}")]
    NodeBudgetExceeded { actual: usize, maximum: usize },
    #[error("binding value string bytes {actual} exceed the configured limit {maximum}")]
    StringBudgetExceeded { actual: usize, maximum: usize },
    #[error(
        "binding value collection has {actual} entries, exceeding the configured limit {maximum}"
    )]
    CollectionEntriesExceeded { actual: usize, maximum: usize },
    #[error("binding map contains a duplicate key")]
    DuplicateMapKey,
    #[error("collection view window length {actual} exceeds the supported limit {maximum}")]
    CollectionViewWindowExceeded { actual: u32, maximum: u32 },
    #[error(
        "collection view window offset {offset} and length {length} exceed total length {total_length}"
    )]
    InvalidCollectionViewWindow {
        offset: u64,
        length: u32,
        total_length: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum UiBindingMapKey {
    String(String),
    Unsigned(u64),
    Signed(i64),
    Bool(bool),
}

impl UiBindingMapKey {
    pub(super) fn native_repr(&self) -> String {
        match self {
            Self::String(value) => quoted(value),
            Self::Unsigned(value) => value.to_string(),
            Self::Signed(value) => value.to_string(),
            Self::Bool(value) => value.to_string(),
        }
    }

    pub(super) fn to_json_value(&self) -> Value {
        match self {
            Self::String(value) => Value::String(value.clone()),
            Self::Unsigned(value) => Value::Number((*value).into()),
            Self::Signed(value) => Value::Number((*value).into()),
            Self::Bool(value) => Value::Bool(*value),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiBindingMap(BTreeMap<UiBindingMapKey, UiBindingValue>);

impl UiBindingMap {
    pub fn try_from_entries(
        entries: impl IntoIterator<Item = (UiBindingMapKey, UiBindingValue)>,
    ) -> Result<Self, UiBindingValueValidationError> {
        let mut values = BTreeMap::new();
        for (key, value) in entries {
            match values.entry(key) {
                Entry::Vacant(entry) => {
                    entry.insert(value);
                }
                Entry::Occupied(_) => return Err(UiBindingValueValidationError::DuplicateMapKey),
            }
        }
        Ok(Self(values))
    }

    pub fn get(&self, key: &UiBindingMapKey) -> Option<&UiBindingValue> {
        self.0.get(key)
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = (&UiBindingMapKey, &UiBindingValue)> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Serialize)]
struct UiBindingMapEntryRef<'a> {
    key: &'a UiBindingMapKey,
    value: &'a UiBindingValue,
}

#[derive(Deserialize)]
struct UiBindingMapEntry {
    key: UiBindingMapKey,
    value: UiBindingValue,
}

impl Serialize for UiBindingMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0
            .iter()
            .map(|(key, value)| UiBindingMapEntryRef { key, value })
            .collect::<Vec<_>>()
            .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for UiBindingMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let entries = Vec::<UiBindingMapEntry>::deserialize(deserializer)?;
        Self::try_from_entries(entries.into_iter().map(|entry| (entry.key, entry.value)))
            .map_err(D::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct UiBindingEnumValue {
    type_id: String,
    variant: String,
    payload: Option<Box<UiBindingValue>>,
}

impl UiBindingEnumValue {
    pub fn try_new(
        type_id: impl Into<String>,
        variant: impl Into<String>,
        payload: Option<UiBindingValue>,
    ) -> Result<Self, UiBindingValueValidationError> {
        let type_id = type_id.into();
        let variant = variant.into();
        validate_identity(UiBindingValueIdentityKind::EnumType, &type_id)?;
        validate_identity(UiBindingValueIdentityKind::EnumVariant, &variant)?;
        Ok(Self {
            type_id,
            variant,
            payload: payload.map(Box::new),
        })
    }

    pub fn type_id(&self) -> &str {
        &self.type_id
    }

    pub fn variant(&self) -> &str {
        &self.variant
    }

    pub fn payload(&self) -> Option<&UiBindingValue> {
        self.payload.as_deref()
    }
}

#[derive(Deserialize)]
struct UiBindingEnumValueWire {
    type_id: String,
    variant: String,
    payload: Option<Box<UiBindingValue>>,
}

impl<'de> Deserialize<'de> for UiBindingEnumValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = UiBindingEnumValueWire::deserialize(deserializer)?;
        Self::try_new(wire.type_id, wire.variant, wire.payload.map(|value| *value))
            .map_err(D::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct UiBindingAssetReference(String);

impl UiBindingAssetReference {
    pub fn try_new(locator: impl Into<String>) -> Result<Self, UiBindingValueValidationError> {
        let locator = locator.into();
        validate_identity(UiBindingValueIdentityKind::AssetLocator, &locator)?;
        Ok(Self(locator))
    }

    pub fn locator(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for UiBindingAssetReference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_new(String::deserialize(deserializer)?).map_err(D::Error::custom)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct UiBindingEntityReference {
    entity_id: u64,
    generation: u64,
}

impl UiBindingEntityReference {
    pub fn try_new(entity_id: u64, generation: u64) -> Result<Self, UiBindingValueValidationError> {
        if generation == 0 {
            return Err(UiBindingValueValidationError::ZeroGeneration {
                kind: UiBindingValueIdentityKind::EntityGeneration,
            });
        }
        Ok(Self {
            entity_id,
            generation,
        })
    }

    pub const fn entity_id(self) -> u64 {
        self.entity_id
    }

    pub const fn generation(self) -> u64 {
        self.generation
    }
}

#[derive(Deserialize)]
struct UiBindingEntityReferenceWire {
    entity_id: u64,
    generation: u64,
}

impl<'de> Deserialize<'de> for UiBindingEntityReference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = UiBindingEntityReferenceWire::deserialize(deserializer)?;
        Self::try_new(wire.entity_id, wire.generation).map_err(D::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct UiBindingCollectionView {
    provider: UiModelProviderKey,
    item_schema: UiModelSchemaKey,
    revision: u64,
    offset: u64,
    length: u32,
    total_length: u64,
}

impl UiBindingCollectionView {
    pub fn try_new(
        provider: UiModelProviderKey,
        item_schema: UiModelSchemaKey,
        revision: u64,
        offset: u64,
        length: u32,
        total_length: u64,
    ) -> Result<Self, UiBindingValueValidationError> {
        let value = Self {
            provider,
            item_schema,
            revision,
            offset,
            length,
            total_length,
        };
        value.validate_window()?;
        Ok(value)
    }

    pub fn provider(&self) -> &UiModelProviderKey {
        &self.provider
    }

    pub fn item_schema(&self) -> &UiModelSchemaKey {
        &self.item_schema
    }

    pub const fn revision(&self) -> u64 {
        self.revision
    }

    pub const fn offset(&self) -> u64 {
        self.offset
    }

    pub const fn length(&self) -> u32 {
        self.length
    }

    pub const fn total_length(&self) -> u64 {
        self.total_length
    }

    pub(super) fn validate_window(&self) -> Result<(), UiBindingValueValidationError> {
        if self.revision == 0 {
            return Err(UiBindingValueValidationError::ZeroGeneration {
                kind: UiBindingValueIdentityKind::CollectionRevision,
            });
        }
        if self.length > UI_BINDING_COLLECTION_VIEW_MAX_LENGTH {
            return Err(
                UiBindingValueValidationError::CollectionViewWindowExceeded {
                    actual: self.length,
                    maximum: UI_BINDING_COLLECTION_VIEW_MAX_LENGTH,
                },
            );
        }
        if self
            .offset
            .checked_add(u64::from(self.length))
            .map_or(true, |end| end > self.total_length)
        {
            return Err(UiBindingValueValidationError::InvalidCollectionViewWindow {
                offset: self.offset,
                length: self.length,
                total_length: self.total_length,
            });
        }
        Ok(())
    }
}

#[derive(Deserialize)]
struct UiBindingCollectionViewWire {
    provider: UiModelProviderKey,
    item_schema: UiModelSchemaKey,
    revision: u64,
    offset: u64,
    length: u32,
    total_length: u64,
}

impl<'de> Deserialize<'de> for UiBindingCollectionView {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = UiBindingCollectionViewWire::deserialize(deserializer)?;
        Self::try_new(
            wire.provider,
            wire.item_schema,
            wire.revision,
            wire.offset,
            wire.length,
            wire.total_length,
        )
        .map_err(D::Error::custom)
    }
}

pub(super) fn validate_identity(
    kind: UiBindingValueIdentityKind,
    value: &str,
) -> Result<(), UiBindingValueValidationError> {
    if value.is_empty() {
        return Err(UiBindingValueValidationError::EmptyIdentity { kind });
    }
    if value.len() > UI_BINDING_VALUE_IDENTITY_MAX_BYTES {
        return Err(UiBindingValueValidationError::IdentityTooLong {
            kind,
            actual_bytes: value.len(),
            maximum_bytes: UI_BINDING_VALUE_IDENTITY_MAX_BYTES,
        });
    }
    Ok(())
}

pub(super) fn quoted(value: &str) -> String {
    format!("\"{}\"", escape_string(value))
}

fn escape_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
