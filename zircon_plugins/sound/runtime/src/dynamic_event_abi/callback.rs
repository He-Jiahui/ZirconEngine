//! ABI 回调适配器把插件函数指针交给普通动态事件执行器；卸载前必须处理其注册身份与代码地址的生命周期。
use zircon_runtime_interface::ZrPluginEventCallbackFnV1;

use crate::service_types::DefaultSoundManager;

use super::executor::sound_dynamic_event_callback_executor;

impl DefaultSoundManager {
    // TODO: [CR-SOUND-AUDIT-0004] 确认原生库卸载前如何撤销此函数指针；注册闭包长期保存在声音状态中，目前未见卸载/热重载桥接，需验证代码地址生命周期。
    /// 附着到已登记的处理器；原生回调所属代码须保持存活至已取出的执行快照全部结束。
    pub fn register_dynamic_event_abi_callback(
        &self,
        plugin_id: impl Into<String>,
        handler_id: impl Into<String>,
        callback: ZrPluginEventCallbackFnV1,
    ) -> Result<(), zircon_runtime::core::framework::sound::SoundError> {
        self.register_dynamic_event_executor(
            plugin_id,
            handler_id,
            sound_dynamic_event_callback_executor(callback),
        )
    }
}
