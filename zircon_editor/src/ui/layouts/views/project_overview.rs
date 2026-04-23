use std::collections::BTreeMap;

use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::view_projection::build_view_template_nodes;
use crate::ui::layouts::windows::workbench_host_window::{
    ProjectOverviewData, ProjectOverviewPaneViewData,
};
use crate::ui::workbench::snapshot::ProjectOverviewSnapshot;
use zircon_runtime::ui::layout::UiSize;

const PROJECT_OVERVIEW_LAYOUT_ASSET_PATH: &str = "/assets/ui/editor/project_overview.ui.toml";
const PROJECT_OVERVIEW_STYLE_ASSET_PATH: &str = "/assets/ui/theme/editor_base.ui.toml";
const PROJECT_OVERVIEW_STYLE_ASSET_ID: &str = "res://ui/theme/editor_base.ui.toml";

pub(crate) fn project_overview_data(snapshot: &ProjectOverviewSnapshot) -> ProjectOverviewData {
    ProjectOverviewData {
        project_name: snapshot.project_name.clone().into(),
        project_root: snapshot.project_root.clone().into(),
        assets_root: snapshot.assets_root.clone().into(),
        library_root: snapshot.library_root.clone().into(),
        default_scene_uri: snapshot.default_scene_uri.clone().into(),
        catalog_revision: snapshot.catalog_revision.to_string().into(),
        folder_count: snapshot.folder_count.to_string().into(),
        asset_count: snapshot.asset_count.to_string().into(),
    }
}

pub(crate) fn project_overview_pane_data(
    snapshot: &ProjectOverviewSnapshot,
    size: UiSize,
) -> ProjectOverviewPaneViewData {
    let mut text_overrides = BTreeMap::new();
    let _ = text_overrides.insert(
        "ProjectOverviewTitleText".to_string(),
        if snapshot.project_name.is_empty() {
            "Directory Project".to_string()
        } else {
            snapshot.project_name.clone()
        },
    );
    let _ = text_overrides.insert(
        "ProjectOverviewPathText".to_string(),
        snapshot.project_root.clone(),
    );
    let _ = text_overrides.insert(
        "ProjectOverviewDefaultSceneValue".to_string(),
        snapshot.default_scene_uri.clone(),
    );
    let _ = text_overrides.insert(
        "ProjectOverviewAssetsRootValue".to_string(),
        snapshot.assets_root.clone(),
    );
    let _ = text_overrides.insert(
        "ProjectOverviewLibraryValue".to_string(),
        format!(
            "Library {} • rev {}",
            snapshot.library_root, snapshot.catalog_revision
        ),
    );
    let _ = text_overrides.insert(
        "ProjectOverviewCatalogSummaryValue".to_string(),
        format!(
            "{} folders • {} assets",
            snapshot.folder_count, snapshot.asset_count
        ),
    );

    ProjectOverviewPaneViewData {
        nodes: model_rc(
            build_view_template_nodes(
                "project_overview.template_projection",
                PROJECT_OVERVIEW_LAYOUT_ASSET_PATH,
                &[(
                    PROJECT_OVERVIEW_STYLE_ASSET_ID,
                    PROJECT_OVERVIEW_STYLE_ASSET_PATH,
                )],
                size,
                &text_overrides,
            )
            .unwrap_or_default(),
        ),
    }
}
