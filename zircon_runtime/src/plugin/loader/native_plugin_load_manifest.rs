use serde::{Deserialize, Serialize};

/// 导出目录的显式插件选择清单；发现 authority 按该集合建立候选，仍需独立校验包与加载授权。
/// 它不包含动态库句柄，不因反序列化成功就允许执行插件。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativePluginLoadManifest {
    #[serde(default)]
    pub plugins: Vec<NativePluginLoadManifestEntry>,
}

/// 单插件的导出布局提示；发现调用端将相对路径约束在导出根内，再读取实际 package manifest。
/// package_report 与 abi 携带发行阶段的协议证据，不能替代实际产物的 trust admission。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativePluginLoadManifestEntry {
    pub id: String,
    pub path: String,
    pub manifest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_report: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub abi: Option<NativePluginLoadManifestAbiV3Contract>,
}

// TODO: [CR-PLUGIN-NATIVE-0003] 确认生产发现链是否应校验 plugins.abi；当前导出工具写入且测试核对字段，发现端只反序列化未读取；下一步检查加载前的权威协议门禁。
/// 导出工具写入并随清单携带的 ABI 协议标识，工具报告和测试会核对它；不是可调用函数表。
/// 实际入口名称、版本和函数指针仍由载入后的描述符及入口报告解码。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativePluginLoadManifestAbiV3Contract {
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
