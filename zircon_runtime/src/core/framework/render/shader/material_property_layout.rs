use std::collections::BTreeMap;

use serde::de::{Error as DeError, Unexpected, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::RenderShaderDefinitionValue;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MaterialPropertyKind {
    Float,
    Vec2,
    Vec3,
    Vec4,
    Color,
    Int,
    UInt,
    Bool,
}

impl MaterialPropertyKind {
    pub const fn token(self) -> &'static str {
        match self {
            Self::Float => "float",
            Self::Vec2 => "vec2",
            Self::Vec3 => "vec3",
            Self::Vec4 => "vec4",
            Self::Color => "color",
            Self::Int => "int",
            Self::UInt => "uint",
            Self::Bool => "bool",
        }
    }

    pub const fn scalar_class(self) -> PropertyScalarClass {
        match self {
            Self::Float | Self::Vec2 | Self::Vec3 | Self::Vec4 | Self::Color => {
                PropertyScalarClass::F32
            }
            Self::Int | Self::UInt | Self::Bool => PropertyScalarClass::U32,
        }
    }

    pub const fn component_count(self) -> u8 {
        match self {
            Self::Float | Self::Int | Self::UInt | Self::Bool => 1,
            Self::Vec2 => 2,
            Self::Vec3 => 3,
            Self::Vec4 | Self::Color => 4,
        }
    }

    pub fn parse_token(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "float" | "f32" | "number" => Some(Self::Float),
            "vec2" | "float2" => Some(Self::Vec2),
            "vec3" | "float3" => Some(Self::Vec3),
            "vec4" | "float4" => Some(Self::Vec4),
            "color" | "color4" => Some(Self::Color),
            "int" | "i32" | "integer" => Some(Self::Int),
            "uint" | "u32" => Some(Self::UInt),
            "bool" | "boolean" => Some(Self::Bool),
            _ => None,
        }
    }
}

impl std::fmt::Display for MaterialPropertyKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.token())
    }
}

impl Serialize for MaterialPropertyKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.token())
    }
}

impl<'de> Deserialize<'de> for MaterialPropertyKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MaterialPropertyKindVisitor;

        impl Visitor<'_> for MaterialPropertyKindVisitor {
            type Value = MaterialPropertyKind;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("one of float, vec2, vec3, vec4, color, int, uint, or bool")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                MaterialPropertyKind::parse_token(value).ok_or_else(|| {
                    DeError::invalid_value(
                        Unexpected::Str(value),
                        &"a supported material property kind",
                    )
                })
            }
        }

        deserializer.deserialize_str(MaterialPropertyKindVisitor)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyScalarClass {
    F32,
    U32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialPropertySlotRef {
    pub name: String,
    pub kind: MaterialPropertyKind,
    pub scalar_class: PropertyScalarClass,
    pub slot: u16,
    pub component: u8,
    pub component_count: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialTextureBindingRef {
    pub name: String,
    pub kind: String,
    pub texture_binding: u16,
    pub sampler_binding: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub option: Option<String>,
    #[serde(default)]
    pub has_st_transform: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialPropertyLayout {
    #[serde(default)]
    pub properties: Vec<MaterialPropertySlotRef>,
    #[serde(default)]
    pub f32_slot_count: u16,
    #[serde(default)]
    pub u32_slot_count: u16,
    #[serde(default)]
    pub packed_size: u32,
    #[serde(default)]
    pub texture_bindings: Vec<MaterialTextureBindingRef>,
    #[serde(default)]
    pub layout_hash: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialOptionKind {
    Bool,
    Enum,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialOptionRef {
    pub name: String,
    pub kind: MaterialOptionKind,
    pub bit_offset: u8,
    pub bit_width: u8,
    #[serde(default)]
    pub enum_values: Vec<String>,
    #[serde(default)]
    pub default_bits: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialOptionTable {
    #[serde(default)]
    pub options: Vec<MaterialOptionRef>,
    #[serde(default)]
    pub total_bits: u8,
}

impl MaterialOptionTable {
    pub fn default_bits(&self) -> u32 {
        self.options.iter().fold(0_u32, |bits, option| {
            let mask = option_bit_mask(option.bit_width);
            bits | ((option.default_bits & mask) << option.bit_offset)
        })
    }

    pub fn bits_for_values(&self, values: &BTreeMap<String, toml::Value>) -> u32 {
        values
            .iter()
            .fold(self.default_bits(), |bits, (name, value)| {
                let Some(option) = self.option(name) else {
                    return bits;
                };
                let Some(value_bits) = option.value_bits(value) else {
                    return bits;
                };
                let local_mask = option_bit_mask(option.bit_width);
                let shifted_mask = local_mask << option.bit_offset;
                (bits & !shifted_mask) | ((value_bits & local_mask) << option.bit_offset)
            })
    }

    pub fn option(&self, name: &str) -> Option<&MaterialOptionRef> {
        self.options.iter().find(|option| option.name == name)
    }

    pub fn definition_values_for_bits(&self, bits: u32) -> Vec<RenderShaderDefinitionValue> {
        self.options
            .iter()
            .map(|option| option.definition_value_for_bits(bits))
            .collect()
    }
}

impl MaterialOptionRef {
    pub fn value_bits(&self, value: &toml::Value) -> Option<u32> {
        match self.kind {
            MaterialOptionKind::Bool => value.as_bool().map(|value| if value { 1 } else { 0 }),
            MaterialOptionKind::Enum => value
                .as_str()
                .and_then(|value| {
                    self.enum_values
                        .iter()
                        .position(|candidate| candidate == value)
                })
                .map(|index| index as u32),
        }
    }

    pub fn expected_value_description(&self) -> String {
        match self.kind {
            MaterialOptionKind::Bool => "bool".to_string(),
            MaterialOptionKind::Enum if self.enum_values.is_empty() => "enum value".to_string(),
            MaterialOptionKind::Enum => {
                format!("one of {}", self.enum_values.join(", "))
            }
        }
    }

    pub fn definition_value_for_bits(&self, bits: u32) -> RenderShaderDefinitionValue {
        let local_bits = (bits >> self.bit_offset) & option_bit_mask(self.bit_width);
        let name = material_option_define_name(&self.name);
        match self.kind {
            MaterialOptionKind::Bool => RenderShaderDefinitionValue::bool(name, local_bits != 0),
            MaterialOptionKind::Enum => RenderShaderDefinitionValue::uint(name, local_bits),
        }
    }
}

fn option_bit_mask(width: u8) -> u32 {
    match width {
        0 => 0,
        32.. => u32::MAX,
        _ => (1_u32 << width) - 1,
    }
}

fn material_option_define_name(name: &str) -> String {
    let mut normalized = String::from("ZR_OPT_");
    for character in name.trim().chars() {
        if character.is_ascii_alphanumeric() {
            normalized.push(character.to_ascii_uppercase());
        } else {
            normalized.push('_');
        }
    }
    normalized
}
