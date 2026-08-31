pub use super::ibl_bake_recipe::{
    CANONICAL_IBL_BAKE_ROUGHEST_MIP_OFFSET as SOURCE_CUBEMAP_ROUGHEST_MIP,
    CANONICAL_IBL_BAKE_ROUGHNESS_MIP_SCALE as SOURCE_CUBEMAP_ROUGHNESS_MIP_SCALE,
};
use super::{CubemapFace, CANONICAL_IBL_BAKE_RECIPE};
use crate::core::framework::tasks::ParallelSliceExecutor;
use crate::core::math::Real;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

mod irradiance_sh9;
mod mipmap;
mod pmrem;
mod pmrem_layout;
mod projection;
mod rebuild;
mod sampling;

use irradiance_sh9::source_cubemap_irradiance_sh9_from_texels;
pub use irradiance_sh9::{source_cubemap_evaluate_irradiance_sh9, SourceCubemapIrradianceSh9};
pub(super) use pmrem_layout::SourceCubemapPmremLayout;
pub use projection::{
    build_source_cubemap_from_equirect, source_cubemap_face_size_from_equirect_height,
};
pub use rebuild::{
    rebuild_source_cubemap_from_source_mips_with_pmrem_layout_and_parallel_executor_and_timing,
    rebuild_source_cubemap_from_source_mips_with_pmrem_layout_and_timing,
};

pub const SOURCE_CUBEMAP_FACE_COUNT: usize = 6;
pub const SOURCE_CUBEMAP_IRRADIANCE_COEFFICIENT_COUNT: usize = 9;
pub use super::ibl_bake_recipe::CANONICAL_IBL_BAKE_DIFFUSE_SOURCE_FACE_SIZE as SOURCE_CUBEMAP_IRRADIANCE_SOURCE_FACE_SIZE;
pub const SOURCE_CUBEMAP_MIN_FACE_SIZE: u32 = 64;
pub const SOURCE_CUBEMAP_MAX_FACE_SIZE: u32 = 1024;
pub const SOURCE_CUBEMAP_PMREM_FACE_SIZE: u32 = 128;
pub const SOURCE_CUBEMAP_PMREM_MIP_COUNT: u32 = 8;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SourceCubemapPrefilterQuality {
    Fast,
    #[default]
    Normal,
    High,
}

/// Phase attribution for the CPU work that creates one source cubemap.
///
/// These phases are strictly build-time diagnostics. They are not sampled from
/// the runtime rendering path and their sum intentionally excludes caller
/// validation and output serialization. Parallel work-item values count
/// submitted chunks, not worker utilization or completed CPU work.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SourceCubemapBuildTiming {
    equirect_projection: Duration,
    source_mip_build: Duration,
    pmrem_build: Duration,
    sh9_build: Duration,
    equirect_projection_parallel_work_items: u64,
    source_mip_build_parallel_work_items: u64,
    pmrem_build_parallel_work_items: u64,
}

impl SourceCubemapBuildTiming {
    pub const fn equirect_projection(&self) -> Duration {
        self.equirect_projection
    }

    pub const fn source_mip_build(&self) -> Duration {
        self.source_mip_build
    }

    pub const fn pmrem_build(&self) -> Duration {
        self.pmrem_build
    }

    pub const fn sh9_build(&self) -> Duration {
        self.sh9_build
    }

    /// Chunks submitted while projecting the equirectangular image into faces.
    pub const fn equirect_projection_parallel_work_items(&self) -> u64 {
        self.equirect_projection_parallel_work_items
    }

    /// Chunks submitted while constructing filtered source mips.
    pub const fn source_mip_build_parallel_work_items(&self) -> u64 {
        self.source_mip_build_parallel_work_items
    }

    /// Chunks submitted while constructing the independent PMREM result.
    pub const fn pmrem_build_parallel_work_items(&self) -> u64 {
        self.pmrem_build_parallel_work_items
    }

    pub const fn total(&self) -> Duration {
        self.equirect_projection
            .saturating_add(self.source_mip_build)
            .saturating_add(self.pmrem_build)
            .saturating_add(self.sh9_build)
    }
}

/// Counts submitted chunks while preserving the caller-owned executor.
///
/// The counter intentionally observes dispatch shape only. It does not infer
/// that a task ran concurrently or that all available workers were occupied.
pub(super) struct WorkItemCountingParallelSliceExecutor<'a, E> {
    inner: &'a E,
    work_items: AtomicUsize,
}

impl<'a, E> WorkItemCountingParallelSliceExecutor<'a, E> {
    pub(super) fn new(inner: &'a E) -> Self {
        Self {
            inner,
            work_items: AtomicUsize::new(0),
        }
    }

    pub(super) fn submitted_work_items(&self) -> u64 {
        self.work_items.load(Ordering::Relaxed) as u64
    }
}

impl<E> ParallelSliceExecutor for WorkItemCountingParallelSliceExecutor<'_, E>
where
    E: ParallelSliceExecutor,
{
    fn parallel_for<T, F>(&self, items: &mut [T], chunk_size: usize, task: F)
    where
        T: Send,
        F: Fn(&mut [T]) + Send + Sync,
    {
        let chunk_size = chunk_size.max(1);
        self.work_items
            .fetch_add(items.len().div_ceil(chunk_size), Ordering::Relaxed);
        self.inner.parallel_for(items, chunk_size, task);
    }
}

pub(super) struct CubemapFaceMipOutput<'a> {
    pub(super) face: CubemapFace,
    pub(super) texels: &'a mut [[Real; 4]],
}

#[derive(Clone, Debug, PartialEq)]
pub struct SourceCubemapMipChain {
    source_face_size: u32,
    source_mip_count: u32,
    source_texels: Arc<[[Real; 4]]>,
    pmrem_face_size: u32,
    pmrem_mip_count: u32,
    pmrem_texels: Arc<[[Real; 4]]>,
    // This remains tied to the immutable source pyramid even when an artifact
    // replaces the active PMREM and diffuse coefficients.
    source_irradiance_sh9: SourceCubemapIrradianceSh9,
    irradiance_sh9: SourceCubemapIrradianceSh9,
}

impl SourceCubemapMipChain {
    pub fn new(
        source_face_size: u32,
        source_mip_count: u32,
        source_texels: Vec<[Real; 4]>,
        pmrem_face_size: u32,
        pmrem_mip_count: u32,
        pmrem_texels: Vec<[Real; 4]>,
    ) -> Self {
        let source_face_size = source_face_size.max(1);
        let source_mip_count =
            source_mip_count.clamp(1, source_cubemap_mip_count(source_face_size));
        assert_eq!(
            source_texels.len(),
            source_cubemap_sample_count(source_face_size, source_mip_count),
            "source cubemap texel count must match its source layout"
        );
        let irradiance_sh9 = Self::source_irradiance_sh9_from_source_texels(
            &source_texels,
            source_face_size,
            source_mip_count,
        );
        Self::new_with_source_texels_and_irradiance_sh9(
            source_face_size,
            source_mip_count,
            source_texels,
            pmrem_face_size,
            pmrem_mip_count,
            pmrem_texels,
            irradiance_sh9,
        )
    }

    pub(super) fn new_with_source_texels_and_irradiance_sh9(
        source_face_size: u32,
        source_mip_count: u32,
        source_texels: Vec<[Real; 4]>,
        pmrem_face_size: u32,
        pmrem_mip_count: u32,
        pmrem_texels: Vec<[Real; 4]>,
        irradiance_sh9: SourceCubemapIrradianceSh9,
    ) -> Self {
        Self::new_with_source_texels_and_irradiance_sh9_pair(
            source_face_size,
            source_mip_count,
            source_texels,
            pmrem_face_size,
            pmrem_mip_count,
            pmrem_texels,
            irradiance_sh9,
            irradiance_sh9,
        )
    }

    pub(super) fn new_with_source_texels_and_irradiance_sh9_pair(
        source_face_size: u32,
        source_mip_count: u32,
        source_texels: Vec<[Real; 4]>,
        pmrem_face_size: u32,
        pmrem_mip_count: u32,
        pmrem_texels: Vec<[Real; 4]>,
        source_irradiance_sh9: SourceCubemapIrradianceSh9,
        irradiance_sh9: SourceCubemapIrradianceSh9,
    ) -> Self {
        let source_face_size = source_face_size.max(1);
        let source_mip_count =
            source_mip_count.clamp(1, source_cubemap_mip_count(source_face_size));
        let pmrem_face_size = pmrem_face_size.max(1);
        let pmrem_mip_count = pmrem_mip_count.clamp(1, source_cubemap_mip_count(pmrem_face_size));
        assert_eq!(
            source_texels.len(),
            source_cubemap_sample_count(source_face_size, source_mip_count),
            "source cubemap regular texel count must match face size and mip count"
        );
        assert_eq!(
            pmrem_texels.len(),
            source_cubemap_sample_count(pmrem_face_size, pmrem_mip_count),
            "source cubemap PMREM texel count must match its independent layout"
        );
        Self {
            source_face_size,
            source_mip_count,
            source_texels: source_texels.into(),
            pmrem_face_size,
            pmrem_mip_count,
            pmrem_texels: pmrem_texels.into(),
            source_irradiance_sh9,
            irradiance_sh9,
        }
    }

    /// Keeps the immutable captured source pyramid while replacing the
    /// artifact-produced PMREM and irradiance coefficients.
    pub(super) fn with_bake_artifact_pmrem(
        &self,
        pmrem_face_size: u32,
        pmrem_mip_count: u32,
        pmrem_texels: Vec<[Real; 4]>,
        irradiance_sh9: SourceCubemapIrradianceSh9,
    ) -> Self {
        let pmrem_face_size = pmrem_face_size.max(1);
        let pmrem_mip_count = pmrem_mip_count.clamp(1, source_cubemap_mip_count(pmrem_face_size));
        assert_eq!(
            pmrem_texels.len(),
            source_cubemap_sample_count(pmrem_face_size, pmrem_mip_count),
            "source cubemap PMREM texel count must match its independent layout"
        );
        Self {
            source_face_size: self.source_face_size,
            source_mip_count: self.source_mip_count,
            source_texels: Arc::clone(&self.source_texels),
            pmrem_face_size,
            pmrem_mip_count,
            pmrem_texels: pmrem_texels.into(),
            source_irradiance_sh9: self.source_irradiance_sh9,
            irradiance_sh9,
        }
    }

    pub const fn source_face_size(&self) -> u32 {
        self.source_face_size
    }

    pub const fn source_mip_count(&self) -> u32 {
        self.source_mip_count
    }

    pub fn source_texels(&self) -> &[[Real; 4]] {
        &self.source_texels
    }

    pub const fn pmrem_face_size(&self) -> u32 {
        self.pmrem_face_size
    }

    pub const fn pmrem_mip_count(&self) -> u32 {
        self.pmrem_mip_count
    }

    pub fn pmrem_texels(&self) -> &[[Real; 4]] {
        &self.pmrem_texels
    }

    pub fn irradiance_sh9(&self) -> &SourceCubemapIrradianceSh9 {
        &self.irradiance_sh9
    }

    pub fn pmrem_texel(&self, face: CubemapFace, mip_level: u32, x: u32, y: u32) -> [Real; 4] {
        let mip_level = mip_level.min(self.pmrem_mip_count.saturating_sub(1));
        let mip_size = source_cubemap_mip_size(self.pmrem_face_size, mip_level);
        let index = source_cubemap_face_mip_offset(
            self.pmrem_face_size,
            self.pmrem_mip_count,
            face,
            mip_level,
        ) + y.min(mip_size.saturating_sub(1)) as usize * mip_size as usize
            + x.min(mip_size.saturating_sub(1)) as usize;
        self.pmrem_texels[index]
    }

    /// Rebuilds the GGX reflection chain at an independent result resolution
    /// while retaining this cubemap's full-resolution source pyramid.
    pub fn with_pmrem_face_size(
        &self,
        pmrem_face_size: u32,
        quality: SourceCubemapPrefilterQuality,
    ) -> Self {
        let pmrem_layout = SourceCubemapPmremLayout::from_face_size(pmrem_face_size);
        let pmrem_face_size = pmrem_layout.face_size();
        let pmrem_mip_count = pmrem_layout.mip_count();
        let mut pmrem_texels =
            vec![[0.0; 4]; source_cubemap_sample_count(pmrem_face_size, pmrem_mip_count)];
        pmrem::prefilter_pmrem_mips_from_source(
            &mut pmrem_texels,
            pmrem_face_size,
            pmrem_mip_count,
            &self.source_texels,
            self.source_face_size,
            self.source_mip_count,
            quality,
        );
        average_last_mip_faces(&mut pmrem_texels, pmrem_face_size, pmrem_mip_count);
        let irradiance_sh9 = self.source_irradiance_sh9;
        self.with_bake_artifact_pmrem(
            pmrem_face_size,
            pmrem_mip_count,
            pmrem_texels,
            irradiance_sh9,
        )
    }

    pub fn from_captured_faces_with_parallel_executor<E>(
        source_face_size: u32,
        captured_face_texels: Vec<[Real; 4]>,
        parallel_executor: &E,
    ) -> Self
    where
        E: ParallelSliceExecutor,
    {
        build_source_cubemap_from_captured_faces_with_parallel_executor(
            source_face_size,
            captured_face_texels,
            parallel_executor,
        )
    }

    pub fn from_captured_faces_with_quality_and_parallel_executor<E>(
        source_face_size: u32,
        captured_face_texels: Vec<[Real; 4]>,
        quality: SourceCubemapPrefilterQuality,
        parallel_executor: &E,
    ) -> Self
    where
        E: ParallelSliceExecutor,
    {
        build_source_cubemap_from_captured_faces_with_quality_and_parallel_executor(
            source_face_size,
            captured_face_texels,
            quality,
            parallel_executor,
        )
    }
}

pub fn source_cubemap_mip_count(face_size: u32) -> u32 {
    let mut size = face_size.max(1);
    let mut count = 1;
    while size > 1 {
        size = (size / 2).max(1);
        count += 1;
    }
    count
}

pub fn source_cubemap_mip_size(face_size: u32, mip_level: u32) -> u32 {
    let shifted = face_size.max(1) >> mip_level.min(u32::BITS - 1);
    shifted.max(1)
}

pub fn source_cubemap_sample_count(face_size: u32, mip_count: u32) -> usize {
    let per_face = source_cubemap_samples_per_face(face_size, mip_count);
    per_face * SOURCE_CUBEMAP_FACE_COUNT
}

pub fn source_cubemap_capture_hash(
    face_size: u32,
    face_major_base_texels: &[[Real; 4]],
) -> [u32; 4] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&face_size.max(1).to_le_bytes());
    for texel in face_major_base_texels {
        for channel in texel {
            hasher.update(&channel.to_bits().to_le_bytes());
        }
    }
    let hash = hasher.finalize();
    let bytes = hash.as_bytes();
    [
        u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
        u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
        u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]),
    ]
}

pub fn source_cubemap_face_mip_offset(
    face_size: u32,
    mip_count: u32,
    face: CubemapFace,
    mip_level: u32,
) -> usize {
    let mip_level = mip_level.min(mip_count.saturating_sub(1));
    face.index() * source_cubemap_samples_per_face(face_size, mip_count)
        + source_cubemap_mip_offset_within_face(face_size, mip_level)
}

pub(super) fn source_cubemap_face_mip_outputs(
    texels: &mut [[Real; 4]],
    face_size: u32,
    mip_count: u32,
    mip: u32,
) -> [CubemapFaceMipOutput<'_>; SOURCE_CUBEMAP_FACE_COUNT] {
    let mip = mip.min(mip_count.saturating_sub(1));
    let samples_per_face = source_cubemap_samples_per_face(face_size, mip_count);
    let face_count = CubemapFace::ALL.len();
    let (face_storage, remainder) = texels.split_at_mut(samples_per_face * face_count);
    assert!(
        remainder.is_empty(),
        "cubemap storage must contain exactly one complete mip chain per face"
    );
    let mip_offset = source_cubemap_mip_offset_within_face(face_size, mip);
    let mip_size = source_cubemap_mip_size(face_size, mip);
    let mip_len = mip_size as usize * mip_size as usize;

    let mut face_storage = face_storage.chunks_exact_mut(samples_per_face);
    std::array::from_fn(|index| {
        let face = CubemapFace::ALL[index];
        let face_texels = face_storage
            .next()
            .expect("cubemap storage must provide each face output slice");
        CubemapFaceMipOutput {
            face,
            texels: &mut face_texels[mip_offset..mip_offset + mip_len],
        }
    })
}

/// Builds the source mip pyramid, PMREM chain, and SH9 data from six captured
/// face images in cmft face order. Input contains only mip zero, face-major.
pub fn build_source_cubemap_from_captured_faces(
    face_size: u32,
    captured_face_texels: Vec<[Real; 4]>,
) -> SourceCubemapMipChain {
    build_source_cubemap_from_captured_faces_with_quality(
        face_size,
        captured_face_texels,
        SourceCubemapPrefilterQuality::Normal,
    )
}

fn build_source_cubemap_from_captured_faces_with_parallel_executor<E>(
    face_size: u32,
    captured_face_texels: Vec<[Real; 4]>,
    parallel_executor: &E,
) -> SourceCubemapMipChain
where
    E: ParallelSliceExecutor,
{
    build_source_cubemap_from_captured_faces_with_quality_and_parallel_executor(
        face_size,
        captured_face_texels,
        SourceCubemapPrefilterQuality::Normal,
        parallel_executor,
    )
}

pub fn build_source_cubemap_from_captured_faces_with_quality(
    face_size: u32,
    captured_face_texels: Vec<[Real; 4]>,
    quality: SourceCubemapPrefilterQuality,
) -> SourceCubemapMipChain {
    let (face_size, mip_count, source_storage) =
        source_cubemap_base_from_captured_faces(face_size, captured_face_texels);
    let source_mips = mipmap::source_cubemap_mips_from_base(&source_storage, face_size, mip_count);
    build_source_cubemap_from_source_mips_with_quality(face_size, mip_count, source_mips, quality)
}

fn build_source_cubemap_from_captured_faces_with_quality_and_parallel_executor<E>(
    face_size: u32,
    captured_face_texels: Vec<[Real; 4]>,
    quality: SourceCubemapPrefilterQuality,
    parallel_executor: &E,
) -> SourceCubemapMipChain
where
    E: ParallelSliceExecutor,
{
    let (face_size, mip_count, source_storage) =
        source_cubemap_base_from_captured_faces(face_size, captured_face_texels);
    let source_mips = mipmap::source_cubemap_mips_from_base_with_parallel_executor(
        &source_storage,
        face_size,
        mip_count,
        parallel_executor,
    );
    build_source_cubemap_from_source_mips_with_pmrem_layout_and_parallel_executor(
        face_size,
        mip_count,
        source_mips,
        SourceCubemapPmremLayout::default(),
        quality,
        parallel_executor,
    )
}

fn source_cubemap_base_from_captured_faces(
    face_size: u32,
    captured_face_texels: Vec<[Real; 4]>,
) -> (u32, u32, Vec<[Real; 4]>) {
    let face_size = face_size.max(1);
    let expected_base_texel_count =
        face_size as usize * face_size as usize * SOURCE_CUBEMAP_FACE_COUNT;
    assert_eq!(
        captured_face_texels.len(),
        expected_base_texel_count,
        "captured cubemap must contain six face-major mip-zero images"
    );

    let mip_count = source_cubemap_mip_count(face_size);
    let mut source_storage = vec![[0.0; 4]; source_cubemap_sample_count(face_size, mip_count)];
    let face_texel_count = face_size as usize * face_size as usize;
    for face in CubemapFace::ALL {
        let source_offset = face.index() * face_texel_count;
        let target_offset = source_cubemap_face_mip_offset(face_size, mip_count, face, 0);
        source_storage[target_offset..target_offset + face_texel_count].copy_from_slice(
            &captured_face_texels[source_offset..source_offset + face_texel_count],
        );
    }

    (face_size, mip_count, source_storage)
}

/// Rebuild Zircon PMREM/SH9 from an external cubemap's source mip pyramid.
pub fn build_source_cubemap_from_source_mips(
    face_size: u32,
    mip_count: u32,
    source_texels: Vec<[Real; 4]>,
) -> SourceCubemapMipChain {
    build_source_cubemap_from_source_mips_with_quality(
        face_size,
        mip_count,
        source_texels,
        SourceCubemapPrefilterQuality::Normal,
    )
}

pub fn build_source_cubemap_from_source_mips_with_quality(
    face_size: u32,
    mip_count: u32,
    source_texels: Vec<[Real; 4]>,
    quality: SourceCubemapPrefilterQuality,
) -> SourceCubemapMipChain {
    build_source_cubemap_from_source_mips_with_pmrem_layout(
        face_size,
        mip_count,
        source_texels,
        SourceCubemapPmremLayout::default(),
        quality,
    )
}

/// Rebuilds GGX PMREM from a source mip pyramid using an independent
/// destination size and mip count.
fn build_source_cubemap_from_source_mips_with_pmrem_layout(
    face_size: u32,
    mip_count: u32,
    source_texels: Vec<[Real; 4]>,
    pmrem_layout: SourceCubemapPmremLayout,
    quality: SourceCubemapPrefilterQuality,
) -> SourceCubemapMipChain {
    build_source_cubemap_from_source_mips_with_pmrem_layout_and_prefilter(
        face_size,
        mip_count,
        source_texels,
        pmrem_layout,
        |pmrem_texels,
         pmrem_face_size,
         pmrem_mip_count,
         source_texels,
         source_face_size,
         source_mip_count| {
            pmrem::prefilter_pmrem_mips_from_source(
                pmrem_texels,
                pmrem_face_size,
                pmrem_mip_count,
                source_texels,
                source_face_size,
                source_mip_count,
                quality,
            );
        },
    )
    .0
}

fn build_source_cubemap_from_source_mips_with_pmrem_layout_and_parallel_executor<E>(
    face_size: u32,
    mip_count: u32,
    source_texels: Vec<[Real; 4]>,
    pmrem_layout: SourceCubemapPmremLayout,
    quality: SourceCubemapPrefilterQuality,
    parallel_executor: &E,
) -> SourceCubemapMipChain
where
    E: ParallelSliceExecutor,
{
    build_source_cubemap_from_source_mips_with_pmrem_layout_and_prefilter(
        face_size,
        mip_count,
        source_texels,
        pmrem_layout,
        |pmrem_texels,
         pmrem_face_size,
         pmrem_mip_count,
         source_texels,
         source_face_size,
         source_mip_count| {
            pmrem::prefilter_pmrem_mips_from_source_with_parallel_executor(
                pmrem_texels,
                pmrem_face_size,
                pmrem_mip_count,
                source_texels,
                source_face_size,
                source_mip_count,
                quality,
                parallel_executor,
            );
        },
    )
    .0
}

fn build_source_cubemap_from_source_mips_with_pmrem_layout_and_prefilter(
    face_size: u32,
    mip_count: u32,
    source_texels: Vec<[Real; 4]>,
    pmrem_layout: SourceCubemapPmremLayout,
    prefilter_pmrem: impl FnOnce(&mut [[Real; 4]], u32, u32, &[[Real; 4]], u32, u32),
) -> (SourceCubemapMipChain, SourceCubemapBuildTiming) {
    let face_size = face_size.max(1);
    let mip_count = mip_count.clamp(1, source_cubemap_mip_count(face_size));
    let pmrem_face_size = pmrem_layout.face_size();
    let pmrem_mip_count = pmrem_layout.mip_count();
    assert_eq!(
        source_texels.len(),
        source_cubemap_sample_count(face_size, mip_count),
        "external source cubemap texel count must match face size and mip count"
    );
    let sh9_started = Instant::now();
    let irradiance_sh9 = source_cubemap_irradiance_sh9_from_texels(
        &source_texels,
        face_size,
        mip_count,
        source_cubemap_irradiance_mip_level(face_size, mip_count),
    );
    let sh9_build = sh9_started.elapsed();
    let pmrem_started = Instant::now();
    let mut pmrem_texels =
        vec![[0.0; 4]; source_cubemap_sample_count(pmrem_face_size, pmrem_mip_count)];
    prefilter_pmrem(
        &mut pmrem_texels,
        pmrem_face_size,
        pmrem_mip_count,
        &source_texels,
        face_size,
        mip_count,
    );
    average_last_mip_faces(&mut pmrem_texels, pmrem_face_size, pmrem_mip_count);
    let pmrem_build = pmrem_started.elapsed();
    (
        SourceCubemapMipChain::new_with_source_texels_and_irradiance_sh9(
            face_size,
            mip_count,
            source_texels,
            pmrem_face_size,
            pmrem_mip_count,
            pmrem_texels,
            irradiance_sh9,
        ),
        SourceCubemapBuildTiming {
            pmrem_build,
            sh9_build,
            ..Default::default()
        },
    )
}

pub fn source_cubemap_pmrem_mip_from_roughness(roughness: Real, mip_count: u32) -> Real {
    CANONICAL_IBL_BAKE_RECIPE.pmrem_mip_from_roughness(roughness, mip_count)
}

pub fn source_cubemap_roughness_from_pmrem_mip(mip_level: u32, mip_count: u32) -> Real {
    CANONICAL_IBL_BAKE_RECIPE.roughness_from_pmrem_mip(mip_level, mip_count)
}

pub fn source_cubemap_irradiance_mip_level(face_size: u32, mip_count: u32) -> u32 {
    CANONICAL_IBL_BAKE_RECIPE.diffuse_source_mip_level(face_size, mip_count)
}

fn source_cubemap_samples_per_face(face_size: u32, mip_count: u32) -> usize {
    let mut total = 0;
    for mip in 0..mip_count.max(1) {
        let size = source_cubemap_mip_size(face_size, mip);
        total += size as usize * size as usize;
    }
    total
}

fn source_cubemap_mip_offset_within_face(face_size: u32, mip_level: u32) -> usize {
    let mut offset = 0;
    for mip in 0..mip_level {
        let size = source_cubemap_mip_size(face_size, mip);
        offset += size as usize * size as usize;
    }
    offset
}

fn average_last_mip_faces(texels: &mut [[Real; 4]], face_size: u32, mip_count: u32) {
    let last_mip = mip_count.saturating_sub(1);
    if source_cubemap_mip_size(face_size, last_mip) != 1 {
        return;
    }

    let mut average = [0.0; 4];
    for face in CubemapFace::ALL {
        let offset = source_cubemap_face_mip_offset(face_size, mip_count, face, last_mip);
        let texel = texels[offset];
        average[0] += texel[0];
        average[1] += texel[1];
        average[2] += texel[2];
        average[3] += texel[3];
    }
    let inv_face_count = 1.0 / SOURCE_CUBEMAP_FACE_COUNT as Real;
    average[0] *= inv_face_count;
    average[1] *= inv_face_count;
    average[2] *= inv_face_count;
    average[3] *= inv_face_count;

    for face in CubemapFace::ALL {
        let offset = source_cubemap_face_mip_offset(face_size, mip_count, face, last_mip);
        texels[offset] = average;
    }
}

#[cfg(test)]
mod tests;
