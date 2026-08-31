use super::{
    source_cubemap_mip_count, IblBakeArtifactContents, IblBakeArtifactRequest, IblBakeKey,
    RenderLayerSet, SourceCubemapPrefilterQuality, SOURCE_CUBEMAP_PMREM_FACE_SIZE,
    SOURCE_CUBEMAP_PMREM_MIP_COUNT,
};
use crate::core::resource::ResourceId;

pub const RENDER_ENVIRONMENT_CAPTURE_WORK_ITEM_COUNT: u32 = 6;

// Version the scene-raster source independently from PMREM/SH filtering so
// material or lighting capture-policy changes invalidate only runtime captures.
const RENDER_ENVIRONMENT_CAPTURE_RASTER_ALGORITHM_VERSION: u64 = 2026_08_31_0001;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RenderEnvironmentCaptureHandle(u64);

impl RenderEnvironmentCaptureHandle {
    pub const fn new(value: u64) -> Option<Self> {
        if value == 0 {
            None
        } else {
            Some(Self(value))
        }
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderEnvironmentCapturePhase {
    Queued,
    Capturing,
    Filtering,
    Persisting,
    Succeeded,
    Failed,
    Cancelled,
    Superseded,
}

impl RenderEnvironmentCapturePhase {
    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Succeeded | Self::Failed | Self::Cancelled | Self::Superseded
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderEnvironmentCaptureOutputIdentity {
    capture_id: String,
    scene_revision: u64,
    environment_revision: u64,
    output_generation: u64,
    capture_layer_mask: RenderLayerSet,
    reflection_probe_target: Option<RenderReflectionProbeCaptureTarget>,
    persistence_target: Option<RenderEnvironmentCapturePersistenceTarget>,
    runtime_cache_artifact_request: Option<IblBakeArtifactRequest>,
}

impl RenderEnvironmentCaptureOutputIdentity {
    pub fn from_request(request: &RenderEnvironmentCaptureRequest) -> Self {
        Self {
            capture_id: request.capture_id.clone(),
            scene_revision: request.scene_revision,
            environment_revision: request.environment_revision,
            output_generation: request.output_generation,
            capture_layer_mask: request.capture_layer_mask.clone(),
            reflection_probe_target: request.reflection_probe_target,
            persistence_target: request.persistence_target.clone(),
            runtime_cache_artifact_request: request.runtime_cache_artifact_request(),
        }
    }

    pub fn capture_id(&self) -> &str {
        &self.capture_id
    }
    pub const fn scene_revision(&self) -> u64 {
        self.scene_revision
    }
    pub const fn environment_revision(&self) -> u64 {
        self.environment_revision
    }
    pub const fn output_generation(&self) -> u64 {
        self.output_generation
    }

    pub const fn capture_layer_mask(&self) -> &RenderLayerSet {
        &self.capture_layer_mask
    }

    pub const fn reflection_probe_target(&self) -> Option<(u64, ResourceId)> {
        match self.reflection_probe_target {
            Some(target) => Some((target.probe_id, target.cubemap)),
            None => None,
        }
    }

    pub const fn has_reflection_probe_target(&self) -> bool {
        self.reflection_probe_target.is_some()
    }

    /// Returns the destination locator for an eventual asynchronous artifact write.
    ///
    /// The locator is deliberately separate from `ibl_bake_key`: changing where an
    /// artifact is stored must not invalidate the content identity used for caching.
    pub fn persistence_output_uri(&self) -> Option<&str> {
        self.persistence_target
            .as_ref()
            .map(|target| target.output_uri.as_str())
    }

    pub const fn persistence_artifact_request(&self) -> Option<IblBakeArtifactRequest> {
        match self.persistence_target {
            Some(target) => target.artifact_request,
            None => None,
        }
    }

    /// Returns the renderer-owned runtime-cache identity for this capture output.
    ///
    /// This is deliberately distinct from `persistence_artifact_request`: the latter is
    /// supplied by the asset/editor owner, while the runtime identity includes the capture
    /// position, clip range, quality and scene/environment generations.
    pub const fn runtime_cache_artifact_request(&self) -> Option<IblBakeArtifactRequest> {
        self.runtime_cache_artifact_request
    }
}

fn hash_environment_capture_bytes(hash: &mut [u32; 4], bytes: &[u8]) {
    const FNV_PRIME: u32 = 0x0100_0193;
    for &byte in bytes {
        for (lane_index, lane) in hash.iter_mut().enumerate() {
            let lane_salt = (lane_index as u32).wrapping_mul(0x9e37_79b9);
            *lane ^= u32::from(byte).wrapping_add(lane_salt);
            *lane = lane.wrapping_mul(FNV_PRIME);
        }
    }
}

const fn environment_capture_quality_tag(quality: SourceCubemapPrefilterQuality) -> u32 {
    match quality {
        SourceCubemapPrefilterQuality::Fast => 0,
        SourceCubemapPrefilterQuality::Normal => 1,
        SourceCubemapPrefilterQuality::High => 2,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct RenderReflectionProbeCaptureTarget {
    probe_id: u64,
    cubemap: ResourceId,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct RenderEnvironmentCapturePersistenceTarget {
    output_uri: String,
    artifact_request: Option<IblBakeArtifactRequest>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderEnvironmentCaptureRequest {
    capture_id: String,
    position: [f32; 3],
    near_plane_bits: u32,
    far_plane_bits: u32,
    face_size: u32,
    quality: SourceCubemapPrefilterQuality,
    scene_revision: u64,
    environment_revision: u64,
    output_generation: u64,
    capture_layer_mask: RenderLayerSet,
    reflection_probe_target: Option<RenderReflectionProbeCaptureTarget>,
    persistence_target: Option<RenderEnvironmentCapturePersistenceTarget>,
}

impl RenderEnvironmentCaptureRequest {
    pub fn new(
        capture_id: impl Into<String>,
        position: [f32; 3],
        source_revision: u64,
    ) -> Result<Self, RenderEnvironmentCaptureRequestError> {
        Self::with_revisions(
            capture_id,
            position,
            0.1,
            200.0,
            128,
            SourceCubemapPrefilterQuality::Normal,
            source_revision,
            source_revision,
            source_revision,
        )
    }

    pub fn with_revisions(
        capture_id: impl Into<String>,
        position: [f32; 3],
        near_plane: f32,
        far_plane: f32,
        face_size: u32,
        quality: SourceCubemapPrefilterQuality,
        scene_revision: u64,
        environment_revision: u64,
        output_generation: u64,
    ) -> Result<Self, RenderEnvironmentCaptureRequestError> {
        let request = Self {
            capture_id: capture_id.into(),
            // PartialEq treats signed zero as equal, so canonicalize it before
            // the bake key hashes the position's bit pattern.
            position: position.map(|component| if component == 0.0 { 0.0 } else { component }),
            near_plane_bits: near_plane.to_bits(),
            far_plane_bits: far_plane.to_bits(),
            face_size,
            quality,
            scene_revision,
            environment_revision,
            output_generation,
            capture_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
            reflection_probe_target: None,
            persistence_target: None,
        };
        request.validate()?;
        Ok(request)
    }

    pub fn with_clip_planes(
        self,
        near_plane: f32,
        far_plane: f32,
    ) -> Result<Self, RenderEnvironmentCaptureRequestError> {
        let mut request = self;
        request.near_plane_bits = near_plane.to_bits();
        request.far_plane_bits = far_plane.to_bits();
        request.validate()?;
        Ok(request)
    }

    pub fn with_face_size(
        self,
        face_size: u32,
    ) -> Result<Self, RenderEnvironmentCaptureRequestError> {
        let mut request = self;
        request.face_size = face_size;
        request.validate()?;
        Ok(request)
    }

    pub fn with_quality(self, quality: SourceCubemapPrefilterQuality) -> Self {
        Self { quality, ..self }
    }

    pub fn with_capture_layer_mask(mut self, capture_layer_mask: RenderLayerSet) -> Self {
        self.capture_layer_mask = capture_layer_mask;
        self
    }

    pub fn with_reflection_probe_target(mut self, probe_id: u64, cubemap: ResourceId) -> Self {
        self.reflection_probe_target =
            Some(RenderReflectionProbeCaptureTarget { probe_id, cubemap });
        self
    }

    pub fn with_persistence_output_uri(
        mut self,
        output_uri: impl Into<String>,
    ) -> Result<Self, RenderEnvironmentCaptureRequestError> {
        let output_uri = output_uri.into();
        if output_uri.trim().is_empty() {
            return Err(RenderEnvironmentCaptureRequestError::EmptyPersistenceOutputUri);
        }
        self.persistence_target = Some(RenderEnvironmentCapturePersistenceTarget {
            output_uri,
            artifact_request: None,
        });
        self.validate()?;
        Ok(self)
    }

    /// Associates an explicit artifact identity with this capture destination.
    ///
    /// The artifact request is supplied by the asset/editor owner because a
    /// runtime snapshot key and an asset-derived IBL key are not interchangeable.
    pub fn with_persistence_artifact_request(
        mut self,
        artifact_request: IblBakeArtifactRequest,
    ) -> Result<Self, RenderEnvironmentCaptureRequestError> {
        let capture_face_size = self.face_size;
        let Some(target) = self.persistence_target.as_mut() else {
            return Err(RenderEnvironmentCaptureRequestError::PersistenceArtifactRequiresOutputUri);
        };
        validate_persistence_artifact_request(capture_face_size, &artifact_request)?;
        target.artifact_request = Some(artifact_request);
        Ok(self)
    }

    /// Returns the stable runtime-cache identity for the rendered scene/environment snapshot.
    pub fn ibl_bake_key(&self) -> IblBakeKey {
        self.ibl_bake_key_with_raster_algorithm_version(
            RENDER_ENVIRONMENT_CAPTURE_RASTER_ALGORITHM_VERSION,
        )
    }

    fn ibl_bake_key_with_raster_algorithm_version(
        &self,
        raster_algorithm_version: u64,
    ) -> IblBakeKey {
        let mut hash = [
            0x811c_9dc5_u32,
            0x9e37_79b9_u32,
            0x85eb_ca6b_u32,
            0xc2b2_ae35_u32,
        ];
        hash_environment_capture_bytes(&mut hash, &raster_algorithm_version.to_le_bytes());
        // Length-prefix the free-form identifier so adjacent fields remain
        // unambiguous even when IDs contain arbitrary bytes.
        hash_environment_capture_bytes(&mut hash, &(self.capture_id.len() as u64).to_le_bytes());
        hash_environment_capture_bytes(&mut hash, self.capture_id.as_bytes());
        for value in self.position {
            hash_environment_capture_bytes(&mut hash, &value.to_bits().to_le_bytes());
        }
        for value in [
            self.near_plane_bits,
            self.far_plane_bits,
            self.face_size,
            environment_capture_quality_tag(self.quality),
        ] {
            hash_environment_capture_bytes(&mut hash, &value.to_le_bytes());
        }
        hash_environment_capture_bytes(
            &mut hash,
            &(self.capture_layer_mask.iter().count() as u64).to_le_bytes(),
        );
        for layer in self.capture_layer_mask.iter() {
            hash_environment_capture_bytes(&mut hash, &layer.to_le_bytes());
        }
        for value in [
            self.scene_revision,
            self.environment_revision,
            self.output_generation,
        ] {
            hash_environment_capture_bytes(&mut hash, &value.to_le_bytes());
        }
        IblBakeKey::source_cubemap(self.output_generation, hash)
    }

    pub fn validate(&self) -> Result<(), RenderEnvironmentCaptureRequestError> {
        if self.capture_id.trim().is_empty() {
            return Err(RenderEnvironmentCaptureRequestError::EmptyCaptureId);
        }
        if self
            .persistence_target
            .as_ref()
            .is_some_and(|target| target.output_uri.trim().is_empty())
        {
            return Err(RenderEnvironmentCaptureRequestError::EmptyPersistenceOutputUri);
        }
        if let Some(target) = self.persistence_target.as_ref() {
            if let Some(artifact_request) = target.artifact_request.as_ref() {
                validate_persistence_artifact_request(self.face_size, artifact_request)?;
            }
        }
        if !self.position.iter().all(|value| value.is_finite()) {
            return Err(RenderEnvironmentCaptureRequestError::NonFinitePosition);
        }
        let near_plane = self.near_plane();
        if !near_plane.is_finite() || near_plane <= 0.0 {
            return Err(RenderEnvironmentCaptureRequestError::InvalidNearPlane(
                near_plane,
            ));
        }
        let far_plane = self.far_plane();
        if !far_plane.is_finite() || far_plane <= near_plane {
            return Err(RenderEnvironmentCaptureRequestError::InvalidFarPlane {
                near: near_plane,
                far: far_plane,
            });
        }
        if !self.face_size.is_power_of_two()
            || !(super::SOURCE_CUBEMAP_MIN_FACE_SIZE..=super::SOURCE_CUBEMAP_MAX_FACE_SIZE)
                .contains(&self.face_size)
        {
            return Err(RenderEnvironmentCaptureRequestError::InvalidFaceSize(
                self.face_size,
            ));
        }
        Ok(())
    }

    pub fn capture_id(&self) -> &str {
        &self.capture_id
    }
    pub const fn position(&self) -> [f32; 3] {
        self.position
    }
    pub const fn near_plane(&self) -> f32 {
        f32::from_bits(self.near_plane_bits)
    }
    pub const fn far_plane(&self) -> f32 {
        f32::from_bits(self.far_plane_bits)
    }
    pub const fn face_size(&self) -> u32 {
        self.face_size
    }
    pub const fn quality(&self) -> SourceCubemapPrefilterQuality {
        self.quality
    }
    pub const fn scene_revision(&self) -> u64 {
        self.scene_revision
    }
    pub const fn environment_revision(&self) -> u64 {
        self.environment_revision
    }
    pub const fn output_generation(&self) -> u64 {
        self.output_generation
    }
    pub const fn capture_layer_mask(&self) -> &RenderLayerSet {
        &self.capture_layer_mask
    }
    pub const fn reflection_probe_target(&self) -> Option<(u64, ResourceId)> {
        match self.reflection_probe_target {
            Some(target) => Some((target.probe_id, target.cubemap)),
            None => None,
        }
    }

    pub fn persistence_output_uri(&self) -> Option<&str> {
        self.persistence_target
            .as_ref()
            .map(|target| target.output_uri.as_str())
    }

    pub const fn persistence_artifact_request(&self) -> Option<IblBakeArtifactRequest> {
        match self.persistence_target {
            Some(target) => target.artifact_request,
            None => None,
        }
    }

    /// Builds the cache request for GPU output produced by this exact capture recipe.
    ///
    /// The external persistence request only authorizes and identifies later asset/editor
    /// publication. Reusing its source key for runtime bytes would alias different capture
    /// positions, clip ranges, qualities or scene generations.
    pub fn runtime_cache_artifact_request(&self) -> Option<IblBakeArtifactRequest> {
        let required_contents = self.persistence_artifact_request()?.required_contents();
        Some(
            IblBakeArtifactRequest::new(
                self.ibl_bake_key(),
                self.face_size,
                source_cubemap_mip_count(self.face_size),
            )
            .with_required_contents(required_contents),
        )
    }
}

fn validate_persistence_artifact_request(
    capture_face_size: u32,
    artifact_request: &IblBakeArtifactRequest,
) -> Result<(), RenderEnvironmentCaptureRequestError> {
    let expected_source_mip_count = source_cubemap_mip_count(capture_face_size);
    if artifact_request.source_face_size() != capture_face_size
        || artifact_request.source_mip_count() != expected_source_mip_count
    {
        return Err(
            RenderEnvironmentCaptureRequestError::PersistenceArtifactSourceLayoutMismatch {
                expected_face_size: capture_face_size,
                expected_mip_count: expected_source_mip_count,
                actual_face_size: artifact_request.source_face_size(),
                actual_mip_count: artifact_request.source_mip_count(),
            },
        );
    }
    if artifact_request.pmrem_face_size() != SOURCE_CUBEMAP_PMREM_FACE_SIZE
        || artifact_request.pmrem_mip_count() != SOURCE_CUBEMAP_PMREM_MIP_COUNT
    {
        return Err(
            RenderEnvironmentCaptureRequestError::PersistenceArtifactPmremLayoutMismatch {
                expected_face_size: SOURCE_CUBEMAP_PMREM_FACE_SIZE,
                expected_mip_count: SOURCE_CUBEMAP_PMREM_MIP_COUNT,
                actual_face_size: artifact_request.pmrem_face_size(),
                actual_mip_count: artifact_request.pmrem_mip_count(),
            },
        );
    }
    let contents = artifact_request.required_contents();
    if contents.bits() == 0 || contents.bits() & !IblBakeArtifactContents::PMREM_SH9.bits() != 0 {
        return Err(
            RenderEnvironmentCaptureRequestError::UnsupportedPersistenceArtifactContents(
                contents.bits(),
            ),
        );
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq)]
pub enum RenderEnvironmentCaptureRequestError {
    EmptyCaptureId,
    EmptyPersistenceOutputUri,
    PersistenceArtifactRequiresOutputUri,
    PersistenceArtifactSourceLayoutMismatch {
        expected_face_size: u32,
        expected_mip_count: u32,
        actual_face_size: u32,
        actual_mip_count: u32,
    },
    PersistenceArtifactPmremLayoutMismatch {
        expected_face_size: u32,
        expected_mip_count: u32,
        actual_face_size: u32,
        actual_mip_count: u32,
    },
    UnsupportedPersistenceArtifactContents(u32),
    NonFinitePosition,
    InvalidNearPlane(f32),
    InvalidFarPlane {
        near: f32,
        far: f32,
    },
    InvalidFaceSize(u32),
}

impl std::fmt::Display for RenderEnvironmentCaptureRequestError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyCaptureId => formatter.write_str("environment capture id must not be empty"),
            Self::EmptyPersistenceOutputUri => {
                formatter.write_str("environment capture persistence output URI must not be empty")
            }
            Self::PersistenceArtifactRequiresOutputUri => formatter.write_str(
                "environment capture artifact identity requires a persistence output URI",
            ),
            Self::PersistenceArtifactSourceLayoutMismatch {
                expected_face_size,
                expected_mip_count,
                actual_face_size,
                actual_mip_count,
            } => write!(
                formatter,
                "environment capture artifact source layout mismatch: expected {expected_face_size} face / {expected_mip_count} mips, got {actual_face_size} face / {actual_mip_count} mips"
            ),
            Self::PersistenceArtifactPmremLayoutMismatch {
                expected_face_size,
                expected_mip_count,
                actual_face_size,
                actual_mip_count,
            } => write!(
                formatter,
                "environment capture artifact PMREM layout mismatch: expected {expected_face_size} face / {expected_mip_count} mips, got {actual_face_size} face / {actual_mip_count} mips"
            ),
            Self::UnsupportedPersistenceArtifactContents(bits) => write!(
                formatter,
                "environment capture artifact contents are unsupported: bits=0x{bits:08x}"
            ),
            Self::NonFinitePosition => {
                formatter.write_str("environment capture position must be finite")
            }
            Self::InvalidNearPlane(value) => write!(
                formatter,
                "environment capture near plane is invalid: {value}"
            ),
            Self::InvalidFarPlane { near, far } => write!(
                formatter,
                "environment capture far plane is invalid: near={near}, far={far}"
            ),
            Self::InvalidFaceSize(value) => write!(
                formatter,
                "environment capture face size is unsupported: {value}"
            ),
        }
    }
}

impl std::error::Error for RenderEnvironmentCaptureRequestError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderEnvironmentCaptureStatus {
    handle: RenderEnvironmentCaptureHandle,
    phase: RenderEnvironmentCapturePhase,
    completed_work_items: u32,
    total_work_items: u32,
    output: Option<RenderEnvironmentCaptureOutputIdentity>,
    diagnostic: Option<String>,
}

impl RenderEnvironmentCaptureStatus {
    pub fn new(
        handle: RenderEnvironmentCaptureHandle,
        phase: RenderEnvironmentCapturePhase,
        completed_work_items: u32,
        total_work_items: u32,
        output: Option<RenderEnvironmentCaptureOutputIdentity>,
        diagnostic: Option<String>,
    ) -> Result<Self, RenderEnvironmentCaptureStatusError> {
        if total_work_items == 0 || completed_work_items > total_work_items {
            return Err(RenderEnvironmentCaptureStatusError::InvalidProgress {
                completed: completed_work_items,
                total: total_work_items,
            });
        }
        if !phase.is_terminal() && output.is_some() {
            return Err(RenderEnvironmentCaptureStatusError::OutputBeforeTerminal);
        }
        if matches!(phase, RenderEnvironmentCapturePhase::Succeeded) && output.is_none() {
            return Err(RenderEnvironmentCaptureStatusError::MissingSuccessfulOutput);
        }
        if !matches!(phase, RenderEnvironmentCapturePhase::Succeeded) && output.is_some() {
            return Err(RenderEnvironmentCaptureStatusError::OutputForUnsuccessfulCapture);
        }
        Ok(Self {
            handle,
            phase,
            completed_work_items,
            total_work_items,
            output,
            diagnostic,
        })
    }

    pub fn queued(
        handle: RenderEnvironmentCaptureHandle,
    ) -> Result<Self, RenderEnvironmentCaptureStatusError> {
        Self::new(
            handle,
            RenderEnvironmentCapturePhase::Queued,
            0,
            RENDER_ENVIRONMENT_CAPTURE_WORK_ITEM_COUNT,
            None,
            None,
        )
    }

    pub const fn handle(&self) -> RenderEnvironmentCaptureHandle {
        self.handle
    }
    pub const fn phase(&self) -> RenderEnvironmentCapturePhase {
        self.phase
    }
    pub const fn completed_work_items(&self) -> u32 {
        self.completed_work_items
    }
    pub const fn total_work_items(&self) -> u32 {
        self.total_work_items
    }
    pub fn output(&self) -> Option<&RenderEnvironmentCaptureOutputIdentity> {
        self.output.as_ref()
    }
    pub fn diagnostic(&self) -> Option<&str> {
        self.diagnostic.as_deref()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderEnvironmentCaptureStatusError {
    InvalidProgress { completed: u32, total: u32 },
    OutputBeforeTerminal,
    MissingSuccessfulOutput,
    OutputForUnsuccessfulCapture,
}

impl std::fmt::Display for RenderEnvironmentCaptureStatusError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidProgress { completed, total } => write!(
                formatter,
                "invalid environment capture progress {completed}/{total}"
            ),
            Self::OutputBeforeTerminal => {
                formatter.write_str("environment capture output is not terminal")
            }
            Self::MissingSuccessfulOutput => {
                formatter.write_str("successful environment capture requires output identity")
            }
            Self::OutputForUnsuccessfulCapture => {
                formatter.write_str("environment capture output requires success")
            }
        }
    }
}

impl std::error::Error for RenderEnvironmentCaptureStatusError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_keeps_revision_identity_and_rejects_invalid_position() {
        let request = RenderEnvironmentCaptureRequest::new("atrium", [1.0, 2.0, 3.0], 9)
            .unwrap()
            .with_clip_planes(0.25, 512.0)
            .unwrap()
            .with_face_size(256)
            .unwrap()
            .with_quality(SourceCubemapPrefilterQuality::High);
        assert_eq!(request.capture_id(), "atrium");
        assert_eq!(request.scene_revision(), 9);
        assert_eq!(request.environment_revision(), 9);
        assert_eq!(request.output_generation(), 9);
        assert_eq!(request.face_size(), 256);
        assert_eq!(request.quality(), SourceCubemapPrefilterQuality::High);

        assert_eq!(
            RenderEnvironmentCaptureRequest::new("atrium", [f32::NAN; 3], 9),
            Err(RenderEnvironmentCaptureRequestError::NonFinitePosition)
        );
    }

    #[test]
    fn typed_probe_target_survives_request_mutators_and_output_identity() {
        let cubemap = ResourceId::from_stable_label("lib://probes/atrium.zcube");
        let request = RenderEnvironmentCaptureRequest::new("atrium", [1.0, 2.0, 3.0], 9)
            .unwrap()
            .with_reflection_probe_target(42, cubemap)
            .with_clip_planes(0.25, 512.0)
            .unwrap()
            .with_face_size(256)
            .unwrap();

        assert_eq!(request.reflection_probe_target(), Some((42, cubemap)));
        let output = RenderEnvironmentCaptureOutputIdentity::from_request(&request);
        assert_eq!(output.reflection_probe_target(), Some((42, cubemap)));
        assert!(output.has_reflection_probe_target());
    }

    #[test]
    fn persistence_target_keeps_asset_and_runtime_cache_identities_distinct() {
        let base = RenderEnvironmentCaptureRequest::new("atrium", [1.0, 2.0, 3.0], 9)
            .unwrap()
            .with_face_size(256)
            .unwrap();
        let artifact_request = IblBakeArtifactRequest::new(
            IblBakeKey::source_cubemap(9, [1, 2, 3, 4]),
            256,
            source_cubemap_mip_count(256),
        );
        let request = base
            .clone()
            .with_persistence_output_uri("lib://probes/atrium.zcube")
            .unwrap()
            .with_persistence_artifact_request(artifact_request)
            .unwrap()
            .with_clip_planes(0.25, 512.0)
            .unwrap()
            .with_quality(SourceCubemapPrefilterQuality::High);

        assert_eq!(
            request.persistence_output_uri(),
            Some("lib://probes/atrium.zcube")
        );
        let output = RenderEnvironmentCaptureOutputIdentity::from_request(&request);
        assert_eq!(
            output.persistence_output_uri(),
            Some("lib://probes/atrium.zcube")
        );
        assert_eq!(
            output.persistence_artifact_request(),
            Some(artifact_request)
        );
        let runtime_cache_request = output
            .runtime_cache_artifact_request()
            .expect("persisted capture must expose its renderer-owned cache identity");
        assert_eq!(
            runtime_cache_request,
            request.runtime_cache_artifact_request().unwrap()
        );
        assert_eq!(runtime_cache_request.bake_key(), request.ibl_bake_key());
        assert_ne!(
            runtime_cache_request.bake_key(),
            artifact_request.bake_key()
        );
        assert_eq!(runtime_cache_request.source_face_size(), 256);
        assert_eq!(runtime_cache_request.source_mip_count(), 9);
        assert_eq!(
            RenderEnvironmentCaptureRequest::new("atrium", [1.0, 2.0, 3.0], 9)
                .unwrap()
                .ibl_bake_key(),
            RenderEnvironmentCaptureRequest::new("atrium", [1.0, 2.0, 3.0], 9)
                .unwrap()
                .with_persistence_output_uri("lib://other/path.zcube")
                .unwrap()
                .ibl_bake_key()
        );
        assert!(matches!(
            base.clone()
                .with_persistence_output_uri("lib://probes/atrium.zcube")
                .unwrap()
                .with_persistence_artifact_request(artifact_request)
                .unwrap()
                .with_face_size(128),
            Err(
                RenderEnvironmentCaptureRequestError::PersistenceArtifactSourceLayoutMismatch { .. }
            )
        ));
        assert_eq!(
            RenderEnvironmentCaptureRequest::new("atrium", [1.0, 2.0, 3.0], 9)
                .unwrap()
                .with_persistence_output_uri("   "),
            Err(RenderEnvironmentCaptureRequestError::EmptyPersistenceOutputUri)
        );
        assert_eq!(
            RenderEnvironmentCaptureRequest::new("atrium", [1.0, 2.0, 3.0], 9)
                .unwrap()
                .with_persistence_artifact_request(artifact_request),
            Err(RenderEnvironmentCaptureRequestError::PersistenceArtifactRequiresOutputUri)
        );
        assert!(matches!(
            RenderEnvironmentCaptureRequest::new("atrium", [1.0, 2.0, 3.0], 9)
                .unwrap()
                .with_persistence_output_uri("lib://probes/atrium.zcube")
                .unwrap()
                .with_persistence_artifact_request(artifact_request.with_pmrem_layout(64, 1)),
            Err(
                RenderEnvironmentCaptureRequestError::PersistenceArtifactPmremLayoutMismatch { .. }
            )
        ));
        assert!(matches!(
            RenderEnvironmentCaptureRequest::new("atrium", [1.0, 2.0, 3.0], 9)
                .unwrap()
                .with_persistence_output_uri("lib://probes/atrium.zcube")
                .unwrap()
                .with_persistence_artifact_request(
                    artifact_request.with_required_contents(IblBakeArtifactContents::IEM)
                ),
            Err(RenderEnvironmentCaptureRequestError::UnsupportedPersistenceArtifactContents(
                bits
            )) if bits == IblBakeArtifactContents::IEM.bits()
        ));
    }

    #[test]
    fn capture_ibl_key_is_stable_and_separates_quality_and_scene_identity() {
        let base = RenderEnvironmentCaptureRequest::with_revisions(
            "atrium",
            [1.0, 2.0, 3.0],
            0.25,
            512.0,
            256,
            SourceCubemapPrefilterQuality::Normal,
            9,
            10,
            11,
        )
        .unwrap();
        let same = base.clone();
        let high = base
            .clone()
            .with_quality(SourceCubemapPrefilterQuality::High);
        let layer_7 = base
            .clone()
            .with_capture_layer_mask(RenderLayerSet::layer(7));
        let newer_scene = RenderEnvironmentCaptureRequest::with_revisions(
            "atrium",
            [1.0, 2.0, 3.0],
            0.25,
            512.0,
            256,
            SourceCubemapPrefilterQuality::Normal,
            12,
            10,
            11,
        )
        .unwrap();

        assert_eq!(base.ibl_bake_key(), same.ibl_bake_key());
        assert_ne!(base.ibl_bake_key(), high.ibl_bake_key());
        assert_ne!(base.ibl_bake_key(), layer_7.ibl_bake_key());
        assert_ne!(base.ibl_bake_key(), newer_scene.ibl_bake_key());

        let positive_zero_position =
            RenderEnvironmentCaptureRequest::new("signed-zero", [0.0, 1.0, 2.0], 1).unwrap();
        let negative_zero_position =
            RenderEnvironmentCaptureRequest::new("signed-zero", [-0.0, 1.0, 2.0], 1).unwrap();
        assert_eq!(positive_zero_position, negative_zero_position);
        assert_eq!(
            positive_zero_position.ibl_bake_key(),
            negative_zero_position.ibl_bake_key()
        );
        assert_eq!(
            negative_zero_position.position()[0].to_bits(),
            0.0_f32.to_bits()
        );
    }

    #[test]
    fn capture_layer_mask_is_explicit_and_defaults_to_all_scene_schema_v1_layers() {
        let default_request = RenderEnvironmentCaptureRequest::new("atrium", [0.0; 3], 1).unwrap();
        let sky_only = default_request
            .clone()
            .with_capture_layer_mask(RenderLayerSet::none());

        assert_eq!(
            default_request
                .capture_layer_mask()
                .to_scene_schema_v1_mask_lossy(),
            u32::MAX
        );
        assert!(sky_only.capture_layer_mask().is_empty());
        assert_ne!(default_request.ibl_bake_key(), sky_only.ibl_bake_key());
        let default_output = RenderEnvironmentCaptureOutputIdentity::from_request(&default_request);
        let sky_only_output = RenderEnvironmentCaptureOutputIdentity::from_request(&sky_only);
        assert_eq!(
            default_output
                .capture_layer_mask()
                .to_scene_schema_v1_mask_lossy(),
            u32::MAX
        );
        assert!(sky_only_output.capture_layer_mask().is_empty());
        assert_ne!(default_output, sky_only_output);
    }

    #[test]
    fn capture_ibl_key_versions_raster_source_independently_of_filter_recipe() {
        let request = RenderEnvironmentCaptureRequest::new("atrium", [1.0, 2.0, 3.0], 9).unwrap();

        assert_eq!(
            request.ibl_bake_key(),
            request.ibl_bake_key_with_raster_algorithm_version(
                RENDER_ENVIRONMENT_CAPTURE_RASTER_ALGORITHM_VERSION,
            ),
        );
        assert_ne!(
            request.ibl_bake_key(),
            request.ibl_bake_key_with_raster_algorithm_version(
                RENDER_ENVIRONMENT_CAPTURE_RASTER_ALGORITHM_VERSION - 1,
            ),
        );
    }

    #[test]
    fn status_rejects_output_before_terminal_phase() {
        let request = RenderEnvironmentCaptureRequest::new("atrium", [0.0; 3], 12).unwrap();
        let handle = RenderEnvironmentCaptureHandle::new(7).unwrap();
        let output = RenderEnvironmentCaptureOutputIdentity::from_request(&request);
        assert_eq!(
            RenderEnvironmentCaptureStatus::queued(handle)
                .unwrap()
                .phase(),
            RenderEnvironmentCapturePhase::Queued
        );
        assert_eq!(
            RenderEnvironmentCaptureStatus::new(
                handle,
                RenderEnvironmentCapturePhase::Capturing,
                1,
                6,
                Some(output),
                None
            ),
            Err(RenderEnvironmentCaptureStatusError::OutputBeforeTerminal)
        );
        assert_eq!(
            RenderEnvironmentCaptureStatus::new(
                handle,
                RenderEnvironmentCapturePhase::Succeeded,
                6,
                6,
                None,
                None
            ),
            Err(RenderEnvironmentCaptureStatusError::MissingSuccessfulOutput)
        );
    }
}
