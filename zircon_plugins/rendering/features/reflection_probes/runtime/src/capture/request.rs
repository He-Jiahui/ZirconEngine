use serde::{Deserialize, Serialize};
use thiserror::Error;
use zircon_runtime::core::framework::render::{
    source_cubemap_mip_count, IblBakeArtifactRequest, IblBakeKey, RenderEnvironmentCaptureRequest,
    RenderEnvironmentCaptureRequestError, RenderLayerSet, SourceCubemapPrefilterQuality,
    SOURCE_CUBEMAP_MAX_FACE_SIZE, SOURCE_CUBEMAP_MIN_FACE_SIZE,
};

pub const REFLECTION_PROBE_CAPTURE_REQUEST_SCHEMA_VERSION: u32 = 2;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReflectionProbeCaptureQuality {
    Fast,
    #[default]
    Normal,
    High,
}

impl ReflectionProbeCaptureQuality {
    pub const fn source_prefilter_quality(self) -> SourceCubemapPrefilterQuality {
        match self {
            Self::Fast => SourceCubemapPrefilterQuality::Fast,
            Self::Normal => SourceCubemapPrefilterQuality::Normal,
            Self::High => SourceCubemapPrefilterQuality::High,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReflectionProbeCaptureRequest {
    pub schema_version: u32,
    pub probe_id: String,
    pub output_uri: String,
    pub position: [f32; 3],
    pub near_plane: f32,
    pub far_plane: f32,
    pub face_size: u32,
    pub quality: ReflectionProbeCaptureQuality,
    pub source_revision: u64,
    /// Geometry layers rendered into the cubemap. This is independent from the
    /// placement layer mask that selects receivers of the finished probe.
    pub capture_layer_mask: u32,
    /// Optional source identity supplied by the asset/editor owner. When present,
    /// the capture also requests the validated runtime-cache artifact writeback.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_hash: Option<[u32; 4]>,
}

impl ReflectionProbeCaptureRequest {
    pub fn new(
        probe_id: impl Into<String>,
        output_uri: impl Into<String>,
        position: [f32; 3],
        source_revision: u64,
    ) -> Self {
        Self {
            schema_version: REFLECTION_PROBE_CAPTURE_REQUEST_SCHEMA_VERSION,
            probe_id: probe_id.into(),
            output_uri: output_uri.into(),
            position,
            near_plane: 0.1,
            far_plane: 200.0,
            face_size: 128,
            quality: ReflectionProbeCaptureQuality::Normal,
            source_revision,
            capture_layer_mask: u32::MAX,
            source_hash: None,
        }
    }

    pub fn with_clip_planes(mut self, near_plane: f32, far_plane: f32) -> Self {
        self.near_plane = near_plane;
        self.far_plane = far_plane;
        self
    }

    pub fn with_face_size(mut self, face_size: u32) -> Self {
        self.face_size = face_size;
        self
    }

    pub fn with_quality(mut self, quality: ReflectionProbeCaptureQuality) -> Self {
        self.quality = quality;
        self
    }

    pub fn with_capture_layer_mask(mut self, capture_layer_mask: u32) -> Self {
        self.capture_layer_mask = capture_layer_mask;
        self
    }

    /// Associates the capture with the source cubemap content identity.
    ///
    /// The hash is intentionally explicit: a capture id, probe placement, or
    /// output URI cannot stand in for the source content hash used by IBL
    /// artifact resolution.
    pub fn with_source_hash(mut self, source_hash: [u32; 4]) -> Self {
        self.source_hash = Some(source_hash);
        self
    }

    pub const fn source_hash(&self) -> Option<[u32; 4]> {
        self.source_hash
    }

    pub fn validate(&self) -> Result<(), ReflectionProbeCaptureRequestError> {
        if self.schema_version != REFLECTION_PROBE_CAPTURE_REQUEST_SCHEMA_VERSION {
            return Err(
                ReflectionProbeCaptureRequestError::UnsupportedSchemaVersion(self.schema_version),
            );
        }
        if self.probe_id.trim().is_empty() {
            return Err(ReflectionProbeCaptureRequestError::EmptyProbeId);
        }
        if self.output_uri.trim().is_empty() {
            return Err(ReflectionProbeCaptureRequestError::EmptyOutputUri);
        }
        if !self.position.iter().all(|value| value.is_finite()) {
            return Err(ReflectionProbeCaptureRequestError::NonFinitePosition);
        }
        if !self.near_plane.is_finite() || self.near_plane <= 0.0 {
            return Err(ReflectionProbeCaptureRequestError::InvalidNearPlane(
                self.near_plane,
            ));
        }
        if !self.far_plane.is_finite() || self.far_plane <= self.near_plane {
            return Err(ReflectionProbeCaptureRequestError::InvalidFarPlane {
                near: self.near_plane,
                far: self.far_plane,
            });
        }
        if !self.face_size.is_power_of_two()
            || !(SOURCE_CUBEMAP_MIN_FACE_SIZE..=SOURCE_CUBEMAP_MAX_FACE_SIZE)
                .contains(&self.face_size)
        {
            return Err(ReflectionProbeCaptureRequestError::InvalidFaceSize(
                self.face_size,
            ));
        }
        Ok(())
    }

    pub fn encode_json(&self) -> Result<String, ReflectionProbeCaptureRequestError> {
        self.validate()?;
        serde_json::to_string_pretty(self)
            .map_err(|error| ReflectionProbeCaptureRequestError::Serialize(error.to_string()))
    }

    pub fn decode_json(json: &str) -> Result<Self, ReflectionProbeCaptureRequestError> {
        let request: Self = serde_json::from_str(json)
            .map_err(|error| ReflectionProbeCaptureRequestError::Deserialize(error.to_string()))?;
        request.validate()?;
        Ok(request)
    }

    pub fn ibl_bake_request(&self, source_hash: [u32; 4]) -> IblBakeArtifactRequest {
        IblBakeArtifactRequest::new(
            IblBakeKey::source_cubemap(self.source_revision, source_hash),
            self.face_size,
            source_cubemap_mip_count(self.face_size),
        )
    }

    pub fn render_request(
        &self,
    ) -> Result<RenderEnvironmentCaptureRequest, ReflectionProbeCaptureRequestError> {
        self.validate()?;
        let mut request = RenderEnvironmentCaptureRequest::with_revisions(
            self.probe_id.clone(),
            self.position,
            self.near_plane,
            self.far_plane,
            self.face_size,
            self.quality.source_prefilter_quality(),
            self.source_revision,
            self.source_revision,
            self.source_revision,
        )?
        .with_capture_layer_mask(RenderLayerSet::from_scene_schema_v1_mask(
            self.capture_layer_mask,
        ))
        .with_persistence_output_uri(self.output_uri.clone())?;
        if let Some(source_hash) = self.source_hash {
            request =
                request.with_persistence_artifact_request(self.ibl_bake_request(source_hash))?;
        }
        Ok(request)
    }

    /// Builds a render request with an explicit artifact identity supplied by the
    /// asset/editor owner. The identity is never inferred from the capture id.
    pub fn render_request_with_artifact_request(
        &self,
        artifact_request: IblBakeArtifactRequest,
    ) -> Result<RenderEnvironmentCaptureRequest, ReflectionProbeCaptureRequestError> {
        self.render_request()?
            .with_persistence_artifact_request(artifact_request)
            .map_err(ReflectionProbeCaptureRequestError::from)
    }
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum ReflectionProbeCaptureRequestError {
    #[error("unsupported reflection-probe capture schema version {0}")]
    UnsupportedSchemaVersion(u32),
    #[error("reflection-probe capture probe_id must not be empty")]
    EmptyProbeId,
    #[error("reflection-probe capture output_uri must not be empty")]
    EmptyOutputUri,
    #[error("reflection-probe capture position must be finite")]
    NonFinitePosition,
    #[error("reflection-probe capture near plane must be finite and positive, got {0}")]
    InvalidNearPlane(f32),
    #[error("reflection-probe capture far plane must be finite and greater than near; near={near}, far={far}")]
    InvalidFarPlane { near: f32, far: f32 },
    #[error("reflection-probe face size must be a supported power of two, got {0}")]
    InvalidFaceSize(u32),
    #[error("serialize reflection-probe capture request: {0}")]
    Serialize(String),
    #[error("deserialize reflection-probe capture request: {0}")]
    Deserialize(String),
    #[error(transparent)]
    RenderContract(#[from] RenderEnvironmentCaptureRequestError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_request_json_roundtrip_preserves_quality_and_clip_contract() {
        let request = ReflectionProbeCaptureRequest::new(
            "probe-lobby",
            "lib://probes/lobby.zcube",
            [1.0, 2.0, 3.0],
            7,
        )
        .with_clip_planes(0.25, 512.0)
        .with_face_size(256)
        .with_quality(ReflectionProbeCaptureQuality::High)
        .with_capture_layer_mask(0x0000_0042);

        let json = request.encode_json().unwrap();
        let decoded = ReflectionProbeCaptureRequest::decode_json(&json).unwrap();

        assert_eq!(decoded, request);
        assert_eq!(decoded.capture_layer_mask, 0x0000_0042);
        let ibl_request = decoded.ibl_bake_request([1, 2, 3, 4]);
        assert_eq!(ibl_request.source_mip_count(), 9);
        assert_eq!(ibl_request.pmrem_mip_count(), 8);

        let render_request = decoded.render_request().unwrap();
        assert_eq!(render_request.capture_id(), "probe-lobby");
        assert_eq!(render_request.scene_revision(), 7);
        assert_eq!(render_request.environment_revision(), 7);
        assert_eq!(render_request.output_generation(), 7);
        assert_eq!(
            render_request.quality(),
            SourceCubemapPrefilterQuality::High
        );
        assert_eq!(
            render_request
                .capture_layer_mask()
                .to_scene_schema_v1_mask_lossy(),
            0x0000_0042
        );
        assert_eq!(
            render_request.persistence_output_uri(),
            Some("lib://probes/lobby.zcube")
        );
        assert!(render_request.persistence_artifact_request().is_none());
        assert_eq!(decoded.source_hash(), None);

        let hashed = decoded
            .clone()
            .with_source_hash([9, 8, 7, 6])
            .render_request()
            .unwrap();
        assert_eq!(
            decoded.clone().with_source_hash([9, 8, 7, 6]).source_hash(),
            Some([9, 8, 7, 6])
        );
        assert_eq!(
            hashed.persistence_artifact_request(),
            Some(decoded.ibl_bake_request([9, 8, 7, 6]))
        );
        let runtime_cache_request = hashed
            .runtime_cache_artifact_request()
            .expect("hashed capture must request runtime-cache persistence");
        assert_eq!(runtime_cache_request.bake_key(), hashed.ibl_bake_key());
        assert_ne!(
            runtime_cache_request.bake_key(),
            hashed.persistence_artifact_request().unwrap().bake_key()
        );
        assert_eq!(
            runtime_cache_request.required_contents(),
            hashed
                .persistence_artifact_request()
                .unwrap()
                .required_contents()
        );

        let explicit = decoded
            .render_request_with_artifact_request(ibl_request)
            .unwrap();
        assert_eq!(explicit.persistence_artifact_request(), Some(ibl_request));
    }

    #[test]
    fn capture_request_rejects_non_power_of_two_face_size() {
        let request =
            ReflectionProbeCaptureRequest::new("probe", "lib://probes/probe.zcube", [0.0; 3], 1)
                .with_face_size(192);

        assert_eq!(
            request.validate(),
            Err(ReflectionProbeCaptureRequestError::InvalidFaceSize(192))
        );
    }

    #[test]
    fn capture_request_hard_rejects_v1_without_an_explicit_capture_mask() {
        let v1 = r#"{
            "schema_version": 1,
            "probe_id": "legacy",
            "output_uri": "lib://probes/legacy.zcube",
            "position": [0.0, 0.0, 0.0],
            "near_plane": 0.1,
            "far_plane": 200.0,
            "face_size": 128,
            "quality": "normal",
            "source_revision": 1
        }"#;

        assert!(ReflectionProbeCaptureRequest::decode_json(v1).is_err());
    }
}
