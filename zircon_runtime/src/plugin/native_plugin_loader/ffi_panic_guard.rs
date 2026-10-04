//! 宿主提供给原生插件的各类 C ABI 回调在这里统一把 Rust panic 映射成对应状态。
use std::ffi::CStr;
use std::panic::{catch_unwind, AssertUnwindSafe};

use zircon_runtime_interface::{ZrByteSlice, ZrStatus, ZrStatusCode};

use super::abi_declarations::{NativePluginCallbackStatusV3, ZIRCON_NATIVE_PLUGIN_STATUS_PANIC};

pub(super) const NATIVE_PLUGIN_OUTPUT_SINK_PANIC_DIAGNOSTIC: &CStr =
    c"native plugin output sink panic caught at FFI boundary";

/// 供返回 `ZrStatus` 的宿主 API 入口使用；错误状态不能把 panic 解栈传给原生插件。
pub(super) fn catch_native_host_api_panic(call: impl FnOnce() -> ZrStatus) -> ZrStatus {
    catch_native_ffi_panic(call, || {
        ZrStatus::new(
            ZrStatusCode::Panic,
            ZrByteSlice::from_static(b"native host API panic caught at FFI boundary"),
        )
    })
}

/// 供插件入口期的 capability、日志和诊断回调使用，返回 ABI V3 的整数状态。
pub(super) fn catch_native_plugin_host_callback_panic(call: impl FnOnce() -> u32) -> u32 {
    catch_native_ffi_panic(call, || ZIRCON_NATIVE_PLUGIN_STATUS_PANIC)
}

/// 供命令输出 sink 使用；panic 状态携带静态 C 字符串，避免返回悬空诊断指针。
pub(super) fn catch_native_plugin_output_sink_panic(
    call: impl FnOnce() -> NativePluginCallbackStatusV3,
) -> NativePluginCallbackStatusV3 {
    catch_native_ffi_panic(call, || NativePluginCallbackStatusV3 {
        code: ZIRCON_NATIVE_PLUGIN_STATUS_PANIC,
        diagnostics: NATIVE_PLUGIN_OUTPUT_SINK_PANIC_DIAGNOSTIC.as_ptr(),
    })
}

// BUG: [CR-PLUGIN-NATIVE-0101] 回调以带 panic 的 Drop 值作为 panic payload 时，`Err(_)` 离开匹配臂会再次 panic，越过 FFI 防护；证据：仓外 rustc 双层 catch_unwind 复现及本函数的 payload 丢弃路径。
fn catch_native_ffi_panic<Status>(
    call: impl FnOnce() -> Status,
    panic_status: impl FnOnce() -> Status,
) -> Status {
    // 这里只转换可展开的 Rust panic；不会回滚已写入的注册、日志或输出状态，返回码不代表事务恢复。
    match catch_unwind(AssertUnwindSafe(call)) {
        Ok(status) => status,
        Err(_) => panic_status(),
    }
}

#[cfg(test)]
#[path = "tests/ffi_panic_guard.rs"]
mod tests;
