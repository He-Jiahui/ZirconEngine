use std::collections::HashSet;
use std::sync::Arc;

use crate::engine_module::EngineModule;
use crate::plugin::ProjectPluginSelection;
#[cfg(feature = "plugin-ui")]
use crate::ui;

use super::RuntimePluginId;

pub(super) fn linked_plugin_is_available(
    selection: &ProjectPluginSelection,
    runtime_id: RuntimePluginId,
    linked_plugin_ids: &HashSet<String>,
) -> bool {
    linked_plugin_ids.contains(&selection.id) || linked_plugin_ids.contains(runtime_id.key())
}

pub(super) fn builtin_runtime_domain_is_available(id: RuntimePluginId) -> bool {
    let _ = id;
    false
}

pub(super) fn builtin_runtime_domain_message(id: &str) -> String {
    format!("runtime plugin {id} is provided by the built-in runtime domain")
}

pub(super) fn module_for_plugin(
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
        RuntimePluginId::Ai => {
            warnings.push(externalized_runtime_plugin_message("ai"));
            None
        }
        RuntimePluginId::Physics => {
            warnings.push(externalized_runtime_plugin_message("physics"));
            None
        }
        RuntimePluginId::Sound => {
            warnings.push(externalized_runtime_plugin_message("sound"));
            None
        }
        RuntimePluginId::Texture => {
            warnings.push(externalized_runtime_plugin_message("texture"));
            None
        }
        RuntimePluginId::Net => {
            warnings.push(externalized_runtime_plugin_message("net"));
            None
        }
        RuntimePluginId::Navigation => {
            warnings.push(externalized_runtime_plugin_message("navigation"));
            None
        }
        RuntimePluginId::Particles => {
            warnings.push(externalized_runtime_plugin_message("particles"));
            None
        }
        RuntimePluginId::Animation => {
            warnings.push(externalized_runtime_plugin_message("animation"));
            None
        }
        RuntimePluginId::Terrain => {
            warnings.push(externalized_runtime_plugin_message("terrain"));
            None
        }
        RuntimePluginId::Tilemap2d => {
            warnings.push(externalized_runtime_plugin_message("tilemap_2d"));
            None
        }
        RuntimePluginId::PrefabTools => {
            warnings.push(externalized_runtime_plugin_message("prefab_tools"));
            None
        }
        RuntimePluginId::GltfImporter => {
            warnings.push(externalized_runtime_plugin_message("gltf_importer"));
            None
        }
        RuntimePluginId::ObjImporter => {
            warnings.push(externalized_runtime_plugin_message("obj_importer"));
            None
        }
        RuntimePluginId::TextureImporter => {
            warnings.push(externalized_runtime_plugin_message("texture_importer"));
            None
        }
        RuntimePluginId::AudioImporter => {
            warnings.push(externalized_runtime_plugin_message("audio_importer"));
            None
        }
        RuntimePluginId::ShaderWgslImporter => {
            warnings.push(externalized_runtime_plugin_message("shader_wgsl_importer"));
            None
        }
        RuntimePluginId::UiDocumentImporter => {
            warnings.push(externalized_runtime_plugin_message("ui_document_importer"));
            None
        }
        RuntimePluginId::Rendering => {
            warnings.push(externalized_runtime_plugin_message("rendering"));
            None
        }
        RuntimePluginId::VirtualGeometry => {
            warnings.push(externalized_runtime_plugin_message("virtual_geometry"));
            None
        }
        RuntimePluginId::HybridGi => {
            warnings.push(externalized_runtime_plugin_message("hybrid_gi"));
            None
        }
        RuntimePluginId::Solari => {
            warnings.push(externalized_runtime_plugin_message("solari"));
            None
        }
        RuntimePluginId::ZrVmLanguage => {
            warnings.push(externalized_runtime_plugin_message("zr_vm_language"));
            None
        }
    }
}

fn externalized_runtime_plugin_message(plugin_id: &str) -> String {
    format!("runtime implementation is externalized to zircon_plugins/{plugin_id}")
}
