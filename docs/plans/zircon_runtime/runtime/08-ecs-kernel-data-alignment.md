---
related_code:
  - zircon_runtime/src/scene/ecs/mod.rs
  - zircon_runtime/src/scene/ecs/storage/component_storage.rs
  - zircon_runtime/src/scene/ecs/storage_type.rs
  - zircon_runtime/src/scene/ecs/entity_registry.rs
  - zircon_runtime/src/scene/ecs/despawned_entity.rs
  - zircon_runtime/src/scene/ecs/stable_entity_location.rs
  - zircon_runtime/src/scene/ecs/observer.rs
  - zircon_runtime/src/scene/ecs/commands/command_queue.rs
  - zircon_runtime/src/scene/ecs/change_detection/component_ticks.rs
  - zircon_runtime/src/scene/ecs/events.rs
  - zircon_runtime/src/scene/ecs/messages.rs
  - zircon_runtime/src/scene/ecs/resource_store.rs
  - zircon_runtime/src/scene/ecs/bundle.rs
  - zircon_runtime/src/scene/ecs/query/query_state/stats.rs
  - zircon_runtime/src/tests/runtime_absorption/ecs_kernel_data.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_query_state_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_kernel_data_boundary.py
  - dev/bevy/crates/bevy_ecs/src
plan_sources:
  - docs/plans/zircon_runtime/runtime/index.md
status: in_progress
last_refined: 2026-06-14
---

# 08 ECS 内核数据面对齐

与子计划 03 的分工：**03 管调度与帧循环**（stage/fixed-step/并行执行），**本计划管数据面内核**——实体生命周期、组件存储、观察者/事件、命令队列、变更追踪的语义定稿与 `bevy_ecs` 逐项对照。性能计数与优化归 07。

## 现状与证据（2026-06-12 实仓盘点）

- 模块齐件（`scene/ecs/` 实测 40 条目）：archetype 族（`archetype_id/index/signature.rs`）、实体族（`entity_registry.rs` + `entity_registry_error.rs` + `despawned_entity.rs` + `stable_entity_location.rs` + `entity_location.rs` + `internal_entity.rs`）、存储（`storage/{component_storage.rs,component_remove_result.rs,storage_error.rs}` + `storage_type.rs`）、`bundle.rs`、`commands/{command.rs,command_queue.rs,commands.rs}`、`change_detection/{change_tick.rs,change_tick_window.rs,component_ticks.rs,wrappers.rs}`、`observer.rs`、`events.rs` + `messages.rs` 双通道、`resource.rs/resource_id.rs/resource_registry.rs/resource_store.rs`、`removal.rs`、查询族（`query/` 含 `query_state/`、`cached_query_iter.rs`、combinations/many/unique 迭代器，见 07 计划盘点）。
- **观察者实测**（`observer.rs`）：`ObserverId(u64)` + `ObserverStore`，三类观察入口——`observe_lifecycle`（:54）、`observe_event<E>`（:70）、`observe_entity_event<E>`（:90）、`remove`（:112）。与 bevy `Observer`/component hooks 的触发时机语义（立即 vs 队列末）需对照定稿。
- **变更追踪同形**：`change_tick/change_tick_window/component_ticks/wrappers` 四件与 `bevy_ecs::change_detection`（Tick/ComponentTicks/Ref-Mut wrappers）形状对应；tick 回绕（wrap-around）语义是否处理需核验。
- **存储形态重核修正**：`storage_type.rs` 已是 `StorageType::{Table,SparseSet}` 双形态枚举，`component_storage.rs` 同时持有 table rows 与 sparse entries。与 bevy 的差异是文件拓扑和统一 owner，而不是缺少 SparseSet。
- **事件双通道**：`events.rs` 与 `messages.rs` 并存——两者语义分工（帧内事件 vs 跨帧消息？double-buffer 清理策略？）未文档化，是本计划裁决项。
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
   - `ls zircon_runtime/src/scene/ecs/`（核 40 条目清单）
   - `grep -n "pub enum\|pub struct" zircon_runtime/src/scene/ecs/storage_type.rs`（已核：`StorageType::{Table,SparseSet}`）
   - `grep -n "generation\|Generation" zircon_runtime/src/scene/ecs/entity_registry.rs`（核 ID 重用语义现状）
4. 基线记录：`cargo test -p zircon_runtime --lib ecs --locked` 通过数记入状态节。

## 里程碑

### M0 bevy_ecs 数据面差距表

#### 切片 0.1 五维对照表

- 目标文件：`docs/zircon_runtime/scene/ecs.md`（执行时核验存在性：`ls docs/zircon_runtime/scene/`；有则扩展、无则新建并挂 `docs/zircon_runtime/` 索引）。
- 改动形态：纯文档。五维逐项对照，已知行预填：

  | 维度 | bevy_ecs 锚点 | 本仓对应物 | 待裁决问题 |
  |---|---|---|---|
  | 存储形态 | `storage/{table,sparse_set}.rs` 双形态 | `storage_type.rs` 双形态枚举 + `storage/component_storage.rs` table/sparse 双 backing store | 当前能力保留；是否拆文件或进一步优化必须等 07 计数证据，不把文件集中形态误判为功能债 |
  | 实体分配/重用 | `entity/mod.rs` generation 复用 | `entity_registry.rs` + `despawned_entity.rs` + `stable_entity_location.rs` | generation 等价物是否存在；despawn 后旧句柄访问行为 |
  | 观察者 | `observer/` + component hooks | `observer.rs` 三类观察（lifecycle/event/entity_event） | 触发时机（立即 vs apply_deferred 后）；观察者内再触发的递归语义 |
  | 事件 | `event/` double-buffer + 显式清理 | `events.rs` + `messages.rs` 双通道 | 双通道分工判词；滞留事件清理策略（prune 在 event_bus 是 core 层，ECS 层呢） |
  | 命令队列 | `Commands` + 队列冲刷点 | `commands/command_queue.rs` + runner 的 apply_deferred | 冲刷点已显式（03 盘点）；EntityCommands 等价面盘点 |

- 调用方迁移：无。
- 验收：五行全部有"对应物 + 差异 + 裁决"；裁决无"待定"。
- DoD：差距表落 `ecs.md`，需修债项进 M1–M3 工作集，"保留差异"项写明理由。

#### M0 测试阶段（milestone-first）

- 纯审计：`git status --porcelain` 仅 docs 变更。

### M1 实体生命周期语义测试完备

#### 切片 1.1 despawn / 重用 / stable location 测试矩阵

- 目标文件：`zircon_runtime/src/scene/tests/`（既有 ecs 测试树，落点执行时定：`ls zircon_runtime/src/scene/tests/`）；若 M0 发现 generation 缺失且裁决为债，修改点在 `entity_registry.rs` + `despawned_entity.rs`。
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

- 目标文件：`observer.rs` 消费点 + `scene/tests/`（观察者测试位执行时核验：Grep `ObserverStore`，path `zircon_runtime/src/scene`）。
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

- 目标文件：`events.rs`、`messages.rs` + `ecs.md`（分工判词）。
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
| M3 | 3.1 双通道定稿 | code_complete_pending_cargo | 2026-06-12 | 已补 `events_require_explicit_update_and_keep_next_queue_hidden`、`clear_events_prunes_current_and_next_event_queues`、`messages_are_retained_until_explicit_clear_independent_of_event_updates`、`event_and_message_clear_boundaries_do_not_cross_channels`；裁决 events=current/next 帧推进，messages=带 id 的显式保留缓冲；Cargo 待活动 lanes 清空后运行 |
| M3 | 3.2 tick 回绕 | code_complete_pending_cargo | 2026-06-12 | 已补 `ChangeTick::next()` 回绕递增、`ChangeTick::relative_to(...)` 回绕差值、`ChangeTick::is_newer_than(...)` 相对年龄比较、`ChangeTickWindow::new(...)` stale `last_run` 截断，以及 `change_tick_comparison_survives_wraparound`、`tick_window_clamps_stale_ticks`；Cargo 待活动 lanes 清空后运行 |
| M4 | QueryState owner 审计同步 | structure_audit_static_passed_cargo_pending | 2026-06-13 | `ecs_query_state_boundary` 已把 `query_state/stats.rs` 归类为 Runtime 07 telemetry sidecar，而不是 ECS 数据面异常 owner module；当前结构审计事实为 `expected_module_count = 8`、`unexpected_modules = []`、`risks = []`，并与 `docs/zircon_runtime/scene/ecs/query_state.md` 的 Boundary Rules 保持一致。Cargo 验证状态仍由本计划验证门守卫统一保持 pending。 |
| M4 | 验证门守卫 | cargo_validation_pending_guarded | 2026-06-13 | 新增 `runtime_absorption::plan_status::runtime_08_ecs_kernel_cargo_pending_gate_stays_explicit_until_ecs_validation`，要求本计划保持 `in_progress`，M1/M2/M3 行继续保留 `code_complete_pending_cargo` 与 Cargo gate，直到 `cargo test -p zircon_runtime --lib entity --locked -- --nocapture`、`cargo test -p zircon_runtime --lib observer --locked -- --nocapture`、`cargo test -p zircon_runtime --lib command --locked -- --nocapture`、`cargo test -p zircon_runtime --lib messages --locked`、`cargo test -p zircon_runtime --lib change_tick --locked -- --nocapture` 与 `cargo test -p zircon_runtime --lib ecs --locked` 有真实通过记录。 |
| M4 | ECS 数据面结构审计镜像 | structure_audit_static_passed_cargo_pending | 2026-06-14 | 新增 `ecs_kernel_data_boundary` 并接入 `audit_runtime_structure.py`；当前结构审计事实为 `expected_source_file_count = 20`、`expected_test_file_count = 7`、`storage_anchors = 9/9`、`entity_lifecycle_anchors = 10/10`、`observer_anchors = 8/8`、`deferred_command_anchors = 11/11`、`event_message_anchors = 11/11`、`change_tick_anchors = 6/6`、`runtime_08_guard_anchors = 17/17`、`doc_anchors = 7/7`、`pending_cargo_gate_anchors = 6/6`、`mirror_docs_guard_present = true`、`risks = []`。这仍是静态结构证据；entity/observer/command/messages/change_tick/ecs Cargo filters 继续 pending。 |
| M4 | ECS 数据面镜像文档守卫 | mirror_docs_static_passed_cargo_pending | 2026-06-14 | 新增 `runtime_absorption::ecs_kernel_data::runtime_08_ecs_kernel_data_mirror_docs_match_structure_audit_counts`，锁定 `docs/zircon_runtime/scene/ecs.md`、本计划、runtime index、M0 review 与 runtime-interface convergence 均同步记录 `ecs_kernel_data_boundary` 的同一组结构事实：`expected_source_file_count = 20`、`expected_test_file_count = 7`、`storage_anchors = 9/9`、`entity_lifecycle_anchors = 10/10`、`observer_anchors = 8/8`、`deferred_command_anchors = 11/11`、`event_message_anchors = 11/11`、`change_tick_anchors = 6/6`、`runtime_08_guard_anchors = 17/17`、`doc_anchors = 7/7`、`pending_cargo_gate_anchors = 6/6`、`mirror_docs_guard_present = true`、`risks = []`。该守卫只封住文档漂移；entity/observer/command/messages/change_tick/ecs Cargo gates 仍保持 pending。 |

基线数值（开工首日记录）：

- `scene/ecs/` 条目基线：40（2026-06-12 ls；含新增 `system_set.rs` 时当前文件树仍由本计划按 40 项口径跟踪）
- 观察者入口基线：3 类（observer.rs:54/:70/:90）
- 存储形态基线：`StorageType::{Table,SparseSet}`，`ComponentStorage` 同文件内实现 table + sparse 双 backing store（2026-06-12 重核）
- 结构审计基线：`ecs_kernel_data_boundary` 当前报告 `expected_source_file_count = 20`、`expected_test_file_count = 7`、`storage_anchors = 9/9`、`entity_lifecycle_anchors = 10/10`、`observer_anchors = 8/8`、`deferred_command_anchors = 11/11`、`event_message_anchors = 11/11`、`change_tick_anchors = 6/6`、`runtime_08_guard_anchors = 17/17`、`doc_anchors = 7/7`、`pending_cargo_gate_anchors = 6/6`、`mirror_docs_guard_present = true`、`risks = []`；它不替代 Cargo gate。
- `cargo test -p zircon_runtime --lib ecs --locked` 通过数基线：未记录；当前有其他 Cargo/rustc lanes 活跃，且上一轮 runtime 03 Cargo 被无关 UI test import 阻断，未重新启动 Cargo。Runtime 08 的 entity/observer/command/messages/change_tick/ecs filters 待验证状态由 `runtime_08_ecs_kernel_cargo_pending_gate_stays_explicit_until_ecs_validation` 锁定。

## 风险与协调

- 与 03（schedule_runner）、07（query_state/change_detection 计数）共享文件区：三计划的切片在同文件上错峰，先开工者在状态节登记占用。
- 10fps 会话 worktree 改动禁止回退；`scene/ecs/**` 动工前逐文件 `git status`。
- M1 若裁决补 generation 是行为级改动（实体句柄表示变化），波及序列化与 dynamic_api 的实体 ID 传递——执行前与 10 计划（ABI 面）对齐实体 ID 的 ABI 表示。
- 所有"债证明"锚定测试在修复切片落地时必须同步翻转断言方向（硬切换，不留过期锚）。
