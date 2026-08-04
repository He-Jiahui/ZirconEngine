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
  - cargo +1.94.1 test -p zircon_runtime --lib runtime_15_ --locked --jobs 1 -- --nocapture --test-threads=1
---

# Runtime15 archive output guard read-path drift

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：Runtime15 archive-output guard read-path audit
- 修复责任计划：`docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md`
- 交接原因：历史状态锚点已经从四份 live root 文档硬迁移到 Runtime15 canonical archives，测试读取入口与归档所有权必须由 Runtime15 收敛。

## 失败现象与复现证据

`ui_text.rs` 与另外七个 Runtime15 测试文件仍直接或通过 clone/fallback 读取四份 live root 文档，并断言 2026-07-09 之前的历史状态锚点。逐源审计确认这些锚点在四份 canonical archive、对应模块文档与 status row owner 中完整存在，而相关 live runtime index、review findings 与 structure convention 文档已经移除历史输出。

## 最低共享层根因

2026-07-16 的 output archive hard cut 迁移了历史证据 owner，但早于迁移创建的结构守卫没有同步切换读取路径，并继续使用允许 archive fallback 的非精确断言。

## 架构修复验收

- 八个测试文件的历史 cohort 显式读取四份 `docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-*-output-records.md` canonical archives。
- 历史输出断言使用 exact source 检查；生产代码、模块文档、current status row 与 current-owner 断言保持原 owner。
- scoped rustfmt、静态锚点核验与 current-source Runtime15 focused Cargo gate 通过，且目标断言真实执行。

## 禁止临时方案

不得把历史锚点复制回 live root 文档，不得删除生产/状态/文档断言，也不得依赖 label-based archive fallback 掩盖错误 source。

## 修复结果与回传

Open state: `八个文件的历史读取 cohort 已静态硬切到 canonical archives 并改用 exact assertion；等待 Text04/Render17 current-source 编译阻断解除后取得真实 Runtime15 focused Cargo 证据`。

## 2026-08-03 successor hard-cut 进展

更高优先级的 receipt-test compile-debt failure 已取代“继续维护历史状态 Rust 断言”的旧验收方向：纯 archive/status receipt guards 及其路径镜像被直接删除，生产源码/owner/预算/禁止 API 断言继续由现有 structure guards 持有。当前不存在 archive fallback 或兼容入口；本地 hard-cut/Runtime03 Python 回归 5/5 通过，独立二次审查为 Critical/Important/Minor = `0/0/0`。managed Runtime lib-test 与 plan-output/handoff evidence 尚未完成，因此本记录保持 `resolving`，待 successor failure 的受管回执一起返回 fixed。
