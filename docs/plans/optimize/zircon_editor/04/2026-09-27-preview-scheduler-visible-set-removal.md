---
title: Editor04 Preview Scheduler Write-Only Visible Set Removal
category: zircon_editor
report_id: Editor04-preview-visible-set-removal-2026-09-27
date: 2026-09-27
implementation_status: implemented_pending_validation
validation_status: managed_cargo_and_release_pending
performance_status: release_measurement_pending
---

# Editor04 preview scheduler visible set removal

## Product defect and contract

The PreviewScheduler portion of Editor04 P1-28 retains a private `visible`
`HashSet<AssetUuid>` on every visible request before checking dirty state or the
64-job admission cap. The set has no read consumers. The retained-host Browser
and Activity preview refresh paths request current candidates with `visible=true`
and do not send `false` for candidates that leave the viewport. Visiting 100,000
distinct assets can therefore retain 100,000 UUIDs in this unused set until the
scheduler is replaced, including candidates rejected by the admission cap.

The patch removes only that resident field and its insert/remove operations.
The request's `visible` argument remains an immediate admission condition:
`false` consumes no dirty state and does not cancel an existing token. Dirty
tracking, the 64-job cap, token allocation, ownership and completion remain
unchanged. Visibility still belongs to the host's viewport-bound demand; this
change does not introduce a queue, fairness policy, preview provider or retry.

## Regression and matched Release comparison

- `editor04_preview_scheduler_invisible_request_preserves_dirty_and_current_token`
  checks that an invisible dirty asset remains eligible for a later visible
  request, that a subsequent invisible request preserves the admitted token,
  and that completion does not cause a clean asset to be admitted again.
- `editor04_preview_scheduler_has_no_resident_visible_set` compares the scheduler
  structure footprint with the legacy structure and requires the removal of one
  resident HashSet. This detects the old unused field without adding a production
  introspection API; it does not measure heap bytes or process RSS.
- Ignored Release
  `editor04_preview_scheduler_visible_set_100k_release_benchmark` calls the real
  optimized request implementation and a test-only copy of the complete prior
  request implementation on the same ordered 100,000 deterministic UUIDs. Every
  pair begins with equivalent cloned dirty sets (same hash seed/capacity), empty
  in-flight maps with the same hash seed, and an empty legacy visible set. UUID
  construction, initial dirty-state construction/cloning, assertions and
  scheduler destruction occur outside the timer. The timer covers only the
  complete request sweep, including legacy visible-set growth, both variants'
  first 64 admissions and the remaining cap-rejected requests; no jobs complete
  during this sweep.

The fixture performs five warmup pairs and 31 measured pairs, alternating the
first implementation in each pair. It retains raw `(legacy_ns, current_ns)` pairs
in acquisition order, prints nearest-rank p50/p95/p99 plus OS, architecture and
package version under `PERF_RESULT EDITOR04_PREVIEW_VISIBLE_SET_100K_BENCH_V1`,
and requires **current request-sweep p95 <= 80% of legacy p95**. With 31 samples,
p99 is the maximum observation and is diagnostic. After each pair the legacy
visible count must be 100,000, both variants must admit the same 64 UUIDs with
owned tokens and retain the same 99,936 dirty UUIDs. The new scheduler has no
resident visible set; intentional dirty tracking can still be proportional to
catalog size.

The local 80% target remains pending actual Release measurements. This sweep
does not measure the manager's locks, catalog lookup/cloning, job submission,
preview generation/completion/refill, chrome construction, paint or native
present. In particular it does not prove the separate manager completion path's
Release admission-release contract. Editor04 viewport-priority/fairness and
100k/1m catalog memory/frame gates, and Editor57 native scroll/action G37 p99,
remain open.

## Validation manifest

| Gate | Scope | State |
|---|---|---|
| Source | `zircon_editor/src/ui/host/editor_asset_manager/preview.rs`; `zircon_editor/src/ui/host/editor_asset_manager/preview/visibility_tests.rs` | Rustfmt `--check`, scoped `git diff --check`, UTF-8/LF, final newline and trailing-whitespace checks passed; Rust compilation pending grouped managed validation |
| Behavior | `editor04_preview_scheduler_invisible_request_preserves_dirty_and_current_token`; `editor04_preview_scheduler_has_no_resident_visible_set`, plus existing scheduler capacity and stale-token tests | Grouped managed Editor test pending; no per-slice Cargo run |
| Release | ignored `editor04_preview_scheduler_visible_set_100k_release_benchmark`, marker `EDITOR04_PREVIEW_VISIBLE_SET_100K_BENCH_V1` | Five alternating warmup pairs, 31 alternating measured pairs; p95 <= 80% gate pending actual output |
| Product | Editor04 large-catalog memory/prepare/commit and watcher UI p99; Editor57 G37 native 100k scroll/navigation/selection/action | Pending; scheduler-only request sweep does not establish these gates |
