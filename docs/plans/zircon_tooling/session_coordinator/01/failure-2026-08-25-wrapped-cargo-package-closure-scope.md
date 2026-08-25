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
  - tools/session_coordinator/tests/test_validation_copies.py
  - tools/session_coordinator/tests/test_workspace_copy.py
  - tools/session_coordinator/validation_copies.py
  - tools/session_coordinator/validation_copy_external.py
  - tools/session_coordinator/workspace_copy.py
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

CargoInputClosurePlanner.plan unconditionally seeded dependency traversal with every workspace member in addition to the explicitly selected package, so an unrelated member's missing compile-time include blocked a package-scoped run. The first forward fix restricted resource scanning, but the copy still included every unrelated workspace source tree; managed copy `ca11d0c0b5f64d5bb9964330bf306ba5` then crossed the old failure point and failed closed on unrelated `zr_rhi_wgpu` baseline drift before process launch. Independent review subsequently found two lower-layer identity gaps: topology-only external packages omitted ancestor workspace manifests, and different pinned repositories could merge at one mount.

## 架构修复验收

- An explicit -p/--package command scans compile-time resources only for the selected package and its transitive build dependencies while preserving the complete workspace topology required by Cargo.
- Unselected workspace and local path packages contribute their tracked Cargo manifests but not unrelated source trees to a package-scoped copy.
- External topology-only packages contribute exact package and ancestor Cargo/toolchain inputs; selected external packages contribute their source root.
- One mount cannot combine different immutable `repo_root + commit` identities, including explicit/discovered source combinations.
- Commands without an explicit package preserve full-workspace closure semantics, and missing package requests fail with the existing typed error.
- A managed wrapped package command reaches a terminal validation-copy run without scanning the unrelated Editor asset.

## 禁止临时方案

- 不回滚已集成快照来掩盖普通测试失败；应通过前向修复返回 `fixed-*` 记录。
- 不得添加别名、兼容垫片、静默回退、测试旁路或调用点特例。
- 不得把由旧 daemon planner 生成的 Cargo 结果当作未提交 planner 的验证证据。

## 修复结果与回传

Forward implementation is locally GREEN and independently reviewed: focused external closure 1/1, full `test_validation_copies` 10/10 in 620.809 seconds, py_compile/diff-check pass, Critical 0 / Important 0. The source is intentionally still `open`: ticket `57103a7429b3442aab759386517c4235` binds the final three-file manifest but was submitted before the daemon loaded those Python modules, so it cannot close the planner failure. The next legal sequence is scoped maintenance commit, controlled rollover, then a new HEAD-aligned managed locked/offline package check under FIFO.

After the committed planner loaded, ticket `57103a7429b3442aab759386517c4235`
reached the selected-package closure but failed at `owned_overlay` because its Session
attribution still named the pre-commit bytes. Copy
`d42a8f1132ce4453833a99d241cacb5a` durably recorded
`validation_copy_attribution_stale`, but lost the exact overlay path as
`error_path=null` / `{}` details. The Session then used the public heartbeat, exact
lease claim and `baseline attribute` protocol to bind all three current hashes at
epoch 440; replacement ticket `035c9dfc765e4735b29c6372224b2973` preserves source
manifest `b7f5f161e8560abca601a12310096f27331c10db9f8a4c99eeef0a3f8c64e44a`
and remains FIFO-managed. The lower diagnostic repair adds canonical `details.path`
to stale, missing and reappeared owned-overlay errors so asynchronous materialization
persists an actionable `errorPath` without parsing messages or changing admission.
RED reproduced `error_path=None`; focused GREEN passes 4/4 across all three error
branches and the durable Cargo worker projection. This additional diagnostic evidence
does not replace the required terminal managed package run.
