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

Text01 current-source default/UI lib-test job `f9f5581fb83b40c2a3cc81aa15f5bcaa`、run
`b98dc769094b4bd9b96fc445fd8a1332` 执行
`runtime_15_hub_raw_text_policy_guard_is_child_owner` 时失败。child-owner 源文件守卫已越过，但 Runtime index
缺少 `Runtime 15 M3 Hub raw-text policy guard child-owner split`、status/guard 名称及两个 child path 锚点。

job 于 `2026-07-22T19:24:42.482382+00:00` 自然结束并 release，exit `101`、live PIDs 为空；
原始日志位于 `.codex/state/session-coordinator/cargo-runs/f9f5581fb83b40c2a3cc81aa15f5bcaa/b98dc769094b4bd9b96fc445fd8a1332/`。

## 修复责任

Runtime15 应恢复 Runtime index、status/date/output row 的同一 canonical child-owner 状态，运行精确守卫、独立 review 与 managed commit。Text01 不修改 Hub/Runtime15 owner。
