//! 将已准入动态库的原生描述符和入口报告转换为宿主拥有的加载信息。
//! 函数地址的长期寿命由加载代际和回调租约管理。

use std::ffi::CString;
use std::path::Path;

use libloading::Library;
use zircon_runtime_interface::{
    SerializedContributionBatch, SERIALIZED_EDITOR_CONTRIBUTION_BATCH_SCHEMA_V1,
};

use crate::plugin::{PluginModuleKind, PluginPackageManifest};

use super::abi_declarations::{
    NativePluginAbiV3, NativePluginEntryReportV3, NativePluginHostFunctionTableV3,
    ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3, ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL_V3,
    ZIRCON_NATIVE_PLUGIN_ENTRY_REPORT_LAYOUT_EPOCH,
};
use super::behavior_calls::NativePluginBehavior;
use super::behavior_validation::NativePluginBehaviorValidationReport;
use super::bridge_method_abi::bridge_method_bindings_from_abi_v3;
use super::bridge_method_bindings::NativeBridgeMethodBinding;
use super::host_callbacks::{
    granted_capabilities_for_entry, native_host_abi_version_v3, native_host_diagnostic_v3,
    native_host_has_capability_v3, native_host_log_v3, register_native_host_callback_capture,
    take_native_host_callback_diagnostics,
};
use super::native_strings::{
    native_symbol_name, package_manifest_from_toml, parse_native_string_list,
    read_optional_c_string, read_required_c_string,
};
use super::plugin_load_error::{
    PluginLoadError, PluginLoadResult, PluginLoadStage, ABI_CONTRACT_HINT, DESCRIPTOR_EXPORT_HINT,
    ENTRY_EXPORT_HINT,
};

/// 探测后的宿主副本；决定入口符号与请求能力，并供后续清单注册消费。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePluginDescriptor {
    pub abi_version: u32,
    pub plugin_id: String,
    pub package_manifest: Option<PluginPackageManifest>,
    pub runtime_entry_name: Option<String>,
    pub editor_entry_name: Option<String>,
    pub requested_capabilities: Vec<String>,
}

/// 单个 runtime/editor 入口的结果；文本已复制，回调地址仍依赖动态库。
#[derive(Clone, Debug)]
pub struct NativePluginEntryReport {
    pub plugin_id: String,
    pub module_kind: PluginModuleKind,
    pub package_manifest: Option<PluginPackageManifest>,
    pub diagnostics: Vec<String>,
    pub negotiated_capabilities: Vec<String>,
    pub missing_required_capabilities: Vec<String>,
    pub denied_capabilities: Vec<String>,
    pub bridge_method_bindings: Vec<NativeBridgeMethodBinding>,
    pub editor_contribution_batch: Option<SerializedContributionBatch>,
    pub(super) behavior: Option<NativePluginBehavior>,
    pub behavior_validation: NativePluginBehaviorValidationReport,
}

type NativePluginDescriptorFnV3 = unsafe extern "C" fn() -> *const NativePluginAbiV3;
type NativePluginEntryFnV3 = unsafe extern "C" fn(
    *const NativePluginHostFunctionTableV3,
) -> *const NativePluginEntryReportV3;

/// 在动态库仍加载时探测固定 v3 导出，拒绝空指针与不一致的插件身份。
///
/// # Safety
/// 导出函数须遵守 v3 签名；返回对象对齐、布局正确且转换期间保持可读和不变，
/// 其中的 C 字符串在复制完成前有效。
pub(super) unsafe fn probe_native_plugin_descriptor(
    library: &Library,
    library_path: &Path,
    plugin_id: &str,
) -> PluginLoadResult<NativePluginDescriptor> {
    let expected_v3_symbol = String::from_utf8_lossy(
        ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL_V3
            .strip_suffix(&[0])
            .unwrap_or(ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL_V3),
    )
    .into_owned();
    let symbol = library
        .get::<NativePluginDescriptorFnV3>(ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL_V3)
        .map_err(|source| {
            PluginLoadError::missing_symbol(
                plugin_id,
                PluginLoadStage::DescriptorProbe,
                expected_v3_symbol,
                library_path,
                DESCRIPTOR_EXPORT_HINT,
                source,
            )
        })?;
    let descriptor = symbol();
    if descriptor.is_null() {
        return Err(PluginLoadError::null_pointer(
            plugin_id,
            PluginLoadStage::DescriptorProbe,
            "NativePluginAbiV3",
            library_path,
            DESCRIPTOR_EXPORT_HINT,
        ));
    }
    NativePluginDescriptor::from_abi_v3(&*descriptor, plugin_id, library_path)
}

/// 调用描述符选出的模块入口，并把能力协商、行为和桥表转换为诊断结果。
///
/// # Safety
/// 导出符号遵守 v3 签名；返回值至少有可读的 u32 epoch，匹配后提供对齐且
/// 可读的完整报告，所有外部字段在转换期间不变；
/// 插件不得在同步入口返回后保留宿主栈表指针。
pub(super) unsafe fn call_native_plugin_entry(
    library: &Library,
    library_path: &Path,
    symbol_name: &str,
    plugin_id: &str,
    module_kind: PluginModuleKind,
    descriptor: &NativePluginDescriptor,
) -> PluginLoadResult<NativePluginEntryReport> {
    let stage = PluginLoadStage::from(module_kind);
    if descriptor.abi_version != ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3 {
        return Err(PluginLoadError::contract_mismatch(
            plugin_id,
            stage,
            "descriptor.abi_version",
            ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3.to_string(),
            descriptor.abi_version.to_string(),
            library_path,
            ABI_CONTRACT_HINT,
        ));
    }
    let symbol_bytes = native_symbol_name(symbol_name);
    let symbol = library
        .get::<NativePluginEntryFnV3>(&symbol_bytes[..])
        .map_err(|source| {
            PluginLoadError::missing_symbol(
                plugin_id,
                stage,
                symbol_name,
                library_path,
                ENTRY_EXPORT_HINT,
                source,
            )
        })?;
    let granted_capabilities = granted_capabilities_for_entry(descriptor, module_kind);
    let granted_capabilities_abi =
        CString::new(granted_capabilities.join("\n")).map_err(|source| {
            PluginLoadError::invalid_payload(
                plugin_id,
                stage,
                "granted_capabilities",
                library_path,
                ABI_CONTRACT_HINT,
                source,
            )
        })?;
    let host_handle = register_native_host_callback_capture();
    let host_functions = NativePluginHostFunctionTableV3 {
        abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
        host_handle,
        granted_capabilities: granted_capabilities_abi.as_ptr(),
        host_abi_version: Some(native_host_abi_version_v3),
        host_has_capability: Some(native_host_has_capability_v3),
        host_log: Some(native_host_log_v3),
        host_diagnostic: Some(native_host_diagnostic_v3),
    };
    // SAFETY: 表、授权文本和捕获句柄仅在本次同步入口调用内有效；插件不可保存表指针。
    let report = symbol(&host_functions);
    let callback_diagnostics = take_native_host_callback_diagnostics(host_handle);
    if report.is_null() {
        return Err(PluginLoadError::null_pointer(
            plugin_id,
            stage,
            "NativePluginEntryReportV3",
            library_path,
            ENTRY_EXPORT_HINT,
        ));
    }
    // SAFETY: 非空报告按 ABI 至少提供一个可读 u32；检查 epoch 后才读取扩展布局。
    let layout_epoch = unsafe { report.cast::<u32>().read_unaligned() };
    if layout_epoch != ZIRCON_NATIVE_PLUGIN_ENTRY_REPORT_LAYOUT_EPOCH {
        return Err(PluginLoadError::contract_mismatch(
            plugin_id,
            stage,
            "entry_report.layout_epoch",
            ZIRCON_NATIVE_PLUGIN_ENTRY_REPORT_LAYOUT_EPOCH.to_string(),
            layout_epoch.to_string(),
            library_path,
            ABI_CONTRACT_HINT,
        ));
    }
    let mut report = NativePluginEntryReport::from_abi_v3(
        plugin_id,
        module_kind,
        library_path,
        &*report,
        &granted_capabilities,
    )?;
    report.diagnostics.extend(callback_diagnostics);
    if !report.missing_required_capabilities.is_empty() || !report.denied_capabilities.is_empty() {
        return Err(PluginLoadError::capability_negotiation(
            plugin_id,
            stage,
            report.missing_required_capabilities,
            report.denied_capabilities,
            report.diagnostics,
            library_path,
        ));
    }
    Ok(report)
}

impl NativePluginDescriptor {
    /// 复制仍在加载中的描述符数据。
    ///
    /// # Safety
    /// 非空字段指针须在本次转换期间可读、有效且以 NUL 结束。
    unsafe fn from_abi_v3(
        abi: &NativePluginAbiV3,
        expected_plugin_id: &str,
        library_path: &Path,
    ) -> PluginLoadResult<Self> {
        if abi.abi_version != ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3 {
            return Err(PluginLoadError::contract_mismatch(
                expected_plugin_id,
                PluginLoadStage::DescriptorProbe,
                "abi_version",
                ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3.to_string(),
                abi.abi_version.to_string(),
                library_path,
                ABI_CONTRACT_HINT,
            ));
        }
        let plugin_id = read_required_descriptor_field(
            abi.plugin_id,
            "plugin_id",
            expected_plugin_id,
            library_path,
        )?;
        if plugin_id != expected_plugin_id {
            return Err(PluginLoadError::contract_mismatch(
                expected_plugin_id,
                PluginLoadStage::DescriptorProbe,
                "plugin_id",
                expected_plugin_id,
                &plugin_id,
                library_path,
                ABI_CONTRACT_HINT,
            ));
        }
        Ok(Self {
            abi_version: abi.abi_version,
            plugin_id,
            // TODO: [CR-PLUGIN-NATIVE-0602] 确认自报清单是否必须与已准入候选的身份和能力一致；
            // 授权及桥注册读此副本，当前未见两个清单的完整字段对账。
            package_manifest: package_manifest_from_toml(
                &read_optional_c_string(abi.package_manifest_toml).unwrap_or_default(),
                "native plugin package manifest is invalid",
            )
            .map_err(|source| {
                PluginLoadError::invalid_payload(
                    expected_plugin_id,
                    PluginLoadStage::DescriptorProbe,
                    "package_manifest_toml",
                    library_path,
                    ABI_CONTRACT_HINT,
                    source,
                )
            })?,
            runtime_entry_name: read_optional_c_string(abi.runtime_entry_name),
            editor_entry_name: read_optional_c_string(abi.editor_entry_name),
            requested_capabilities: parse_native_string_list(
                &read_optional_c_string(abi.requested_capabilities).unwrap_or_default(),
            ),
        })
    }
}

unsafe fn read_required_descriptor_field(
    value: *const std::ffi::c_char,
    field_name: &'static str,
    plugin_id: &str,
    library_path: &Path,
) -> PluginLoadResult<String> {
    unsafe { read_required_c_string(value, field_name) }.map_err(|source| {
        PluginLoadError::invalid_payload(
            plugin_id,
            PluginLoadStage::DescriptorProbe,
            field_name,
            library_path,
            ABI_CONTRACT_HINT,
            source,
        )
    })
}

impl NativePluginEntryReport {
    /// 转换入口报告，并将行为、编辑器贡献和桥表错误纳入同一加载阶段。
    ///
    /// # Safety
    /// epoch 已核对；报告及行为、桥表均按对应布局对齐，非空字段在转换结束前有效。
    unsafe fn from_abi_v3(
        plugin_id: &str,
        module_kind: PluginModuleKind,
        library_path: &Path,
        abi: &NativePluginEntryReportV3,
        granted_capabilities: &[String],
    ) -> PluginLoadResult<Self> {
        let stage = PluginLoadStage::from(module_kind);
        let required_capabilities =
            unsafe { read_required_c_string(abi.required_capabilities, "required_capabilities") }
                .map_err(|source| {
                PluginLoadError::invalid_payload(
                    plugin_id,
                    stage,
                    "required_capabilities",
                    library_path,
                    ABI_CONTRACT_HINT,
                    source,
                )
            })?;
        let denied_capability_declarations =
            unsafe { read_required_c_string(abi.denied_capabilities, "denied_capabilities") }
                .map_err(|source| {
                    PluginLoadError::invalid_payload(
                        plugin_id,
                        stage,
                        "denied_capabilities",
                        library_path,
                        ABI_CONTRACT_HINT,
                        source,
                    )
                })?;
        let (missing_required_capabilities, denied_capabilities) = capability_negotiation_details(
            &parse_native_string_list(&required_capabilities),
            &parse_native_string_list(&denied_capability_declarations),
            granted_capabilities,
        );
        let diagnostics = unsafe { read_required_c_string(abi.diagnostics, "diagnostics") }
            .map_err(|source| {
                PluginLoadError::invalid_payload(
                    plugin_id,
                    stage,
                    "diagnostics",
                    library_path,
                    ABI_CONTRACT_HINT,
                    source,
                )
            })?;
        let behavior = if abi.behavior.is_null() {
            None
        } else {
            Some(
                NativePluginBehavior::from_abi_v4(&*abi.behavior).map_err(|source| {
                    PluginLoadError::invalid_payload(
                        plugin_id,
                        stage,
                        "behavior",
                        library_path,
                        ABI_CONTRACT_HINT,
                        source,
                    )
                })?,
            )
        };
        let behavior_validation = NativePluginBehaviorValidationReport::from_behavior(
            plugin_id,
            module_kind,
            ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
            behavior.as_ref(),
        );
        let editor_contribution_batch = editor_contribution_batch_from_behavior(
            plugin_id,
            module_kind,
            library_path,
            behavior.as_ref(),
        )?;
        Ok(Self {
            plugin_id: plugin_id.to_string(),
            module_kind,
            package_manifest: package_manifest_from_toml(
                &read_optional_c_string(abi.package_manifest_toml).unwrap_or_default(),
                "native plugin entry package manifest is invalid",
            )
            .map_err(|source| {
                PluginLoadError::invalid_payload(
                    plugin_id,
                    stage,
                    "package_manifest_toml",
                    library_path,
                    ABI_CONTRACT_HINT,
                    source,
                )
            })?,
            diagnostics: entry_diagnostics(&diagnostics),
            negotiated_capabilities: parse_native_string_list(
                &read_optional_c_string(abi.negotiated_capabilities).unwrap_or_default(),
            ),
            missing_required_capabilities,
            denied_capabilities,
            bridge_method_bindings: bridge_method_bindings_from_abi_v3(abi.bridge_methods)
                .map_err(|source| {
                    PluginLoadError::invalid_payload(
                        plugin_id,
                        stage,
                        "bridge_methods",
                        library_path,
                        ABI_CONTRACT_HINT,
                        source,
                    )
                })?,
            editor_contribution_batch,
            behavior_validation,
            behavior,
        })
    }
}

/// 只有 editor 入口且声明匹配 schema 时解码贡献；包 ID 必须等于当前插件。
fn editor_contribution_batch_from_behavior(
    plugin_id: &str,
    module_kind: PluginModuleKind,
    library_path: &Path,
    behavior: Option<&NativePluginBehavior>,
) -> PluginLoadResult<Option<SerializedContributionBatch>> {
    if module_kind != PluginModuleKind::Editor {
        return Ok(None);
    }
    let Some(behavior) = behavior else {
        return Ok(None);
    };
    if behavior.registration_manifest_schema.as_deref()
        != Some(SERIALIZED_EDITOR_CONTRIBUTION_BATCH_SCHEMA_V1)
    {
        return Ok(None);
    }
    let batch = serde_json::from_str::<SerializedContributionBatch>(
        behavior
            .registration_manifest
            .as_deref()
            .unwrap_or_default(),
    )
    .map_err(|source| {
        PluginLoadError::invalid_payload(
            plugin_id,
            PluginLoadStage::EditorEntry,
            "editor_contribution_batch",
            library_path,
            ABI_CONTRACT_HINT,
            source,
        )
    })?;
    if batch.package_id() != plugin_id {
        return Err(PluginLoadError::contract_mismatch(
            plugin_id,
            PluginLoadStage::EditorEntry,
            "editor_contribution_batch.package_id",
            plugin_id,
            batch.package_id(),
            library_path,
            ABI_CONTRACT_HINT,
        ));
    }
    Ok(Some(batch))
}

/// 对比插件必需或拒绝能力与宿主本次实际授权，为入口拒绝生成精确原因。
fn capability_negotiation_details(
    required_capabilities: &[String],
    denied_capabilities: &[String],
    granted_capabilities: &[String],
) -> (Vec<String>, Vec<String>) {
    let missing_required = required_capabilities
        .iter()
        .filter(|capability| !granted_capabilities.contains(capability))
        .cloned()
        .collect();
    let denied = denied_capabilities
        .iter()
        .filter(|capability| granted_capabilities.contains(capability))
        .cloned()
        .collect();
    (missing_required, denied)
}

fn entry_diagnostics(diagnostics: &str) -> Vec<String> {
    diagnostics
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
#[path = "tests/native_plugin_abi.rs"]
mod tests;
