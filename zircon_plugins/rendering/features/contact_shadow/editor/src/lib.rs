pub const FEATURE_ID: &str = zircon_plugin_rendering_contact_shadow_runtime::FEATURE_ID;
pub const CAPABILITY: &str = zircon_plugin_rendering_contact_shadow_runtime::EDITOR_CAPABILITY;

#[derive(Clone, Debug)]
pub struct RenderingContactShadowEditorFeature {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl RenderingContactShadowEditorFeature {
    pub fn new() -> Self {
        Self {
            descriptor: zircon_editor::EditorPluginDescriptor::new(
                FEATURE_ID,
                "Contact Shadow",
                "zircon_plugin_rendering_contact_shadow_editor",
            )
            .with_capability(CAPABILITY),
        }
    }
}

impl zircon_editor::EditorPlugin for RenderingContactShadowEditorFeature {
    fn descriptor(&self) -> &zircon_editor::EditorPluginDescriptor {
        &self.descriptor
    }
}

pub fn editor_feature() -> RenderingContactShadowEditorFeature {
    RenderingContactShadowEditorFeature::new()
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_feature()).to_vec()
}

pub fn feature_manifest() -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    zircon_plugin_rendering_contact_shadow_runtime::feature_manifest()
}
