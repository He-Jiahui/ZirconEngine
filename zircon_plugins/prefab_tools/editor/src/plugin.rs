use zircon_editor::core::editor_extension::EditorExtensionRegistry;
use zircon_editor::core::extension::InspectorCustomizationDescriptor;
use zircon_plugin_editor_support::{
    register_authoring_contribution_batch, register_authoring_extensions,
    EditorAuthoringContributionBatch, EditorAuthoringExtensions, EditorAuthoringSurface,
};

use crate::{
    CAPABILITY, PLUGIN_ID, PREFAB_AUTHORING_VIEW_ID, PREFAB_DRAWER_ID, PREFAB_TEMPLATE_ID,
};

#[derive(Clone, Debug)]
pub struct PrefabToolsEditorPlugin {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl PrefabToolsEditorPlugin {
    pub fn new() -> Self {
        Self {
            descriptor: editor_plugin_descriptor(),
        }
    }
}

impl zircon_editor::EditorPlugin for PrefabToolsEditorPlugin {
    fn descriptor(&self) -> &zircon_editor::EditorPluginDescriptor {
        &self.descriptor
    }

    fn register_editor_extensions(
        &self,
        registry: &mut EditorExtensionRegistry,
    ) -> Result<(), zircon_editor::core::editor_extension::EditorExtensionRegistryError> {
        register_authoring_extensions(
            registry,
            EditorAuthoringExtensions {
                drawer_id: PREFAB_DRAWER_ID,
                drawer_display_name: "Prefab Tools",
                template_id: PREFAB_TEMPLATE_ID,
                template_document: "plugins://prefab_tools/editor/authoring.zui",
                surfaces: &[EditorAuthoringSurface::new(
                    PREFAB_AUTHORING_VIEW_ID,
                    "Prefabs",
                    "World",
                )],
            },
        )?;
        register_authoring_contribution_batch(registry, prefab_authoring_batch())
    }
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    zircon_editor::EditorPluginDescriptor::new(
        PLUGIN_ID,
        "Prefab Tools",
        "zircon_plugin_prefab_tools_editor",
    )
    .with_capability(CAPABILITY)
}

pub fn editor_plugin() -> PrefabToolsEditorPlugin {
    PrefabToolsEditorPlugin::new()
}

pub fn package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    zircon_editor::EditorPlugin::package_manifest(
        &editor_plugin(),
        zircon_plugin_prefab_tools_runtime::package_manifest(),
    )
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_plugin()).to_vec()
}

pub fn plugin_registration() -> zircon_editor::EditorPluginRegistrationReport {
    zircon_editor::EditorPluginRegistrationReport::from_plugin(
        &editor_plugin(),
        zircon_plugin_prefab_tools_runtime::package_manifest(),
    )
}

fn prefab_authoring_batch() -> EditorAuthoringContributionBatch {
    EditorAuthoringContributionBatch {
        inspector_customizations: vec![InspectorCustomizationDescriptor::new(
            zircon_plugin_prefab_tools_runtime::PREFAB_INSTANCE_COMPONENT_TYPE,
            "plugins://prefab_tools/editor/prefab_instance.zui",
            "prefab_tools.editor.component",
        )],
        ..Default::default()
    }
}
