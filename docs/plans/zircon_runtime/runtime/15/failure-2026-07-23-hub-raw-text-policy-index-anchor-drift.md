---
handoff_kind: failure
status: open
created_at: 2026-07-23
summary_slug: hub-raw-text-policy-index-anchor-drift
origin_plan: docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
fixing_plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
origin_child_dir: docs/plans/zircon_runtime/text/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/15
plan_link_mode: child_record_only
related_code:
  - docs/plans/zircon_runtime/runtime/index.md
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/naming_boundary_hub_raw_text.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/hub.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/hub/raw_text_policy.rs
---

# Runtime15：Hub raw-text policy index 锚点漂移

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md`
- 来源执行切片：Text01 current-source default/UI lib-test 对 Runtime15 Hub raw-text policy child-owner guard 的 upward validation。
- 修复责任计划：`docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md`
- 交接原因：Runtime15 拥有 Runtime index、status/date/output row 与结构守卫锚点；Text01 只能报告 upward guard 失败，不能修改 Hub/Runtime15 owner 文档来制造通过。

## 失败现象与复现证据

Text01 current-source default/UI lib-test job `f9f5581fb83b40c2a3cc81aa15f5bcaa`、run
`b98dc769094b4bd9b96fc445fd8a1332` 执行
`runtime_15_hub_raw_text_policy_guard_is_child_owner` 时失败。child-owner 源文件守卫已越过，但 Runtime index
缺少 `Runtime 15 M3 Hub raw-text policy guard child-owner split`、status/guard 名称及两个 child path 锚点。

job 于 `2026-07-22T19:24:42.482382+00:00` 自然结束并 release，exit `101`、live PIDs 为空；
原始日志位于 `.codex/state/session-coordinator/cargo-runs/f9f5581fb83b40c2a3cc81aa15f5bcaa/b98dc769094b4bd9b96fc445fd8a1332/`。

## 最低共享层根因

Hub raw-text policy guard 已拆到 child owner，但旧 structure guard 仍把 Runtime index/status wording 当作 Rust lib-test 契约。2026-08-02 的 plan-status receipt-tree hard cut 已把计划 lifecycle 迁出 Runtime Rust 编译面；继续向 current index 回填完整 tuple 会恢复已退役的双重事实源。当前应保留真实 parent/child owner、生产 Hub policy、canonical archive evidence 与 module contract，删除顶层 structure guard 中未使用的 status 常量和 plan 文档读取。

## 架构修复验收

- 顶层 structure guard 验证 parent 挂载、child guard/scan helper owner 与两个文件预算，不再编译 Runtime index/status wording。
- child behavior guard 继续验证真实 Hub production API、canonical archive evidence 与 module convention；计划 lifecycle/schema 由 Coordinator/Python tooling 验证。
- Runtime index 保持概览/路由职责，不复制完整 status/date/output tuple，也不恢复已删除的 Rust plan-status tables。
- 通过独立 review 与 managed current-source gate 后才能写 fixed return；Text01 不修改 Hub/Runtime15 owner。

## 禁止临时方案

- 不得放宽 child-owner 或 Hub production guard，也不得在 Text01 测试中复制 Runtime15 状态字符串。
- 不得恢复 plan-status Rust tables、把完整历史 tuple 复制回 Runtime index，或增加 alias/shim/compatibility route。

## 修复结果与回传

Open state：`resolving_failure`。2026-08-14 current-source 前向修复已删除顶层 Hub structure guard 的三个退役 status constants 与五个未使用 plan/status 文档读取；parent/child owner、真实 Hub policy、canonical archive evidence 和 module contract 均保持。scoped static/review、managed current-source Cargo 与 failure return 尚未完成，因此不声明 fixed/accepted。
