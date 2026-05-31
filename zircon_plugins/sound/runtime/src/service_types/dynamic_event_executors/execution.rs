use zircon_runtime::core::framework::sound::{
    SoundDynamicEventExecutionReport, SoundDynamicEventExecutionStatus,
    SoundDynamicEventHandlerExecution, SoundError,
};

use crate::dynamic_events::dispatch::dispatch_dynamic_events;
use crate::engine::SoundDynamicEventExecutorKey;

use super::super::DefaultSoundManager;

impl DefaultSoundManager {
    pub(in crate::service_types) fn execute_dynamic_events_impl(
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
