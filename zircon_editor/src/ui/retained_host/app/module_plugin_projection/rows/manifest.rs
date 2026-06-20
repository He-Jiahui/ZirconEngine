use zircon_runtime::asset::{project::ProjectManifest, AssetUri};

pub(in crate::ui::retained_host::app::module_plugin_projection) fn fallback_project_manifest(
) -> ProjectManifest {
    ProjectManifest::new(
        "Unsaved",
        AssetUri::parse("res://scenes/main.scene.toml")
            .expect("fallback project asset URI is valid"),
        1,
    )
}
