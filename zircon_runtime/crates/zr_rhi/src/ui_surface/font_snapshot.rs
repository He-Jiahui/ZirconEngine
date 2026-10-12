//! Admitted font bytes and metadata supplied by the existing Text collection owner.
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct UiSurfaceFontSource {
    pub bytes: Arc<[u8]>,
    pub face_index: u32,
    pub raster_sha256: String,
    pub resource_path: Option<String>,
    pub resource_sha256: Option<String>,
    pub families: Vec<String>,
    pub post_script_name: String,
    pub weight: u16,
    pub stretch: u16,
    /// 0 = normal, 1 = italic, 2 = oblique, from the admitted backend metadata.
    pub style: u8,
    pub monospaced: bool,
}

/// Byte allocations retain the exact source lease; no Runtime or call guard is
/// retained by this render packet. A newer generation replaces this whole set.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiSurfaceFontSnapshot {
    pub collection_id: u64,
    pub generation: u64,
    pub sources: Vec<UiSurfaceFontSource>,
    pub generic_families: [String; 5],
    pub admission_error: Option<String>,
}
