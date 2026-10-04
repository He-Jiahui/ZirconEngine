use super::collect_shader_module_sources;
use crate::plugin::{PluginShaderModuleSource, RuntimeExtensionRegistry};

#[test]
fn identical_feature_extension_shader_modules_are_collected_once() {
    let source = PluginShaderModuleSource::new(
        "feature-extension-fixture",
        "zircon_fixture::feature_extension",
        "fn feature_extension_lighting() -> vec3f { return vec3f(0.2); }",
        "feature extension fixture",
    );
    let mut first = RuntimeExtensionRegistry::default();
    first
        .register_plugin_shader_module_source("feature-extension-fixture", source.clone())
        .expect("first feature registration should accept its module");
    let mut second = RuntimeExtensionRegistry::default();
    second
        .register_plugin_shader_module_source("feature-extension-fixture", source)
        .expect("second feature registration should accept its module");

    let collected = collect_shader_module_sources(&[&first, &second]);

    assert_eq!(
        collected.len(),
        1,
        "the same package module attached to multiple active features is one runtime source"
    );
}
