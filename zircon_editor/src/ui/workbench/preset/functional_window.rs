use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// 默认设计stack的功能窗口身份；与实际原生句柄和视图实例身份分开。
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
/// preset建议的宿主分工，供默认布局及注册表解释；实际停靠仍由布局许可验证。
pub enum EditorWindowDockPolicy {
    MainWorkbench,
    DockedDocument,
    FloatingAllowed,
    DrawerBacked,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 默认功能窗口组合；公开列表允许重排，查找不能依赖固定位置等于功能身份。
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

    /// 常见默认顺序走预期槽，核验kind后才返回；重排配置按真实kind回退查找。
    pub fn window(
        &self,
        kind: EditorFunctionalWindowKind,
    ) -> Option<&EditorFunctionalWindowPreset> {
        self.windows
            .get(expected_functional_window_index(kind))
            .filter(|window| window.kind == kind)
            .or_else(|| self.windows.iter().find(|window| window.kind == kind))
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

const fn expected_functional_window_index(kind: EditorFunctionalWindowKind) -> usize {
    match kind {
        EditorFunctionalWindowKind::Workbench => 0,
        EditorFunctionalWindowKind::SceneGame => 1,
        EditorFunctionalWindowKind::PrefabEditor => 2,
        EditorFunctionalWindowKind::MaterialEditor => 3,
        EditorFunctionalWindowKind::UiAssetEditor => 4,
        EditorFunctionalWindowKind::AnimationEditor => 5,
        EditorFunctionalWindowKind::AssetBrowser => 6,
        EditorFunctionalWindowKind::Diagnostics => 7,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 功能窗口的主要视图和辅助抽屉集合，记录的是描述符而非已创建实例。
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

#[cfg(test)]
#[path = "tests/functional_window.rs"]
mod tests;
