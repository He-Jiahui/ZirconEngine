use std::ffi::CString;

use super::abi_declarations::{
    NativePluginBehaviorV2, NativePluginBehaviorV3, NativePluginByteSliceV2,
    NativePluginCallbackStatusV2, NativePluginInvokeCommandFnV2, NativePluginOwnedByteBufferV2,
    NativePluginRestoreStateFnV2, NativePluginSaveStateFnV2, NativePluginUnloadFnV2,
    ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V2, ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
    ZIRCON_NATIVE_PLUGIN_STATUS_ERROR, ZIRCON_NATIVE_PLUGIN_STATUS_OK,
};
use super::native_strings::read_optional_c_string;

#[derive(Debug)]
pub(super) struct NativePluginBehavior {
    pub(super) is_stateless: bool,
    pub(super) state_schema_version: u32,
    pub(super) command_manifest_schema: Option<String>,
    pub(super) event_manifest_schema: Option<String>,
    pub(super) command_manifest: Option<String>,
    pub(super) event_manifest: Option<String>,
    pub(super) invoke_command: Option<NativePluginInvokeCommandFnV2>,
    pub(super) save_state: Option<NativePluginSaveStateFnV2>,
    pub(super) restore_state: Option<NativePluginRestoreStateFnV2>,
    pub(super) unload: Option<NativePluginUnloadFnV2>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePluginBehaviorCallReport {
    pub status_code: u32,
    pub diagnostics: Vec<String>,
    pub payload: Option<Vec<u8>>,
}

impl NativePluginBehavior {
    pub(super) unsafe fn from_abi_v2(abi: &NativePluginBehaviorV2) -> Result<Self, String> {
        if abi.abi_version != ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V2 {
            return Err(format!(
                "unsupported native plugin behavior ABI version {}; expected {}",
                abi.abi_version, ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V2
            ));
        }
        Ok(Self {
            is_stateless: abi.is_stateless != 0,
            state_schema_version: 0,
            command_manifest_schema: None,
            event_manifest_schema: None,
            command_manifest: read_optional_c_string(abi.command_manifest),
            event_manifest: read_optional_c_string(abi.event_manifest),
            invoke_command: abi.invoke_command,
            save_state: abi.save_state,
            restore_state: abi.restore_state,
            unload: abi.unload,
        })
    }

    pub(super) unsafe fn from_abi_v3(abi: &NativePluginBehaviorV3) -> Result<Self, String> {
        if abi.abi_version != ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3 {
            return Err(format!(
                "unsupported native plugin behavior ABI version {}; expected {}",
                abi.abi_version, ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3
            ));
        }
        Ok(Self {
            is_stateless: abi.is_stateless != 0,
            state_schema_version: abi.schema_versions.state_schema_version,
            command_manifest_schema: read_optional_c_string(
                abi.schema_versions.command_manifest_schema,
            ),
            event_manifest_schema: read_optional_c_string(
                abi.schema_versions.event_manifest_schema,
            ),
            command_manifest: read_optional_c_string(abi.command_manifest),
            event_manifest: read_optional_c_string(abi.event_manifest),
            invoke_command: abi.invoke_command,
            save_state: abi.save_state,
            restore_state: abi.restore_state,
            unload: abi.unload,
        })
    }

    pub(super) fn invoke_command(
        &self,
        name: &str,
        payload: &[u8],
    ) -> NativePluginBehaviorCallReport {
        let Some(invoke_command) = self.invoke_command else {
            return missing_callback_report("invoke_command");
        };
        let Ok(name) = CString::new(name) else {
            return error_report("native plugin command name contained an interior NUL");
        };
        let mut output = NativePluginOwnedByteBufferV2::empty();
        let status = unsafe {
            invoke_command(
                name.as_ptr(),
                NativePluginByteSliceV2 {
                    data: payload.as_ptr(),
                    len: payload.len(),
                },
                &mut output,
            )
        };
        let mut report = NativePluginBehaviorCallReport::from_status(status);
        report.payload = take_owned_bytes(output, &mut report.diagnostics);
        report
    }

    pub(super) fn save_state(&self) -> NativePluginBehaviorCallReport {
        let Some(save_state) = self.save_state else {
            return missing_callback_report("save_state");
        };
        let mut output = NativePluginOwnedByteBufferV2::empty();
        let status = unsafe { save_state(&mut output) };
        let mut report = NativePluginBehaviorCallReport::from_status(status);
        report.payload = take_owned_bytes(output, &mut report.diagnostics);
        report
    }

    pub(super) fn restore_state(&self, state: &[u8]) -> NativePluginBehaviorCallReport {
        let Some(restore_state) = self.restore_state else {
            return missing_callback_report("restore_state");
        };
        let status = unsafe {
            restore_state(NativePluginByteSliceV2 {
                data: state.as_ptr(),
                len: state.len(),
            })
        };
        NativePluginBehaviorCallReport::from_status(status)
    }

    pub(super) fn unload(&self) -> NativePluginBehaviorCallReport {
        let Some(unload) = self.unload else {
            return missing_callback_report("unload");
        };
        NativePluginBehaviorCallReport::from_status(unsafe { unload() })
    }

    pub(super) fn has_invoke_command(&self) -> bool {
        self.invoke_command.is_some()
    }

    pub(super) fn has_save_state(&self) -> bool {
        self.save_state.is_some()
    }

    pub(super) fn has_restore_state(&self) -> bool {
        self.restore_state.is_some()
    }

    pub(super) fn has_unload(&self) -> bool {
        self.unload.is_some()
    }
}

impl NativePluginBehaviorCallReport {
    fn from_status(status: NativePluginCallbackStatusV2) -> Self {
        Self {
            status_code: status.code,
            diagnostics: status_diagnostics(status),
            payload: None,
        }
    }
}

fn error_report(message: &str) -> NativePluginBehaviorCallReport {
    NativePluginBehaviorCallReport {
        status_code: ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
        diagnostics: vec![message.to_string()],
        payload: None,
    }
}

fn missing_callback_report(callback_name: &str) -> NativePluginBehaviorCallReport {
    error_report(&format!(
        "native plugin behavior callback {callback_name} is missing"
    ))
}

fn status_diagnostics(status: NativePluginCallbackStatusV2) -> Vec<String> {
    unsafe { read_optional_c_string(status.diagnostics) }
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

fn take_owned_bytes(
    output: NativePluginOwnedByteBufferV2,
    diagnostics: &mut Vec<String>,
) -> Option<Vec<u8>> {
    if output.data.is_null() {
        if output.len != 0 || output.capacity != 0 {
            diagnostics.push(format!(
                "native plugin owned buffer was malformed: null data with len {} and capacity {}",
                output.len, output.capacity
            ));
        }
        return None;
    }
    if output.len > output.capacity {
        diagnostics.push(format!(
            "native plugin owned buffer was malformed: len {} exceeds capacity {}",
            output.len, output.capacity
        ));
    }
    let bytes =
        unsafe { std::slice::from_raw_parts(output.data.cast_const(), output.len) }.to_vec();
    let Some(free) = output.free else {
        diagnostics.push("native plugin owned buffer did not provide a free callback".to_string());
        return Some(bytes);
    };
    let free_status = unsafe { free(output) };
    if free_status.code != ZIRCON_NATIVE_PLUGIN_STATUS_OK {
        diagnostics.extend(
            status_diagnostics(free_status)
                .into_iter()
                .map(|message| format!("native plugin owned buffer free failed: {message}")),
        );
    }
    Some(bytes)
}
