#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AssetListViewMode {
    List,
    Thumbnail,
}

impl From<crate::ui::workbench::snapshot::AssetViewMode> for AssetListViewMode {
    fn from(value: crate::ui::workbench::snapshot::AssetViewMode) -> Self {
        match value {
            crate::ui::workbench::snapshot::AssetViewMode::List => Self::List,
            crate::ui::workbench::snapshot::AssetViewMode::Thumbnail => Self::Thumbnail,
        }
    }
}
