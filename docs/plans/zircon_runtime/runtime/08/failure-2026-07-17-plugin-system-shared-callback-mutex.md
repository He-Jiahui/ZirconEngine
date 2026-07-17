---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: plugin-system-shared-callback-mutex
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/08
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/plugin/extension_registry/register/system_registration.rs
  - zircon_runtime/src/plugin/extension_registry/register/runtime_scene_system_registration.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/registration_replay.rs
  - zircon_runtime/src/scene/ecs/system/mod.rs
  - zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs
tests:
  - stateful plugin system per-instance factory test
  - stateless plugin system no-mutex callback test
  - multi-world same-registration parallelism and state-isolation test
---

# Runtime08：plugin system instances 共享 callback mutex

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：plugin extension registry 与 ECS system runtime 静态审查
- 修复责任计划：`docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md`
- 共同验收：Plugins01 native generation/lifetime、Runtime11 worker scheduling
- 交接原因：回调所有权属于 system-instance/factory 模型；只在 native plugin replay 层绕开 mutex 会让 Rust plugin 与 runtime-scene systems 继续漂移。

## 失败现象与复现证据

`SystemRegistrationBuilder::register` 将传入的 `S: FnMut` 包装为 `Arc<Mutex<S>>`。每次 build 创建的
`SharedCallbackSceneSystem` 都 clone 同一 Arc，`SceneSystem::run` 每帧获取 mutex 后调用回调。
`RuntimeSceneSystemRegistrationBuilder` 使用相同结构。

这意味着单 World 稳态每次 system run 都有不必要的互斥开销；同一 registration 被多个 World/preview/PIE
实例 build 后，共享同一个可变 callback 状态并跨 World 串行。Native registration replay 的 callback 仅持有稳定
bridge scope/slots、逻辑上无状态，却也被该通用 builder 强制放入 mutex。

## 最低共享层根因

Registration 同时被当作“一份可变 callback 实例”和“可重复 build 的 system factory”。为支持重复 build，代码用
共享 mutex 掩盖所有权矛盾，而不是区分 per-instance stateful factory 与可 `Sync` 共享的 stateless callable。

## 架构修复验收

- Registration 存可重复调用的 system factory；每次 build 产生独立 callback/state 与 `SystemState<P>`。
- stateful `FnMut` callback 归单个 system instance 独占，`run(&mut self)` 直接调用，不获取共享 mutex。
- 明确支持 stateless `Fn + Send + Sync` callable，共享时不使用独占 mutex；native replay 走该契约或 generation-owned factory。
- 同一 registration 构建到两个 World 时 callback state 隔离，并能在无 access conflict 时由不同 worker 并行。
- hot reload/unload 不提前释放在途 system generation；panic/poison recovery 不再依赖共享 callback mutex 语义。
- 单/双 World benchmark 记录 callback mutex acquire=0、overlap、p95 与 worker utilization。

## 参考引擎原则

- Bevy `dev/bevy/crates/bevy_ecs/src/system/function_system.rs` 的 `FunctionSystem` 直接拥有 `func: F` 与 per-system state；
  可 clone factory 的约束显式存在，不用所有运行实例共享一把 callback mutex。
- Zircon 可迁移“system instance owns mutable state”原则，但必须保留自身 plugin generation、owner revocation 与 native ABI lifetime。

## 禁止临时方案

- 不得用 `try_lock` 跳帧或在冲突时静默丢 system run。
- 不得为移除 mutex 而用 `unsafe impl Sync` 包装 `FnMut`。
- 不得只给 native callback 增加旁路而继续让通用 plugin registration 共享可变状态。

## 修复结果与回传

Open state: `待 Runtime08 引入 per-instance system factory/stateless callable 契约并完成多 World 并行验收`。
