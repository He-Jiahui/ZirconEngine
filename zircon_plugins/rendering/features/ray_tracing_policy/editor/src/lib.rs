pub const FEATURE_ID: &str = zircon_plugin_rendering_ray_tracing_policy_runtime::FEATURE_ID;
pub const CAPABILITY: &str = "editor.feature.rendering.ray_tracing_policy";

#[derive(Clone, Debug)]
pub struct RenderingRayTracingPolicyEditorFeature {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl RenderingRayTracingPolicyEditorFeature {
    pub fn new() -> Self {
        Self {
            descriptor: zircon_editor::EditorPluginDescriptor::new(
                FEATURE_ID,
                "Ray Tracing Policy",
                "zircon_plugin_rendering_ray_tracing_policy_editor",
            )
            .with_capability(CAPABILITY),
        }
    }
}

impl zircon_editor::EditorPlugin for RenderingRayTracingPolicyEditorFeature {
    fn descriptor(&self) -> &zircon_editor::EditorPluginDescriptor {
        &self.descriptor
    }
}

pub fn editor_feature() -> RenderingRayTracingPolicyEditorFeature {
    RenderingRayTracingPolicyEditorFeature::new()
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_feature()).to_vec()
}

pub fn feature_manifest() -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    zircon_plugin_rendering_ray_tracing_policy_runtime::feature_manifest()
}
