use zircon_runtime_interface::ui::{
    binding::UiBindingSourceKind,
    surface::{UiEditableTextState, UiTextCaretAffinity, UiTextEditAction},
    text::{
        UiTextDocumentKey, UiTextEditSource, UiTextModelUpdateFailure, UiTextModelUpdateOrigin,
        UiTextModelUpdateReceipt, UiTextModelUpdateStatus, UI_TEXT_MODEL_UPDATE_SCHEMA_VERSION,
    },
};

use crate::{
    text::document::TextDocumentStoreEditCommit,
    ui::{
        dispatch::{UiTextDocumentSession, UiTextHistoryCommit},
        surface::{
            input::{
                editable_text_state_for_node, editable_value_property,
                prepare_editable_text_properties_with_edit,
                PreparedUiEditableTextDocumentTransaction,
            },
            UiSurface,
        },
        text::{
            apply_text_edit_action_with_intent, clamp_grapheme_boundary, CommittedTextEditIntent,
        },
    },
};

use super::{current_document_key, UiTextModelUpdateEnvelope};

// 即刻请求和失焦后延迟请求共用此提交点；必须重新核对文档版本，不能把排队时的快照当作当前内容。
// 属性与文档先准备再共同提交；程序化替换成功后清除编辑历史，避免撤销恢复被外部模型取代的文本。
pub(super) fn apply_now(
    text_documents: &mut UiTextDocumentSession,
    surface: &mut UiSurface,
    envelope: UiTextModelUpdateEnvelope,
    value: String,
) -> UiTextModelUpdateReceipt {
    let current_document = match current_document_key(text_documents, surface, envelope.node_id) {
        Ok(current_document) => current_document,
        Err(failure) => {
            return receipt(
                &envelope,
                UiTextModelUpdateStatus::Rejected,
                None,
                None,
                Some(failure),
            );
        }
    };
    if current_document != envelope.expected_document {
        return receipt(
            &envelope,
            UiTextModelUpdateStatus::Conflict,
            Some(current_document),
            None,
            Some(UiTextModelUpdateFailure::StaleDocument),
        );
    }
    let Some(surface_state) = editable_text_state_for_node(surface, envelope.node_id) else {
        return receipt(
            &envelope,
            UiTextModelUpdateStatus::Rejected,
            Some(current_document),
            None,
            Some(UiTextModelUpdateFailure::InvalidTarget),
        );
    };
    let Some(value_property) = editable_value_property(surface, envelope.node_id) else {
        return receipt(
            &envelope,
            UiTextModelUpdateStatus::Rejected,
            Some(current_document),
            None,
            Some(UiTextModelUpdateFailure::InvalidTarget),
        );
    };
    let current_state = committed_document_state(surface_state.clone());
    let next_state = projected_state(
        current_state.clone(),
        value,
        envelope.origin,
        surface.focus.focused == Some(envelope.node_id),
    );
    if next_state == surface_state {
        return receipt(
            &envelope,
            UiTextModelUpdateStatus::Unchanged,
            Some(current_document),
            None,
            None,
        );
    }
    if next_state.text == current_state.text {
        return match prepare_editable_text_properties_with_edit(
            surface,
            envelope.node_id,
            value_property.as_str(),
            &next_state,
            UiBindingSourceKind::RuntimeState,
            None,
        )
        .and_then(|prepared| prepared.preserving_committed_source().commit())
        {
            Ok(_) => receipt(
                &envelope,
                UiTextModelUpdateStatus::Applied,
                Some(current_document),
                None,
                None,
            ),
            Err(_) => receipt(
                &envelope,
                UiTextModelUpdateStatus::Rejected,
                Some(current_document),
                None,
                Some(UiTextModelUpdateFailure::PropertyRejected),
            ),
        };
    }

    let tree_id = surface.tree.tree_id.clone();
    let source_epoch = match surface.input.text_document_epoch(envelope.node_id) {
        Some(source_epoch) => source_epoch,
        None => {
            return receipt(
                &envelope,
                UiTextModelUpdateStatus::Rejected,
                Some(current_document),
                None,
                Some(UiTextModelUpdateFailure::DocumentUnavailable),
            );
        }
    };
    let intent = CommittedTextEditIntent::for_replacement(
        0..current_state.text.len(),
        next_state.text.len(),
    );
    let document = match text_documents.prepare_edit(
        &tree_id,
        envelope.node_id,
        source_epoch,
        &intent,
        &next_state,
        UiTextEditSource::Programmatic,
    ) {
        Ok(document) => document,
        Err(_) => {
            return receipt(
                &envelope,
                UiTextModelUpdateStatus::Rejected,
                Some(current_document),
                None,
                Some(UiTextModelUpdateFailure::DocumentRejected),
            );
        }
    };
    let properties = match prepare_editable_text_properties_with_edit(
        surface,
        envelope.node_id,
        value_property.as_str(),
        &next_state,
        UiBindingSourceKind::RuntimeState,
        Some(intent),
    ) {
        Ok(properties) => properties,
        Err(_) => {
            return receipt(
                &envelope,
                UiTextModelUpdateStatus::Rejected,
                Some(current_document),
                None,
                Some(UiTextModelUpdateFailure::PropertyRejected),
            );
        }
    };
    let committed =
        match PreparedUiEditableTextDocumentTransaction::new(properties, document).commit() {
            Ok(committed) => committed,
            Err(_) => {
                return receipt(
                    &envelope,
                    UiTextModelUpdateStatus::Rejected,
                    Some(current_document),
                    None,
                    Some(UiTextModelUpdateFailure::PropertyRejected),
                );
            }
        };
    let next_source_epoch = source_epoch + 1;
    debug_assert_eq!(
        surface.input.text_document_epoch(envelope.node_id),
        Some(next_source_epoch)
    );
    let (current_document, document_edit) = match &committed.document {
        TextDocumentStoreEditCommit::Changed { public_receipt, .. } => (
            UiTextDocumentKey {
                document_id: public_receipt.document_id,
                revision: public_receipt.revision,
            },
            Some(public_receipt.clone()),
        ),
        TextDocumentStoreEditCommit::Unchanged {
            document_id,
            revision,
        } => (
            UiTextDocumentKey {
                document_id: *document_id,
                revision: *revision,
            },
            None,
        ),
    };
    text_documents.finish_edit(
        &tree_id,
        envelope.node_id,
        next_source_epoch,
        &committed.document,
        UiTextHistoryCommit::Barrier,
    );
    receipt(
        &envelope,
        UiTextModelUpdateStatus::Applied,
        Some(current_document),
        document_edit,
        None,
    )
}

fn projected_state(
    mut state: UiEditableTextState,
    value: String,
    origin: UiTextModelUpdateOrigin,
    focused: bool,
) -> UiEditableTextState {
    if state.text != value {
        let previous_caret = state.caret.offset;
        state.text = value;
        state.caret.offset = clamp_grapheme_boundary(state.text.as_str(), previous_caret);
        if state.caret.offset != previous_caret {
            state.caret.affinity = UiTextCaretAffinity::Downstream;
        }
        state.selection = None;
        state.composition = None;
    }
    if focused
        && matches!(
            origin,
            UiTextModelUpdateOrigin::ExplicitSetText | UiTextModelUpdateOrigin::ExplicitLoadText
        )
    {
        state.caret.offset = state.text.len();
        state.caret.affinity = UiTextCaretAffinity::Downstream;
        state.selection = None;
        state.composition = None;
    }
    state
}

// 同步文档时只采用已提交源；IME preedit 是暂态显示，取消投影后才能与 expected_document 比较。
pub(super) fn committed_document_state(state: UiEditableTextState) -> UiEditableTextState {
    if state.composition.is_none() {
        return state;
    }
    apply_text_edit_action_with_intent(state, UiTextEditAction::CancelComposition).state
}

pub(super) fn receipt(
    envelope: &UiTextModelUpdateEnvelope,
    status: UiTextModelUpdateStatus,
    current_document: Option<UiTextDocumentKey>,
    document_edit: Option<zircon_runtime_interface::ui::text::UiTextEditReceipt>,
    failure: Option<UiTextModelUpdateFailure>,
) -> UiTextModelUpdateReceipt {
    let receipt = UiTextModelUpdateReceipt {
        schema_version: UI_TEXT_MODEL_UPDATE_SCHEMA_VERSION,
        request_id: envelope.request_id,
        tree_id: envelope.tree_id.clone(),
        node_id: envelope.node_id,
        origin: envelope.origin,
        status,
        expected_document: envelope.expected_document,
        current_document,
        document_edit,
        failure,
    };
    debug_assert!(receipt.validate().is_ok());
    receipt
}
