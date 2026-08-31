use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::reflect::{
    ReflectEditorHint, ReflectFieldInfo, ReflectNumericRange, ReflectScriptVisibility,
    ReflectSerializationStrategy, ReflectTypeKind, ReflectTypeRegistration, ReflectTypeRole,
    ReflectedValue,
};

use super::ReflectSchemaCatalogEntry;

pub const REFLECT_SCHEMA_CATALOG_ALGORITHM_VERSION: u32 = 1;
const FINGERPRINT_DOMAIN: &[u8] = b"zircon-reflect-schema-catalog-v1\0";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ReflectSchemaFingerprint([u8; blake3::OUT_LEN]);

impl ReflectSchemaFingerprint {
    pub fn as_bytes(&self) -> &[u8; blake3::OUT_LEN] {
        &self.0
    }
}

impl Display for ReflectSchemaFingerprint {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(blake3::Hash::from_bytes(self.0).to_hex().as_str())
    }
}

impl Serialize for ReflectSchemaFingerprint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ReflectSchemaFingerprint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let encoded = String::deserialize(deserializer)?;
        blake3::Hash::from_hex(&encoded)
            .map(|hash| Self(*hash.as_bytes()))
            .map_err(serde::de::Error::custom)
    }
}

pub(super) fn fingerprint(
    entries: &BTreeMap<String, ReflectSchemaCatalogEntry>,
) -> ReflectSchemaFingerprint {
    let mut hasher = blake3::Hasher::new();
    hasher.update(FINGERPRINT_DOMAIN);
    hash_u32(&mut hasher, REFLECT_SCHEMA_CATALOG_ALGORITHM_VERSION);
    hash_len(&mut hasher, entries.len());
    for entry in entries.values() {
        hash_registration(&mut hasher, &entry.registration);
        hash_len(&mut hasher, entry.dependencies.len());
        for dependency in &entry.dependencies {
            hash_str(&mut hasher, dependency);
        }
    }
    ReflectSchemaFingerprint(*hasher.finalize().as_bytes())
}

fn hash_registration(hasher: &mut blake3::Hasher, registration: &ReflectTypeRegistration) {
    let type_path = &registration.type_path;
    hash_str(hasher, type_path.type_path());
    hash_str(hasher, type_path.short_type_path());
    hash_optional_str(hasher, type_path.module_path());
    hash_optional_str(hasher, type_path.plugin_id());
    hash_str(hasher, &registration.display_name);
    hash_optional_str(hasher, registration.documentation.as_deref());
    hash_u8(hasher, type_kind_tag(&registration.type_info.kind));
    hash_len(hasher, registration.type_info.fields.len());
    for field in &registration.type_info.fields {
        hash_field(hasher, field);
    }
    hash_u8(hasher, serialization_tag(&registration.serialization));
    hash_u8(hasher, role_tag(registration.role));
    hash_bool(hasher, registration.serializable);
    hash_bool(hasher, registration.editor_visible);
    hash_bool(hasher, registration.remote_visible);
    hash_u8(
        hasher,
        script_visibility_tag(registration.script_visibility),
    );
}

fn hash_field(hasher: &mut blake3::Hasher, field: &ReflectFieldInfo) {
    hasher.update(field.id.as_uuid().as_bytes());
    hash_str(hasher, &field.name);
    hash_str(hasher, &field.display_name);
    hash_len(hasher, field.aliases.len());
    for alias in &field.aliases {
        hash_str(hasher, alias);
    }
    hash_str(hasher, &field.value_type_path);
    hash_bool(hasher, field.editable);
    hash_bool(hasher, field.serializable);
    hash_bool(hasher, field.editor_visible);
    hash_optional_reflected_value(hasher, field.default_value.as_ref());
    hash_optional_numeric_range(hasher, field.numeric_range.as_ref());
    hash_len(hasher, field.enum_options.len());
    for option in &field.enum_options {
        hash_str(hasher, &option.value);
        hash_str(hasher, &option.display_name);
        hash_optional_str(hasher, option.documentation.as_deref());
    }
    hash_u8(hasher, editor_hint_tag(&field.editor_hint));
    hash_optional_str(hasher, field.documentation.as_deref());
}

fn hash_optional_reflected_value(hasher: &mut blake3::Hasher, value: Option<&ReflectedValue>) {
    match value {
        Some(value) => {
            hash_u8(hasher, 1);
            hash_reflected_value(hasher, value);
        }
        None => hash_u8(hasher, 0),
    }
}

enum ValueWork<'a> {
    Reflected(&'a ReflectedValue),
    Json(&'a serde_json::Value),
    Text(&'a str),
}

fn hash_reflected_value(hasher: &mut blake3::Hasher, root: &ReflectedValue) {
    let mut work = vec![ValueWork::Reflected(root)];
    while let Some(item) = work.pop() {
        match item {
            ValueWork::Text(value) => hash_str(hasher, value),
            ValueWork::Reflected(value) => match value {
                ReflectedValue::Null => hash_u8(hasher, 0),
                ReflectedValue::Bool(value) => {
                    hash_u8(hasher, 1);
                    hash_bool(hasher, *value);
                }
                ReflectedValue::Integer(value) => {
                    hash_u8(hasher, 2);
                    hasher.update(&value.to_le_bytes());
                }
                ReflectedValue::Unsigned(value) => {
                    hash_u8(hasher, 3);
                    hasher.update(&value.to_le_bytes());
                }
                ReflectedValue::Scalar(value) => {
                    hash_u8(hasher, 4);
                    hasher.update(&value.to_bits().to_le_bytes());
                }
                ReflectedValue::String(value) => {
                    hash_u8(hasher, 5);
                    hash_str(hasher, value);
                }
                ReflectedValue::Enum(value) => {
                    hash_u8(hasher, 6);
                    hash_str(hasher, value);
                }
                ReflectedValue::Vec2(values) => {
                    hash_u8(hasher, 7);
                    hash_floats(hasher, values);
                }
                ReflectedValue::Vec3(values) => {
                    hash_u8(hasher, 8);
                    hash_floats(hasher, values);
                }
                ReflectedValue::Vec4(values) => {
                    hash_u8(hasher, 9);
                    hash_floats(hasher, values);
                }
                ReflectedValue::Quaternion(values) => {
                    hash_u8(hasher, 10);
                    hash_floats(hasher, values);
                }
                ReflectedValue::Entity(value) => {
                    hash_u8(hasher, 11);
                    match value {
                        Some(value) => {
                            hash_u8(hasher, 1);
                            hasher.update(&value.to_le_bytes());
                        }
                        None => hash_u8(hasher, 0),
                    }
                }
                ReflectedValue::Resource(value) => {
                    hash_u8(hasher, 12);
                    hash_str(hasher, value);
                }
                ReflectedValue::List(values) => {
                    hash_u8(hasher, 13);
                    hash_len(hasher, values.len());
                    work.extend(values.iter().rev().map(ValueWork::Reflected));
                }
                ReflectedValue::Map(values) => {
                    hash_u8(hasher, 14);
                    hash_len(hasher, values.len());
                    for (key, value) in values.iter().rev() {
                        work.push(ValueWork::Reflected(value));
                        work.push(ValueWork::Text(key));
                    }
                }
                ReflectedValue::Json(value) => {
                    hash_u8(hasher, 15);
                    work.push(ValueWork::Json(value));
                }
            },
            ValueWork::Json(value) => match value {
                serde_json::Value::Null => hash_u8(hasher, 0),
                serde_json::Value::Bool(value) => {
                    hash_u8(hasher, 1);
                    hash_bool(hasher, *value);
                }
                serde_json::Value::Number(value) => {
                    hash_u8(hasher, 2);
                    hash_str(hasher, &value.to_string());
                }
                serde_json::Value::String(value) => {
                    hash_u8(hasher, 3);
                    hash_str(hasher, value);
                }
                serde_json::Value::Array(values) => {
                    hash_u8(hasher, 4);
                    hash_len(hasher, values.len());
                    work.extend(values.iter().rev().map(ValueWork::Json));
                }
                serde_json::Value::Object(values) => {
                    hash_u8(hasher, 5);
                    hash_len(hasher, values.len());
                    let mut entries = values.iter().collect::<Vec<_>>();
                    entries.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));
                    for (key, value) in entries.into_iter().rev() {
                        work.push(ValueWork::Json(value));
                        work.push(ValueWork::Text(key));
                    }
                }
            },
        }
    }
}

fn hash_optional_numeric_range(hasher: &mut blake3::Hasher, range: Option<&ReflectNumericRange>) {
    let Some(range) = range else {
        hash_u8(hasher, 0);
        return;
    };
    hash_u8(hasher, 1);
    hash_optional_f32(hasher, range.min());
    hash_optional_f32(hasher, range.max());
    hash_optional_f32(hasher, range.step());
    match range.precision() {
        Some(value) => {
            hash_u8(hasher, 1);
            hash_u8(hasher, value);
        }
        None => hash_u8(hasher, 0),
    }
}

fn hash_optional_f32(hasher: &mut blake3::Hasher, value: Option<f32>) {
    match value {
        Some(value) => {
            hash_u8(hasher, 1);
            hasher.update(&value.to_bits().to_le_bytes());
        }
        None => hash_u8(hasher, 0),
    }
}

fn hash_floats<const N: usize>(hasher: &mut blake3::Hasher, values: &[f32; N]) {
    for value in values {
        hasher.update(&value.to_bits().to_le_bytes());
    }
}

fn hash_optional_str(hasher: &mut blake3::Hasher, value: Option<&str>) {
    match value {
        Some(value) => {
            hash_u8(hasher, 1);
            hash_str(hasher, value);
        }
        None => hash_u8(hasher, 0),
    }
}

fn hash_str(hasher: &mut blake3::Hasher, value: &str) {
    hash_len(hasher, value.len());
    hasher.update(value.as_bytes());
}

fn hash_len(hasher: &mut blake3::Hasher, value: usize) {
    hasher.update(&(value as u64).to_le_bytes());
}

fn hash_bool(hasher: &mut blake3::Hasher, value: bool) {
    hash_u8(hasher, u8::from(value));
}

fn hash_u32(hasher: &mut blake3::Hasher, value: u32) {
    hasher.update(&value.to_le_bytes());
}

fn hash_u8(hasher: &mut blake3::Hasher, value: u8) {
    hasher.update(&[value]);
}

fn type_kind_tag(kind: &ReflectTypeKind) -> u8 {
    match kind {
        ReflectTypeKind::Struct => 0,
        ReflectTypeKind::TupleStruct => 1,
        ReflectTypeKind::Tuple => 2,
        ReflectTypeKind::Enum => 3,
        ReflectTypeKind::List => 4,
        ReflectTypeKind::Map => 5,
        ReflectTypeKind::Scalar => 6,
        ReflectTypeKind::Opaque => 7,
        ReflectTypeKind::Json => 8,
    }
}

fn serialization_tag(strategy: &ReflectSerializationStrategy) -> u8 {
    match strategy {
        ReflectSerializationStrategy::None => 0,
        ReflectSerializationStrategy::Value => 1,
        ReflectSerializationStrategy::Json => 2,
        ReflectSerializationStrategy::ResourceHandle => 3,
        ReflectSerializationStrategy::EntityReference => 4,
    }
}

fn role_tag(role: ReflectTypeRole) -> u8 {
    match role {
        ReflectTypeRole::Value => 0,
        ReflectTypeRole::Component => 1,
        ReflectTypeRole::Resource => 2,
    }
}

fn script_visibility_tag(visibility: ReflectScriptVisibility) -> u8 {
    match visibility {
        ReflectScriptVisibility::Private => 0,
        ReflectScriptVisibility::Public => 1,
    }
}

fn editor_hint_tag(hint: &ReflectEditorHint) -> u8 {
    match hint {
        ReflectEditorHint::None => 0,
        ReflectEditorHint::String => 1,
        ReflectEditorHint::MultilineString => 2,
        ReflectEditorHint::Bool => 3,
        ReflectEditorHint::Integer => 4,
        ReflectEditorHint::Unsigned => 5,
        ReflectEditorHint::Scalar => 6,
        ReflectEditorHint::Vec2 => 7,
        ReflectEditorHint::Vec3 => 8,
        ReflectEditorHint::Vec4 => 9,
        ReflectEditorHint::Enum => 10,
        ReflectEditorHint::Entity => 11,
        ReflectEditorHint::Resource => 12,
        ReflectEditorHint::Color => 13,
        ReflectEditorHint::Json => 14,
    }
}
