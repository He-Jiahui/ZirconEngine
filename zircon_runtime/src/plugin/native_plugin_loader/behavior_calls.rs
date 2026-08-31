use std::collections::BTreeMap;
use std::sync::Arc;

use serde::Deserialize;

mod output_sink;

use super::abi_declarations::{
    NativePluginBehaviorV4, NativePluginByteSliceV3, NativePluginCallbackStatusV3,
    NativePluginInvokeCommandFnV4, NativePluginOutputSinkV4, NativePluginOwnedByteBufferV3,
    NativePluginRestoreStateFnV3, NativePluginSaveStateFnV3, NativePluginUnloadFnV3,
    ZIRCON_NATIVE_PLUGIN_BEHAVIOR_ABI_VERSION_V4, ZIRCON_NATIVE_PLUGIN_STATUS_DENIED,
    ZIRCON_NATIVE_PLUGIN_STATUS_ERROR, ZIRCON_NATIVE_PLUGIN_STATUS_OK,
    ZIRCON_NATIVE_PLUGIN_STATUS_PANIC,
};
use super::behavior_validation::ZIRCON_NATIVE_COMMAND_MANIFEST_SCHEMA_V4;
use super::ffi_panic_guard::NATIVE_PLUGIN_OUTPUT_SINK_PANIC_DIAGNOSTIC;
use super::native_strings::read_optional_c_string;
use output_sink::{write_host_output_v4, NativePluginHostOutput};

pub(super) const NATIVE_COMMAND_MAX_OUTPUT_BYTES_V4: usize = 256 * 1024 * 1024;

pub(super) type NativePluginBehaviorResult<T> = std::result::Result<T, NativePluginBehaviorError>;

#[derive(Debug)]
pub(super) enum NativePluginBehaviorError {
    UnsupportedAbiVersion { actual: u32, expected: u32 },
    InvalidCommandManifest { reason: String },
}

impl std::fmt::Display for NativePluginBehaviorError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedAbiVersion { actual, expected } => write!(
                formatter,
                "unsupported native plugin behavior ABI version {actual}; expected {expected}"
            ),
            Self::InvalidCommandManifest { reason } => {
                write!(
                    formatter,
                    "invalid native plugin command manifest v4: {reason}"
                )
            }
        }
    }
}

impl std::error::Error for NativePluginBehaviorError {}

#[derive(Clone, Debug)]
pub(super) struct NativePluginBehavior {
    pub(super) is_stateless: bool,
    pub(super) state_schema_version: u32,
    pub(super) command_manifest_schema: Option<String>,
    pub(super) event_manifest_schema: Option<String>,
    pub(super) registration_manifest_schema: Option<String>,
    pub(super) command_manifest: Option<String>,
    pub(super) event_manifest: Option<String>,
    pub(super) registration_manifest: Option<String>,
    pub(super) command_table: Option<Arc<NativePluginCommandTable>>,
    pub(super) invoke_command: Option<NativePluginInvokeCommandFnV4>,
    pub(super) save_state: Option<NativePluginSaveStateFnV3>,
    pub(super) restore_state: Option<NativePluginRestoreStateFnV3>,
    pub(super) unload: Option<NativePluginUnloadFnV3>,
}

/// The callback snapshot retains the stable library generation without holding a callback lease.
/// A lease is acquired only for foreign dispatch; the immutable host table needs no plugin or host
/// mutex for command lookup.
#[derive(Clone, Debug)]
pub(super) struct NativePluginBehaviorCallbacks {
    command_table: Option<Arc<NativePluginCommandTable>>,
    invoke_command: Option<NativePluginInvokeCommandFnV4>,
    save_state: Option<NativePluginSaveStateFnV3>,
    restore_state: Option<NativePluginRestoreStateFnV3>,
    unload: Option<NativePluginUnloadFnV3>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePluginBehaviorCallReport {
    pub status_code: u32,
    pub diagnostics: Vec<String>,
    pub payload: Option<Vec<u8>>,
}

#[derive(Clone, Debug)]
pub(super) struct NativePluginCommandTable {
    commands: BTreeMap<String, NativePluginCommandBinding>,
}

#[derive(Clone, Debug)]
struct NativePluginCommandBinding {
    slot: u32,
    payload_schema: String,
    max_output_bytes: usize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct NativePluginCommandManifestV4 {
    schema: String,
    #[serde(default)]
    commands: Vec<NativePluginCommandV4>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct NativePluginCommandV4 {
    name: String,
    slot: u32,
    payload_schema: String,
    max_output_bytes: usize,
}

impl NativePluginCommandTable {
    pub(super) fn from_manifest_v4(manifest: &str) -> NativePluginBehaviorResult<Self> {
        let manifest =
            toml::from_str::<NativePluginCommandManifestV4>(manifest).map_err(|error| {
                NativePluginBehaviorError::InvalidCommandManifest {
                    reason: error.to_string(),
                }
            })?;
        if manifest.schema.trim() != ZIRCON_NATIVE_COMMAND_MANIFEST_SCHEMA_V4 {
            return Err(NativePluginBehaviorError::InvalidCommandManifest {
                reason: format!(
                    "schema is {}; expected {ZIRCON_NATIVE_COMMAND_MANIFEST_SCHEMA_V4}",
                    manifest.schema.trim()
                ),
            });
        }

        let mut commands = BTreeMap::new();
        for (index, command) in manifest.commands.into_iter().enumerate() {
            let expected_slot = u32::try_from(index).map_err(|_| {
                NativePluginBehaviorError::InvalidCommandManifest {
                    reason: "contains more commands than the v4 slot address space".to_string(),
                }
            })?;
            if command.slot != expected_slot {
                return Err(NativePluginBehaviorError::InvalidCommandManifest {
                    reason: format!(
                        "command {} uses slot {}; expected dense slot {expected_slot}",
                        command.name, command.slot
                    ),
                });
            }
            if command.name.is_empty() {
                return Err(NativePluginBehaviorError::InvalidCommandManifest {
                    reason: format!("command slot {expected_slot} has an empty name"),
                });
            }
            if command.payload_schema.trim().is_empty() {
                return Err(NativePluginBehaviorError::InvalidCommandManifest {
                    reason: format!("command {} has an empty payload schema", command.name),
                });
            }
            if command.max_output_bytes > NATIVE_COMMAND_MAX_OUTPUT_BYTES_V4 {
                return Err(NativePluginBehaviorError::InvalidCommandManifest {
                    reason: format!(
                        "command {} declares {} output bytes; maximum is {NATIVE_COMMAND_MAX_OUTPUT_BYTES_V4}",
                        command.name, command.max_output_bytes
                    ),
                });
            }
            if commands
                .insert(
                    command.name.clone(),
                    NativePluginCommandBinding {
                        slot: command.slot,
                        payload_schema: command.payload_schema,
                        max_output_bytes: command.max_output_bytes,
                    },
                )
                .is_some()
            {
                return Err(NativePluginBehaviorError::InvalidCommandManifest {
                    reason: format!("command name {} is declared more than once", command.name),
                });
            }
        }
        Ok(Self { commands })
    }

    fn resolve(&self, name: &str) -> Option<NativePluginCommandBinding> {
        self.commands.get(name).cloned()
    }
}

impl NativePluginBehavior {
    pub(super) unsafe fn from_abi_v4(
        abi: &NativePluginBehaviorV4,
    ) -> NativePluginBehaviorResult<Self> {
        if abi.abi_version != ZIRCON_NATIVE_PLUGIN_BEHAVIOR_ABI_VERSION_V4 {
            return Err(NativePluginBehaviorError::UnsupportedAbiVersion {
                actual: abi.abi_version,
                expected: ZIRCON_NATIVE_PLUGIN_BEHAVIOR_ABI_VERSION_V4,
            });
        }
        let command_manifest = read_optional_c_string(abi.command_manifest);
        let command_table = command_manifest
            .as_deref()
            .map(NativePluginCommandTable::from_manifest_v4)
            .transpose()?
            .map(Arc::new);
        Ok(Self {
            is_stateless: abi.is_stateless != 0,
            state_schema_version: abi.schema_versions.state_schema_version,
            command_manifest_schema: read_optional_c_string(
                abi.schema_versions.command_manifest_schema,
            ),
            event_manifest_schema: read_optional_c_string(
                abi.schema_versions.event_manifest_schema,
            ),
            registration_manifest_schema: read_optional_c_string(
                abi.schema_versions.registration_manifest_schema,
            ),
            command_manifest,
            event_manifest: read_optional_c_string(abi.event_manifest),
            registration_manifest: read_optional_c_string(abi.registration_manifest),
            command_table,
            invoke_command: abi.invoke_command,
            save_state: abi.save_state,
            restore_state: abi.restore_state,
            unload: abi.unload,
        })
    }

    pub(super) fn save_state(&self) -> NativePluginBehaviorCallReport {
        self.callback_snapshot().save_state()
    }

    pub(super) fn restore_state(&self, state: &[u8]) -> NativePluginBehaviorCallReport {
        self.callback_snapshot().restore_state(state)
    }

    pub(super) fn unload(&self) -> NativePluginBehaviorCallReport {
        self.callback_snapshot().unload()
    }

    pub(super) fn callback_snapshot(&self) -> NativePluginBehaviorCallbacks {
        NativePluginBehaviorCallbacks {
            command_table: self.command_table.clone(),
            invoke_command: self.invoke_command,
            save_state: self.save_state,
            restore_state: self.restore_state,
            unload: self.unload,
        }
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

impl NativePluginBehaviorCallbacks {
    pub(super) fn has_invoke_command(&self) -> bool {
        self.invoke_command.is_some()
    }

    pub(super) fn declares_command(&self, name: &str) -> bool {
        self.command_table
            .as_ref()
            .is_some_and(|table| table.resolve(name).is_some())
    }

    pub(super) fn command_metadata(&self, name: &str) -> Option<(String, usize)> {
        self.command_table
            .as_ref()
            .and_then(|table| table.resolve(name))
            .map(|command| (command.payload_schema, command.max_output_bytes))
    }

    pub(super) fn invoke_command(
        &self,
        name: &str,
        payload: &[u8],
    ) -> NativePluginBehaviorCallReport {
        let Some(invoke_command) = self.invoke_command else {
            return missing_callback_report("invoke_command");
        };
        let Some(command_table) = &self.command_table else {
            return error_report("native plugin behavior has no v4 command manifest table");
        };
        let Some(command) = command_table.resolve(name) else {
            return NativePluginBehaviorCallReport {
                status_code: ZIRCON_NATIVE_PLUGIN_STATUS_DENIED,
                diagnostics: vec![format!(
                    "native plugin command {name} is not declared in its v4 manifest"
                )],
                payload: None,
            };
        };

        let mut output = NativePluginHostOutput::new(command.max_output_bytes);
        let status = unsafe {
            invoke_command(
                command.slot,
                NativePluginByteSliceV3 {
                    data: payload.as_ptr(),
                    len: payload.len(),
                },
                NativePluginOutputSinkV4 {
                    context: (&mut output as *mut NativePluginHostOutput).cast(),
                    max_output_bytes: command.max_output_bytes,
                    write: Some(write_host_output_v4),
                },
            )
        };
        let mut report = NativePluginBehaviorCallReport::from_status(status);
        report.diagnostics.append(&mut output.diagnostics);
        // The callback controls its own status, but cannot turn a host-owned sink rejection into
        // a successful command result or expose output that was only partially written.
        if output.sink_panicked {
            report.status_code = ZIRCON_NATIVE_PLUGIN_STATUS_PANIC;
            let diagnostic = NATIVE_PLUGIN_OUTPUT_SINK_PANIC_DIAGNOSTIC
                .to_string_lossy()
                .into_owned();
            if !report.diagnostics.contains(&diagnostic) {
                report.diagnostics.push(diagnostic);
            }
        } else if output.sink_failed {
            report.status_code = ZIRCON_NATIVE_PLUGIN_STATUS_ERROR;
        } else if !output.bytes.is_empty() {
            report.payload = Some(output.bytes);
        }
        report
    }

    pub(super) fn save_state(&self) -> NativePluginBehaviorCallReport {
        let Some(save_state) = self.save_state else {
            return missing_callback_report("save_state");
        };
        let mut output = NativePluginOwnedByteBufferV3::empty();
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
            restore_state(NativePluginByteSliceV3 {
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
}

impl NativePluginBehaviorCallReport {
    fn from_status(status: NativePluginCallbackStatusV3) -> Self {
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

fn status_diagnostics(status: NativePluginCallbackStatusV3) -> Vec<String> {
    unsafe { read_optional_c_string(status.diagnostics) }
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

fn take_owned_bytes(
    output: NativePluginOwnedByteBufferV3,
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
        // Both fields are foreign ABI input. Do not read through the pointer or hand this
        // malformed descriptor back to a plugin free callback.
        return None;
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

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::super::abi_declarations::NativePluginSchemaVersionsV3;
    use super::*;

    fn minimal_behavior(abi_version: u32) -> NativePluginBehaviorV4 {
        NativePluginBehaviorV4 {
            abi_version,
            is_stateless: 1,
            schema_versions: NativePluginSchemaVersionsV3 {
                state_schema_version: 0,
                command_manifest_schema: std::ptr::null(),
                event_manifest_schema: std::ptr::null(),
                registration_manifest_schema: std::ptr::null(),
            },
            command_manifest: std::ptr::null(),
            event_manifest: std::ptr::null(),
            registration_manifest: std::ptr::null(),
            invoke_command: None,
            save_state: None,
            restore_state: None,
            unload: None,
        }
    }

    #[test]
    fn native_behavior_reports_unsupported_abi_version_with_typed_error() {
        let behavior = minimal_behavior(ZIRCON_NATIVE_PLUGIN_BEHAVIOR_ABI_VERSION_V4 + 1);
        let error = unsafe { NativePluginBehavior::from_abi_v4(&behavior) }
            .expect_err("unsupported behavior ABI should report typed error");

        assert!(matches!(
            error,
            NativePluginBehaviorError::UnsupportedAbiVersion { actual, expected }
                if actual == ZIRCON_NATIVE_PLUGIN_BEHAVIOR_ABI_VERSION_V4 + 1
                    && expected == ZIRCON_NATIVE_PLUGIN_BEHAVIOR_ABI_VERSION_V4
        ));
    }

    #[test]
    fn native_behavior_v4_resolves_dense_slot_without_c_string_or_plugin_owned_buffer() {
        unsafe extern "C" fn write_echo(
            slot: u32,
            _payload: NativePluginByteSliceV3,
            output: NativePluginOutputSinkV4,
        ) -> NativePluginCallbackStatusV3 {
            assert_eq!(slot, 0);
            let bytes = b"host-owned";
            unsafe {
                output.write.expect("host writer")(
                    output.context,
                    NativePluginByteSliceV3 {
                        data: bytes.as_ptr(),
                        len: bytes.len(),
                    },
                )
            }
        }

        let command_manifest = r#"
            schema = "zircon.native.command-manifest/4"
            [[commands]]
            name = "nul\u0000safe"
            slot = 0
            payload_schema = "bytes"
            max_output_bytes = 32
        "#;
        let behavior = NativePluginBehavior {
            is_stateless: true,
            state_schema_version: 0,
            command_manifest_schema: Some(ZIRCON_NATIVE_COMMAND_MANIFEST_SCHEMA_V4.to_string()),
            event_manifest_schema: None,
            registration_manifest_schema: None,
            command_manifest: Some(command_manifest.to_string()),
            event_manifest: None,
            registration_manifest: None,
            command_table: Some(Arc::new(
                NativePluginCommandTable::from_manifest_v4(command_manifest).unwrap(),
            )),
            invoke_command: Some(write_echo),
            save_state: None,
            restore_state: None,
            unload: None,
        };

        let callbacks = behavior.callback_snapshot();
        assert!(callbacks.has_invoke_command());
        assert!(callbacks.declares_command("nul\0safe"));
        assert!(!callbacks.declares_command("undeclared"));

        let report = callbacks.invoke_command("nul\0safe", b"ignored");
        assert_eq!(report.status_code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
        assert_eq!(report.payload.as_deref(), Some(&b"host-owned"[..]));
    }

    #[test]
    fn native_behavior_v4_rejects_callback_that_ignores_host_sink_failure() {
        unsafe extern "C" fn ignore_sink_failure(
            _slot: u32,
            _payload: NativePluginByteSliceV3,
            output: NativePluginOutputSinkV4,
        ) -> NativePluginCallbackStatusV3 {
            let bytes = b"exceeds-limit";
            let _ = unsafe {
                output.write.expect("host writer")(
                    output.context,
                    NativePluginByteSliceV3 {
                        data: bytes.as_ptr(),
                        len: bytes.len(),
                    },
                )
            };
            NativePluginCallbackStatusV3 {
                code: ZIRCON_NATIVE_PLUGIN_STATUS_OK,
                diagnostics: std::ptr::null(),
            }
        }

        let command_manifest = r#"
            schema = "zircon.native.command-manifest/4"
            [[commands]]
            name = "bounded"
            slot = 0
            payload_schema = "bytes"
            max_output_bytes = 4
        "#;
        let behavior = NativePluginBehavior {
            is_stateless: true,
            state_schema_version: 0,
            command_manifest_schema: Some(ZIRCON_NATIVE_COMMAND_MANIFEST_SCHEMA_V4.to_string()),
            event_manifest_schema: None,
            registration_manifest_schema: None,
            command_manifest: Some(command_manifest.to_string()),
            event_manifest: None,
            registration_manifest: None,
            command_table: Some(Arc::new(
                NativePluginCommandTable::from_manifest_v4(command_manifest).unwrap(),
            )),
            invoke_command: Some(ignore_sink_failure),
            save_state: None,
            restore_state: None,
            unload: None,
        };

        let report = behavior.callback_snapshot().invoke_command("bounded", b"");

        assert_eq!(report.status_code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR);
        assert!(report.payload.is_none());
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains("exceeded its declared 4 byte limit")));
    }

    #[test]
    fn native_behavior_v4_rejects_non_dense_duplicate_and_oversized_command_metadata() {
        for manifest in [
            r#"schema = "zircon.native.command-manifest/4"
[[commands]]
name = "first"
slot = 1
payload_schema = "bytes"
max_output_bytes = 1"#,
            r#"schema = "zircon.native.command-manifest/4"
[[commands]]
name = "first"
slot = 0
payload_schema = "bytes"
max_output_bytes = 1
[[commands]]
name = "first"
slot = 1
payload_schema = "bytes"
max_output_bytes = 1"#,
            r#"schema = "zircon.native.command-manifest/4"
[[commands]]
name = "first"
slot = 0
payload_schema = "bytes"
max_output_bytes = 268435457"#,
        ] {
            assert!(NativePluginCommandTable::from_manifest_v4(manifest).is_err());
        }
    }

    #[test]
    fn native_behavior_v4_rejects_unknown_command_manifest_fields() {
        for manifest in [
            r#"schema = "zircon.native.command-manifest/4"
unexpected_root_field = true"#,
            r#"schema = "zircon.native.command-manifest/4"
[[commands]]
name = "first"
slot = 0
payload_schema = "bytes"
max_output_bytes = 1
unexpected_command_field = true"#,
        ] {
            assert!(NativePluginCommandTable::from_manifest_v4(manifest).is_err());
        }
    }

    #[test]
    fn native_behavior_rejects_malformed_owned_buffer_before_copying_or_freeing() {
        let backing = *b"ok";
        let buffer = NativePluginOwnedByteBufferV3 {
            data: backing.as_ptr() as *mut u8,
            len: backing.len(),
            capacity: backing.len() - 1,
            owner_token: 0,
            free: None,
        };
        let mut diagnostics = Vec::new();

        let payload = take_owned_bytes(buffer, &mut diagnostics);

        assert!(payload.is_none());
        assert_eq!(
            diagnostics,
            vec!["native plugin owned buffer was malformed: len 2 exceeds capacity 1"]
        );
    }

    #[test]
    fn native_behavior_host_output_rejects_unallocatable_chunk_before_reading_it() {
        let mut output = NativePluginHostOutput::new(usize::MAX);

        let status = unsafe {
            write_host_output_v4(
                (&mut output as *mut NativePluginHostOutput).cast(),
                NativePluginByteSliceV3 {
                    data: std::ptr::NonNull::<u8>::dangling().as_ptr(),
                    len: usize::MAX,
                },
            )
        };

        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR);
        assert!(output.bytes.is_empty());
        assert!(output
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains("could not reserve")));
    }

    #[test]
    fn native_behavior_typed_error_preserves_unsupported_abi_message() {
        let error = NativePluginBehaviorError::UnsupportedAbiVersion {
            actual: ZIRCON_NATIVE_PLUGIN_BEHAVIOR_ABI_VERSION_V4 + 2,
            expected: ZIRCON_NATIVE_PLUGIN_BEHAVIOR_ABI_VERSION_V4,
        };

        assert_eq!(
            error.to_string(),
            format!(
                "unsupported native plugin behavior ABI version {}; expected {}",
                ZIRCON_NATIVE_PLUGIN_BEHAVIOR_ABI_VERSION_V4 + 2,
                ZIRCON_NATIVE_PLUGIN_BEHAVIOR_ABI_VERSION_V4
            )
        );
    }
}
