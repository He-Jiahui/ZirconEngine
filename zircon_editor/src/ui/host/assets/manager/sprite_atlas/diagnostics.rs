#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SpriteAtlasBuildDiagnostics {
    pub source_count: usize,
    pub packed_count: usize,
    pub atlas_width: u32,
    pub atlas_height: u32,
    pub padding: (u32, u32),
    pub packed_area: u64,
    pub atlas_area: u64,
    pub skipped_sources: Vec<String>,
    pub message: String,
}
