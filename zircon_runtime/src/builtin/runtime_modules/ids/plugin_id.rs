use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimePluginId {
    Ui,
    Ai,
    Physics,
    Sound,
    Texture,
    Net,
    Navigation,
    Particles,
    Animation,
    Terrain,
    Tilemap2d,
    PrefabTools,
    GltfImporter,
    ObjImporter,
    AssetImporterData,
    AssetImporterModel,
    AssetImporterShader,
    TextureImporter,
    AudioImporter,
    OpusImporter,
    ShaderWgslImporter,
    UiDocumentImporter,
    Rendering,
    VirtualGeometry,
    HybridGi,
    Solari,
    ZrVmLanguage,
}

impl RuntimePluginId {
    pub const fn key(self) -> &'static str {
        match self {
            Self::Ui => "ui",
            Self::Ai => "ai",
            Self::Physics => "physics",
            Self::Sound => "sound",
            Self::Texture => "texture",
            Self::Net => "net",
            Self::Navigation => "navigation",
            Self::Particles => "particles",
            Self::Animation => "animation",
            Self::Terrain => "terrain",
            Self::Tilemap2d => "tilemap_2d",
            Self::PrefabTools => "prefab_tools",
            Self::GltfImporter => "gltf_importer",
            Self::ObjImporter => "obj_importer",
            Self::AssetImporterData => "asset_importer.data",
            Self::AssetImporterModel => "asset_importer.model",
            Self::AssetImporterShader => "asset_importer.shader",
            Self::TextureImporter => "texture_importer",
            Self::AudioImporter => "audio_importer",
            Self::OpusImporter => "opus_importer",
            Self::ShaderWgslImporter => "shader_wgsl_importer",
            Self::UiDocumentImporter => "ui_document_importer",
            Self::Rendering => "rendering",
            Self::VirtualGeometry => "virtual_geometry",
            Self::HybridGi => "hybrid_gi",
            Self::Solari => "solari",
            Self::ZrVmLanguage => "zr_vm_language",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Ui => "Ui",
            Self::Ai => "AI",
            Self::Physics => "Physics",
            Self::Sound => "Sound",
            Self::Texture => "Texture",
            Self::Net => "Net",
            Self::Navigation => "Navigation",
            Self::Particles => "Particles",
            Self::Animation => "Animation",
            Self::Terrain => "Terrain",
            Self::Tilemap2d => "Tilemap2d",
            Self::PrefabTools => "PrefabTools",
            Self::GltfImporter => "GltfImporter",
            Self::ObjImporter => "ObjImporter",
            Self::AssetImporterData => "AssetImporterData",
            Self::AssetImporterModel => "AssetImporterModel",
            Self::AssetImporterShader => "AssetImporterShader",
            Self::TextureImporter => "TextureImporter",
            Self::AudioImporter => "AudioImporter",
            Self::OpusImporter => "OpusImporter",
            Self::ShaderWgslImporter => "ShaderWgslImporter",
            Self::UiDocumentImporter => "UiDocumentImporter",
            Self::Rendering => "Rendering",
            Self::VirtualGeometry => "VirtualGeometry",
            Self::HybridGi => "HybridGi",
            Self::Solari => "Solari",
            Self::ZrVmLanguage => "ZrVM Language",
        }
    }

    pub fn parse_key(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "ui" => Some(Self::Ui),
            "ai" | "artificial_intelligence" | "game_ai" => Some(Self::Ai),
            "physics" => Some(Self::Physics),
            "sound" | "audio" => Some(Self::Sound),
            "texture" => Some(Self::Texture),
            "net" | "network" => Some(Self::Net),
            "navigation" | "nav" => Some(Self::Navigation),
            "particles" => Some(Self::Particles),
            "animation" => Some(Self::Animation),
            "terrain" => Some(Self::Terrain),
            "tilemap_2d" | "tilemap" | "tile_map_2d" => Some(Self::Tilemap2d),
            "prefab_tools" | "prefab" | "prefabs" => Some(Self::PrefabTools),
            "gltf_importer" | "gltf" | "glb_importer" => Some(Self::GltfImporter),
            "obj_importer" | "obj" | "wavefront_obj" => Some(Self::ObjImporter),
            "asset_importer.data" | "asset_importer_data" | "data_asset_importer" => {
                Some(Self::AssetImporterData)
            }
            "asset_importer.model" | "asset_importer_model" | "model_asset_importer" => {
                Some(Self::AssetImporterModel)
            }
            "asset_importer.shader" | "asset_importer_shader" | "shader_asset_importer" => {
                Some(Self::AssetImporterShader)
            }
            "texture_importer" | "image_importer" => Some(Self::TextureImporter),
            "audio_importer" | "sound_importer" | "wav_importer" => Some(Self::AudioImporter),
            "opus_importer" | "opus" => Some(Self::OpusImporter),
            "shader_wgsl_importer" | "wgsl_importer" => Some(Self::ShaderWgslImporter),
            "ui_document_importer" | "ui_importer" | "ui_asset_importer" => {
                Some(Self::UiDocumentImporter)
            }
            "rendering" | "renderer" | "graphics" => Some(Self::Rendering),
            "vg" | "virtual_geometry" => Some(Self::VirtualGeometry),
            "gi" | "hybrid_gi" => Some(Self::HybridGi),
            "solari" => Some(Self::Solari),
            "zr_vm_language" | "zr_vm" | "zrvmlanguage" => Some(Self::ZrVmLanguage),
            _ => None,
        }
    }
}
