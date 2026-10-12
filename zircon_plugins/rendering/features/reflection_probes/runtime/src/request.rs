use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use zircon_runtime::asset::artifact::IBL_SOURCE_CUBEMAP_STAGING_EXTENSION;
use zircon_runtime::asset::AssetUri;
use zircon_runtime::core::framework::render::{
    source_cubemap_mip_count, IblBakeArtifactRequest, IblBakeKey, RenderEnvironmentCaptureRequest,
    RenderEnvironmentCaptureRequestError, RenderLayerSet, SourceCubemapPrefilterQuality,
    SOURCE_CUBEMAP_MAX_FACE_SIZE, SOURCE_CUBEMAP_MIN_FACE_SIZE,
};
use zircon_runtime::core::resource::ResourceScheme;

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
    pub output_uri: AssetUri,
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
        output_uri: AssetUri,
        position: [f32; 3],
        source_revision: u64,
    ) -> Self {
        Self {
            schema_version: REFLECTION_PROBE_CAPTURE_REQUEST_SCHEMA_VERSION,
            probe_id: probe_id.into(),
            output_uri,
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
        validate_reflection_probe_capture_output_uri(&self.output_uri)?;
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
        .with_persistence_output_uri(self.output_uri.to_string())?;
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

pub(super) fn validate_reflection_probe_capture_output_uri(
    output_uri: &AssetUri,
) -> Result<(), ReflectionProbeCaptureRequestError> {
    if output_uri.scheme() != ResourceScheme::Res {
        return Err(
            ReflectionProbeCaptureRequestError::UnsupportedOutputUriScheme(output_uri.scheme()),
        );
    }
    if output_uri.label().is_some() {
        return Err(ReflectionProbeCaptureRequestError::OutputUriHasLabel);
    }
    let has_zcube_extension = Path::new(output_uri.path())
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case(IBL_SOURCE_CUBEMAP_STAGING_EXTENSION)
        });
    if !has_zcube_extension {
        return Err(
            ReflectionProbeCaptureRequestError::UnsupportedOutputUriExtension(
                output_uri.path().to_owned(),
            ),
        );
    }
    Ok(())
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum ReflectionProbeCaptureRequestError {
    #[error("unsupported reflection-probe capture schema version {0}")]
    UnsupportedSchemaVersion(u32),
    #[error("reflection-probe capture probe_id must not be empty")]
    EmptyProbeId,
    #[error("reflection-probe capture output_uri must use res://, got {0:?}")]
    UnsupportedOutputUriScheme(ResourceScheme),
    #[error("reflection-probe capture output_uri must not contain a subasset label")]
    OutputUriHasLabel,
    #[error("reflection-probe capture output_uri must end in .zcube, got {0}")]
    UnsupportedOutputUriExtension(String),
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
#[path = "tests/request.rs"]
mod tests;
