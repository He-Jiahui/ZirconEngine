---
handoff_kind: fixed
status: fixed
created_at: 2026-07-16
summary_slug: lock-poison-guard-archive-owner-drift
origin_plan: docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
fixing_plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
origin_child_dir: docs/plans/zircon_runtime/runtime/11
fixing_child_dir: docs/plans/zircon_runtime/runtime/15
plan_link_mode: child_record_only
priority: 100
related_code:
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/asset_render_input/asset_pipeline.rs
  - docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md
  - docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md
  - docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md
  - docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md
tests:
  - cargo test -p zircon_runtime --lib --locked worker_pool -- --test-threads=1 --nocapture
resolved_at: 2026-07-16
---


# Runtime 15: lock-poison guard archive owner drift

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/11-job-system-task-model.md`
- 来源执行切片：Runtime11 M2.4 asset IO pool hard cut managed worker-pool gate
- 修复责任计划：`docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md`
- 交接原因：Runtime11 生产行为测试全部通过后，Runtime15 lock-poison compile-time guard 仍读取已完成硬切的活动父计划路径，形成跨计划静态镜像阻断。

## 失败现象与复现证据

Runtime11 managed worker-pool job `c9204aa054724539970bd3ec01726dfb` executed 18 tests and reached 17 passed / 1 failed. The remaining Runtime15 guard still read active parent plans after their 2026-07-09 output records were hard-cut to `_archive`, so it reported five missing asset-worker lock-poison anchors even though all five remain in every canonical archived output owner.

## 最低共享层根因

The plan-output hard cut moved status evidence without atomically migrating `asset_pipeline.rs` compile-time readers. Reintroducing the extracted rows into protected parent plans would create duplicate facts and violate the archive cutover.

## 架构修复验收

Hard-cut both asset lock-poison guard cases to the four canonical Runtime15 archive outputs. Keep active parent plans concise, keep child-only failure lifecycle metadata, and add no alias, fallback, duplicate output, or weakened assertion.

- All four archive files contain the required worker-pool status anchor.
- The guard reads no active parent plan for extracted 2026-07-09 status rows.
- Runtime11 `worker_pool` filter reaches a natural `0 failed` summary.

## 禁止临时方案

- 不得把归档状态段复制回 Runtime15、runtime index 或两份优先父计划。
- 不得保留活动路径 fallback、双读、alias、shim 或弱化 required-anchor 断言。
- 不得用更窄过滤跳过 Runtime15 guard 后宣称 `worker_pool` 完整通过。

## 修复结果与回传

- 根因：The 2026-07-09 status outputs were hard-cut to canonical archive files, but the Runtime15 lock-poison guard continued reading protected active parent plans.
- 架构修复：Hard-cut both asset lock-poison guard cases to the four canonical Runtime15 archive outputs; no parent-row restoration, fallback, alias, shim, or duplicate fact source.
- 验证：rustfmt and diff-check passed; runtime audits 2/2 passed; managed Windows job c7c9a84482e34825aa1b0d94a08aee97 ran the original worker_pool filter with 18 passed, 0 failed, 8152 filtered and exit 0.
- 回传：Runtime15 lock-poison plan mirror now consumes canonical archive owners and Runtime11 worker_pool validation may continue.
