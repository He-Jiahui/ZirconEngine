use crate::core::framework::render::{ShaderFeatureBits, ShaderPassType};

use super::{
    MaterialShaderTemplateRequest, assemble_material_shader_template, material_template_request,
    standard_material_surface_source_for_features, static_mesh_descriptor,
};

fn environment_pbr_composition_source(source: &str) -> String {
    wgsl_function_source(source, "fn zr_environment_pbr_components_from_reflection(")
}

fn wgsl_line_break_len_at(bytes: &[u8], index: usize) -> Option<usize> {
    match bytes.get(index) {
        Some(b'\n' | b'\x0B' | b'\x0C') => Some(1),
        Some(b'\r') => Some(if bytes.get(index + 1) == Some(&b'\n') {
            2
        } else {
            1
        }),
        Some(&0xC2) if bytes.get(index + 1) == Some(&0x85) => Some(2),
        Some(&0xE2)
            if bytes.get(index + 1) == Some(&0x80)
                && matches!(bytes.get(index + 2), Some(0xA8 | 0xA9)) =>
        {
            Some(3)
        }
        _ => None,
    }
}

pub(super) fn wgsl_without_comments(source: &str) -> String {
    let mut code = String::with_capacity(source.len());
    let mut index = 0;
    while index < source.len() {
        let remaining = &source[index..];
        if remaining.starts_with("//") {
            index += 2;
            while index < source.len() && wgsl_line_break_len_at(source.as_bytes(), index).is_none()
            {
                index += source[index..]
                    .chars()
                    .next()
                    .expect("source index must remain on a UTF-8 boundary")
                    .len_utf8();
            }
            code.push(' ');
            continue;
        }
        if remaining.starts_with("/*") {
            let mut depth = 1usize;
            index += 2;
            while depth > 0 {
                let comment = source
                    .get(index..)
                    .expect("WGSL block comment must be terminated");
                if comment.starts_with("/*") {
                    depth += 1;
                    index += 2;
                } else if comment.starts_with("*/") {
                    depth -= 1;
                    index += 2;
                } else {
                    index += comment
                        .chars()
                        .next()
                        .expect("WGSL block comment must be terminated")
                        .len_utf8();
                }
            }
            code.push(' ');
            continue;
        }
        let character = remaining
            .chars()
            .next()
            .expect("source index must remain on a UTF-8 boundary");
        code.push(character);
        index += character.len_utf8();
    }
    code
}

pub(super) fn wgsl_function_source(source: &str, signature: &str) -> String {
    let code = wgsl_without_comments(source);
    let start = code
        .find(signature)
        .unwrap_or_else(|| panic!("missing WGSL function `{signature}`"));
    let body_start = code[start..]
        .find('{')
        .map(|offset| start + offset)
        .unwrap_or_else(|| panic!("missing opening brace for `{signature}`"));
    let mut depth = 0usize;
    for (offset, byte) in code.as_bytes()[body_start..].iter().enumerate() {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth = depth
                    .checked_sub(1)
                    .unwrap_or_else(|| panic!("unbalanced braces for `{signature}`"));
                if depth == 0 {
                    return code[start..=body_start + offset].to_string();
                }
            }
            _ => {}
        }
    }
    panic!("missing closing brace for `{signature}`");
}

#[path = "environment/provider_guards.rs"]
mod provider_guards;
#[path = "environment/sampling.rs"]
mod sampling;
