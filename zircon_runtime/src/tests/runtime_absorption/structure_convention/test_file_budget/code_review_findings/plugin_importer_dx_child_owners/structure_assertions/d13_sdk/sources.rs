use super::super::super::super::super::*;
use super::*;

pub(super) fn plugin_importer_d13_sdk_structure_sources() -> PluginImporterD13SdkStructureSources {
    PluginImporterD13SdkStructureSources {
        structure_assertions_child: read_runtime_src(PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_CHILD),
        d13_sdk_child: read_runtime_src(PLUGIN_IMPORTER_D13_STRUCTURE_ASSERTIONS_CHILD),
        paths_child: read_runtime_src(PLUGIN_IMPORTER_D13_PATHS_CHILD),
        sources_child: read_runtime_src(PLUGIN_IMPORTER_D13_SOURCES_CHILD),
        parent_mounts_child: read_runtime_src(PLUGIN_IMPORTER_D13_PARENT_MOUNTS_CHILD),
        review_children_child: read_runtime_src(PLUGIN_IMPORTER_D13_REVIEW_CHILDREN_CHILD),
        budgets_child: read_runtime_src(PLUGIN_IMPORTER_D13_BUDGETS_CHILD),
        status_mirrors_child: read_runtime_src(PLUGIN_IMPORTER_D13_STATUS_MIRRORS_CHILD),
        plugin_importer_dx_d13: read_runtime_src(paths::PLUGIN_IMPORTER_D13_SOURCE_PATH),
        plugin_importer_dx_d13_manifest_parity: read_runtime_src(
            paths::PLUGIN_IMPORTER_D13_MANIFEST_PARITY_SOURCE_PATH,
        ),
        plugin_importer_dx_d13_runtime_crates: read_runtime_src(
            paths::PLUGIN_IMPORTER_D13_RUNTIME_CRATES_SOURCE_PATH,
        ),
        plugin_importer_dx_d13_runtime_exports: read_runtime_src(
            paths::PLUGIN_IMPORTER_D13_RUNTIME_EXPORTS_SOURCE_PATH,
        ),
        plugin_importer_dx_d13_runtime_manifests: read_runtime_src(
            paths::PLUGIN_IMPORTER_D13_RUNTIME_MANIFESTS_SOURCE_PATH,
        ),
    }
}
