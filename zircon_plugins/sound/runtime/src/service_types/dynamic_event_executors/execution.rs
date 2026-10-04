//! 执行阶段先在锁内生成交付快照并克隆回调句柄，再在锁外调用插件代码；缺失执行器以跳过状态报告。
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
            let mut state = crate::poison_recovery::lock_recover(&self.state);
            let state = &mut *state;
            let deliveries = dispatch_dynamic_events(
                &state.dynamic_event_handlers,
                &mut state.pending_dynamic_events,
            );
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
