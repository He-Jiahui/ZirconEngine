---
handoff_kind: failure
status: open
created_at: 2026-08-15
summary_slug: isolated-patch-finalize-missing
origin_plan: docs/plans/zircon_runtime/render/17-performance-and-profiling.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_runtime/render/17
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
failure_scope: cross_plan
related_code:
  - tools/session_coordinator/isolated_patch_contract.py
  - tools/session_coordinator/isolated_patch_finalize.py
  - tools/session_coordinator/git_finalize.py
  - tools/session_coordinator/cli.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/tests/test_isolated_patch_finalize.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_isolated_patch_finalize
---

# Coordinator01: mixed worktree files cannot publish one validated HEAD-derived patch

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/render/17-performance-and-profiling.md`
- 来源执行切片：UI12 current-source construction repair in a foreign mixed worktree file
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns the only managed Git mutex, temporary index, CAS publication, and durable finalize ledger.

## 失败现象与复现证据

The required product repair is one insertion in
`zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework_construction/construct.rs`.
The live worktree file also contains foreign IBL/cache and rustfmt changes. Ordinary
maintenance finalize stages the entire live file. Integration candidate submit also
hashes the entire live file and requires a compile ticket that did not validate a
separately derived blob. Patch enqueue similarly treats the mixed live target as its
base and mutates the shared worktree.

The immutable source evidence is base HEAD
`fe449298eb100bd6e91f69f9c7d03131baf911b6`, target blob
`1582622509ace3fd66c4a20bf5a3dc25799dbfc5`, and a patch whose only change inserts
`viewport_products: Default::default(),` after `graphics_debugger,`. Current HEAD may
advance only while that target blob remains exact.

## 最低共享层根因

The Coordinator has no command that derives a commit blob from an immutable HEAD
blob plus an explicit patch while keeping the shared worktree and foreign staged
projection untouched. Reusing a compile-passed candidate would break validation
identity because that ticket describes the live mixed overlay, not the derived blob.

## 架构修复验收

- Expose an explicit isolated maintenance finalize command; do not label it a
  compile-gated candidate and do not accept a compile ticket.
- Bind the operation to a target Session's live lease, one repo-relative target,
  expected ancestor HEAD, expected target blob, exact patch bytes, and non-empty
  maintenance validation commands.
- Build and validate the derived blob in a temporary index/checkout. Reject a patch
  touching another path, changing mode, or observing target blob drift.
- Persist `baseHead`, `baseBlob`, `patchHash`, `derivedBlob`, current parent HEAD,
  validation commands, staged projection fingerprints, and the resulting commit.
- Publish with an expected-parent `update-ref` CAS under the Coordinator Git mutex.
- Prove the mixed worktree bytes and the complete foreign staged path/content
  projection are unchanged; only align the target index entry to the new HEAD.
- Preserve ordinary finalize interruption recovery through the durable
  `finalize_requests` ledger.

## 禁止临时方案

- Do not stage, restore, overwrite, format, or otherwise edit the mixed product file.
- Do not use direct Git plumbing outside the Coordinator, a caller-controlled temp
  index, or a live-overlay compile ticket.
- Do not broaden the patch to include IBL/cache or rustfmt hunks.
- Do not accept a changed target blob merely because the original base HEAD remains
  an ancestor.

## 修复结果与回传

Production implementation and focused regression evidence are complete. The managed
Coordinator maintenance commit, daemon rollover, and first Render17 immutable patch
consumption are intentionally pending before this record can transition to fixed.

## 产出记录与时间

- 2026-08-15 | status: open | RED contract records exact derived-blob identity,
  target CAS, isolated validation, 283-path staged projection preservation, and
  byte-exact mixed-worktree preservation before production implementation.
- 2026-08-15 | status: open | Independent review closed index-CAS, same-OID branch
  switch, durable validation/recovery, validation-environment leakage, mode-change,
  and final worktree-drift windows. Focused recovery and publication regressions are
  green; managed commit and first immutable Render17 consumption remain pending.
