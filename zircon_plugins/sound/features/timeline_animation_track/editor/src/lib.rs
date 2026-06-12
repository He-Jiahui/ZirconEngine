pub const FEATURE_ID: &str = zircon_plugin_sound_timeline_animation_runtime::FEATURE_ID;
pub const CAPABILITY: &str = zircon_plugin_sound_timeline_animation_runtime::EDITOR_CAPABILITY;

#[derive(Clone, Debug)]
pub struct SoundTimelineAnimationEditorFeature {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl SoundTimelineAnimationEditorFeature {
    pub fn new() -> Self {
        Self {
            descriptor: zircon_editor::EditorPluginDescriptor::new(
                FEATURE_ID,
                "Sound Timeline Animation Track",
                "zircon_plugin_sound_timeline_animation_editor",
            )
            .with_capability(CAPABILITY),
        }
    }
}

impl zircon_editor::EditorPlugin for SoundTimelineAnimationEditorFeature {
    fn descriptor(&self) -> &zircon_editor::EditorPluginDescriptor {
        &self.descriptor
    }
}

pub fn editor_feature() -> SoundTimelineAnimationEditorFeature {
    SoundTimelineAnimationEditorFeature::new()
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_feature()).to_vec()
}

pub fn feature_manifest() -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    zircon_plugin_sound_timeline_animation_runtime::feature_manifest()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeline_editor_feature_descriptor_matches_runtime_feature_contract() {
        let feature = editor_feature();
        let descriptor = zircon_editor::EditorPlugin::descriptor(&feature);

        assert_eq!(descriptor.package_id, FEATURE_ID);
        assert_eq!(descriptor.display_name, "Sound Timeline Animation Track");
        assert_eq!(
            descriptor.crate_name,
            "zircon_plugin_sound_timeline_animation_editor"
        );
        assert_eq!(descriptor.capabilities, vec![CAPABILITY.to_string()]);
        assert_eq!(editor_capabilities(), vec![CAPABILITY.to_string()]);
    }

    #[test]
    fn timeline_editor_feature_manifest_matches_runtime_provider_manifest() {
        assert_eq!(
            feature_manifest(),
            zircon_plugin_sound_timeline_animation_runtime::feature_manifest()
        );
    }
}
