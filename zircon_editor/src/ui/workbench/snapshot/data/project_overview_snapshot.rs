#[derive(Clone, Debug, Default)]
/// source catalog统计；revision与资产面包含预览/资源更新的发布代次不同。
pub struct ProjectOverviewSnapshot {
    pub project_name: String,
    pub project_root: String,
    pub assets_root: String,
    pub cache_root: String,
    pub default_scene_uri: String,
    pub catalog_revision: u64,
    pub folder_count: usize,
    pub asset_count: usize,
}
