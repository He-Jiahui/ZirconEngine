---
related_code:
  - zircon_runtime/src/animation/mod.rs
  - zircon_runtime/src/animation/manager/mod.rs
  - zircon_runtime/src/animation/sequence.rs
  - zircon_runtime/src/navigation/mod.rs
  - zircon_runtime/src/navigation/runtime.rs
implementation_files:
  - zircon_runtime/src/animation/manager
  - zircon_runtime/src/animation/sequence
  - zircon_runtime/src/navigation/runtime
  - zircon_runtime/src/navigation/operation
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/plans/zircon_plugins/04/2026-08-01-current-state-and-performance-handoffs.md
  - docs/plans/zircon_runtime/runtime/07/2026-09-01-animation-compiled-channel-sampling-current-review.md
tests:
  - zircon_runtime/src/animation/sequence/tests.rs
  - zircon_runtime/src/animation/manager/pose/performance_tests.rs
  - zircon_runtime/src/navigation/runtime/tests.rs
  - zircon_runtime/src/navigation/runtime/state/repath_entry_tests.rs
doc_type: module-detail
---

# 动画与导航

## 动画管理器

`AnimationModule` 注册 `DefaultAnimationManager`，实现 `AnimationManager` trait 的 playback settings、track path、parameter map、graph/state-machine evaluation 和 clip pose sampling。播放设置以 `ANIMATION_PLAYBACK_CONFIG_KEY` 进入 core config store；manager 反向引用 core 使用 `CoreWeak`，避免 registry ownership cycle。

```rust
use zircon_runtime::animation::DefaultAnimationManager;
use zircon_runtime::core::framework::animation::AnimationPlaybackSettings;

let manager = DefaultAnimationManager::default();
let mut settings = AnimationPlaybackSettings::default();
settings.enabled = true;
manager.store_playback_settings(settings)?;
# Ok::<(), zircon_runtime::core::CoreError>(())
```

调用 graph/state machine 时使用 framework 的 asset DTO；manager 只负责求值，不拥有项目文件或 editor timeline。

## Sequence 编译

`compile_sequence_for_world` 把 sequence asset 编译成 `CompiledAnimationSequence`，`apply_compiled_sequence_to_world` 将采样结果写入 World，并返回 `CompiledAnimationSequenceApplyStats`。编译/应用必须在规定的 runtime/editor operation 阶段执行，不能让 UI callback 直接修改组件。

## 内建导航

`BuiltinNavigationManager` 能加载 baked `NavMeshAsset`、执行 path/sample/raycast，并按 `NavRepathBudget` tick World 中的 agents/obstacles。`tick_world_agents` 和 `tick_world_agent` 会将移动结果写回 `Transform`，同时维护 generation 对齐的导航投影。

```rust
use zircon_runtime::navigation::BuiltinNavigationManager;
use zircon_runtime::core::framework::navigation::NavSampleQuery;

let navigation = BuiltinNavigationManager::new();
// 先通过 NavigationManager::load_nav_mesh 装载非空 baked asset，
// 再调用 find_path/sample_position/raycast。
let _ = std::mem::size_of::<NavSampleQuery>();
```

内建 manager 的 `bake_surface` 会返回 backend failure：它可以加载 baked navmesh，但不负责烘焙 surface。带 per-query `NavQueryFilter` 的查询同样要求激活 navigation plugin；调用前应检查 capability，而不是把错误当作无路径。

## Agent tick 语义

每帧 agent tick 会：

1. 从 World 投影 agent/obstacle 行；
2. 按 repath cursor 和预算选择 agent；
3. 使用缓存路径或 baked mesh 求路径；
4. 做局部避障和 stopping distance 计算；
5. sample 新位置、按需要更新 rotation；
6. 写回 transform 并保存与 world generation 对齐的 projection。

非有限或非正 `dt_seconds` 返回空报告。找不到 agent/navmesh、没有 transform、路径为空和预算耗尽都会记录 `NavAgentTickReport`/`NavigationError`，不应静默 teleport。

## 插件扩展

完整 Recast/Detour 烘焙、过滤器和高级导航通过 `zircon_plugins/navigation` 的 runtime/native/editor 包接入。runtime core 的内建 fallback 保持可加载/查询的最小能力；插件 descriptor、feature 和 backend 成熟度需按[插件清单](../plugins/inventory.md)核对。
