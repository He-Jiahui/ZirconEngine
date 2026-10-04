#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// 资产内容的行组织选择；布局、虚拟化及pointer命中须消费同一发布mode。
pub enum AssetViewMode {
    #[default]
    List,
    Thumbnail,
}
