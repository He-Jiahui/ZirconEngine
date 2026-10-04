use serde::{Deserialize, Serialize};

use crate::core::resource::{AssetReference, ResourceId};

use super::{RenderMaterialDiagnosticSource, RenderMaterialTextureDimension};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "error", rename_all = "snake_case")]
/// 可序列化的材质契约失败原因；资产和 streamer 汇入 readiness 报告，供回退与管理诊断而非立刻丢失上下文。
pub enum RenderMaterialValidationError {
    InvalidMaskCutoff {
        cutoff: f32,
    },
    UnresolvedMaterialReference {
        material: ResourceId,
    },
    MissingRuntimeShaderSource,
    UnresolvedShaderReference {
        reference: AssetReference,
    },
    UnresolvedTextureReference {
        slot: String,
        reference: AssetReference,
    },
    TextureNotUploadReady {
        slot: String,
        reference: AssetReference,
        reason: String,
    },
    TextureDimensionMismatch {
        slot: String,
        reference: AssetReference,
        expected: RenderMaterialTextureDimension,
        actual: RenderMaterialTextureDimension,
    },
    UnsupportedTextureUvChannel {
        slot: String,
        channel: u32,
        supported_channel_count: u32,
    },
    InvalidLightingModel {
        path: String,
        value: String,
    },
    RenderQueueAlphaModeConflict {
        source: RenderMaterialDiagnosticSource,
        path: String,
        alpha_mode: String,
        render_queue: u16,
        expected: String,
    },
    UnregisteredShadingModel {
        path: String,
        token: String,
    },
    UnknownPropertyOverride {
        source: RenderMaterialDiagnosticSource,
        path: String,
        name: String,
    },
    PropertyOverrideTypeMismatch {
        source: RenderMaterialDiagnosticSource,
        path: String,
        name: String,
        expected: String,
    },
    MissingRequiredProperty {
        source: RenderMaterialDiagnosticSource,
        path: String,
        name: String,
    },
    MissingRequiredTextureSlot {
        source: RenderMaterialDiagnosticSource,
        path: String,
        slot: String,
    },
    UnknownTextureSlot {
        source: RenderMaterialDiagnosticSource,
        path: String,
        slot: String,
    },
    UnknownMaterialOption {
        source: RenderMaterialDiagnosticSource,
        path: String,
        name: String,
    },
    MaterialOptionTypeMismatch {
        source: RenderMaterialDiagnosticSource,
        path: String,
        name: String,
        expected: String,
    },
    InvalidMaterialQueueOffset {
        source: RenderMaterialDiagnosticSource,
        path: String,
        offset: i16,
        expected: String,
    },
    InvalidMaterialParent {
        source: RenderMaterialDiagnosticSource,
        path: String,
        diagnostic: String,
    },
    MissingWgslCapture {
        source: RenderMaterialDiagnosticSource,
        path: String,
        name: String,
    },
    ShaderReadinessDiagnostic {
        source: RenderMaterialDiagnosticSource,
        path: String,
        diagnostic: String,
    },
}
