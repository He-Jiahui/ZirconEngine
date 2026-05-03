mod authoring;

#[cfg(test)]
mod tests;

pub use authoring::{
    PARTICLES_AUTHORING_CAPABILITY, PARTICLES_AUTHORING_VIEW_ID, PARTICLES_COMPONENT_DRAWER_ID,
    PARTICLES_CPU_SPRITE_TEMPLATE_DOCUMENT, PARTICLES_CPU_SPRITE_TEMPLATE_ID, PARTICLES_DRAWER_ID,
    PARTICLES_PREVIEW_TEMPLATE_ID, PARTICLES_PREVIEW_VIEW_ID, PARTICLES_SYSTEM_ASSET_KIND,
    PARTICLES_TEMPLATE_ID,
};

pub const PLUGIN_ID: &str = zircon_plugin_particles_runtime::PLUGIN_ID;

#[derive(Clone, Debug)]
pub struct ParticlesEditorPlugin {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl ParticlesEditorPlugin {
    pub fn new() -> Self {
        Self {
            descriptor: editor_plugin_descriptor(),
        }
    }
}

impl zircon_editor::EditorPlugin for ParticlesEditorPlugin {
    fn descriptor(&self) -> &zircon_editor::EditorPluginDescriptor {
        &self.descriptor
    }

    fn register_editor_extensions(
        &self,
        registry: &mut zircon_editor::core::editor_extension::EditorExtensionRegistry,
    ) -> Result<(), zircon_editor::core::editor_extension::EditorExtensionRegistryError> {
        authoring::register_particles_authoring_extensions(registry)
    }
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    zircon_editor::EditorPluginDescriptor::new(
        PLUGIN_ID,
        "Particles",
        "zircon_plugin_particles_editor",
    )
    .with_capability(PARTICLES_AUTHORING_CAPABILITY)
}

pub fn editor_plugin() -> ParticlesEditorPlugin {
    ParticlesEditorPlugin::new()
}

pub fn package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    zircon_editor::EditorPlugin::package_manifest(
        &editor_plugin(),
        zircon_plugin_particles_runtime::package_manifest(),
    )
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_plugin()).to_vec()
}

pub fn plugin_registration() -> zircon_editor::EditorPluginRegistrationReport {
    zircon_editor::EditorPluginRegistrationReport::from_plugin(
        &editor_plugin(),
        zircon_plugin_particles_runtime::package_manifest(),
    )
}

pub fn editor_host_contract_marker() -> &'static str {
    zircon_editor::EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY
}
