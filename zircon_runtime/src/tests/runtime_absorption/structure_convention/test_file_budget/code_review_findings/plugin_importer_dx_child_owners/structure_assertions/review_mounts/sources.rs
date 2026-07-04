use super::super::super::super::super::*;
use super::*;

pub(super) fn plugin_importer_dx_review_mount_sources() -> PluginImporterDxReviewMountSources {
    PluginImporterDxReviewMountSources {
        structure_assertions_child: read_runtime_src(PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_CHILD),
        review_mounts_child: read_runtime_src(PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_CHILD),
        paths_child: read_runtime_src(PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_PATHS_CHILD),
        sources_child: read_runtime_src(PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_SOURCES_CHILD),
        parent_mounts_child: read_runtime_src(PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_PARENT_MOUNTS_CHILD),
        review_children_child: read_runtime_src(
            PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_REVIEW_CHILDREN_CHILD,
        ),
        budgets_child: read_runtime_src(PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_BUDGETS_CHILD),
        status_mirrors_child: read_runtime_src(
            PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_STATUS_MIRRORS_CHILD,
        ),
        plugin_importer_dx: read_runtime_src(paths::PLUGIN_IMPORTER_DX_REVIEW_SOURCE_PATH),
        plugin_importer_dx_d10: read_runtime_src(paths::PLUGIN_IMPORTER_DX_D10_SOURCE_PATH),
        plugin_importer_dx_d1: read_runtime_src(paths::PLUGIN_IMPORTER_DX_D1_SOURCE_PATH),
        plugin_importer_dx_d11: read_runtime_src(paths::PLUGIN_IMPORTER_DX_D11_SOURCE_PATH),
        plugin_importer_dx_d12: read_runtime_src(paths::PLUGIN_IMPORTER_DX_D12_SOURCE_PATH),
        plugin_importer_dx_d5: read_runtime_src(paths::PLUGIN_IMPORTER_DX_D5_SOURCE_PATH),
        plugin_importer_dx_d6: read_runtime_src(paths::PLUGIN_IMPORTER_DX_D6_SOURCE_PATH),
        plugin_importer_dx_d8: read_runtime_src(paths::PLUGIN_IMPORTER_DX_D8_SOURCE_PATH),
        plugin_importer_dx_d9: read_runtime_src(paths::PLUGIN_IMPORTER_DX_D9_SOURCE_PATH),
    }
}
