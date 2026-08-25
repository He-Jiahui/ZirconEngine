---
handoff_kind: fixed
status: fixed
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
resolved_at: 2026-08-26
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
The first narrowed topology retained only unselected manifests. Managed copy
`946d4be03e1b4819956dea3d4b2b7b0a` then reached Cargo, which rejected the
workspace because `zircon_runtime/Cargo.toml` declared a default library target but
the topology-only copy omitted `zircon_runtime/src/lib.rs`. Cargo topology therefore
requires every metadata-declared target entrypoint while still excluding the rest of
an unselected source tree and its compile-time resources.

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

- 根因：The planner conflated complete workspace topology with selected build source
  closure. Narrowing source roots exposed two additional lower layers: topology-only
  external packages need ancestor workspace files and immutable mount identity, and
  every topology-only local/external package needs its Cargo metadata target entrypoint
  to survive workspace manifest parsing.
- 架构修复：Selected packages and transitive build dependencies retain complete tracked
  source and compile-time resources. Unselected packages retain only Cargo manifests,
  ancestor Cargo/toolchain inputs and `targets[].src_path`; sibling sources and their
  includes remain excluded. Pinned external trees are cached independently from
  per-manifest topology, mount identity is `repo_root + commit`, and owned-overlay
  failures persist their exact path.
- 验证：Target-entrypoint RED covered unselected workspace members, manifest-only
  excluded optional path dependencies, external metadata/manifest double registration,
  and sibling-source exclusion. Final `test_validation_copies` passed 10/10 in 89.807
  seconds; py_compile and diff-check passed. Independent review reported Critical 0 /
  Important 0; its fixture-consistency Minor was resolved before commit. Maintenance
  finalizer committed exact source snapshot
  `c1d1e76b22915969da8b3e732d4744778c12662e`; controlled rollover action
  `2e3f7c1e23354df1b0f00c1be27e0a3e` loaded healthy schema-67 successor
  `4139b1e4c17a43fc9f9c8f6bcea14c66`.
- 生产证据：HEAD-aligned ticket `64f834a26e464b529e51427672bb2e9c`
  bound source manifest
  `cf201c33bf4f774ae60521d619fe7f0445346e8b65be0d0c53e618af016142fa`.
  Copy `297f33e7414049ceb376fbdc599d7d27` materialized once with immutable input
  manifest `3c4eeca21b9d5f8f2ec4e10ddfd6a6bd5bbfd8c302bfd04206600828d44a5571`;
  managed Cargo job `b12344f1c6ff46e185074fbc70ef7892` started and terminalized.
  The run no longer failed on an absent workspace target or unrelated Editor resource;
  the terminal run reached offline dependency resolution and exited 101 only because the local
  crates.io cache lacked `image`. This record does not claim the Cargo check passed.
- 回传：The package-scoped closure failure is fixed. Its stated managed acceptance was
  a terminal validation-copy run beyond unrelated-source scanning and manifest parsing;
  the remaining offline-cache miss is downstream environment evidence, not a closure
  regression.
