---
title: Runtime200 Hover Diff Membership Scratch
category: zircon_runtime
report_id: Runtime200-hover-diff-membership-scratch-2026-09-14
date: 2026-09-14
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime200 Hover Diff Membership Scratch

## Scope

Pointer hover routing compares the newly hit stacked path with the previous
path on every pointer event. The large-path branch previously constructed a
fresh `HashSet` for membership on every diff, even though the surface owns the
entire event route and the membership table is only scratch state.

## Change

- Add a surface-owned, non-serialized `UiSurfaceHoverDiffScratch` with a
  retained membership `HashSet<UiNodeId>`.
- Move the production route to `hover_diff_with_scratch`; it clears the
  retained table and calls `reserve` only when the required path exceeds warm
  capacity, then restores the scratch to the surface.
- Preserve the equal-path fast path and the bounded linear comparison branch
  for small paths. A defensive capacity ceiling drops an unusually large table
  instead of retaining pathological input history.
- Keep entered/left ordering and all pointer capture, pressed, click, and focus
  semantics unchanged. The old allocating helper remains available to its
  focused compatibility tests; production no longer calls it.

## Complexity boundary

For large hover paths, steady-state membership storage is reused across route
events: the deterministic one-million-event model changes the membership-table
allocation count from one per event to one warm allocation. Small/equal paths
retain their existing zero-membership-table fast paths. This is an allocation
and hot-path reduction, not a managed CPU/RSS or product-latency result.

## TDD and local evidence

- The new source contract was run RED before implementation and is GREEN at
  `4/4`; the existing hover-diff and hover-pressure contracts run with it in a
  single focused invocation at `13/13`.
- The post-change one-process Runtime/Editor performance-contract and pressure
  loader covers `557` modules and passes `2072/2072` in `13.644s`; this is local
  source/model evidence only.
- A subsequent all-current Runtime/Editor Python contract invocation covered
  `869` modules and passed `3557/3557` tests in `337.155s` in one process.
  This broad source-contract receipt remains local evidence and excludes the
  deferred tooling/WOC scope.
- A Rust behavior regression covers disjoint large paths and stable scratch
  capacity. The ignored Release comparison is marked
  `RUNTIME200_HOVER_DIFF_MEMBERSHIP_SCRATCH_BENCH_V1` and uses 16,384 events
  per sample across 11 alternating samples; it is intentionally not claimed as
  passing until the managed Release lane runs.
- Python compilation, scoped Rustfmt, whitespace/diff checks, and Wiki
  validation (`272/272` pages, zero errors) pass after the final batched
  Runtime/Editor contract run. Wiki reports one pre-existing unrelated
  metadata warning for a missing `.codex` skill path.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/surface/surface.rs` | `26FA9866478B63BE830181ED3F126B946935DBBE86A550C25385ADC804650CC7` |
| `zircon_runtime/src/ui/surface/surface/event_routing.rs` | `DA85539C6EEED66D65EB991F3C8078CB1F8AECE8F2A99C0FD703C95A6D9CDB36` |
| `tools/tests/test_runtime_pointer_hover_membership_scratch_performance_contract.py` | `4A960805A0816DAC172B7C1891699ADE5B46EB60AB7CB10C5632E3AE27E546BD` |

## Managed gate

This slice joins the existing owner-attributed Runtime/Editor batch. Managed
Windows Cargo/Release compilation, allocation receipts, and Runtime pointer
input p50/p95/p99 evidence remain pending. The prior admission was rejected by
the external dirty `E:\Git\zr_vm` worktree and overlay/manifest ownership
guards; no new coordinator request or status query was made for this slice.
