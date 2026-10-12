//! 编译选项记录特性门控与图资源形状；它进入缓存键，调用方须先解析有效视图配置。
use std::collections::BTreeSet;

use crate::core::framework::render::{
    AoSourceSettingsKey, IblBakeArtifactRequest, PostProcessStackDescriptor, ShaderQualityTier,
};
use crate::graphics::feature::{BuiltinRenderFeature, RenderFeatureCapabilityRequirement};

use super::AdvancedLightingCompileInputs;

/// 单次视图编译的有效门控和资源形状；构造缓存键后不应再改变。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RenderPipelineCompileOptions {
    pub enabled_features: BTreeSet<BuiltinRenderFeature>,
    pub disabled_features: BTreeSet<BuiltinRenderFeature>,
    pub disabled_plugin_features: BTreeSet<String>,
    pub enabled_capabilities: BTreeSet<RenderFeatureCapabilityRequirement>,
    pub allow_async_compute: bool,
    pub enable_hzb_occlusion_culling: bool,
    pub enable_half_resolution_transparency: bool,
    pub half_resolution_transparency_depth_sigma: u16,
    pub graph_msaa_sample_count: Option<u32>,
    pub shader_quality: ShaderQualityTier,
    pub ambient_occlusion_source: Option<AoSourceSettingsKey>,
    pub post_process_stack: Option<PostProcessStackDescriptor>,
    pub environment_ibl_bake_request: Option<IblBakeArtifactRequest>,
    pub(crate) advanced_lighting_inputs: Option<AdvancedLightingCompileInputs>,
}
