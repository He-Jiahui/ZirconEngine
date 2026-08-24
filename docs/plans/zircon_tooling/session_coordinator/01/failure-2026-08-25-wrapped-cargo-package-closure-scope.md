---
handoff_kind: failure
status: open
failure_scope: local
created_at: 2026-08-25
summary_slug: wrapped-cargo-package-closure-scope
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/tests/test_workspace_copy.py
  - tools/session_coordinator/validation_copies.py
  - tools/session_coordinator/validation_ticket_worker.py
---

# wrapped-cargo-package-closure-scope: 验证失败回写

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：Managed validation ticket Cargo closure planning for an explicit package-bound wrapped command
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：同一编号计划拥有已集成快照及其前向修复。

## 失败现象与复现证据

- 验证回写：`Managed validation ticket Cargo closure planning for an explicit package-bound wrapped command` — Tickets 4a63e3334cef48029190d9976e0963c4, 2bf1a5bc03474f4dbdf07006cda57cbf, and e48fd4c939f34798b2e5b2484e912b85 were classified as Cargo and included --package zircon_runtime_interface, but copies 130d956bfbfe4cae97c7144d7004d7a6, 0847d42f, and 930c9dc2 failed while scanning unrelated zircon_editor compile-time resources before Cargo.

## 最低共享层根因

CargoInputClosurePlanner.plan unconditionally seeds dependency traversal with every workspace member in addition to the explicitly selected package, so an unrelated member's missing compile-time include blocks a package-scoped run.
The first forward fix restricted resource scanning, but the copy still included every unrelated workspace source tree; managed copy `ca11d0c0b5f64d5bb9964330bf306ba5` then crossed the old failure point and failed closed on unrelated `zr_rhi_wgpu` baseline drift before process launch. Package-scoped source selection must therefore be narrow as well as resource-scoped, while retaining workspace manifests for Cargo topology parsing.

## 架构修复验收

- An explicit -p/--package command scans compile-time resources only for the selected package and its transitive build dependencies while preserving the complete workspace topology required by Cargo.
- Unselected workspace and local path packages contribute their tracked Cargo manifests but not unrelated source trees to a package-scoped copy.
- An unrelated workspace member with a missing compile-time resource cannot fail package-scoped closure planning.
- Commands without an explicit package preserve full-workspace closure semantics, and missing package requests fail with the existing typed error.
- A managed wrapped package command reaches a terminal validation-copy run without scanning the unrelated Editor asset.

## 禁止临时方案

- 不回滚已集成快照来掩盖普通测试失败；应通过前向修复返回 `fixed-*` 记录。
- 不得添加别名、兼容垫片、静默回退、测试旁路或调用点特例。

## 修复结果与回传

Open state: `待修复`; the coordinator must keep the validation ticket and route this Plan to repair work.
