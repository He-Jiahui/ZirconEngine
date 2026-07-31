use std::collections::BTreeSet;

use serde::{Deserialize, Deserializer, Serialize};
use zircon_runtime_interface::ui::design_tokens::EditorDesignTokens;

use super::{EditorCommandPaletteMru, EditorKeymapOverrides, SettingsScope};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct SettingsKey(String);

impl SettingsKey {
    pub fn parse(value: impl Into<String>) -> Result<Self, String> {
        let value = value.into();
        if value.is_empty()
            || value.starts_with('.')
            || value.ends_with('.')
            || value.split('.').any(|segment| {
                segment.is_empty()
                    || !segment.bytes().all(|byte| {
                        byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'
                    })
            })
        {
            return Err(format!(
                "settings key `{value}` must use non-empty lowercase dot-separated segments"
            ));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for SettingsKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(value).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum SettingValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Enum(String),
    Color([u8; 4]),
    Chord(String),
    DesignTokens(EditorDesignTokens),
    KeymapOverrides(EditorKeymapOverrides),
    CommandPaletteMru(EditorCommandPaletteMru),
}

#[derive(Clone, Debug, PartialEq)]
pub enum SettingSchema {
    Bool,
    Int { minimum: i64, maximum: i64 },
    Float { minimum: f64, maximum: f64 },
    String { maximum_bytes: usize },
    Enum { variants: BTreeSet<String> },
    Color,
    Chord,
    DesignTokens,
    KeymapOverrides,
    CommandPaletteMru,
}

impl SettingSchema {
    pub(crate) fn validate(&self, value: &SettingValue) -> Result<(), String> {
        match (self, value) {
            (Self::Bool, SettingValue::Bool(_))
            | (Self::Color, SettingValue::Color(_))
            | (Self::DesignTokens, SettingValue::DesignTokens(_))
            | (Self::KeymapOverrides, SettingValue::KeymapOverrides(_))
            | (Self::CommandPaletteMru, SettingValue::CommandPaletteMru(_)) => Ok(()),
            (Self::Int { minimum, maximum }, SettingValue::Int(value))
                if minimum <= maximum && (*minimum..=*maximum).contains(value) =>
            {
                Ok(())
            }
            (Self::Float { minimum, maximum }, SettingValue::Float(value))
                if minimum.is_finite()
                    && maximum.is_finite()
                    && minimum <= maximum
                    && value.is_finite()
                    && (*minimum..=*maximum).contains(value) =>
            {
                Ok(())
            }
            (Self::String { maximum_bytes }, SettingValue::String(value))
                if value.len() <= *maximum_bytes =>
            {
                Ok(())
            }
            (Self::Enum { variants }, SettingValue::Enum(value)) if variants.contains(value) => {
                Ok(())
            }
            (Self::Chord, SettingValue::Chord(value)) if !value.trim().is_empty() => Ok(()),
            _ => Err(format!("setting value does not satisfy schema {self:?}")),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SettingDefinition {
    pub key: SettingsKey,
    pub scope: SettingsScope,
    pub schema: SettingSchema,
    pub default: SettingValue,
    pub requires_restart: bool,
    pub category_path: String,
}

impl SettingDefinition {
    pub fn new(
        key: SettingsKey,
        scope: SettingsScope,
        schema: SettingSchema,
        default: SettingValue,
        requires_restart: bool,
        category_path: impl Into<String>,
    ) -> Result<Self, String> {
        let category_path = category_path.into();
        let definition = Self {
            key,
            scope,
            schema,
            default,
            requires_restart,
            category_path,
        };
        definition.validate()?;
        Ok(definition)
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        validate_schema_definition(&self.schema)?;
        self.schema.validate(&self.default)?;
        if self.category_path.is_empty() || self.category_path.split('/').any(str::is_empty) {
            return Err("setting category path must use non-empty slash-separated segments".into());
        }
        Ok(())
    }
}

fn validate_schema_definition(schema: &SettingSchema) -> Result<(), String> {
    match schema {
        SettingSchema::Int { minimum, maximum } if minimum > maximum => {
            Err("integer setting minimum must not exceed maximum".into())
        }
        SettingSchema::Float { minimum, maximum }
            if !minimum.is_finite() || !maximum.is_finite() || minimum > maximum =>
        {
            Err("float setting bounds must be finite and ordered".into())
        }
        SettingSchema::Enum { variants } if variants.is_empty() => {
            Err("enum settings must define at least one variant".into())
        }
        _ => Ok(()),
    }
}
