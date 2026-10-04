//! 定义 net 插件身份、可装载目标及 native v3 入口，供 SDK 声明和包分发链消费。
//! 能力是准入与注册契约，具体传输服务由运行时模块的 manager/provider 提供。

zircon_plugin_sdk::declare_plugin! {
    pub NET_DECLARATION {
        id: PLUGIN_ID = "net",
        display_name: "Network",
        category: runtime,
        module: MODULE_NAME = "net.runtime",
        crate_name: RUNTIME_CRATE_NAME = "zircon_plugin_net_runtime",
        module_description: "Network transport and runtime service integration",
        targets: [server_runtime, client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            NET_RUNTIME_CAPABILITY = "runtime.plugin.net" => runtime_registration,
        ],
        maturity: beta,
        packaging: [source_template, library_embed, native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            runtime: {
                entry: NATIVE_RUNTIME_ENTRY = "zircon_plugin_net_runtime_entry_v3",
                registration_manifest: NATIVE_RUNTIME_REGISTRATION_MANIFEST,
                modules: [{ name: "runtime", kind: "runtime" }],
                systems: [],
                events: [],
                extensions: [
                    { point: "runtime.net.transport", contribution: "plugin.net.runtime", schema: "zircon.runtime.net-transport/1" },
                ],
            },
        },
    }
}

pub const RUNTIME_CAPABILITIES: &[&str] = &[NET_RUNTIME_CAPABILITY];
