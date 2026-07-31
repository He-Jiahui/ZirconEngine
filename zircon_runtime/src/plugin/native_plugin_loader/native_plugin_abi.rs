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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePluginDescriptor {
    pub abi_version: u32,
    pub plugin_id: String,
    pub package_manifest: Option<PluginPackageManifest>,
    pub runtime_entry_name: Option<String>,
    pub editor_entry_name: Option<String>,
    pub requested_capabilities: Vec<String>,
}

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

pub(super) unsafe fn probe_native_plugin_descriptor(
    library: &Library,
    library_path: &Path,
    plugin_id: &str,
) -> PluginLoadResult<NativePluginDescriptor> {
    let expected_symbol = String::from_utf8_lossy(
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
                expected_symbol,
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
mod tests {
    use super::*;

    fn editor_contribution_behavior(payload: &str) -> NativePluginBehavior {
        NativePluginBehavior {
            is_stateless: true,
            state_schema_version: 0,
            command_manifest_schema: None,
            event_manifest_schema: None,
            registration_manifest_schema: Some(
                SERIALIZED_EDITOR_CONTRIBUTION_BATCH_SCHEMA_V1.to_string(),
            ),
            command_manifest: None,
            event_manifest: None,
            registration_manifest: Some(payload.to_string()),
            command_table: None,
            invoke_command: None,
            save_state: None,
            restore_state: None,
            unload: None,
        }
    }

    #[test]
    fn native_entry_payload_error_preserves_granted_capability_source() {
        let source = CString::new("native\0capability")
            .expect_err("interior NUL should be rejected by CString");
        let error = PluginLoadError::invalid_payload(
            "fixture",
            PluginLoadStage::RuntimeEntry,
            "granted_capabilities",
            Path::new("fixture.dll"),
            ABI_CONTRACT_HINT,
            source,
        );

        assert!(std::error::Error::source(&error).is_some());
    }

    #[test]
    fn native_entry_contract_error_preserves_expected_and_actual_versions() {
        let error = PluginLoadError::contract_mismatch(
            "fixture",
            PluginLoadStage::RuntimeEntry,
            "entry_report.layout_epoch",
            ZIRCON_NATIVE_PLUGIN_ENTRY_REPORT_LAYOUT_EPOCH.to_string(),
            "3",
            Path::new("fixture.dll"),
            ABI_CONTRACT_HINT,
        );

        let message = error.to_string();
        assert!(message.contains(&format!(
            "expected {}, actual 3",
            ZIRCON_NATIVE_PLUGIN_ENTRY_REPORT_LAYOUT_EPOCH
        )));
        assert!(std::error::Error::source(&error).is_none());
    }

    #[test]
    fn capability_negotiation_reports_missing_required_and_granted_denied_details() {
        let required = vec![
            "runtime.required".to_string(),
            "runtime.available".to_string(),
        ];
        let denied = vec!["runtime.denied".to_string(), "runtime.absent".to_string()];
        let granted = vec![
            "runtime.available".to_string(),
            "runtime.denied".to_string(),
        ];

        let (missing_required, denied) =
            capability_negotiation_details(&required, &denied, &granted);

        assert_eq!(missing_required, vec!["runtime.required"]);
        assert_eq!(denied, vec!["runtime.denied"]);
    }

    #[test]
    fn editor_contribution_batch_decodes_valid_editor_payload() {
        let behavior = editor_contribution_behavior(
            r#"{
                "package_id": "fixture.editor",
                "contributions": [{
                    "kind": "view",
                    "id": "fixture.editor.view",
                    "schema": "zircon.editor.view/1",
                    "title": "Fixture",
                    "category": "Tests"
                }]
            }"#,
        );

        let batch = editor_contribution_batch_from_behavior(
            "fixture.editor",
            PluginModuleKind::Editor,
            Path::new("fixture.dll"),
            Some(&behavior),
        )
        .expect("valid editor contribution payload should decode")
        .expect("editor contribution schema should produce a batch");

        assert_eq!(batch.package_id(), "fixture.editor");
        assert_eq!(
            batch.contributions()[0].key(),
            ("view", "fixture.editor.view")
        );
    }

    #[test]
    fn editor_contribution_batch_rejects_package_mismatch() {
        let behavior = editor_contribution_behavior(
            r#"{
                "package_id": "foreign.plugin",
                "contributions": []
            }"#,
        );

        let error = editor_contribution_batch_from_behavior(
            "fixture.editor",
            PluginModuleKind::Editor,
            Path::new("fixture.dll"),
            Some(&behavior),
        )
        .expect_err("foreign package payload must be rejected");

        assert!(error
            .to_string()
            .contains("editor_contribution_batch.package_id"));
    }

    #[test]
    fn editor_contribution_batch_is_ignored_for_non_editor_entries() {
        let behavior = editor_contribution_behavior("not JSON");

        let batch = editor_contribution_batch_from_behavior(
            "fixture.runtime",
            PluginModuleKind::Runtime,
            Path::new("fixture.dll"),
            Some(&behavior),
        )
        .expect("runtime entries must not parse an editor-only payload");

        assert!(batch.is_none());
    }
}
