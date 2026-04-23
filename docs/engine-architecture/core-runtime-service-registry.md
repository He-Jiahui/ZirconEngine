---
related_code:
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/core/runtime/mod.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/weak.rs
  - zircon_runtime/src/core/runtime/contexts/mod.rs
  - zircon_runtime/src/core/runtime/contexts/module_context.rs
  - zircon_runtime/src/core/runtime/contexts/plugin_context.rs
  - zircon_runtime/src/core/runtime/descriptors/mod.rs
  - zircon_runtime/src/core/runtime/descriptors/service_factory.rs
  - zircon_runtime/src/core/runtime/descriptors/plugin_factory.rs
  - zircon_runtime/src/core/runtime/descriptors/registry_name.rs
  - zircon_runtime/src/core/runtime/descriptors/dependency_spec.rs
  - zircon_runtime/src/core/runtime/descriptors/driver_descriptor.rs
  - zircon_runtime/src/core/runtime/descriptors/manager_descriptor.rs
  - zircon_runtime/src/core/runtime/descriptors/plugin_descriptor.rs
  - zircon_runtime/src/core/runtime/descriptors/module_descriptor.rs
  - zircon_runtime/src/core/runtime/handle/mod.rs
  - zircon_runtime/src/core/runtime/handle/core_handle.rs
  - zircon_runtime/src/core/runtime/handle/registration.rs
  - zircon_runtime/src/core/runtime/handle/activation.rs
  - zircon_runtime/src/core/runtime/handle/resolution.rs
  - zircon_runtime/src/core/runtime/handle/events.rs
  - zircon_runtime/src/core/runtime/state/mod.rs
  - zircon_runtime/src/core/runtime/state/runtime_inner.rs
  - zircon_runtime/src/core/runtime/state/module_entry.rs
  - zircon_runtime/src/core/runtime/state/service_entry.rs
  - zircon_runtime/src/core/runtime/tests.rs
  - zircon_runtime/src/engine_module/service_factory.rs
implementation_files:
  - zircon_runtime/src/core/runtime/mod.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/weak.rs
  - zircon_runtime/src/core/runtime/contexts/mod.rs
  - zircon_runtime/src/core/runtime/contexts/module_context.rs
  - zircon_runtime/src/core/runtime/contexts/plugin_context.rs
  - zircon_runtime/src/core/runtime/descriptors/mod.rs
  - zircon_runtime/src/core/runtime/descriptors/service_factory.rs
  - zircon_runtime/src/core/runtime/descriptors/plugin_factory.rs
  - zircon_runtime/src/core/runtime/descriptors/registry_name.rs
  - zircon_runtime/src/core/runtime/descriptors/dependency_spec.rs
  - zircon_runtime/src/core/runtime/descriptors/driver_descriptor.rs
  - zircon_runtime/src/core/runtime/descriptors/manager_descriptor.rs
  - zircon_runtime/src/core/runtime/descriptors/plugin_descriptor.rs
  - zircon_runtime/src/core/runtime/descriptors/module_descriptor.rs
  - zircon_runtime/src/core/runtime/handle/mod.rs
  - zircon_runtime/src/core/runtime/handle/core_handle.rs
  - zircon_runtime/src/core/runtime/handle/registration.rs
  - zircon_runtime/src/core/runtime/handle/activation.rs
  - zircon_runtime/src/core/runtime/handle/resolution.rs
  - zircon_runtime/src/core/runtime/handle/events.rs
  - zircon_runtime/src/core/runtime/state/mod.rs
  - zircon_runtime/src/core/runtime/state/runtime_inner.rs
  - zircon_runtime/src/core/runtime/state/module_entry.rs
  - zircon_runtime/src/core/runtime/state/service_entry.rs
  - zircon_runtime/src/core/runtime/tests.rs
  - zircon_runtime/src/engine_module/service_factory.rs
plan_sources:
  - user: 2026-04-16 全部积极拆分并按模块边界持续重构所有脚本
  - .codex/plans/全系统重构方案.md
  - .codex/plans/收敛缺口修复 Spec 与 Implementation Plan.md
tests:
  - cargo test -p zircon_runtime core::runtime --locked --target-dir F:\cargo-targets\zircon-codex-a -- --nocapture
  - cargo test -p zircon_runtime script::vm --locked --target-dir F:\cargo-targets\zircon-codex-a -- --nocapture
  - cargo build --workspace --locked --verbose --target-dir F:\cargo-targets\zircon-codex-a
  - cargo test --workspace --locked --verbose --target-dir F:\cargo-targets\zircon-codex-a
doc_type: module-detail
---

# Core Runtime Service Registry

## Purpose

这份文档记录 `zircon_runtime::core::runtime` 目录化后的当前边界。目标不是改变 `CoreRuntime` 的公开契约，而是把 descriptor、上下文、handle 行为和内部状态拆成可扩展子树，并明确 plugin 构造链现在已经从普通 `ServiceFactory` 分流为显式 `PluginFactory + PluginContext`。

## Public Entry Surface

对外公开入口仍然只有 [`zircon_runtime/src/core/runtime/mod.rs`](../../zircon_runtime/src/core/runtime/mod.rs)：

- `CoreRuntime`
- `CoreHandle`
- `CoreWeak`
- `RegistryName`
- `DependencySpec`
- `DriverDescriptor`
- `ManagerDescriptor`
- `PluginDescriptor`
- `ModuleDescriptor`
- `ModuleContext`
- `PluginContext`
- `PluginFactory`
- `ServiceFactory`

也就是说，调用方不需要知道内部子模块是怎么拆的；`runtime/mod.rs` 仍然只是导出层，而不是行为实现层。

## Folder Boundary

当前 `runtime` 子树固定成四层：

- `contexts/`
  - 只放 `ModuleContext`、`PluginContext` 这种纯声明类型。
- `descriptors/`
  - 每个 registry 声明各占一个文件，`ServiceFactory` 与 `PluginFactory` 分文件持有，不再和 runtime 逻辑混写。
- `handle/`
  - `CoreHandle` 自身只保留声明和极小 accessor。
  - 注册、激活/停用、解析、事件/配置 分别拆到独立行为文件。
- `state/`
  - `CoreRuntimeInner`、`ModuleEntry`、`ServiceEntry` 都被压到内部状态层，不再和公开 descriptor 混在一起。

这样之后继续增加新的 service kind、生命周期规则、配置桥或调度行为时，不需要再回到一个巨型 `core.rs` 里追加段落。

## Behavior Split

`CoreHandle` 现在按行为族拆分：

- `registration.rs`
  - module 注册
  - service entry 插入
- `activation.rs`
  - module activate / deactivate
  - unload blocking 检查
- `resolution.rs`
  - driver / manager / plugin 解析
  - dependency chain 递归初始化
  - plugin 解析显式构造 `PluginContext`
  - kind mismatch / cycle / initialization error 收口
- `events.rs`
  - event bus publish / subscribe
  - config store load / store

这一步的核心价值是让“声明一个类型”和“实现某个行为族”分离。以后如果要继续加 metrics、profiling、lifecycle tracing，不会再把 descriptor 层和 resolution 层耦在一起。

## Internal State Discipline

`state/` 目录现在只承载运行时内部权威状态：

- `CoreRuntimeInner`
  - modules registry
  - services registry
  - event bus
  - config store
  - scheduler
- `ModuleEntry`
  - module descriptor + lifecycle
- `ServiceEntry`
  - registry name / owner / kind / startup / dependencies / factory / lifecycle / cached instance
  - factory 进一步细分为普通 `ServiceFactory` 与 plugin 专用 `PluginFactory`

这些结构不是公开 API，不允许再被重新暴露到 `mod.rs`。外部应继续通过 `CoreRuntime` / `CoreHandle` 工作。

## Extension Rule

后续继续扩展 `zircon_runtime::core::runtime` 时，保持以下规则：

- 新的公开声明放到 `descriptors/` 或 `contexts/`，每个顶层声明一个文件。
- 新的 `CoreHandle` 行为先判断属于哪个行为族；如果已经跨出当前文件职责，就新增行为文件。
- `runtime/mod.rs` 只做 `mod` 和 `pub use`。
- 不再恢复单文件 `core.rs` 式混合实现。

## Validation

这轮重构后的验证证据：

- `cargo test -p zircon_runtime core::runtime --locked --target-dir F:\cargo-targets\zircon-codex-a -- --nocapture`
- `cargo test -p zircon_runtime script::vm --locked --target-dir F:\cargo-targets\zircon-codex-a -- --nocapture`
- `cargo build --workspace --locked --verbose --target-dir F:\cargo-targets\zircon-codex-a`
- `cargo test --workspace --locked --verbose --target-dir F:\cargo-targets\zircon-codex-a`
