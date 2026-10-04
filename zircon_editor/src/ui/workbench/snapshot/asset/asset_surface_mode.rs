#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// workspace投影所属产品面；Activity与Explorer可共享资产代次，但各自保留显示模式和utility状态。
pub enum AssetSurfaceMode {
    #[default]
    Activity,
    Explorer,
}
