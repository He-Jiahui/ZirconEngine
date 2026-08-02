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

Hub raw-text policy guard 已拆到 child owner，但 Runtime15 的 canonical index/status 证据没有在同一结构迁移中更新。测试因此正确地拒绝只有 child 文件、没有 owner index 可追踪锚点的半完成状态。

## 架构修复验收

- Runtime15 恢复 Runtime index、status/date/output row 的同一 canonical child-owner 状态。
- 精确守卫必须验证 index、guard 名称、status 与两个 child path 属于同一状态切片。
- 通过独立 review 与 managed current-source gate 后才能写 fixed return；Text01 不修改 Hub/Runtime15 owner。

## 禁止临时方案

- 不得放宽 child-owner guard、删除 canonical index 断言或在 Text01 测试中复制 Runtime15 状态字符串。
- 不得只补一个路径字符串而继续遗漏 status/date/output row，也不得用历史 archive 代替当前 Runtime index。

## 修复结果与回传

Open state：`待修复`。本次只把既有 failure 迁移到当前 handoff schema；当前 `docs/plans/zircon_runtime/runtime/index.md` 仍缺 canonical Hub raw-text policy child-owner 锚点，因此没有源码修复、managed GREEN、fixed return 或 commit 声明。
