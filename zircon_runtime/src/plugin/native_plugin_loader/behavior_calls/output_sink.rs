//! 同步命令回调的宿主输出端；context 借用当前调用栈，输出大小由加载时清单预算约束。
//! 错误和 panic 标志留在宿主，外来回调自行返回 OK 也不能抹去 sink 的拒绝。

use super::super::abi_declarations::{
    NativePluginByteSliceV3, NativePluginCallbackStatusV3, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
    ZIRCON_NATIVE_PLUGIN_STATUS_OK, ZIRCON_NATIVE_PLUGIN_STATUS_PANIC,
};
use super::super::ffi_panic_guard::catch_native_plugin_output_sink_panic;

const COMMAND_OUTPUT_LIMIT_DIAGNOSTICS: &[u8] =
    b"native plugin command output exceeds its declared host-owned limit\0";
const COMMAND_OUTPUT_INVALID_SLICE_DIAGNOSTICS: &[u8] =
    b"native plugin command output sink received a null byte slice\0";
const COMMAND_OUTPUT_ALLOCATION_DIAGNOSTICS: &[u8] =
    b"native plugin command output host sink could not reserve capacity\0";

#[derive(Debug)]
pub(super) struct NativePluginHostOutput {
    max_output_bytes: usize,
    pub(super) bytes: Vec<u8>,
    pub(super) diagnostics: Vec<String>,
    pub(super) sink_failed: bool,
    pub(super) sink_panicked: bool,
}

impl NativePluginHostOutput {
    pub(super) fn new(max_output_bytes: usize) -> Self {
        Self {
            max_output_bytes,
            bytes: Vec::new(),
            diagnostics: Vec::new(),
            sink_failed: false,
            sink_panicked: false,
        }
    }
}

/// 仅作为本次命令下发的 sink 回调使用，不是可保存或跨线程共享的输出句柄。
///
/// # Safety
/// context 必须是宿主传给当前同步 invoke 的原始指针，保持有效、正确对齐并可独占访问；
/// 非空 chunk 须在此次写入期间可读，外来代码不得并发或返回后再使用同一 context。
pub(super) unsafe extern "C" fn write_host_output_v4(
    context: *mut std::ffi::c_void,
    chunk: NativePluginByteSliceV3,
) -> NativePluginCallbackStatusV3 {
    if context.is_null() {
        return callback_status_error(COMMAND_OUTPUT_INVALID_SLICE_DIAGNOSTICS);
    }
    // SAFETY: 插件按当前同步 sink 契约原样回传有效、对齐且独占的栈上 context，宿主不保存该裸指针。
    let output = unsafe { &mut *context.cast::<NativePluginHostOutput>() };
    let status = catch_native_plugin_output_sink_panic(|| unsafe {
        write_host_output_v4_inner(output, chunk)
    });
    if status.code == ZIRCON_NATIVE_PLUGIN_STATUS_PANIC {
        output.sink_failed = true;
        output.sink_panicked = true;
    }
    status
}

unsafe fn write_host_output_v4_inner(
    output: &mut NativePluginHostOutput,
    chunk: NativePluginByteSliceV3,
) -> NativePluginCallbackStatusV3 {
    if chunk.data.is_null() && chunk.len != 0 {
        output.sink_failed = true;
        output
            .diagnostics
            .push("native plugin command output sink received a null byte slice".to_string());
        return callback_status_error(COMMAND_OUTPUT_INVALID_SLICE_DIAGNOSTICS);
    }
    let Some(next_len) = output.bytes.len().checked_add(chunk.len) else {
        output.sink_failed = true;
        output
            .diagnostics
            .push("native plugin command output length overflowed the host sink".to_string());
        return callback_status_error(COMMAND_OUTPUT_LIMIT_DIAGNOSTICS);
    };
    if next_len > output.max_output_bytes {
        output.sink_failed = true;
        output.diagnostics.push(format!(
            "native plugin command output exceeded its declared {} byte limit",
            output.max_output_bytes
        ));
        return callback_status_error(COMMAND_OUTPUT_LIMIT_DIAGNOSTICS);
    }
    if chunk.len != 0 {
        if let Err(error) = output.bytes.try_reserve(chunk.len) {
            output.sink_failed = true;
            output.diagnostics.push(format!(
                "native plugin command output host sink could not reserve {} bytes: {error}",
                chunk.len
            ));
            return callback_status_error(COMMAND_OUTPUT_ALLOCATION_DIAGNOSTICS);
        }
        // SAFETY: 空片段已绕过此分支，长度与容量预算已检查；chunk 的可读性仍依赖当前插件回调的 ABI 契约。
        let bytes = unsafe { std::slice::from_raw_parts(chunk.data, chunk.len) };
        output.bytes.extend_from_slice(bytes);
    }
    NativePluginCallbackStatusV3 {
        code: ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        diagnostics: std::ptr::null(),
    }
}

fn callback_status_error(diagnostics: &'static [u8]) -> NativePluginCallbackStatusV3 {
    NativePluginCallbackStatusV3 {
        code: ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
        diagnostics: diagnostics.as_ptr().cast(),
    }
}
