pub const FEATURE_ID: &str = zircon_plugin_sound_timeline_animation_runtime::FEATURE_ID;
pub const CAPABILITY: &str = "editor.feature.sound.timeline_animation_track";

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
