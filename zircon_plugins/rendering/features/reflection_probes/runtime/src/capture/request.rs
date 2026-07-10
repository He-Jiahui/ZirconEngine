use serde::{Deserialize, Serialize};
use thiserror::Error;
use zircon_runtime::core::framework::render::{
    source_cubemap_mip_count, IblBakeArtifactRequest, IblBakeKey, SourceCubemapPrefilterQuality,
    SOURCE_CUBEMAP_MAX_FACE_SIZE, SOURCE_CUBEMAP_MIN_FACE_SIZE,
};

pub const REFLECTION_PROBE_CAPTURE_REQUEST_SCHEMA_VERSION: u32 = 1;

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
        .with_quality(ReflectionProbeCaptureQuality::High);

        let json = request.encode_json().unwrap();
        let decoded = ReflectionProbeCaptureRequest::decode_json(&json).unwrap();

        assert_eq!(decoded, request);
        assert_eq!(decoded.ibl_bake_request([1, 2, 3, 4]).mip_count(), 9);
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
}
