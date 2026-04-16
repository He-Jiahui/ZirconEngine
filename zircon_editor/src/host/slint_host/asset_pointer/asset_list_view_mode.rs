#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AssetListViewMode {
    List,
    Thumbnail,
}

impl From<crate::workbench::snapshot::AssetViewMode> for AssetListViewMode {
    fn from(value: crate::workbench::snapshot::AssetViewMode) -> Self {
        match value {
            crate::workbench::snapshot::AssetViewMode::List => Self::List,
            crate::workbench::snapshot::AssetViewMode::Thumbnail => Self::Thumbnail,
        }
    }
}
