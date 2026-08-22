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

## Lowest shared root cause

The Coordinator could not derive, validate, and publish one explicit patch from a
known HEAD blob without reading or changing a foreign mixed worktree file. Ordinary
finalization and integration candidates therefore could not preserve compile and
Git identity for the one-line Render17 repair.

## Architecture repair

- `123b0376193a0eeb21c51ea754c0c600da890f9f` added a service-owned isolated
  maintenance finalizer bound to expected HEAD/blob, exact patch bytes, validation
  commands, shared-index fingerprints, and an expected-parent update-ref CAS.
- `a82485da17b30856b2e3e60f306b7e51c0b012b9` made the derived checkout select a
  private Windows root whose longest indexed path remains within the enforced
  UTF-16 path budget, including the real long reflection-probe resources.
- PowerShell rejects non-byte-exact `--patch-stdin`; `--patch-file` and direct Python
  binary stdin remain the explicit byte-preserving inputs.
- Recovery retains the 30-second minimum age, stable observation, and live-owner
  checks for an unowned zero-byte index lock. Tests age only their synthetic lock.

## Durable production evidence

- Render17 replay finalize `6fa51d7defe5476b94145801d761093e` reached
  `committed` and published `4ef70ac5b3bcef55f8c3eb77c929e85b4691ed0d`.
  Its only product change inserted `viewport_products: Default::default(),` while
  preserving the foreign mixed worktree bytes and staged projection.
- Cleanup protection commit `7762880fd1d8db3d3872888ba8377910177574af`
  prevented Cargo cleanup from deleting the live immutable validation copy.
- Schema-65 successor `fe6522979a994d3d84d99e10a59c822f` completed managed
  ticket `e48636e4fa324b65973158358b756256`: all 17 isolated-patch tests passed from
  copy `77fddbd58f8b4e6fb2089c62f0ff0c43`, then the copy was removed only after its
  terminal run evidence was durable.

## Resolution

Coordinator implementation, Windows checkout hardening, real owner replay, current
immutable-copy validation, and cleanup ordering are complete. Render17 may accept
this fixed record and publish the ordinary origin return without rerunning or
modifying the already committed product patch.
