use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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
