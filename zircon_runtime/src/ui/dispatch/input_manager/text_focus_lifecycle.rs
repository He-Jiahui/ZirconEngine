use crate::ui::surface::UiSurface;

use super::{
    bound_text_model_updates::UiTextModelUpdateState, text_document_session::UiTextDocumentSession,
};

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
