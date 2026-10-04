//! 原生动态库包装主包的静态 ABI 元数据；可选渲染特性的实现仍由宿主的独立注册链提供。
use zircon_plugin_rendering_runtime::{
    NATIVE_PLUGIN_ID, NATIVE_REQUESTED_CAPABILITIES, NATIVE_RUNTIME_ENTRY,
    NATIVE_RUNTIME_REGISTRATION_MANIFEST,
};
use zircon_plugin_sdk::native::{self, ZIRCON_NATIVE_PLUGIN_ABI_VERSION};

// 编译期清单及终止符随动态库存活；宿主只能在该库仍加载时借用宏导出的静态指针。
const PLUGIN_MANIFEST: &str = concat!(include_str!("../../plugin.toml"), "\0");

const RUNTIME_DIAGNOSTICS: &[u8] =
    b"rendering dist entry ready; rendering feature ownership remains hosted by the runtime module\0";
const MISSING_HOST_DIAGNOSTICS: &[u8] =
    b"rendering dist entry requires runtime.plugin.rendering host capability\0";
const EMPTY_MANIFEST: &[u8] = b"\0";

// 先协商宿主主包能力再报告无状态行为；没有命令或卸载回调，也没有特性 executor 的原生注册。
zircon_plugin_sdk::native_dist_runtime_plugin_v3! {
    plugin_id: NATIVE_PLUGIN_ID,
    package_manifest: PLUGIN_MANIFEST,
    descriptor_abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
    runtime_entry: zircon_plugin_rendering_runtime_entry_v3,
    runtime_entry_name: NATIVE_RUNTIME_ENTRY.cstr(),
    requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
    missing_host_diagnostics: MISSING_HOST_DIAGNOSTICS,
    runtime: {
        required_capabilities: ["runtime.plugin.rendering"],
        denied_capabilities: [],
        negotiated_capabilities: NATIVE_REQUESTED_CAPABILITIES,
        diagnostics: RUNTIME_DIAGNOSTICS,
        is_stateless: true,
        state_schema_version: 0,
        command_manifest_schema: None,
        event_manifest_schema: None,
        registration_manifest_schema: Some(native::NATIVE_REGISTRATION_MANIFEST_SCHEMA_V3),
        command_manifest: Some(EMPTY_MANIFEST),
        event_manifest: Some(EMPTY_MANIFEST),
        registration_manifest: Some(NATIVE_RUNTIME_REGISTRATION_MANIFEST),
        invoke_command: None,
        save_state: None,
        restore_state: None,
        unload: None,
        bridge_methods: [],
        on_host_ready: None,
    },
}

// ABI 测试使用同一库内的静态返回值和有效宿主表，确认符号与报告连接；不验证 GPU 效果。
#[cfg(test)]
#[path = "tests/lib.rs"]
mod tests;
