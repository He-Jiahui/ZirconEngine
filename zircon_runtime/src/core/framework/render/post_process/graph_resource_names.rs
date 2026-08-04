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
    pub const AMBIENT_OCCLUSION: &'static str = "ambient-occlusion";
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
    pub const UPSCALED: &'static str = "postprocess.upscaled";
    pub const FINAL_COMPOSITED: &'static str = "postprocess.terminal-aa-input";
    pub const FINAL_COLOR: &'static str = "final-color";
    pub const VIEWPORT_OUTPUT: &'static str = "viewport-output";
}
