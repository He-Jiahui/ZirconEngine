use crate::{CAPABILITY, FEATURE_ID};

#[derive(Clone, Debug)]
pub struct RenderingLightCookiesEditorFeature {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl RenderingLightCookiesEditorFeature {
    pub fn new() -> Self {
        Self {
            descriptor: zircon_editor::EditorPluginDescriptor::new(
                FEATURE_ID,
                "Light Cookies",
                "zircon_plugin_rendering_light_cookies_editor",
            )
            .with_capability(CAPABILITY),
        }
    }
}

impl Default for RenderingLightCookiesEditorFeature {
    fn default() -> Self {
        Self::new()
    }
}

impl zircon_editor::EditorPlugin for RenderingLightCookiesEditorFeature {
    fn descriptor(&self) -> &zircon_editor::EditorPluginDescriptor {
        &self.descriptor
    }
}

pub fn editor_feature() -> RenderingLightCookiesEditorFeature {
    RenderingLightCookiesEditorFeature::new()
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_feature()).to_vec()
}

pub fn feature_manifest() -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    zircon_plugin_rendering_light_cookies_runtime::feature_manifest()
}
