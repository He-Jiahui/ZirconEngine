---
related_code:
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/prelude.rs
  - zircon_app/src/lib.rs
  - zircon_editor/src/lib.rs
  - zircon_runtime_interface/src/lib.rs
  - zircon_plugins/plugin_sdk/src/lib.rs
implementation_files:
  - zircon_runtime/src/lib.rs
  - zircon_app/src/lib.rs
  - zircon_editor/src/lib.rs
  - zircon_runtime_interface/src/lib.rs
  - zircon_plugins/plugin_sdk/src/lib.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/engine-architecture/runtime-interface-convergence.md
  - docs/engine-architecture/native-plugin-boundary.md
tests:
  - zircon_runtime/src/tests
  - zircon_runtime_interface/src/tests
  - zircon_editor/src/tests
  - zircon_plugins/plugin_sdk/src
doc_type: module-detail
---

# Rust API 约定

本页是各专题 API 页的共同参考。完整符号清单见[Rust API 索引](api-index.md)。

## 入口优先级

1. 使用 `zircon_runtime`、`zircon_app`、`zircon_editor` crate root 的 curated re-export。
2. 需要跨动态库时只使用 `zircon_runtime_interface` 的 `#[repr(C)]` 和序列化 DTO。
3. 只有实现同一 crate 内模块时才直接引用深层路径；`pub(crate)` 不属于外部合同。
4. 插件使用 `zircon_plugin_sdk` 和生成的 descriptor/entry，而不是链接 runtime 内部实现。

## 错误与所有权

运行时 API 通常返回 `Result<T, E>`，其中 `E` 是域错误树（如 `CoreError`、`ResourceError`、`GatewayError`）。错误应携带阶段、期望/实际、资源或 session 身份；调用方不能用 `unwrap` 把可恢复的加载/ABI/平台错误变成进程崩溃。

资源、服务和动态结果以 handle、lease 或 owned buffer 表达所有权：

```rust
let handle = runtime.resolve_manager_handle::<MyManager>("Foundation.Config")?;
let manager = handle.enter()?;
// `manager` 是带 in-flight admission 的 ServiceCallGuard，离开作用域即释放调用资格。
```

上例只展示语义；实际服务名和类型必须来自对应模块的公开常量与 resolver。不要缓存跨生命周期的 `Arc` 或裸引用来替代 handle identity。

## Feature 门控

文档代码必须标注所需 feature：

```toml
zircon_runtime = { path = "../zircon_runtime", default-features = false, features = ["core-min", "graphics"] }
```

`graphics` 会带入 RHI/WGPU 和 text 依赖；`ui` 依赖 graphics/taffy；`dynamic-api` 会启用 animation、diagnostic、graphics、navigation、script 和 ui。不要在 `target-server` 中假设窗口、winit 或 WGPU surface 存在。

## ABI 安全

动态函数表字段顺序冻结，版本演进使用新版本结构并配合 `size_bytes`/`abi_version` 校验。跨界值应是 `#[repr(C)]` 标量、句柄或 `ZrByteSlice`/`ZrOwnedByteBuffer`；字符串和复杂对象通过受限 UTF-8/JSON/bincode 载荷传输。宿主必须调用对应 free/release 函数，不得跨 allocator 释放指针。

## Rustdoc 示例标准

每个专题页的示例应：

- 使用真实存在的类型和函数名；
- 展示最小错误处理和生命周期验证；
- 说明同步/异步、feature、线程和平台前提；
- 在代码块下方链接 owner source/test；
- 不把测试-only helper 或规划 API 写成生产调用。

## 常用入口速查

| 任务 | 入口 |
| --- | --- |
| 创建核心运行时 | `zircon_runtime::core::CoreRuntime::try_new` |
| 注册/激活模块 | `CoreRuntime::register_module`、`activate_registered_modules` |
| 推进时间 | `CoreRuntime::tick_time`、`advance_time_by` |
| 解析服务 | `CoreRuntime::resolve_manager` / `resolve_manager_handle` |
| 启动产品 | `zircon_app::entry::EntryRunner`、`bootstrap_export_runtime` |
| 启动编辑器 | `zircon_editor::run_editor_with_config` |
| 使用 ABI | `zircon_runtime_interface::runtime_api::ZrRuntimeApiV8` |
| 编写插件 | `zircon_plugin_sdk` 的 plugin declaration/build API |
