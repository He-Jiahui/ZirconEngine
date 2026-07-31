use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::core::math::{Vec2, Vec3, Vec4};
use crate::core::resource::ResourceId as AssetId;

use super::{RGBA16F_TEXEL_SIZE_BYTES, ShL2Rgb};

pub const LIGHTMAP_CONSUME_CONTRACT_VERSION: u32 = 1;
pub const LIGHTMAP_SCENE_SNAPSHOT_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LightmapAtlasFormat {
    Rgba16Float,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LightmapAtlasDescriptor {
    pub page_size: u32,
    pub page_count: u32,
    pub format: LightmapAtlasFormat,
}

impl LightmapAtlasDescriptor {
    pub fn validate(&self) -> Result<(), LightmapContractValidationError> {
        if self.page_size == 0 || self.page_count == 0 {
            return Err(LightmapContractValidationError::InvalidAtlasDescriptor);
        }
        Ok(())
    }

    fn page_payload_size(&self) -> Result<usize, LightmapContractValidationError> {
        let edge = self.page_size as usize;
        edge.checked_mul(edge)
            .and_then(|texels| texels.checked_mul(RGBA16F_TEXEL_SIZE_BYTES))
            .ok_or(LightmapContractValidationError::AtlasPayloadSizeOverflow)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct LightmapInstanceSlot {
    pub atlas_page: u32,
    pub uv_rect: Vec4,
}

impl LightmapInstanceSlot {
    pub fn validate(&self) -> Result<(), LightmapContractValidationError> {
        if !self.uv_rect.is_finite() {
            return Err(LightmapContractValidationError::InvalidUvRect);
        }

        let scale = Vec2::new(self.uv_rect.x, self.uv_rect.y);
        let offset = Vec2::new(self.uv_rect.z, self.uv_rect.w);
        if scale.min_element() <= 0.0
            || offset.min_element() < 0.0
            || (scale + offset).max_element() > 1.0
        {
            return Err(LightmapContractValidationError::InvalidUvRect);
        }
        Ok(())
    }

    pub fn transform_uv2(&self, uv2: Vec2) -> Result<Vec2, LightmapContractValidationError> {
        self.validate()?;
        validate_unit_uv(uv2)?;
        let scale = Vec2::new(self.uv_rect.x, self.uv_rect.y);
        let offset = Vec2::new(self.uv_rect.z, self.uv_rect.w);
        Ok(uv2 * scale + offset)
    }

    pub fn inverse_transform_uv2(
        &self,
        atlas_uv: Vec2,
    ) -> Result<Vec2, LightmapContractValidationError> {
        self.validate()?;
        validate_unit_uv(atlas_uv)?;
        let scale = Vec2::new(self.uv_rect.x, self.uv_rect.y);
        let offset = Vec2::new(self.uv_rect.z, self.uv_rect.w);
        let uv2 = (atlas_uv - offset) / scale;
        validate_unit_uv(uv2)?;
        Ok(uv2)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LightmapConsumeContract {
    pub contract_version: u32,
    pub light_set_generation: u64,
    pub atlas: AssetId,
    pub atlas_descriptor: LightmapAtlasDescriptor,
    pub slots: Vec<(u64, LightmapInstanceSlot)>,
}

impl LightmapConsumeContract {
    pub fn new(
        light_set_generation: u64,
        atlas: AssetId,
        atlas_descriptor: LightmapAtlasDescriptor,
        slots: Vec<(u64, LightmapInstanceSlot)>,
    ) -> Self {
        Self {
            contract_version: LIGHTMAP_CONSUME_CONTRACT_VERSION,
            light_set_generation,
            atlas,
            atlas_descriptor,
            slots,
        }
    }

    pub fn validate(&self) -> Result<(), LightmapContractValidationError> {
        validate_contract_version(self.contract_version)?;
        validate_generation(self.light_set_generation)?;
        self.atlas_descriptor.validate()?;

        let mut instance_ids = HashSet::with_capacity(self.slots.len());
        for (instance_id, slot) in &self.slots {
            if !instance_ids.insert(*instance_id) {
                return Err(LightmapContractValidationError::DuplicateInstanceId {
                    instance_id: *instance_id,
                });
            }
            slot.validate()?;
            if slot.atlas_page >= self.atlas_descriptor.page_count {
                return Err(LightmapContractValidationError::AtlasPageOutOfRange {
                    page: slot.atlas_page,
                    page_count: self.atlas_descriptor.page_count,
                });
            }
        }
        Ok(())
    }

    pub fn slot_for_instance(&self, instance_id: u64) -> Option<LightmapInstanceSlot> {
        self.slots
            .iter()
            .find_map(|(candidate, slot)| (*candidate == instance_id).then_some(*slot))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LightProbeGridData {
    pub light_set_generation: u64,
    pub bounds_min: Vec3,
    pub cell_size: Vec3,
    pub dims: [u32; 3],
    pub sh: Vec<ShL2Rgb>,
}

impl LightProbeGridData {
    pub fn validate(&self) -> Result<(), LightmapContractValidationError> {
        validate_generation(self.light_set_generation)?;
        if !self.bounds_min.is_finite()
            || !self.cell_size.is_finite()
            || self.cell_size.min_element() <= 0.0
        {
            return Err(LightmapContractValidationError::InvalidProbeGridBounds);
        }
        if self.dims.contains(&0) {
            return Err(
                LightmapContractValidationError::InvalidProbeGridDimensions { dims: self.dims },
            );
        }

        let expected_count = self
            .dims
            .into_iter()
            .try_fold(1usize, |count, dimension| {
                count.checked_mul(dimension as usize)
            })
            .ok_or(LightmapContractValidationError::ProbeGridSizeOverflow)?;
        if self.sh.len() != expected_count {
            return Err(LightmapContractValidationError::ProbeCoefficientCount {
                expected: expected_count,
                actual: self.sh.len(),
            });
        }
        if self.sh.iter().any(|coefficients| !coefficients.is_finite()) {
            return Err(LightmapContractValidationError::NonFiniteProbeCoefficient);
        }
        Ok(())
    }

    pub fn sample_trilinear(
        &self,
        world_position: Vec3,
    ) -> Result<ShL2Rgb, LightmapContractValidationError> {
        self.validate()?;
        if !world_position.is_finite() {
            return Err(LightmapContractValidationError::ProbeSampleOutOfBounds);
        }

        let grid_position = (world_position - self.bounds_min) / self.cell_size;
        let (x0, x1, tx) = probe_axis_sample(grid_position.x, self.dims[0])?;
        let (y0, y1, ty) = probe_axis_sample(grid_position.y, self.dims[1])?;
        let (z0, z1, tz) = probe_axis_sample(grid_position.z, self.dims[2])?;
        let mut coefficients = [Vec3::ZERO; super::SH_L2_RGB_COEFFICIENT_COUNT];

        for (z, wz) in [(z0, 1.0 - tz), (z1, tz)] {
            for (y, wy) in [(y0, 1.0 - ty), (y1, ty)] {
                for (x, wx) in [(x0, 1.0 - tx), (x1, tx)] {
                    let weight = wx * wy * wz;
                    let sample = &self.sh[probe_grid_index(self.dims, x, y, z)?];
                    for (result, source) in coefficients.iter_mut().zip(sample.coefficients()) {
                        *result += *source * weight;
                    }
                }
            }
        }

        Ok(ShL2Rgb(coefficients))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LightmapBakeSceneSnapshot {
    pub format_version: u32,
    pub content_hash: [u8; 32],
    pub payload: Vec<u8>,
}

impl LightmapBakeSceneSnapshot {
    pub fn validate(&self) -> Result<(), LightmapContractValidationError> {
        if self.format_version != LIGHTMAP_SCENE_SNAPSHOT_VERSION {
            return Err(
                LightmapContractValidationError::UnsupportedSceneSnapshotVersion {
                    expected: LIGHTMAP_SCENE_SNAPSHOT_VERSION,
                    actual: self.format_version,
                },
            );
        }
        if self.payload.is_empty() || self.content_hash.iter().all(|byte| *byte == 0) {
            return Err(LightmapContractValidationError::InvalidSceneSnapshot);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LightmapAtlasBudget {
    pub page_size: u32,
    pub max_pages: u32,
}

impl LightmapAtlasBudget {
    pub fn validate(&self) -> Result<(), LightmapContractValidationError> {
        if self.page_size == 0 || self.max_pages == 0 {
            return Err(LightmapContractValidationError::InvalidAtlasBudget);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LightmapBakeRequest {
    pub contract_version: u32,
    pub request_id: u64,
    pub scene_revision: u64,
    pub light_set_generation: u64,
    pub static_instance_ids: Vec<u64>,
    pub scene_snapshot: LightmapBakeSceneSnapshot,
    pub atlas_budget: LightmapAtlasBudget,
    pub texel_density: f32,
    pub probe_bounds_min: Vec3,
    pub probe_bounds_max: Vec3,
    pub probe_cell_size: Vec3,
}

impl LightmapBakeRequest {
    pub fn validate(&self) -> Result<(), LightmapContractValidationError> {
        validate_contract_version(self.contract_version)?;
        validate_generation(self.light_set_generation)?;
        if self.request_id == 0 || self.scene_revision == 0 {
            return Err(LightmapContractValidationError::InvalidBakeIdentity);
        }
        self.scene_snapshot.validate()?;
        self.atlas_budget.validate()?;
        if !self.texel_density.is_finite() || self.texel_density <= 0.0 {
            return Err(LightmapContractValidationError::InvalidTexelDensity);
        }
        if !self.probe_bounds_min.is_finite()
            || !self.probe_bounds_max.is_finite()
            || !self.probe_cell_size.is_finite()
            || self.probe_cell_size.min_element() <= 0.0
            || (self.probe_bounds_max - self.probe_bounds_min).min_element() <= 0.0
        {
            return Err(LightmapContractValidationError::InvalidProbeGridBounds);
        }

        let mut instance_ids = HashSet::with_capacity(self.static_instance_ids.len());
        for instance_id in &self.static_instance_ids {
            if !instance_ids.insert(*instance_id) {
                return Err(LightmapContractValidationError::DuplicateInstanceId {
                    instance_id: *instance_id,
                });
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LightmapAtlasPage {
    pub page_index: u32,
    pub texels_rgba16f_le: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LightmapBakeOutput {
    pub contract_version: u32,
    pub request_id: u64,
    pub scene_revision: u64,
    pub light_set_generation: u64,
    pub atlas: LightmapAtlasDescriptor,
    pub atlas_pages: Vec<LightmapAtlasPage>,
    pub slots: Vec<(u64, LightmapInstanceSlot)>,
    pub probe_grid: Option<LightProbeGridData>,
}

impl LightmapBakeOutput {
    pub fn validate(&self) -> Result<(), LightmapContractValidationError> {
        validate_contract_version(self.contract_version)?;
        validate_generation(self.light_set_generation)?;
        if self.request_id == 0 || self.scene_revision == 0 {
            return Err(LightmapContractValidationError::InvalidBakeIdentity);
        }
        self.atlas.validate()?;
        if self.atlas_pages.len() != self.atlas.page_count as usize {
            return Err(LightmapContractValidationError::AtlasPageCountMismatch {
                expected: self.atlas.page_count,
                actual: self.atlas_pages.len(),
            });
        }

        let expected_page_bytes = self.atlas.page_payload_size()?;
        let mut page_indices = HashSet::with_capacity(self.atlas_pages.len());
        for page in &self.atlas_pages {
            if page.page_index >= self.atlas.page_count || !page_indices.insert(page.page_index) {
                return Err(LightmapContractValidationError::InvalidAtlasPageIndex {
                    page: page.page_index,
                });
            }
            if page.texels_rgba16f_le.len() != expected_page_bytes {
                return Err(LightmapContractValidationError::AtlasPagePayloadSize {
                    page: page.page_index,
                    expected: expected_page_bytes,
                    actual: page.texels_rgba16f_le.len(),
                });
            }
        }

        validate_slots(&self.slots, self.atlas.page_count)?;
        if self.atlas.format != LightmapAtlasFormat::Rgba16Float {
            return Err(LightmapContractValidationError::InvalidAtlasDescriptor);
        }
        if let Some(probe_grid) = &self.probe_grid {
            probe_grid.validate()?;
            if probe_grid.light_set_generation != self.light_set_generation {
                return Err(LightmapContractValidationError::GenerationMismatch);
            }
        }
        Ok(())
    }

    pub fn validate_against(
        &self,
        request: &LightmapBakeRequest,
    ) -> Result<(), LightmapContractValidationError> {
        request.validate()?;
        self.validate()?;
        if self.contract_version != request.contract_version
            || self.request_id != request.request_id
            || self.scene_revision != request.scene_revision
            || self.light_set_generation != request.light_set_generation
        {
            return Err(LightmapContractValidationError::BakeRequestMismatch);
        }
        if self.atlas.page_size != request.atlas_budget.page_size
            || self.atlas.page_count > request.atlas_budget.max_pages
        {
            return Err(LightmapContractValidationError::AtlasBudgetExceeded);
        }

        let requested_instances: HashSet<_> = request.static_instance_ids.iter().copied().collect();
        if let Some((instance_id, _)) = self
            .slots
            .iter()
            .find(|(instance_id, _)| !requested_instances.contains(instance_id))
        {
            return Err(LightmapContractValidationError::UnexpectedBakedInstanceId {
                instance_id: *instance_id,
            });
        }
        Ok(())
    }

    pub fn into_consume_contract(
        self,
        atlas_asset: AssetId,
    ) -> Result<
        (LightmapConsumeContract, Option<LightProbeGridData>),
        LightmapContractValidationError,
    > {
        self.validate()?;
        let contract = LightmapConsumeContract::new(
            self.light_set_generation,
            atlas_asset,
            self.atlas,
            self.slots,
        );
        contract.validate()?;
        Ok((contract, self.probe_grid))
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum LightmapContractValidationError {
    #[error("unsupported lightmap contract version {actual}; expected {expected}")]
    UnsupportedContractVersion { expected: u32, actual: u32 },
    #[error("light set generation must be non-zero")]
    MissingLightSetGeneration,
    #[error("lightmap slot has an invalid UV rectangle")]
    InvalidUvRect,
    #[error("UV coordinate must be finite and inside the unit square")]
    InvalidUvCoordinate,
    #[error("duplicate baked-lighting instance id {instance_id}")]
    DuplicateInstanceId { instance_id: u64 },
    #[error("lightmap atlas descriptor must contain at least one non-empty RGBA16F page")]
    InvalidAtlasDescriptor,
    #[error("lightmap atlas page {page} is outside the {page_count}-page array")]
    AtlasPageOutOfRange { page: u32, page_count: u32 },
    #[error("lightmap atlas payload size overflowed the address space")]
    AtlasPayloadSizeOverflow,
    #[error("unsupported lightmap scene snapshot version {actual}; expected {expected}")]
    UnsupportedSceneSnapshotVersion { expected: u32, actual: u32 },
    #[error("lightmap scene snapshot payload or content hash is invalid")]
    InvalidSceneSnapshot,
    #[error("lightmap bake request id and scene revision must be non-zero")]
    InvalidBakeIdentity,
    #[error("lightmap atlas budget must contain a non-zero page size and page count")]
    InvalidAtlasBudget,
    #[error("lightmap texel density must be finite and positive")]
    InvalidTexelDensity,
    #[error("lightmap output contains {actual} atlas pages; expected {expected}")]
    AtlasPageCountMismatch { expected: u32, actual: usize },
    #[error("lightmap output has duplicate or out-of-range atlas page {page}")]
    InvalidAtlasPageIndex { page: u32 },
    #[error("lightmap atlas page {page} contains {actual} bytes; expected {expected}")]
    AtlasPagePayloadSize {
        page: u32,
        expected: usize,
        actual: usize,
    },
    #[error("lightmap bake output does not belong to the supplied request")]
    BakeRequestMismatch,
    #[error("lightmap bake output exceeds the request atlas budget")]
    AtlasBudgetExceeded,
    #[error("lightmap bake output contains unrequested instance {instance_id}")]
    UnexpectedBakedInstanceId { instance_id: u64 },
    #[error("probe grid bounds or cell size are invalid")]
    InvalidProbeGridBounds,
    #[error("probe grid dimensions are invalid: {dims:?}")]
    InvalidProbeGridDimensions { dims: [u32; 3] },
    #[error("probe grid dimensions overflow the addressable coefficient count")]
    ProbeGridSizeOverflow,
    #[error("probe grid requires {expected} SH entries but contains {actual}")]
    ProbeCoefficientCount { expected: usize, actual: usize },
    #[error("probe grid contains a non-finite SH coefficient")]
    NonFiniteProbeCoefficient,
    #[error("probe sample position is outside the grid")]
    ProbeSampleOutOfBounds,
    #[error("baked-lighting generations do not match")]
    GenerationMismatch,
}

fn validate_contract_version(version: u32) -> Result<(), LightmapContractValidationError> {
    if version != LIGHTMAP_CONSUME_CONTRACT_VERSION {
        return Err(
            LightmapContractValidationError::UnsupportedContractVersion {
                expected: LIGHTMAP_CONSUME_CONTRACT_VERSION,
                actual: version,
            },
        );
    }
    Ok(())
}

fn validate_generation(generation: u64) -> Result<(), LightmapContractValidationError> {
    if generation == 0 {
        return Err(LightmapContractValidationError::MissingLightSetGeneration);
    }
    Ok(())
}

fn validate_unit_uv(uv: Vec2) -> Result<(), LightmapContractValidationError> {
    if !uv.is_finite() || uv.min_element() < 0.0 || uv.max_element() > 1.0 {
        return Err(LightmapContractValidationError::InvalidUvCoordinate);
    }
    Ok(())
}

fn validate_slots(
    slots: &[(u64, LightmapInstanceSlot)],
    page_count: u32,
) -> Result<(), LightmapContractValidationError> {
    let mut instance_ids = HashSet::with_capacity(slots.len());
    for (instance_id, slot) in slots {
        if !instance_ids.insert(*instance_id) {
            return Err(LightmapContractValidationError::DuplicateInstanceId {
                instance_id: *instance_id,
            });
        }
        slot.validate()?;
        if slot.atlas_page >= page_count {
            return Err(LightmapContractValidationError::AtlasPageOutOfRange {
                page: slot.atlas_page,
                page_count,
            });
        }
    }
    Ok(())
}

fn probe_axis_sample(
    coordinate: f32,
    dimension: u32,
) -> Result<(u32, u32, f32), LightmapContractValidationError> {
    let maximum = dimension.saturating_sub(1) as f32;
    if !coordinate.is_finite() || coordinate < 0.0 || coordinate > maximum {
        return Err(LightmapContractValidationError::ProbeSampleOutOfBounds);
    }
    let lower = coordinate.floor() as u32;
    let upper = lower.saturating_add(1).min(dimension - 1);
    let fraction = if lower == upper {
        0.0
    } else {
        coordinate - lower as f32
    };
    Ok((lower, upper, fraction))
}

fn probe_grid_index(
    dims: [u32; 3],
    x: u32,
    y: u32,
    z: u32,
) -> Result<usize, LightmapContractValidationError> {
    let width = dims[0] as usize;
    let height = dims[1] as usize;
    let row = height
        .checked_mul(z as usize)
        .and_then(|offset| offset.checked_add(y as usize))
        .ok_or(LightmapContractValidationError::ProbeGridSizeOverflow)?;
    width
        .checked_mul(row)
        .and_then(|offset| offset.checked_add(x as usize))
        .ok_or(LightmapContractValidationError::ProbeGridSizeOverflow)
}

#[cfg(test)]
mod tests;
