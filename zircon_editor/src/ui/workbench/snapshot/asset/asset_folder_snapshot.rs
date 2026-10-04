#[derive(Clone, Debug, Default)]
/// 目录来源树/内容行的展示快照；身份与父关系用folder_id，display_name只用于呈现。
pub struct AssetFolderSnapshot {
    pub folder_id: String,
    pub parent_folder_id: Option<String>,
    pub display_name: String,
    pub recursive_asset_count: usize,
    pub depth: usize,
    pub selected: bool,
}
