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
