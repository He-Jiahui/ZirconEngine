---
related_code:
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/core/error.rs
  - zircon_runtime/src/core/lifecycle.rs
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
  - zircon_runtime/src/core/error.rs
  - zircon_runtime/src/core/lifecycle.rs
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
  - user: 2026-06-04 optimize Zircon Engine runtime architecture with breaking changes allowed
  - .codex/plans/全系统重构方案.md
  - .codex/plans/Zircon Runtime 架构渐进式 Review 与优化计划.md
  - .codex/plans/收敛缺口修复 Spec 与 Implementation Plan.md
tests:
  - rustfmt --edition 2021 --check zircon_runtime\src\core\error.rs zircon_runtime\src\core\lifecycle.rs zircon_runtime\src\core\runtime\descriptors\registry_name.rs zircon_runtime\src\core\runtime\state\runtime_inner.rs zircon_runtime\src\core\runtime\handle\registration.rs zircon_runtime\src\core\runtime\handle\activation.rs zircon_runtime\src\core\runtime\handle\resolution.rs zircon_runtime\src\core\runtime\tests.rs
  - registry name static source guard for exact three-segment validation, canonical ServiceKind segment validation, preallocated from_parts construction, canonical module name validation, registration-time descriptor owner/kind consistency, driver dependency direction rejection, transactional service entry commit, typed RegistryName service table keys, typed activation/deactivation service lists, typed resolution recursion stack, dependency-name collection, and borrowed string lookup
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

## Registry Name Contract

`ModuleDescriptor::name` 是 module registry 的 canonical key。`CoreRuntime::register_module(...)` 在拿到 module lock 之前先拒绝空 module name 和首尾包含空白的 module name；这类输入返回 `CoreError::InvalidModuleName`，不会进入 module table，也不会继续注册服务。插件扩展 registry 可以继续用空名 fixture 测自己的无效输入报告，但 runtime 主链不再把空 module name 当成可注册实体。

`RegistryName` 是 runtime service table 的 canonical key，形状固定为 `Module.Kind.Service`，并且每一段都必须非空。`Module` 和 `Service` 段不能带首尾空白；`Kind` 段必须是 `ServiceKind::{Driver, Manager, Plugin}` 对应的 canonical 字符串，不能写成临时的 `Service`、小写别名或旧式分类名。`RegistryName::new(...)` 拒绝少于三段、多于三段、包含空段、包含 module/service 首尾空白或包含未知 kind 段的名字，避免旧式分层名字被悄悄注册进同一张服务表。`RegistryName::from_parts(...)` 现在先按已知长度预分配字符串，再回到同一条 `new(...)` 校验路径；这让 descriptor 构造保持单一 canonical 入口，同时避免 `format!` 在模块装配路径上做额外格式化工作。`module_name()`、`service_kind()`、`service_name()` 是读取三段 registry key 的唯一 owner API；注册、依赖校验和后续性能切片应通过这些 accessor 工作，不再散落手写 `split('.')` 逻辑。

runtime 内部 service table 直接使用 `HashMap<RegistryName, ServiceEntry>`，不再把 canonical key 降级成裸 `String`。`RegistryName` 实现 `Borrow<str>`，所以 `resolve_driver(...)`、`resolve_manager(...)`、`resolve_plugin(...)` 仍然可以用调用方传入的 `&str` 做查找，不需要在每次解析前构造新的 `RegistryName`；注册路径的 pending duplicate set 也使用 `RegistryName`，保证已有服务、同一 descriptor 内的待提交服务、最终写入表三者使用同一种 key 语义。module activation/deactivation 也保持 typed service list 和 typed unloading set，只有对外错误载荷需要 service 名称文本时才转成 `String`。service resolution 的 recursion stack 同样使用 `RegistryName`，依赖遍历只收集 dependency registry keys，而不是直接 clone 整个 `DependencySpec` 列表。这是 M5 performance pass 的基础约束：内部状态保持 typed canonical key，对外入口保持字符串访问兼容，热路径不额外分配 key。

这个契约和 `DependencySpec`、`qualified_name(...)` 共享：依赖声明只存储 canonical `RegistryName`，不再接受带额外层级的兼容格式。模块、driver、manager、plugin 仍然可以用业务命名表达层级，但那一层级必须进入 service 名称本身之前先收敛成一个合法的三段 registry key。

注册期也会重新检查 descriptor 所在集合和 registry key 的 module/kind 段是否一致。服务 key 的 module 段必须等于所属 `ModuleDescriptor::name`；`drivers` 集合里的 descriptor 必须使用 `*.Driver.*`，`managers` 必须使用 `*.Manager.*`，`plugins` 必须使用 `*.Plugin.*`。module 不一致时直接返回 `CoreError::ServiceOwnerMismatch`，kind 不一致时返回 `CoreError::ServiceKindMismatch`，而不是把一个语义矛盾的 entry 插入 runtime service table。

Driver 依赖方向也在注册期执行。`DriverDescriptor` 只能依赖 `*.Driver.*` 服务；如果依赖 `*.Manager.*` 或 `*.Plugin.*`，在 service entry 插入前直接返回 `CoreError::InvalidServiceDependencyKind`。这让底层 IDriver 规则成为 core runtime 的显式契约，而不是依赖后续递归 resolve 路径才暴露问题。Manager 到 Plugin 的依赖清理仍然归 M4 plugin lifecycle 收束，因为当前 `ScriptModule.Manager.VmPluginManager` facade 有意共享 VM plugin runtime 实例；这条 facade 应在插件生命周期切片中反转或替换，而不是在插件活跃会话期间顺手改写。

`register_module(...)` 的 service table 写入是事务式的：先把 driver、manager、plugin descriptor 全部验证并准备成 pending entries，同时检查已有 service 和同一 descriptor 内的重复 service key；只有所有 pending entries 都通过后，才一次性写入 service table。任何一个 descriptor 失败时，前面已经准备好的 service 不会留在 runtime 内部状态里，避免出现 module table 没有 owner、service table 却能 resolve 的孤儿服务。

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
