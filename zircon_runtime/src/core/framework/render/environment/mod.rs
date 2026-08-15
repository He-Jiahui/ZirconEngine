mod ambient;
mod cubemap_projection;
mod environment_brdf_lut;
mod extract;
mod ibl_bake_artifact;
mod ibl_bake_artifact_blob;
mod ibl_bake_artifact_readback;
mod ibl_bake_artifact_resolution;
mod ibl_bake_recipe;
mod irradiance_comparison;
mod lightmap;
mod reflection_probe;
mod rgba16f;
mod skybox;
mod source_cubemap;
mod source_cubemap_artifact;
mod source_cubemap_upload;
mod source_irradiance_cubemap;

pub use ambient::{ShL2Rgb, SH_L2_RGB_COEFFICIENT_COUNT};
pub use cubemap_projection::{
    cubemap_direction_from_scaled_uv, cubemap_face_scaled_uv_from_direction,
    cubemap_face_size_from_equirect_height, cubemap_scaled_uv_for_texel,
    cubemap_solid_angle_from_scaled_uv, cubemap_texel_direction, cubemap_texel_solid_angle,
    equirect_uv_from_direction, CubemapFace,
};
pub use environment_brdf_lut::{
    build_environment_brdf_lut, build_environment_brdf_lut_with_extent,
    environment_brdf_lut_integrate, environment_brdf_lut_texel_index, EnvironmentBrdfLutTexel,
    ENVIRONMENT_BRDF_LUT_HEIGHT, ENVIRONMENT_BRDF_LUT_SAMPLE_COUNT, ENVIRONMENT_BRDF_LUT_WIDTH,
};
pub use extract::EnvironmentExtract;
pub use ibl_bake_artifact::{
    select_ibl_bake_artifact, IblBakeArtifactCandidate, IblBakeArtifactContents,
    IblBakeArtifactDescriptor, IblBakeArtifactHeader, IblBakeArtifactHeaderError,
    IblBakeArtifactPayload, IblBakeArtifactPayloadError, IblBakeArtifactProducer,
    IblBakeArtifactRequest, IblBakeArtifactSelection, IblBakeArtifactSource,
    IBL_BAKE_ALGORITHM_VERSION, IBL_BAKE_ARTIFACT_HEADER_SIZE,
    IBL_BAKE_ARTIFACT_PAYLOAD_CHECKSUM_SIZE, IBL_BAKE_ARTIFACT_RGBA16F_TEXEL_SIZE_BYTES,
    IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES,
};
pub use ibl_bake_artifact_blob::{IblBakeArtifactBlob, IblBakeArtifactBlobError};
pub use ibl_bake_artifact_readback::{
    IblBakeArtifactReadbackError, IblBakeArtifactReadbackSectionKind,
    IblBakeArtifactReadbackSections,
};
pub use ibl_bake_artifact_resolution::{
    resolve_ibl_bake_artifact_payload, IblBakeArtifactBlobCandidate, IblBakeArtifactResolvedPayload,
};
pub use ibl_bake_recipe::{
    IblBakeDiffuseIntegrator, IblBakeDiffuseRepresentation, IblBakeOutputFormat,
    IblBakePmremIntegrator, IblBakeRecipe, IblBakeRecipeIdentity,
    CANONICAL_IBL_BAKE_ALGORITHM_VERSION, CANONICAL_IBL_BAKE_DIFFUSE_SOURCE_FACE_SIZE,
    CANONICAL_IBL_BAKE_IRRADIANCE_CUBE_FACE_SIZE, CANONICAL_IBL_BAKE_RECIPE,
    CANONICAL_IBL_BAKE_ROUGHEST_MIP_OFFSET, CANONICAL_IBL_BAKE_ROUGHNESS_MIP_SCALE,
};
pub use irradiance_comparison::{
    compare_source_cubemap_irradiance, SourceCubemapIrradianceComparisonError,
    SourceCubemapIrradianceErrorStatistics,
};
pub use lightmap::{
    LightProbeGridData, LightmapAtlasBudget, LightmapAtlasDescriptor, LightmapAtlasFormat,
    LightmapAtlasPage, LightmapBakeOutput, LightmapBakeRequest, LightmapBakeSceneSnapshot,
    LightmapConsumeContract, LightmapContractValidationError, LightmapInstanceSlot,
    LIGHTMAP_CONSUME_CONTRACT_VERSION, LIGHTMAP_SCENE_SNAPSHOT_VERSION,
};
pub use reflection_probe::{
    reflection_probe_box_project_direction, reflection_probe_influence_weight,
    select_reflection_probe_blend, ProbeBakeTiming, ProbeInfluenceShape, ReflectionProbeBlend,
    ReflectionProbeBlendEntry, ReflectionProbeData, ReflectionProbeValidationError,
};
pub use rgba16f::{
    append_rgb_as_rgba16f_texels, append_rgba16f_texels, decode_rgb_from_rgba16f_texels,
    decode_rgba16f_texels, encode_rgba16f_texels, RGBA16F_TEXEL_SIZE_BYTES,
};
pub use skybox::{
    IblBakeKey, ProceduralSkyParams, SkyboxMode, SkyboxSettings, SourceCubemapEnvironment,
    SourceCubemapUploadKey, PROCEDURAL_SKY_DEFAULT_SOURCE_REVISION,
};
use source_cubemap::SourceCubemapPmremLayout;
pub use source_cubemap::{
    build_source_cubemap_from_captured_faces,
    build_source_cubemap_from_captured_faces_with_quality, build_source_cubemap_from_equirect,
    build_source_cubemap_from_source_mips, build_source_cubemap_from_source_mips_with_quality,
    rebuild_source_cubemap_from_source_mips_with_pmrem_layout_and_parallel_executor_and_timing,
    rebuild_source_cubemap_from_source_mips_with_pmrem_layout_and_timing,
    source_cubemap_capture_hash, source_cubemap_evaluate_irradiance_sh9,
    source_cubemap_face_mip_offset, source_cubemap_face_size_from_equirect_height,
    source_cubemap_irradiance_mip_level, source_cubemap_mip_count, source_cubemap_mip_size,
    source_cubemap_pmrem_mip_from_roughness, source_cubemap_roughness_from_pmrem_mip,
    source_cubemap_sample_count, SourceCubemapBuildTiming, SourceCubemapIrradianceSh9,
    SourceCubemapMipChain, SourceCubemapPrefilterQuality, SOURCE_CUBEMAP_FACE_COUNT,
    SOURCE_CUBEMAP_IRRADIANCE_COEFFICIENT_COUNT, SOURCE_CUBEMAP_IRRADIANCE_SOURCE_FACE_SIZE,
    SOURCE_CUBEMAP_MAX_FACE_SIZE, SOURCE_CUBEMAP_MIN_FACE_SIZE, SOURCE_CUBEMAP_PMREM_FACE_SIZE,
    SOURCE_CUBEMAP_PMREM_MIP_COUNT, SOURCE_CUBEMAP_ROUGHEST_MIP,
    SOURCE_CUBEMAP_ROUGHNESS_MIP_SCALE,
};
pub use source_cubemap_artifact::{
    source_cubemap_environment_from_source_mips_with_bake_artifact,
    source_cubemap_environment_with_bake_artifact, source_cubemap_mip_chain_with_bake_artifact,
    SourceCubemapBakeArtifactError,
};
pub use source_cubemap_upload::{
    build_source_cubemap_upload_artifact, SourceCubemapUploadArtifact, SourceCubemapUploadMip,
};
pub use source_irradiance_cubemap::{
    build_source_cubemap_irradiance_cube,
    build_source_cubemap_irradiance_cube_with_parallel_executor,
    source_cubemap_sample_irradiance_cube, SourceCubemapIrradianceCube,
    SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
};
