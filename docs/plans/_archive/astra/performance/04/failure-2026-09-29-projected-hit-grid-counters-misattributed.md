---
handoff_kind: failure
status: open
created_at: 2026-09-29
summary_slug: projected-hit-grid-counters-misattributed
origin_plan: docs/plans/astra/optimize/01-review-and-repair.md
fixing_plan: docs/plans/astra/performance/04-hit-grid-cell-batching.md
origin_child_dir: docs/plans/astra/optimize/01
fixing_child_dir: docs/plans/astra/performance/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/tree/hit_test/cell_membership_patch.rs
  - zircon_runtime/src/ui/tree/hit_test.rs
  - zircon_runtime/src/ui/tree/hit_test/geometry_patch.rs
  - zircon_runtime/src/ui/surface/frame_hit_test.rs
  - zircon_runtime/src/ui/surface/frame_hit_test/tests.rs
tests:
  - cargo +1.94.1 check -p zircon_runtime --features profiling --locked
  - cargo +1.94.1 test -p zircon_runtime --features profiling --locked --lib dense_batches_match_expected_membership_for_1k_and_10k_entries -- --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --features profiling --locked --lib unchanged_footprints_stage_no_cell_work -- --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --features profiling --locked --lib rejects_missing_cell_before_publishing_valid_replacements -- --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --features profiling --locked --lib geometry_patch_activates_and_deactivates_stable_entry_cells -- --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --features profiling --locked --lib projected_nonmonotonic_membership_move_matches_full_rebuild -- --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --features profiling --locked --lib dense_geometry_batches_match_full_rebuild_and_visit_each_cell_once -- --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --features profiling --locked --lib projected_dense_geometry_batches_match_full_rebuild_at_scale -- --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --features profiling --locked --lib failure_preserves -- --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --features profiling --locked --lib painter_reorder_then_cell_move_removes_nonmonotonic_entry_index -- --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --features profiling --release --locked --lib dense_batch_release_measurement -- --ignored --test-threads=1
---

# UI04: projected hit-grid patch writes base profiling counters

Frontmatter 中的 Cargo 命令是协调器受管验证票据的命令载荷；由协调器在批准的 D/E/F `cargo-targets` 目录执行，不作为手动 Cargo 命令运行。

## 来源执行者

- 来源计划：`docs/plans/astra/optimize/01-review-and-repair.md`
- 来源执行切片：UI-A3 命中网格计数器归属审查。
- 修复责任计划：`docs/plans/astra/performance/04-hit-grid-cell-batching.md`
- 交接原因：UI04 负责共享 membership 补丁及 base/projected 记录点。
- 稳定 Session：`failure-roll-01a0df1a-ui04-hit-grid-counter-attribution-r1`。

## 失败现象与复现证据

原始源码在共享 `UiCellMembershipPatches::apply` 内无条件发出 `ui.hit_grid.cell_patch_materialized_membership_count` 和 `ui.hit_grid.cell_patch_replacement_buffer_count`。Projected 路径也调用这个共享方法，所以仅运行 projected 补丁时，base 命名空间包含 projected 工作，而 `ui.surface_projected_hit` 缺少这两项。该结论来自直接生产调用链和源码；受管动态复现仍待执行。

新增的 profiling 回归分别捕获一次真实 base 和 projected 补丁，要求本路径的四项计数均有记录，另一命名空间不出现同名计数。base 激活样本的 removal 计数为 0，其余三项为正；projected 移动样本的四项均为正。精确测试过滤必须报告实际执行的目标测试；静态检查不构成验收。

## 最低共享层根因

共享补丁层计算两个统计值，但不知道调用者的 telemetry 归属。它直接使用 base 名称导致 projected 采样混入 base 计数。两个 owner 记录点已经分别负责相邻的 cell patch 统计，适合接收这两个值。

## 架构修复验收

- 共享补丁只计算并返回统计；base 记录点写 `ui.hit_grid`，projected 记录点写 `ui.surface_projected_hit`，并保持现有 membership 顺序、命中结果和失败原子性。
- 对 base/projected 的 removal、addition、materialized 和 replacement 计数执行路径归属回归；覆盖共享层、原始复现以及 1,000/10,000 dense 几何变更与全量重建一致性。
- 在同机 Windows 受管 Release 验证中按 UI04 M2 记录成对 p50/p95/p99；一项门槛不满足则保持待验。

## 禁止临时方案

不得在共享层同时发出两种命名空间、清零 projected 统计、以静态检查代替动态复现，或削弱 UI04 的性能门槛。

## 修复结果与回传

Open。源码及测试候选已在精确归属下完成，`rustfmt --check` 和范围内 `git diff --check` 通过。独立最终审查、受管 Cargo 与 Release 验收、failure return 和 closeout 尚未完成。
