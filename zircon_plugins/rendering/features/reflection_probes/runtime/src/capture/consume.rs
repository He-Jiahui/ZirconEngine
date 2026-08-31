use std::fs;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use zircon_runtime::asset::artifact::{
    IblBakeArtifactBlob, IblBakeArtifactCacheError, IblBakeArtifactCacheRead,
    IblBakeArtifactCacheStore,
};
use zircon_runtime::asset::assets::{
    encode_source_cubemap_zcube_rgba16f_mips_owned, texture_asset_from_ibl_bake_artifact_pmrem,
    IblPmremTextureError, TextureAsset, ZcubeSourceCubemapError,
};
use zircon_runtime::asset::pipeline::manager::ProjectAssetManager;
use zircon_runtime::asset::AssetUri;
use zircon_runtime::core::framework::render::{
    ProbeInfluenceShape, ReflectionProbeData, ReflectionProbeValidationError,
    RenderEnvironmentCaptureSourcePayload, RenderLayerSet,
};
use zircon_runtime::core::math::{Quat, Vec3};
use zircon_runtime::core::resource::{ResourceId, ResourceKind, ResourceRecord};

use zircon_runtime::asset::artifact::IblSourceCubemapStagedBundleReport;

use super::ReflectionProbeCaptureRequest;

#[derive(Debug)]
pub struct PersistedReflectionProbeCapture {
    source_hash: [u32; 4],
    staged_bundle: IblSourceCubemapStagedBundleReport,
}

pub struct EncodedReflectionProbeCaptureSource {
    output_uri: AssetUri,
    bytes: Vec<u8>,
}

impl EncodedReflectionProbeCaptureSource {
    pub fn output_uri(&self) -> &AssetUri {
        &self.output_uri
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn into_parts(self) -> (AssetUri, Vec<u8>) {
        (self.output_uri, self.bytes)
    }
}

impl std::fmt::Debug for EncodedReflectionProbeCaptureSource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("EncodedReflectionProbeCaptureSource")
            .field("output_uri", &self.output_uri)
            .field("bytes", &self.bytes.len())
            .finish()
    }
}

pub fn encode_reflection_probe_capture_source(
    payload: RenderEnvironmentCaptureSourcePayload,
) -> Result<EncodedReflectionProbeCaptureSource, CapturedReflectionProbeConsumeError> {
    let output_uri = payload
        .output()
        .persistence_output_uri()
        .ok_or(CapturedReflectionProbeConsumeError::MissingSourceOutputUri)?;
    let output_uri = AssetUri::parse(output_uri)
        .map_err(|error| CapturedReflectionProbeConsumeError::SourceOutputUri(error.to_string()))?;
    let face_size = payload.face_size();
    let mip_count = payload.mip_count();
    let bytes = encode_source_cubemap_zcube_rgba16f_mips_owned(
        face_size,
        mip_count,
        payload.into_source_rgba16f_bytes(),
    )?;
    Ok(EncodedReflectionProbeCaptureSource { output_uri, bytes })
}

impl PersistedReflectionProbeCapture {
    pub fn new(source_hash: [u32; 4], staged_bundle: IblSourceCubemapStagedBundleReport) -> Self {
        Self {
            source_hash,
            staged_bundle,
        }
    }

    pub const fn source_hash(&self) -> [u32; 4] {
        self.source_hash
    }

    pub fn staged_bundle(&self) -> &IblSourceCubemapStagedBundleReport {
        &self.staged_bundle
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CapturedReflectionProbeInfluence {
    Box {
        half_extents: [f32; 3],
        blend_distance: f32,
    },
    Sphere {
        radius: f32,
        blend_distance: f32,
    },
}

impl CapturedReflectionProbeInfluence {
    fn runtime_shape(self) -> Result<ProbeInfluenceShape, ReflectionProbeValidationError> {
        match self {
            Self::Box {
                half_extents,
                blend_distance,
            } => ProbeInfluenceShape::box_shape(Vec3::from_array(half_extents), blend_distance),
            Self::Sphere {
                radius,
                blend_distance,
            } => ProbeInfluenceShape::sphere(radius, blend_distance),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapturedReflectionProbePlacement {
    pub probe_id: u64,
    pub pmrem_uri: String,
    pub rotation: [f32; 4],
    pub influence: CapturedReflectionProbeInfluence,
    pub box_projection: bool,
    pub projection_half_extents: [f32; 3],
    pub intensity: f32,
    pub priority: i32,
    pub layer_mask: u32,
}

impl CapturedReflectionProbePlacement {
    pub fn box_probe(
        probe_id: u64,
        pmrem_uri: impl Into<String>,
        half_extents: [f32; 3],
        blend_distance: f32,
    ) -> Self {
        Self {
            probe_id,
            pmrem_uri: pmrem_uri.into(),
            rotation: [0.0, 0.0, 0.0, 1.0],
            influence: CapturedReflectionProbeInfluence::Box {
                half_extents,
                blend_distance,
            },
            box_projection: true,
            projection_half_extents: half_extents,
            intensity: 1.0,
            priority: 0,
            layer_mask: u32::MAX,
        }
    }

    pub fn validate(&self) -> Result<(), CapturedReflectionProbeConsumeError> {
        if self.pmrem_uri.trim().is_empty() {
            return Err(CapturedReflectionProbeConsumeError::EmptyPmremUri);
        }
        let _ = self.runtime_probe(Vec3::ZERO, None)?;
        Ok(())
    }

    pub fn encode_json(&self) -> Result<String, CapturedReflectionProbeConsumeError> {
        self.validate()?;
        serde_json::to_string_pretty(self).map_err(|error| {
            CapturedReflectionProbeConsumeError::SerializePlacement(error.to_string())
        })
    }

    pub fn decode_json(json: &str) -> Result<Self, CapturedReflectionProbeConsumeError> {
        let placement: Self = serde_json::from_str(json).map_err(|error| {
            CapturedReflectionProbeConsumeError::DeserializePlacement(error.to_string())
        })?;
        placement.validate()?;
        Ok(placement)
    }

    fn runtime_probe(
        &self,
        position: Vec3,
        cubemap: Option<ResourceId>,
    ) -> Result<ReflectionProbeData, CapturedReflectionProbeConsumeError> {
        let shape = self.influence.runtime_shape()?;
        let probe = ReflectionProbeData::try_new(
            self.probe_id,
            position,
            Quat::from_array(self.rotation),
            shape,
            Vec3::from_array(self.projection_half_extents),
        )?
        .with_box_projection(self.box_projection)
        .with_baked_cubemap(cubemap)
        .with_priority(self.priority)
        .with_layer_mask(RenderLayerSet::from_scene_schema_v1_mask(self.layer_mask))
        .try_with_intensity(self.intensity)?;
        Ok(probe)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CapturedReflectionProbeAsset {
    pub texture_id: ResourceId,
    pub probe: ReflectionProbeData,
}

pub fn register_captured_reflection_probe(
    asset_manager: &ProjectAssetManager,
    request: &ReflectionProbeCaptureRequest,
    capture: &PersistedReflectionProbeCapture,
    placement: &CapturedReflectionProbePlacement,
) -> Result<CapturedReflectionProbeAsset, CapturedReflectionProbeConsumeError> {
    request.validate()?;
    placement.validate()?;
    let bytes = fs::read(capture.staged_bundle.asset_derived().path()).map_err(|source| {
        CapturedReflectionProbeConsumeError::ReadArtifact {
            path: capture.staged_bundle.asset_derived().path().to_path_buf(),
            source,
        }
    })?;
    let blob = IblBakeArtifactBlob::decode_current_for_request(
        &request.ibl_bake_request(capture.source_hash),
        &bytes,
    )
    .map_err(|error| CapturedReflectionProbeConsumeError::Artifact(format!("{error:?}")))?;
    register_captured_reflection_probe_blob(asset_manager, request, placement, &blob)
}

/// Registers a completed GPU capture directly from the runtime cache.
///
/// Runtime cache artifacts are deliberately distinct from asset-derived bundles: this entry
/// requires explicit persistence intent and resolves the renderer-owned capture recipe key. It
/// never manufactures a `.zcube` source asset from runtime-cache bytes.
pub fn register_captured_reflection_probe_from_runtime_cache(
    asset_manager: &ProjectAssetManager,
    cache_store: &IblBakeArtifactCacheStore,
    request: &ReflectionProbeCaptureRequest,
    placement: &CapturedReflectionProbePlacement,
) -> Result<CapturedReflectionProbeAsset, CapturedReflectionProbeConsumeError> {
    request.validate()?;
    placement.validate()?;
    request
        .source_hash()
        .ok_or(CapturedReflectionProbeConsumeError::MissingSourceHash)?;
    let artifact_request = request
        .render_request()?
        .runtime_cache_artifact_request()
        .ok_or(CapturedReflectionProbeConsumeError::MissingRuntimeCacheIdentity)?;
    let blob = match cache_store
        .read_runtime_cache(&artifact_request)
        .map_err(CapturedReflectionProbeConsumeError::RuntimeCache)?
    {
        IblBakeArtifactCacheRead::Hit(blob) => blob,
        IblBakeArtifactCacheRead::Missing => {
            return Err(CapturedReflectionProbeConsumeError::RuntimeCacheMissing {
                path: cache_store.runtime_cache_path(&artifact_request),
            });
        }
        IblBakeArtifactCacheRead::Rejected(error) => {
            return Err(CapturedReflectionProbeConsumeError::RuntimeCacheArtifact(
                format!("{error:?}"),
            ));
        }
    };
    register_captured_reflection_probe_blob(asset_manager, request, placement, &blob)
}

fn register_captured_reflection_probe_blob(
    asset_manager: &ProjectAssetManager,
    request: &ReflectionProbeCaptureRequest,
    placement: &CapturedReflectionProbePlacement,
    blob: &IblBakeArtifactBlob,
) -> Result<CapturedReflectionProbeAsset, CapturedReflectionProbeConsumeError> {
    let uri = AssetUri::parse(&placement.pmrem_uri)
        .map_err(|error| CapturedReflectionProbeConsumeError::PmremUri(error.to_string()))?;
    let texture_id = ResourceId::from_locator(&uri);
    let texture = texture_asset_from_ibl_bake_artifact_pmrem(uri.clone(), &blob)?;
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(texture_id, ResourceKind::Texture, uri),
            texture,
        )
        .ok_or(CapturedReflectionProbeConsumeError::RegisterTexture)?;
    let probe = placement.runtime_probe(Vec3::from_array(request.position), Some(texture_id))?;
    Ok(CapturedReflectionProbeAsset { texture_id, probe })
}

#[derive(Debug, Error)]
pub enum CapturedReflectionProbeConsumeError {
    #[error(transparent)]
    Request(#[from] super::ReflectionProbeCaptureRequestError),
    #[error("captured reflection-probe PMREM URI must not be empty")]
    EmptyPmremUri,
    #[error(transparent)]
    Probe(#[from] ReflectionProbeValidationError),
    #[error("read captured reflection-probe artifact {path:?}: {source}")]
    ReadArtifact {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("captured reflection-probe artifact is invalid or stale: {0}")]
    Artifact(String),
    #[error("captured reflection-probe source payload has no output URI")]
    MissingSourceOutputUri,
    #[error("invalid captured reflection-probe source output URI: {0}")]
    SourceOutputUri(String),
    #[error(transparent)]
    SourceZcube(#[from] ZcubeSourceCubemapError),
    #[error("captured reflection-probe runtime cache requires an explicit source hash")]
    MissingSourceHash,
    #[error("captured reflection-probe request did not produce a runtime-cache identity")]
    MissingRuntimeCacheIdentity,
    #[error("read captured reflection-probe runtime cache: {0}")]
    RuntimeCache(#[source] IblBakeArtifactCacheError),
    #[error("captured reflection-probe runtime cache is missing at {path:?}")]
    RuntimeCacheMissing { path: std::path::PathBuf },
    #[error("captured reflection-probe runtime cache is invalid or stale: {0}")]
    RuntimeCacheArtifact(String),
    #[error("invalid captured reflection-probe PMREM URI: {0}")]
    PmremUri(String),
    #[error(transparent)]
    Pmrem(#[from] IblPmremTextureError),
    #[error("register captured reflection-probe PMREM texture")]
    RegisterTexture,
    #[error("serialize captured reflection-probe placement: {0}")]
    SerializePlacement(String),
    #[error("deserialize captured reflection-probe placement: {0}")]
    DeserializePlacement(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_cache_registration_requires_explicit_source_hash_before_io() {
        let request = ReflectionProbeCaptureRequest::new(
            "atrium",
            "lib://probes/atrium.zcube",
            [0.0, 1.0, 0.0],
            1,
        );
        let placement = CapturedReflectionProbePlacement::box_probe(
            17,
            "lib://probes/atrium.pmrem",
            [4.0, 4.0, 4.0],
            1.0,
        );
        let cache_store = IblBakeArtifactCacheStore::new("E:/zircon-runtime-cache-contract");

        let result = register_captured_reflection_probe_from_runtime_cache(
            &ProjectAssetManager::default(),
            &cache_store,
            &request,
            &placement,
        );

        assert!(matches!(
            result,
            Err(CapturedReflectionProbeConsumeError::MissingSourceHash)
        ));
    }

    #[test]
    fn runtime_cache_registration_stays_distinct_from_asset_derived_source_staging() {
        let source = include_str!("consume.rs");
        let runtime = source
            .split_once("pub fn register_captured_reflection_probe_from_runtime_cache(")
            .and_then(|(_, tail)| tail.split_once("fn register_captured_reflection_probe_blob"))
            .map(|(runtime, _)| runtime)
            .expect("runtime cache registration owner");

        assert!(runtime.contains("read_runtime_cache(&artifact_request)"));
        assert!(runtime.contains("MissingSourceHash"));
        assert!(runtime.contains("runtime_cache_artifact_request()"));
        assert!(!runtime.contains("ibl_bake_request(source_hash)"));
        assert!(!runtime.contains("IblSourceCubemapStagingStore"));
        assert!(!runtime.contains("write_source_cubemap"));
    }

    #[test]
    fn captured_probe_placement_json_roundtrip_preserves_runtime_fields() {
        let placement = CapturedReflectionProbePlacement::box_probe(
            17,
            "lib://reflection-probes/atrium.pmrem",
            [8.0, 4.0, 6.0],
            1.5,
        );
        let json = placement.encode_json().unwrap();
        let decoded = CapturedReflectionProbePlacement::decode_json(&json).unwrap();

        assert_eq!(decoded, placement);
        decoded.validate().unwrap();
        assert!(!json.contains("bake_timing"));

        let mut legacy = serde_json::to_value(&placement).unwrap();
        legacy.as_object_mut().expect("placement object").insert(
            "bake_timing".to_owned(),
            serde_json::Value::String("EditorManual".to_owned()),
        );
        assert!(CapturedReflectionProbePlacement::decode_json(&legacy.to_string()).is_err());
    }
}
