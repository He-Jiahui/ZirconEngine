use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EditorFunctionalWindowKind {
    Workbench,
    SceneGame,
    PrefabEditor,
    MaterialEditor,
    UiAssetEditor,
    AnimationEditor,
    AssetBrowser,
    Diagnostics,
}

impl EditorFunctionalWindowKind {
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Workbench => "workbench",
            Self::SceneGame => "scene_game",
            Self::PrefabEditor => "prefab_editor",
            Self::MaterialEditor => "material_editor",
            Self::UiAssetEditor => "ui_asset_editor",
            Self::AnimationEditor => "animation_editor",
            Self::AssetBrowser => "asset_browser",
            Self::Diagnostics => "diagnostics",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EditorWindowDockPolicy {
    MainWorkbench,
    DockedDocument,
    FloatingAllowed,
    DrawerBacked,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnrealWindowModelPreset {
    pub windows: Vec<EditorFunctionalWindowPreset>,
    pub workbench_kind: EditorFunctionalWindowKind,
}

impl UnrealWindowModelPreset {
    pub fn new(windows: impl IntoIterator<Item = EditorFunctionalWindowPreset>) -> Self {
        Self {
            windows: windows.into_iter().collect(),
            workbench_kind: EditorFunctionalWindowKind::Workbench,
        }
    }

    pub fn window(
        &self,
        kind: EditorFunctionalWindowKind,
    ) -> Option<&EditorFunctionalWindowPreset> {
        self.windows.iter().find(|window| window.kind == kind)
    }

    pub fn feature_editor_windows(&self) -> impl Iterator<Item = &EditorFunctionalWindowPreset> {
        self.windows
            .iter()
            .filter(|window| window.dock_policy == EditorWindowDockPolicy::FloatingAllowed)
    }

    pub fn drawer_backed_windows(&self) -> impl Iterator<Item = &EditorFunctionalWindowPreset> {
        self.windows
            .iter()
            .filter(|window| window.dock_policy == EditorWindowDockPolicy::DrawerBacked)
    }

    pub fn workbench(&self) -> Option<&EditorFunctionalWindowPreset> {
        self.window(self.workbench_kind)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorFunctionalWindowPreset {
    pub kind: EditorFunctionalWindowKind,
    pub title: String,
    pub dock_policy: EditorWindowDockPolicy,
    pub primary_views: Vec<String>,
    pub drawer_views: Vec<String>,
}

impl EditorFunctionalWindowPreset {
    pub fn new(
        kind: EditorFunctionalWindowKind,
        title: impl Into<String>,
        dock_policy: EditorWindowDockPolicy,
    ) -> Self {
        Self {
            kind,
            title: title.into(),
            dock_policy,
            primary_views: Vec::new(),
            drawer_views: Vec::new(),
        }
    }

    pub fn with_primary_views(mut self, views: impl IntoIterator<Item = &'static str>) -> Self {
        self.primary_views = views.into_iter().map(str::to_string).collect();
        self
    }

    pub fn with_drawer_views(mut self, views: impl IntoIterator<Item = &'static str>) -> Self {
        self.drawer_views = views.into_iter().map(str::to_string).collect();
        self
    }
}
