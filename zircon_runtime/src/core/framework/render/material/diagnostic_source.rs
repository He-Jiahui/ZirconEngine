use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// 标识问题产生的层级，供资产校验、资源流送和渲染器 ABI 诊断合并后仍能定位责任方。
pub enum RenderMaterialDiagnosticSource {
    MaterialAsset,
    ShaderSchema,
    ShaderReadiness,
    RendererMaterialAbi,
    MaterialUniform,
    WgslCapture,
    MaterialOverride,
    TextureSlot,
    DependencyResolution,
}
