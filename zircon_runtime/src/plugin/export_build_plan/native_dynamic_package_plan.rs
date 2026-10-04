//! 从选定 NativeDynamic 包生成目录、加载清单位置及 ABI V3 元数据；后续库存和归档按该身份消费。
use std::collections::HashSet;
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

pub(super) const NATIVE_DYNAMIC_PACKAGE_REPORT_FILE: &str = "native_dynamic_package.toml";

const NATIVE_DYNAMIC_PACKAGE_REPORT_FORMAT_VERSION: u32 = 1;
const NATIVE_DYNAMIC_ABI_VERSION_V3: u32 = 3;
const NATIVE_DYNAMIC_DESCRIPTOR_SYMBOL_V3: &str = "zircon_native_plugin_descriptor_v3";
const NATIVE_DYNAMIC_DESCRIPTOR_CONTRACT_V3: &str = "NativePluginAbiV3";
const NATIVE_DYNAMIC_RUNTIME_ENTRY_SOURCE_V3: &str = "NativePluginAbiV3.runtime_entry_name";
const NATIVE_DYNAMIC_EDITOR_ENTRY_SOURCE_V3: &str = "NativePluginAbiV3.editor_entry_name";
const NATIVE_DYNAMIC_HOST_FUNCTION_TABLE_V3: &str = "NativePluginHostFunctionTableV3";
const NATIVE_DYNAMIC_ENTRY_REPORT_CONTRACT_V3: &str = "NativePluginEntryReportV3";
const NATIVE_DYNAMIC_BEHAVIOR_CONTRACT_V4: &str = "NativePluginBehaviorV4";
const NATIVE_DYNAMIC_STATE_SNAPSHOT_CONTRACT_V4: &str =
    "NativePluginBehaviorV4.save_state/restore_state";
const NATIVE_DYNAMIC_BRIDGE_METHOD_TABLE_V3: &str = "NativePluginBridgeMethodTableV3";

pub(super) struct NativeDynamicPackagePlan {
    pub(super) packages: Vec<String>,
    pub(super) package_exports: Vec<NativeDynamicPackageExportPlan>,
    pub(super) diagnostics: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 单个包在导出产物内的路径与 ABI 契约；加载清单、复制和归档必须引用相同包身份。
pub struct NativeDynamicPackageExportPlan {
    pub package_id: String,
    pub directory: String,
    pub path: String,
    pub manifest: String,
    pub abi: NativeDynamicPackageAbiV3Contract,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 导出报告声明的宿主/插件 ABI 表；实际二进制仍须通过原生 authority 与加载器校验。
pub struct NativeDynamicPackageAbiV3Contract {
    pub abi_version: u32,
    pub descriptor_symbol: String,
    pub descriptor_contract: String,
    pub runtime_entry_source: String,
    pub editor_entry_source: String,
    pub host_function_table: String,
    pub entry_report_contract: String,
    pub behavior_contract: String,
    pub state_snapshot_contract: String,
    pub bridge_method_table: String,
}

#[derive(Default)]
pub(super) struct NativeDynamicPackageAccumulator {
    package_ids: HashSet<String>,
    package_directories: HashSet<String>,
    packages: Vec<String>,
    package_exports: Vec<NativeDynamicPackageExportPlan>,
    diagnostics: Vec<String>,
}

impl NativeDynamicPackageAccumulator {
    /// 同一 provider 可能经插件和特征两路被选中；按包 ID 首次出现保留一个导出条目。
    pub(super) fn push(&mut self, package_id: &str) {
        if !self.package_ids.insert(package_id.to_string()) {
            return;
        }
        let package_export = NativeDynamicPackageExportPlan::for_package_id(package_id);
        let package_directory = package_export.directory.clone();
        if !self.package_directories.insert(package_directory.clone()) {
            self.diagnostics.push(format!(
                "plugin {package_id} uses NativeDynamic packaging but resolves to duplicate output directory plugins/{package_directory}"
            ));
            return;
        }
        self.packages.push(package_id.to_string());
        self.package_exports.push(package_export);
    }

    pub(super) fn finish(self) -> NativeDynamicPackagePlan {
        NativeDynamicPackagePlan {
            packages: self.packages,
            package_exports: self.package_exports,
            diagnostics: self.diagnostics,
        }
    }
}

impl NativeDynamicPackageExportPlan {
    /// 包目录及 ABI 元数据必须与加载清单、包报告和 materialize 的输出位置保持一致。
    pub fn for_package_id(package_id: impl Into<String>) -> Self {
        let package_id = package_id.into();
        let directory = native_dynamic_package_directory(&package_id);
        let path = format!("plugins/{directory}");
        let manifest = format!("{path}/plugin.toml");
        Self {
            package_id,
            directory,
            path,
            manifest,
            abi: NativeDynamicPackageAbiV3Contract::native_abi_v3(),
        }
    }
}

impl NativeDynamicPackageAbiV3Contract {
    /// 为加载清单和每包报告提供同一套契约名称；变更应同步实际 native ABI 消费者。
    pub fn native_abi_v3() -> Self {
        Self {
            abi_version: NATIVE_DYNAMIC_ABI_VERSION_V3,
            descriptor_symbol: NATIVE_DYNAMIC_DESCRIPTOR_SYMBOL_V3.to_string(),
            descriptor_contract: NATIVE_DYNAMIC_DESCRIPTOR_CONTRACT_V3.to_string(),
            runtime_entry_source: NATIVE_DYNAMIC_RUNTIME_ENTRY_SOURCE_V3.to_string(),
            editor_entry_source: NATIVE_DYNAMIC_EDITOR_ENTRY_SOURCE_V3.to_string(),
            host_function_table: NATIVE_DYNAMIC_HOST_FUNCTION_TABLE_V3.to_string(),
            entry_report_contract: NATIVE_DYNAMIC_ENTRY_REPORT_CONTRACT_V3.to_string(),
            behavior_contract: NATIVE_DYNAMIC_BEHAVIOR_CONTRACT_V4.to_string(),
            state_snapshot_contract: NATIVE_DYNAMIC_STATE_SNAPSHOT_CONTRACT_V4.to_string(),
            bridge_method_table: NATIVE_DYNAMIC_BRIDGE_METHOD_TABLE_V3.to_string(),
        }
    }
}

impl Default for NativeDynamicPackageAbiV3Contract {
    fn default() -> Self {
        Self::native_abi_v3()
    }
}

pub(super) fn native_dynamic_package_report_template(
    package: &NativeDynamicPackageExportPlan,
) -> String {
    let mut output = String::from("# Generated by Zircon export. Native dynamic package report.\n");
    writeln!(
        output,
        "format_version = {NATIVE_DYNAMIC_PACKAGE_REPORT_FORMAT_VERSION}\npackage_id = {:?}\ndirectory = {:?}\npath = {:?}\nmanifest = {:?}",
        package.package_id, package.directory, package.path, package.manifest
    )
    .expect("writing native package report to String cannot fail");
    append_native_dynamic_abi_contract_toml(&mut output, "abi", &package.abi);
    output
}

pub(super) fn append_native_dynamic_abi_contract_toml(
    output: &mut String,
    table_name: &str,
    abi: &NativeDynamicPackageAbiV3Contract,
) {
    writeln!(
        output,
        "\n[{table_name}]\nabi_version = {}\ndescriptor_symbol = {:?}\ndescriptor_contract = {:?}\nruntime_entry_source = {:?}\neditor_entry_source = {:?}\nhost_function_table = {:?}\nentry_report_contract = {:?}\nbehavior_contract = {:?}\nstate_snapshot_contract = {:?}\nbridge_method_table = {:?}",
        abi.abi_version,
        abi.descriptor_symbol,
        abi.descriptor_contract,
        abi.runtime_entry_source,
        abi.editor_entry_source,
        abi.host_function_table,
        abi.entry_report_contract,
        abi.behavior_contract,
        abi.state_snapshot_contract,
        abi.bridge_method_table
    )
    .expect("writing native ABI report to String cannot fail");
}

pub(super) fn native_dynamic_package_directory(package_id: &str) -> String {
    let sanitized: String = package_id
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect();
    if sanitized.is_empty() {
        "_".to_string()
    } else {
        sanitized
    }
}

#[cfg(test)]
#[path = "tests/native_dynamic_package_plan.rs"]
mod tests;
