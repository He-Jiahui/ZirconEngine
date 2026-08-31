use serde::{Deserialize, Serialize};

use crate::ui::binding::UiBindingUpdateReport;
use crate::ui::component::{UiComponentEvent, UiDragMetrics};
use crate::ui::event_ui::UiNodeId;

use super::super::UiTemplateActionInvocation;
use super::{
    UiClipboardRequest, UiClipboardTransferReceipt, UiDispatchEffect, UiDispatchReply,
    UiDispatchReplyStepTrace, UiInputEvent, UiInputMethodRequest, UiPointerLockPolicy,
    UiPopupEffectKind, UiTooltipEffectKind, UiTransientDismissalReason, UiTransientDismissalTarget,
};
use crate::ui::layout::UiPoint;
use crate::ui::surface::{UiHitPath, UiPointerRoutingPath};
use crate::ui::text::UiRichLinkTarget;
use crate::ui::widget::UiWidgetEvent;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiInputDiagnosticsMode {
    #[default]
    Summary,
    Full,
}

impl UiInputDiagnosticsMode {
    pub const fn captures_full_trace(self) -> bool {
        matches!(self, Self::Full)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiInputRoutePolicy {
    #[default]
    Unrouted,
    PreviewTunnel,
    Bubble,
    Direct,
    FocusPath,
    PointerCapture,
    DefaultAction,
}

impl UiInputRoutePolicy {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unrouted => "unrouted",
            Self::PreviewTunnel => "preview_tunnel",
            Self::Bubble => "bubble",
            Self::Direct => "direct",
            Self::FocusPath => "focus_path",
            Self::PointerCapture => "pointer_capture",
            Self::DefaultAction => "default_action",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiInputRouteTrace {
    pub preview_tunnel: Vec<UiNodeId>,
    pub direct_target: Option<UiNodeId>,
    pub target: Option<UiNodeId>,
    pub bubble_path: Vec<UiNodeId>,
    pub focus_path: Vec<UiNodeId>,
    pub capture_target: Option<UiNodeId>,
    pub root_targets: Vec<UiNodeId>,
    pub popup_stack: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiInputDiagnosticsTruncationReceipt {
    pub route_nodes_dropped: u64,
    pub route_steps_dropped: u64,
    pub notes_dropped: u64,
    pub popup_entries_dropped: u64,
    pub string_bytes_dropped: u64,
}

impl UiInputDiagnosticsTruncationReceipt {
    pub const fn is_empty(&self) -> bool {
        self.route_nodes_dropped == 0
            && self.route_steps_dropped == 0
            && self.notes_dropped == 0
            && self.popup_entries_dropped == 0
            && self.string_bytes_dropped == 0
    }
}

/// Behavioral pointer-routing state retained independently from optional diagnostics.
///
/// The physical path remains the under-cursor authority. The dispatch path reuses it for ordinary
/// input and owns a second sequence only when capture or redirection changes the event route.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiPointerRoutingReceipt {
    pub route_target: Option<UiNodeId>,
    pub capture_target: Option<UiNodeId>,
    pub physical_hit_path: UiHitPath,
    pub dispatch_path: UiPointerRoutingPath,
}

impl UiPointerRoutingReceipt {
    pub fn physical_root_to_leaf(&self) -> &[UiNodeId] {
        &self.physical_hit_path.root_to_leaf
    }

    pub fn dispatch_root_to_leaf(&self) -> &[UiNodeId] {
        self.dispatch_path.root_to_leaf(&self.physical_hit_path)
    }

    pub fn physical_bubble_route(
        &self,
    ) -> impl DoubleEndedIterator<Item = UiNodeId> + ExactSizeIterator + Clone + '_ {
        self.physical_hit_path.bubble_route()
    }

    pub fn dispatch_bubble_route(
        &self,
    ) -> impl DoubleEndedIterator<Item = UiNodeId> + ExactSizeIterator + Clone + '_ {
        self.dispatch_root_to_leaf().iter().rev().copied()
    }
}

/// Low-cardinality evidence that an editable-text replacement was constrained before mutation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiTextInputConstraintReceipt {
    /// Canonical hard-line separators removed by a single-line input policy. CRLF counts once.
    pub removed_hard_line_count: u64,
    /// Unicode scalar values rejected by the configured character filter.
    pub removed_filter_scalar_count: u64,
    /// The accepted replacement was shortened to the remaining grapheme capacity.
    pub max_graphemes_truncated: bool,
    /// The platform preedit cursor range moved or was clamped after constraints were applied.
    pub preedit_cursor_range_adjusted: bool,
    /// Non-empty platform preedit clause ranges remapped to different UTF-8 byte offsets.
    pub preedit_clause_range_adjusted_count: u64,
    /// Platform preedit clauses removed because their complete range was constrained away.
    pub preedit_clause_dropped_count: u64,
}

impl UiTextInputConstraintReceipt {
    pub const fn is_empty(self) -> bool {
        self.removed_hard_line_count == 0
            && self.removed_filter_scalar_count == 0
            && !self.max_graphemes_truncated
            && !self.preedit_cursor_range_adjusted
            && self.preedit_clause_range_adjusted_count == 0
            && self.preedit_clause_dropped_count == 0
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiInputDispatchDiagnostics {
    pub routed: bool,
    pub handled_phase: Option<String>,
    pub route_policy: UiInputRoutePolicy,
    pub route_target: Option<UiNodeId>,
    pub route_trace: UiInputRouteTrace,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub route_steps: Vec<UiDispatchReplyStepTrace>,
    pub blocked_by: Option<UiNodeId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_constraint: Option<UiTextInputConstraintReceipt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number_input: Option<super::UiNumberInputReceiptV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clipboard_transfer: Option<UiClipboardTransferReceipt>,
    /// Secure input payloads and binding values were removed from this public result.
    #[serde(default, skip_serializing_if = "is_false")]
    pub secure_text_redacted: bool,
    pub notes: Vec<String>,
    #[serde(
        default,
        skip_serializing_if = "UiInputDiagnosticsTruncationReceipt::is_empty"
    )]
    pub truncation: UiInputDiagnosticsTruncationReceipt,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiDispatchAppliedEffect {
    pub effect_index: usize,
    pub effect: UiDispatchEffect,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiDispatchRejectedEffect {
    pub effect_index: usize,
    pub effect: UiDispatchEffect,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UiDispatchHostRequestKind {
    InputMethod(UiInputMethodRequest),
    Clipboard(UiClipboardRequest),
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiDispatchHostRequest {
    pub effect_index: usize,
    pub request: UiDispatchHostRequestKind,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiComponentEventReport {
    pub target: UiNodeId,
    pub event: UiComponentEvent,
    pub delivered: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drag: Option<UiDragMetrics>,
    /// Action resolved from the template binding that produced this event.
    ///
    /// Absent reports are ordinary component notifications and must not be
    /// interpreted by a host as an action invocation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template_action: Option<UiTemplateActionInvocation>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiInputDispatchResult {
    pub event: UiInputEvent,
    pub reply: UiDispatchReply,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pointer_routing: Option<UiPointerRoutingReceipt>,
    #[serde(default)]
    pub diagnostics: UiInputDispatchDiagnostics,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub applied_effects: Vec<UiDispatchAppliedEffect>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rejected_effects: Vec<UiDispatchRejectedEffect>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub host_requests: Vec<UiDispatchHostRequest>,
    /// Semantic widget events published after their authoritative mutation commits.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub widget_events: Vec<UiWidgetEvent>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub component_events: Vec<UiComponentEventReport>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub binding_reports: Vec<UiBindingUpdateReport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drag: Option<UiDragMetrics>,
}

impl UiInputDispatchResult {
    pub fn new(event: UiInputEvent, reply: UiDispatchReply) -> Self {
        Self {
            event,
            reply,
            pointer_routing: None,
            diagnostics: UiInputDispatchDiagnostics::default(),
            applied_effects: Vec::new(),
            rejected_effects: Vec::new(),
            host_requests: Vec::new(),
            widget_events: Vec::new(),
            component_events: Vec::new(),
            binding_reports: Vec::new(),
            drag: None,
        }
    }

    pub fn record_binding_report(&mut self, report: UiBindingUpdateReport) {
        if !report.updates.is_empty()
            || report.applied_count > 0
            || report.unchanged_count > 0
            || report.rejected_count > 0
        {
            self.binding_reports.push(report);
        }
    }
}

fn is_false(value: &bool) -> bool {
    !*value
}

#[cfg(test)]
mod tests {
    use super::{
        UiInputDiagnosticsTruncationReceipt, UiInputDispatchDiagnostics, UiInputDispatchResult,
        UiPointerRoutingReceipt, UiTextInputConstraintReceipt,
    };
    use crate::ui::dispatch::{
        UiClipboardTransferId, UiClipboardTransferIntent, UiClipboardTransferReceipt,
        UiClipboardTransferStatus, UiDispatchReply, UiInputEvent, UiInputEventMetadata,
        UiMouseMotionInputEvent, UiNumberInputCommitMethod, UiNumberInputCommitStatus,
        UiNumberInputParseStatus, UiNumberInputReceiptV1,
    };
    use crate::ui::event_ui::UiNodeId;
    use crate::ui::surface::{UiHitPath, UiPointerRoutingPath};

    #[test]
    fn pointer_routing_receipt_reuses_physical_path_until_dispatch_is_redirected() {
        let physical_hit_path = UiHitPath {
            target: Some(UiNodeId::new(3)),
            root_to_leaf: vec![UiNodeId::new(1), UiNodeId::new(3)],
            virtual_pointer: None,
        };
        let ordinary = UiPointerRoutingReceipt {
            route_target: Some(UiNodeId::new(3)),
            capture_target: None,
            physical_hit_path: physical_hit_path.clone(),
            dispatch_path: UiPointerRoutingPath::HitPath,
        };

        assert_eq!(
            ordinary.dispatch_root_to_leaf(),
            ordinary.physical_root_to_leaf()
        );
        assert_eq!(
            ordinary.dispatch_bubble_route().collect::<Vec<_>>(),
            vec![UiNodeId::new(3), UiNodeId::new(1)]
        );

        let captured = UiPointerRoutingReceipt {
            route_target: Some(UiNodeId::new(2)),
            capture_target: Some(UiNodeId::new(2)),
            physical_hit_path,
            dispatch_path: UiPointerRoutingPath::from_root_to_leaf(vec![
                UiNodeId::new(1),
                UiNodeId::new(2),
            ]),
        };
        assert_eq!(
            captured.physical_root_to_leaf(),
            &[UiNodeId::new(1), UiNodeId::new(3)]
        );
        assert_eq!(
            captured.dispatch_root_to_leaf(),
            &[UiNodeId::new(1), UiNodeId::new(2)]
        );

        let roundtrip: UiPointerRoutingReceipt =
            serde_json::from_value(serde_json::to_value(&captured).unwrap()).unwrap();
        assert_eq!(roundtrip, captured);
    }

    #[test]
    fn pointer_routing_receipt_physical_bubble_route_is_bidirectional_and_repeatable() {
        let receipt = UiPointerRoutingReceipt {
            physical_hit_path: UiHitPath {
                target: Some(UiNodeId::new(3)),
                root_to_leaf: vec![UiNodeId::new(1), UiNodeId::new(2), UiNodeId::new(3)],
                virtual_pointer: None,
            },
            ..UiPointerRoutingReceipt::default()
        };
        let bubble_route = receipt.physical_bubble_route();

        assert_eq!(
            bubble_route.collect::<Vec<_>>(),
            vec![UiNodeId::new(3), UiNodeId::new(2), UiNodeId::new(1)]
        );
        assert_eq!(
            receipt.physical_bubble_route().rev().collect::<Vec<_>>(),
            vec![UiNodeId::new(1), UiNodeId::new(2), UiNodeId::new(3)]
        );
        assert_eq!(
            receipt.physical_bubble_route().collect::<Vec<_>>(),
            vec![UiNodeId::new(3), UiNodeId::new(2), UiNodeId::new(1)]
        );
    }

    #[test]
    fn legacy_dispatch_result_without_pointer_receipt_defaults_to_none() {
        let result = UiInputDispatchResult::new(
            UiInputEvent::MouseMotion(UiMouseMotionInputEvent {
                metadata: UiInputEventMetadata::default(),
                delta_x: 0.0,
                delta_y: 0.0,
            }),
            UiDispatchReply::unhandled(),
        );
        let mut json = serde_json::to_value(result).unwrap();
        json.as_object_mut().unwrap().remove("pointer_routing");

        let legacy: UiInputDispatchResult = serde_json::from_value(json).unwrap();
        assert!(legacy.pointer_routing.is_none());
    }

    #[test]
    fn text_constraint_receipt_roundtrips_and_defaults_when_missing() {
        let expected = UiTextInputConstraintReceipt {
            removed_hard_line_count: 2,
            removed_filter_scalar_count: 3,
            max_graphemes_truncated: true,
            preedit_cursor_range_adjusted: true,
            preedit_clause_range_adjusted_count: 4,
            preedit_clause_dropped_count: 5,
        };
        let diagnostics = UiInputDispatchDiagnostics {
            text_constraint: Some(expected),
            number_input: Some(UiNumberInputReceiptV1 {
                parse_status: UiNumberInputParseStatus::Valid,
                commit_method: UiNumberInputCommitMethod::Enter,
                commit_status: UiNumberInputCommitStatus::Applied,
                ..UiNumberInputReceiptV1::default()
            }),
            clipboard_transfer: Some(UiClipboardTransferReceipt {
                transfer_id: UiClipboardTransferId::issue(),
                intent: Some(UiClipboardTransferIntent::Paste),
                status: UiClipboardTransferStatus::Applied,
            }),
            secure_text_redacted: true,
            ..UiInputDispatchDiagnostics::default()
        };

        let mut json = serde_json::to_value(&diagnostics).unwrap();
        let roundtrip: UiInputDispatchDiagnostics = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(roundtrip.text_constraint, Some(expected));
        assert_eq!(roundtrip.number_input, diagnostics.number_input);
        assert_eq!(roundtrip.clipboard_transfer, diagnostics.clipboard_transfer);
        assert!(roundtrip.secure_text_redacted);

        json.as_object_mut().unwrap().remove("text_constraint");
        json.as_object_mut().unwrap().remove("number_input");
        json.as_object_mut().unwrap().remove("clipboard_transfer");
        json.as_object_mut().unwrap().remove("secure_text_redacted");
        json.as_object_mut().unwrap().remove("truncation");
        let legacy: UiInputDispatchDiagnostics = serde_json::from_value(json).unwrap();
        assert!(legacy.text_constraint.is_none());
        assert!(legacy.number_input.is_none());
        assert!(legacy.clipboard_transfer.is_none());
        assert!(!legacy.secure_text_redacted);
        assert!(legacy.truncation.is_empty());

        let truncation = UiInputDiagnosticsTruncationReceipt {
            route_nodes_dropped: 1,
            route_steps_dropped: 2,
            notes_dropped: 3,
            popup_entries_dropped: 4,
            string_bytes_dropped: 5,
        };
        let roundtrip: UiInputDiagnosticsTruncationReceipt =
            serde_json::from_value(serde_json::to_value(truncation).unwrap()).unwrap();
        assert_eq!(roundtrip, truncation);
        assert!(!roundtrip.is_empty());
    }
}
