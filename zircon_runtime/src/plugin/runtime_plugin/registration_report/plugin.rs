use crate::plugin::runtime_plugin::{
    descriptor::validate_runtime_plugin_descriptor, RuntimePlugin,
};
use crate::plugin::RuntimeExtensionRegistry;

use super::{
    package_contributions::register_package_manifest_contributions,
    validation::{
        validate_runtime_plugin_package_manifest, validate_runtime_plugin_registration_interfaces,
        validate_runtime_plugin_registration_system_anchors,
    },
    RuntimePluginRegistrationReport,
};

impl RuntimePluginRegistrationReport {
    pub fn from_plugin(plugin: &dyn RuntimePlugin) -> Self {
        let mut extensions = RuntimeExtensionRegistry::default();
        let mut diagnostics = Vec::new();
        validate_runtime_plugin_descriptor(plugin, &mut diagnostics);
        if let Err(error) = extensions.register_module(plugin.module_descriptor().clone()) {
            diagnostics.push(error.to_string());
        }
        if let Err(error) = plugin.register(&mut extensions) {
            diagnostics.push(error.to_string());
        }
        let package_manifest = plugin.package_manifest();
        for source in plugin.shader_module_sources() {
            if let Err(error) =
                extensions.register_plugin_shader_module_source(&package_manifest.id, source)
            {
                diagnostics.push(error.to_string());
            }
        }
        let projection = validate_runtime_plugin_package_manifest(
            Some(plugin.descriptor()),
            &package_manifest,
            &mut diagnostics,
        );
        register_package_manifest_contributions(
            &package_manifest,
            &mut extensions,
            &mut diagnostics,
        );
        validate_runtime_plugin_registration_interfaces(
            &package_manifest,
            &projection,
            &extensions,
            &mut diagnostics,
        );
        validate_runtime_plugin_registration_system_anchors(
            &package_manifest,
            &projection,
            &extensions,
            &mut diagnostics,
        );
        Self {
            package_manifest,
            project_selection: plugin.project_selection(),
            extensions,
            diagnostics,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RuntimePluginRegistrationReport;
    use crate::builtin::RuntimePluginId;
    use crate::plugin::{PluginShaderModuleSource, RuntimePlugin, RuntimePluginDescriptor};

    struct LinkedShaderModuleFixture {
        descriptor: RuntimePluginDescriptor,
        source: PluginShaderModuleSource,
    }

    impl RuntimePlugin for LinkedShaderModuleFixture {
        fn descriptor(&self) -> &RuntimePluginDescriptor {
            &self.descriptor
        }

        fn shader_module_sources(&self) -> Vec<PluginShaderModuleSource> {
            vec![self.source.clone()]
        }
    }

    #[test]
    fn linked_plugin_shader_module_source_is_registered_with_the_runtime_owner() {
        let package_id = "zircon.fixture.linked";
        let plugin = LinkedShaderModuleFixture {
            descriptor: RuntimePluginDescriptor::builder(
                package_id,
                "Linked Shader Fixture",
                RuntimePluginId::new("linked_shader_fixture"),
                "zircon_fixture_linked_shader",
            )
            .build(),
            source: PluginShaderModuleSource::new(
                package_id,
                "zircon_fixture::linked_lighting",
                "fn linked_fixture_lighting() -> vec3f { return vec3f(0.3); }",
                "linked shader fixture",
            ),
        };

        let report = RuntimePluginRegistrationReport::from_plugin(&plugin);

        assert!(report.diagnostics.is_empty());
        assert_eq!(report.extensions.shader_module_sources(), &[plugin.source]);
    }
}
