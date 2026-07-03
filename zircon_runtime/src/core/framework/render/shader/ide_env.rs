use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::asset::AssetUri;

use super::variant_key::ShaderPassType;
use super::ShaderAssetKind;

pub const SHADER_IDE_ENV_SCHEMA_VERSION: u32 = 1;
pub const SHADER_IDE_ENV_CACHE_DIR: &str = ".zircon-cache/shader_ide/v1";
pub const SHADER_IDE_MODULE_MAP_FILE: &str = "module_map.json";
pub const SHADER_IDE_PREVIEW_DEFAULT_VARIANT: &str = "default";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShaderIdePreviewVariant {
    pub name: String,
    pub pass_type: ShaderPassType,
    pub material_option_bits: u32,
}

impl ShaderIdePreviewVariant {
    pub fn default_forward() -> Self {
        Self::new(ShaderPassType::Forward, 0)
    }

    pub fn new(pass_type: ShaderPassType, material_option_bits: u32) -> Self {
        Self {
            name: shader_ide_preview_variant_name(pass_type, material_option_bits),
            pass_type,
            material_option_bits,
        }
    }
}

pub fn shader_ide_preview_variant_name(
    pass_type: ShaderPassType,
    material_option_bits: u32,
) -> String {
    if pass_type == ShaderPassType::Forward && material_option_bits == 0 {
        return SHADER_IDE_PREVIEW_DEFAULT_VARIANT.to_string();
    }
    if material_option_bits == 0 {
        return pass_type.token().to_string();
    }
    format!("{}_options_0x{material_option_bits:08x}", pass_type.token())
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderIdeModuleMap {
    pub schema_version: u32,
    pub project_name: String,
    pub entries: Vec<ShaderIdeModuleMapEntry>,
}

impl ShaderIdeModuleMap {
    pub fn new(project_name: impl Into<String>, entries: Vec<ShaderIdeModuleMapEntry>) -> Self {
        Self {
            schema_version: SHADER_IDE_ENV_SCHEMA_VERSION,
            project_name: project_name.into(),
            entries,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderIdeModuleMapEntry {
    pub import_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_uri: Option<AssetUri>,
    pub kind: ShaderAssetKind,
    pub stub_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_uri: Option<AssetUri>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_files: Vec<String>,
    pub content_hash: String,
    #[serde(default)]
    pub generated: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderIdePreviewMap {
    pub schema_version: u32,
    pub shader_uri: AssetUri,
    pub variant: String,
    pub wgsl_path: String,
    pub segments: Vec<ShaderIdePreviewSegment>,
}

impl ShaderIdePreviewMap {
    pub fn new(
        shader_uri: AssetUri,
        variant: impl Into<String>,
        wgsl_path: impl Into<String>,
        segments: Vec<ShaderIdePreviewSegment>,
    ) -> Self {
        Self {
            schema_version: SHADER_IDE_ENV_SCHEMA_VERSION,
            shader_uri,
            variant: variant.into(),
            wgsl_path: wgsl_path.into(),
            segments,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderIdePreviewSegment {
    pub module_id: String,
    pub kind: String,
    pub assembled_start_line: u32,
    pub assembled_line_count: u32,
    pub source_line_offset: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShaderIdeModuleSource {
    pub import_path: String,
    pub kind: ShaderAssetKind,
    pub source: String,
}

impl ShaderIdeModuleSource {
    pub fn new(
        import_path: impl Into<String>,
        kind: ShaderAssetKind,
        source: impl Into<String>,
    ) -> Self {
        Self {
            import_path: import_path.into(),
            kind,
            source: source.into(),
        }
    }
}

pub fn shader_ide_module_stub_relative_path(import_path: &str) -> PathBuf {
    let mut path = PathBuf::from("modules");
    if import_path.contains("::") {
        let segments = import_path
            .split("::")
            .map(sanitize_shader_ide_path_segment)
            .collect::<Vec<_>>();
        if let Some((file_name, directories)) = segments.split_last() {
            for directory in directories {
                path.push(directory);
            }
            path.push(format!("{file_name}.wgsl"));
        }
    } else {
        path.push("builtin");
        let file_name = sanitize_shader_ide_builtin_file_name(import_path);
        path.push(file_name);
    }
    path
}

pub fn shader_ide_generated_material_stub_relative_path(source_uri: &AssetUri) -> PathBuf {
    let file_name = format!(
        "{}.material.wgsl",
        sanitize_shader_ide_path_segment(&source_uri.to_string())
    );
    PathBuf::from("generated").join(file_name)
}

pub fn shader_ide_preview_relative_path(source_uri: &AssetUri, variant: &str) -> PathBuf {
    let file_name = format!(
        "{}.{}.wgsl",
        sanitize_shader_ide_path_segment(&source_uri.to_string()),
        sanitize_shader_ide_path_segment(variant)
    );
    PathBuf::from("preview").join(file_name)
}

pub fn shader_ide_preview_segments_relative_path(source_uri: &AssetUri, variant: &str) -> PathBuf {
    let file_name = format!(
        "{}.{}.segments.json",
        sanitize_shader_ide_path_segment(&source_uri.to_string()),
        sanitize_shader_ide_path_segment(variant)
    );
    PathBuf::from("preview").join(file_name)
}

pub fn shader_ide_relative_path_string(path: &std::path::Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn sanitize_shader_ide_builtin_file_name(value: &str) -> String {
    let name = value
        .rsplit(['/', '\\'])
        .next()
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("module");
    if name.ends_with(".wgsl") {
        sanitize_shader_ide_file_stem_with_extension(name)
    } else {
        format!("{}.wgsl", sanitize_shader_ide_path_segment(name))
    }
}

fn sanitize_shader_ide_file_stem_with_extension(name: &str) -> String {
    let stem = name.strip_suffix(".wgsl").unwrap_or(name);
    format!("{}.wgsl", sanitize_shader_ide_path_segment(stem))
}

fn sanitize_shader_ide_path_segment(value: &str) -> String {
    let mut output = String::new();
    let mut previous_underscore = false;
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            output.push(character.to_ascii_lowercase());
            previous_underscore = false;
        } else if !previous_underscore && !output.is_empty() {
            output.push('_');
            previous_underscore = true;
        }
    }
    while output.ends_with('_') {
        output.pop();
    }
    if output.is_empty() {
        output.push_str("module");
    }
    if output
        .as_bytes()
        .first()
        .is_some_and(|first| first.is_ascii_digit())
    {
        output.insert(0, '_');
    }
    output
}

#[cfg(test)]
mod tests {
    use crate::asset::AssetUri;

    use super::*;

    #[test]
    fn shader_ide_module_stub_path_maps_logical_modules_to_files() {
        assert_eq!(
            shader_ide_relative_path_string(&shader_ide_module_stub_relative_path(
                "myproj::cloth::common"
            )),
            "modules/myproj/cloth/common.wgsl"
        );
        assert_eq!(
            shader_ide_relative_path_string(&shader_ide_module_stub_relative_path(
                "zr_surface_types.wgsl"
            )),
            "modules/builtin/zr_surface_types.wgsl"
        );
    }

    #[test]
    fn shader_ide_generated_material_stub_path_is_scoped_by_source_uri() {
        let uri = AssetUri::parse("res://shaders/hero_cloth").unwrap();

        assert_eq!(
            shader_ide_relative_path_string(&shader_ide_generated_material_stub_relative_path(
                &uri
            )),
            "generated/res_shaders_hero_cloth.material.wgsl"
        );
    }

    #[test]
    fn shader_ide_preview_paths_are_scoped_by_source_uri_and_variant() {
        let uri = AssetUri::parse("res://shaders/hero_cloth").unwrap();
        let variant = ShaderIdePreviewVariant::new(ShaderPassType::GBuffer, 1);

        assert_eq!(
            shader_ide_relative_path_string(&shader_ide_preview_relative_path(&uri, "default")),
            "preview/res_shaders_hero_cloth.default.wgsl"
        );
        assert_eq!(
            shader_ide_relative_path_string(&shader_ide_preview_segments_relative_path(
                &uri, "default"
            )),
            "preview/res_shaders_hero_cloth.default.segments.json"
        );
        assert_eq!(variant.name, "gbuffer_options_0x00000001");
        assert_eq!(
            shader_ide_relative_path_string(&shader_ide_preview_relative_path(&uri, &variant.name)),
            "preview/res_shaders_hero_cloth.gbuffer_options_0x00000001.wgsl"
        );
    }
}
