#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// 选中资产详情面的状态；browser覆盖四种，活动面当前只呈现Preview/References。
pub enum AssetUtilityTab {
    #[default]
    Preview,
    References,
    Metadata,
    Plugins,
}
