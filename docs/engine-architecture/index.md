---
related_code:
  - zircon_app/src/lib.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_runtime/src/foundation/mod.rs
  - zircon_runtime/src/extensions/mod.rs
  - zircon_core/src/lib.rs
  - zircon_core/src/runtime/mod.rs
  - zircon_core/src/runtime/runtime.rs
  - zircon_core/src/runtime/handle/mod.rs
  - zircon_core/src/runtime/descriptors/mod.rs
  - zircon_module/src/lib.rs
  - zircon_module/src/engine_module.rs
  - zircon_module/src/engine_service.rs
  - zircon_manager/src/lib.rs
  - zircon_math/src/lib.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/scene/mod.rs
  - zircon_scene/src/lib.rs
  - zircon_scene/src/semantics.rs
  - zircon_asset/src/assets/scene.rs
  - zircon_graphics/src/scene/resources/mod.rs
  - zircon_graphics/src/scene/scene_renderer/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/mod.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/mod.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/mod.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/viewport_overlay_renderer/mod.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/passes/mod.rs
  - zircon_graphics/src/scene/scene_renderer/primitives/mod.rs
  - zircon_editor/src/lib.rs
  - zircon_runtime/src/script/mod.rs
implementation_files:
  - zircon_app/src/lib.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_runtime/src/foundation/mod.rs
  - zircon_runtime/src/extensions/mod.rs
  - zircon_core/src/lib.rs
  - zircon_core/src/runtime/mod.rs
  - zircon_core/src/runtime/runtime.rs
  - zircon_core/src/runtime/handle/mod.rs
  - zircon_core/src/runtime/descriptors/mod.rs
  - zircon_module/src/lib.rs
  - zircon_module/src/engine_module.rs
  - zircon_module/src/engine_service.rs
  - zircon_manager/src/lib.rs
  - zircon_math/src/lib.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/scene/mod.rs
  - zircon_scene/src/lib.rs
  - zircon_scene/src/semantics.rs
  - zircon_asset/src/assets/scene.rs
  - zircon_graphics/src/scene/resources/mod.rs
  - zircon_graphics/src/scene/scene_renderer/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/mod.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/mod.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/mod.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/viewport_overlay_renderer/mod.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/passes/mod.rs
  - zircon_graphics/src/scene/scene_renderer/primitives/mod.rs
  - zircon_editor/src/lib.rs
  - zircon_runtime/src/script/mod.rs
plan_sources:
  - user: 2026-04-13 将架构优先规则保留到 docs 下面用于生产项目 wiki
  - user: 2026-04-15 implement the f64-ready runtime foundation plan with math/scene/asset/graphics boundaries
  - .codex/plans/全系统重构方案.md
tests:
  - docs/engine-architecture/architecture-first-development.md
  - docs/engine-architecture/core-runtime-service-registry.md
  - docs/engine-architecture/runtime-interface-convergence.md
  - docs/engine-architecture/runtime-foundation-precision-and-scene-authority.md
  - cargo check --workspace
doc_type: category-index
---

# Engine Architecture

## Purpose

本目录记录 `zirconEngine` 的全局引擎架构约束，而不是某个单独模块的实现细节。这里的文档用于约束跨 crate 设计、系统重构、模块接入和上层功能开发，确保所有工作都优先落在明确的引擎框架结构上，而不是用一次性的直接实现推进主链路。

## Documents

- [Architecture-First Development](./architecture-first-development.md): `zircon_app -> zircon_core -> zircon_module/zircon_manager -> zircon_runtime + subsystem modules` 主干、ECS 运行时世界、manager façade、runtime absorption 模块、`LevelManager -> LevelSystem -> World` 分层、VM 插件边界、架构优先设计流程、主流引擎对齐要求和实现红线。
- [Core Runtime Service Registry](./core-runtime-service-registry.md): `zircon_core::runtime` 的目录化边界，公开导出层、descriptor 子树、`CoreHandle` 行为文件、内部状态层，以及后续继续扩展 service registry 时必须遵守的模块纪律。
- [Runtime Interface Convergence](./runtime-interface-convergence.md): `EngineEntry`、`EngineModule`、`EngineService`、ECS 语义合同、内建 module owner 收敛、`zircon_runtime::extensions` 对可选扩展注册面的吸收、结构审计 skill，以及当前 `converged/skeleton/needs-refactor` 诊断基线。
- [Runtime Foundation Precision And Scene Authority](./runtime-foundation-precision-and-scene-authority.md): `zircon_math` 精度 seam、`zircon_scene` 的 `LocalTransform + WorldMatrix + ActiveSelf/ActiveInHierarchy + RenderLayerMask + Mobility` authority、scene asset 的默认化新字段，以及 `zircon_graphics` 的 runtime-to-render downcast 边界。

## Current Scope

当前目录覆盖的系统级约束包括：

- 以 [全系统重构方案](../../.codex/plans/全系统重构方案.md) 为默认权威路线图的全局架构基线
- `EntryRunner`、`CoreRuntime`、模块 descriptor、manager façade、`zircon_runtime` 吸收的 foundation/input/platform/script 实现目录与 asset/scene/graphics/ui/optional-extensions module-registration surface、`LevelManager -> LevelSystem -> World`、editor host、VM plugin 的职责分层
- `EngineEntry`、`EngineModule`、`EngineService` 与 `RuntimeObject/RuntimeSystem/EntityIdentity/ComponentData` 这组接口家族和语义合同
- `CoreRuntime` service registry 的文件级边界和 `runtime/mod.rs` 只做导出层的结构纪律
- `zircon_math -> zircon_scene -> zircon_asset -> zircon_graphics` 这条 runtime foundation 的精度与派生态边界
- “先抽象框架，后写功能实现”的工程规则
- “先检查是否和主流引擎模式对齐，过于简单时优先深化架构设计”的设计规则
- 跨 crate 功能接入时对 sibling `zircon_*` crates 的一致性要求

后续如果继续细化 `zircon_core` 生命周期、`zircon_manager` façade 族、`zircon_runtime::foundation` 的 clock/config/event/scheduler 内建模块拆分、`zircon_scene` 的 `LevelSystem` 子系统托管、runtime `f64` 切换过程或 `zircon_runtime::script` VM 热替换协议，可以在本目录继续追加叶子文档。

