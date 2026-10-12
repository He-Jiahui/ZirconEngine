//! 后处理框架契约：场景提取生成相机生效设置，帧提交据此组装并校验通道图，执行器再消费图资源。
//! 此模块只描述设置、依赖与诊断；历史纹理和 GPU 通道由图形运行层拥有。

#[path = "ambient_occlusion_settings.rs"]
mod ambient_occlusion_settings;
#[path = "chain.rs"]
mod chain;
#[path = "color_lut_readback.rs"]
mod color_lut_readback;
#[path = "color_space.rs"]
mod color_space;
#[path = "effect.rs"]
mod effect;
#[path = "effects.rs"]
mod effect_stack_settings;
#[path = "exposure_readback.rs"]
mod exposure_readback;
#[path = "exposure_settings.rs"]
mod exposure_settings;
#[path = "graph_resource_names.rs"]
mod graph_resource_names;
#[path = "pass_graph.rs"]
mod pass_graph;
#[path = "pass_node.rs"]
mod pass_node;
#[path = "resolved_stack.rs"]
mod resolved_stack;
#[path = "stack.rs"]
mod stack;
#[path = "validation.rs"]
mod validation;
#[path = "volume_component.rs"]
mod volume_component;
#[path = "volume_evaluator.rs"]
mod volume_evaluator;
#[path = "volume_extract.rs"]
mod volume_extract;
#[path = "volume_profile.rs"]
mod volume_profile;
#[path = "volume_registry.rs"]
mod volume_registry;

pub use ambient_occlusion_settings::{
    AoQualityTier, AoSourceSettings, AoSourceSettingsKey, AO_SOURCE_SETTINGS_VERSION,
};
pub use chain::PostProcessChainSlot;
pub use color_lut_readback::{
    RenderColorLutReadbackReference, RenderColorLutReadbackReport, COLOR_LUT_IDENTITY_EPSILON_MICRO,
};
pub use color_space::{
    RenderOutputTransfer, RenderPostProcessTextureFormat, COLOR_LUT_FORMAT, COLOR_LUT_SIZE_DEFAULT,
    COLOR_LUT_SIZE_HIGH_QUALITY, INTERMEDIATE_HDR_FORMAT_DEFAULT,
    INTERMEDIATE_HDR_FORMAT_HIGH_QUALITY, OUTPUT_TRANSFER_DEFAULT, TONEMAPPED_SDR_FORMAT,
};
pub use effect::{PostProcessEffectKind, PostProcessEffectSettings};
pub use effect_stack_settings::{
    RenderBlurSettings, RenderChromaticAberrationSettings, RenderColorLookupSettings,
    RenderColorLookupTextureLayout, RenderDepthOfFieldSettings, RenderDitherSettings,
    RenderFilmGrainSettings, RenderFogSettings, RenderMotionBlurSettings,
    RenderPostProcessEffectStackReport, RenderPostProcessEffectStackResourceStatus,
    RenderPostProcessEffectStackSettings, RenderScreenSpaceReflectionSettings,
    RenderTonemapOperator, RenderTonemapSettings, RenderVignetteSettings,
    MAX_COLOR_LOOKUP_TEXTURE_SIZE, MIN_COLOR_LOOKUP_TEXTURE_SIZE,
};
pub use exposure_readback::{RenderExposureReadbackReport, EXPOSURE_READBACK_EXPECTED_BYTE_LEN};
pub use exposure_settings::{
    RenderExposureMode, RenderExposureSettings, EXPOSURE_BUFFER_WORD_COUNT,
    EXPOSURE_HISTOGRAM_BIN_COUNT,
};
pub use graph_resource_names::PostProcessGraphResourceNames;
pub use pass_graph::PostProcessPassGraph;
pub use pass_node::PostProcessPassNode;
pub use resolved_stack::RenderResolvedPostProcessSettings;
pub use stack::PostProcessStackDescriptor;
pub use validation::PostProcessGraphValidationError;
pub use volume_component::{
    interp_bool, interp_discrete, interp_float_lerp, interp_vec3_lerp, VolumeComponentApplyError,
    VolumeComponentApplyFn, VolumeComponentDescriptor, VolumeComponentReadFn, VolumeParamInterpFn,
    VolumeParamSchema, VolumeParamType, VolumeParamValue, BUILTIN_POST_PROCESS_VOLUME_COMPONENTS,
};
pub use volume_evaluator::{
    ResolvedPostProcessStack, VolumeEvaluationError, VolumeEvaluationRequest, VolumeEvaluator,
};
pub use volume_extract::{PostProcessVolumeExtract, VolumeComponentOverride, VolumeShapeExtract};
pub use volume_profile::RenderPostProcessVolumeProfile;
pub use volume_registry::{VolumeComponentRegistry, VolumeRegistryError};
