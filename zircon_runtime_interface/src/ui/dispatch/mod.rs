//! 公共 UI 派发契约入口，汇集输入、导航与指针派发类型；实际事件处理由 Runtime 执行。
#[path = "input/mod.rs"]
pub mod input;
#[path = "navigation/mod.rs"]
mod navigation;
#[path = "pointer/mod.rs"]
mod pointer;

pub use input::{
    UiAccessibilityInputEvent, UiAnalogInputEvent, UiClipboardInputEvent, UiClipboardRequest,
    UiClipboardRequestKind, UiClipboardTransferFailure, UiClipboardTransferId,
    UiClipboardTransferIntent, UiClipboardTransferOutcome, UiClipboardTransferReceipt,
    UiClipboardTransferStatus, UiComponentEmissionPolicy, UiComponentEventReport, UiDeviceId,
    UiDispatchAppliedEffect, UiDispatchDisposition, UiDispatchEffect, UiDispatchHostRequest,
    UiDispatchHostRequestKind, UiDispatchPhase, UiDispatchRejectedEffect, UiDispatchReply,
    UiDispatchReplyMergeReport, UiDispatchReplyStep, UiDispatchReplyStepTrace,
    UiDragDropEffectKind, UiDragDropInputEvent, UiDragDropInputEventKind, UiDragSessionId,
    UiFocusEffectReason, UiImeDeleteSurrounding, UiImeInputEvent, UiImeInputEventKind,
    UiImePreeditClause, UiImePreeditClauseError, UiImePreeditClauseKind, UiInputDiagnosticsMode,
    UiInputDiagnosticsTruncationReceipt, UiInputDispatchDiagnostics, UiInputDispatchResult,
    UiInputEvent, UiInputEventMetadata, UiInputMethodRequest, UiInputMethodRequestKind,
    UiInputMethodSurroundingText, UiInputMethodSurroundingTextError, UiInputModifiers,
    UiInputRoutePolicy, UiInputRouteTrace, UiInputSequence, UiInputTimestamp, UiKeyboardInputEvent,
    UiKeyboardInputState, UiMouseMotionInputEvent, UiNavigationInputEvent,
    UiNavigationRequestPolicy, UiNumberFormatIdentityV1, UiNumberInputCommitMethod,
    UiNumberInputCommitStatus, UiNumberInputParseStatus, UiNumberInputReceiptV1,
    UiPointerCaptureReason, UiPointerId, UiPointerInputEvent, UiPointerLockPolicy,
    UiPointerRoutingReceipt, UiPointerSource, UiPopupEffectKind, UiPopupInputEvent,
    UiPopupInputEventKind, UiPreciseScrollDelta, UiRedrawRequestReason, UiScrollDeltaUnit,
    UiSubmenuHoverTimerInputEvent, UiSurfaceId, UiTextByteRange, UiTextInputConstraintReceipt,
    UiTextInputEvent, UiToastTimerInputEvent, UiTooltipEffectKind, UiTooltipTimerInputEvent,
    UiTooltipTimerInputEventKind, UiTransientDismissalReason, UiTransientDismissalTarget,
    UiTypeaheadTimerInputEvent, UiUserId, UiWindowId, UI_INPUT_METHOD_SURROUNDING_TEXT_BYTE_LIMIT,
    UI_NUMBER_INPUT_RECEIPT_VERSION_V1,
};
pub use navigation::{
    UiNavigationDispatchContext, UiNavigationDispatchEffect, UiNavigationDispatchInvocation,
    UiNavigationDispatchResult,
};
pub use pointer::{
    UiPointerComponentEvent, UiPointerComponentEventReason, UiPointerDispatchContext,
    UiPointerDispatchDiagnostics, UiPointerDispatchEffect, UiPointerDispatchInvocation,
    UiPointerDispatchResult, UiPointerEvent, UiTemplateActionInvocation,
};
