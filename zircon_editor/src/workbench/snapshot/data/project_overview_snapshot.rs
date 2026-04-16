#[derive(Clone, Debug, Default)]
pub struct ProjectOverviewSnapshot {
    pub project_name: String,
    pub project_root: String,
    pub assets_root: String,
    pub library_root: String,
    pub default_scene_uri: String,
    pub catalog_revision: u64,
    pub folder_count: usize,
    pub asset_count: usize,
}
