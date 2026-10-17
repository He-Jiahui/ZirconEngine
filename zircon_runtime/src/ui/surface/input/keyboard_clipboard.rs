//! 将键盘剪贴板意图转为宿主请求，并处理带传输身份的异步完成事件。
//!
//! 待处理身份与编辑版本由 Surface 的 clipboard_transfers 保存；这里不直接访问系统剪贴板。
//! 成功的 Cut/Paste 仍委托 editable_text 的统一文本事务，约束、事件与安全文本遮蔽沿原输入链完成。

use zircon_runtime_interface::ui::{
    dispatch::{
        UiClipboardInputEvent, UiClipboardTransferIntent, UiClipboardTransferOutcome,
        UiClipboardTransferReceipt, UiClipboardTransferStatus, UiDispatchEffect,
        UiInputDispatchResult, UiInputEvent, UiKeyboardInputEvent,
    },
    event_ui::UiNodeId,
    surface::{UiEditableTextState, UiTextEditAction},
};

use crate::ui::{dispatch::UiTextDocumentSession, text::apply_text_edit_action_with_intent};

use super::super::surface::UiSurface;
use super::{
    editable_text::{
        apply_committed_text_payload, apply_editable_text_state, TextComponentEventKind,
    },
    effect::append_dispatch_effect_to_result,
    is_valid_input_owner,
    owner_route::owner_routed_result,
    route_policy::annotate_route_policy,
    route_steps::annotate_result_route_steps,
    text_keyboard::KeyboardClipboardAction,
    text_state::{
        editable_text_input_is_secure, editable_text_state_for_node, editable_value_property,
    },
};

// secure 文本禁用 Copy/Cut，只读文本禁用 Cut/Paste；Copy/Cut 还需非空选区。通过所有权与属性检查的异步请求携带 edit revision，供回执拒绝过期编辑。
/// 在已识别键盘快捷键后发起异步传输；此时不会提前删除 Cut 的选区。
///
/// # 参数与返回
/// `target` 与拥有的 `editable` 是当前路由目标及文本快照，`action` 决定读写方向。
/// 返回拥有的派发结果；只有通过检查并被效应层接纳的请求才留给宿主执行。
///
/// # 生命周期
/// 后续完成事件经 `dispatch_clipboard_input` 重新校验；不能把发起成功当作编辑已提交。
pub(super) fn dispatch_keyboard_clipboard(
    surface: &mut UiSurface,
    keyboard: UiKeyboardInputEvent,
    target: UiNodeId,
    editable: UiEditableTextState,
    action: KeyboardClipboardAction,
) -> UiInputDispatchResult {
    let phase = match action {
        KeyboardClipboardAction::Copy => "keyboard.clipboard_copy",
        KeyboardClipboardAction::Cut => "keyboard.clipboard_cut",
        KeyboardClipboardAction::Paste => "keyboard.clipboard_paste",
    };
    let event = UiInputEvent::Keyboard(keyboard);
    if !is_valid_input_owner(surface, target) {
        return owner_routed_result(surface, event, Some(target), phase);
    }
    if matches!(
        action,
        KeyboardClipboardAction::Copy | KeyboardClipboardAction::Cut
    ) && editable_text_input_is_secure(surface, target)
    {
        let mut result = owner_routed_result(surface, event, Some(target), phase);
        result
            .diagnostics
            .notes
            .push("clipboard copy and cut disabled for secure text input".to_string());
        return result;
    }

    let Some(property) = editable_value_property(surface, target) else {
        let mut result = owner_routed_result(surface, event, Some(target), phase);
        result
            .diagnostics
            .notes
            .push("editable value property missing".to_string());
        return result;
    };

    match action {
        KeyboardClipboardAction::Copy => {
            let mut result = owner_routed_result(surface, event, Some(target), phase);
            let Some(text) = selected_clipboard_text(&editable) else {
                result
                    .diagnostics
                    .notes
                    .push("clipboard selection empty".to_string());
                return result;
            };
            begin_clipboard_request(
                surface,
                &mut result,
                target,
                property,
                UiClipboardTransferIntent::Copy,
                Some(text),
            );
            result
        }
        KeyboardClipboardAction::Cut => {
            let Some(text) = selected_clipboard_text(&editable) else {
                let mut result = owner_routed_result(surface, event, Some(target), phase);
                result
                    .diagnostics
                    .notes
                    .push("clipboard selection empty".to_string());
                return result;
            };
            if editable.read_only {
                let mut result = owner_routed_result(surface, event, Some(target), phase);
                result
                    .diagnostics
                    .notes
                    .push("clipboard cut blocked by read-only text".to_string());
                return result;
            }

            let mut result = owner_routed_result(surface, event, Some(target), phase);
            begin_clipboard_request(
                surface,
                &mut result,
                target,
                property,
                UiClipboardTransferIntent::Cut,
                Some(text),
            );
            result
        }
        KeyboardClipboardAction::Paste => {
            let mut result = owner_routed_result(surface, event, Some(target), phase);
            if editable.read_only {
                result
                    .diagnostics
                    .notes
                    .push("clipboard paste blocked by read-only text".to_string());
                return result;
            }
            begin_clipboard_request(
                surface,
                &mut result,
                target,
                property,
                UiClipboardTransferIntent::Paste,
                None,
            );
            result
        }
    }
}

// 宿主回执必须先取出一次性 pending transfer，并匹配 owner、焦点、属性、编辑 revision 与 secure 标志；任一快照过期都拒绝应用，成功的 Cut/Paste 才进入统一文本事务。
/// 由 Clipboard 事件分支消费宿主完成结果，返回协议回执及可能产生的文本效应。
///
/// # 参数与生命周期
/// `clipboard` 拥有宿主回传值；可选文档会话只在本次同步与事务提交期间借用。
/// 传输记录先被取出再校验，故拒绝结果也不能再次使用同一个传输身份。
///
/// # 返回与参考
/// 状态为 Applied 表示接受该回执，不保证文本发生变化；Copy 本来就不改文本。
/// 参考 Surface 的 clipboard_transfers 与同级 editable_text 的提交入口。
pub(super) fn dispatch_clipboard_input(
    surface: &mut UiSurface,
    clipboard: UiClipboardInputEvent,
    mut text_documents: Option<&mut UiTextDocumentSession>,
) -> UiInputDispatchResult {
    let event = UiInputEvent::Clipboard(clipboard.clone());
    let Some(pending) = surface.take_clipboard_transfer(clipboard.transfer_id) else {
        let result = owner_routed_result(surface, event, Some(clipboard.owner), "clipboard.result");
        return finish_clipboard_result(
            surface,
            result,
            clipboard.transfer_id,
            None,
            UiClipboardTransferStatus::RejectedUnknown,
        );
    };
    let intent = pending.intent;
    if pending.owner != clipboard.owner {
        let result = owner_routed_result(surface, event, Some(clipboard.owner), "clipboard.result");
        return finish_clipboard_result(
            surface,
            result,
            clipboard.transfer_id,
            Some(intent),
            UiClipboardTransferStatus::RejectedOwner,
        );
    }
    if surface.clipboard_edit_revision(pending.owner) != Some(pending.expected_edit_revision)
        || surface.focus.focused != Some(pending.owner)
        || editable_value_property(surface, pending.owner).as_deref()
            != Some(pending.property.as_str())
        || editable_text_input_is_secure(surface, pending.owner) != pending.secure
    {
        let result = owner_routed_result(surface, event, Some(pending.owner), "clipboard.result");
        return finish_clipboard_result(
            surface,
            result,
            clipboard.transfer_id,
            Some(intent),
            UiClipboardTransferStatus::RejectedStale,
        );
    }
    let Some(editable) = editable_text_state_for_node(surface, pending.owner) else {
        let result = owner_routed_result(surface, event, Some(pending.owner), "clipboard.result");
        return finish_clipboard_result(
            surface,
            result,
            clipboard.transfer_id,
            Some(intent),
            UiClipboardTransferStatus::RejectedPolicy,
        );
    };
    if let Some(text_documents) = text_documents.as_deref_mut() {
        if let Some(source_epoch) = surface.input.text_document_epoch(pending.owner) {
            text_documents.synchronize_source(
                &surface.tree.tree_id,
                pending.owner,
                source_epoch,
                &editable.text,
            );
        }
    }
    if matches!(
        &clipboard.outcome,
        UiClipboardTransferOutcome::Failed { .. }
    ) {
        let mut result =
            owner_routed_result(surface, event, Some(pending.owner), "clipboard.result");
        result
            .diagnostics
            .notes
            .push("clipboard host transfer failed".to_string());
        return finish_clipboard_result(
            surface,
            result,
            clipboard.transfer_id,
            Some(intent),
            UiClipboardTransferStatus::Failed,
        );
    }

    let result = match (intent, clipboard.outcome) {
        (UiClipboardTransferIntent::Copy, UiClipboardTransferOutcome::WriteText) => {
            owner_routed_result(surface, event, Some(pending.owner), "clipboard.copy_result")
        }
        (UiClipboardTransferIntent::Cut, UiClipboardTransferOutcome::WriteText)
            if !editable.read_only && !pending.secure =>
        {
            let transition = apply_text_edit_action_with_intent(editable, UiTextEditAction::Delete);
            apply_editable_text_state(
                surface,
                text_documents.as_deref_mut(),
                event,
                pending.owner,
                transition.state,
                transition.committed,
                "clipboard.cut_commit",
                TextComponentEventKind::Change,
            )
        }
        (UiClipboardTransferIntent::Paste, UiClipboardTransferOutcome::ReadText { text })
            if !editable.read_only =>
        {
            apply_committed_text_payload(
                surface,
                text_documents.as_deref_mut(),
                event,
                pending.owner,
                editable,
                text,
                "clipboard.paste_commit",
            )
        }
        (UiClipboardTransferIntent::Cut | UiClipboardTransferIntent::Paste, _)
            if editable.read_only
                || (intent == UiClipboardTransferIntent::Cut && pending.secure) =>
        {
            let result =
                owner_routed_result(surface, event, Some(pending.owner), "clipboard.result");
            return finish_clipboard_result(
                surface,
                result,
                clipboard.transfer_id,
                Some(intent),
                UiClipboardTransferStatus::RejectedPolicy,
            );
        }
        _ => {
            let result =
                owner_routed_result(surface, event, Some(pending.owner), "clipboard.result");
            return finish_clipboard_result(
                surface,
                result,
                clipboard.transfer_id,
                Some(intent),
                UiClipboardTransferStatus::RejectedOutcome,
            );
        }
    };
    finish_clipboard_result(
        surface,
        result,
        clipboard.transfer_id,
        Some(intent),
        UiClipboardTransferStatus::Applied,
    )
}

/// 从规范化的选择范围复制文本；无选择、空范围、越界或非 UTF-8 边界均返回 None。
fn selected_clipboard_text(state: &UiEditableTextState) -> Option<String> {
    let range = state.selection.as_ref()?.range();
    if range.start == range.end || range.end > state.text.len() {
        return None;
    }
    state
        .text
        .get(range.start..range.end)
        .map(ToString::to_string)
}

/// 写出文本先受宿主桥接的字节上限约束；Paste 的读请求不携带文本。
/// 创建的传输若未形成宿主请求就立即取消，避免留下永远收不到完成事件的身份。
fn begin_clipboard_request(
    surface: &mut UiSurface,
    result: &mut UiInputDispatchResult,
    target: UiNodeId,
    property: String,
    intent: UiClipboardTransferIntent,
    text: Option<String>,
) {
    if text.as_ref().is_some_and(|text| {
        text.len() > zircon_runtime_interface::ZR_RUNTIME_CLIPBOARD_TEXT_MAX_ENCODED_BYTES_V1
    }) {
        result
            .diagnostics
            .notes
            .push("clipboard text exceeds the host bridge limit".to_string());
        return;
    }
    let Some(request) = surface.begin_clipboard_transfer(target, property, intent, text) else {
        result
            .diagnostics
            .notes
            .push("clipboard edit revision exhausted".to_string());
        return;
    };
    let transfer_id = request.transfer_id;
    let host_request_count = result.host_requests.len();
    append_dispatch_effect_to_result(
        surface,
        result,
        UiDispatchEffect::RequestClipboard {
            request: request.clone(),
        },
    );
    if result.host_requests.len() == host_request_count {
        surface.cancel_clipboard_transfer(transfer_id);
    }
}

/// 汇合各拒绝/完成分支的传输诊断，再按最终事件补齐策略与路由步骤。
fn finish_clipboard_result(
    surface: &UiSurface,
    mut result: UiInputDispatchResult,
    transfer_id: zircon_runtime_interface::ui::dispatch::UiClipboardTransferId,
    intent: Option<UiClipboardTransferIntent>,
    status: UiClipboardTransferStatus,
) -> UiInputDispatchResult {
    result.diagnostics.clipboard_transfer = Some(UiClipboardTransferReceipt {
        transfer_id,
        intent,
        status,
    });
    let event = result.event.clone();
    annotate_route_policy(surface, &event, &mut result);
    annotate_result_route_steps(&mut result);
    result
}
