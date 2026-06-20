use crate::ui::retained_host::primitives::{ModelRc, SharedString};

use super::super::TemplatePaneNodeData;

#[derive(Clone, Default)]
pub(crate) struct ProjectOverviewData {
    pub project_name: SharedString,
    pub project_root: SharedString,
    pub assets_root: SharedString,
    pub library_root: SharedString,
    pub default_scene_uri: SharedString,
    pub catalog_revision: SharedString,
    pub folder_count: SharedString,
    pub asset_count: SharedString,
}

#[derive(Clone, Default)]
pub(crate) struct ConsolePaneData {
    pub nodes: ModelRc<TemplatePaneNodeData>,
    pub status_text: SharedString,
}

#[derive(Clone, Default)]
pub(crate) struct AssetsActivityPaneData {
    pub nodes: ModelRc<TemplatePaneNodeData>,
}

#[derive(Clone, Default)]
pub(crate) struct AssetBrowserPaneData {
    pub nodes: ModelRc<TemplatePaneNodeData>,
}

#[derive(Clone, Default)]
pub(crate) struct ProjectOverviewPaneData {
    pub nodes: ModelRc<TemplatePaneNodeData>,
}

#[derive(Clone, Default)]
pub(crate) struct GeneratedBottomPaneData {
    pub nodes: ModelRc<TemplatePaneNodeData>,
    pub status: SharedString,
}
