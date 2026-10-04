//! 材质属性布局由 shader schema 唯一生成，并同时供导入产物、材质覆写与 GPU 上传使用；布局算法版本参与缓存与运行时一致性约束。

use serde::{Deserialize, Serialize};

use crate::core::framework::render::{
    MaterialOptionKind, MaterialOptionRef, MaterialOptionTable, MaterialPropertyKind,
    MaterialPropertyLayout, MaterialPropertySlotRef, MaterialTextureBindingRef,
    PropertyScalarClass, RenderMaterialTextureDimension,
};

use super::{ShaderMaterialPropertyAsset, ShaderOptionAsset, ShaderTextureSlotAsset};

const MATERIAL_PROPERTY_LAYOUT_ALGORITHM_VERSION: u8 = 2;
const MATERIAL_PROPERTY_EMPTY_UNIFORM_SIZE: u32 = 16;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ShaderGeneratedMaterialArtifact {
    pub property_layout: MaterialPropertyLayout,
    pub option_table: MaterialOptionTable,
    pub wgsl_source: String,
}

pub fn generate_material_artifact(
    properties: &[ShaderMaterialPropertyAsset],
    options: &[ShaderOptionAsset],
    texture_slots: &[ShaderTextureSlotAsset],
) -> ShaderGeneratedMaterialArtifact {
    let mut layout_properties = properties
        .iter()
        .map(MaterialPropertyInput::from)
        .collect::<Vec<_>>();
    for slot in texture_slots {
        if slot.st {
            layout_properties.push(MaterialPropertyInput {
                name: format!("{}_st", slot.name),
                kind: MaterialPropertyKind::Vec4,
            });
        }
    }

    let mut layout = pack_material_properties(&layout_properties);
    layout.texture_bindings = material_texture_bindings(texture_slots);
    layout.layout_hash = material_layout_hash(&layout, &layout_properties, options);
    let option_table = material_option_table(options);
    let wgsl_source = generate_material_wgsl(&layout);

    ShaderGeneratedMaterialArtifact {
        property_layout: layout,
        option_table,
        wgsl_source,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MaterialPropertyInput {
    name: String,
    kind: MaterialPropertyKind,
}

impl From<&ShaderMaterialPropertyAsset> for MaterialPropertyInput {
    fn from(property: &ShaderMaterialPropertyAsset) -> Self {
        Self {
            name: property.name.clone(),
            kind: property.kind,
        }
    }
}

fn pack_material_properties(properties: &[MaterialPropertyInput]) -> MaterialPropertyLayout {
    let mut f32_slots = Vec::<[bool; 4]>::new();
    let mut u32_slots = Vec::<[bool; 4]>::new();
    let mut refs = Vec::with_capacity(properties.len());

    for property in properties {
        let scalar_class = property.kind.scalar_class();
        let slots = match scalar_class {
            PropertyScalarClass::F32 => &mut f32_slots,
            PropertyScalarClass::U32 => &mut u32_slots,
        };
        let (slot, component) = assign_property_slot(slots, property.kind.component_count());
        refs.push(MaterialPropertySlotRef {
            name: property.name.clone(),
            kind: property.kind,
            scalar_class,
            slot,
            component,
            component_count: property.kind.component_count(),
        });
    }

    let total_slots = f32_slots.len() + u32_slots.len();
    MaterialPropertyLayout {
        properties: refs,
        f32_slot_count: f32_slots.len() as u16,
        u32_slot_count: u32_slots.len() as u16,
        packed_size: (total_slots as u32 * 16).max(MATERIAL_PROPERTY_EMPTY_UNIFORM_SIZE),
        texture_bindings: Vec::new(),
        layout_hash: 0,
    }
}

fn assign_property_slot(slots: &mut Vec<[bool; 4]>, component_count: u8) -> (u16, u8) {
    match component_count {
        4 => {
            let slot = slots.len() as u16;
            slots.push([true; 4]);
            (slot, 0)
        }
        3 => {
            let slot = slots.len() as u16;
            slots.push([true, true, true, false]);
            (slot, 0)
        }
        2 => assign_vec2_slot(slots),
        1 => assign_scalar_slot(slots),
        _ => unreachable!("material property component counts are closed"),
    }
}

fn assign_vec2_slot(slots: &mut Vec<[bool; 4]>) -> (u16, u8) {
    for (slot_index, slot) in slots.iter_mut().enumerate() {
        if !slot[0] && !slot[1] {
            slot[0] = true;
            slot[1] = true;
            return (slot_index as u16, 0);
        }
        if !slot[2] && !slot[3] {
            slot[2] = true;
            slot[3] = true;
            return (slot_index as u16, 2);
        }
    }
    let slot = slots.len() as u16;
    slots.push([true, true, false, false]);
    (slot, 0)
}

fn assign_scalar_slot(slots: &mut Vec<[bool; 4]>) -> (u16, u8) {
    for (slot_index, slot) in slots.iter_mut().enumerate() {
        if let Some(component) = slot.iter().position(|occupied| !occupied) {
            slot[component] = true;
            return (slot_index as u16, component as u8);
        }
    }
    let slot = slots.len() as u16;
    slots.push([true, false, false, false]);
    (slot, 0)
}

fn material_texture_bindings(
    texture_slots: &[ShaderTextureSlotAsset],
) -> Vec<MaterialTextureBindingRef> {
    texture_slots
        .iter()
        .enumerate()
        .map(|(index, slot)| {
            let texture_binding = 1 + (index as u16 * 2);
            MaterialTextureBindingRef {
                name: slot.name.clone(),
                kind: slot.kind.clone(),
                texture_binding,
                sampler_binding: texture_binding + 1,
                option: slot.option.clone(),
                has_st_transform: slot.st,
            }
        })
        .collect()
}

fn material_option_table(options: &[ShaderOptionAsset]) -> MaterialOptionTable {
    let mut bit_offset = 0_u8;
    let mut refs = Vec::with_capacity(options.len());
    for option in options {
        let (kind, enum_values) = option_kind_and_values(option);
        let bit_width = match kind {
            MaterialOptionKind::Bool => 1,
            MaterialOptionKind::Enum => enum_bit_width(enum_values.len()),
        };
        let default_bits = option_default_bits(option, kind, &enum_values);
        refs.push(MaterialOptionRef {
            name: option.name.clone(),
            kind,
            bit_offset,
            bit_width,
            enum_values,
            default_bits,
        });
        bit_offset = bit_offset.saturating_add(bit_width);
    }
    MaterialOptionTable {
        options: refs,
        total_bits: bit_offset,
    }
}

fn option_kind_and_values(option: &ShaderOptionAsset) -> (MaterialOptionKind, Vec<String>) {
    let kind = material_option_kind_from_token(&option.kind);
    match kind {
        MaterialOptionKind::Bool => (kind, Vec::new()),
        MaterialOptionKind::Enum => (kind, option_enum_values(option)),
    }
}

fn material_option_kind_from_token(kind: &str) -> MaterialOptionKind {
    if kind.trim().eq_ignore_ascii_case("enum") {
        MaterialOptionKind::Enum
    } else {
        MaterialOptionKind::Bool
    }
}

fn option_enum_values(option: &ShaderOptionAsset) -> Vec<String> {
    option
        .editor
        .get("values")
        .or_else(|| option.editor.get("enum_values"))
        .map(|values| {
            values
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn enum_bit_width(value_count: usize) -> u8 {
    let variants = value_count.max(1) - 1;
    let bits = usize::BITS - variants.leading_zeros();
    bits.max(1) as u8
}

fn option_default_bits(
    option: &ShaderOptionAsset,
    kind: MaterialOptionKind,
    enum_values: &[String],
) -> u32 {
    let Some(default) = option.default.as_ref() else {
        return 0;
    };
    match kind {
        MaterialOptionKind::Bool => default.as_bool().map(u32::from).unwrap_or(0),
        MaterialOptionKind::Enum => default
            .as_str()
            .and_then(|value| enum_values.iter().position(|candidate| candidate == value))
            .unwrap_or(0) as u32,
    }
}

fn material_layout_hash(
    layout: &MaterialPropertyLayout,
    properties: &[MaterialPropertyInput],
    options: &[ShaderOptionAsset],
) -> u64 {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"zircon.material_property_layout");
    hasher.update(&[MATERIAL_PROPERTY_LAYOUT_ALGORITHM_VERSION]);
    for property in properties {
        hasher.update(property.name.as_bytes());
        hasher.update(&[0]);
        hasher.update(property.kind.token().as_bytes());
        hasher.update(&[0]);
    }
    for binding in &layout.texture_bindings {
        hasher.update(binding.name.as_bytes());
        hasher.update(&binding.texture_binding.to_le_bytes());
        hasher.update(&binding.sampler_binding.to_le_bytes());
        hasher.update(binding.kind.as_bytes());
        hasher.update(&[0]);
        hasher.update(&[u8::from(binding.has_st_transform)]);
    }
    for option in options {
        hasher.update(option.name.as_bytes());
        hasher.update(&[0]);
        hasher.update(option.kind.as_bytes());
        hasher.update(&[0]);
    }
    let hash = hasher.finalize();
    u64::from_le_bytes(
        hash.as_bytes()[..8]
            .try_into()
            .expect("hash prefix is 8 bytes"),
    )
}

fn generate_material_wgsl(layout: &MaterialPropertyLayout) -> String {
    let mut source = String::new();
    source.push_str(&format!(
        "// -- zr generated: material properties (layout_hash = 0x{:016x}) --\n",
        layout.layout_hash
    ));
    source.push_str("struct ZrMaterialProperties {\n");
    if layout.f32_slot_count == 0 && layout.u32_slot_count == 0 {
        source.push_str("    _pad: vec4<u32>,\n");
    } else {
        for slot in 0..layout.f32_slot_count {
            source.push_str(&format!("    f{slot}: vec4<f32>,\n"));
        }
        for slot in 0..layout.u32_slot_count {
            source.push_str(&format!("    u{slot}: vec4<u32>,\n"));
        }
    }
    source.push_str("};\n");
    source.push_str("@group(2) @binding(0) var<uniform> zr_material: ZrMaterialProperties;\n");
    for binding in &layout.texture_bindings {
        let name = material_identifier(&binding.name);
        source.push_str(&format!(
            "@group(2) @binding({}) var zr_tex_{name}: {};\n",
            binding.texture_binding,
            texture_binding_type(&binding.kind)
        ));
        source.push_str(&format!(
            "@group(2) @binding({}) var zr_smp_{name}: sampler;\n",
            binding.sampler_binding
        ));
    }
    source.push('\n');
    for property in &layout.properties {
        source.push_str(&property_accessor_wgsl(property));
    }
    for binding in &layout.texture_bindings {
        let name = material_identifier(&binding.name);
        source.push_str(&texture_sample_helper_wgsl(
            &name,
            RenderMaterialTextureDimension::from_shader_kind(&binding.kind),
        ));
        if binding.has_st_transform {
            source.push_str(&format!(
                "fn zr_uv_{name}(uv: vec2<f32>) -> vec2<f32> {{\n    let st = zr_mat_{name}_st();\n    return uv * st.xy + st.zw;\n}}\n"
            ));
        }
    }
    source
}

fn property_accessor_wgsl(property: &MaterialPropertySlotRef) -> String {
    let name = material_identifier(&property.name);
    let field = match property.scalar_class {
        PropertyScalarClass::F32 => format!("f{}", property.slot),
        PropertyScalarClass::U32 => format!("u{}", property.slot),
    };
    let swizzle = component_swizzle(property.component, property.component_count);
    let value = format!("zr_material.{field}.{swizzle}");
    let body = match property.kind {
        MaterialPropertyKind::Float => format!("return {value};"),
        MaterialPropertyKind::Vec2 => format!("return {value};"),
        MaterialPropertyKind::Vec3 => format!("return {value};"),
        MaterialPropertyKind::Vec4 | MaterialPropertyKind::Color => {
            format!("return {value};")
        }
        MaterialPropertyKind::Int => format!("return i32({value});"),
        MaterialPropertyKind::UInt => format!("return {value};"),
        MaterialPropertyKind::Bool => format!("return {value} != 0u;"),
    };
    format!(
        "fn zr_mat_{name}() -> {} {{\n    {body}\n}}\n",
        property_return_type(property.kind)
    )
}

fn property_return_type(kind: MaterialPropertyKind) -> &'static str {
    match kind {
        MaterialPropertyKind::Float => "f32",
        MaterialPropertyKind::Vec2 => "vec2<f32>",
        MaterialPropertyKind::Vec3 => "vec3<f32>",
        MaterialPropertyKind::Vec4 | MaterialPropertyKind::Color => "vec4<f32>",
        MaterialPropertyKind::Int => "i32",
        MaterialPropertyKind::UInt => "u32",
        MaterialPropertyKind::Bool => "bool",
    }
}

fn component_swizzle(component: u8, count: u8) -> String {
    const COMPONENTS: &[u8; 4] = b"xyzw";
    (component..component + count)
        .map(|index| COMPONENTS[index as usize] as char)
        .collect()
}

fn texture_binding_type(kind: &str) -> &'static str {
    RenderMaterialTextureDimension::from_shader_kind(kind).wgsl_sampled_texture_type()
}

fn texture_sample_helper_wgsl(name: &str, dimension: RenderMaterialTextureDimension) -> String {
    let (parameters, coordinates) = match dimension {
        RenderMaterialTextureDimension::D1 => ("coord: f32", "coord"),
        RenderMaterialTextureDimension::D2 => ("uv: vec2<f32>", "uv"),
        RenderMaterialTextureDimension::D2Array => ("uv: vec2<f32>, layer: i32", "uv, layer"),
        RenderMaterialTextureDimension::Cube => ("direction: vec3<f32>", "direction"),
        RenderMaterialTextureDimension::CubeArray => {
            ("direction: vec3<f32>, layer: i32", "direction, layer")
        }
        RenderMaterialTextureDimension::D3 => ("position: vec3<f32>", "position"),
    };
    format!(
        "fn zr_sample_{name}({parameters}) -> vec4<f32> {{\n    return textureSample(zr_tex_{name}, zr_smp_{name}, {coordinates});\n}}\n"
    )
}

fn material_identifier(name: &str) -> String {
    let mut identifier = String::with_capacity(name.len());
    for character in name.chars() {
        if character.is_ascii_alphanumeric() || character == '_' {
            identifier.push(character.to_ascii_lowercase());
        } else {
            identifier.push('_');
        }
    }
    if identifier.is_empty() {
        identifier.push_str("unnamed");
    }
    if identifier
        .as_bytes()
        .first()
        .is_some_and(|first| first.is_ascii_digit())
    {
        identifier.insert(0, '_');
    }
    identifier
}

#[cfg(test)]
#[path = "tests/property_layout.rs"]
mod tests;
