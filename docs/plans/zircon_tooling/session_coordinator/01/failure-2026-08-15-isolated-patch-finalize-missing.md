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
  - tools/zircon-session.ps1
  - tools/session_coordinator/isolated_patch_contract.py
  - tools/session_coordinator/isolated_patch_checkout.py
  - tools/session_coordinator/isolated_patch_finalize.py
  - tools/session_coordinator/git_finalize.py
  - tools/session_coordinator/cli.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/tests/test_isolated_patch_finalize.py
  - tools/session_coordinator/tests/test_powershell_wrapper_arguments.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_isolated_patch_finalize
  - python -m unittest tools.session_coordinator.tests.test_powershell_wrapper_arguments
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

Production implementation was committed as
`123b0376193a0eeb21c51ea754c0c600da890f9f` and loaded by healthy schema-62 daemon
`0516959fe6184e6dbbb83e82ff7e4e09`. The first real request
`824ec1dc29114dcaa4ef4de4a5d3b166` failed before derivation because its caller patch
header was malformed; it did not publish or modify shared state.

The corrected owner-issued request `67135178e5664f4185cafb6b009492c2`
(`finalize_requests` row `5fbecb6326e14818be2c363abfff420a`) crossed patch
application and failed during `checkout-index`. The default Windows `%TEMP%` prefix
made the two tracked reflection-probe resources ending in
`face_0064_mips_07.{zribl,zcube}` exceed the Git for Windows filename limit. On
Windows the service now creates an atomic random child under an ordered set of
existing user-writable bases, computes the longest indexed path in UTF-16 code units,
and accepts only a root whose maximum is at most 248. It revalidates the child as a
non-reparse directory immediately before checkout, sets a Git discovery ceiling, and
removes only the child it created. The same production relative paths are covered by
a deterministic over-budget-to-short-root checkout/finalize regression.

Independent reproduction also found that a PowerShell object pipeline cannot carry
`--patch-stdin` byte-exactly because it may encode line records and append a newline.
`tools/zircon-session.ps1` now rejects that flag with an explicit `--patch-file`
instruction. Direct Python CLI binary stdin remains unchanged, and `--patch-file`
remains byte-exact.

The Windows checkout hardening was committed as
`a82485da17b30856b2e3e60f306b7e51c0b012b9` and loaded by a healthy successor.
Render17 then replayed the owner-issued exact patch through production. Durable
finalize row `6fa51d7defe5476b94145801d761093e` reached `committed` and published
`4ef70ac5b3bcef55f8c3eb77c929e85b4691ed0d`, whose only product change is the
required `viewport_products: Default::default(),` insertion. The derived
validation used the isolated rustfmt gate; the mixed live worktree bytes and the
foreign staged projection were preserved by the production path.

Open state: `Coordinator implementation and real replay complete / Render17 return
pending`. The fixing plan has no remaining code or replay work. The Render17 origin
owner must still accept the durable evidence and execute the ordinary lifecycle
return; Coordinator does not claim that foreign owner step.

## 产出记录与时间

- 2026-08-15 | status: open | RED contract records exact derived-blob identity,
  target CAS, isolated validation, 283-path staged projection preservation, and
  byte-exact mixed-worktree preservation before production implementation.
- 2026-08-15 | status: open | Independent review closed index-CAS, same-OID branch
  switch, durable validation/recovery, validation-environment leakage, mode-change,
  and final worktree-drift windows. Focused recovery and publication regressions are
  green; managed implementation commit and schema-62 rollover completed.
- 2026-08-15 | status: open | Real owner request `67135178...` reproduced Windows
  checkout failure 128 on the two 192/193-character reflection-probe resource paths.
  The exact-path regression is RED under `%TEMP%` and GREEN after a deterministic
  over-budget candidate falls back to a private short root. Reparse rejection,
  child-only cleanup, and PowerShell 7/5.1 `--patch-stdin` rejection are covered.
- 2026-08-15 | status: open | Windows hardening committed as `a82485da1`; the
  successor accepted production replay finalize `6fa51d7d...`, which committed the
  exact one-line Render17 patch as `4ef70ac5b`. Coordinator work is complete; only
  the origin-owner fixed return remains open.
