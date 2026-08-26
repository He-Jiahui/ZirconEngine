---
handoff_kind: failure
status: open
created_at: 2026-08-23
summary_slug: editor-delete-subtree-all-cameras-invariant
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/08
plan_link_mode: child_record_only
failure_scope: cross_plan
related_code:
  - zircon_runtime/src/scene/world/transaction/detached_entity_batch.rs
  - zircon_editor/src/core/editing/command.rs
  - zircon_editor/src/tests/editing/node_ops.rs
tests:
  - cargo test -p zircon_runtime --lib detached_entity_batch --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib deleting --locked --jobs 1 -- --nocapture --test-threads=1
  - 2/128 camera subtree and 100k non-camera node scale probes
---

# Runtime08: delete subtree camera-count preflight invariant

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：Editor delete-subtree move-only transaction correctness/performance review, 2026-08-23
- 修复责任计划：`docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md`
- 交接原因：Runtime08 owns the generation-bound `World` subtree preflight and detach transaction; Editor05 and Editor03 consume that lower-layer contract for editing commands and history.

## 失败现象与复现证据

当前 move-only delete capture 只在 `scene.camera_count() == 1` 且唯一 active camera 位于目标 subtree 时拒绝。如果场景有两个或更多 camera，且它们全部位于同一个待删除父 subtree，capture 会通过；Runtime detach 完成后没有 surviving camera，`active_camera` 变为 `0`，违反 Editor 的 cannot-delete-last-camera invariant。

现有 `deleting_multiple_cameras_cancels_the_whole_transaction` 选择多个独立 camera root。command 逐个 capture/apply，所以最后一个 command 能观察到 `camera_count == 1`；它没有覆盖一条 subtree command 同时包含全部 camera 的路径。

## 最低共享层根因

Runtime 的 subtree prepare artifact 没有同时封存 normalized roots、affected entity count、affected camera count 与 world generation。Editor 因而只能在 detach 前读取全局 camera count，却不能用同一 generation-bound artifact 判断本次 subtree 会移除多少 camera；如果 detach 后再 restore 进行试探，又会错误发布生命周期事件并推进 generation。

`DetachedEntityBatch` 已正确移动 ECS rows、ticks、dynamic components 与 observer ownership，避免完整 `World`/`NodeRecord` clone。该性能进展必须保留；缺口是 prepare artifact 的 camera-count invariant，而不是 move-only batch 本身。

## 架构修复验收

- Runtime08 提供 generation-bound subtree preflight/ticket，至少包含 normalized roots、affected entity count 与 affected camera count；成本随 affected rows 缩放，不扫描或克隆完整 `World`。
- Editor05 以同一 ticket 验证 `world_camera_count - affected_camera_count >= 1`；同一 generation 执行 detach，stale ticket typed reject 后重新 prepare。
- Editor03 的 command/history consumer 只提交通过上述 preflight 的 move-only batch；拒绝路径不得改变 selection、history 或 dirty state。
- 增加单父节点包含全部 2/128 cameras，以及混合 100k non-camera nodes 的回归。拒绝后 world digest、active camera、selection、history、dirty state、lifecycle counters 与 generation 必须完全不变。
- 通过声明的 managed Runtime/Editor focused tests、1/1k/100k scale probes 与 F4 product gate 后，才能生成 `fixed-*` 回传。

## 禁止临时方案

- 禁止回退到 `subtree_records()`、`Vec<NodeRecord>`、完整 `World` clone 或全场景 camera scan。
- 禁止 detach 后再 restore 来模拟 preflight；该路径会发布生命周期事件并推进 generation。
- 禁止在 Editor command 中另建不绑定 Runtime generation 的 camera cache、旁路索引或 special case。
- 禁止 aliases、compatibility shims、silent fallback、test-only bypasses 或重复真值。

## 修复结果与回传

Open state: Runtime08 generation-bound subtree preflight/ticket 尚未提供，Editor05/Editor03 consumer 与 managed product evidence 也尚未闭合。本记录仅修复 canonical routing/schema，不声明源码修复、Cargo green、`fixed-*` return 或完成通知。
