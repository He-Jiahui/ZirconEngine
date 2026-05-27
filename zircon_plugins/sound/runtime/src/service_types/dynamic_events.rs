use zircon_runtime::core::framework::sound::{
    SoundDynamicEventCatalog, SoundDynamicEventDelivery, SoundDynamicEventDescriptor,
    SoundDynamicEventExecutionReport, SoundDynamicEventExecutionStatus,
    SoundDynamicEventHandlerDescriptor, SoundDynamicEventHandlerExecution,
    SoundDynamicEventInvocation, SoundError,
};

use crate::dynamic_events::{
    dispatch_dynamic_events, register_dynamic_event, register_dynamic_event_handler,
    submit_dynamic_event, unregister_dynamic_event, unregister_dynamic_event_handler,
};
use crate::engine::{SoundDynamicEventExecutor, SoundDynamicEventExecutorKey};

use super::DefaultSoundManager;

impl DefaultSoundManager {
    pub fn register_dynamic_event_executor<F>(
        &self,
        plugin_id: impl Into<String>,
        handler_id: impl Into<String>,
        executor: F,
    ) -> Result<(), SoundError>
    where
        F: Fn(&SoundDynamicEventDelivery) -> Result<(), String> + Send + Sync + 'static,
    {
        let key = SoundDynamicEventExecutorKey::new(plugin_id, handler_id);
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        if !state.dynamic_event_handlers.iter().any(|handler| {
            handler.plugin_id == key.plugin_id && handler.handler_id == key.handler_id
        }) {
            return Err(SoundError::UnknownDynamicEventHandler {
                plugin_id: key.plugin_id,
                handler_id: key.handler_id,
            });
        }
        state
            .dynamic_event_executors
            .insert(key, SoundDynamicEventExecutor::new(executor));
        Ok(())
    }

    pub fn unregister_dynamic_event_executor(
        &self,
        plugin_id: &str,
        handler_id: &str,
    ) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let key = SoundDynamicEventExecutorKey::new(plugin_id, handler_id);
        state
            .dynamic_event_executors
            .remove(&key)
            .map(|_| ())
            .ok_or_else(|| SoundError::UnknownDynamicEventHandler {
                plugin_id: plugin_id.to_string(),
                handler_id: handler_id.to_string(),
            })
    }

    pub(super) fn dynamic_event_catalog_impl(
        &self,
    ) -> Result<SoundDynamicEventCatalog, SoundError> {
        Ok(self
            .state
            .lock()
            .expect("sound state mutex poisoned")
            .dynamic_events
            .clone())
    }

    pub(super) fn register_dynamic_event_impl(
        &self,
        descriptor: SoundDynamicEventDescriptor,
    ) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        register_dynamic_event(&mut state.dynamic_events, descriptor)
    }

    pub(super) fn unregister_dynamic_event_impl(&self, event_id: &str) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        unregister_dynamic_event(&mut state.dynamic_events, event_id)?;
        state
            .dynamic_event_handlers
            .retain(|handler| handler.event_id != event_id);
        let handlers = state.dynamic_event_handlers.clone();
        state
            .dynamic_event_executors
            .retain(|key, _| handler_exists(&handlers, key));
        state
            .pending_dynamic_events
            .retain(|event| event.event_id != event_id);
        Ok(())
    }

    pub(super) fn dynamic_event_handlers_impl(
        &self,
    ) -> Result<Vec<SoundDynamicEventHandlerDescriptor>, SoundError> {
        Ok(self
            .state
            .lock()
            .expect("sound state mutex poisoned")
            .dynamic_event_handlers
            .clone())
    }

    pub(super) fn register_dynamic_event_handler_impl(
        &self,
        handler: SoundDynamicEventHandlerDescriptor,
    ) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let catalog = state.dynamic_events.clone();
        register_dynamic_event_handler(&catalog, &mut state.dynamic_event_handlers, handler)
    }

    pub(super) fn unregister_dynamic_event_handler_impl(
        &self,
        plugin_id: &str,
        handler_id: &str,
    ) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        unregister_dynamic_event_handler(&mut state.dynamic_event_handlers, plugin_id, handler_id)?;
        state
            .dynamic_event_executors
            .remove(&SoundDynamicEventExecutorKey::new(plugin_id, handler_id));
        Ok(())
    }

    pub(super) fn submit_dynamic_event_impl(
        &self,
        invocation: SoundDynamicEventInvocation,
    ) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let catalog = state.dynamic_events.clone();
        submit_dynamic_event(&catalog, &mut state.pending_dynamic_events, invocation)
    }

    pub(super) fn drain_dynamic_events_impl(
        &self,
    ) -> Result<Vec<SoundDynamicEventInvocation>, SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        Ok(state.pending_dynamic_events.drain(..).collect())
    }

    pub(super) fn dispatch_dynamic_events_impl(
        &self,
    ) -> Result<Vec<SoundDynamicEventDelivery>, SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let handlers = state.dynamic_event_handlers.clone();
        Ok(dispatch_dynamic_events(
            &handlers,
            &mut state.pending_dynamic_events,
        ))
    }

    pub(super) fn execute_dynamic_events_impl(
        &self,
    ) -> Result<SoundDynamicEventExecutionReport, SoundError> {
        let (deliveries, executors) = {
            let mut state = self.state.lock().expect("sound state mutex poisoned");
            let handlers = state.dynamic_event_handlers.clone();
            let deliveries = dispatch_dynamic_events(&handlers, &mut state.pending_dynamic_events);
            (deliveries, state.dynamic_event_executors.clone())
        };
        let executions = deliveries
            .into_iter()
            .map(|delivery| {
                let key = SoundDynamicEventExecutorKey::from_handler(&delivery.handler);
                match executors.get(&key) {
                    Some(executor) => match executor.execute(&delivery) {
                        Ok(()) => SoundDynamicEventHandlerExecution {
                            delivery,
                            status: SoundDynamicEventExecutionStatus::Succeeded,
                            detail: None,
                        },
                        Err(detail) => SoundDynamicEventHandlerExecution {
                            delivery,
                            status: SoundDynamicEventExecutionStatus::Failed,
                            detail: Some(detail),
                        },
                    },
                    None => SoundDynamicEventHandlerExecution {
                        delivery,
                        status: SoundDynamicEventExecutionStatus::SkippedMissingExecutor,
                        detail: None,
                    },
                }
            })
            .collect();
        Ok(SoundDynamicEventExecutionReport { executions })
    }
}

fn handler_exists(
    handlers: &[SoundDynamicEventHandlerDescriptor],
    key: &SoundDynamicEventExecutorKey,
) -> bool {
    handlers
        .iter()
        .any(|handler| handler.plugin_id == key.plugin_id && handler.handler_id == key.handler_id)
}
