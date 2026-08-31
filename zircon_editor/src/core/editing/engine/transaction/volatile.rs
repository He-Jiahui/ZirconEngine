use crate::core::play::{PlayInstanceId, WorldDomain};

use super::{EditCommandError, EditorTransactionEngine, HistoryContextId};

impl EditorTransactionEngine {
    pub fn discard_play_history(&self, instance: PlayInstanceId) -> Result<bool, EditCommandError> {
        self.flush_operation_group()?;
        self.start_operation("discard play history")?;
        let history = HistoryContextId::PlaySession(instance);
        let (mut context, route) = {
            let mut state = self.lock_state();
            if !state.active.is_empty() {
                self.clear_operation_locked(&mut state);
                return Err(EditCommandError::InvariantViolation {
                    invariant: "discarding play history requires no active transaction scope",
                });
            }
            let route = match state.histories.get(&history) {
                Some(store) => match store.world_route() {
                    Ok(Some(route)) => Some(route.clone()),
                    Ok(None) => {
                        self.clear_operation_locked(&mut state);
                        return Err(EditCommandError::InvariantViolation {
                            invariant: "a retained play history must own a world route",
                        });
                    }
                    Err(error) => {
                        self.clear_operation_locked(&mut state);
                        return Err(error);
                    }
                },
                None => None,
            };
            let context = match Self::take_context_from(&mut state) {
                Ok(context) => context,
                Err(error) => {
                    self.clear_operation_locked(&mut state);
                    return Err(error);
                }
            };
            (context, route)
        };
        let authoring_route = match context.capture_world_route(WorldDomain::Edit) {
            Ok(route) => route,
            Err(error) => {
                self.finish_operation(context, false);
                return Err(error);
            }
        };
        if let Some(route) = route.as_ref() {
            if let Err(error) = context.activate_world_route(route) {
                self.finish_operation(context, false);
                return Err(error);
            }
        }
        let mut removed = {
            let mut state = self.lock_state();
            let removed = state
                .histories
                .remove(&history)
                .map(|mut store| store.clear())
                .unwrap_or_default();
            state.history_generations.remove(&history);
            removed
        };

        for record in &mut removed {
            record.finalize(context.as_mut());
        }
        if let Err(error) = context.activate_world_route(&authoring_route) {
            self.finish_operation(context, true);
            return Err(error);
        }
        if let Err(error) = context.retire_world_route(WorldDomain::Play(instance)) {
            self.finish_operation(context, true);
            return Err(error);
        }
        let discarded = route.is_some();
        self.finish_operation(context, false);
        Ok(discarded)
    }
}
