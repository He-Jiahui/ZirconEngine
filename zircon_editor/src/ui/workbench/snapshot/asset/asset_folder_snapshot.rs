#[derive(Clone, Debug, Default)]
pub struct AssetFolderSnapshot {
    pub folder_id: String,
    pub parent_folder_id: Option<String>,
    pub display_name: String,
    pub recursive_asset_count: usize,
    pub depth: usize,
    pub selected: bool,
}
