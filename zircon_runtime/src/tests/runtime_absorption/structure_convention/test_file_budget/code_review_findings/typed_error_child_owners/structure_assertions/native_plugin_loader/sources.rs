use super::super::super::super::super::*;
use super::*;

pub(super) struct TypedErrorNativePluginLoaderSources {
    pub(super) structure_assertions_parent: String,
    pub(super) native_structure_child: String,
    pub(super) native_plugin_loader_parent: String,
    pub(super) native_abi_surfaces_parent: String,
    pub(super) native_plugin_descriptor_parent: String,
    pub(super) native_live_host_parent: String,
    pub(super) native_live_host_lifecycle_paths_parent: String,
    pub(super) native_live_host_replay_and_runtime_parent: String,
    pub(super) native_manifest_sources_parent: String,
}

pub(super) fn typed_error_native_plugin_loader_sources() -> TypedErrorNativePluginLoaderSources {
    TypedErrorNativePluginLoaderSources {
        structure_assertions_parent: read_runtime_src(TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD),
        native_structure_child: read_runtime_src(TYPED_ERROR_NATIVE_STRUCTURE_CHILD),
        native_plugin_loader_parent: read_runtime_src(
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader.rs",
        ),
        native_abi_surfaces_parent: read_runtime_src(
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces.rs",
        ),
        native_plugin_descriptor_parent: read_runtime_src(
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces/plugin_descriptor.rs",
        ),
        native_live_host_parent: read_runtime_src(
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host.rs",
        ),
        native_live_host_lifecycle_paths_parent: read_runtime_src(
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host/lifecycle_paths.rs",
        ),
        native_live_host_replay_and_runtime_parent: read_runtime_src(
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host/replay_and_runtime.rs",
        ),
        native_manifest_sources_parent: read_runtime_src(
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/manifest_sources.rs",
        ),
    }
}

pub(super) fn typed_error_native_plugin_loader_child_sources() -> Vec<(&'static str, String)> {
    TYPED_ERROR_NATIVE_STRUCTURE_GUARD_CHILDREN
        .iter()
        .chain(TYPED_ERROR_NATIVE_STRUCTURE_SOURCE_HELPER_CHILDREN.iter())
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn typed_error_native_plugin_loader_child_source_blob() -> String {
    source_blob_from(typed_error_native_plugin_loader_child_sources())
}

pub(super) fn typed_error_native_plugin_loader_source_helper_child_sources(
) -> Vec<(&'static str, String)> {
    TYPED_ERROR_NATIVE_STRUCTURE_SOURCE_HELPER_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn typed_error_native_plugin_loader_source_helper_child_source_blob() -> String {
    source_blob_from(typed_error_native_plugin_loader_source_helper_child_sources())
}

fn source_blob_from(sources: Vec<(&'static str, String)>) -> String {
    let mut blob = String::new();
    for (_, source) in sources {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}
