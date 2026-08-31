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
  - zircon_runtime/src/scene/ecs/storage/component_storage/sparse/locator.rs
  - zircon_runtime/src/scene/ecs/storage/component_storage/store.rs
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
  - zircon_runtime/src/scene/world/property_access/entries.rs
  - zircon_runtime/src/scene/world/property_access/entries/camera.rs
  - zircon_runtime/src/scene/world/property_access/entries/mesh.rs
  - zircon_runtime/src/scene/world/property_access/entries/lighting.rs
  - zircon_runtime/src/scene/world/property_access/entries/animation.rs
  - tools/tests/test_runtime_scene_property_entry_owner_structure.py
  - zircon_runtime/src/animation/sequence/compiled.rs
  - zircon_runtime/src/animation/sequence/target.rs
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
last_refined: 2026-08-01
---

# 08 ECS 内核数据面对齐

Runtime 08 current hard-cut sync (2026-08-28): `ecs_kernel_data_boundary` now owns `expected_source_file_count = 77`; the inventory includes `component/registry/transferred.rs` and `component_storage/sparse/locator.rs` as explicit child owners, retains the six-file archetype-table owner plus `typed_api/{component_row,projection_rebuild}.rs`, and keeps `expected_test_file_count = 10`. This supersedes earlier current-count paragraphs while preserving their dated historical evidence.

2026-08-01 Runtime 08 当前 child-owner 同步：`ecs_kernel_data_boundary` 报告 `expected_source_file_count = 69`、`expected_test_file_count = 10`、`archetype_anchors = 15/15`、`storage_anchors = 9/9`、`component_storage_private_reexport_anchors = 9/9`、`component_identity_anchors = 18/18`、`entity_lifecycle_anchors = 10/10`、`observer_anchors = 8/8`、`deferred_command_anchors = 11/11`、`event_message_anchors = 12/12`、`resource_identity_anchors = 12/12`、`change_tick_anchors = 6/6`、`runtime_08_guard_anchors = 21/21`、`behavior_test_anchor_count = 16`、`missing_behavior_test_anchors = []`、`doc_anchors = 13/13`、`pending_cargo_gate_anchors = 6/6`、`mirror_docs_guard_present = true` 与 `risks = []`。archetype owner 已以已知 row 的 `remove_entity_at(...)` 取代退役的线性 `entity_row(...)` 扫描锚点；两条已删除的 plugin `sequence/{apply,target}.rs` 路径也已从 frontmatter 移除。该静态口径不关闭 pending `entity/observer/command/messages/change_tick/ecs` 受管 Cargo gates。

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
   - `Get-ChildItem zircon_runtime/src/scene/ecs/`（核当前 30 个顶层条目清单）
   - `Select-String -Path zircon_runtime/src/scene/ecs/storage_type.rs -Pattern 'pub enum','pub struct'`（已核：`StorageType::{Table,SparseSet}`）
   - `Select-String -Path zircon_runtime/src/scene/ecs/entity/slot.rs -Pattern 'generation','Generation'`（核 ID 重用语义现状）
4. 基线记录：通过 `validate-matrix.ps1` 的 `zircon_runtime/ecs` 受管过滤门记录通过数；禁止直接运行裸 Cargo。

## 里程碑

### M0 bevy_ecs 数据面差距表

#### 切片 0.1 五维对照表

- 目标文件：`docs/zircon_runtime/scene/ecs.md`（执行时核验存在性：`ls docs/zircon_runtime/scene/`；有则扩展、无则新建并挂 `docs/zircon_runtime/` 索引）。
- 改动形态：纯文档。五维逐项对照，已知行预填：

  | 维度 | bevy_ecs 锚点 | 本仓对应物 | 待裁决问题 |
  |---|---|---|---|
  | 存储形态 | `storage/{table,sparse_set}.rs` 双形态 | `storage_type.rs` 双形态枚举 + `storage/component_storage/{store,table,sparse,location}.rs` table/sparse 双 backing store | 当前能力保留；是否进一步优化必须等 07 计数证据，不把统一 public facade 误判为功能债 |
  | 实体分配/重用 | `entity/mod.rs` generation 复用 | `entity/{mod,despawned,error,internal,location,registry,slot,stable_location}.rs` | generation 等价物是否存在；despawn 后旧句柄访问行为 |
  | 观察者 | `observer/` + component hooks | `observer/{store,entry,callbacks,id,callback_registry}.rs` 三类观察（lifecycle/event/entity_event） | 触发时机（立即 vs apply_deferred 后）；观察者内再触发的递归语义 |
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
- DoD：受管 `entity` 过滤门全绿（含锚定测试）。

#### M1 测试阶段（milestone-first）

- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -LibTests -TestFilter entity`
- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter ecs`
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
- DoD：受管 `observer` 过滤门全绿。

#### 切片 2.2 命令队列冲刷与错误路径

- 目标文件：`commands/command_queue.rs` + 测试位。
- 改动形态：补错误路径测试——对已 despawn 实体的排队命令（insert/remove）冲刷时的行为定稿（静默跳过 vs 显式错误记录，二选一判词）；`command_queue_on_despawned_entity_target_is_reported_not_silently_dropped`（名按判词定）。
- 调用方迁移：无。
- 验收：判词 + 测试。
- DoD：受管 `command` 过滤门全绿。

#### M2 测试阶段（milestone-first）

- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter observer`
- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter command`
- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter ecs`（全族无回归）

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

- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter change_tick`
- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter messages`
- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter ecs`（收尾全族）
- 验收证据：差距表全部行闭环；`docs/zircon_runtime/scene/ecs.md` 与代码一致。

## 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

- 迁入记录：[`08/2026-07-09-ecs-kernel-data-alignment-output-records.md`](08/2026-07-09-ecs-kernel-data-alignment-output-records.md)
## Open Failure Handoffs

- fixed 已修复：[ecs-resource-marker-owner-missing](../../zircon_editor/editor/02/fixed-2026-07-14-ecs-resource-marker-owner-missing.md)
- fixed 已修复：[system-stage-owner-guard-drift](08/fixed-2026-07-14-system-stage-owner-guard-drift.md)
- open / 待修复：[scene-binding-generations-visibility](08/failure-2026-07-29-scene-binding-generations-visibility.md) 已完成最小可见性与同 ID root reuse 修复，仍需原始 managed `zircon_runtime --lib` compile gate 证明后再回传。
- 2026-07-18 scene path性能交接：framework scene 24/24确认Entity/ComponentProperty path各自重复拥有raw与segment正文，clone到animation/property/editor action及stable generation字符串resolve缺少统一identity。Runtime08需联动scene/animation consumers建立interned PathId或Arc range storage与scene-generation dense resolution cache；见PERF-MVP-329及`docs/plans/performance/01/2026-07-18-runtime-core-framework-scene-static-review.md`。
- 2026-07-22 despawn archetype定位性能同步：World原先在每个entity删除后调用`rebuild_archetype_index`全场重建；现按EntityRegistry已有archetype/row直接swap-remove并只修正swapped entity row，行为/源码守卫已落盘，归PERF-MVP-458。Runtime08后续Cargo需覆盖generation handle、lifecycle/removal event、archetype query与recursive delete，不得恢复全量refresh；hierarchy增量owner另见Runtime07 PERF-MVP-459。
- 2026-07-22 scene property path编译分派交接：候选segment临时String已止损；animation fallback仍全entity×ancestor×同名扫描，single read枚举并构造命中前entries，write每track/frame分配normalized String/Vec。PERF-MVP-329提升P0，Runtime08发布唯一PathId与world/schema-generation compiled accessor，Plugins04/Runtime animation和Editor05共同硬切；见`08/failure-2026-07-22-scene-property-path-compiled-dispatch.md`。
- 2026-07-22 World固定组件/query index交接：27类固定组件在专用HashMap与ComponentStorage双写并clone，restore逐组件产生中间archetype迁移；matched-archetype query cache miss仍为稳定顺序全扫World entities。Runtime08硬切单一storage/row authority和增量stable query-order index，分别归PERF-MVP-464/466；见`08/failure-2026-07-22-world-fixed-component-storage-and-stable-query-index.md`。
- 2026-07-22 World batch transaction交接：`insert_node_records`为原子性深clone完整World后逐record clone/插入，undo/import小批次也按全世界复制。Runtime08提供预验证batch plan、affected-row undo delta/COW storage和单次generation commit，Editor03共同验收；见PERF-MVP-467与`08/failure-2026-07-22-world-batch-mutation-clone-transaction.md`。
- 2026-08-02 Runtime08前向实施更新：`NodeRecord` 已改为预验证后移交 owned batch，单次 World generation 发布且 lifecycle 在所有记录可见后分派；dynamic scene remap 已复用该 owned batch，反射 dense field 已能构造一次候选组件/JSON 后单次发布。dynamic scene 的 preview 与 apply 现共用一次编译出的 remap、预写入 records 与 preview summary，prepared staging 不再重复解释这三项；compiled plan 已绑定 target World 与 component schema catalog generation，并在任何写入前以 typed stale error 拒绝失效 apply。component/resource adapter、dense field slot 与 remapped value 现由 compile 一次解析并以 owned plan 交给 apply；资源写事务与一次 affected-row publish 尚未完成。F5仅部分完成：`spawn_empty_at` 已改为 `SceneResult`，allocator overflow/registry 失败会传播 typed error，deferred spawn 记录失败，重复的 `spawn_at` 显式拒绝；但 `remove_entity -> bool` 与 `remove_entity_recursive -> Vec<NodeRecord>` 仍压扁 missing-entity 错误，必须与`DetachedEntityBatch`和全部调用方一起硬切。`rustfmt --check`、`git diff --check` 与 source guards 已通过；archetype row 单一所有权、资源写事务、full-World staging 删除及 managed Cargo 证据仍未完成，因此相关 failure 保持 open。
- 2026-07-22 dynamic scene compiled spawn交接：background Prepared只做schema自检，preview/apply在主线程重复remap/field物化；actual spawn逐field clone adapter/metadata并可能O(F²)，且失败留下partial World。Runtime08发布target/schema-generation compiled transaction，preview共享plan、apply一次原子commit；见PERF-MVP-472和`08/failure-2026-07-22-dynamic-scene-compiled-spawn-transaction.md`。
- 2026-07-22 dynamic scene session索引事务交接：slot/manifest线性查找、每项push/upsert/rename全量sort，selection构造完整owned manifest后再查找，preview/commit重复全档案验证；merge/retention逐项contains/sort并可深clone archive。Runtime08发布canonical slot index、generation validation ticket、borrowed selection handle和单次batch mutation plan，复用affected-row原子事务。见PERF-MVP-476和`08/failure-2026-07-22-dynamic-scene-session-indexed-transaction.md`。
- 2026-07-22 ECS Bundle事务交接：当前1..8元tuple Bundle逐组件调用`World::insert`，spawn从empty开始产生多个中间archetype/storage/event状态且失败可留下partial bundle。Runtime08以component-id/signature staging、最终row reservation和一次affected-row commit硬切，复用464单storage与467 transaction；见PERF-MVP-479和`08/failure-2026-07-22-ecs-bundle-single-archetype-transaction.md`。
- 2026-07-22 ECS columnar storage交接：archetype same-signature/move已复用known row并删除线性entity scan（PERF-MVP-480）；但当前Table仍是per-component HashMap+Vec<Box<Any>>，ArchetypeIndex另存membership，component mutation/despawn扫描全部storages且query每row hash/downcast。Runtime08硬切archetype-owned row-aligned columns与generation compiled query slots，见PERF-MVP-481和`08/failure-2026-07-22-ecs-archetype-columnar-storage.md`。
- 2026-07-22 ECS lazy change detection交接：ResourceStore replacement单次entry probe止损已完成（PERF-MVP-482）；但`Mut<T>`/`ResMut<T>`在fetch mutable access时已写changed tick，未实际修改也会触发Changed并放大下游系统。Runtime08把tick mutable authority纳入wrapper，在首次DerefMut/into_inner才mark，同时保留raw &mut和显式bypass语义；见PERF-MVP-483和`08/failure-2026-07-22-ecs-lazy-change-detection.md`。
- 2026-07-22 ECS observer indexed dispatch交接：三类observer每trigger先count再filter双扫Vec，逐命中clone Arc；entity event无(type,entity)索引，lifecycle还clone type-name/event。Runtime08发布generation-owned callback buckets和id→slot removal index；见PERF-MVP-484和`08/failure-2026-07-22-ecs-observer-indexed-dispatch.md`。
- 2026-07-22 ECS event/message lifecycle交接：Messages依赖显式clear且产品无清理调用，retained Vec可无界；Events每帧update所有注册通道。Runtime08定稿双通道语义，以cursor-aware硬预算和dirty channel scheduling收敛长会话RSS与idle CPU；见PERF-MVP-485和`08/failure-2026-07-22-ecs-event-message-bounded-lifecycle.md`。
- 2026-08-11 ECS deferred command buffer前向实施：`CommandQueue`已从每命令`Box`所有权切到64 KiB分块、64-byte对齐的packed inline arena；单payload上限192 bytes、producer-local active arena上限4 MiB。超尺寸、超对齐或超预算命令保留显式fallback，`with_capacity`可预热可复用的队列与arena backing storage。队列metadata只保留block index/offset和apply/drop函数，合并时移动完整block并重映射index，不保留可因Vec迁移失效的payload pointer。metrics区分逻辑inline字节与含padding的arena占用，并记录backing growth、fallback alloc/release、inline release、分派和panic discard；100k小命令、192-byte饱和、64-byte对齐、64 worker arena merge、panic清理及大arena小merge容量保留的回归已写入。`apply_deferred`在unwind后释放本批未消费payload，并把嵌套enqueue保留到下一窗口。`WorkerCommandBuffer`与worker-safe schedule已经按`(system_order, system_id)`在barrier前确定性合并；仍未完成的是从`CommandsParam`进入该路径的typed deferred spawn token、barrier id resolve与结构化batch发布，不能以共享`next_entity`、mutex或atomic计数冒充完成。F5收口已使`DeferredCommandError`保留原始typed `SceneError`（而非字符串）；故PERF-MVP-487 failure保持open，且未声明Cargo接受；见`08/failure-2026-07-22-ecs-deferred-command-dense-buffer.md`。
- 2026-07-23 versioned serialization消费补充：统一壳 current text/binary仍经JSON Value与多次全树遍历，DynamicScene 5k/100k实体save/load会放大CPU/RSS。Runtime08按PERF-MVP-570/571迁移到Editor11提供的header-first current direct typed/flat-node路径，只有旧schema才物化migration Value；scene generation artifact只持一个typed或sealed wire owner，不为ECS建立第二serializer/cache。现有migration、future/error、canonical bytes和binary v1 golden不变。
- open / 待修复（2026-07-27 dynamic component property generation）：reflection/direct dynamic property write 当前绕过唯一 generation/inspection 发布，需由 Runtime08 收敛 mutation boundary，并回传 Navigation typed projection gate；见 [failure](08/failure-2026-07-27-dynamic-component-property-world-generation.md)。
- 当前 pending Cargo 收口门已在 M1-M3 测试阶段显式覆盖 `entity/observer/command/messages/change_tick/ecs` 六个受管过滤词；静态 `6/6` 只证明命令锚存在，不代表这些 open failure 或性能交接已验收。

## 2026-08-28 Scene Property Entry Component Owner Split

状态：`runtime_08_15_scene_property_entry_component_owner_split_static_passed_cargo_profile_deferred`。

`scene/world/property_access/entries.rs` 从 567 行收束为 210 行，只保留基础实体属性、固定组件
域调用顺序、dynamic metadata 投影与总容量编排。camera、mesh、lighting、animation 的枚举和
容量预算分别迁入 49/122/153/175 行 child；既有 513 行 physics owner 保持不变。所有 child
仍是同一 `World` 的 `pub(super)` inherent implementation，没有第二 reflection registry、Editor
cache、property DTO 或写入路径。

结构 RED 先以旧根 567 行超过 280 行失败；迁移后 source/status guard 2/2 通过，四个投影块、
四个容量块及两个专属 helper 的 whitespace-normalized SHA-256 10/10 与 `HEAD` 基线一致。
边界参考 Unreal `FProperty`/`TFieldIterator` 与 component-owned `UPROPERTY`，并以 Fyrox scene
component domains、Bevy `TypeRegistry` 交叉检查。property 顺序、path、value、animatable 标记、
容量算法、targeted read 和 write transaction 均未改变；Cargo、Editor Inspector 产品链与
CPU/allocation/RSS/power profile 延后，不声明 Runtime08/15 acceptance 或性能收益。

## 2026-08-28 Component Registry Transfer Transaction Owner Split

状态：`runtime_08_15_component_registry_transfer_owner_split_static_passed_cargo_deferred`。

`zircon_runtime/src/scene/ecs/component/registry.rs` 从 559 行收束为 154 行，只保留 component
identity、descriptor/layout storage 与普通 Rust/dynamic registration。transferred descriptor 的
preflight/import log、冲突匹配和一次 publish 迁入 259 行
`zircon_runtime/src/scene/ecs/component/registry/transferred.rs`；六个原行为测试迁入 174 行
`zircon_runtime/src/scene/ecs/component/registry/tests.rs`。父级 re-export 保留现有内部调用路径，
没有新增 registry、ID authority、兼容 facade 或发布阶段。

结构守卫与 Runtime08 inventory 已覆盖该阶段的新生产 owner，当时 `expected_source_file_count = 76`、
`expected_test_file_count = 10`。两个事务类型、六个事务入口/辅助方法、匹配谓词及六个测试
相对 `HEAD` 的规范化 SHA-256 为 16/16 等价。Unreal `CoreUObject` 的稳定类型/布局身份与
package reload 阶段事务分层作为主工程参考，Bevy `Components` 与 queued registrator/apply
分层作为 ECS 交叉检查。预检的 map probe/append 与 publication 的 pending 顺序、reserve、
冲突语义和复杂度均未改变；Cargo 与 product validation 延后，不声明性能收益或 Runtime08
acceptance。

## 2026-08-28 Archetype Topology Equality Receipt

状态：`runtime_08_15_archetype_topology_equality_receipt_static_passed_cargo_deferred`。

Runtime60 的 `RECS-P1-10` 源码缺陷已收敛：`ArchetypeIndex::PartialEq` 不再恒定返回 `true`，
而是委托给零分配 borrowed `ArchetypeTopologySnapshot`。receipt 同时核对 signature index、
component inverted index、record 顺序及每个 record 的 archetype ID、signature、entity rows；
性能计数器和 `membership_generation` 历史不属于当前结构身份，避免 diagnostics/read history
改变 `World` 相等语义。

`archetype/index/tests.rs` 已写入三项回归，分别覆盖签名差异、实体行差异以及 diagnostics/
membership history 排除规则；`tools/tests/test_runtime_archetype_topology_equality_contract.py`
固定实现与五份计划镜像。Unreal Mass 的 data-identity handle 与 versioned handle 分层为主参考，
Bevy 的 world-local `ArchetypeId`/`ArchetypeGeneration`/`Archetypes` 分离为 Rust 交叉检查。显式
相等比较从错误的 O(1) 恒真改为 O(signature/component index + archetype records + entity rows)，
不进入 query/frame 热路径；Cargo/product/profile 尚未执行，不声明 Runtime08/15 acceptance、
性能收益、milestone commit 或企微同步。

基础设施同步将 `tools/tests/test_runtime_ecs_kernel_data_audit.py` 的陈旧 source count 从 75
更新为当时 inventory 的 76；本次 locator child-owner 同步后当前计数为 77。该 aggregate audit 因共享脏 owner 仍报告四项独立风险：dual
storage getter、component-storage sibling import、generational entity anchors、observer bucket
anchor；相关 production 与 audit owner 正由其它改动占用，本切片未覆盖或宣称关闭。上述状态
中的 `static_passed` 仅指 topology focused guard 2/2 与 registry/inventory focused guard 2/2。

## 2026-08-28 Sparse Component Locator Pages

状态：`runtime_08_60_sparse_component_locator_algorithm_source_passed_diagnostics_cargo_product_profile_deferred`。

Runtime60 `RECS-P1-11` 的分页算法项已收敛，但整项保持 partial。`SparseComponentStorage` 不再把
locator 连续 `Vec<Option<_>>` 扩到最高 entity index；独立 locator owner 使用 256-slot packed
page、零起始 flat prefix 与一个 page-aligned 高位热点 window；两个 flat span 均以每个 live
locator 最多 1,024 slots 的全局密度界提升。其它不相邻页进入私有 `u32` identity-hash 目录，
`BTreeSet` 只承担有序页 ownership/range absorption。空页立即退休；低密度前缀先重基址、window
先裁剪空边缘，仍低于 1/2,048 才降级，locator 全空释放所有容量，swap-remove 与 stale-generation
语义不变。

独立 Rust 1.94.1 release 模型在 4,000,000 高水位下测得 `96,000,024 B -> 2,048 B`
(-99.9979%)，262,144 连续 rows 为 `6,291,456 B -> 2,097,152 B` (-66.667%)。最终三轮
31-pair dense P50 为改善 10.3439% 到回退 4.2631%，所有 partial 最差回退 28.0677%；高位聚簇
mixed 改善 6.4199%-15.0733%，hit-only 改善 7.2077%-13.7094%，双 span 回退
4.3081%-16.1994%，均通过 30% 上限。原 sparse
HashMap 高位聚簇路径回退 203.3120%-446.0091%，radix/open-row 候选也未过线，故改为有界
offset window；真正第三离散簇仍是待产品 profile 证明为冷路径的 memory-first overflow。
focused contract 3/3、真实 owner Rust harness 16/16（含跨表示删除后统一 compaction）、非测试编译壳与 checksum 通过。生产 locator-byte diagnostics 尚未
聚合到共享 `ComponentStorage` owner；Cargo、百万 counters/RSS slope、真实 scene P95、WPR/
CPU/power 与 G06 仍 pending，因此不关闭 `RECS-P1-11`、Runtime08 failure 或 managed milestone。
