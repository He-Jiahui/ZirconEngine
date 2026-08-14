---
handoff_kind: failure
status: open
created_at: 2026-08-01
summary_slug: runtime15-archive-output-guard-read-path-drift
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/15
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/facade_surface.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/animation_manager.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/runtime_dead_code/guard_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/runtime_dead_code/production_scan.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/runtime_dead_code/runtime_owned.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/runtime_dead_code/runtime_ui.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/runtime_dead_code/script_host.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/runtime_dead_code/ui_text.rs
tests:
  - python -m unittest tools.tests.test_runtime15_archive_guard_hard_cut -v
  - cargo +1.94.1 test -p zircon_runtime --lib runtime_15_ --locked --jobs 1 -- --nocapture --test-threads=1
---

# Runtime15 archive output guard read-path drift

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：Runtime15 archive-output guard read-path audit
- 修复责任计划：`docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md`
- 交接原因：历史状态锚点已经从四份 live root 文档硬迁移到 Runtime15 canonical archives，测试读取入口与归档所有权必须由 Runtime15 收敛。

## 失败现象与复现证据

七个 Runtime15 Rust 结构守卫仍编译并读取 `docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-*` 历史输出，仅用于重复验证旧 milestone receipt。`ui_text.rs` 已先行收敛到当前生产源码约束，不再读取 archive；其余生产 owner、模块边界、文件预算和禁止 API 断言仍有效。

## 最低共享层根因

历史 plan receipt 的所有权已经硬迁移到 Coordinator/Python/plan lifecycle，Rust lib-test 仍承担历史输出镜像，导致归档文档变化进入 Runtime 编译和测试边界。最低层修复不是继续维护 archive 路径，而是删除 Rust guard 中的历史 receipt cohort，同时保留当前生产源码契约。

## 架构修复验收

- 七个剩余 Rust guard 不再读取或编译 `2026-07-09-*` 历史 output receipts。
- 生产 owner、模块边界、目录挂载、行数预算、dead-code 禁止项和当前模块文档断言保持原 owner。
- 独立 Python 回归固定该 hard cut，防止历史 archive 路径重新进入 live Rust guard。
- scoped rustfmt、静态回归、二次审查与 current-source Runtime15 focused Cargo gate 均需形成可追溯证据。

## 禁止临时方案

不得把历史锚点复制回 live root 文档，不得恢复 archive 读取、fallback 或兼容入口，也不得删除仍验证当前生产源码和模块边界的断言。

## 修复结果与回传

Resolving state: 七个 Rust guard 的历史 receipt cohort 已删除；生产/owner/预算/禁止 API 断言保留。`python -m unittest tools.tests.test_runtime15_archive_guard_hard_cut -v` 当前 1/1 GREEN，Rust 1.94.1 rustfmt 与 scoped `git diff --check` 通过。二次审查和 managed Runtime15 focused Cargo 回执待完成，因此本记录保持 `open`，不提前返回 fixed。

## 2026-08-03 successor hard-cut 进展

更高优先级的 receipt-test compile-debt failure 将验收方向收敛为“Rust 只验证当前源码与模块边界，历史 plan receipt 由 Coordinator/Python 生命周期持有”。本次前向修复落实该边界，不恢复旧归档兼容路径；managed Runtime lib-test 与 failure return 尚未完成，因此只记录 resolving evidence。

## 2026-08-15 exact cohort regression repair

The first Python regression scan incorrectly covered all Runtime tests and then
the entire `structure_convention` directory. Those scopes include independent
archive consumers outside this handoff (159 and then 31 paths respectively),
so their failures were not evidence against this repair. The regression now
enumerates exactly the eight guard paths in this handoff's `related_code`
cohort: the animation/facade guards and the six `runtime_dead_code` children.

`PYTHONDONTWRITEBYTECODE=1 python -m unittest
tools.tests.test_runtime15_archive_guard_hard_cut -v` now executes `1` test
with `1 passed; 0 failed`; a direct cohort scan reports `8` paths and `0`
historical archive readers. Rust 1.94.1 scoped `rustfmt --check` and scoped
`git diff --check` also pass. This is static source evidence only: independent
immutable review, managed Runtime15 Cargo, and the canonical fixed return are
still required, so the handoff remains `open`.
