use serde::{Deserialize, Serialize};

use super::UiBatchKey;

/// 相邻批次的边界原因，供批处理计划与可视化诊断共享。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiBatchSplitReason {
    #[default]
    FirstBatch,
    Merged,
    LayerChanged,
    ClipChanged,
    PrimitiveChanged,
    ShaderChanged,
    ResourceChanged,
    TextBackendChanged,
    DrawEffectsChanged,
    OpacityChanged,
}

impl UiBatchSplitReason {
    /// 按批次键的比较优先级报告首个差异；层级变化由调用方先处理。
    pub(super) fn between(current: &UiBatchKey, next: &UiBatchKey) -> Self {
        if current.clip != next.clip {
            Self::ClipChanged
        } else if current.primitive != next.primitive {
            Self::PrimitiveChanged
        } else if current.shader != next.shader {
            Self::ShaderChanged
        } else if current.resource != next.resource {
            Self::ResourceChanged
        } else if current.text_backend != next.text_backend {
            Self::TextBackendChanged
        } else if current.draw_effects != next.draw_effects {
            Self::DrawEffectsChanged
        } else if current.opacity_class != next.opacity_class {
            Self::OpacityChanged
        } else {
            Self::Merged
        }
    }
}
