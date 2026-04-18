#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssetChangeKind {
    Added,
    Modified,
    Removed,
    Renamed,
}
