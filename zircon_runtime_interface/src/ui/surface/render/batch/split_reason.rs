use serde::{Deserialize, Serialize};

use super::UiBatchKey;

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
