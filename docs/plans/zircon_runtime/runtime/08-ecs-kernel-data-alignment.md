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
  - zircon_runtime/src/scene/ecs/storage/component_storage/component_results.rs
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
  - zircon_runtime/src/scene/ecs/observer/callback_registry.rs
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
  - zircon_runtime/src/core/framework/scene/resource.rs
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
  - zircon_runtime/src/tests/runtime_absorption/ecs_kernel_data/inventory.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates/early/runtime_08.rs
  - tools/tests/test_runtime_ecs_kernel_data_audit.py
  - tests/acceptance/runtime-ecs-kernel-data-audit-owner-sync.md
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
last_refined: 2026-07-12
---

# 08 ECS 内核数据面对齐

2026-07-10 Runtime 08 当前 child-owner 同步：`ecs_kernel_data_boundary` 报告 `expected_source_file_count = 69`、`expected_test_file_count = 10`、`archetype_anchors = 15/15`、`storage_anchors = 9/9`、`component_storage_private_reexport_anchors = 9/9`、`component_identity_anchors = 18/18`、`entity_lifecycle_anchors = 10/10`、`observer_anchors = 8/8`、`deferred_command_anchors = 11/11`、`event_message_anchors = 12/12`、`resource_identity_anchors = 12/12`、`change_tick_anchors = 6/6`、`runtime_08_guard_anchors = 21/21`、`behavior_test_anchor_count = 16`、`missing_behavior_test_anchors = []`、`doc_anchors = 13/13`、`pending_cargo_gate_anchors = 6/6`、`mirror_docs_guard_present = true` 与 `risks = []`；`runtime_08_ecs_kernel_data_mirror_docs_match_structure_audit_counts` 保持计划、runtime index、ECS 模块文档、M0 review 与 interface-convergence 镜像一致。10 个 test owner 显式包含 `ecs_kernel_data/inventory.rs` 与 `cargo_gates/early/runtime_08.rs`；该口径取代历史 8-route-owner 镜像，但不关闭 pending `entity/observer/command/messages/change_tick/ecs` Cargo gates。

与子计划 03 的分工：**03 管调度与帧循环**（stage/fixed-step/并行执行），**本计划管数据面内核**——实体生命周期、组件存储、观察者/事件、命令队列、变更追踪的语义定稿与 `bevy_ecs` 逐项对照。性能计数与优化归 07。

## 现状与证据（2026-06-12 实仓盘点）

- 模块齐件（`scene/ecs/` 当前实测 30 个顶层条目，2026-06-20 archetype/component/entity/event/message/resource/resource-store/component-storage/observer/commands facade/change-detection owner hard-cut 后 archetype owner 为 `archetype/{mod,id,index,move_result,record,signature}.rs` 子树，component identity owner 为 `component/{mod,marker,id,registry}.rs` 子树，entity identity owner 为 `entity/{mod,despawned,error,internal,location,registry,slot,stable_location}.rs` 子树，事件 owner 为 `events/{mod,cursor,id,metrics,queue,store,subscription}.rs` 子树，message owner 为 `messages/{mod,cursor,id,queue,store}.rs` 子树，resource identity owner 为 `resource/{mod,marker,id,registry}.rs` 子树，resource store owner 为 `resource_store/{mod,stored_resource,store}.rs` 子树，component storage owner 为 `storage/component_storage/{mod,component_results,entry,location,sparse,store,table}.rs` 子树，observer owner 为 `observer/{mod,callback_registry,callbacks,entry,id,store}.rs` 子树，commands facade owner 为 `commands/commands/{mod,entity_commands,facade,param}.rs` 子树，change detection owner 为 `change_detection/{mod,change_tick,change_tick_window,component_ticks,stats,wrappers}.rs` 子树）：archetype 族（`archetype/`）、实体族（`entity/`）、存储（`component/` + `storage/component_storage/` + `storage/{component_remove_result.rs,storage_error.rs}` + `storage_type.rs`）、`bundle.rs`、`commands/{command.rs,command_queue.rs,commands/{mod,entity_commands,facade,param}.rs}`、`change_detection/{mod,change_tick.rs,change_tick_window.rs,component_ticks.rs,stats.rs,wrappers.rs}`、`observer/`、`events/` + `messages/` 双通道、`resource/` + `resource_store/`、`removal.rs`、查询族（`query/` 含 `query_state/`、`cached_query_iter.rs`、combinations/many/unique 迭代器，见 07 计划盘点）。
- **观察者实测**（`observer/{id,store,entry,callbacks,callback_registry}.rs`）：`ObserverId(u64)` + `ObserverStore`，三类观察入口——`observe_lifecycle`、`observe_event<E>`、`observe_entity_event<E>`、`remove`。与 bevy `Observer`/component hooks 的触发时机语义（立即 vs 队列末）需对照定稿。
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

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

- 迁入记录：[`08/2026-07-09-ecs-kernel-data-alignment-output-records.md`](08/2026-07-09-ecs-kernel-data-alignment-output-records.md)
