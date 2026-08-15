Plan: docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
Milestone: M3
Status: source_bound_validation_pending_after_m1
Files: ["docs/plans/zircon_editor/editor/14/2026-08-11-m3-progress-snapshot-current-source-manifest.md", "docs/plans/zircon_editor/editor/14/2026-08-11-primary-progress-generation-architecture.md", "zircon_editor/src/core/jobs/mod.rs", "zircon_editor/src/core/jobs/progress.rs", "zircon_editor/src/core/jobs/progress/primary_generation_tests.rs"]
Depends-On-Milestones: ["M1"]
Depends-On-Failures: ["docs/plans/zircon_editor/editor/14/failure-2026-07-17-job-pump-budget-and-pending-scan.md"]
---

# Editor14 M3 Progress Snapshot Current-Source Manifest

## Scope

This successor manifest owns the primary-progress generation optimization added
after the Coordinator-bound M1 source snapshot. The source publishes
`published_primary_generation` as an `AtomicU64` negative probe. Stable
equal-generation reads therefore bypass the state mutex; only first, stale,
changed, or future observations lock and can clone the primary snapshot.

This exact manifest does not own `core/notifications/progress/center.rs` and
does not claim that the separate explicit-id notification lookup is integrated.
That consumer currently carries foreign notification capacity and lifecycle
changes and must close through its owning Session. M3 does not modify
notification ownership, job lifecycle state, scheduler admission, or Editor02
message delivery.

The manifest implements the source side of the required primary-progress
generation architecture. The source owns visible-primary change facts,
preflights the next checked generation before changing the projection, and
publishes the exact state generation mirror with `Release` before unlocking.
EditorUI08 owns retained cursor consumption and presentation invalidation. The
staged hard cut removes the legacy unconditional primary-snapshot path after
the UI consumer migrates; this source manifest does not claim that UI migration
is already complete.

## Static Evidence

- Generation behavior coverage forces an atomic-mirror mismatch to exercise
  the locked authoritative comparison and uses a two-party startup barrier to
  prove that a stable cursor returns `None` while the state mutex is retained.
- Overflow coverage drives registration, primary progress, terminal transition,
  and primary removal at `u64::MAX` and requires every projection to remain
  unchanged after the checked increment panics.
- All four visible-primary mutation routes preflight the next generation before
  mutating, then publish that exact value with `Release` while the guard is live.
- `rustfmt --edition 2024 --check` and scoped `git diff --check` passed.
- The first atomic review reported `Critical/Important/Minor = 0/1/3`; all four
  findings were forward-corrected. The successor review reported `0/2/3` and
  drove the exact-scope correction, folder-backed test extraction, exact-step
  assertion, and documentation fixes. A third independent read-only review of
  the current five-path snapshot reports `0/0/0`.
- The primary-progress architecture records the source/UI ownership split,
  atomic fast path, locked mismatch comparison, terminal transition semantics,
  and managed non-`C:` Windows profile gate before the UI consumer migration.

## Validation Boundary

The Coordinator already bound the 24-path M1 manifest before this source
change. This M3 manifest must receive its own managed current-source
validation after M1 reaches a terminal result; it must not reuse M1's run,
validation receipt, or immutable file list. No Cargo command was run locally.

## Failure State

The JobPump failure remains open. This optimization removes the stable
equal-generation progress mutex acquisition, but it does not satisfy the
outstanding lifecycle delivery reservation, Editor02 lossless producer, WPR,
or scale-evidence requirements. The separate notification active-table lookup
is not accepted by this manifest.
