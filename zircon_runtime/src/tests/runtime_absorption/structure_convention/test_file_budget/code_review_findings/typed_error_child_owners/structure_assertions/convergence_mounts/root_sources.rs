use super::*;

pub(super) struct TypedErrorConvergenceMountSources {
    pub(super) typed_error_parent: String,
    pub(super) asset_loaders_parent: String,
    pub(super) asset_records_parent: String,
    pub(super) scene_world_parent: String,
    pub(super) script_host_parent: String,
    pub(super) shader_prewarm_cli_parent: String,
    pub(super) ui_input_parent: String,
}

pub(super) fn typed_error_convergence_mount_sources() -> TypedErrorConvergenceMountSources {
    TypedErrorConvergenceMountSources {
        typed_error_parent: read_runtime_src(TYPED_ERROR_CONVERGENCE_PARENT),
        asset_loaders_parent: read_runtime_src(TYPED_ERROR_ASSET_LOADERS_PARENT),
        asset_records_parent: read_runtime_src(TYPED_ERROR_ASSET_RECORDS_PARENT),
        scene_world_parent: read_runtime_src(TYPED_ERROR_SCENE_WORLD_PARENT),
        script_host_parent: read_runtime_src(TYPED_ERROR_SCRIPT_HOST_PARENT),
        shader_prewarm_cli_parent: read_runtime_src(TYPED_ERROR_SHADER_PREWARM_CLI_PARENT),
        ui_input_parent: read_runtime_src(TYPED_ERROR_UI_INPUT_PARENT),
    }
}

pub(super) fn typed_error_convergence_mount_source_files<'a>(
    sources: &'a TypedErrorConvergenceMountSources,
) -> [(&'static str, &'a str); 7] {
    [
        (
            TYPED_ERROR_CONVERGENCE_PARENT,
            sources.typed_error_parent.as_str(),
        ),
        (
            TYPED_ERROR_ASSET_LOADERS_PARENT,
            sources.asset_loaders_parent.as_str(),
        ),
        (
            TYPED_ERROR_ASSET_RECORDS_PARENT,
            sources.asset_records_parent.as_str(),
        ),
        (
            TYPED_ERROR_SCENE_WORLD_PARENT,
            sources.scene_world_parent.as_str(),
        ),
        (
            TYPED_ERROR_SCRIPT_HOST_PARENT,
            sources.script_host_parent.as_str(),
        ),
        (
            TYPED_ERROR_SHADER_PREWARM_CLI_PARENT,
            sources.shader_prewarm_cli_parent.as_str(),
        ),
        (
            TYPED_ERROR_UI_INPUT_PARENT,
            sources.ui_input_parent.as_str(),
        ),
    ]
}

pub(super) fn typed_error_convergence_mount_child_sources() -> Vec<(&'static str, String)> {
    TYPED_ERROR_CONVERGENCE_MOUNT_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn typed_error_convergence_mount_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in typed_error_convergence_mount_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}
