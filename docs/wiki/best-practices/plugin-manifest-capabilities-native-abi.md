---
related_code:
  - zircon_plugins/plugin_sdk/src/native.rs
  - zircon_plugins/plugin_sdk/src/dist.rs
  - zircon_plugins/plugin_sdk/src/manifest/importer_runtime.rs
  - zircon_runtime/src/plugin/native_plugin_loader
implementation_files:
  - zircon_plugins/plugin_sdk/src/native.rs
  - zircon_plugins/plugin_sdk/src/dist.rs
  - zircon_runtime/src/plugin/native_plugin_loader
plan_sources:
  - user: 2026-09-09 补充 ZirconEngine 最佳实践、方案示例与详细 Wiki
tests:
  - zircon_plugins/native_dynamic_fixture
  - zircon_plugins/editor_contribution_fixture
  - zircon_runtime/src/plugin/native_plugin_loader
doc_type: workflow-detail
---

# 插件清单、能力与原生 ABI 打包实践

插件先是一个可验证的 package manifest，然后才是 runtime/editor 贡献或动态库。第一方 native 路径以 `#[repr(C)]` 数据、NUL 结尾字符串和 `extern "C"` 导出跨边界，不能把 Rust trait object、`String`、`Vec` 或 panic 当成 ABI。权威类型在 [native.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_plugins/plugin_sdk/src/native.rs) 与 [dist.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_plugins/plugin_sdk/src/dist.rs)，加载端在 [native_plugin_loader](https://github.com/He-Jiahui/ZirconEngine/tree/main/zircon_runtime/src/plugin/native_plugin_loader)。

## 选择交付形态

| 目标 | 选项 | 可依赖的接口 | 不适用时机 |
| --- | --- | --- | --- |
| 与宿主同版本编译、进程内扩展 | `linked` | Rust trait 与 `RuntimeExtensionRegistry` | 需要独立发包、运行时替换或二进制隔离 |
| 运行时装载动态库 | `native` | descriptor、host function table、entry report | 只想为同一 workspace 增加模块 |
| 将清单和回调投影为动态库制品 | `dist` crate + native packaging | SDK 宏生成的 C ABI 导出 | 不需要 native 分发 |

不要把 `distribution.forms = ["dist"]` 理解为“业务逻辑都在 dist crate”。现有 SDK 模型中，dist crate 是声明、manifest 和回调的投影层；runtime/editor crate 仍是业务代码的自然所有者。详情见 [原生 ABI 与插件分发](../plugins/native-abi-and-distribution.md) 与 [Catalog 与 manifest](../plugins/catalogs-and-manifests.md)。

## 把能力协商当作准入门槛

```text
package manifest
      |  plugin id / target / dependencies / requested capabilities
      v
host table -- actually granted capabilities --> native entry
      |                                      |
      |<--- entry report: diagnostics + negotiated/denied --->|
      v
validate ABI, epoch, schemas, registration manifest
      |
      +--> register behavior and contributions
      +--> reject and display diagnostics
```

能力字段不是插件自行宣称后就自动拥有的权限。host table 仅列出实际授予能力；若 required capability 不满足，SDK 会形成 missing-host report，此时 behavior 和 bridge table 为空。加载器必须展示 diagnostics 并停止注册，而不是继续调用回调。这与 [`NativePluginHostFunctionTableV3` 及 report 类型](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_plugins/plugin_sdk/src/native.rs) 的职责一致。

| 声明内容 | 写入位置 | 加载时必须检查 | 常见错误 |
| --- | --- | --- | --- |
| plugin ID、目标、引擎兼容、依赖 | package manifest | canonical ID、目标平台、依赖与版本范围 | 用展示名称代替稳定 ID |
| requested/required/denied/negotiated capability | manifest 与 entry report | host 授予集、report diagnostics | 只看 requested，跳过 denied |
| module/system/resource/event contribution | registration manifest | schema、owner、访问域、线程亲和性 | unload 后留下注册项 |
| native binary 与 descriptor | 分发制品 | 符号、ABI version、layout epoch | 把不同层版本号当成一个版本 |

## 使用 SDK 宏，不手写同名导出

**契约形状（不可直接复制）：**

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
        negotiated_capabilities: REQUESTED_CAPABILITIES_CSTR,
        diagnostics: RUNTIME_DIAGNOSTICS,
        is_stateless: true,
        state_schema_version: 0,
        command_manifest_schema: None,
        event_manifest_schema: None,
        registration_manifest_schema: Some(zircon_plugin_sdk::native::NATIVE_REGISTRATION_MANIFEST_SCHEMA_V3),
        command_manifest: Some(EMPTY_MANIFEST),
        event_manifest: Some(EMPTY_MANIFEST),
        registration_manifest: Some(REGISTRATION_MANIFEST_CSTR),
        invoke_command: None,
        save_state: None,
        restore_state: None,
        unload: None,
        bridge_methods: [],
        on_host_ready: None,
    },
}
```

上面是字段契约形状：`runtime` 内的每个字段都是宏匹配项，不能省略或用注释占位；有 editor entry 的插件还必须提供对应的 `editor` 块。可编译的完整实例见 [asset_importer.model dist fixture](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_plugins/asset_importers/model/dist/src/lib.rs)。宏负责导出 descriptor 与 entry。生产包应从 `declare_plugin!` 及 manifest 常量派生 C 字符串，避免 package manifest、symbol 和 capability 列表分别维护。宏和 ABI 常量定义见 [dist.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_plugins/plugin_sdk/src/dist.rs)。

**反模式，示意伪代码：**

```rust
// 错误：宏已导出 descriptor，又手写相同 no_mangle symbol；并让 panic 穿越 ABI。
#[no_mangle]
extern "C" fn zircon_native_plugin_descriptor_v3() { panic!("bad") }
```

同名导出会制造符号冲突或不一致 descriptor；panic 穿越 C ABI 是未定义的边界行为。现有 SDK 以 callback status 和 `catch_native_callback_panic` 将 panic 转为 ABI 可报告状态。

## 内存、状态与热替换

| 跨边界数据 | 正确所有权 | 约束 |
| --- | --- | --- |
| 输入 payload | `NativePluginByteSliceV3` 借用 | 只在回调调用期读取，不保存指针 |
| 需要交给宿主的字节 | `owned_bytes(Vec<u8>)` 产生 owned buffer | 使用 SDK 携带的释放函数与 owner token |
| C 字符串 | 静态且 NUL 结尾 | 在 descriptor/entry 被读取期间地址稳定 |
| 可恢复插件状态 | `save_state` / `restore_state` 回调 | 受 schema version 与兼容策略约束 |

热替换顺序应固定为：停止新调用、保存兼容状态、撤销 owner contributions、调用 `unload`、卸载库、加载新库并恢复状态。不要先 unload 再撤销贡献，也不要在 report 表示 capability denied 后尝试 bridge 调用。`NativePluginBehaviorV4` 的 state、unload 和 bridge 责任可从 [native.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_plugins/plugin_sdk/src/native.rs) 追溯。

## 失败诊断与发布核对

| 症状 | 首先检查 | 修复 |
| --- | --- | --- |
| 找不到 descriptor | 动态库导出表与 `zircon_native_plugin_descriptor_v3` | 使用 SDK 宏，检查产物是否为目标平台动态库 |
| ABI/epoch 不匹配 | descriptor ABI、entry report layout epoch、behavior ABI 各自的版本 | 分层比较，重建与宿主兼容的制品 |
| 插件已加载却没有贡献 | entry report diagnostics 与 capability granted/denied | 满足 required capability 后重新协商；不要强行注册 |
| 热替换后菜单或系统重复 | owner retirement 与 registration manifest | 先撤销同一 owner 的贡献，再注册新实例 |
| 运行时崩溃或乱码 | borrowed slice、NUL 终止和释放回调 | 不保留 borrowed 指针；只用 SDK owned buffer |

- [ ] package manifest 的 ID、目标、依赖、能力和分发形态经过验证。
- [ ] 动态库使用 SDK 宏导出，未重复手写 descriptor/entry symbol。
- [ ] loader 在注册前验证 descriptor、report、schema 与 capability 协商结果。
- [ ] 每个跨 ABI buffer 与字符串都有明确的借用期或释放所有者。
- [ ] native fixture 与 loader 测试覆盖至少一个成功路径和一个 capability/ABI 拒绝路径。
