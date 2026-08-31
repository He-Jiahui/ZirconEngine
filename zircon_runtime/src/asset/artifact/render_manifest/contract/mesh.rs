use std::ops::Range;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::core::framework::render::RenderMeshTopology;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderArtifactMeshVertexFormat {
    StaticMeshV1,
}

impl RenderArtifactMeshVertexFormat {
    pub const fn stride(self) -> u32 {
        match self {
            Self::StaticMeshV1 => 96,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderArtifactMeshIndexFormat {
    Uint32,
}

impl RenderArtifactMeshIndexFormat {
    pub const fn byte_width(self) -> u32 {
        match self {
            Self::Uint32 => 4,
        }
    }

    pub const fn alignment(self) -> u64 {
        self.byte_width() as u64
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderArtifactMeshBounds {
    min_bits: [u32; 3],
    max_bits: [u32; 3],
}

impl RenderArtifactMeshBounds {
    pub fn from_min_max(min: [f32; 3], max: [f32; 3]) -> Self {
        Self {
            min_bits: min.map(f32::to_bits),
            max_bits: max.map(f32::to_bits),
        }
    }

    pub fn min(self) -> [f32; 3] {
        self.min_bits.map(f32::from_bits)
    }

    pub fn max(self) -> [f32; 3] {
        self.max_bits.map(f32::from_bits)
    }

    pub(in crate::asset::artifact::render_manifest) fn is_finite_canonical(self) -> bool {
        let min = self.min();
        let max = self.max();
        (0..3).all(|axis| min[axis].is_finite() && max[axis].is_finite() && min[axis] <= max[axis])
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderArtifactMeshLodLayout {
    lod: u16,
    topology: RenderMeshTopology,
    vertex_count: u32,
    index_count: u32,
    index_offset: u64,
    bounds: RenderArtifactMeshBounds,
}

impl RenderArtifactMeshLodLayout {
    pub fn new(
        lod: u16,
        topology: RenderMeshTopology,
        vertex_count: u32,
        index_count: u32,
        index_offset: u64,
        bounds: RenderArtifactMeshBounds,
    ) -> Self {
        Self {
            lod,
            topology,
            vertex_count,
            index_count,
            index_offset,
            bounds,
        }
    }

    pub const fn lod(&self) -> u16 {
        self.lod
    }

    pub const fn topology(&self) -> RenderMeshTopology {
        self.topology
    }

    pub const fn vertex_count(&self) -> u32 {
        self.vertex_count
    }

    pub const fn index_count(&self) -> u32 {
        self.index_count
    }

    pub const fn index_offset(&self) -> u64 {
        self.index_offset
    }

    pub const fn bounds(&self) -> RenderArtifactMeshBounds {
        self.bounds
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderArtifactMeshLayout {
    platform_format: Arc<str>,
    vertex_format: RenderArtifactMeshVertexFormat,
    index_format: RenderArtifactMeshIndexFormat,
    bootstrap_first_lod: u16,
    lods: Arc<[RenderArtifactMeshLodLayout]>,
}

impl RenderArtifactMeshLayout {
    pub fn new(
        platform_format: Arc<str>,
        vertex_format: RenderArtifactMeshVertexFormat,
        index_format: RenderArtifactMeshIndexFormat,
        bootstrap_first_lod: u16,
        mut lods: Vec<RenderArtifactMeshLodLayout>,
    ) -> Self {
        lods.sort_unstable_by_key(RenderArtifactMeshLodLayout::lod);
        Self {
            platform_format,
            vertex_format,
            index_format,
            bootstrap_first_lod,
            lods: lods.into(),
        }
    }

    pub fn platform_format(&self) -> &str {
        self.platform_format.as_ref()
    }

    pub const fn vertex_format(&self) -> RenderArtifactMeshVertexFormat {
        self.vertex_format
    }

    pub const fn index_format(&self) -> RenderArtifactMeshIndexFormat {
        self.index_format
    }

    pub const fn bootstrap_first_lod(&self) -> u16 {
        self.bootstrap_first_lod
    }

    pub fn lods(&self) -> &[RenderArtifactMeshLodLayout] {
        self.lods.as_ref()
    }

    pub fn lod_count(&self) -> usize {
        self.lods.len()
    }

    pub fn lod(&self, lod: u16) -> Option<&RenderArtifactMeshLodLayout> {
        self.lods
            .binary_search_by_key(&lod, RenderArtifactMeshLodLayout::lod)
            .ok()
            .and_then(|index| self.lods.get(index))
    }

    pub fn subresource_layout(&self, lod: u16) -> Option<RenderArtifactMeshLodUploadLayout> {
        let layout = self.lod(lod)?;
        let vertex_bytes =
            u64::from(layout.vertex_count()).checked_mul(u64::from(self.vertex_format.stride()))?;
        let index_bytes = u64::from(layout.index_count())
            .checked_mul(u64::from(self.index_format.byte_width()))?;
        let decoded_bytes = layout.index_offset().checked_add(index_bytes)?;
        Some(RenderArtifactMeshLodUploadLayout {
            lod,
            topology: layout.topology(),
            vertex_format: self.vertex_format,
            index_format: self.index_format,
            vertex_count: layout.vertex_count(),
            index_count: layout.index_count(),
            vertex_bytes,
            index_offset: layout.index_offset(),
            index_bytes,
            decoded_bytes,
            bounds: layout.bounds(),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderArtifactMeshLodUploadLayout {
    lod: u16,
    topology: RenderMeshTopology,
    vertex_format: RenderArtifactMeshVertexFormat,
    index_format: RenderArtifactMeshIndexFormat,
    vertex_count: u32,
    index_count: u32,
    vertex_bytes: u64,
    index_offset: u64,
    index_bytes: u64,
    decoded_bytes: u64,
    bounds: RenderArtifactMeshBounds,
}

impl RenderArtifactMeshLodUploadLayout {
    pub const fn lod(self) -> u16 {
        self.lod
    }

    pub const fn topology(self) -> RenderMeshTopology {
        self.topology
    }

    pub const fn vertex_format(self) -> RenderArtifactMeshVertexFormat {
        self.vertex_format
    }

    pub const fn index_format(self) -> RenderArtifactMeshIndexFormat {
        self.index_format
    }

    pub const fn vertex_count(self) -> u32 {
        self.vertex_count
    }

    pub const fn index_count(self) -> u32 {
        self.index_count
    }

    pub fn vertex_range(self) -> Range<u64> {
        0..self.vertex_bytes
    }

    pub fn index_range(self) -> Range<u64> {
        self.index_offset..self.index_offset + self.index_bytes
    }

    pub const fn decoded_bytes(self) -> u64 {
        self.decoded_bytes
    }

    pub const fn bounds(self) -> RenderArtifactMeshBounds {
        self.bounds
    }
}
