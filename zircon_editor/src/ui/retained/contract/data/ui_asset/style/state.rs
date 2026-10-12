#[derive(Clone, Default)]
pub(crate) struct UiAssetStyleStateData {
    pub hover: bool,
    pub focus: bool,
    pub pressed: bool,
    pub disabled: bool,
    pub selected: bool,
}
