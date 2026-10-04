//! 原生行为 ABI 到宿主命令/状态调用的边界。命令先按清单解析名称和输出预算，
//! 输出复制进宿主容器；状态快照仍由插件分配，并经其释放回调跨越分配器边界。
//! 此模块不持有库代次，外调准入由外层行为快照和生命周期调用方负责。

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

// 清单预算同时约束加载时的元数据和调用时的宿主输出；插件不能用成功状态绕过 sink 拒绝。
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

/// 从入口报告复制的行为元数据与回调地址。字符串脱离插件存储，函数指针仍依赖所属库代次。
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
/// 此处所依赖的库代次由外层行为快照持有；本回调快照只复制函数地址。
/// 执行外来回调仍须通过外层快照获取租约，命令表查询本身不取得租约。
#[derive(Clone, Debug)]
pub(super) struct NativePluginBehaviorCallbacks {
    command_table: Option<Arc<NativePluginCommandTable>>,
    invoke_command: Option<NativePluginInvokeCommandFnV4>,
    save_state: Option<NativePluginSaveStateFnV3>,
    restore_state: Option<NativePluginRestoreStateFnV3>,
    unload: Option<NativePluginUnloadFnV3>,
}

/// 一次调用的宿主所有结果；消费方应先判断状态，再决定是否接受 payload。
/// diagnostics 是可长期保存的文本，不保留插件返回的字符串指针。
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
    /// 加载阶段固定命令名称、连续 slot 和输出上限，避免每次命令调用解析外来清单。
    /// 名称按原字符串匹配，允许内嵌 NUL；跨 ABI 实际发送 slot，名称不转换成 C 字符串。
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
    /// 入口报告解码后建立可复制元数据；版本匹配不等于外来指针有效性验证。
    ///
    /// # Safety
    /// 调用者须保证 ABI 结构及非空字符串在复制期间有效，函数地址属于外层持有的库代次；
    /// 加载入口只能校验版本与空指针，不能验证任意插件返回地址的有效性。
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

    /// 供已加载插件建立外层行为快照。此处不增加 active callback 计数，实际外调才取得租约。
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

    /// 编辑器绑定命令时保存展示/校验元数据；不会执行插件，也不会改变当前库代次。
    pub(super) fn command_metadata(&self, name: &str) -> Option<(String, usize)> {
        self.command_table
            .as_ref()
            .and_then(|table| table.resolve(name))
            .map(|command| (command.payload_schema, command.max_output_bytes))
    }

    /// 由持有库代次与准入租约的外层调用；payload 和宿主 sink 仅在同步回调期间有效。
    /// 外来代码不得缓存 sink context，或在返回后/并发地写入同一输出对象。
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
        // SAFETY: 已准入的调用者在整个同步外调期间保活库、payload 与栈上 sink；
        // 插件须按 ABI 契约仅在该期间访问借用，不得缓存或并发使用 sink context。
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
        // SAFETY: 输出描述符由本次调用独占并保持到回调返回；库由外层快照或迁移持有者保活。
        let status = unsafe { save_state(&mut output) };
        let mut report = NativePluginBehaviorCallReport::from_status(status);
        // TODO: [CR-PLUGIN-NATIVE-0002] 确认缺少释放回调或释放失败时仍接受状态快照的策略；当前 save 的 OK 状态不变，热重载可继续；下一步补故障释放回调的迁移测试。
        report.payload = take_owned_bytes(output, &mut report.diagnostics);
        report
    }

    pub(super) fn restore_state(&self, state: &[u8]) -> NativePluginBehaviorCallReport {
        let Some(restore_state) = self.restore_state else {
            return missing_callback_report("restore_state");
        };
        // SAFETY: state 在同步回调期间保持只读借用；普通调用的快照租约或迁移调用方都保持库代次存活。
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
        // SAFETY: 迁移调用方仍持有库；普通快照调用已取得租约，卸载行为回调返回后才释放句柄。
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

// 插件诊断必须在回调返回后仍是有效的 NUL 结尾文本；SDK 的静态诊断符合该约定，立即复制后不外借。
fn status_diagnostics(status: NativePluginCallbackStatusV3) -> Vec<String> {
    unsafe { read_optional_c_string(status.diagnostics) }
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

// 状态快照跨插件分配器边界：宿主复制有效负载，原分配只能交给该描述符的 free 回调。
// 非空 data 的可读范围仍是可信插件 ABI 前提；len/capacity 检查无法验证任意地址。
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
    // SAFETY: SDK 的 owned_bytes 从活着的 Vec 导出 data/len/capacity，并把释放延后到此处；
    // 入口调用方保持库存活，读取完成后才调用插件的 free。其他插件必须遵守同一所有权契约。
    let bytes =
        unsafe { std::slice::from_raw_parts(output.data.cast_const(), output.len) }.to_vec();
    let Some(free) = output.free else {
        diagnostics.push("native plugin owned buffer did not provide a free callback".to_string());
        return Some(bytes);
    };
    // SAFETY: 插件须保持原始分配存活，并把与该分配匹配的 free 函数与 owner_token 一并返回；
    // 宿主只把完整原描述符交回同一释放函数，形状检查本身不证明任意指针有效。
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
#[path = "tests/behavior_calls.rs"]
mod tests;
