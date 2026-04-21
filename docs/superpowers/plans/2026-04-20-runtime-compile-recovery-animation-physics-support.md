---
related_code:
  - zircon_runtime/src/animation/service_types.rs
  - zircon_runtime/src/core/framework/physics/mod.rs
  - zircon_runtime/src/core/framework/scene/world_handle.rs
  - zircon_runtime/src/physics/physics_interface.rs
implementation_files:
  - zircon_runtime/src/animation/service_types.rs
  - zircon_runtime/src/core/framework/physics/mod.rs
  - zircon_runtime/src/core/framework/scene/world_handle.rs
  - zircon_runtime/src/physics/physics_interface.rs
plan_sources:
  - user: 2026-04-20 focus graphics/rendering recovery first, but keep current absorbed runtime layout compiling while continuing M5
  - .codex/plans/Runtime Core Fold-In And Compile Recovery.md
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/sessions/20260417-1415-m5-vg-slot-assignment-ownership.md
tests:
  - cargo check -p zircon_runtime --locked --offline --lib --target-dir D:/cargo-targets/zircon-workspace-compile-recovery
  - cargo test -p zircon_runtime --locked --offline virtual_geometry_submission_records_survive_with_execution_authority_buffer_only --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
doc_type: milestone-detail
---

# Runtime Compile Recovery Animation Physics Support

## Goal

在继续 M5 graphics 路线时，先把阻塞 `zircon_runtime` 本地验证的 animation/physics 定义层 compile drift 用最小修补补平，不回退目录结构，也不扩展行为边界。

## Problem

这轮 Virtual Geometry 单测验证一度被更低层 compile drift 卡住，根因集中在三处：

- `PhysicsSimulationMode` 被上层 `Default` 派生依赖，但自身没有 `Default`
- `WorldHandle` 被 `PhysicsWorldSyncState::default()` 依赖，但自身没有 `Default`
- `PhysicsInterface::sync_scene_world()` 同时继承了 `PhysicsInterface::sync_world` 和 `PhysicsManager::sync_world`，导致方法解析二义性

这些问题都不属于 graphics 行为本身，但它们会直接阻塞 `cargo check` 和 graphics 单测闭环。

## Delivered Slice

### 1. PhysicsSimulationMode 明确默认值

`core/framework/physics/mod.rs` 现在给 `PhysicsSimulationMode` 增加了 `Default` 派生，并把默认值固定为 `Disabled`。这和现有 `PhysicsSettings::default()` 的语义保持一致，不引入新的运行时行为。

### 2. WorldHandle 可参与 default 派生

`core/framework/scene/world_handle.rs` 现在增加了 `Default` 派生，使 `PhysicsWorldSyncState::default()` 这类 DTO 默认构造重新合法。

### 3. PhysicsInterface 同名转发去二义性

`physics/physics_interface.rs` 里的 `sync_scene_world()` 现在显式调用 `PhysicsManager::sync_world(...)`，不再依赖歧义的 trait method 解析。

### 4. Animation service_types 清理无效导入

`animation/service_types.rs` 移除了未使用的 `BTreeMap` 导入，使这轮 compile recovery 不再带无意义噪音。

## Why This Matters

这几处都是最底层的 contract/default/wiring 问题。它们一旦漂移，就会把上层 graphics 验证链一起拖住。先把它们补平，才不会让 M5 graphics 的每一刀都被无关 compile drift 反复打断。

## Validation

- `cargo check -p zircon_runtime --locked --offline --lib --target-dir D:/cargo-targets/zircon-workspace-compile-recovery`
- `cargo test -p zircon_runtime --locked --offline virtual_geometry_submission_records_survive_with_execution_authority_buffer_only --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`
