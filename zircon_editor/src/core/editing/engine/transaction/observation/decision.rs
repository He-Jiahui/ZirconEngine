use std::sync::Arc;

use super::super::{
    EditCommandError, EditWorldRoute, EditorTransactionEngine, HistoryContextId, TransactionId,
};

/// An atomic dirty decision, including applied commands that have not entered history.
/// Equality binds a decision to one engine, history, current route, and pending route.
#[derive(Clone, Debug)]
pub struct HistoryDecisionToken {
    lineage: Arc<()>,
    history: HistoryContextId,
    generation: u64,
    route: EditWorldRoute,
    pending: Option<PendingDecision>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PendingDecision {
    transaction: TransactionId,
    applied_scope: TransactionId,
    route: EditWorldRoute,
    applied_revision: u64,
}

impl PartialEq for HistoryDecisionToken {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.lineage, &other.lineage)
            && self.history == other.history
            && self.generation == other.generation
            && self.route == other.route
            && self.pending == other.pending
    }
}
impl Eq for HistoryDecisionToken {}

impl EditorTransactionEngine {
    /// Observes dirty state and its decision identity under the same admission gate.
    /// A query never flushes, saves, commits, or cancels the pending interaction.
    pub fn dirty_history_decision_token(
        &self,
        history: HistoryContextId,
    ) -> Result<Option<HistoryDecisionToken>, EditCommandError> {
        self.start_observation("capture history decision")?;
        let result = self.dirty_history_decision_reserved(history);
        self.clear_operation();
        result
    }

    /// Shares the exact observation contract with an already-held exclusive admission.
    pub(in super::super) fn dirty_history_decision_reserved(
        &self,
        history: HistoryContextId,
    ) -> Result<Option<HistoryDecisionToken>, EditCommandError> {
        let (generation, pending, context) = {
            let mut state = self.lock_state();
            if !Self::observed_history_status(&state, history).dirty {
                return Ok(None);
            }
            let pending = state
                .active
                .iter()
                .any(|active| active.history == history && !active.commands.is_empty())
                .then(|| state.active.iter().find(|active| active.history == history))
                .flatten()
                .map(|root| PendingDecision {
                    transaction: root.id,
                    // Scope removal through commit or automatic rollback changes
                    // pending identity even when the root and apply revision survive.
                    applied_scope: state
                        .active
                        .iter()
                        .rev()
                        .find(|active| active.history == history && !active.commands.is_empty())
                        .expect("a pending decision retains an applied scope")
                        .id,
                    route: root.route.clone(),
                    applied_revision: state.applied_command_revision,
                });
            let context = match Self::take_context_from(&mut state) {
                Ok(context) => context,
                Err(error) => {
                    return Err(error);
                }
            };
            (Self::history_generation(&state, history), pending, context)
        };
        // Client routing code runs outside the state mutex; the operation gate
        // still excludes commands until the context and observation are restored.
        let route = context.capture_world_route(history.world_domain());
        self.restore_operation_context(context, false);
        route.map(|route| {
            Some(HistoryDecisionToken {
                lineage: Arc::clone(&self.save_token_lineage),
                history,
                generation,
                route,
                pending,
            })
        })
    }

    /// A fresh decision cannot transfer commands or history from a retired world route.
    pub(in super::super) fn validate_history_decision_route_reserved(
        &self,
        history: HistoryContextId,
        token: &HistoryDecisionToken,
    ) -> Result<(), EditCommandError> {
        let pending_matches = token
            .pending
            .as_ref()
            .is_none_or(|pending| pending.route == token.route);
        let state = self.lock_state();
        let stored = state
            .histories
            .get(&history)
            .map(|store| store.world_route())
            .transpose()?;
        let stored_matches = stored.flatten().is_none_or(|route| route == &token.route);
        if !pending_matches || !stored_matches {
            return Err(EditCommandError::WorldRouteStale {
                world_domain: history.world_domain(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "decision/tests/cases.rs"]
mod tests;
