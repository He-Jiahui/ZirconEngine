use std::collections::{BTreeMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::core::framework::render::{
    MaterialPropertyKind, MaterialPropertyLayout, MaterialPropertySlotRef, PropertyScalarClass,
};

use super::{
    MaterialPropertyOverrideBlock, RenderMaterialDiagnosticSource, RenderMaterialPropertyValue,
    RenderMaterialReadinessDiagnostic,
};

// CPU-side material property bytes are prepared once during resource streaming
// so later renderer binding work can upload them without reparsing assets.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RenderMaterialPropertyUniformPayload {
    pub layout: Vec<RenderMaterialPropertyUniformField>,
    pub bytes: Vec<u8>,
    pub unsupported: Vec<RenderMaterialPropertyUniformUnsupported>,
}

impl RenderMaterialPropertyUniformPayload {
    pub fn from_layout_and_values(
        layout: &MaterialPropertyLayout,
        values: &BTreeMap<String, RenderMaterialPropertyValue>,
    ) -> Self {
        let mut payload = Self {
            bytes: vec![0; layout.packed_size as usize],
            ..Self::default()
        };
        for property in &layout.properties {
            let offset = property_uniform_offset(layout, property);
            payload.layout.push(RenderMaterialPropertyUniformField {
                name: property.name.clone(),
                kind: property.kind.to_string(),
                offset: offset as u32,
                size: u32::from(property.component_count) * 4,
                alignment: 4,
            });
            let Some(value) = values.get(&property.name) else {
                continue;
            };
            if !write_layout_value(&mut payload.bytes, offset, property, value) {
                payload
                    .unsupported
                    .push(RenderMaterialPropertyUniformUnsupported {
                        name: property.name.clone(),
                        reason: RenderMaterialPropertyUniformUnsupportedReason::UnsupportedType,
                    });
            }
        }
        let layout_names = layout
            .properties
            .iter()
            .map(|property| property.name.as_str())
            .collect::<HashSet<_>>();
        for (name, value) in values {
            if layout_names.contains(name.as_str()) {
                continue;
            }
            if !value.is_uniform_eligible() {
                payload
                    .unsupported
                    .push(RenderMaterialPropertyUniformUnsupported {
                        name: name.clone(),
                        reason: RenderMaterialPropertyUniformUnsupportedReason::UnsupportedType,
                    });
            }
        }
        if payload.bytes.is_empty() {
            payload.bytes.resize(MATERIAL_PROPERTY_UNIFORM_ALIGNMENT, 0);
        }
        payload
    }

    pub fn from_values(values: &BTreeMap<String, RenderMaterialPropertyValue>) -> Self {
        let mut payload = Self::default();
        for (name, value) in values {
            let Some(encoded) = EncodedUniformValue::from_value(value) else {
                payload
                    .unsupported
                    .push(RenderMaterialPropertyUniformUnsupported {
                        name: name.clone(),
                        reason: RenderMaterialPropertyUniformUnsupportedReason::UnsupportedType,
                    });
                continue;
            };
            let offset = align_to(payload.bytes.len(), encoded.alignment);
            payload.bytes.resize(offset, 0);
            payload.bytes.extend_from_slice(&encoded.bytes);
            payload.layout.push(RenderMaterialPropertyUniformField {
                name: name.clone(),
                kind: value.kind_name().to_string(),
                offset: offset as u32,
                size: encoded.bytes.len() as u32,
                alignment: encoded.alignment as u32,
            });
        }
        let final_size = align_to(payload.bytes.len(), MATERIAL_PROPERTY_UNIFORM_ALIGNMENT);
        payload.bytes.resize(final_size, 0);
        payload
    }

    pub fn with_override_block(&self, overrides: &MaterialPropertyOverrideBlock) -> Self {
        if overrides.is_empty() {
            return self.clone();
        }

        let mut payload = self.clone();
        for (name, value) in overrides.values() {
            let Some(field) = payload.layout.iter().find(|field| field.name == *name) else {
                payload
                    .unsupported
                    .push(RenderMaterialPropertyUniformUnsupported {
                        name: name.clone(),
                        reason: RenderMaterialPropertyUniformUnsupportedReason::UnknownProperty,
                    });
                continue;
            };
            let Some(kind) = MaterialPropertyKind::parse_token(&field.kind) else {
                payload
                    .unsupported
                    .push(RenderMaterialPropertyUniformUnsupported {
                        name: name.clone(),
                        reason: RenderMaterialPropertyUniformUnsupportedReason::UnsupportedType,
                    });
                continue;
            };
            if let Some(reason) = write_field_override_value(&mut payload.bytes, field, kind, value)
            {
                payload
                    .unsupported
                    .push(RenderMaterialPropertyUniformUnsupported {
                        name: name.clone(),
                        reason,
                    });
            }
        }
        payload
    }

    pub fn is_empty(&self) -> bool {
        self.layout.is_empty() && self.bytes.is_empty() && self.unsupported.is_empty()
    }

    pub fn summary(&self) -> RenderMaterialPropertyUniformSummary {
        RenderMaterialPropertyUniformSummary {
            payload_byte_len: self.bytes.len() as u64,
            field_count: self.layout.len(),
            unsupported_count: self.unsupported.len(),
        }
    }

    pub fn unsupported_diagnostics(&self) -> Vec<RenderMaterialReadinessDiagnostic> {
        self.unsupported
            .iter()
            .map(|unsupported| RenderMaterialReadinessDiagnostic {
                source: RenderMaterialDiagnosticSource::MaterialUniform,
                path: format!("uniform.{}", unsupported.name),
                diagnostic: format!(
                    "material property {} cannot be encoded into the renderer uniform payload: {}",
                    unsupported.name,
                    unsupported.reason.description()
                ),
            })
            .collect()
    }
}

fn property_uniform_offset(
    layout: &MaterialPropertyLayout,
    property: &MaterialPropertySlotRef,
) -> usize {
    let class_slot = match property.scalar_class {
        PropertyScalarClass::F32 => property.slot,
        PropertyScalarClass::U32 => layout.f32_slot_count + property.slot,
    };
    usize::from(class_slot) * MATERIAL_PROPERTY_UNIFORM_ALIGNMENT
        + usize::from(property.component) * 4
}

fn write_layout_value(
    bytes: &mut [u8],
    offset: usize,
    property: &MaterialPropertySlotRef,
    value: &RenderMaterialPropertyValue,
) -> bool {
    write_kind_value(bytes, offset, property.kind, value)
}

fn write_field_override_value(
    bytes: &mut [u8],
    field: &RenderMaterialPropertyUniformField,
    kind: MaterialPropertyKind,
    value: &RenderMaterialPropertyValue,
) -> Option<RenderMaterialPropertyUniformUnsupportedReason> {
    let offset = field.offset as usize;
    let size = u32::from(kind.component_count()) as usize * 4;
    if offset
        .checked_add(size)
        .filter(|end| *end <= bytes.len())
        .is_none()
    {
        return Some(RenderMaterialPropertyUniformUnsupportedReason::PayloadOutOfBounds);
    }
    (!write_kind_value(bytes, offset, kind, value))
        .then_some(RenderMaterialPropertyUniformUnsupportedReason::TypeMismatch)
}

fn write_kind_value(
    bytes: &mut [u8],
    offset: usize,
    kind: MaterialPropertyKind,
    value: &RenderMaterialPropertyValue,
) -> bool {
    match (kind, value) {
        (MaterialPropertyKind::Bool, RenderMaterialPropertyValue::Bool { value }) => {
            write_u32(bytes, offset, u32::from(*value));
            true
        }
        (MaterialPropertyKind::Float, RenderMaterialPropertyValue::Float { value }) => {
            write_f32(bytes, offset, *value);
            true
        }
        (MaterialPropertyKind::Int, RenderMaterialPropertyValue::Int { value }) => {
            write_i32(bytes, offset, *value);
            true
        }
        (MaterialPropertyKind::UInt, RenderMaterialPropertyValue::UInt { value }) => {
            write_u32(bytes, offset, *value);
            true
        }
        (MaterialPropertyKind::Vec2, RenderMaterialPropertyValue::Vec2 { value }) => {
            write_f32_array(bytes, offset, value);
            true
        }
        (MaterialPropertyKind::Vec3, RenderMaterialPropertyValue::Vec3 { value }) => {
            write_f32_array(bytes, offset, value);
            true
        }
        (
            MaterialPropertyKind::Vec4 | MaterialPropertyKind::Color,
            RenderMaterialPropertyValue::Vec4 { value },
        ) => {
            write_f32_array(bytes, offset, value);
            true
        }
        _ => false,
    }
}

fn write_f32(bytes: &mut [u8], offset: usize, value: f32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_i32(bytes: &mut [u8], offset: usize, value: i32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_f32_array<const N: usize>(bytes: &mut [u8], offset: usize, values: &[f32; N]) {
    for (index, value) in values.iter().enumerate() {
        write_f32(bytes, offset + index * 4, *value);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderMaterialPropertyUniformSummary {
    pub payload_byte_len: u64,
    pub field_count: usize,
    pub unsupported_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderMaterialPropertyUniformField {
    pub name: String,
    pub kind: String,
    pub offset: u32,
    /// Encoded byte span inside a packed vec4 slot, not a WGSL member stride.
    pub size: u32,
    /// Start granularity inside the slot; slot placement remains layout-owned.
    pub alignment: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderMaterialPropertyUniformUnsupported {
    pub name: String,
    pub reason: RenderMaterialPropertyUniformUnsupportedReason,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderMaterialPropertyUniformUnsupportedReason {
    UnsupportedType,
    UnknownProperty,
    TypeMismatch,
    PayloadOutOfBounds,
}

impl RenderMaterialPropertyUniformUnsupportedReason {
    const fn description(self) -> &'static str {
        match self {
            Self::UnsupportedType => "unsupported property type",
            Self::UnknownProperty => "property is not present in the material uniform layout",
            Self::TypeMismatch => "override value type does not match the material uniform layout",
            Self::PayloadOutOfBounds => "material uniform layout points outside the payload bytes",
        }
    }
}

const MATERIAL_PROPERTY_UNIFORM_ALIGNMENT: usize = 16;

struct EncodedUniformValue {
    alignment: usize,
    bytes: Vec<u8>,
}

impl EncodedUniformValue {
    fn from_value(value: &RenderMaterialPropertyValue) -> Option<Self> {
        match value {
            RenderMaterialPropertyValue::Bool { value } => {
                Some(Self::scalar_u32(u32::from(*value)))
            }
            RenderMaterialPropertyValue::Float { value } => Some(Self::scalar_f32(*value)),
            RenderMaterialPropertyValue::Int { value } => Some(Self::scalar_i32(*value)),
            RenderMaterialPropertyValue::UInt { value } => Some(Self::scalar_u32(*value)),
            RenderMaterialPropertyValue::String { .. } => None,
            RenderMaterialPropertyValue::Vec2 { value } => Some(Self::float_array(value, 8)),
            RenderMaterialPropertyValue::Vec3 { value } => Some(Self::float_array(value, 16)),
            RenderMaterialPropertyValue::Vec4 { value } => Some(Self::float_array(value, 16)),
        }
    }

    fn scalar_f32(value: f32) -> Self {
        Self {
            alignment: 4,
            bytes: value.to_le_bytes().to_vec(),
        }
    }

    fn scalar_i32(value: i32) -> Self {
        Self {
            alignment: 4,
            bytes: value.to_le_bytes().to_vec(),
        }
    }

    fn scalar_u32(value: u32) -> Self {
        Self {
            alignment: 4,
            bytes: value.to_le_bytes().to_vec(),
        }
    }

    fn float_array<const N: usize>(values: &[f32; N], alignment: usize) -> Self {
        let mut bytes = Vec::with_capacity(N * std::mem::size_of::<f32>());
        for value in values {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        Self { alignment, bytes }
    }
}

impl RenderMaterialPropertyValue {
    fn kind_name(&self) -> &'static str {
        match self {
            Self::Bool { .. } => "bool",
            Self::Float { .. } => "float",
            Self::Int { .. } => "int",
            Self::UInt { .. } => "uint",
            Self::String { .. } => "string",
            Self::Vec2 { .. } => "vec2",
            Self::Vec3 { .. } => "vec3",
            Self::Vec4 { .. } => "vec4",
        }
    }
}

fn align_to(value: usize, alignment: usize) -> usize {
    debug_assert!(alignment.is_power_of_two());
    (value + alignment - 1) & !(alignment - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn material_property_uniform_payload_aligns_and_encodes_numeric_values() {
        let mut values = BTreeMap::new();
        values.insert(
            "enabled".to_string(),
            RenderMaterialPropertyValue::Bool { value: true },
        );
        values.insert(
            "gain".to_string(),
            RenderMaterialPropertyValue::Float { value: 2.5 },
        );
        values.insert(
            "rim".to_string(),
            RenderMaterialPropertyValue::Vec3 {
                value: [0.25, 0.5, 0.75],
            },
        );
        values.insert(
            "tint".to_string(),
            RenderMaterialPropertyValue::Vec4 {
                value: [1.0, 0.5, 0.25, 1.0],
            },
        );

        let payload = RenderMaterialPropertyUniformPayload::from_values(&values);

        assert_eq!(payload.layout[0].name, "enabled");
        assert_eq!(payload.layout[0].offset, 0);
        assert_eq!(u32_at(&payload.bytes, 0), 1);
        assert_eq!(payload.layout[1].name, "gain");
        assert_eq!(payload.layout[1].offset, 4);
        assert_eq!(f32_at(&payload.bytes, 4), 2.5);
        assert_eq!(payload.layout[2].name, "rim");
        assert_eq!(payload.layout[2].offset, 16);
        assert_eq!(f32_at(&payload.bytes, 16), 0.25);
        assert_eq!(f32_at(&payload.bytes, 20), 0.5);
        assert_eq!(f32_at(&payload.bytes, 24), 0.75);
        assert_eq!(payload.layout[3].name, "tint");
        assert_eq!(payload.layout[3].offset, 32);
        assert_eq!(f32_at(&payload.bytes, 32), 1.0);
        assert_eq!(f32_at(&payload.bytes, 36), 0.5);
        assert_eq!(f32_at(&payload.bytes, 40), 0.25);
        assert_eq!(f32_at(&payload.bytes, 44), 1.0);
        assert_eq!(payload.bytes.len(), 48);
        assert!(payload.unsupported.is_empty());
    }

    #[test]
    fn material_property_uniform_payload_records_unsupported_strings() {
        let mut values = BTreeMap::new();
        values.insert(
            "debug_label".to_string(),
            RenderMaterialPropertyValue::String {
                value: "paint".to_string(),
            },
        );
        values.insert(
            "gain".to_string(),
            RenderMaterialPropertyValue::Float { value: 1.0 },
        );

        let payload = RenderMaterialPropertyUniformPayload::from_values(&values);

        assert_eq!(payload.layout.len(), 1);
        assert_eq!(payload.layout[0].name, "gain");
        assert_eq!(payload.unsupported.len(), 1);
        assert_eq!(payload.unsupported[0].name, "debug_label");
        assert_eq!(
            payload.unsupported[0].reason,
            RenderMaterialPropertyUniformUnsupportedReason::UnsupportedType
        );
    }

    #[test]
    fn material_property_uniform_payload_reports_unsupported_diagnostics() {
        let mut values = BTreeMap::new();
        values.insert(
            "debug_label".to_string(),
            RenderMaterialPropertyValue::String {
                value: "paint".to_string(),
            },
        );

        let diagnostics =
            RenderMaterialPropertyUniformPayload::from_values(&values).unsupported_diagnostics();

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].source,
            RenderMaterialDiagnosticSource::MaterialUniform
        );
        assert_eq!(diagnostics[0].path, "uniform.debug_label");
        assert_eq!(
            diagnostics[0].diagnostic,
            "material property debug_label cannot be encoded into the renderer uniform payload: unsupported property type"
        );
    }

    #[test]
    fn material_property_uniform_payload_applies_runtime_override_block() {
        let mut values = BTreeMap::new();
        values.insert(
            "enabled".to_string(),
            RenderMaterialPropertyValue::Bool { value: true },
        );
        values.insert(
            "gain".to_string(),
            RenderMaterialPropertyValue::Float { value: 1.0 },
        );
        values.insert(
            "tint".to_string(),
            RenderMaterialPropertyValue::Vec4 {
                value: [1.0, 1.0, 1.0, 1.0],
            },
        );
        let payload = RenderMaterialPropertyUniformPayload::from_values(&values);
        let overrides = MaterialPropertyOverrideBlock::new()
            .with_value("gain", RenderMaterialPropertyValue::Float { value: 2.5 })
            .with_value(
                "tint",
                RenderMaterialPropertyValue::Vec4 {
                    value: [0.25, 0.5, 0.75, 1.0],
                },
            );

        let overridden = payload.with_override_block(&overrides);

        assert_eq!(f32_at(&overridden.bytes, 4), 2.5);
        assert_eq!(f32_at(&overridden.bytes, 16), 0.25);
        assert_eq!(f32_at(&overridden.bytes, 20), 0.5);
        assert_eq!(f32_at(&overridden.bytes, 24), 0.75);
        assert_eq!(f32_at(&overridden.bytes, 28), 1.0);
        assert!(overridden.unsupported.is_empty());
    }

    #[test]
    fn material_property_uniform_payload_reports_invalid_override_block_entries() {
        let mut values = BTreeMap::new();
        values.insert(
            "gain".to_string(),
            RenderMaterialPropertyValue::Float { value: 1.0 },
        );
        let payload = RenderMaterialPropertyUniformPayload::from_values(&values);
        let overrides = MaterialPropertyOverrideBlock::new()
            .with_value("gain", RenderMaterialPropertyValue::Bool { value: true })
            .with_value("missing", RenderMaterialPropertyValue::Float { value: 1.0 });

        let overridden = payload.with_override_block(&overrides);

        assert_eq!(f32_at(&overridden.bytes, 0), 1.0);
        assert_eq!(overridden.unsupported.len(), 2);
        assert_eq!(overridden.unsupported[0].name, "gain");
        assert_eq!(
            overridden.unsupported[0].reason,
            RenderMaterialPropertyUniformUnsupportedReason::TypeMismatch
        );
        assert_eq!(overridden.unsupported[1].name, "missing");
        assert_eq!(
            overridden.unsupported[1].reason,
            RenderMaterialPropertyUniformUnsupportedReason::UnknownProperty
        );
    }

    fn f32_at(bytes: &[u8], offset: usize) -> f32 {
        f32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
    }

    fn u32_at(bytes: &[u8], offset: usize) -> u32 {
        u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
    }
}
