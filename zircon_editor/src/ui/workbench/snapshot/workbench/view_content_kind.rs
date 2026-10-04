#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 内建内容的布局/呈现/关闭策略；真实identity仍由descriptor ID与instance ID持有。
pub enum ViewContentKind {
    Welcome,
    Project,
    Hierarchy,
    Inspector,
    Scene,
    Game,
    Assets,
    Console,
    PrefabEditor,
    AssetBrowser,
    UiAssetEditor,
    UiComponentShowcase,
    AnimationSequenceEditor,
    AnimationGraphEditor,
    RuntimeDiagnostics,
    PerformanceTimeline,
    ModulePlugins,
    BuildExport,
    GeneratedBottom,
    Placeholder,
}
