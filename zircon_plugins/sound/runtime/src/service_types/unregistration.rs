use zircon_runtime::core::framework::sound::SoundError;

use crate::engine::SoundDynamicEventExecutorKey;

use super::super::DefaultSoundManager;

impl DefaultSoundManager {
    /// 只撤销后续派发使用的注册；已经复制的执行快照仍可调用旧回调，不能把成功返回当作执行屏障。
    pub fn unregister_dynamic_event_executor(
        &self,
        plugin_id: &str,
        handler_id: &str,
    ) -> Result<(), SoundError> {
        let mut state = crate::poison_recovery::lock_recover(&self.state);
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
}
