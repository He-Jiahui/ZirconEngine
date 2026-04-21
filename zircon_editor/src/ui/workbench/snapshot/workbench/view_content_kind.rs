#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    AnimationSequenceEditor,
    AnimationGraphEditor,
    Placeholder,
}
