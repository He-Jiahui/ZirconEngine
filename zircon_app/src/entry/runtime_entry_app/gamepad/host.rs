//! App 创建期间的可选原生手柄初始化。
//! 状态更新由轮询调用点承担，初始化失败只禁用该输入源。

use gilrs::GilrsBuilder;
use zircon_runtime::diagnostic_log::write_warn;

/// 创建由 App 显式更新的手柄宿主；失败时保留产品启动并返回无手柄状态。
pub(in crate::entry::runtime_entry_app) fn create_gilrs() -> Option<gilrs::Gilrs> {
    match GilrsBuilder::new()
        .with_default_filters(false)
        .set_update_state(false)
        .build()
    {
        Ok(gilrs) => Some(gilrs),
        Err(error) => {
            write_warn(
                "runtime_gamepad",
                format!("runtime_gamepad_gilrs_unavailable: {error}"),
            );
            None
        }
    }
}
