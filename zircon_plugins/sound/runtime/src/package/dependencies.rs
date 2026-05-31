use zircon_runtime::plugin::PluginDependencyManifest;

pub fn sound_dependencies() -> Vec<PluginDependencyManifest> {
    vec![
        PluginDependencyManifest::new("asset", true).with_capability("runtime.module.asset"),
        PluginDependencyManifest::new("scene", true).with_capability("runtime.module.scene"),
        PluginDependencyManifest::new("ray_query", false)
            .with_capability("runtime.capability.ray_query"),
        PluginDependencyManifest::new("timeline_sequence", false)
            .with_capability("editor.extension.timeline_sequence_authoring"),
    ]
}
