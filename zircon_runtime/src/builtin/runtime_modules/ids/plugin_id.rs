use std::collections::HashSet;
use std::fmt;
use std::str::FromStr;
use std::sync::{Mutex, OnceLock};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RuntimePluginId(&'static str);

#[allow(non_upper_case_globals)]
impl RuntimePluginId {
    pub const Ui: Self = Self::from_static("ui");
    pub const Ai: Self = Self::from_static("ai");
    pub const Physics: Self = Self::from_static("physics");
    pub const Sound: Self = Self::from_static("sound");
    pub const Texture: Self = Self::from_static("texture");
    pub const Net: Self = Self::from_static("net");
    pub const Navigation: Self = Self::from_static("navigation");
    pub const Particles: Self = Self::from_static("particles");
    pub const Animation: Self = Self::from_static("animation");
    pub const Terrain: Self = Self::from_static("terrain");
    pub const Tilemap2d: Self = Self::from_static("tilemap_2d");
    pub const PrefabTools: Self = Self::from_static("prefab_tools");
    pub const GltfImporter: Self = Self::from_static("gltf_importer");
    pub const ObjImporter: Self = Self::from_static("obj_importer");
    pub const AssetImporterData: Self = Self::from_static("asset_importer.data");
    pub const AssetImporterModel: Self = Self::from_static("asset_importer.model");
    pub const AssetImporterShader: Self = Self::from_static("asset_importer.shader");
    pub const TextureImporter: Self = Self::from_static("texture_importer");
    pub const AudioImporter: Self = Self::from_static("audio_importer");
    pub const OpusImporter: Self = Self::from_static("opus_importer");
    pub const ShaderWgslImporter: Self = Self::from_static("shader_wgsl_importer");
    pub const UiDocumentImporter: Self = Self::from_static("ui_document_importer");
    pub const Rendering: Self = Self::from_static("rendering");
    pub const VirtualGeometry: Self = Self::from_static("virtual_geometry");
    pub const HybridGi: Self = Self::from_static("hybrid_gi");
    pub const Solari: Self = Self::from_static("solari");
    pub const ZrVmLanguage: Self = Self::from_static("zr_vm_language");

    pub const fn from_static(key: &'static str) -> Self {
        Self(key)
    }

    pub fn new(raw: impl AsRef<str>) -> Self {
        Self::parse_key(raw.as_ref()).expect("runtime plugin id must be a non-empty key")
    }

    pub const fn key(self) -> &'static str {
        self.0
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }

    pub fn label(self) -> &'static str {
        match self.key() {
            "ui" => "Ui",
            "ai" => "AI",
            "physics" => "Physics",
            "sound" => "Sound",
            "texture" => "Texture",
            "net" => "Net",
            "navigation" => "Navigation",
            "particles" => "Particles",
            "animation" => "Animation",
            "terrain" => "Terrain",
            "tilemap_2d" => "Tilemap2d",
            "prefab_tools" => "PrefabTools",
            "gltf_importer" => "GltfImporter",
            "obj_importer" => "ObjImporter",
            "asset_importer.data" => "AssetImporterData",
            "asset_importer.model" => "AssetImporterModel",
            "asset_importer.shader" => "AssetImporterShader",
            "texture_importer" => "TextureImporter",
            "audio_importer" => "AudioImporter",
            "opus_importer" => "OpusImporter",
            "shader_wgsl_importer" => "ShaderWgslImporter",
            "ui_document_importer" => "UiDocumentImporter",
            "rendering" => "Rendering",
            "virtual_geometry" => "VirtualGeometry",
            "hybrid_gi" => "HybridGi",
            "solari" => "Solari",
            "zr_vm_language" => "ZrVM Language",
            key => key,
        }
    }

    pub fn parse_key(raw: &str) -> Option<Self> {
        let normalized = normalize_runtime_plugin_key(raw)?;
        Some(match normalized.as_str() {
            "ui" => Self::Ui,
            "ai" | "artificial_intelligence" | "game_ai" => Self::Ai,
            "physics" => Self::Physics,
            "sound" | "audio" => Self::Sound,
            "texture" => Self::Texture,
            "net" | "network" => Self::Net,
            "navigation" | "nav" => Self::Navigation,
            "particles" => Self::Particles,
            "animation" => Self::Animation,
            "terrain" => Self::Terrain,
            "tilemap_2d" | "tilemap" | "tile_map_2d" => Self::Tilemap2d,
            "prefab_tools" | "prefab" | "prefabs" => Self::PrefabTools,
            "gltf_importer" | "gltf" | "glb_importer" => Self::GltfImporter,
            "obj_importer" | "obj" | "wavefront_obj" => Self::ObjImporter,
            "asset_importer.data" | "asset_importer_data" | "data_asset_importer" => {
                Self::AssetImporterData
            }
            "asset_importer.model" | "asset_importer_model" | "model_asset_importer" => {
                Self::AssetImporterModel
            }
            "asset_importer.shader" | "asset_importer_shader" | "shader_asset_importer" => {
                Self::AssetImporterShader
            }
            "texture_importer" | "image_importer" => Self::TextureImporter,
            "audio_importer" | "sound_importer" | "wav_importer" => Self::AudioImporter,
            "opus_importer" | "opus" => Self::OpusImporter,
            "shader_wgsl_importer" | "wgsl_importer" => Self::ShaderWgslImporter,
            "ui_document_importer" | "ui_importer" | "ui_asset_importer" => {
                Self::UiDocumentImporter
            }
            "rendering" | "renderer" | "graphics" => Self::Rendering,
            "vg" | "virtual_geometry" => Self::VirtualGeometry,
            "gi" | "hybrid_gi" => Self::HybridGi,
            "solari" => Self::Solari,
            "zr_vm_language" | "zr_vm" | "zrvmlanguage" => Self::ZrVmLanguage,
            _ => Self(intern_runtime_plugin_key(normalized)),
        })
    }
}

impl AsRef<str> for RuntimePluginId {
    fn as_ref(&self) -> &str {
        self.key()
    }
}

impl fmt::Display for RuntimePluginId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.key())
    }
}

impl FromStr for RuntimePluginId {
    type Err = RuntimePluginIdParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        Self::parse_key(raw).ok_or(RuntimePluginIdParseError)
    }
}

impl Serialize for RuntimePluginId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.key())
    }
}

impl<'de> Deserialize<'de> for RuntimePluginId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Self::parse_key(&raw)
            .ok_or_else(|| serde::de::Error::custom(format!("invalid runtime plugin id `{raw}`")))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimePluginIdParseError;

impl fmt::Display for RuntimePluginIdParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid runtime plugin id")
    }
}

impl std::error::Error for RuntimePluginIdParseError {}

fn normalize_runtime_plugin_key(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty()
        || !trimmed
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
        || !trimmed
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_alphanumeric())
    {
        return None;
    }
    Some(trimmed.to_ascii_lowercase())
}

fn intern_runtime_plugin_key(key: String) -> &'static str {
    static INTERNED_KEYS: OnceLock<Mutex<HashSet<&'static str>>> = OnceLock::new();

    let mut keys = INTERNED_KEYS
        .get_or_init(|| Mutex::new(HashSet::new()))
        .lock()
        .expect("runtime plugin id interner lock should not be poisoned");
    if let Some(existing) = keys.get(key.as_str()) {
        return *existing;
    }
    let leaked = Box::leak(key.into_boxed_str());
    keys.insert(leaked);
    leaked
}

#[cfg(test)]
mod tests {
    use super::RuntimePluginId;

    #[test]
    fn runtime_plugin_id_accepts_external_keys_without_core_variant() {
        let id = RuntimePluginId::new("third_party.weather_sim");

        assert_eq!(id.key(), "third_party.weather_sim");
        assert_eq!(id.label(), "third_party.weather_sim");
        assert_eq!(
            RuntimePluginId::parse_key("Third_Party.Weather_Sim"),
            Some(id)
        );
    }

    #[test]
    fn runtime_plugin_id_rejects_empty_or_non_key_text() {
        for raw in ["", " ", ".starts_with_dot", "bad id", "bad/id"] {
            assert_eq!(RuntimePluginId::parse_key(raw), None);
        }
    }
}
