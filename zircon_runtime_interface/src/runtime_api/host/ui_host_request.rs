use std::fmt;

use serde::{Deserialize, Serialize};

use crate::handles::ZrRuntimeViewportHandle;
use crate::ui::{
    dispatch::{
        UiDispatchHostRequestKind, UiPointerLockPolicy, UiPopupEffectKind, UiTooltipEffectKind,
        UiTransientDismissalReason, UiTransientDismissalTarget,
    },
    event_ui::{UiNodeId, UiTreeId},
    layout::UiPoint,
    text::UiRichLinkTarget,
};

/// One platform-facing operation carried by a handled Runtime UI reply.
///
/// Input-method and clipboard requests keep their dedicated transaction contracts and never enter
/// this generic channel. Dynamic identifiers remain serializable but are omitted from `Debug`.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ZrRuntimeUiHostRequestV1 {
    pub target_viewport: ZrRuntimeViewportHandle,
    pub target_surface: u32,
    pub input_sequence: u64,
    pub request_index: u32,
    pub tree_id: UiTreeId,
    pub effect_index: u32,
    pub kind: ZrRuntimeUiHostRequestKindV1,
}

impl ZrRuntimeUiHostRequestV1 {
    // EXEMPT(GEN-Q7): host request projection mirrors the fixed ABI payload fields.
    #[allow(clippy::too_many_arguments)]
    pub fn from_dispatch_request(
        target_viewport: ZrRuntimeViewportHandle,
        target_surface: u32,
        input_sequence: u64,
        request_index: u32,
        tree_id: UiTreeId,
        effect_index: u32,
        request: &UiDispatchHostRequestKind,
    ) -> Option<Self> {
        Some(Self {
            target_viewport,
            target_surface,
            input_sequence,
            request_index,
            tree_id,
            effect_index,
            kind: ZrRuntimeUiHostRequestKindV1::from_dispatch_request(request)?,
        })
    }
}

impl fmt::Debug for ZrRuntimeUiHostRequestV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ZrRuntimeUiHostRequestV1")
            .field("target_viewport", &self.target_viewport)
            .field("target_surface", &self.target_surface)
            .field("input_sequence", &self.input_sequence)
            .field("request_index", &self.request_index)
            .field("tree_id", &self.tree_id)
            .field("effect_index", &self.effect_index)
            .field("kind", &self.kind.as_str())
            .finish()
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum ZrRuntimeUiHostRequestKindV1 {
    PointerLock {
        target: UiNodeId,
        policy: UiPointerLockPolicy,
    },
    PointerUnlock {
        policy: UiPointerLockPolicy,
    },
    HighPrecisionPointer {
        target: UiNodeId,
        enabled: bool,
    },
    Popup {
        kind: UiPopupEffectKind,
        popup_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        anchor: Option<UiPoint>,
    },
    Tooltip {
        kind: UiTooltipEffectKind,
        tooltip_id: String,
    },
    DismissTransientUi {
        target: UiTransientDismissalTarget,
        reason: UiTransientDismissalReason,
    },
    ActivateLink {
        target: UiNodeId,
        #[serde(rename = "href")]
        link_target: UiRichLinkTarget,
    },
}

impl ZrRuntimeUiHostRequestKindV1 {
    /// 只投影通用宿主副作用；输入法和剪贴板返回 `None`，由各自的事务 ABI 继续处理。
    pub fn from_dispatch_request(request: &UiDispatchHostRequestKind) -> Option<Self> {
        match request {
            UiDispatchHostRequestKind::InputMethod(_) | UiDispatchHostRequestKind::Clipboard(_) => {
                None
            }
            UiDispatchHostRequestKind::PointerLock { target, policy } => Some(Self::PointerLock {
                target: *target,
                policy: *policy,
            }),
            UiDispatchHostRequestKind::PointerUnlock { policy } => {
                Some(Self::PointerUnlock { policy: *policy })
            }
            UiDispatchHostRequestKind::HighPrecisionPointer { target, enabled } => {
                Some(Self::HighPrecisionPointer {
                    target: *target,
                    enabled: *enabled,
                })
            }
            UiDispatchHostRequestKind::Popup {
                kind,
                popup_id,
                anchor,
            } => Some(Self::Popup {
                kind: *kind,
                popup_id: popup_id.clone(),
                anchor: *anchor,
            }),
            UiDispatchHostRequestKind::Tooltip { kind, tooltip_id } => Some(Self::Tooltip {
                kind: *kind,
                tooltip_id: tooltip_id.clone(),
            }),
            UiDispatchHostRequestKind::DismissTransientUi { target, reason } => {
                Some(Self::DismissTransientUi {
                    target: *target,
                    reason: *reason,
                })
            }
            UiDispatchHostRequestKind::ActivateLink {
                target,
                link_target,
            } => Some(Self::ActivateLink {
                target: *target,
                link_target: link_target.clone(),
            }),
        }
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::PointerLock { .. } => "pointer_lock",
            Self::PointerUnlock { .. } => "pointer_unlock",
            Self::HighPrecisionPointer { .. } => "high_precision_pointer",
            Self::Popup { .. } => "popup",
            Self::Tooltip { .. } => "tooltip",
            Self::DismissTransientUi { .. } => "dismiss_transient_ui",
            Self::ActivateLink { .. } => "activate_link",
        }
    }
}

#[cfg(test)]
#[path = "tests/ui_host_request.rs"]
mod tests;
