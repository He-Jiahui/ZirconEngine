---
related_code:
  - zircon_plugins/plugin_sdk/src/native.rs
  - zircon_plugins/plugin_sdk/src/dist.rs
  - zircon_plugins/plugin_sdk/src/manifest/importer_runtime.rs
  - zircon_plugins/native_dynamic_fixture/native/src/lib.rs
  - zircon_plugins/editor_contribution_fixture/native/src/lib.rs
implementation_files:
  - zircon_plugins/plugin_sdk/src/native.rs
  - zircon_plugins/plugin_sdk/src/dist.rs
  - zircon_plugins/native_dynamic_fixture/native/src
  - zircon_plugins/editor_contribution_fixture/native/src
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/plans/zircon_plugins/01
tests:
  - zircon_plugins/native_dynamic_fixture
  - zircon_plugins/editor_contribution_fixture
  - zircon_runtime/src/plugin/native_plugin_loader
doc_type: abi-reference
title: 原生 ABI 与插件分发
status: source-audited
---

# 原生 ABI 与插件分发

原生动态插件通过 `dist` 或 `native` crate 产出 `cdylib`。宿主不依赖插件的 Rust ABI，而只读取 `#[repr(C)]` 数据、NUL 结尾字符串与 `extern "C"` 函数。当前 descriptor ABI 为 v3，entry report layout epoch 为 5，behavior ABI 为 v4；这些版本号属于不同层，不能混为一个版本。

## linked、native 与 dist

**linked** 模式直接链接 runtime/editor crate，使用 Rust trait 与 `RuntimeExtensionRegistry`。**native** 模式运行时装载动态库。**dist crate** 是把插件声明、manifest 和回调投影为 native 动态库的薄包装；业务逻辑通常仍由 runtime/editor crate 所有。`distribution.forms = ["dist"]` 与 `default_packaging = ["native_dynamic"]` 表示该包提供这种制品。

## 必需导出符号

动态库必须导出 `zircon_native_plugin_descriptor_v3`。`NativePluginAbiV3` 包含：

| 字段 | 含义 |
|---|---|
| `abi_version` | descriptor ABI，当前为 3 |
| `plugin_id` | canonical、NUL 结尾的插件 ID |
| `package_manifest_toml` | 完整包清单 |
| `runtime_entry_name` | runtime 入口符号，可为空 |
| `editor_entry_name` | editor 入口符号，可为空 |
| `requested_capabilities` | NUL 结尾、换行分隔的能力表 |

具体入口由包声明命名，例如 `zircon_plugin_ai_runtime_entry_v3`。入口签名接收 `NativePluginHostFunctionTableV3`，返回 `NativePluginEntryReportV3` 指针。

## 宿主函数表和入口报告

宿主表提供 ABI 版本、host handle、已授予能力，以及可选的版本查询、能力查询、日志和诊断函数。插件入口必须把宿主指针视为只在调用契约内有效，并处理空指针或不匹配版本。

入口报告包含 layout epoch、package manifest、diagnostics、negotiated/required/denied capabilities、behavior 指针和 bridge method table。能力不满足时 SDK 会返回 missing-host report，其 behavior 和 bridge table 为空；宿主应展示 diagnostics，而不是继续注册。

## Behavior v4

`NativePluginBehaviorV4` 描述插件的可调用行为：

- `is_stateless` 与 `state_schema_version`；
- command/event/registration manifest 的 schema ID；
- 三份 manifest 的 C 字符串；
- `invoke_command`、`save_state`、`restore_state`、`unload` 回调。

command manifest 当前 schema 为 `zircon.native.command-manifest/4`，registration manifest 为 `zircon.native.registration-manifest/3`。命令 slot 必须稠密且稳定；payload 是 borrowed byte slice，输出通过 `NativePluginOutputSinkV4` 写入。SDK 的 `callback_status` 返回 OK/ERROR/DENIED/PANIC 状态，`catch_native_callback_panic` 防止 Rust panic 穿越 C ABI。

## Bridge method

`NativePluginBridgeMethodTableV3` 是一组 `NativePluginBridgeMethodV3`：每项声明 interface ID、method name、函数指针和 `user_data`。调用时 `NativePluginBridgeMethodCallV3` 带 interface/method slot、payload、输出缓冲引用及 user data。ABI 层只运输字节；序列化 schema、版本升级和上限检查由接口契约负责。

## Registration manifest

`NativePluginRegistrationManifestV3` 可投影 modules、systems、resources、events 与 extensions。system 条目含 ID、owner module、stage、order、sets、access、thread affinity，以及用于执行的 bridge interface/method。worker-safe 系统还需要宿主授予 `runtime.native.system.worker_safe`，不能只通过 thread affinity 字段自我声明。

## dist 宏

SDK 提供三个一文件导出宏：

| 宏 | 用途 |
|---|---|
| `native_dist_plugin_v3!` | 同一库同时提供 runtime 和 editor 入口 |
| `native_dist_runtime_plugin_v3!` | 只有 runtime 入口 |
| `native_dist_editor_plugin_v3!` | 只有 editor 入口 |

宏接受 package manifest、descriptor ABI、入口名、请求能力、缺失宿主诊断，并分别配置 required/denied/negotiated capability、状态 schema、manifest、回调与 bridge methods。宏自动导出 descriptor 和 entry，不应再手写同名 `no_mangle` 符号。

```rust
zircon_plugin_sdk::native_dist_runtime_plugin_v3! {
    plugin_id: PLUGIN_ID_CSTR,
    package_manifest: PACKAGE_MANIFEST_TOML,
    descriptor_abi_version: zircon_plugin_sdk::native::ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
    runtime_entry: zircon_plugin_example_runtime_entry_v3,
    runtime_entry_name: RUNTIME_ENTRY_CSTR,
    requested_capabilities: REQUESTED_CAPABILITIES_CSTR,
    missing_host_diagnostics: MISSING_HOST_CSTR,
    runtime: {
        required_capabilities: ["runtime.plugin.example"],
        denied_capabilities: [],
        negotiated_capabilities: NEGOTIATED_CSTR,
        diagnostics: READY_CSTR,
        is_stateless: true,
        state_schema_version: 0,
        command_manifest_schema: None,
        event_manifest_schema: None,
        registration_manifest_schema: None,
        command_manifest: None,
        event_manifest: None,
        registration_manifest: None,
        invoke_command: None,
        save_state: None,
        restore_state: None,
        unload: None,
        bridge_methods: [],
        on_host_ready: None,
    },
}
```

上例字段形状直接来自 SDK 宏；实际第一方 dist 通常使用 `declare_plugin!` 生成的 C 字符串与 registration manifest 常量，避免手写漂移。

## 内存所有权

`NativePluginByteSliceV3` 是借用输入。需要把字节所有权交给宿主时使用 `owned_bytes(Vec<u8>)` 生成 `NativePluginOwnedByteBufferV3`，其释放函数会校验 owner token 和缓冲布局。输出 sink 有长度上限与 writer 校验。任何跨 ABI 字符串都必须 NUL 结尾且在加载期间保持稳定地址。

## 装载与热替换检查表

1. 校验 descriptor symbol、ABI v3 和插件 ID。
2. 解析 package manifest，校验引擎范围、目标、平台和依赖。
3. 构造 host table，只列出实际授予能力。
4. 调用 runtime/editor entry，校验 report epoch 与 diagnostics。
5. 校验 behavior schema 和 registration manifest 后再冻结计划。
6. 热替换时停止调用、保存状态、撤销 owner、调用 unload、卸载库，再加载新版本并恢复兼容状态。

`native_dynamic_fixture` 与 `editor_contribution_fixture/native` 是验证 ABI 和 editor contribution 的测试包，不属于生产功能插件。
