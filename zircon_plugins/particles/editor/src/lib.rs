use zircon_plugin_editor_support::{
    register_authoring_extensions, EditorAuthoringExtensions, EditorAuthoringSurface,
};
use zircon_plugin_particles_runtime::PARTICLE_SYSTEM_COMPONENT_TYPE;

pub const PLUGIN_ID: &str = zircon_plugin_particles_runtime::PLUGIN_ID;
pub const PARTICLES_AUTHORING_VIEW_ID: &str = "particles.authoring";
pub const PARTICLES_PREVIEW_VIEW_ID: &str = "particles.preview";
pub const PARTICLES_DRAWER_ID: &str = "particles.drawer";
pub const PARTICLES_TEMPLATE_ID: &str = "particles.authoring";
pub const PARTICLES_PREVIEW_TEMPLATE_ID: &str = "particles.preview";
pub const PARTICLES_COMPONENT_DRAWER_ID: &str = "particles.Component.ParticleSystem.drawer";

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
        register_authoring_extensions(
            registry,
            EditorAuthoringExtensions {
                drawer_id: PARTICLES_DRAWER_ID,
                drawer_display_name: "Particles Tools",
                template_id: PARTICLES_TEMPLATE_ID,
                template_document: "plugins://particles/editor/authoring.ui.toml",
                surfaces: &[
                    EditorAuthoringSurface::new(
                        PARTICLES_AUTHORING_VIEW_ID,
                        "Particles",
                        "Effects",
                        "Plugins/Particles/Authoring",
                    ),
                    EditorAuthoringSurface::new(
                        PARTICLES_PREVIEW_VIEW_ID,
                        "Particle Preview",
                        "Effects",
                        "Plugins/Particles/Preview",
                    ),
                ],
            },
        )?;
        register_particles_component_drawers(registry)
    }
}

fn register_particles_component_drawers(
    registry: &mut zircon_editor::core::editor_extension::EditorExtensionRegistry,
) -> Result<(), zircon_editor::core::editor_extension::EditorExtensionRegistryError> {
    use zircon_editor::core::editor_extension::{
        ComponentDrawerDescriptor, EditorUiTemplateDescriptor,
    };

    registry.register_ui_template(EditorUiTemplateDescriptor::new(
        PARTICLES_PREVIEW_TEMPLATE_ID,
        "plugins://particles/editor/preview.ui.toml",
    ))?;
    registry.register_component_drawer(ComponentDrawerDescriptor::new(
        PARTICLE_SYSTEM_COMPONENT_TYPE,
        "plugins://particles/editor/particle_system.drawer.ui.toml",
        PARTICLES_COMPONENT_DRAWER_ID,
    ))
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    zircon_editor::EditorPluginDescriptor::new(
        PLUGIN_ID,
        "Particles",
        "zircon_plugin_particles_editor",
    )
    .with_capability("editor.extension.particles_authoring")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn particles_editor_plugin_contributes_authoring_extensions() {
        let registration = plugin_registration();

        assert!(registration.is_success(), "{:?}", registration.diagnostics);
        assert!(registration
            .capabilities
            .contains(&"editor.extension.particles_authoring".to_string()));
        assert!(registration
            .extensions
            .views()
            .iter()
            .any(|view| view.id() == PARTICLES_AUTHORING_VIEW_ID));
        assert!(registration
            .extensions
            .views()
            .iter()
            .any(|view| view.id() == PARTICLES_PREVIEW_VIEW_ID));
        assert!(registration
            .extensions
            .drawers()
            .iter()
            .any(|drawer| drawer.id() == PARTICLES_DRAWER_ID));
        assert!(registration
            .extensions
            .ui_templates()
            .iter()
            .any(|template| template.id() == PARTICLES_TEMPLATE_ID));
        assert!(registration
            .extensions
            .ui_templates()
            .iter()
            .any(|template| template.id() == PARTICLES_PREVIEW_TEMPLATE_ID));
        assert!(registration
            .extensions
            .component_drawers()
            .iter()
            .any(|drawer| {
                drawer.component_type() == PARTICLE_SYSTEM_COMPONENT_TYPE
                    && drawer.controller() == PARTICLES_COMPONENT_DRAWER_ID
            }));
        assert!(registration
            .extensions
            .menu_items()
            .iter()
            .any(|menu| menu.operation().as_str() == "View.particles.authoring.Open"));
        assert!(registration
            .extensions
            .menu_items()
            .iter()
            .any(|menu| menu.operation().as_str() == "View.particles.preview.Open"));
        assert!(registration
            .extensions
            .operations()
            .descriptors()
            .any(|operation| operation.path().as_str() == "View.particles.authoring.Open"));
        assert!(registration
            .extensions
            .operations()
            .descriptors()
            .any(|operation| operation.path().as_str() == "View.particles.preview.Open"));
    }
}
