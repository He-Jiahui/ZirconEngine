use crate::ui::surface::UiSurface;

use super::{
    bound_text_model_updates::UiTextModelUpdateState, text_document_session::UiTextDocumentSession,
};

// 失焦是历史隔离和延迟模型刷新生效的边界；先销毁旧历史，再尝试刷新，不能让刷新内容成为旧撤销链的一部分。
// 有界失焦队列溢出时按全局失焦处理，宁可清除历史也不能遗漏失去 owner 的待处理载荷。
pub(super) fn finish_pending_text_focus_loss(
    text_documents: &mut UiTextDocumentSession,
    text_model_updates: &mut UiTextModelUpdateState,
    surface: &mut UiSurface,
) {
    let pending = surface.input.take_focus_loss_owners();
    if pending.overflowed {
        text_documents.discard_all_histories();
        text_model_updates.finish_all_unfocused(text_documents, surface);
        return;
    }
    for owner in pending.owners {
        text_documents.discard_history(&surface.tree.tree_id, owner);
        text_model_updates.finish_focus_loss(text_documents, surface, owner);
    }
}
