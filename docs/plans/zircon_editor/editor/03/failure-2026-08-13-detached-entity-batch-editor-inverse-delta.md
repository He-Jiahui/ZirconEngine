---
handoff_kind: failure
status: open
created_at: 2026-08-13
summary_slug: detached-entity-batch-editor-inverse-delta
origin_plan: docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
fixing_plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
origin_child_dir: docs/plans/zircon_runtime/runtime/08
fixing_child_dir: docs/plans/zircon_editor/editor/03
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/editing/command.rs
  - zircon_editor/src/tests/editing/state/viewport.rs
  - zircon_runtime/src/scene/world/transaction/detached_entity_batch.rs
tests:
  - cargo test -p zircon_editor --lib editing --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib detached_batch --locked --jobs 1 -- --nocapture --test-threads=1
---

# Editor03：消费 move-only DetachedEntityBatch inverse delta

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md`
- 来源执行切片：Runtime08 F5 World affected-row detach/restore hard cut
- 修复责任计划：`docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md`
- 交接原因：Runtime08 拥有 World storage/archetype delta；Editor03 拥有 `DeleteNodeCommand`、undo/redo execution payload 和 journal descriptor 的唯一事实源。

## 失败现象与复现证据

Runtime 当前 public contract 已硬切为 `remove_entity(EntityId) -> SceneResult<()>`、`remove_entity_recursive(EntityId) -> SceneResult<DetachedEntityBatch>` 和 `restore_detached_entity_batch(DetachedEntityBatch)`。Editor `DeleteNodeCommand` 仍按旧 contract clone `Vec<NodeRecord>`，递归删除后保存 fixed-component-only record，undo 再经 borrowed record insertion；这既无法编译消费新返回类型，也会丢失 generic table/sparse rows、ticks、dynamic JSON、observer 和 active-camera ownership。`zircon_editor/src/tests/editing/state/viewport.rs` 仍把 typed removal result 作为 bool 断言。

## 最低共享层根因

Editor command execution payload 和 cloneable/serializable journal metadata 尚未分离。`DeleteNodeCommand` 需要在 World 与 command 之间唯一移动完整 inverse delta，不能继续用可 Clone 的 `NodeRecord` 投影充当运行时正文。

## 架构修复验收

- `DeleteNodeCommand` 以 `Option<DetachedEntityBatch>` 表示执行正文当前由 command 或 World 唯一拥有；delete/redo detach 并取得 delta，undo consume delta restore，下一次 redo 再从 World 获取新的 delta。
- journal 仅在首次 capture 时生成独立、可序列化的 affected-payload descriptor；不得为了日志长期复制 table/sparse/dynamic 组件正文，也不得用 `Arc<Mutex<DetachedEntityBatch>>` 共享可变 owner。
- delete/undo/redo 覆盖 nested subtree、table+sparse+dynamic rows、ticks、stable order、active camera 和 failed restore returning ownership；typed error 不得降回 bool/string。
- 1/1k/100k affected payload 与 1/1%/100% batch 记录 World/NodeRecord clone bytes、moved rows、rollback bytes、p95 和 peak RSS；full World clone 必须为 0。
- 重新执行 Editor editing gate 与 Runtime detached-batch upward gate；获得 raw terminal 结果和独立二次审查后才能 fixed return。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- Do not weaken tests or plan acceptance criteria to hide the failure.
- 禁止把 typed `SceneError` 映射回 bool，禁止继续让 `NodeRecord` 充当完整 inverse delta，禁止复制/序列化整个 World 作为 undo 正文。

## 修复结果与回传

Open state: `Runtime contract available / Editor inverse delta pending`; no Cargo pass is claimed.

- Runtime08 已提供 move-only `DetachedEntityBatch`、完整 preflight、failed restore ownership return、stable/dense/hierarchy/camera indexed boundary 和精确诊断计数。
- Runtime source-bound validation-copy 请求在 client timeout 前未返回 durable receipt；本 handoff 不声称 Runtime Cargo green。
- Editor03 应在其现有 primary/owner policy 下前向迁移，不由 Runtime08 Session 吸收 Editor source。
