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
mod tests {
    use super::*;
    use crate::ui::{
        dispatch::{
            UiClipboardRequest, UiClipboardRequestKind, UiClipboardTransferId,
            UiClipboardTransferIntent, UiDispatchHostRequestKind, UiPointerLockPolicy,
        },
        event_ui::{UiNodeId, UiTreeId},
    };
    use crate::{
        ZrRuntimeHostRequestBatchV1, ZrRuntimeHostRequestV1, ZIRCON_RUNTIME_ABI_VERSION_V1,
    };

    #[test]
    fn generic_ui_host_request_maps_platform_reply_identity() {
        let request = ZrRuntimeUiHostRequestV1::from_dispatch_request(
            ZrRuntimeViewportHandle::new(3),
            5,
            11,
            2,
            UiTreeId::new("runtime.ui.host"),
            7,
            &UiDispatchHostRequestKind::PointerLock {
                target: UiNodeId::new(13),
                policy: UiPointerLockPolicy::RawDelta,
            },
        )
        .expect("pointer lock is a generic host request");

        assert_eq!(request.target_viewport, ZrRuntimeViewportHandle::new(3));
        assert_eq!(request.target_surface, 5);
        assert_eq!(request.input_sequence, 11);
        assert_eq!(request.request_index, 2);
        assert_eq!(request.effect_index, 7);
        assert!(matches!(
            request.kind,
            ZrRuntimeUiHostRequestKindV1::PointerLock {
                target,
                policy: UiPointerLockPolicy::RawDelta,
            } if target == UiNodeId::new(13)
        ));
    }

    #[test]
    fn text_service_requests_remain_with_their_dedicated_queues() {
        let clipboard = UiDispatchHostRequestKind::Clipboard(UiClipboardRequest {
            transfer_id: UiClipboardTransferId::issue(),
            intent: UiClipboardTransferIntent::Paste,
            expected_edit_revision: 1,
            kind: UiClipboardRequestKind::ReadText,
            owner: UiNodeId::new(7),
            text: None,
        });

        assert!(ZrRuntimeUiHostRequestKindV1::from_dispatch_request(&clipboard).is_none());
    }

    #[test]
    fn generic_ui_host_request_round_trips_without_debugging_dynamic_content() {
        let dynamic_link_target =
            UiRichLinkTarget::parse("res://private/token-never-log.zui").unwrap();
        let request = ZrRuntimeUiHostRequestV1::from_dispatch_request(
            ZrRuntimeViewportHandle::new(3),
            5,
            11,
            2,
            UiTreeId::new("runtime.ui.host"),
            7,
            &UiDispatchHostRequestKind::ActivateLink {
                target: UiNodeId::new(13),
                link_target: dynamic_link_target.clone(),
            },
        )
        .expect("approved link activation is a generic host request");
        let batch = ZrRuntimeHostRequestBatchV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            vec![ZrRuntimeHostRequestV1::ui_host(request.clone())],
        );

        let encoded = serde_json::to_vec(&batch).expect("serialize generic UI host request");
        let encoded_text = std::str::from_utf8(&encoded).expect("host request JSON is UTF-8");
        let decoded: ZrRuntimeHostRequestBatchV1 =
            serde_json::from_slice(&encoded).expect("deserialize generic UI host request");

        assert_eq!(decoded, batch);
        assert!(encoded_text.contains("\"href\""));
        assert!(!encoded_text.contains("\"link_target\""));
        assert!(matches!(
            request.kind,
            ZrRuntimeUiHostRequestKindV1::ActivateLink {
                link_target,
                ..
            } if link_target == dynamic_link_target
        ));
        assert!(!format!("{request:?}").contains(&dynamic_link_target.to_string()));
    }
}
