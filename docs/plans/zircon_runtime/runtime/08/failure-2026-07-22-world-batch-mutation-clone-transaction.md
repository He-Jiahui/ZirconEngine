---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: world-batch-mutation-clone-transaction
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/08
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/scene/world/records.rs
  - zircon_runtime/src/scene/world/typed_api.rs
  - zircon_runtime/src/scene/ecs/archetype
  - zircon_editor/src
tests:
  - cargo test -p zircon_runtime --lib scene --locked --jobs 1 -- --nocapture --test-threads=1
  - editor undo/import batch success and failure scale fixtures
---

# Runtime08：World batch mutation零全场clone事务交接

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：Runtime scene world query/records/typed API 4/4逐Rust文件性能审查，PERF-MVP-467
- 修复责任计划：`docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md`
- 共同验收：`docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md`
- 交接原因：Runtime08拥有World storage/archetype mutation transaction；Editor03拥有command/undo artifact与inverse delta消费边界。
- 生命周期键：`world-batch-mutation-clone-transaction`

## 失败现象与复现证据

`insert_node_records`为保证undo/import批次原子性先深clone完整World，再clone每个NodeRecord并通过普通insert执行全部component写入和中间archetype更新；成功后整体替换World，失败则丢弃整份clone。大世界中的单实体undo/import因此也复制所有节点、组件、dynamic JSON、registry和derived state。

## 最低共享层根因

World只有单项mutation API和“clone whole authority”事务办法，没有可预验证的batch mutation plan、affected-row undo delta、copy-on-write storage page或单次generation commit边界。Editor命令层也无法传递already-owned records/compiled component writes。

## 架构修复验收

- Runtime08提供batch plan：先验证entity identity、schema/reference和每entity最终signature，不修改authority；记录affected rows/components的before delta或共享copy-on-write pages。
- commit只写affected storage/archetype rows并一次发布query/derived/world generation；同entity多record/component更新合并，不产生中间archetype churn。
- Editor03 command/undo传递owned/Arc artifact和inverse delta，不先构造第二份完整World；failure/cancel只丢未发布plan。
- world/records/payload 1/1k/100k、batch 1/1%/100%、success/failure记录World/NodeRecord clone bytes、component writes、archetype moves、rollback bytes和p95：full World clone=0、工作近affected payload、failure authority零变化。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止把clone从World移动到Editor snapshot或serde bytes后声称解决；验收统计端到端owned bytes/RSS。
- 禁止逐record commit后用补偿命令回滚；authority必须在预验证成功前不可见。
- 禁止保留大批次走新transaction、小批次默认clone World的双权威路径。

## 修复结果与回传

Open state: `前向修复中`; no pass is claimed.

- 已完成：`insert_owned_node_records(Vec<NodeRecord>)` 先整体验证 stable identity、重复项、transform、next id 与 mobility，随后将 owned records 一次发布；World generation、derived/query invalidation 与 lifecycle dispatch 在整个 batch 可见后只发布一次。
- 2026-08-11 current-source reconciliation：dynamic-scene transaction 已把 generic table/sparse component rows、dynamic JSON 与 resource rows 纳入 affected-schema preflight；descriptor/local-id compatibility 在 live mutation 前完成，commit 按 entity 聚合 final signature 并只做一次 row transition，随后统一发布 generation/lifecycle。`insert_owned_node_records` 同样不 clone World，并按 record 一次提交完整 final row。因此早期“generic component/resource 与 final signature 未进入 transaction”的描述已过时。
- 仍未完成的最低边界已缩小为 Editor03 inverse delta：`DeleteNodeCommand::capture` 先 clone subtree `NodeRecord`，undo 又经 borrowed `insert_node_records(&records)` 执行 `to_vec()`；`NodeRecord` 还不携带 generic/dynamic rows。简单改成 consuming Vec 会让 undo 后 command 丢失 redo/journal 正文，不是可接受修复。
- 当前证据：仅完成格式、diff 与静态 source guard；未运行声明的受管 Cargo、Editor undo/import 上游门或 scale fixtures。

### Owned affected-row delta 设计（下一原子实现边界）

- Runtime08 新增不可 Clone 的 `DetachedEntityBatch`，唯一拥有：按稳定 world order 排列的 entity identity/kind、原 stable-order key、完整 table row values+ticks、SparseSet rows+ticks、dynamic component JSON 与 active-camera/parent 边界元数据。不得用 `NodeRecord` 作为正文，因为它只覆盖固定持久组件。
- F5 public mutation hard cut is part of this boundary: `remove_entity` must become `SceneResult<()>`, and the recursive public operation must become `SceneResult<DetachedEntityBatch>`. A missing root/entity is a typed `SceneError::MissingEntity`, never `false` or an empty `Vec`; success has no compatibility bool. Migrate the deferred `Commands` facade, Editor delete/undo, script host, plugin callers, and assertions together rather than converting typed errors back to bool at an adapter.
- `detach_entity_subtrees` 先验证 roots、重复/ancestor 覆盖、last-camera 与外部 hierarchy references；成功后每 entity 只 take 一次 archetype row、同步修复 source swap location，移出 sparse/dynamic rows，最后一次发布 removal lifecycle/world generation。preflight 失败时 World 零变化；take 开始后路径必须 infallible。
- `restore_detached_entity_batch` 先验证 identity、descriptor/schema、parent dependency、stable-order key 与 final signature；随后移入 identity 和完整 rows，每 entity 一次 archetype append，统一恢复 active camera 并一次发布 add/insert lifecycle/generation。失败返回原 `DetachedEntityBatch` 给调用者，不能丢弃 owned payload 或补偿回滚。
- Editor03 `DeleteNodeCommand` 用 `Option<DetachedEntityBatch>` 表示“正文当前在 command 中还是 World 中”：delete/redo detach 并取得 delta，undo consume delta restore；下一次 redo 再从 World detach 新 delta。journal 在首次 capture 时生成一次独立 serializable affected-payload record，避免为了日志让 command 与 World 长期双持组件正文。若现有 `EditorCommand: Clone` 只服务 UI/history metadata，应硬切为 cloneable journal descriptor + non-clone execution payload，禁止以 `Arc<Mutex<DetachedEntityBatch>>` 共享可变 owner。
- 验收必须覆盖 nested roots 去重、external-parent boundary、table+sparse+dynamic rows、ticks、stable query order、active camera、lifecycle order、failed restore 返回 delta、delete/undo/redo 循环，以及 affected 1/1k/100k 对 full-World clone bytes=0、component move/drop exactly once、archetype publish <=1/entity 的计数。

### 2026-08-11 batch algorithm/research refinement

- Current source confirms the remaining inverse path is real: `remove_entity_recursive` clones `subtree_records` then invokes complete `remove_entity` per node; `DeleteNodeCommand` retains those fixed-only records and undo routes through borrowed insertion. Generic table/sparse rows, ticks, and dynamic JSON therefore cannot survive through `NodeRecord`.
- Unreal Mass primary reference (`FMassEntityManager::BatchDestroyEntities` and `FMassArchetypeData::BatchDestroyEntityChunks`) first groups/deduplicates entities by archetype and processes source ranges from the back so swap-removal cannot invalidate later locations. Zircon will reuse that ordering principle, not Unreal's handle ABI.
- Detach preflight must collect all fallible identity/schema/parent/camera checks before mutation, group selected entities by source archetype, then take rows in descending table-row order. Each take immediately repairs its swapped entity registry and stable-query location; sparse rows and dynamic JSON move into the single batch owner in the same entity pass. Restore preflights all target signatures before appending any row, then performs one final lifecycle/generation publication.
- 2026-08-11 complexity audit correction: the storage row algorithm alone is not sufficient to claim whole-operation `O(affected rows + affected columns + affected dynamic entries)`. Current `World::remove_entity` linearly locates an id in `World::entities`, uses `Vec::remove`, scans all surviving entities for orphaned children and a replacement camera; the same ordered vector is consumed by query fallback, hierarchy/derived rebuild, serialization, bindings, and property traversal. `EntityRegistry` has no stable-id-to-world-order removal location, and `StableQueryOrderIndex` is query-only. The batch implementation must therefore first replace or augment this shared order with one authoritative O(1)-removable dense order plus a deterministic iteration view, and precompute the hierarchy/active-camera boundary from indexed data during preflight. Until that cut is complete, the only valid complexity claim is local storage extraction; the whole batch remains open.
- Managed Windows profiling under `E:\Git\ZirconEngine\.codex\artifacts\runtime08\detached_entity_batch\` will compare 1/1k/100k fixed-only, table+sparse, and dynamic payload subtrees. Required counters: full-World/NodeRecord clone bytes, moved rows, swap repairs, archetype publications, lifecycle dispatches, generation advances, ordered-entity removals, hierarchy/camera boundary lookups, CPU median/p95, and peak RSS. Final target is full-operation `O(affected rows + affected columns + affected dynamic entries)` with full-World clone `0` and publication `1/batch`; it is not accepted until the ordered-entity and boundary-index prerequisites are measured.

### 2026-08-13 Runtime08 affected-row implementation

- Runtime public mutation has been hard-cut: `remove_entity` returns `SceneResult<()>`; `remove_entity_recursive` returns a move-only `DetachedEntityBatch`; `remove_entity_subtrees` is the canonical multi-root operation. Missing roots return typed `SceneError`, and duplicate/nested roots are normalized before preflight so every affected entity moves exactly once.
- `DetachedEntityBatch` owns complete archetype table values and ticks, SparseSet rows and ticks, dynamic JSON, entity observers, node kind, stable-order key, hierarchy boundary and active-camera owner. Restore preflights the complete batch and returns `DetachedEntityBatchRestoreError` with the original batch on rejection. Detach and restore each publish lifecycle visibility, derived dirtiness and World generation once after all rows are visible.
- Physical World identity now uses `entity_dense_rows` plus `Vec::swap_remove`; deterministic iteration remains in `StableQueryOrderIndex`. Subtree and direct-child discovery use `HierarchyMutationIndex`, while active-camera fallback queries the camera archetype through stable order. The retired `Vec::remove`, orphan full-world scan and stable-entity camera scan are no longer part of the current removal source.
- `EcsFramePerformanceDiagnostics` now exposes detached-batch counters for commits, rejected preflights, full-World/NodeRecord clone bytes, moved rows/table/sparse/dynamic values, swap repairs, archetype publications, lifecycle events, generation advances, ordered removals, hierarchy/camera index lookups and rollback bytes. Runtime behavior fixtures cover table+sparse+dynamic rows, ticks, observers, stable order, active camera, failed restore ownership, nested root normalization and exact-once counters.
- The dedicated managed profiling fixture fixes World cardinality at 100,000, exercises affected sizes 1/1,000/100,000, takes 20 detach/restore samples per size and prints p95 with movement counters. A non-ignored fixture also compares identical affected=1 diagnostics in World sizes 1 and 10,001 and requires exact counter equality.
- Source formatting and scoped diff checks pass for the implementation paths. The source-bound validation-copy request did not return a durable receipt before its client timeout, so no Cargo result is claimed and the request was not retried or polled.
- Remaining joint boundary: Editor03 must replace `DeleteNodeCommand`'s cloned `Vec<NodeRecord>` with `Option<DetachedEntityBatch>` execution ownership plus an independently serializable journal descriptor, then validate delete/undo/redo and failure/cancel. Foreign Editor and plugin call sites that still consume the retired bool/record contract remain cross-plan work; Runtime08 does not absorb them.
