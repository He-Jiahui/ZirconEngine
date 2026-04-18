#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AssetUtilityTab {
    #[default]
    Preview,
    References,
    Metadata,
    Plugins,
}
