//! 执行器只能附着到已登记的处理器身份；声学事件目录和处理器必须先就绪，之后才能派发。
use zircon_runtime::core::framework::sound::{SoundDynamicEventDelivery, SoundError};

use crate::engine::{SoundDynamicEventExecutor, SoundDynamicEventExecutorKey};

use super::super::DefaultSoundManager;

impl DefaultSoundManager {
    /// 为已登记的处理器安装或替换执行器；回调在声音状态锁外同步调用，可重新进入经理服务。
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
        let mut state = crate::poison_recovery::lock_recover(&self.state);
        if !state
            .dynamic_event_handlers
            .handlers()
            .iter()
            .any(|handler| {
                handler.plugin_id == key.plugin_id && handler.handler_id == key.handler_id
            })
        {
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
}
