---
related_code:
  - zircon_runtime/src/scene/ecs/mod.rs
  - zircon_runtime/src/scene/ecs/component/mod.rs
  - zircon_runtime/src/scene/ecs/component/id.rs
  - zircon_runtime/src/scene/ecs/component/marker.rs
  - zircon_runtime/src/scene/ecs/component/registry.rs
  - zircon_runtime/src/scene/ecs/storage/component_storage/mod.rs
  - zircon_runtime/src/scene/ecs/storage/component_storage/entry.rs
  - zircon_runtime/src/scene/ecs/storage/component_storage/location.rs
  - zircon_runtime/src/scene/ecs/storage/component_storage/sparse.rs
  - zircon_runtime/src/scene/ecs/storage/component_storage/store.rs
  - zircon_runtime/src/scene/ecs/storage/component_storage/table.rs
  - zircon_runtime/src/scene/ecs/storage/component_storage/utils.rs
  - zircon_runtime/src/scene/ecs/storage_type.rs
  - zircon_runtime/src/scene/ecs/entity/mod.rs
  - zircon_runtime/src/scene/ecs/entity/despawned.rs
  - zircon_runtime/src/scene/ecs/entity/error.rs
  - zircon_runtime/src/scene/ecs/entity/internal.rs
  - zircon_runtime/src/scene/ecs/entity/location.rs
  - zircon_runtime/src/scene/ecs/entity/registry.rs
  - zircon_runtime/src/scene/ecs/entity/slot.rs
  - zircon_runtime/src/scene/ecs/entity/stable_location.rs
  - zircon_runtime/src/scene/ecs/observer/mod.rs
  - zircon_runtime/src/scene/ecs/observer/callbacks.rs
  - zircon_runtime/src/scene/ecs/observer/entry.rs
  - zircon_runtime/src/scene/ecs/observer/id.rs
  - zircon_runtime/src/scene/ecs/observer/store.rs
  - zircon_runtime/src/scene/ecs/observer/utils.rs
  - zircon_runtime/src/scene/ecs/commands/mod.rs
  - zircon_runtime/src/scene/ecs/commands/command.rs
  - zircon_runtime/src/scene/ecs/commands/command_queue.rs
  - zircon_runtime/src/scene/ecs/commands/commands/mod.rs
  - zircon_runtime/src/scene/ecs/commands/commands/entity_commands.rs
  - zircon_runtime/src/scene/ecs/commands/commands/facade.rs
  - zircon_runtime/src/scene/ecs/commands/commands/param.rs
  - zircon_runtime/src/scene/ecs/change_detection/component_ticks.rs
  - zircon_runtime/src/scene/ecs/events/mod.rs
  - zircon_runtime/src/scene/ecs/events/cursor.rs
  - zircon_runtime/src/scene/ecs/events/id.rs
  - zircon_runtime/src/scene/ecs/events/metrics.rs
  - zircon_runtime/src/scene/ecs/events/queue.rs
  - zircon_runtime/src/scene/ecs/events/store.rs
  - zircon_runtime/src/scene/ecs/events/subscription.rs
  - zircon_runtime/src/scene/ecs/messages/mod.rs
  - zircon_runtime/src/scene/ecs/messages/cursor.rs
  - zircon_runtime/src/scene/ecs/messages/id.rs
  - zircon_runtime/src/scene/ecs/messages/queue.rs
  - zircon_runtime/src/scene/ecs/messages/store.rs
  - zircon_runtime/src/scene/ecs/resource/mod.rs
  - zircon_runtime/src/scene/ecs/resource/id.rs
  - zircon_runtime/src/scene/ecs/resource/marker.rs
  - zircon_runtime/src/scene/ecs/resource/registry.rs
  - zircon_runtime/src/scene/ecs/resource_store/mod.rs
  - zircon_runtime/src/scene/ecs/resource_store/stored_resource.rs
  - zircon_runtime/src/scene/ecs/resource_store/store.rs
  - zircon_runtime/src/scene/ecs/bundle.rs
  - zircon_runtime/src/scene/ecs/query/query_state/cache.rs
  - zircon_runtime/src/scene/ecs/query/query_state/stats.rs
  - zircon_runtime/src/scene/world/property_access/path_resolution.rs
  - zircon_runtime/src/animation/sequence/apply.rs
  - zircon_runtime/src/animation/sequence/target.rs
  - zircon_plugins/animation/runtime/src/sequence/apply.rs
  - zircon_plugins/animation/runtime/src/sequence/target.rs
  - zircon_runtime/src/scene/tests/component_structure.rs
  - zircon_runtime/src/scene/tests/component_structure/runtime_08_owner_tree.rs
  - zircon_runtime/src/scene/tests/property_paths.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings.rs
  - zircon_runtime/src/tests/runtime_absorption/ecs_kernel_data.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_query_state_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_query_state_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_kernel_data_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_kernel_data_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_kernel_data_anchor_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_kernel_data_source_inventory.py
  - dev/bevy/crates/bevy_ecs/src
plan_sources:
  - docs/plans/zircon_runtime/runtime/index.md
status: in_progress
last_refined: 2026-06-22
---

# 08 ECS 内核数据面对齐

与子计划 03 的分工：**03 管调度与帧循环**（stage/fixed-step/并行执行），**本计划管数据面内核**——实体生命周期、组件存储、观察者/事件、命令队列、变更追踪的语义定稿与 `bevy_ecs` 逐项对照。性能计数与优化归 07。

## 现状与证据（2026-06-12 实仓盘点）

- 模块齐件（`scene/ecs/` 当前实测 30 个顶层条目，2026-06-20 archetype/component/entity/event/message/resource/resource-store/component-storage/observer/commands facade/change-detection owner hard-cut 后 archetype owner 为 `archetype/{mod,id,index,move_result,record,signature}.rs` 子树，component identity owner 为 `component/{mod,marker,id,registry}.rs` 子树，entity identity owner 为 `entity/{mod,despawned,error,internal,location,registry,slot,stable_location}.rs` 子树，事件 owner 为 `events/{mod,cursor,id,metrics,queue,store,subscription}.rs` 子树，message owner 为 `messages/{mod,cursor,id,queue,store}.rs` 子树，resource identity owner 为 `resource/{mod,marker,id,registry}.rs` 子树，resource store owner 为 `resource_store/{mod,stored_resource,store}.rs` 子树，component storage owner 为 `storage/component_storage/{mod,entry,location,sparse,store,table,utils}.rs` 子树，observer owner 为 `observer/{mod,callbacks,entry,id,store,utils}.rs` 子树，commands facade owner 为 `commands/commands/{mod,entity_commands,facade,param}.rs` 子树，change detection owner 为 `change_detection/{mod,change_tick,change_tick_window,component_ticks,stats,wrappers}.rs` 子树）：archetype 族（`archetype/`）、实体族（`entity/`）、存储（`component/` + `storage/component_storage/` + `storage/{component_remove_result.rs,storage_error.rs}` + `storage_type.rs`）、`bundle.rs`、`commands/{command.rs,command_queue.rs,commands/{mod,entity_commands,facade,param}.rs}`、`change_detection/{mod,change_tick.rs,change_tick_window.rs,component_ticks.rs,stats.rs,wrappers.rs}`、`observer/`、`events/` + `messages/` 双通道、`resource/` + `resource_store/`、`removal.rs`、查询族（`query/` 含 `query_state/`、`cached_query_iter.rs`、combinations/many/unique 迭代器，见 07 计划盘点）。
- **观察者实测**（`observer/{id,store,entry,callbacks,utils}.rs`）：`ObserverId(u64)` + `ObserverStore`，三类观察入口——`observe_lifecycle`、`observe_event<E>`、`observe_entity_event<E>`、`remove`。与 bevy `Observer`/component hooks 的触发时机语义（立即 vs 队列末）需对照定稿。
- **变更追踪同形**：`change_tick/change_tick_window/component_ticks/wrappers` 四件与 `bevy_ecs::change_detection`（Tick/ComponentTicks/Ref-Mut wrappers）形状对应；tick 回绕（wrap-around）语义是否处理需核验。
- **存储形态重核修正**：`storage_type.rs` 已是 `StorageType::{Table,SparseSet}` 双形态枚举，`storage/component_storage/` 由 `store.rs` 作为 `ComponentStorage` facade，并把 table rows、sparse entries、location DTO、raw entry 与工具函数拆成 folder-backed owner set。与 bevy 的差异是本仓保留统一 public storage facade，而不是缺少 SparseSet。
- **事件双通道**：`events/` 与 `messages/` 并存——两者语义分工（帧内事件 vs 跨帧消息？double-buffer 清理策略？）未文档化，是本计划裁决项。
- 已确认健康项（03/07 盘点继承）：`query_state/` 缓存与 `cached_query_iter.rs` 已存在；conflict graph 测试 11 个；`apply_deferred` 屏障语义在 `schedule_runner.rs` 已实现。
- 参考锚点（每点一行）：bevy_ecs 存储双形态 — `dev/bevy/crates/bevy_ecs/src/storage/{table,sparse_set}.rs`；bevy 实体分配/重用（generation 复用）— `dev/bevy/crates/bevy_ecs/src/entity/mod.rs`；bevy Observer/hooks — `dev/bevy/crates/bevy_ecs/src/observer/`；bevy Events double-buffer — `dev/bevy/crates/bevy_ecs/src/event/`。

补充参考锚点（2026-06-13 实测核验，实现型切片动工前先读——index 公约 §7.9）：

- bevy_ecs 存储双形态本体（M0 存储行对照的精确落点）— `dev/bevy/crates/bevy_ecs/src/storage/{table/,sparse_set.rs,blob_array.rs}`
- 实体编码/分配/重用 — `dev/bevy/crates/bevy_ecs/src/entity/mod.rs`
- 观察者（集中/分布式存储 + runner 触发时机，M2.1 时序判词必读）— `dev/bevy/crates/bevy_ecs/src/observer/{mod.rs,runner.rs,centralized_storage.rs,distributed_storage.rs}`
- 事件与触发（M3.1 双通道分工对照）— `dev/bevy/crates/bevy_ecs/src/event/{mod.rs,trigger.rs}`

## 目标

1. 与 `bevy_ecs` 的数据面逐项差距表：存储形态、实体 generation/重用、观察者触发时机、事件缓冲、命令队列冲刷点——每项裁决"有意取舍 / 债"。
2. 实体生命周期语义测试完备：despawn 后句柄失效、ID 重用安全（generation）、stable location 失效规则、removal 事件时序。
3. events/messages 双通道分工定稿并文档化；观察者触发时机定稿。

## 非目标

- 不重写存储/查询实现（除非 M0 裁决出必修债且有 07 的计数证据支撑）；不引入 SparseSet 等新存储形态作为投机优化。
- 调度/并行/fixed-step 归 03；查询缓存命中率计数归 07；reflection 序列化纯净性归 05。

### 全局硬约束（继承总计划 §4，违反即返工）

- 不新增 crate；硬切换不留兼容层；渲染骨架归 render 计划 01-08；非网络语义 server 命名是 blocker。

## 执行前检查清单

1. 活动会话对齐：`scene/ecs/**` 可能被 10fps 会话（`20260611-0416`）触及——`git status --porcelain -- zircon_runtime/src/scene/ecs/`，脏文件避让，禁止回退。
2. 与 03/07 的切片排期对齐：03-M2（FixedStepPlan 接通）会改 `schedule_runner.rs`；07-M1（计数点）会改 `query_state/`/`change_detection/`——同文件切片错峰执行。
3. 事实重核：
   - `ls zircon_runtime/src/scene/ecs/`（核当前 32 个顶层条目清单）
   - `grep -n "pub enum\|pub struct" zircon_runtime/src/scene/ecs/storage_type.rs`（已核：`StorageType::{Table,SparseSet}`）
   - `grep -n "generation\|Generation" zircon_runtime/src/scene/ecs/entity/slot.rs`（核 ID 重用语义现状）
4. 基线记录：`cargo test -p zircon_runtime --lib ecs --locked` 通过数记入状态节。

## 里程碑

### M0 bevy_ecs 数据面差距表

#### 切片 0.1 五维对照表

- 目标文件：`docs/zircon_runtime/scene/ecs.md`（执行时核验存在性：`ls docs/zircon_runtime/scene/`；有则扩展、无则新建并挂 `docs/zircon_runtime/` 索引）。
- 改动形态：纯文档。五维逐项对照，已知行预填：

  | 维度 | bevy_ecs 锚点 | 本仓对应物 | 待裁决问题 |
  |---|---|---|---|
  | 存储形态 | `storage/{table,sparse_set}.rs` 双形态 | `storage_type.rs` 双形态枚举 + `storage/component_storage/{store,table,sparse,location}.rs` table/sparse 双 backing store | 当前能力保留；是否进一步优化必须等 07 计数证据，不把统一 public facade 误判为功能债 |
  | 实体分配/重用 | `entity/mod.rs` generation 复用 | `entity/{mod,despawned,error,internal,location,registry,slot,stable_location}.rs` | generation 等价物是否存在；despawn 后旧句柄访问行为 |
  | 观察者 | `observer/` + component hooks | `observer/{store,entry,callbacks,id,utils}.rs` 三类观察（lifecycle/event/entity_event） | 触发时机（立即 vs apply_deferred 后）；观察者内再触发的递归语义 |
  | 事件 | `event/` double-buffer + 显式清理 | `events/` + `messages/` 双通道 | 双通道分工判词；滞留事件清理策略（prune 在 event_bus 是 core 层，ECS 层呢） |
  | 命令队列 | `Commands` + 队列冲刷点 | `commands/command_queue.rs` + `commands/commands/{facade,entity_commands,param}.rs` + runner 的 apply_deferred | 冲刷点已显式（03 盘点）；Commands facade / EntityCommands / CommandsParam owner 已拆分 |

- 调用方迁移：无。
- 验收：五行全部有"对应物 + 差异 + 裁决"；裁决无"待定"。
- DoD：差距表落 `ecs.md`，需修债项进 M1–M3 工作集，"保留差异"项写明理由。

#### M0 测试阶段（milestone-first）

- 纯审计：`git status --porcelain` 仅 docs 变更。

### M1 实体生命周期语义测试完备

#### 切片 1.1 despawn / 重用 / stable location 测试矩阵

- 目标文件：`zircon_runtime/src/scene/tests/`（既有 ecs 测试树，落点执行时定：`ls zircon_runtime/src/scene/tests/`）；若 M0 发现 generation 缺失且裁决为债，修改点在 `entity/registry.rs` + `entity/despawned.rs` + `entity/slot.rs`。
- 改动形态：测试矩阵（测试名草案）：
  - `despawned_entity_handle_is_rejected_by_world_access`（旧句柄访问 → 显式错误/None，不得读到重用实体）
  - `entity_id_reuse_does_not_alias_previous_generation_handle`（若有 generation；无则此测试是"债证明"）
  - `stable_entity_location_survives_archetype_move_and_invalidates_on_despawn`
  - `component_removal_emits_removal_record_in_same_frame`（`removal.rs` 时序锚）
- 调用方迁移：无（纯测试；若补 generation 字段则构造点枚举：Grep `EntityRegistry::new|spawn`，path `zircon_runtime/src/scene`）。
- 验收：四测试落地；任何"债证明"测试以 `#[should_panic]`/显式注释形式锚定现状并列 M3 修复条目。
- DoD：`cargo test -p zircon_runtime --lib entity --locked` 全绿（含锚定测试）。

#### M1 测试阶段（milestone-first）

- `cargo check -p zircon_runtime --lib --locked`（切片期）
- `cargo test -p zircon_runtime --lib entity --locked -- --nocapture`；`cargo test -p zircon_runtime --lib ecs --locked`
- 验收证据：生命周期测试矩阵 + 差距表对应行回写实测结论。

### M2 观察者与命令队列语义定稿

#### 切片 2.1 观察者触发时机测试

- 目标文件：`observer/` 消费点 + `scene/tests/`（观察者测试位执行时核验：Grep `ObserverStore`，path `zircon_runtime/src/scene`）。
- 改动形态：定稿三类观察（lifecycle/event/entity_event）的触发时机并各补时序测试：
  - `lifecycle_observer_fires_after_apply_deferred_not_during_command_replay`（或按定稿改名——时机判词二选一：立即触发 / 冲刷后触发）
  - `entity_event_observer_only_fires_for_target_entity`
  - `observer_remove_during_dispatch_does_not_skip_or_double_fire`（迭代中移除的安全性）
- 调用方迁移：无（语义收紧若改触发点，消费方枚举：Grep `observe_lifecycle|observe_event|observe_entity_event`，path `zircon_runtime/src`）。
- 验收：三测试 + 触发时机判词写入 `ecs.md`。
- DoD：`cargo test -p zircon_runtime --lib observer --locked` 全绿。

#### 切片 2.2 命令队列冲刷与错误路径

- 目标文件：`commands/command_queue.rs` + 测试位。
- 改动形态：补错误路径测试——对已 despawn 实体的排队命令（insert/remove）冲刷时的行为定稿（静默跳过 vs 显式错误记录，二选一判词）；`command_queue_on_despawned_entity_target_is_reported_not_silently_dropped`（名按判词定）。
- 调用方迁移：无。
- 验收：判词 + 测试。
- DoD：`cargo test -p zircon_runtime --lib command --locked` 全绿。

#### M2 测试阶段（milestone-first）

- `cargo test -p zircon_runtime --lib observer --locked -- --nocapture`
- `cargo test -p zircon_runtime --lib command --locked -- --nocapture`
- `cargo test -p zircon_runtime --lib ecs --locked`（全族无回归）

### M3 事件双通道与变更追踪收尾

#### 切片 3.1 events/messages 分工定稿

- 目标文件：`events/`、`messages/` + `ecs.md`（分工判词）。
- 改动形态：盘点两通道的生产/消费方（Grep `ecs::events|ecs::messages`，path `zircon_runtime/src`），定稿分工（候选口径：events = 帧内即时、messages = 跨帧缓冲）；清理策略测试：`undrained_messages_are_pruned_after_tick_window_not_leaked`（按实仓语义定名）。若盘点发现两通道职责重叠 → 合并裁决（硬切换，被并方调用方同切片迁移）。
- 调用方迁移：仅合并裁决时发生；枚举命令同上。
- 验收：分工判词 + 清理测试；或合并完成。
- DoD：`ecs.md` 双通道节与代码一致。

#### 切片 3.2 change tick 回绕与窗口测试

- 目标文件：`change_detection/{change_tick.rs,change_tick_window.rs}` 测试位。
- 改动形态：核验 tick 回绕处理（对照 bevy `Tick::is_newer_than` 的 wrap 语义）；缺则补（修改点 `change_tick.rs`，签名草案执行时定稿）；测试：`change_tick_comparison_survives_wraparound`、`tick_window_clamps_stale_ticks`。
- 调用方迁移：无公共面变化。
- 验收：两测试绿。
- DoD：回绕语义有测试锚；07 的 change_detection 计数点（07-M1）可安全叠加。

#### M3 测试阶段（milestone-first）

- `cargo test -p zircon_runtime --lib change_tick --locked -- --nocapture`；`cargo test -p zircon_runtime --lib messages --locked`
- `cargo test -p zircon_runtime --lib ecs --locked`（收尾全族）
- 验收证据：差距表全部行闭环；`docs/zircon_runtime/scene/ecs.md` 与代码一致。

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据（命令输出 / 文件 / 测试名） |
|---|---|---|---|---|
| M0 | 0.1 五维对照表 | 完成 | 2026-06-12 | `docs/zircon_runtime/scene/ecs.md` 新增 Runtime 08 Data-Kernel Alignment Verdict；裁决 storage/entity/observer/events/messages/commands/change tick 六行，无待定项 |
| M1 | 1.1 生命周期测试矩阵 | code_complete_pending_cargo | 2026-06-12 | 已补 `despawned_entity_handle_is_rejected_by_world_access`、`entity_id_reuse_does_not_alias_previous_generation_handle`、`stable_entity_location_survives_archetype_move_and_invalidates_on_despawn`、`component_removal_emits_removal_record_in_same_frame`；Cargo 待活动 lanes 清空后运行 |
| M2 | 2.1 观察者时序 | code_complete_pending_cargo | 2026-06-12 | 已补 `lifecycle_observer_fires_immediately_during_component_mutation`、`entity_event_observer_only_fires_for_target_entity`、`observer_remove_during_dispatch_does_not_skip_or_double_fire`；`docs/zircon_runtime/scene/ecs.md` 写入同步触发、目标实体、dispatch 移除判词；Cargo 待活动 lanes 清空后运行 |
| M2 | 2.2 命令队列错误路径 | code_complete_pending_cargo | 2026-06-12 | 已补 `DeferredCommandReport` / `DeferredCommandError` / `DeferredCommandOperation` 报告面与 `command_queue_on_despawned_entity_target_is_reported_not_silently_dropped`、`deferred_command_success_report_counts_applied_commands_without_errors`；Cargo 待活动 lanes 清空后运行 |
| M3 | 3.1 双通道定稿 | code_complete_pending_cargo | 2026-06-12 | 已补 `events_require_explicit_update_and_keep_next_queue_hidden`、`first_stage_updates_all_registered_event_channels`、`clear_events_prunes_current_and_next_event_queues`、`messages_are_retained_until_explicit_clear_independent_of_event_updates`、`event_and_message_clear_boundaries_do_not_cross_channels`；裁决 events=current/next 帧推进并在 `SystemStage::First` 统一执行 `EventStore::update_all()`，messages=带 id 的显式保留缓冲；Cargo 待活动 lanes 清空后运行 |
| M3 | 3.2 tick 回绕 | code_complete_pending_cargo | 2026-06-12 | 已补 `ChangeTick::next()` 回绕递增、`ChangeTick::relative_to(...)` 回绕差值、`ChangeTick::is_newer_than(...)` 相对年龄比较、`ChangeTickWindow::new(...)` stale `last_run` 截断，以及 `change_tick_comparison_survives_wraparound`、`tick_window_clamps_stale_ticks`；Cargo 待活动 lanes 清空后运行 |
| M4 | QueryState owner 审计同步 | structure_audit_static_passed_cargo_pending | 2026-06-13 | `ecs_query_state_boundary` 已把 `query_state/stats.rs` 归类为 Runtime 07 telemetry sidecar，而不是 ECS 数据面异常 owner module；当前结构审计事实为 `expected_module_count = 8`、`unexpected_modules = []`、`risks = []`，并与 `docs/zircon_runtime/scene/ecs/query_state.md` 的 Boundary Rules 保持一致。Cargo 验证状态仍由本计划验证门守卫统一保持 pending。 |
| M4 | QueryState cache owner split | query_state_cache_owner_split_static_passed_cargo_pending | 2026-06-17 | `zircon_runtime/src/scene/ecs/query/query_state/cache.rs` 新增为 cache rebuild、cache-slot lookup、cached entity/component-location accessor 与 cache metadata accessor owner；`query_state/mod.rs` 收缩为 `QueryState` 字段、构造与 access descriptor，当前 84/180 non-empty lines。`ecs_query_state_boundary` 同步 `expected_module_count = 9`、`unexpected_modules = []`、`root_non_empty_lines = 84/180`、`risks = []`；`ArchetypeIndex::matching_archetypes(...)` 当前保持直接预分配投影：有 required component 时从 borrowed shortest bucket 推入匹配 id，无 required component 时按 full record count 预分配并直接扫描 records，不恢复 `all_archetype_ids(...)` helper 或 clone-then-retain。验证：rustfmt check、Python py_compile、direct `ecs_query_state_boundary_audit` / `ecs_kernel_data_boundary_audit` risks=[]、standalone `ecs_query_structure.rs` 11/11、standalone `ecs_kernel_data.rs` 1/1、standalone `performance_hotspots.rs` 8/8、standalone `plan_status.rs` 32/32；`docs/zircon_runtime/scene/ecs/query_state.md`、`docs/zircon_runtime/scene/ecs.md`、M0 review、runtime-interface convergence 与 runtime index 已同步；entity/observer/command/messages/change_tick/ecs Cargo gates 仍 pending。 |
| M4 | ECS event owner folder split | ecs_events_folder_split_static_passed_cargo_deferred | 2026-06-20 | `zircon_runtime/src/scene/ecs/events.rs` 已硬切换为 folder-backed `events/` 子树：`mod.rs` 只保留 public re-export，`id.rs` 持有 `Event` / `EventTypeId`，`metrics.rs` 持有 payload/capacity DTO 与阈值常量，`queue.rs` 持有 `Events<T>` 双缓冲队列和容量策略，`cursor.rs` 持有 `EventCursor` / `EventReadIter`，`subscription.rs` 持有 dormant/connected reader boundary，`store.rs` 持有 erased typed channel registry、dense slot lookup、send/update/drain/report API。`scene::ecs` public exports 与 `EventStore::send_by_id(...)` registered-channel send 语义保持不变，producer write 不回退到 active-reader gating。`ecs_kernel_data_boundary` 和 `runtime_08_ecs_kernel_data_mirror_docs_match_structure_audit_counts` 已同步 `expected_source_file_count = 26`；按“首先实现功能，测试可以暂时延后”不新增行为测试，后续 entity/observer/command/messages/change_tick/ecs Cargo gate 仍 pending。 |
| M4 | ECS message owner folder split | ecs_messages_folder_split_static_passed_cargo_deferred | 2026-06-20 | `zircon_runtime/src/scene/ecs/messages/` 已硬切换为 folder-backed 子树：`mod.rs` 只保留 public re-export，`id.rs` 持有 `Message` / `MessageId<T>`，`queue.rs` 持有 retained `Messages<T>` 队列与 explicit-clear generation，`cursor.rs` 持有 `MessageCursor` / `MessageReadIter`，`store.rs` 持有 erased `MessageStore` TypeId registry 与 typed read/write API。`scene::ecs` public exports、`MessageStore::write(...)` / `write_batch(...)`、reader cursor 与 explicit `clear_messages` retention boundary 保持不变；该切片当时同步 source files 30/30；当前 archetype/component/entity/resource identity/resource-store owner 后 `ecs_kernel_data_boundary` 和 `runtime_08_ecs_kernel_data_mirror_docs_match_structure_audit_counts` 已提升为 `expected_source_file_count = 51` / source files 51/51；按“首先实现功能，测试可以暂时延后”不新增行为测试，后续 entity/observer/command/messages/change_tick/ecs Cargo gate 仍 pending。 |
| M4 | ECS resource store owner folder split | ecs_resource_store_folder_split_static_passed_cargo_deferred | 2026-06-20 | `ResourceStore` flat owner 已硬切换为 folder-backed `resource_store/` 子树：`mod.rs` 只保留 `ResourceStore` public re-export，`stored_resource.rs` 持有 erased boxed value、type name 与 `ComponentTicks` 记录，`store.rs` 持有 `TypeId` registry、typed `insert/get/get_mut/remove/ticks/type_names` API 与 direct-branch hot paths。`scene::ecs::ResourceStore` public export、resource change tick 语义、typed downcast 行为与 source hot-path guard 保持不变；`ecs_schedule.rs`、`ecs_change_detection.rs`、`ecs_kernel_data.rs` 与 `ecs_kernel_data_boundary` 已改读 folder-backed owner set，当前 source files 33/33；Cargo 行为 gate 按“首先实现功能，测试可以暂时延后”保持 deferred。 |
| M4 | ECS resource identity owner folder split | ecs_resource_identity_folder_split_static_passed_cargo_deferred | 2026-06-20 | `Resource` / `ResourceId` / `ResourceRegistry` 旧 flat owners 已硬切换为 folder-backed `resource/` 子树：`mod.rs` 只保留 public re-export，`marker.rs` 持有 `Resource` marker trait，`id.rs` 持有 typed `ResourceId`，`registry.rs` 持有 `ResourceDescriptor` 与 `ResourceRegistry` TypeId registry。`scene::ecs::{Resource, ResourceId, ResourceDescriptor, ResourceRegistry}` public exports、resource id allocation、registered lookup 与 descriptor access 行为保持不变；`component_structure.rs`、`ecs_kernel_data.rs` 与 `ecs_kernel_data_boundary` 已改读 folder-backed owner set，当前 source files 37/37；Cargo 行为 gate 按“首先实现功能，测试可以暂时延后”保持 deferred。 |
| M4 | ECS component identity owner folder split | ecs_component_identity_folder_split_static_passed_cargo_deferred | 2026-06-20 | `Component` / `ComponentId` / `ComponentRegistry` 旧 flat owners 已硬切换为 folder-backed `component/` 子树：`mod.rs` 只保留 public re-export，`marker.rs` 持有 `Component` marker trait 与默认 `StorageType::Table`，`id.rs` 持有 typed `ComponentId`，`registry.rs` 持有 `ComponentDescriptor`、`ComponentDescriptorSource` 与 Rust/dynamic component TypeId registry。`scene::ecs::{Component, ComponentId, ComponentDescriptor, ComponentDescriptorSource, ComponentRegistry}` public exports、Rust component id allocation、dynamic component id allocation、registered lookup、descriptor access 与 `rust_type_for_id(...)` 行为保持不变；`component_structure.rs`、`ecs_kernel_data.rs` 与 `ecs_kernel_data_boundary` 已改读 folder-backed owner set，当前 source files 41/41；Cargo 行为 gate 按“首先实现功能，测试可以暂时延后”保持 deferred。 |
| M4 | ECS entity identity owner folder split | ecs_entity_identity_folder_split_static_passed_cargo_deferred | 2026-06-20 | `EntityRegistry` / `EntityRegistryError` / `EntityLocation` / `InternalEntity` / `StableEntityLocation` / `DespawnedEntity` 旧 flat owners 已硬切换为 folder-backed `entity/` 子树：`mod.rs` 只保留 public re-export，`registry.rs` 持有 stable-to-internal registry 行为，`slot.rs` 持有 generation slot 与 wrap helper，`internal.rs` 持有 generational handle，`location.rs` 与 `stable_location.rs` 持有 location DTO，`despawned.rs` 持有 despawn report，`error.rs` 持有 registry error。`scene::ecs::{EntityRegistry, EntityRegistryError, EntityLocation, InternalEntity, StableEntityLocation, DespawnedEntity}` public exports、spawn/despawn、stable lookup、generation wrap 与 stale-handle rejection 行为保持不变；`component_structure.rs`、`ecs_identity_storage.rs`、`ecs_kernel_data.rs` 与 `ecs_kernel_data_boundary` 已改读 folder-backed owner set，当前 source files 45/45；Cargo 行为 gate 按“首先实现功能，测试可以暂时延后”保持 deferred。 |
| M4 | ECS archetype owner folder split | ecs_archetype_owner_split_static_passed_cargo_deferred | 2026-06-20 | `ArchetypeId` / `ArchetypeSignature` / `ArchetypeIndex` 旧 flat owners 已硬切换为 folder-backed `archetype/` 子树：`mod.rs` 只保留 public re-export，`id.rs` 持有 typed archetype id，`signature.rs` 持有 table/sparse component signature 与 normalization，`record.rs` 持有 archetype entity rows，`move_result.rs` 持有 move report，`index.rs` 持有 signature map、component inverted index、direct matching projection 与 entity row move/remove 行为。`scene::ecs::{ArchetypeId, ArchetypeIndex, ArchetypeMove, ArchetypeRecord, ArchetypeSignature}` public exports、signature lookup、entity move reporting、shortest-bucket query matching 与 no-required full-record scan 保持不变；`ecs_archetype_index_structure.rs`、`ecs_query_structure.rs`、`component_structure.rs`、`ecs_kernel_data.rs` 与 `ecs_kernel_data_boundary` 已改读 folder-backed owner set，当前 source files 51/51、archetype anchors 15/15；Cargo 行为 gate 按“首先实现功能，测试可以暂时延后”保持 deferred。 |
| M4 | ECS component storage owner folder split | ecs_component_storage_owner_split_static_passed_cargo_deferred | 2026-06-20 | `ComponentStorage` 旧 flat owner 已硬切换为 folder-backed `storage/component_storage/` 子树：`mod.rs` 只保留 `ComponentStorage` / `ComponentStorageLocation` public re-export，`store.rs` 持有 public facade、typed dispatch、type/storage guards 与 table/sparse bucket partition helper，`table.rs` 持有 row-index table backend，`sparse.rs` 持有 sparse-set backend，`location.rs` 持有 location DTO，`entry.rs` 持有 erased raw value/remove result，`utils.rs` 持有 deterministic sort/downcast helpers。`scene::ecs::{ComponentStorage, ComponentStorageLocation}` public export、table/sparse insert/get/remove/tick/location semantics、component id partition 与 debug output 行为保持不变；`component_structure.rs`、`ecs_component_storage_structure.rs`、`ecs_typed_api.rs`、`ecs_kernel_data.rs` 与 `ecs_kernel_data_boundary` 已改读 folder-backed owner set，当前 source files 57/57、storage anchors 9/9；Cargo 行为 gate 按“首先实现功能，测试可以暂时延后”保持 deferred。 |
| M4 | ECS component storage private re-export cleanup | ecs_component_storage_private_reexport_cargo_check_passed | 2026-06-20 | 状态锚 `ecs_component_storage_private_reexport_cargo_check_passed`；`storage/component_storage/mod.rs` 现在只保留 `ComponentStorage` / `ComponentStorageLocation` public re-export，不再作为 `RawRemoveResult`、`StoredComponent`、`SparseComponentStorage`、`TableComponentStorage`、`downcast_component` 与 `sort_component_ids_if_needed` 的 parent private re-export hub；`sparse.rs` / `table.rs` 直接导入 sibling `entry` owner，`utils.rs` 直接导入 `entry::StoredComponent`，`store.rs` 直接导入 `location`、`sparse`、`table` 与 `utils` owners。该切片修复 folder-backed owner visibility drift，不改变 public ECS storage API 或行为；验证：Python py_compile、direct `ecs_kernel_data_boundary_audit` 报告 `component_storage_private_reexport_anchors = 9/9`、`unexpected_component_storage_private_reexports = []`，standalone `ecs_kernel_data.rs` 1/1 与 rustfmt check 通过；`cargo check -p zircon_runtime --lib --locked --no-default-features --features core-min --jobs 1 --target-dir D:\cargo-targets\zircon-runtime08-component-storage-private-0620 --message-format short --color never` passed with existing warnings only；不提升 entity/observer/command/messages/change_tick/ecs behavior Cargo gates。 |
| M4 | ECS observer owner folder split | ecs_observer_owner_split_static_passed_cargo_deferred | 2026-06-20 | `ObserverStore` 旧 flat owner 已硬切换为 folder-backed `observer/` 子树：`mod.rs` 只保留 `ObserverId` / `ObserverStore` public re-export 与 callback type owner wiring，`id.rs` 持有 observer handle，`callbacks.rs` 持有 erased lifecycle/global/entity callback 类型，`entry.rs` 持有三类 observer 记录，`store.rs` 持有 public observe/remove/callback clone-out API，`utils.rs` 持有 exact-capacity callback counter 与 first-match removal helper。`scene::ecs::{ObserverId, ObserverStore}` public export、immediate synchronous dispatch、clone-out callback invocation、dispatch 中移除只影响后续触发的语义保持不变；`ecs_observers_messages.rs`、`component_structure.rs`、`ecs_kernel_data.rs` 与 `ecs_kernel_data_boundary` 已改读 folder-backed owner set，当前 source files 65/65、observer anchors 8/8；Cargo 行为 gate 按“首先实现功能，测试可以暂时延后”保持 deferred。 |
| M4 | ECS commands facade owner split | ecs_commands_facade_owner_split_static_passed_cargo_deferred | 2026-06-20 | `Commands` 旧 broad owner 已硬切换为 folder-backed `commands/commands/` 子树：`mod.rs` 只保留 `Commands` / `EntityCommands` / `CommandsParam` public re-export，`facade.rs` 持有 `Commands` 队列、spawn/entity/resource facade 与 deferred error reporting，`entity_commands.rs` 持有 `EntityCommands` builder，`param.rs` 持有 `CommandsParam` SystemParam bridge。`scene::ecs::{Commands, EntityCommands, CommandsParam}` public export、deferred visibility、`DeferredCommandReport` 与 missing-target error reporting 行为保持不变；`ecs_commands.rs`、`ecs_kernel_data.rs` 与 `ecs_kernel_data_boundary` 已改读 folder-backed owner set，当前 source files 65/65、deferred command anchors 11/11；Cargo 行为 gate 按“首先实现功能，测试可以暂时延后”保持 deferred。 |
| 横切 | Runtime 08 ECS command Cargo 验证窗口探测 | cargo_recheck_timeout_no_result | 2026-06-20 | `cargo test -p zircon_runtime --lib command --locked --no-default-features --features core-min --jobs 1 --target-dir target\codex-runtime08-commands-owner-0620 --message-format short --color never -- --test-threads=1 --nocapture` ended with 904s timeout no result（15 分钟工具窗口超时，未产生测试结果）；target-dir process scan reported no residual cargo/rustc for `codex-runtime08-commands-owner-0620`。该窗口只记录 no-result timeout，不提升 commands facade owner split 或 entity/observer/command/messages/change_tick/ecs Cargo gates。 |
| 横切 | Runtime 08 ECS entity Cargo 验证窗口探测 | cargo_recheck_timeout_no_result | 2026-06-20 | `cargo test -p zircon_runtime --lib entity --locked --no-default-features --features core-min --jobs 1 --target-dir D:\cargo-targets\zircon-runtime08-component-storage-private-0620 --message-format short --color never -- --test-threads=1 --nocapture` exceeded the 1200s tool window with no test result；residual cargo/rustc processes in that target dir were stopped. 该窗口只记录 no-result timeout，不提升 entity/observer/command/messages/change_tick/ecs behavior Cargo gates。 |
| M4 | ECS data owner-tree guard | ecs_data_owner_tree_guard_static_passed_cargo_pending | 2026-06-20 | 新增 `runtime_08_ecs_data_owner_trees_stay_folder_backed_after_cutover`，要求 `scene/ecs/{archetype,component,entity,events,messages,observer,resource,resource_store}` 八个 Runtime 08 data owner、`scene/ecs/storage/component_storage/` 与 `scene/ecs/commands/commands/{mod,entity_commands,facade,param}.rs` 继续保持 folder-backed，每个 owner `mod.rs` 保持 structural module/export owner，并要求 retired flat Runtime 08 ECS owner 文件 `component.rs`、`component_id.rs`、`component_registry.rs`、`despawned_entity.rs`、`entity_location.rs`、`entity_registry.rs`、`entity_registry_error.rs`、`internal_entity.rs`、`stable_entity_location.rs`、`events.rs`、`messages.rs`、`observer.rs`、`resource.rs`、`resource_id.rs`、`resource_registry.rs`、`resource_store.rs`、`archetype_id.rs`、`archetype_index.rs`、`archetype_signature.rs`、`storage/component_storage.rs` 与 `commands/commands.rs` 不复活。该守卫只封住 owner-tree hard cutover，不提升 entity/observer/command/messages/change_tick/ecs Cargo gates。 |
| M4 | ECS change detection owner-tree guard | ecs_change_detection_owner_tree_guard_static_passed_cargo_pending | 2026-06-20 | 新增 `runtime_08_ecs_change_detection_owner_tree_stays_folder_backed_after_cutover`，要求 `scene/ecs/change_detection/{mod,change_tick,change_tick_window,component_ticks,stats,wrappers}.rs` 继续保持 folder-backed，`change_detection/mod.rs` 保持 structural module/export owner，并要求 retired flat `scene/ecs/change_detection.rs`、`scene/ecs/change_tick.rs`、`scene/ecs/change_tick_window.rs`、`scene/ecs/component_ticks.rs`、`scene/ecs/change_detection_stats.rs` 与 `scene/ecs/change_detection_wrappers.rs` 不复活。`ecs_kernel_data_boundary` 与镜像文档已同步到 source files 69/69、Runtime 08 guard anchors 21/21；该守卫只封住 change detection owner-tree hard cutover，不提升 entity/observer/command/messages/change_tick/ecs Cargo gates。 |
| M4 | ECS root leaf owner guard | ecs_root_leaf_owner_guard_static_passed_cargo_pending | 2026-06-20 | 新增 `runtime_08_ecs_root_leaf_owners_stay_explicit_after_data_cutover`，要求 `scene/ecs/{bundle,removal,storage_type}.rs` 继续作为显式根层叶子 owner，`scene/ecs/mod.rs` 继续声明并导出 `Bundle`、`RemovedComponentEvent` / `RemovedComponentEvents` / `RemovedComponentReader` 与 `StorageType`，并防止这些小型叶子未经计划漂移成新子树。`ecs_kernel_data_boundary` 现在把 `bundle.rs` 纳入 Runtime 08 数据面源文件，当前 source files 69/69、Runtime 08 guard anchors 21/21；该守卫只封住 root leaf owner 形态，不提升 entity/observer/command/messages/change_tick/ecs Cargo gates。 |
| M4 | 验证门守卫 | cargo_validation_pending_guarded | 2026-06-13 | 新增 `runtime_absorption::plan_status::runtime_08_ecs_kernel_cargo_pending_gate_stays_explicit_until_ecs_validation`，要求本计划保持 `in_progress`，M1/M2/M3 行继续保留 `code_complete_pending_cargo` 与 Cargo gate，直到 `cargo test -p zircon_runtime --lib entity --locked -- --nocapture`、`cargo test -p zircon_runtime --lib observer --locked -- --nocapture`、`cargo test -p zircon_runtime --lib command --locked -- --nocapture`、`cargo test -p zircon_runtime --lib messages --locked`、`cargo test -p zircon_runtime --lib change_tick --locked -- --nocapture` 与 `cargo test -p zircon_runtime --lib ecs --locked` 有真实通过记录。 |
| M4 | ECS 数据面结构审计镜像 | structure_audit_static_passed_cargo_pending | 2026-06-14 | 新增 `ecs_kernel_data_boundary` 并接入 `audit_runtime_structure.py`；当前结构审计事实为 `expected_source_file_count = 69`、`expected_test_file_count = 8`、`archetype_anchors = 15/15, storage_anchors = 9/9`、`component_storage_private_reexport_anchors = 9/9`、`unexpected_component_storage_private_reexports = []`、`component_identity_anchors = 18/18`、`entity_lifecycle_anchors = 10/10`、`observer_anchors = 8/8`、`deferred_command_anchors = 11/11`、`event_message_anchors = 12/12`、`resource_identity_anchors = 12/12`、`change_tick_anchors = 6/6`、`runtime_08_guard_anchors = 21/21`、`behavior_test_anchor_count = 16`、`missing_behavior_test_anchors = []`、`doc_anchors = 13/13`、`pending_cargo_gate_anchors = 6/6`、`mirror_docs_guard_present = true`、`risks = []`。这仍是静态结构证据；entity/observer/command/messages/change_tick/ecs Cargo filters 继续 pending。 |
| M4 | ECS 数据面镜像文档守卫 | mirror_docs_static_passed_cargo_pending | 2026-06-14 | 新增 `runtime_absorption::ecs_kernel_data::runtime_08_ecs_kernel_data_mirror_docs_match_structure_audit_counts`，锁定 `docs/zircon_runtime/scene/ecs.md`、本计划、runtime index、M0 review 与 runtime-interface convergence 均同步记录 `ecs_kernel_data_boundary` 的同一组结构事实：`expected_source_file_count = 69`、`expected_test_file_count = 8`、`archetype_anchors = 15/15, storage_anchors = 9/9`、`component_storage_private_reexport_anchors = 9/9`、`unexpected_component_storage_private_reexports = []`、`component_identity_anchors = 18/18`、`entity_lifecycle_anchors = 10/10`、`observer_anchors = 8/8`、`deferred_command_anchors = 11/11`、`event_message_anchors = 12/12`、`resource_identity_anchors = 12/12`、`change_tick_anchors = 6/6`、`runtime_08_guard_anchors = 21/21`、`behavior_test_anchor_count = 16`、`missing_behavior_test_anchors = []`、`doc_anchors = 13/13`、`pending_cargo_gate_anchors = 6/6`、`mirror_docs_guard_present = true`、`risks = []`。验证：rustfmt check 通过；Python py_compile 与 direct `ecs_kernel_data_boundary_audit` 断言通过；standalone `rustc --edition 2021 --test zircon_runtime/src/tests/runtime_absorption/ecs_kernel_data.rs` 1/1 passed；scoped diff/conflict checks clean（仅 LF-to-CRLF warning）。该守卫只封住文档漂移；entity/observer/command/messages/change_tick/ecs Cargo gates 仍保持 pending。 |
| M4 | First-stage event update 守卫同步 | mirror_docs_static_passed_cargo_pending | 2026-06-15 | `ecs_kernel_data_boundary` 与 `runtime_absorption::ecs_kernel_data::runtime_08_ecs_kernel_data_mirror_docs_match_structure_audit_counts` 现在锁定 `first_stage_updates_all_registered_event_channels`，要求 `EventStore::update_all()` 的 First-stage 统一推进语义留在 Runtime 08 结构审计中；当前审计报告 `event_message_anchors = 12/12`、`runtime_08_guard_anchors = 21/21`、`behavior_test_anchor_count = 16`、`missing_behavior_test_anchors = []`、`missing_event_message_anchors = []`、`missing_test_anchors = []` 与 `risks = []`。验证：rustfmt check、Python py_compile、direct `ecs_kernel_data_boundary_audit`、standalone ecs_kernel_data 1/1、standalone status-output 2/2；entity/observer/command/messages/change_tick/ecs Cargo gates 仍 pending。 |
| M4 | ECS 行为测试锚审计同步 | mirror_docs_static_passed_cargo_pending | 2026-06-15 | `ecs_kernel_data_boundary` 现在把 Runtime 08 M1/M2/M3 的 16 个 ECS 行为测试锚从 20 项 guard/test 总锚点中拆出单独计数，当前 `behavior_test_anchor_count = 16`、`missing_behavior_test_anchors = []`、`runtime_08_guard_anchors = 21/21`、`doc_anchors = 13/13` 与 `risks = []`；`runtime_absorption::ecs_kernel_data::runtime_08_ecs_kernel_data_mirror_docs_match_structure_audit_counts` 要求本计划、runtime index、`docs/zircon_runtime/scene/ecs.md`、M0 review 与 runtime-interface convergence 都记录同一组行为锚事实。验证：rustfmt check、Python py_compile、direct `ecs_kernel_data_boundary_audit`、standalone ecs_kernel_data 1/1、standalone status-output 2/2；entity/observer/command/messages/change_tick/ecs Cargo gates 仍 pending。 |
| 横切 | ECS 数据面 current audit recheck | ecs_kernel_data_current_audit_static_passed_cargo_pending | 2026-06-20 | 本轮复核 Runtime 08 当前 ECS 数据面结构事实并纳入 archetype/component/entity/event/message/resource/resource-store/component-storage/observer/commands facade/change-detection owner folder split：`ecs_kernel_data_boundary_audit` 报告 source files 69/69、guard/test files 8/8、archetype anchors 15/15、storage anchors 9/9、component-storage private re-export anchors 9/9、unexpected component-storage private re-exports 0、component identity anchors 18/18、entity lifecycle anchors 10/10、observer anchors 8/8、deferred command anchors 11/11、event/message anchors 12/12、resource identity anchors 12/12、change tick anchors 6/6、Runtime 08 guard anchors 21/21、behavior-test anchors 16/16、doc anchors 13/13、pending Cargo gate anchors 6/6、`mirror_docs_guard_present = true`、`risks = []`。验证通过：Python py_compile、direct `ecs_kernel_data_boundary_audit` risks=[]、standalone `ecs_kernel_data.rs` 1/1；entity/observer/command/messages/change_tick/ecs Cargo gates 仍 pending。 |
| 横切 | Runtime 08 ECS source/test inventory split | ecs_kernel_data_source_inventory_split_static_passed_cargo_deferred_tests_deferred | 2026-06-21 | 状态锚 `ecs_kernel_data_source_inventory_split_static_passed_cargo_deferred_tests_deferred`；`ecs_kernel_data_source_inventory.py` 现在持有 `RUNTIME_08_SOURCE_FILES`、`RUNTIME_08_TEST_FILES`、`EXPECTED_SOURCE_FILE_COUNT = 69`、`EXPECTED_TEST_FILE_COUNT = 8` 与 mirror-doc guard 名称；`ecs_kernel_data_boundary.py` 保持审计编排与 domain anchor 逻辑，拆分后 727 行，direct audit 仍报告 source files 69/69、guard/test files 8/8、`mirror_docs_guard_present = true`、`risks = []`。验证：Python py_compile、direct `ecs_kernel_data_boundary_audit` risks=[]、standalone `ecs_kernel_data.rs` 1/1、standalone `plan_status.rs` 通过；外部 HZB/render Cargo lane 活跃，本切片不启动 package-level Cargo，不提升 entity/observer/command/messages/change_tick/ecs gates。 |
| 横切 | Runtime 08 ECS anchor inventory split | ecs_kernel_data_anchor_inventory_split_static_passed_cargo_deferred_tests_deferred | 2026-06-21 | 状态锚 `ecs_kernel_data_anchor_inventory_split_static_passed_cargo_deferred_tests_deferred`；`ecs_kernel_data_anchor_inventory.py` 现在持有 Runtime 08 archetype/storage/component/entity/observer/command/event-message/resource/change-tick/test/doc/Cargo gate anchor inventory；`ecs_kernel_data_boundary.py` 只保留审计读取、缺失计算、风险聚合与 Markdown 渲染，拆分后 497 行。direct audit 仍报告 `archetype_anchor_count = 15`、storage 9、component identity 18、entity lifecycle 10、observer 8、command 11、event/message 12、resource identity 12、change tick 6、`behavior_test_anchor_count = 16`、doc anchors 13/13、Cargo gate anchors 6/6、source files 69/69、guard/test files 8/8、`risks = []`。验证：Python py_compile、direct `ecs_kernel_data_boundary_audit` risks=[]、rustfmt touched plan-status guards、standalone `ecs_kernel_data.rs` 1/1、standalone `plan_status.rs` 33/33；外部 render/editor Cargo lanes 活跃，本切片不启动 package-level Cargo，不提升 entity/observer/command/messages/change_tick/ecs gates。 |
| 横切 | Runtime 08 ECS markdown renderer split | ecs_kernel_data_markdown_split_static_passed_cargo_deferred_tests_deferred | 2026-06-21 | 状态锚 `ecs_kernel_data_markdown_split_static_passed_cargo_deferred_tests_deferred`；`ecs_kernel_data_markdown.py` now owns `render_ecs_kernel_data_boundary_markdown`; `ecs_kernel_data_boundary.py` now owns audit read, missing-anchor calculation, and risk aggregation at 344 lines, while the Markdown owner is 154 lines. Direct audit reports source files 69/69, guard/test files 8/8, `archetype_anchor_count = 15`, storage 9, component-storage private re-export anchors 9, unexpected component-storage private re-exports 0, component identity 18, entity lifecycle 10, observer 8, command 11, event/message 12, resource identity 12, change tick 6, `behavior_test_anchor_count = 16`, doc anchors 13/13, Cargo gate anchors 6/6, `mirror_docs_guard_present = true`, and `risks = []`. Validation: Python py_compile, direct `ecs_kernel_data_boundary_audit` risks=[], standalone `ecs_kernel_data.rs` 1/1, standalone `plan_status.rs` 33/33; entity/observer/command/messages/change_tick/ecs Cargo gates remain deferred while external compile lanes are active. |
| 横切 | Runtime 08 QueryState Markdown renderer split | ecs_query_state_markdown_split_static_passed_cargo_deferred_tests_deferred | 2026-06-21 | 状态锚 `ecs_query_state_markdown_split_static_passed_cargo_deferred_tests_deferred`；`ecs_query_state_markdown.py` now owns `render_ecs_query_state_boundary_markdown`; `ecs_query_state_boundary.py` now owns QueryState owner-module audit, root budget checks, forbidden-root behavior scan, and risk aggregation at 141 lines, while the Markdown owner is 33 lines. Direct audit reports the legacy `query_state.rs` absent, owner modules 9/9, `root_non_empty_lines = 84/180`, missing/unexpected/oversized owner modules all empty, and `risks = []`. Validation: Python py_compile, direct `ecs_query_state_boundary_audit` risks=[], standalone `ecs_query_structure.rs` 11/11, standalone `plan_status.rs` 33/33; entity/observer/command/messages/change_tick/ecs Cargo gates remain deferred while external compile lanes are active. |
| 横切 | Runtime 08 F17 entity path lookup verb rename | runtime_08_entity_path_lookup_getter_rename_coremin_check_passed | 2026-06-22 | 状态锚 `runtime_08_entity_path_lookup_getter_rename_coremin_check_passed`；F17 entity path Option lookup verb rename 已将 `World::get_entity_by_path(&EntityPath) -> Option<EntityId>` 作为 scene/world 路径查找公开入口，按 E4 规则让 Option-returning lookup 使用 `get_*`，old resolve-verb entity path method absent。runtime animation sequence 与 first-party animation runtime plugin 调用点均改用 `get_entity_by_path(...)`，没有保留旧名 shim、alias 或转发。新增结构守卫 `review_f17_entity_path_option_lookup_uses_get_verb`，并让 `property_paths.rs` 锁定新签名和旧公开签名缺席。验证：scoped rustfmt、F17 structure guard、status-output guard、runtime core-min `cargo check` 与 `zircon_plugin_animation_runtime` package `cargo check` 通过；Runtime 08 entity/observer/command/messages/change_tick/ecs 行为 gates 仍 pending。 |
| 横切 | Runtime 08 F5 world typed mutation errors | world_typed_mutation_errors_coremin_check_passed_partial | 2026-06-22 | 状态锚 `world_typed_mutation_errors_coremin_check_passed_partial`；F5 world typed mutation errors 切片把 `scene/world/error.rs` 作为 `SceneError` / `SceneResult` owner，并将 `World::spawn`、`spawn_at`、`insert_bundle`、`insert`、`remove` 与 `Bundle::insert_into(...)` 的公共 typed mutation surface 从裸 `Result<_, String>` 收敛为 `SceneResult`；缺失实体现在返回 `SceneError::MissingEntity`，storage error 通过 `SceneError::Storage(#[from] StorageError)` 保留 source，`DeferredCommandError` 只在 deferred report 边界 `error.to_string()`。本轮继续把 `scene/world/{component_access,hierarchy,query,records}.rs` 的固定 world mutation façade 改为 `SceneResult`，并在 `property_access/write.rs`、dynamic-scene spawn、fixed reflection adapters 与 script gameplay host 字符串合同边界显式降级为 `error.to_string()`。新增行为锚 `world_typed_mutation_errors_report_missing_entities_as_scene_errors` 与结构守卫 `review_f5_world_spawn_bundle_surface_uses_scene_error`。验证状态：F5 结构守卫 1/1、F5 behavior 1/1 与 `cargo check -p zircon_runtime --lib --no-default-features --features core-min` 通过；`dynamic_components` String error 与 `DynamicSceneError::WorldMutation(String)` 桥已由下一行 `dynamic_component_typed_errors_coremin_check_passed` 关闭，Runtime 08 entity/observer/command/messages/change_tick/ecs 行为 Cargo gates 仍 pending。 |
| 横切 | Runtime 08 F5 dynamic component typed errors | dynamic_component_typed_errors_coremin_check_passed | 2026-06-22 | 状态锚 `dynamic_component_typed_errors_coremin_check_passed`；`SceneError` 新增 `Reflect` source、component type prefix/duplicate、unregistered dynamic component、`SceneError::PluginComponentsActive` plugin-active unload、dynamic property path/write/object/descriptor/editability 变体；`ComponentTypeRegistry::register(...)`、`World::register_component_type(...)`、`set_dynamic_component(...)`、`remove_dynamic_component(...)`、`ensure_plugin_components_can_unload(...)` 与 `set_dynamic_component_property(...)` 改返 `SceneResult`，不再在 world owner 内部 `Err(format!())` 或 `error.to_string()`。`DynamicSceneError::WorldMutation(SceneError)` 改为 source-preserving world mutation 变体（代码形态 `WorldMutation(#[from] SceneError)`），`scene/dynamic_scene/scene/spawn.rs` 通过 `?` 保留 world mutation source；property access、reflection adapter、plugin registry 与 gameplay script host 在各自外部字符串合同边界显式分类或 stringify。新增行为锚 `dynamic_component_mutation_errors_report_scene_error_variants`、`dynamic_scene_world_mutation_preserves_scene_error_source` 与结构守卫 `review_f5_dynamic_component_errors_preserve_scene_error_sources`。验证：scoped rustfmt、F5 dynamic 结构守卫、status-output 守卫与 core-min `cargo check` 通过；聚焦 behavior `cargo test` 构建 15m 超时无测试结果、不计通过；Runtime 08 entity/observer/command/messages/change_tick/ecs 行为 Cargo gates 仍 pending。 |
| 横切 | Runtime 08 ecs_events_messages Cargo 验证窗口探测 | cargo_recheck_timeout_no_result | 2026-06-20 | `cargo test -p zircon_runtime --lib ecs_events_messages --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-08-ecs-events-messages-0620 --message-format short --color never -- --test-threads=1 --nocapture` ended with 1200s timeout no result（1200s 工具窗口超时，未产生测试结果）；residual target-dir processes stopped：残留在该 target-dir 的 cargo/rustc 进程已停止。该窗口只记录 no-result timeout，不提升 event/message/resource-store owner split 或 entity/observer/command/messages/change_tick/ecs Cargo gates。 |

基线数值（开工首日记录）：

- `scene/ecs/` 条目基线：40（2026-06-12 ls；含新增 `system_set.rs` 时当前文件树仍由本计划按 40 项口径跟踪）
- 观察者入口基线：3 类（`observer/store.rs` 的 `observe_lifecycle` / `observe_event` / `observe_entity_event`）
- 存储形态基线：`StorageType::{Table,SparseSet}`，`storage/component_storage/` folder-backed owner set 实现 table + sparse 双 backing store，并由 `store.rs` 暴露稳定 `ComponentStorage` facade（2026-06-20 重核）
- 结构审计基线：`ecs_kernel_data_boundary` 当前报告 `expected_source_file_count = 69`、`expected_test_file_count = 8`、`archetype_anchors = 15/15, storage_anchors = 9/9`、`component_storage_private_reexport_anchors = 9/9`、`unexpected_component_storage_private_reexports = []`、`component_identity_anchors = 18/18`、`entity_lifecycle_anchors = 10/10`、`observer_anchors = 8/8`、`deferred_command_anchors = 11/11`、`event_message_anchors = 12/12`、`resource_identity_anchors = 12/12`、`change_tick_anchors = 6/6`、`runtime_08_guard_anchors = 21/21`、`behavior_test_anchor_count = 16`、`missing_behavior_test_anchors = []`、`doc_anchors = 13/13`、`pending_cargo_gate_anchors = 6/6`、`mirror_docs_guard_present = true`、`risks = []`；它不替代 Cargo gate。
- `cargo test -p zircon_runtime --lib ecs --locked` 通过数基线：未记录；当前有其他 Cargo/rustc lanes 活跃，且上一轮 runtime 03 Cargo 被无关 UI test import 阻断，未重新启动 Cargo。Runtime 08 的 entity/observer/command/messages/change_tick/ecs filters 待验证状态由 `runtime_08_ecs_kernel_cargo_pending_gate_stays_explicit_until_ecs_validation` 锁定。

## 风险与协调

- 与 03（schedule_runner）、07（query_state/change_detection 计数）共享文件区：三计划的切片在同文件上错峰，先开工者在状态节登记占用。
- 10fps 会话 worktree 改动禁止回退；`scene/ecs/**` 动工前逐文件 `git status`。
- M1 若裁决补 generation 是行为级改动（实体句柄表示变化），波及序列化与 dynamic_api 的实体 ID 传递——执行前与 10 计划（ABI 面）对齐实体 ID 的 ABI 表示。
- 所有"债证明"锚定测试在修复切片落地时必须同步翻转断言方向（硬切换，不留过期锚）。
