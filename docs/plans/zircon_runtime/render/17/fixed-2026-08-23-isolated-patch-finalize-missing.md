---
handoff_kind: fixed
status: fixed
created_at: 2026-08-15
resolved_at: 2026-08-23
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
  - python -B -m unittest tools.session_coordinator.tests.test_isolated_patch_finalize -v
  - python -B -m unittest tools.session_coordinator.tests.test_powershell_wrapper_arguments -v
---

# Coordinator01: isolated HEAD-derived patch finalization fixed

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/render/17-performance-and-profiling.md`
- 来源执行切片：Render17 exact one-line viewport-products repair over a foreign mixed worktree file
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns isolated derived-blob validation, Git publication, and recovery.

## 失败现象与复现证据

The Coordinator could not derive, validate, and publish one explicit patch from a
known HEAD blob without reading or changing a foreign mixed worktree file. Ordinary
finalization and integration candidates therefore could not preserve compile and
Git identity for the one-line Render17 repair.

## 最低共享层根因

The public finalization paths sealed live worktree bytes. They had no contract for
an expected HEAD/blob plus exact patch bytes, a private checkout/index, validation
of the derived blob, and a compare-and-swap ref update that left the shared
worktree and index unchanged.

## 架构修复验收

- Derive the target from expected HEAD/blob and exact patch bytes in a private tree.
- Bind validation and publication to the derived blob and patch hash.
- Use a Windows checkout root short enough for the repository's longest tracked paths.
- Preserve the shared mixed worktree bytes and staged-index fingerprint exactly.
- Reject HEAD/blob drift, malformed patch input, unvalidated derived bytes, and live index-lock ownership.

## 禁止临时方案

- Do not stage the mixed worktree file, restore foreign bytes, or use direct Git plumbing outside the Coordinator.
- Do not reuse a compile ticket for a different live overlay or derived blob.
- Do not weaken long-path, stale-lock, validation, or expected-parent CAS checks.

## 修复结果与回传

- 根因：Coordinator finalization lacked an isolated expected-HEAD/blob plus explicit-patch publication contract, and the first private checkout root exceeded Windows path limits.
- 架构修复：Commits `123b0376193a0eeb21c51ea754c0c600da890f9f` and `a82485da17b30856b2e3e60f306b7e51c0b012b9` added service-owned isolated maintenance finalization, byte-exact patch input, private validation/index state, short-root selection, and expected-parent CAS publication.
- 验证：Render17 finalize `6fa51d7defe5476b94145801d761093e` committed `4ef70ac5b3bcef55f8c3eb77c929e85b4691ed0d` with only `viewport_products: Default::default(),`; schema-65 ticket `e48636e4fa324b65973158358b756256` passed all `17/17` isolated-patch tests from immutable copy `77fddbd58f8b4e6fb2089c62f0ff0c43` after cleanup-ordering commit `7762880fd1d8db3d3872888ba8377910177574af`.
- 回传：Render17 may consume the committed one-line repair without recreating the patch; the foreign mixed worktree bytes and shared staged projection remain unchanged.
