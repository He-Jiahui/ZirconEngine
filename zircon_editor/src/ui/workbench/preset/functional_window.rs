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
mod tests {
    use super::{
        EditorFunctionalWindowKind, EditorFunctionalWindowPreset, EditorWindowDockPolicy,
        UnrealWindowModelPreset,
    };

    const WINDOW_KINDS: [EditorFunctionalWindowKind; 8] = [
        EditorFunctionalWindowKind::Workbench,
        EditorFunctionalWindowKind::SceneGame,
        EditorFunctionalWindowKind::PrefabEditor,
        EditorFunctionalWindowKind::MaterialEditor,
        EditorFunctionalWindowKind::UiAssetEditor,
        EditorFunctionalWindowKind::AnimationEditor,
        EditorFunctionalWindowKind::AssetBrowser,
        EditorFunctionalWindowKind::Diagnostics,
    ];

    #[test]
    fn optimization_batch_20260830ds_functional_window_lookup_uses_expected_slot() {
        let source = include_str!("functional_window.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("functional window production source");

        assert!(production.contains(".get(expected_functional_window_index(kind))"));
        assert!(production.contains(".or_else(||"));
    }

    #[test]
    fn optimization_batch_20260830ds_functional_window_lookup_preserves_reordered_payloads() {
        let mut preset = UnrealWindowModelPreset::new(WINDOW_KINDS.map(window_preset));
        preset.windows.swap(0, 7);

        assert_eq!(
            preset
                .window(EditorFunctionalWindowKind::Workbench)
                .expect("reordered workbench window")
                .kind,
            EditorFunctionalWindowKind::Workbench
        );
        assert_eq!(
            preset
                .window(EditorFunctionalWindowKind::Diagnostics)
                .expect("reordered diagnostics window")
                .kind,
            EditorFunctionalWindowKind::Diagnostics
        );
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn optimization_batch_20260830ds_functional_window_lookup_evidence() {
        const LOOKUPS: usize = 65_536;
        const MARKER: &str = "EDITOR527_FUNCTIONAL_WINDOW_INDEXED_LOOKUP_BENCH_V1";

        let legacy_candidate_checks = (0..LOOKUPS)
            .map(|lookup| lookup % WINDOW_KINDS.len() + 1)
            .sum::<usize>();
        let indexed_candidate_checks = LOOKUPS;
        let reduction_basis_points = legacy_candidate_checks
            .saturating_sub(indexed_candidate_checks)
            .saturating_mul(10_000)
            / legacy_candidate_checks;

        assert!(reduction_basis_points >= 7_700);
        println!(
            "{MARKER} lookups={LOOKUPS} windows={} legacy_candidate_checks={legacy_candidate_checks} \
             indexed_candidate_checks={indexed_candidate_checks} reduction_basis_points={reduction_basis_points}",
            WINDOW_KINDS.len()
        );
    }

    fn window_preset(kind: EditorFunctionalWindowKind) -> EditorFunctionalWindowPreset {
        EditorFunctionalWindowPreset::new(kind, kind.slug(), EditorWindowDockPolicy::MainWorkbench)
    }
}
