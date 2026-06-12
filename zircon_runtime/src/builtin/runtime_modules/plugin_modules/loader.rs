use std::sync::Arc;

use crate::engine_module::EngineModule;
#[cfg(feature = "plugin-ui")]
use crate::ui;

use super::super::ids::RuntimePluginId;

pub(in crate::builtin::runtime_modules) fn module_for_plugin(
    id: RuntimePluginId,
    warnings: &mut Vec<String>,
) -> Option<Arc<dyn EngineModule>> {
    match id {
        RuntimePluginId::Ui => {
            #[cfg(feature = "plugin-ui")]
            {
                return Some(Arc::new(ui::UiModule));
            }
            #[cfg(not(feature = "plugin-ui"))]
            {
                warnings.push("plugin-ui feature is disabled".to_string());
                None
            }
        }
        RuntimePluginId::Ai => externalized_runtime_plugin_module("ai", warnings),
        RuntimePluginId::Physics => externalized_runtime_plugin_module("physics", warnings),
        RuntimePluginId::Sound => externalized_runtime_plugin_module("sound", warnings),
        RuntimePluginId::Texture => externalized_runtime_plugin_module("texture", warnings),
        RuntimePluginId::Net => externalized_runtime_plugin_module("net", warnings),
        RuntimePluginId::Navigation => externalized_runtime_plugin_module("navigation", warnings),
        RuntimePluginId::Particles => externalized_runtime_plugin_module("particles", warnings),
        RuntimePluginId::Animation => externalized_runtime_plugin_module("animation", warnings),
        RuntimePluginId::Terrain => externalized_runtime_plugin_module("terrain", warnings),
        RuntimePluginId::Tilemap2d => externalized_runtime_plugin_module("tilemap_2d", warnings),
        RuntimePluginId::PrefabTools => {
            externalized_runtime_plugin_module("prefab_tools", warnings)
        }
        RuntimePluginId::GltfImporter => {
            externalized_runtime_plugin_module("gltf_importer", warnings)
        }
        RuntimePluginId::ObjImporter => {
            externalized_runtime_plugin_module("obj_importer", warnings)
        }
        RuntimePluginId::TextureImporter => {
            externalized_runtime_plugin_module("texture_importer", warnings)
        }
        RuntimePluginId::AudioImporter => {
            externalized_runtime_plugin_module("audio_importer", warnings)
        }
        RuntimePluginId::ShaderWgslImporter => {
            externalized_runtime_plugin_module("shader_wgsl_importer", warnings)
        }
        RuntimePluginId::UiDocumentImporter => {
            externalized_runtime_plugin_module("ui_document_importer", warnings)
        }
        RuntimePluginId::Rendering => externalized_runtime_plugin_module("rendering", warnings),
        RuntimePluginId::VirtualGeometry => {
            externalized_runtime_plugin_module("virtual_geometry", warnings)
        }
        RuntimePluginId::HybridGi => externalized_runtime_plugin_module("hybrid_gi", warnings),
        RuntimePluginId::Solari => externalized_runtime_plugin_module("solari", warnings),
        RuntimePluginId::ZrVmLanguage => {
            externalized_runtime_plugin_module("zr_vm_language", warnings)
        }
    }
}

fn externalized_runtime_plugin_module(
    plugin_id: &str,
    warnings: &mut Vec<String>,
) -> Option<Arc<dyn EngineModule>> {
    warnings.push(externalized_runtime_plugin_message(plugin_id));
    None
}

fn externalized_runtime_plugin_message(plugin_id: &str) -> String {
    format!("runtime implementation is externalized to zircon_plugins/{plugin_id}")
}
