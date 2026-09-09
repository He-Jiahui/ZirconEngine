---
related_code:
  - zircon_plugins/plugin_sdk/src/registration.rs
  - zircon_plugins/plugin_sdk/src/native.rs
  - zircon_plugins/plugin_sdk/src/runtime.rs
  - zircon_runtime/src/plugin
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/first_party_editor_catalog/src/lib.rs
implementation_files:
  - zircon_plugins/plugin_sdk/src/registration.rs
  - zircon_plugins/plugin_sdk/src/native.rs
  - zircon_runtime/src/plugin
  - zircon_plugins/first_party_runtime_catalog/src
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/plans/zircon_plugins/01
tests:
  - zircon_runtime/src/plugin
  - zircon_plugins/first_party_runtime_catalog/src
  - zircon_plugins/native_dynamic_fixture
doc_type: lifecycle-reference
title: 插件生命周期与能力协商
status: source-audited
---

# 插件生命周期与能力协商

插件生命周期由“发现包、验证元数据、选择模块、协商能力、收集注册、冻结计划、激活模块、运行、撤销 owner”组成。linked 和 native 在注册报告之前走不同入口，但都会收敛到运行时扩展注册表与模块生命周期。

## 1. 发现与目录

第一方 runtime/editor catalog 提供内建包集合。外部包则由清单和 distribution 信息发现。候选包首先按 ID 去重，再校验 SDK/API、引擎兼容范围、目标模式和平台。`package_role = sample/test_fixture/developer_tool` 的包不应被生产目录无条件启用。

## 2. 选择目标与模块

宿主从 `client_runtime`、`server_runtime` 或 `editor_host` 选择目标。包内每个 `modules` 条目还有自己的 target modes：运行时包可以同时带 runtime、editor、native 模块，但 editor 模块不会进入客户端或服务端计划。feature bundle 只有在被选择或 `enabled_by_default` 时加入。

## 3. 能力协商

能力字符串是权限和功能可用性的共同标识。需要区分：

- **declared/provided**：包声明可提供的能力；
- **requested**：native descriptor 请求宿主授予的能力；
- **required**：入口报告要求必须存在，缺失则不能激活行为；
- **denied**：入口明确拒绝的组合；
- **negotiated/granted**：宿主与插件实际达成的集合；
- **status**：`complete/partial` 等实现进度，不等于权限。

native `NativePluginEntryPointV3::entry_report` 会先检查 host function table 和 ABI，再使用 `host_supports_all_capabilities_v3` / `host_supports_any_capability_v3` / `host_supports_capability_v3` 判断能力。缺少 required 或出现 denied 时返回不带 behavior 的报告，并附 diagnostics。插件不得仅凭 requested 列表假定能力已获授。

## 4. 生成注册报告

linked 插件实现 `RuntimePlugin`，通过 `plugin_registration()` 向 `RuntimeExtensionRegistry` 注册模块、系统、组件、资源、事件、选项、feature 和 bridge 接口。native 插件则提供 TOML registration manifest、command/event manifest 和 bridge method table，由宿主解析为同类扩展计划。

注册项都绑定到 `PluginModuleId owner`。这是资源归属、重复检查与卸载撤销的基础。

## 5. 冻结和激活

registry 完成收集后被 finalize。此后系统排序解析 `before/after`、set、stage、order 与 tick policy；模块图解析 init level 和 module dependencies。manager/driver 使用模块描述符激活，场景系统工厂则为每个 scene-system 实例产生独立回调。注册成功不代表服务已立即启动，lazy service 会在第一次解析时构造。

## 6. 运行时交互

插件通过以下稳定通道与宿主/其他插件通信：

| 通道 | linked | native |
|---|---|---|
| Scene system | Rust 回调工厂 | registration manifest + bridge method |
| 资源/组件 | 强类型注册 | schema/registration projection |
| 事件 | `Event` + event manifest | event manifest 与序列化 payload |
| 命令 | 通常直接 API/bridge | command slot + byte payload/output sink |
| 跨插件接口 | `PluginInterface` / `BridgeImport` | interface ID + method name + ABI call |

能力 ID 与接口 ID 不可互换。能力说明“是否允许/是否具备”；接口 ID 说明“如何调用”。接口应带版本后缀，例如 `physics.query.v1`。

## 7. 状态保存与热卸载

native behavior v4 可声明 `is_stateless`、`state_schema_version`、`save_state`、`restore_state` 与 `unload`。有状态插件在热替换前应保存 opaque bytes，新版本只有在理解相同 schema 时恢复。命令输出经 `NativePluginOutputSinkV4` 写回，并受 256 MiB 上限保护。

卸载顺序应先阻止新调用，再等待/取消插件拥有的工作，撤销 owner 注册，令 `BridgeImport` 失效，最后调用 native `unload` 和释放库。`owner_revocation_listener` 用于清理导入缓存或外部句柄。插件自己的线程、GPU 工作和 FFI 回调必须在库卸载前完成，否则函数指针会悬空。

## capability status 与成熟度

`PluginMaturityLevel` 是包级发布标签；`CapabilityStatusManifest` 是能力级事实。例如 `ai` 为 experimental，感知能力可标 complete，而行为树/黑板仍 partial。`rendering` 为 stable 且主 capability complete，但其可选高级 feature 各自仍可能是实验实现。Wiki 页面因此始终分别列出包成熟度和能力实现状态。

## 故障诊断

启动失败时按以下顺序检查：包 ID 是否 canonical；目标/平台是否匹配；dependency 与 required capability 是否存在；feature 是否选中；native descriptor ABI 和 symbol 是否匹配；entry report diagnostics；registration manifest schema；系统约束是否形成环；模块依赖是否缺失；bridge interface 版本是否一致。不要把“动态库成功加载”当作“插件成功激活”。
