---
related_code:
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/scene/module/mod.rs
  - zircon_runtime/src/scene/world_time/mod.rs
implementation_files:
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/scene/module/mod.rs
plan_sources:
  - user: 2026-09-09 场景运行时说明
tests:
  - zircon_runtime/src/scene/level_system/subsystem_snapshot_tests.rs
  - zircon_runtime/src/scene/tests
doc_type: module-detail
---

# 场景运行时与 Level

## 用途

场景运行时把一个可替换的 ECS `World` 包装成 `LevelSystem`，并向渲染、物理、动画、脚本和编辑器同步暴露稳定入口。`SceneModule` 注册默认 Level 管理器（`DEFAULT_LEVEL_MANAGER_NAME`）和世界驱动（`WORLD_DRIVER_NAME`）。

## 概念模型

- `WorldHandle` 标识 Level；`LevelMetadata` 保存 `project_root`、`asset_uri`、`display_name`。
- `LevelLifecycleState` 只有 `Loaded` 与 `Unloaded`。
- `world_generation` 随结构或组件修改递增；`world_replacement_epoch` 在整世界替换时递增，用来使旧查询、绑定和异步任务失效。
- `WorldTimeState` 管理虚拟时钟、固定步进和插值；可用 `TimePolicyTransaction` 原子更新策略。

## 生命周期与数据流

1. `create_level`/`prepare_level` 创建 `World` 并由 `LevelSystem::new` 安装世界同步订阅。
2. 每帧 `LevelSystem::tick` 解析 `WorldDriver`，驱动 ECS schedule；固定步进通过 `begin_fixed_step`、`commit_fixed_step` 或 `abort_fixed_step` 完成。
3. 资产加载或动态场景提交可调用 `replace_world_if_generation`。提交前检查 generation，成功后重置物理、动画、脚本、帧快照和时钟状态，并记录 `WorldFact::WorldReplaced`。
4. 编辑器可用 `watch_world`/`drain_world_invalidations` 获取批量失效事实；视口高亮以 generation 做新旧覆盖判断。

## Rust 调用示例

```rust
use zircon_runtime::scene::{create_default_level, LevelLifecycleState, LevelMetadata, World};

let level = create_default_level(core)?;
level.set_metadata(LevelMetadata {
    display_name: Some("Gameplay".into()), ..Default::default()
});
level.apply_time_policy(/* TimePolicyTransaction */)?;
level.pause_virtual_time();
let snapshot: World = level.snapshot();
assert_eq!(level.lifecycle(), LevelLifecycleState::Loaded);
```

## 错误与限制

`replace_world_if_generation` 在并发修改时返回实际 generation；旧 `CompiledScenePropertyWriter`、动态场景计划和固定步进票据不得跨越世界替换。时间策略错误为 `TimePolicyError`，固定步进错误为 `WorldTimeAdvanceError`/`WorldFixedStepError`（通过 `LevelTickError` 汇总）。

## 实现状态

已实现：Level 包装、世界替换、generation/epoch、时间控制、失效订阅和默认驱动注册。动画和物理运行时部分受 `animation`、`physics-contracts` feature 控制。

## 源码与测试

[level_system.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/scene/level_system.rs)、[module/mod.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/scene/module/mod.rs)、[world_time/mod.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/scene/world_time/mod.rs)；测试见 [subsystem_snapshot_tests.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/scene/level_system/subsystem_snapshot_tests.rs)。
