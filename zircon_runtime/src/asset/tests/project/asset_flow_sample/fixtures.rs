use super::assertions::uri;
use super::*;

pub(super) fn write_minimal_textured_gltf(path: PathBuf) {
    write_text(
        path,
        r#"
{
  "asset": { "version": "2.0" },
  "buffers": [
    {
      "uri": "data:application/octet-stream;base64,AAAAAAAAAAAAAAAAAACAPwAAAAAAAAAAAAAAAAAAgD8AAAAAAAABAAIAAAAAAAAAAAAAAM3MTD4AAAAAAAAAAM3MTD4AAAAAAAAAAM3MTD4=",
      "byteLength": 80
    }
  ],
  "bufferViews": [
    { "buffer": 0, "byteOffset": 0, "byteLength": 36, "target": 34962 },
    { "buffer": 0, "byteOffset": 36, "byteLength": 6, "target": 34963 },
    { "buffer": 0, "byteOffset": 44, "byteLength": 36, "target": 34962 }
  ],
  "accessors": [
    {
      "bufferView": 0,
      "componentType": 5126,
      "count": 3,
      "type": "VEC3",
      "min": [0.0, 0.0, 0.0],
      "max": [1.0, 1.0, 0.0]
    },
    {
      "bufferView": 1,
      "componentType": 5123,
      "count": 3,
      "type": "SCALAR"
    },
    {
      "bufferView": 2,
      "componentType": 5126,
      "count": 3,
      "type": "VEC3"
    }
  ],
  "images": [
    {
      "uri": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGP4////fwAJ+wP9KobjigAAAABJRU5ErkJggg=="
    }
  ],
  "textures": [
    { "source": 0 }
  ],
  "materials": [
    {
      "name": "HeroGLTFMaterial",
      "pbrMetallicRoughness": {
        "baseColorTexture": { "index": 0 },
        "baseColorFactor": [0.8, 0.9, 1.0, 1.0]
      }
    }
  ],
  "meshes": [
    {
      "name": "HeroTriangle",
      "weights": [0.5],
      "primitives": [
        {
          "attributes": { "POSITION": 0 },
          "indices": 1,
          "material": 0,
          "targets": [
            { "POSITION": 2 }
          ]
        }
      ]
    }
  ],
  "nodes": [
    { "name": "Hero", "mesh": 0 }
  ],
  "scenes": [
    { "name": "MainScene", "nodes": [0] }
  ],
  "scene": 0
}
"#,
    );
}

pub(super) fn write_sample_shader_package(paths: &ProjectPaths) {
    let shader_uri = uri("res://shaders/lit_sample");
    let mut meta = AssetMetaDocument::new(
        AssetUuid::from_stable_label("minimal-asset-flow/lit-sample-shader"),
        shader_uri,
        AssetKind::Shader,
    );
    meta.unit = AssetSourceUnit::Compound;
    meta.save(paths.assets_root().join("shaders").join("lit_sample.zmeta"))
        .unwrap();

    write_text(
        paths
            .assets_root()
            .join("shaders")
            .join("lit_sample")
            .join("lit.zshader"),
        r#"
version = 1
name = "Lit Sample"
wgsl_files = ["lit.wgsl"]

[[entry_points]]
name = "vs_main"
stage = "vertex"

[[entry_points]]
name = "fs_main"
stage = "fragment"

[[properties]]
name = "base_color"
kind = "vec4"
required = true
default = [1.0, 1.0, 1.0, 1.0]

[[properties]]
name = "roughness"
kind = "float"
default = 1.0

[[texture_slots]]
name = "base_color"
kind = "texture2d"
required = true
default = "white"
sampler = "linear_repeat"
"#,
    );
    write_text(
        paths
            .assets_root()
            .join("shaders")
            .join("lit_sample")
            .join("lit.wgsl"),
        r#"
struct VsOut {
    @builtin(position) position: vec4f,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VsOut {
    var positions = array<vec2f, 3>(
        vec2f(0.0, 0.5),
        vec2f(-0.5, -0.5),
        vec2f(0.5, -0.5),
    );
    var out: VsOut;
    out.position = vec4f(positions[vertex_index], 0.0, 1.0);
    return out;
}

@fragment
fn fs_main() -> @location(0) vec4f {
    let base_color = vec4f(1.0, 0.9, 0.7, 1.0);
    let roughness = 0.65;
    let base_color_texture = base_color;
    return vec4f(base_color_texture.rgb * roughness, base_color_texture.a);
}
"#,
    );
}

pub(super) fn write_default_pbr_shader_package(paths: &ProjectPaths) {
    let shader_uri = uri("res://shaders/default_pbr");
    let mut meta = AssetMetaDocument::new(
        AssetUuid::from_stable_label("minimal-asset-flow/default-pbr-shader"),
        shader_uri,
        AssetKind::Shader,
    );
    meta.unit = AssetSourceUnit::Compound;
    meta.save(
        paths
            .assets_root()
            .join("shaders")
            .join("default_pbr.zmeta"),
    )
    .unwrap();

    write_text(
        paths
            .assets_root()
            .join("shaders")
            .join("default_pbr")
            .join("default_pbr.zshader"),
        r#"
version = 1
name = "Default PBR Sample"
wgsl_files = ["default_pbr.wgsl"]

[[entry_points]]
name = "vs_main"
stage = "vertex"

[[entry_points]]
name = "fs_main"
stage = "fragment"

[[properties]]
name = "base_color"
kind = "vec4"
default = [1.0, 1.0, 1.0, 1.0]

[[texture_slots]]
name = "base_color"
kind = "texture2d"
default = "white"
sampler = "linear_repeat"
"#,
    );
    write_text(
        paths
            .assets_root()
            .join("shaders")
            .join("default_pbr")
            .join("default_pbr.wgsl"),
        r#"
struct VsOut {
    @builtin(position) position: vec4f,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VsOut {
    var positions = array<vec2f, 3>(
        vec2f(0.0, 0.5),
        vec2f(-0.5, -0.5),
        vec2f(0.5, -0.5),
    );
    var out: VsOut;
    out.position = vec4f(positions[vertex_index], 0.0, 1.0);
    return out;
}

@fragment
fn fs_main() -> @location(0) vec4f {
    let base_color = vec4f(1.0, 1.0, 1.0, 1.0);
    let base_color_texture = base_color;
    return base_color_texture;
}
"#,
    );
}

pub(super) fn write_sample_material(paths: &ProjectPaths) {
    let shader = AssetReference::from_locator(uri("res://shaders/lit_sample"));
    let texture = AssetReference::from_locator(uri("res://textures/hero_albedo_bc1.dds"));
    let mut property_values = BTreeMap::new();
    property_values.insert(
        "base_color".to_string(),
        toml::Value::Array(vec![
            toml::Value::Float(1.0),
            toml::Value::Float(0.85),
            toml::Value::Float(0.55),
            toml::Value::Float(1.0),
        ]),
    );
    let mut base_color_slot = MaterialTextureSlotValue::new(texture.clone());
    base_color_slot.fallback = Some("white".to_string());
    let mut texture_slots = BTreeMap::new();
    texture_slots.insert("base_color".to_string(), base_color_slot);
    let material = MaterialAsset {
        name: Some("HeroSurface".to_string()),
        shader,
        base_color: [1.0, 0.85, 0.55, 1.0],
        base_color_texture: Some(texture),
        normal_texture: None,
        metallic: 0.0,
        roughness: 0.65,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
        property_values,
        texture_slots,
        validation_diagnostics: Vec::new(),
    };
    write_text(
        paths
            .assets_root()
            .join("materials")
            .join("hero_surface.zmaterial"),
        &material.to_toml_string().unwrap(),
    );
}

pub(super) fn write_bc1_texture(path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, dds_legacy_bytes("DXT1", 8)).unwrap();
}

fn write_text(path: PathBuf, text: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, text.trim_start()).unwrap();
}

fn dds_legacy_bytes(fourcc: &str, payload_bytes: usize) -> Vec<u8> {
    let mut bytes = vec![0_u8; 128];
    bytes[0..4].copy_from_slice(b"DDS ");
    write_u32_le(&mut bytes, 4, 124);
    write_u32_le(&mut bytes, 8, DDSD_REQUIRED_FLAGS | DDSD_LINEARSIZE);
    write_u32_le(&mut bytes, 12, 4);
    write_u32_le(&mut bytes, 16, 4);
    write_u32_le(&mut bytes, 20, payload_bytes as u32);
    write_u32_le(&mut bytes, 76, 32);
    write_u32_le(&mut bytes, 80, DDPF_FOURCC);
    bytes[84..88].copy_from_slice(fourcc.as_bytes());
    write_u32_le(&mut bytes, 108, DDSCAPS_TEXTURE);
    bytes.extend(vec![1_u8; payload_bytes]);
    bytes
}

fn write_u32_le(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

const DDPF_FOURCC: u32 = 0x0000_0004;
const DDSCAPS_TEXTURE: u32 = 0x0000_1000;
const DDSD_CAPS: u32 = 0x0000_0001;
const DDSD_HEIGHT: u32 = 0x0000_0002;
const DDSD_WIDTH: u32 = 0x0000_0004;
const DDSD_PIXELFORMAT: u32 = 0x0000_1000;
const DDSD_LINEARSIZE: u32 = 0x0008_0000;
const DDSD_REQUIRED_FLAGS: u32 = DDSD_CAPS | DDSD_HEIGHT | DDSD_WIDTH | DDSD_PIXELFORMAT;
