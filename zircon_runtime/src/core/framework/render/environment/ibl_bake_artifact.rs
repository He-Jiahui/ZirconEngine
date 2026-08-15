use super::ibl_bake_recipe::{
    IblBakeDiffuseIntegrator, IblBakeRecipeIdentity, CANONICAL_IBL_BAKE_ALGORITHM_VERSION,
    CANONICAL_IBL_BAKE_RECIPE,
};
use super::rgba16f::{
    append_rgb_as_rgba16f_texels, append_rgba16f_texels, decode_rgb_from_rgba16f_texels,
    decode_rgba16f_texels, RGBA16F_TEXEL_SIZE_BYTES,
};
use super::{
    source_cubemap_sample_count, IblBakeKey, SourceCubemapIrradianceCube,
    SourceCubemapIrradianceSh9, SourceCubemapMipChain, SourceCubemapPmremLayout,
    SOURCE_CUBEMAP_FACE_COUNT, SOURCE_CUBEMAP_IRRADIANCE_COEFFICIENT_COUNT,
    SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
};
use crate::core::math::Real;
use std::ops::{BitOr, BitOrAssign, Range};

const IBL_BAKE_ARTIFACT_MAGIC: [u8; 8] = *b"ZRIBLBAK";
const IBL_BAKE_ARTIFACT_FORMAT_VERSION: u32 = 4;

// Bump when a persisted bake recipe changes so stale CPU or runtime-cache HDRI artifacts rebuild.
pub const IBL_BAKE_ALGORITHM_VERSION: u64 = CANONICAL_IBL_BAKE_ALGORITHM_VERSION;
pub const IBL_BAKE_ARTIFACT_HEADER_SIZE: usize = 120;
pub const IBL_BAKE_ARTIFACT_PAYLOAD_CHECKSUM_SIZE: usize = 32;
pub const IBL_BAKE_ARTIFACT_RGBA16F_TEXEL_SIZE_BYTES: usize = RGBA16F_TEXEL_SIZE_BYTES;
pub const IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES: usize =
    SOURCE_CUBEMAP_IRRADIANCE_COEFFICIENT_COUNT * 4 * std::mem::size_of::<f32>();

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct IblBakeArtifactContents(u32);

/// Distinguishes artifacts produced by non-equivalent CPU and GPU integrators.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IblBakeArtifactProducer {
    AssetImporterCpu = 1,
    RendererGpuRuntime = 2,
}

impl IblBakeArtifactProducer {
    const fn from_bits(bits: u32) -> Option<Self> {
        match bits {
            1 => Some(Self::AssetImporterCpu),
            2 => Some(Self::RendererGpuRuntime),
            _ => None,
        }
    }
}

impl IblBakeArtifactContents {
    pub const NONE: Self = Self(0);
    pub const PMREM: Self = Self(1 << 0);
    pub const SH9: Self = Self(1 << 1);
    pub const IEM: Self = Self(1 << 2);
    pub const PMREM_SH9: Self = Self(Self::PMREM.0 | Self::SH9.0);
    pub const PMREM_SH9_IEM: Self = Self(Self::PMREM.0 | Self::SH9.0 | Self::IEM.0);

    pub const fn bits(self) -> u32 {
        self.0
    }

    pub const fn contains(self, required: Self) -> bool {
        self.0 & required.0 == required.0
    }

    pub const fn runtime_compute_dispatch_count(self) -> u32 {
        self.0.count_ones()
    }
}

impl BitOr for IblBakeArtifactContents {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for IblBakeArtifactContents {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct IblBakeArtifactRequest {
    bake_key: IblBakeKey,
    source_face_size: u32,
    source_mip_count: u32,
    pmrem_face_size: u32,
    pmrem_mip_count: u32,
    required_contents: IblBakeArtifactContents,
}

impl IblBakeArtifactRequest {
    pub fn new(bake_key: IblBakeKey, source_face_size: u32, source_mip_count: u32) -> Self {
        let pmrem_layout = SourceCubemapPmremLayout::default();
        Self {
            bake_key,
            source_face_size: source_face_size.max(1),
            source_mip_count: source_mip_count.max(1),
            pmrem_face_size: pmrem_layout.face_size(),
            pmrem_mip_count: pmrem_layout.mip_count(),
            required_contents: IblBakeArtifactContents::PMREM_SH9,
        }
    }

    pub const fn bake_key(&self) -> IblBakeKey {
        self.bake_key
    }

    pub const fn source_face_size(&self) -> u32 {
        self.source_face_size
    }

    pub const fn source_mip_count(&self) -> u32 {
        self.source_mip_count
    }

    pub const fn pmrem_face_size(&self) -> u32 {
        self.pmrem_face_size
    }

    pub const fn pmrem_mip_count(&self) -> u32 {
        self.pmrem_mip_count
    }

    pub const fn required_contents(&self) -> IblBakeArtifactContents {
        self.required_contents
    }

    pub fn with_pmrem_layout(mut self, face_size: u32, mip_count: u32) -> Self {
        let layout = SourceCubemapPmremLayout::new(face_size, mip_count);
        self.pmrem_face_size = layout.face_size();
        self.pmrem_mip_count = layout.mip_count();
        self
    }

    pub const fn with_required_contents(
        mut self,
        required_contents: IblBakeArtifactContents,
    ) -> Self {
        self.required_contents = required_contents;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IblBakeArtifactDescriptor {
    bake_key: IblBakeKey,
    algorithm_version: u64,
    producer: IblBakeArtifactProducer,
    source_face_size: u32,
    source_mip_count: u32,
    face_size: u32,
    mip_count: u32,
    contents: IblBakeArtifactContents,
}

impl IblBakeArtifactDescriptor {
    pub fn current(
        bake_key: IblBakeKey,
        face_size: u32,
        mip_count: u32,
        contents: IblBakeArtifactContents,
    ) -> Self {
        Self {
            bake_key,
            algorithm_version: IBL_BAKE_ALGORITHM_VERSION,
            producer: IblBakeArtifactProducer::AssetImporterCpu,
            source_face_size: face_size.max(1),
            source_mip_count: mip_count.max(1),
            face_size: face_size.max(1),
            mip_count: mip_count.max(1),
            contents,
        }
    }

    pub fn current_for_request(request: &IblBakeArtifactRequest) -> Self {
        Self {
            bake_key: request.bake_key(),
            algorithm_version: IBL_BAKE_ALGORITHM_VERSION,
            producer: IblBakeArtifactProducer::AssetImporterCpu,
            source_face_size: request.source_face_size(),
            source_mip_count: request.source_mip_count(),
            face_size: request.pmrem_face_size(),
            mip_count: request.pmrem_mip_count(),
            contents: request.required_contents(),
        }
    }

    /// Runtime-cache artifacts are GPU-produced fallbacks, never canonical derived assets.
    pub fn current_for_runtime_cache_request(request: &IblBakeArtifactRequest) -> Self {
        Self {
            bake_key: request.bake_key(),
            algorithm_version: IBL_BAKE_ALGORITHM_VERSION,
            producer: IblBakeArtifactProducer::RendererGpuRuntime,
            source_face_size: request.source_face_size(),
            source_mip_count: request.source_mip_count(),
            face_size: request.pmrem_face_size(),
            mip_count: request.pmrem_mip_count(),
            contents: request.required_contents(),
        }
    }

    pub const fn bake_key(&self) -> IblBakeKey {
        self.bake_key
    }

    pub const fn algorithm_version(&self) -> u64 {
        self.algorithm_version
    }

    pub const fn producer(&self) -> IblBakeArtifactProducer {
        self.producer
    }

    pub const fn recipe_identity(&self) -> IblBakeRecipeIdentity {
        match self.producer {
            IblBakeArtifactProducer::AssetImporterCpu => IblBakeRecipeIdentity::new(
                self.algorithm_version,
                CANONICAL_IBL_BAKE_RECIPE.pmrem_integrator(),
                IblBakeDiffuseIntegrator::AssetImporterCpuSolidAngle,
                CANONICAL_IBL_BAKE_RECIPE.output_format(),
            ),
            IblBakeArtifactProducer::RendererGpuRuntime => IblBakeRecipeIdentity::new(
                self.algorithm_version,
                CANONICAL_IBL_BAKE_RECIPE.pmrem_integrator(),
                IblBakeDiffuseIntegrator::RendererGpuRuntimeHammersley,
                CANONICAL_IBL_BAKE_RECIPE.output_format(),
            ),
        }
    }

    pub const fn source_face_size(&self) -> u32 {
        self.source_face_size
    }

    pub const fn source_mip_count(&self) -> u32 {
        self.source_mip_count
    }

    pub const fn face_size(&self) -> u32 {
        self.face_size
    }

    pub const fn mip_count(&self) -> u32 {
        self.mip_count
    }

    pub const fn contents(&self) -> IblBakeArtifactContents {
        self.contents
    }

    pub fn expected_payload_size_bytes(&self) -> usize {
        artifact_payload_ranges(*self).total_size
    }

    pub fn expected_pmrem_rgba16f_size_bytes(&self) -> Option<usize> {
        artifact_payload_ranges(*self)
            .pmrem
            .map(|range| range.len())
    }

    pub fn expected_irradiance_sh9_size_bytes(&self) -> Option<usize> {
        artifact_payload_ranges(*self).sh9.map(|range| range.len())
    }

    pub fn expected_irradiance_cube_rgba16f_size_bytes(&self) -> Option<usize> {
        artifact_payload_ranges(*self).iem.map(|range| range.len())
    }

    pub const fn with_algorithm_version(mut self, algorithm_version: u64) -> Self {
        self.algorithm_version = algorithm_version;
        self
    }

    pub fn is_current_for(&self, request: &IblBakeArtifactRequest) -> bool {
        self.producer == IblBakeArtifactProducer::AssetImporterCpu && self.matches_request(request)
    }

    pub fn is_current_runtime_cache_for(&self, request: &IblBakeArtifactRequest) -> bool {
        self.producer == IblBakeArtifactProducer::RendererGpuRuntime
            && self.matches_request(request)
    }

    fn matches_request(&self, request: &IblBakeArtifactRequest) -> bool {
        self.algorithm_version == IBL_BAKE_ALGORITHM_VERSION
            && self.bake_key == request.bake_key
            && self.source_face_size == request.source_face_size()
            && self.source_mip_count == request.source_mip_count()
            && self.face_size == request.pmrem_face_size()
            && self.mip_count == request.pmrem_mip_count()
            && self.contents.contains(request.required_contents)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IblBakeArtifactHeader {
    descriptor: IblBakeArtifactDescriptor,
}

impl IblBakeArtifactHeader {
    pub const fn from_descriptor(descriptor: IblBakeArtifactDescriptor) -> Self {
        Self { descriptor }
    }

    pub const fn descriptor(&self) -> IblBakeArtifactDescriptor {
        self.descriptor
    }

    pub fn encode(&self) -> [u8; IBL_BAKE_ARTIFACT_HEADER_SIZE] {
        let mut bytes = [0; IBL_BAKE_ARTIFACT_HEADER_SIZE];
        let mut cursor = 0;
        write_bytes(&mut bytes, &mut cursor, &IBL_BAKE_ARTIFACT_MAGIC);
        write_u32(&mut bytes, &mut cursor, IBL_BAKE_ARTIFACT_FORMAT_VERSION);
        write_u64(&mut bytes, &mut cursor, self.descriptor.algorithm_version);
        write_u32(&mut bytes, &mut cursor, self.descriptor.source_face_size);
        write_u32(&mut bytes, &mut cursor, self.descriptor.source_mip_count);
        write_u32(&mut bytes, &mut cursor, self.descriptor.face_size);
        write_u32(&mut bytes, &mut cursor, self.descriptor.mip_count);
        write_u32(&mut bytes, &mut cursor, self.descriptor.contents.bits());
        write_u32(&mut bytes, &mut cursor, self.descriptor.producer as u32);
        write_ibl_bake_key(&mut bytes, &mut cursor, self.descriptor.bake_key);
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, IblBakeArtifactHeaderError> {
        if bytes.len() != IBL_BAKE_ARTIFACT_HEADER_SIZE {
            return Err(IblBakeArtifactHeaderError::InvalidLength);
        }
        let mut cursor = 0;
        if read_bytes::<8>(bytes, &mut cursor) != IBL_BAKE_ARTIFACT_MAGIC {
            return Err(IblBakeArtifactHeaderError::InvalidMagic);
        }
        let format_version = read_u32(bytes, &mut cursor);
        if format_version != IBL_BAKE_ARTIFACT_FORMAT_VERSION {
            return Err(IblBakeArtifactHeaderError::UnsupportedFormatVersion(
                format_version,
            ));
        }
        let algorithm_version = read_u64(bytes, &mut cursor);
        let source_face_size = read_u32(bytes, &mut cursor);
        let source_mip_count = read_u32(bytes, &mut cursor);
        let face_size = read_u32(bytes, &mut cursor);
        let mip_count = read_u32(bytes, &mut cursor);
        let contents = IblBakeArtifactContents(read_u32(bytes, &mut cursor));
        let producer = IblBakeArtifactProducer::from_bits(read_u32(bytes, &mut cursor))
            .ok_or(IblBakeArtifactHeaderError::InvalidProducer)?;
        let bake_key = read_ibl_bake_key(bytes, &mut cursor);
        Ok(Self {
            descriptor: IblBakeArtifactDescriptor {
                bake_key,
                algorithm_version,
                producer,
                source_face_size,
                source_mip_count,
                face_size,
                mip_count,
                contents,
            },
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IblBakeArtifactHeaderError {
    InvalidLength,
    InvalidMagic,
    UnsupportedFormatVersion(u32),
    InvalidProducer,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IblBakeArtifactPayload {
    descriptor: IblBakeArtifactDescriptor,
    bytes: Vec<u8>,
}

impl IblBakeArtifactPayload {
    pub fn from_source_cubemap(
        descriptor: IblBakeArtifactDescriptor,
        cubemap: &SourceCubemapMipChain,
        irradiance_cube: Option<&SourceCubemapIrradianceCube>,
    ) -> Result<Self, IblBakeArtifactPayloadError> {
        if cubemap.source_face_size() != descriptor.source_face_size
            || cubemap.source_mip_count() != descriptor.source_mip_count
        {
            return Err(IblBakeArtifactPayloadError::SourceCubemapLayoutMismatch {
                expected_face_size: descriptor.source_face_size,
                actual_face_size: cubemap.source_face_size(),
                expected_mip_count: descriptor.source_mip_count,
                actual_mip_count: cubemap.source_mip_count(),
            });
        }
        if descriptor.contents.contains(IblBakeArtifactContents::PMREM)
            && (cubemap.pmrem_face_size() != descriptor.face_size
                || cubemap.pmrem_mip_count() != descriptor.mip_count)
        {
            return Err(IblBakeArtifactPayloadError::SourceCubemapLayoutMismatch {
                expected_face_size: descriptor.face_size,
                actual_face_size: cubemap.pmrem_face_size(),
                expected_mip_count: descriptor.mip_count,
                actual_mip_count: cubemap.pmrem_mip_count(),
            });
        }

        let ranges = artifact_payload_ranges(descriptor);
        let mut bytes = Vec::with_capacity(ranges.total_size);
        if descriptor.contents.contains(IblBakeArtifactContents::PMREM) {
            append_rgba16f_texels(&mut bytes, cubemap.pmrem_texels());
        }
        if descriptor.contents.contains(IblBakeArtifactContents::SH9) {
            push_sh9(&mut bytes, cubemap.irradiance_sh9());
        }
        if descriptor.contents.contains(IblBakeArtifactContents::IEM) {
            let irradiance_cube =
                irradiance_cube.ok_or(IblBakeArtifactPayloadError::MissingIrradianceCube)?;
            if irradiance_cube.face_size() != SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE {
                return Err(IblBakeArtifactPayloadError::IrradianceCubeLayoutMismatch {
                    expected_face_size: SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
                    actual_face_size: irradiance_cube.face_size(),
                });
            }
            append_rgb_as_rgba16f_texels(&mut bytes, irradiance_cube.texels(), 1.0);
        }

        debug_assert_eq!(bytes.len(), ranges.total_size);
        Ok(Self { descriptor, bytes })
    }

    pub fn decode(
        descriptor: IblBakeArtifactDescriptor,
        bytes: &[u8],
    ) -> Result<Self, IblBakeArtifactPayloadError> {
        let expected = descriptor.expected_payload_size_bytes();
        if bytes.len() != expected {
            return Err(IblBakeArtifactPayloadError::InvalidPayloadLength {
                expected,
                actual: bytes.len(),
            });
        }
        Ok(Self {
            descriptor,
            bytes: bytes.to_vec(),
        })
    }

    pub const fn descriptor(&self) -> IblBakeArtifactDescriptor {
        self.descriptor
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn pmrem_rgba16f_byte_range(&self) -> Option<Range<usize>> {
        artifact_payload_ranges(self.descriptor).pmrem
    }

    pub fn irradiance_sh9_byte_range(&self) -> Option<Range<usize>> {
        artifact_payload_ranges(self.descriptor).sh9
    }

    pub fn irradiance_cube_rgba16f_byte_range(&self) -> Option<Range<usize>> {
        artifact_payload_ranges(self.descriptor).iem
    }

    pub fn decode_pmrem_texels(&self) -> Option<Vec<[Real; 4]>> {
        let range = self.pmrem_rgba16f_byte_range()?;
        Some(decode_rgba16f_texels(&self.bytes[range]))
    }

    pub fn decode_irradiance_sh9(&self) -> Option<SourceCubemapIrradianceSh9> {
        let range = self.irradiance_sh9_byte_range()?;
        Some(decode_sh9(&self.bytes[range]))
    }

    pub fn decode_irradiance_cube_texels(&self) -> Option<Vec<[Real; 3]>> {
        let range = self.irradiance_cube_rgba16f_byte_range()?;
        Some(decode_rgb_from_rgba16f_texels(&self.bytes[range]))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IblBakeArtifactPayloadError {
    SourceCubemapLayoutMismatch {
        expected_face_size: u32,
        actual_face_size: u32,
        expected_mip_count: u32,
        actual_mip_count: u32,
    },
    MissingIrradianceCube,
    IrradianceCubeLayoutMismatch {
        expected_face_size: u32,
        actual_face_size: u32,
    },
    InvalidPayloadLength {
        expected: usize,
        actual: usize,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IblBakeArtifactSource {
    AssetDerivedArtifact,
    RuntimeCache,
    RuntimeCompute,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IblBakeArtifactCandidate {
    source: IblBakeArtifactSource,
    descriptor: IblBakeArtifactDescriptor,
}

impl IblBakeArtifactCandidate {
    pub const fn asset_derived(descriptor: IblBakeArtifactDescriptor) -> Self {
        Self {
            source: IblBakeArtifactSource::AssetDerivedArtifact,
            descriptor,
        }
    }

    pub const fn runtime_cache(descriptor: IblBakeArtifactDescriptor) -> Self {
        Self {
            source: IblBakeArtifactSource::RuntimeCache,
            descriptor,
        }
    }

    pub const fn source(&self) -> IblBakeArtifactSource {
        self.source
    }

    pub const fn descriptor(&self) -> IblBakeArtifactDescriptor {
        self.descriptor
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IblBakeArtifactSelection {
    source: IblBakeArtifactSource,
    descriptor: Option<IblBakeArtifactDescriptor>,
    rejected_candidate_count: usize,
    environment_compute_dispatch_count: u32,
}

impl IblBakeArtifactSelection {
    pub const fn source(&self) -> IblBakeArtifactSource {
        self.source
    }

    pub const fn descriptor(&self) -> Option<IblBakeArtifactDescriptor> {
        self.descriptor
    }

    pub const fn rejected_candidate_count(&self) -> usize {
        self.rejected_candidate_count
    }

    pub const fn environment_compute_dispatch_count(&self) -> u32 {
        self.environment_compute_dispatch_count
    }

    pub const fn requires_runtime_compute(&self) -> bool {
        matches!(self.source, IblBakeArtifactSource::RuntimeCompute)
    }
}

pub fn select_ibl_bake_artifact(
    request: &IblBakeArtifactRequest,
    candidates: &[IblBakeArtifactCandidate],
) -> IblBakeArtifactSelection {
    let rejected_candidate_count = candidates
        .iter()
        .filter(|candidate| match candidate.source {
            IblBakeArtifactSource::AssetDerivedArtifact => {
                !candidate.descriptor.is_current_for(request)
            }
            IblBakeArtifactSource::RuntimeCache => {
                !candidate.descriptor.is_current_runtime_cache_for(request)
            }
            IblBakeArtifactSource::RuntimeCompute => true,
        })
        .count();

    for source in [
        IblBakeArtifactSource::AssetDerivedArtifact,
        IblBakeArtifactSource::RuntimeCache,
    ] {
        if let Some(candidate) = candidates.iter().find(|candidate| {
            candidate.source == source
                && match source {
                    IblBakeArtifactSource::AssetDerivedArtifact => {
                        candidate.descriptor.is_current_for(request)
                    }
                    IblBakeArtifactSource::RuntimeCache => {
                        candidate.descriptor.is_current_runtime_cache_for(request)
                    }
                    IblBakeArtifactSource::RuntimeCompute => false,
                }
        }) {
            return IblBakeArtifactSelection {
                source,
                descriptor: Some(candidate.descriptor),
                rejected_candidate_count,
                environment_compute_dispatch_count: 0,
            };
        }
    }

    IblBakeArtifactSelection {
        source: IblBakeArtifactSource::RuntimeCompute,
        descriptor: None,
        rejected_candidate_count,
        environment_compute_dispatch_count: request
            .required_contents
            .runtime_compute_dispatch_count(),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ArtifactPayloadRanges {
    pmrem: Option<Range<usize>>,
    sh9: Option<Range<usize>>,
    iem: Option<Range<usize>>,
    total_size: usize,
}

fn artifact_payload_ranges(descriptor: IblBakeArtifactDescriptor) -> ArtifactPayloadRanges {
    let mut cursor = 0;
    let pmrem = if descriptor.contents.contains(IblBakeArtifactContents::PMREM) {
        let byte_len = source_cubemap_sample_count(descriptor.face_size, descriptor.mip_count)
            * IBL_BAKE_ARTIFACT_RGBA16F_TEXEL_SIZE_BYTES;
        let range = cursor..cursor + byte_len;
        cursor += byte_len;
        Some(range)
    } else {
        None
    };

    let sh9 = if descriptor.contents.contains(IblBakeArtifactContents::SH9) {
        let range = cursor..cursor + IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES;
        cursor += IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES;
        Some(range)
    } else {
        None
    };

    let iem = if descriptor.contents.contains(IblBakeArtifactContents::IEM) {
        let texel_count = SOURCE_CUBEMAP_FACE_COUNT
            * SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE as usize
            * SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE as usize;
        let byte_len = texel_count * IBL_BAKE_ARTIFACT_RGBA16F_TEXEL_SIZE_BYTES;
        let range = cursor..cursor + byte_len;
        cursor += byte_len;
        Some(range)
    } else {
        None
    };

    ArtifactPayloadRanges {
        pmrem,
        sh9,
        iem,
        total_size: cursor,
    }
}

fn push_sh9(bytes: &mut Vec<u8>, coefficients: &SourceCubemapIrradianceSh9) {
    for coefficient in coefficients {
        for channel in *coefficient {
            bytes.extend_from_slice(&channel.to_le_bytes());
        }
    }
}

fn decode_sh9(bytes: &[u8]) -> SourceCubemapIrradianceSh9 {
    let mut coefficients = [[0.0; 4]; SOURCE_CUBEMAP_IRRADIANCE_COEFFICIENT_COUNT];
    let mut cursor = 0;
    for coefficient in &mut coefficients {
        for channel in coefficient {
            *channel = f32::from_le_bytes(read_dynamic_bytes::<4>(bytes, &mut cursor));
        }
    }
    coefficients
}

fn write_ibl_bake_key(
    bytes: &mut [u8; IBL_BAKE_ARTIFACT_HEADER_SIZE],
    cursor: &mut usize,
    bake_key: IblBakeKey,
) {
    write_u32(bytes, cursor, bake_key.source_kind);
    write_u64(bytes, cursor, bake_key.source_revision);
    for value in bake_key.horizon_color {
        write_u32(bytes, cursor, value);
    }
    for value in bake_key.zenith_color {
        write_u32(bytes, cursor, value);
    }
    for value in bake_key.ground_color {
        write_u32(bytes, cursor, value);
    }
    for value in bake_key.source_hash {
        write_u32(bytes, cursor, value);
    }
}

fn read_ibl_bake_key(bytes: &[u8], cursor: &mut usize) -> IblBakeKey {
    IblBakeKey {
        source_kind: read_u32(bytes, cursor),
        source_revision: read_u64(bytes, cursor),
        horizon_color: read_u32_array(bytes, cursor),
        zenith_color: read_u32_array(bytes, cursor),
        ground_color: read_u32_array(bytes, cursor),
        source_hash: read_u32_array(bytes, cursor),
    }
}

fn write_bytes(bytes: &mut [u8; IBL_BAKE_ARTIFACT_HEADER_SIZE], cursor: &mut usize, value: &[u8]) {
    let next = *cursor + value.len();
    bytes[*cursor..next].copy_from_slice(value);
    *cursor = next;
}

fn write_u32(bytes: &mut [u8; IBL_BAKE_ARTIFACT_HEADER_SIZE], cursor: &mut usize, value: u32) {
    write_bytes(bytes, cursor, &value.to_le_bytes());
}

fn write_u64(bytes: &mut [u8; IBL_BAKE_ARTIFACT_HEADER_SIZE], cursor: &mut usize, value: u64) {
    write_bytes(bytes, cursor, &value.to_le_bytes());
}

fn read_bytes<const N: usize>(bytes: &[u8], cursor: &mut usize) -> [u8; N] {
    let mut value = [0; N];
    let next = *cursor + N;
    value.copy_from_slice(&bytes[*cursor..next]);
    *cursor = next;
    value
}

fn read_dynamic_bytes<const N: usize>(bytes: &[u8], cursor: &mut usize) -> [u8; N] {
    let mut value = [0; N];
    let next = *cursor + N;
    value.copy_from_slice(&bytes[*cursor..next]);
    *cursor = next;
    value
}

fn read_u32(bytes: &[u8], cursor: &mut usize) -> u32 {
    u32::from_le_bytes(read_bytes(bytes, cursor))
}

fn read_u64(bytes: &[u8], cursor: &mut usize) -> u64 {
    u64::from_le_bytes(read_bytes(bytes, cursor))
}

fn read_u32_array(bytes: &[u8], cursor: &mut usize) -> [u32; 4] {
    [
        read_u32(bytes, cursor),
        read_u32(bytes, cursor),
        read_u32(bytes, cursor),
        read_u32(bytes, cursor),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framework::render::ProceduralSkyParams;

    #[test]
    fn header_round_trips_descriptor() {
        let key = ProceduralSkyParams::default_gradient().ibl_bake_key();
        let descriptor =
            IblBakeArtifactDescriptor::current(key, 128, 8, IblBakeArtifactContents::PMREM_SH9);

        let encoded = IblBakeArtifactHeader::from_descriptor(descriptor).encode();

        assert_eq!(
            IblBakeArtifactHeader::decode(&encoded)
                .unwrap()
                .descriptor(),
            descriptor
        );
    }

    #[test]
    fn descriptor_recipe_identity_keeps_cpu_and_runtime_integrators_distinct() {
        let request = IblBakeArtifactRequest::new(
            ProceduralSkyParams::default_gradient().ibl_bake_key(),
            128,
            8,
        );
        let asset = IblBakeArtifactDescriptor::current_for_request(&request);
        let runtime = IblBakeArtifactDescriptor::current_for_runtime_cache_request(&request);
        let stale = runtime.with_algorithm_version(IBL_BAKE_ALGORITHM_VERSION - 1);

        assert_eq!(
            asset.recipe_identity(),
            CANONICAL_IBL_BAKE_RECIPE.asset_recipe_identity()
        );
        assert_eq!(
            runtime.recipe_identity(),
            CANONICAL_IBL_BAKE_RECIPE.runtime_recipe_identity()
        );
        assert_ne!(asset.recipe_identity(), runtime.recipe_identity());
        assert_ne!(stale.recipe_identity(), runtime.recipe_identity());
        assert!(!stale.is_current_runtime_cache_for(&request));
    }

    #[test]
    fn sh9_only_payload_does_not_require_pmrem_layout_match() {
        let cubemap = SourceCubemapMipChain::new(
            4,
            3,
            vec![[0.25, 0.5, 0.75, 1.0]; source_cubemap_sample_count(4, 3)],
            4,
            3,
            vec![[0.25, 0.5, 0.75, 1.0]; source_cubemap_sample_count(4, 3)],
        );
        let request = IblBakeArtifactRequest::new(
            ProceduralSkyParams::default_gradient().ibl_bake_key(),
            4,
            3,
        )
        .with_required_contents(IblBakeArtifactContents::SH9);
        let descriptor = IblBakeArtifactDescriptor::current_for_request(&request);

        let payload = IblBakeArtifactPayload::from_source_cubemap(descriptor, &cubemap, None)
            .expect("SH9 serialization depends on coefficients, not PMREM texture layout");

        assert_eq!(payload.bytes().len(), IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES);
    }
}
