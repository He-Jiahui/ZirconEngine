mod ambient;
mod cubemap_projection;
mod environment_brdf_lut;
mod extract;
mod ibl_bake_artifact;
mod ibl_bake_artifact_blob;
mod ibl_bake_artifact_readback;
mod ibl_bake_artifact_resolution;
mod lightmap;
mod reflection_probe;
mod rgba16f;
mod skybox;
mod source_cubemap;
mod source_cubemap_artifact;
mod source_irradiance_cubemap;

pub use ambient::{SH_L2_RGB_COEFFICIENT_COUNT, ShL2Rgb};
pub use cubemap_projection::{
    CubemapFace, cubemap_direction_from_scaled_uv, cubemap_face_scaled_uv_from_direction,
    cubemap_face_size_from_equirect_height, cubemap_scaled_uv_for_texel,
    cubemap_solid_angle_from_scaled_uv, cubemap_texel_direction, cubemap_texel_solid_angle,
    equirect_uv_from_direction,
};
pub use environment_brdf_lut::{
    ENVIRONMENT_BRDF_LUT_SAMPLE_COUNT, ENVIRONMENT_BRDF_LUT_SIZE, EnvironmentBrdfLutTexel,
    build_environment_brdf_lut, environment_brdf_lut_integrate, environment_brdf_lut_texel_index,
};
pub use extract::EnvironmentExtract;
pub use ibl_bake_artifact::{
    IBL_BAKE_ALGORITHM_VERSION, IBL_BAKE_ARTIFACT_HEADER_SIZE,
    IBL_BAKE_ARTIFACT_RGBA16F_TEXEL_SIZE_BYTES, IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES,
    IblBakeArtifactCandidate, IblBakeArtifactContents, IblBakeArtifactDescriptor,
    IblBakeArtifactHeader, IblBakeArtifactHeaderError, IblBakeArtifactPayload,
    IblBakeArtifactPayloadError, IblBakeArtifactRequest, IblBakeArtifactSelection,
    IblBakeArtifactSource, select_ibl_bake_artifact,
};
pub use ibl_bake_artifact_blob::{IblBakeArtifactBlob, IblBakeArtifactBlobError};
pub use ibl_bake_artifact_readback::{
    IblBakeArtifactReadbackError, IblBakeArtifactReadbackSectionKind,
    IblBakeArtifactReadbackSections,
};
pub use ibl_bake_artifact_resolution::{
    IblBakeArtifactBlobCandidate, IblBakeArtifactResolvedPayload, resolve_ibl_bake_artifact_payload,
};
pub use lightmap::{
    LIGHTMAP_CONSUME_CONTRACT_VERSION, LIGHTMAP_SCENE_SNAPSHOT_VERSION, LightProbeGridData,
    LightmapAtlasBudget, LightmapAtlasDescriptor, LightmapAtlasFormat, LightmapAtlasPage,
    LightmapBakeOutput, LightmapBakeRequest, LightmapBakeSceneSnapshot, LightmapConsumeContract,
    LightmapContractValidationError, LightmapInstanceSlot,
};
pub use reflection_probe::{
    ProbeBakeTiming, ProbeInfluenceShape, ReflectionProbeBlend, ReflectionProbeBlendEntry,
    ReflectionProbeData, ReflectionProbeValidationError, reflection_probe_box_project_direction,
    reflection_probe_influence_weight, select_reflection_probe_blend,
};
pub use rgba16f::{
    RGBA16F_TEXEL_SIZE_BYTES, append_rgb_as_rgba16f_texels, append_rgba16f_texels,
    decode_rgb_from_rgba16f_texels, decode_rgba16f_texels, encode_rgba16f_texels,
};
pub use skybox::{
    IblBakeKey, PROCEDURAL_SKY_DEFAULT_SOURCE_REVISION, ProceduralSkyParams, SkyboxMode,
    SkyboxSettings, SourceCubemapEnvironment, SourceCubemapUploadKey,
};
use source_cubemap::SourceCubemapPmremLayout;
pub use source_cubemap::{
    SOURCE_CUBEMAP_FACE_COUNT, SOURCE_CUBEMAP_IRRADIANCE_COEFFICIENT_COUNT,
    SOURCE_CUBEMAP_IRRADIANCE_SOURCE_FACE_SIZE, SOURCE_CUBEMAP_MAX_FACE_SIZE,
    SOURCE_CUBEMAP_MIN_FACE_SIZE, SOURCE_CUBEMAP_PMREM_FACE_SIZE, SOURCE_CUBEMAP_PMREM_MIP_COUNT,
    SOURCE_CUBEMAP_ROUGHEST_MIP, SOURCE_CUBEMAP_ROUGHNESS_MIP_SCALE, SourceCubemapIrradianceSh9,
    SourceCubemapMipChain, SourceCubemapPrefilterQuality, build_source_cubemap_from_captured_faces,
    build_source_cubemap_from_captured_faces_with_quality, build_source_cubemap_from_equirect,
    build_source_cubemap_from_source_mips, build_source_cubemap_from_source_mips_with_quality,
    source_cubemap_capture_hash, source_cubemap_evaluate_irradiance_sh9,
    source_cubemap_face_mip_offset, source_cubemap_face_size_from_equirect_height,
    source_cubemap_irradiance_mip_level, source_cubemap_mip_count, source_cubemap_mip_size,
    source_cubemap_pmrem_mip_from_roughness, source_cubemap_roughness_from_pmrem_mip,
    source_cubemap_sample_count,
};
pub use source_cubemap_artifact::{
    SourceCubemapBakeArtifactError, source_cubemap_environment_with_bake_artifact,
    source_cubemap_mip_chain_with_bake_artifact,
};
pub use source_irradiance_cubemap::{
    SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE, SourceCubemapIrradianceCube,
    build_source_cubemap_irradiance_cube, source_cubemap_sample_irradiance_cube,
};
