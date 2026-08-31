use super::super::RenderPipelinePhase;

pub struct PostProcessGraphResourceNames;

impl PostProcessGraphResourceNames {
    pub const SCENE_COLOR: &'static str = "scene-color";
    pub const TRANSMISSION_SCENE_COLOR: &'static str = "transmission.scene-color";
    pub const SCENE_DEPTH: &'static str = "scene-depth";
    pub const HALF_RES_TRANSPARENCY_COLOR: &'static str = "transparency.half-res.color";
    pub const HALF_RES_TRANSPARENCY_DEPTH: &'static str = "transparency.half-res.depth";
    pub const SHADOW_ATLAS: &'static str = "shadow-atlas";
    pub const SCENE_VELOCITY: &'static str = "scene-velocity";
    pub const MOTION_VECTOR_TILE_MAX: &'static str = "postprocess.motion-vector.tile-max";
    pub const MOTION_VECTOR_TILE_MAX_COARSE: &'static str =
        "postprocess.motion-vector.tile-max.coarse";
    pub const MOTION_VECTOR_NEIGHBOR_MAX: &'static str = "postprocess.motion-vector.neighbor-max";
    pub const DEPTH_OF_FIELDED: &'static str = "postprocess.depth-of-fielded";
    pub const MOTION_BLURRED: &'static str = "postprocess.motion-blurred";
    pub const GBUFFER_ALBEDO: &'static str = "gbuffer-albedo";
    pub const GBUFFER_NORMAL: &'static str = "gbuffer-normal";
    pub const GBUFFER_MATERIAL: &'static str = "gbuffer-material";
    pub const GBUFFER_EMISSIVE: &'static str = "gbuffer-emissive";
    pub const AMBIENT_OCCLUSION_RAW: &'static str = "ambient-occlusion.raw";
    pub const AMBIENT_OCCLUSION_SPATIAL: &'static str = "ambient-occlusion.spatial";
    pub const AMBIENT_OCCLUSION: &'static str = "ambient-occlusion";
    pub const HISTORY_PREVIOUS_AMBIENT_OCCLUSION: &'static str =
        "history.previous.ambient-occlusion";
    pub const SSAO_PARAMS: &'static str = "postprocess.ssao.params";
    pub const CONTACT_SHADOW_OCCLUSION: &'static str = "contact-shadow-occlusion";
    pub const GLOBAL_ILLUMINATION: &'static str = "global-illumination";
    pub const HYBRID_GI_SCENE: &'static str = "hybrid-gi-scene";
    pub const HYBRID_GI_TRACE: &'static str = "hybrid-gi-trace";
    pub const HYBRID_GI_LIGHTING: &'static str = "hybrid-gi-lighting";
    pub const HYBRID_GI_TEMPORAL_METADATA: &'static str = "hybrid-gi-temporal-metadata";
    pub const HISTORY_PREVIOUS_HYBRID_GI: &'static str = "history-global-illumination";
    pub const HISTORY_PREVIOUS_HYBRID_GI_TEMPORAL_METADATA: &'static str =
        "history-global-illumination-temporal-metadata";
    pub const SCENE_LIGHT_DATA: &'static str = "scene-light-data";
    pub const LIGHT_LIST: &'static str = "light-list";
    pub const LIGHT_GRID_PARAMS: &'static str = "light-grid-params";
    pub const LIGHT_ZBINS: &'static str = "light-zbins";
    pub const LIGHT_TILE_MASKS: &'static str = "light-tile-masks";
    pub const VOLUMETRIC_MEDIA: &'static str = "volumetric.media";
    pub const VOLUMETRIC_SCATTERING: &'static str = "volumetric.scattering";
    pub const VOLUMETRIC_INTEGRATED: &'static str = "volumetric.integrated";
    pub const OIT_LAYERS: &'static str = "oit.layers";
    pub const OIT_COUNTS: &'static str = "oit.counts";
    pub const SSS_DIFFUSE: &'static str = "sss.diffuse";
    pub const SSS_SPECULAR: &'static str = "sss.specular";
    pub const SSS_SCATTERED: &'static str = "sss.scattered";
    pub const SSS_TILE_LIST: &'static str = "sss.tile-list";
    pub const SSS_INDIRECT_ARGS: &'static str = "sss.indirect-args";
    pub const SSS_PARAMS: &'static str = "sss.params";
    pub const SSS_PROFILES: &'static str = "sss.profiles";
    pub const HISTORY_PREVIOUS_VOLUMETRIC_SCATTERING: &'static str =
        "history.previous.volumetric.scattering";
    pub const HZB_FURTHEST: &'static str = "hzb-furthest";
    // Temporal resources use distinct names so a pass cannot silently read and overwrite the same history slot.
    pub const HISTORY_PREVIOUS_SCREEN_SPACE_REFLECTION: &'static str =
        "history.previous.screen-space-reflection";
    pub const HISTORY_PREVIOUS_HZB_FURTHEST: &'static str = "history.previous.hzb-furthest";
    pub const TAA_HISTORY_PREVIOUS: &'static str = "taa.history.previous.scene-color";
    pub const TAA_HISTORY_CURRENT: &'static str = "taa.history.current.scene-color";
    pub const TAA_OUTPUT: &'static str = "taa.output.scene-color";
    pub const TAA_REACTIVE_MASK: &'static str = "taa.reactive-mask";
    pub const BLOOM: &'static str = "bloom-texture";
    pub const EXPOSURE_HISTOGRAM: &'static str = "postprocess.exposure.histogram";
    pub const EXPOSURE_PREVIOUS: &'static str = "history.previous.exposure";
    pub const EXPOSURE_CURRENT: &'static str = "history.current.exposure";
    pub const COLOR_LUT: &'static str = "postprocess.color-lut";
    pub const COLOR_GRADED: &'static str = "postprocess.color-graded";
    pub const SCENE_COMPOSITED: &'static str = "postprocess.scene-composited";
    pub const BLURRED: &'static str = "postprocess.blurred";
    pub const EFFECT_STACKED: &'static str = "postprocess.effect-stacked";
    pub const TONEMAPPED: &'static str = "postprocess.tonemapped";
    pub const DEPTH_OF_FIELD_COC: &'static str = "postprocess.depth-of-field.coc";
    pub const DEPTH_OF_FIELD_BOKEH: &'static str = "postprocess.depth-of-field.bokeh";
    pub const SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID: &'static str =
        "postprocess.screen-space-reflection.reflection-pyramid";
    pub const SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE: &'static str =
        "postprocess.screen-space-reflection.reflection-pyramid.coarse";
    pub const SCREEN_SPACE_REFLECTION_SPECULAR_OCCLUSION: &'static str =
        "postprocess.screen-space-reflection.specular-occlusion";
    pub const SCREEN_SPACE_REFLECTION_HISTORY: &'static str =
        "postprocess.screen-space-reflection.history";
    pub const PRIMARY_UPSCALED: &'static str = "postprocess.primary-upscaled";
    pub const SECONDARY_UPSCALED: &'static str = "postprocess.secondary-upscaled";
    pub const FINAL_COMPOSITED: &'static str = "postprocess.terminal-aa-input";
    pub const FINAL_COLOR: &'static str = "final-color";
    pub const VIEWPORT_OUTPUT: &'static str = "viewport-output";

    /// Returns the ViewFamily phase that owns a view-sized texture's geometry.
    ///
    /// Fixed-shape textures such as the colour LUT and volumetric froxel grids return `None`.
    /// Allocation and execution share this table so a resource cannot use one phase's backing
    /// extent with another phase's viewport.
    pub fn view_family_pipeline_phase(name: &str) -> Option<RenderPipelinePhase> {
        match name {
            Self::SCENE_COLOR
            | Self::TRANSMISSION_SCENE_COLOR
            | Self::SCENE_DEPTH
            | Self::HALF_RES_TRANSPARENCY_COLOR
            | Self::HALF_RES_TRANSPARENCY_DEPTH
            | Self::SHADOW_ATLAS
            | Self::SCENE_VELOCITY
            | Self::MOTION_VECTOR_TILE_MAX
            | Self::MOTION_VECTOR_TILE_MAX_COARSE
            | Self::MOTION_VECTOR_NEIGHBOR_MAX
            | Self::GBUFFER_ALBEDO
            | Self::GBUFFER_NORMAL
            | Self::GBUFFER_MATERIAL
            | Self::GBUFFER_EMISSIVE
            | Self::AMBIENT_OCCLUSION_RAW
            | Self::AMBIENT_OCCLUSION_SPATIAL
            | Self::AMBIENT_OCCLUSION
            | Self::HISTORY_PREVIOUS_AMBIENT_OCCLUSION
            | Self::CONTACT_SHADOW_OCCLUSION
            | Self::GLOBAL_ILLUMINATION
            | Self::HYBRID_GI_LIGHTING
            | Self::HYBRID_GI_TEMPORAL_METADATA
            | Self::HISTORY_PREVIOUS_HYBRID_GI
            | Self::HISTORY_PREVIOUS_HYBRID_GI_TEMPORAL_METADATA
            | Self::SSS_DIFFUSE
            | Self::SSS_SPECULAR
            | Self::SSS_SCATTERED
            | Self::HISTORY_PREVIOUS_VOLUMETRIC_SCATTERING
            | Self::HZB_FURTHEST
            | Self::HISTORY_PREVIOUS_HZB_FURTHEST
            | Self::TAA_REACTIVE_MASK => Some(RenderPipelinePhase::SceneLinear),
            Self::DEPTH_OF_FIELD_COC | Self::DEPTH_OF_FIELD_BOKEH | Self::DEPTH_OF_FIELDED => {
                Some(RenderPipelinePhase::PreReconstructionScenePostProcess)
            }
            Self::TAA_HISTORY_PREVIOUS | Self::TAA_HISTORY_CURRENT | Self::TAA_OUTPUT => {
                Some(RenderPipelinePhase::TemporalReconstruction)
            }
            Self::MOTION_BLURRED
            | Self::BLOOM
            | Self::SCENE_COMPOSITED
            | Self::BLURRED
            | Self::HISTORY_PREVIOUS_SCREEN_SPACE_REFLECTION
            | Self::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID
            | Self::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE
            | Self::SCREEN_SPACE_REFLECTION_SPECULAR_OCCLUSION
            | Self::SCREEN_SPACE_REFLECTION_HISTORY => {
                Some(RenderPipelinePhase::PostReconstructionScenePostProcess)
            }
            Self::COLOR_GRADED | Self::EFFECT_STACKED | Self::TONEMAPPED => {
                Some(RenderPipelinePhase::DisplayMapping)
            }
            Self::FINAL_COMPOSITED => Some(RenderPipelinePhase::DisplayPostProcess),
            Self::PRIMARY_UPSCALED => Some(RenderPipelinePhase::PrimarySpatialUpscale),
            Self::SECONDARY_UPSCALED => Some(RenderPipelinePhase::SecondarySpatialUpscale),
            Self::FINAL_COLOR => Some(RenderPipelinePhase::OutputTransform),
            Self::VIEWPORT_OUTPUT => Some(RenderPipelinePhase::Present),
            Self::COLOR_LUT
            | Self::VOLUMETRIC_MEDIA
            | Self::VOLUMETRIC_SCATTERING
            | Self::VOLUMETRIC_INTEGRATED => None,
            _ => None,
        }
    }
}
