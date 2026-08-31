use std::collections::BTreeSet;

use crate::core::editor_message::DocumentId;
use crate::core::play::WorldDomain;

use super::{EditCommandError, HistoryContextId};

pub fn resolve_history_context(
    world_domain: WorldDomain,
    explicit: Option<HistoryContextId>,
    target_document: Option<DocumentId>,
    participants: &BTreeSet<DocumentId>,
) -> Result<HistoryContextId, EditCommandError> {
    if let WorldDomain::Play(instance) = world_domain {
        let play_history = HistoryContextId::PlaySession(instance);
        if let Some(requested) = explicit.filter(|requested| *requested != play_history) {
            return Err(EditCommandError::CrossWorldHistory {
                world_domain,
                requested,
            });
        }
        if target_document.is_some() || !participants.is_empty() {
            let requested = target_document
                .map(HistoryContextId::Document)
                .unwrap_or(HistoryContextId::Global);
            return Err(EditCommandError::CrossWorldHistory {
                world_domain,
                requested,
            });
        }
        return Ok(HistoryContextId::PlaySession(instance));
    }

    if let Some(requested @ HistoryContextId::PlaySession(_)) = explicit {
        return Err(EditCommandError::CrossWorldHistory {
            world_domain,
            requested,
        });
    }
    if let Some(context) = explicit {
        return Ok(context);
    }
    if participants.len() > 1 {
        return Ok(HistoryContextId::Global);
    }
    Ok(match target_document.map(HistoryContextId::Document) {
        Some(context) => context,
        None => HistoryContextId::Global,
    })
}
