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

pub(super) unsafe extern "C" fn write_host_output_v4(
    context: *mut std::ffi::c_void,
    chunk: NativePluginByteSliceV3,
) -> NativePluginCallbackStatusV3 {
    if context.is_null() {
        return callback_status_error(COMMAND_OUTPUT_INVALID_SLICE_DIAGNOSTICS);
    }
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
