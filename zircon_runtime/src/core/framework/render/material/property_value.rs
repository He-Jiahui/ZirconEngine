use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

// Typed values stay independent from source TOML so runtime preparation can
// encode them without re-reading material documents.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RenderMaterialPropertyValue {
    Bool { value: bool },
    Float { value: f32 },
    Int { value: i32 },
    UInt { value: u32 },
    String { value: String },
    Vec2 { value: [f32; 2] },
    Vec3 { value: [f32; 3] },
    Vec4 { value: [f32; 4] },
}

impl RenderMaterialPropertyValue {
    pub const fn is_uniform_eligible(&self) -> bool {
        !matches!(self, Self::String { .. })
    }
}

// Compact inspection data for prepared shader property values before uniform byte encoding.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderMaterialPropertyValueSummary {
    pub total_count: usize,
    pub bool_count: usize,
    pub float_count: usize,
    pub int_count: usize,
    pub uint_count: usize,
    pub string_count: usize,
    pub vec2_count: usize,
    pub vec3_count: usize,
    pub vec4_count: usize,
}

impl RenderMaterialPropertyValueSummary {
    pub fn from_values(values: &BTreeMap<String, RenderMaterialPropertyValue>) -> Self {
        let mut summary = Self {
            total_count: values.len(),
            ..Self::default()
        };
        for value in values.values() {
            match value {
                RenderMaterialPropertyValue::Bool { .. } => summary.bool_count += 1,
                RenderMaterialPropertyValue::Float { .. } => summary.float_count += 1,
                RenderMaterialPropertyValue::Int { .. } => summary.int_count += 1,
                RenderMaterialPropertyValue::UInt { .. } => summary.uint_count += 1,
                RenderMaterialPropertyValue::String { .. } => summary.string_count += 1,
                RenderMaterialPropertyValue::Vec2 { .. } => summary.vec2_count += 1,
                RenderMaterialPropertyValue::Vec3 { .. } => summary.vec3_count += 1,
                RenderMaterialPropertyValue::Vec4 { .. } => summary.vec4_count += 1,
            }
        }
        summary
    }

    pub const fn uniform_eligible_count(&self) -> usize {
        self.bool_count
            + self.float_count
            + self.int_count
            + self.uint_count
            + self.vec2_count
            + self.vec3_count
            + self.vec4_count
    }

    pub const fn non_uniform_count(&self) -> usize {
        self.string_count
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenderMaterialPropertyValueState {
    pub name: String,
    pub value: RenderMaterialPropertyValue,
}

impl RenderMaterialPropertyValueState {
    pub fn from_values(values: &BTreeMap<String, RenderMaterialPropertyValue>) -> Vec<Self> {
        values
            .iter()
            .map(|(name, value)| Self {
                name: name.clone(),
                value: value.clone(),
            })
            .collect()
    }

    pub const fn is_uniform_eligible(&self) -> bool {
        self.value.is_uniform_eligible()
    }
}

#[cfg(test)]
#[path = "tests/property_value.rs"]
mod tests;
