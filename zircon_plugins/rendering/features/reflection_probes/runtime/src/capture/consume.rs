use std::fs;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use zircon_runtime::asset::assets::{
    texture_asset_from_ibl_bake_artifact_pmrem, IblPmremTextureError, TextureAsset,
};
use zircon_runtime::asset::pipeline::manager::ProjectAssetManager;
use zircon_runtime::asset::AssetUri;
use zircon_runtime::core::framework::render::{
    IblBakeArtifactBlob, ProbeBakeTiming, ProbeInfluenceShape, ReflectionProbeData,
    ReflectionProbeValidationError, RenderLayerSet,
};
use zircon_runtime::core::math::{Quat, Vec3};
use zircon_runtime::core::resource::{ResourceId, ResourceKind, ResourceRecord};

use super::{ReflectionProbeCaptureReport, ReflectionProbeCaptureRequest};

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
    pub bake_timing: ProbeBakeTiming,
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
            bake_timing: ProbeBakeTiming::EditorManual,
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
        .with_bake_timing(self.bake_timing)
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
    report: &ReflectionProbeCaptureReport,
    placement: &CapturedReflectionProbePlacement,
) -> Result<CapturedReflectionProbeAsset, CapturedReflectionProbeConsumeError> {
    request.validate()?;
    placement.validate()?;
    let bytes = fs::read(report.staged_bundle.asset_derived().path()).map_err(|source| {
        CapturedReflectionProbeConsumeError::ReadArtifact {
            path: report.staged_bundle.asset_derived().path().to_path_buf(),
            source,
        }
    })?;
    let blob = IblBakeArtifactBlob::decode_current_for_request(
        &request.ibl_bake_request(report.source_hash),
        &bytes,
    )
    .map_err(|error| CapturedReflectionProbeConsumeError::Artifact(format!("{error:?}")))?;
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
    }
}
