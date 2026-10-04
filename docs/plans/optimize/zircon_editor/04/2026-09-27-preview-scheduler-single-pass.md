---
title: Editor04 Preview Scheduler Single Pass During Catalog Prepare
category: zircon_editor
report_id: Editor04-preview-scheduler-single-pass-2026-09-27
date: 2026-09-27
implementation_status: implemented_pending_validation
validation_status: managed_cargo_and_release_pending
---

# Editor04 preview scheduler single pass during catalog prepare

## Product path and observable contract

`DefaultEditorAssetManager::sync_from_project_with_runtime_generation` prepares a
new Runtime-authoritative catalog before publication. Its common matching-generation
branch previously built a scheduler from every projected record, merged current
preview results, then discarded that scheduler and built another one from the same
catalog. The O(N) merge and second full scan occurred while the editor state read
lock was held.

The preparation now snapshots the matching catalog generation as an `Arc` under
the read lock, releases the lock, merges preview results, and builds the scheduler
once. Matching source hash and metadata path still preserve completed preview
state and artifact location. A changed source hash remains dirty and is admitted
for a fresh preview. A concurrent preview publication after the snapshot is
detected by the existing commit identity check; a same-revision publication
rebases and merges the latest preview results before retry. Runtime authority
and publication order are unchanged.

## Regression and scale comparison

`editor04_preview_merge_keeps_matching_results_and_preserves_new_source_demand`
checks both preview outcomes and scheduler admission.
`editor04_preview_snapshot_rebases_a_later_preview_publication` checks that a
same-revision preview publication after the short snapshot survives the rebase.
The ignored Windows Release
benchmark `EDITOR04_PREVIEW_SCHEDULER_LARGE_CATALOG_BENCH_V1` compares the prior
two-scan prepare stage with the current one-scan stage on 10,000 and 100,000
in-memory catalog records. Both variants perform the same current-preview merge;
the prior first scheduler is materialized so Release optimization cannot erase
the baseline work. Each variant runs five warmup pairs and 31 alternating
measurement pairs. The benchmark retains raw acquisition-order samples and
prints nearest-rank p50/p95/p99 for both whole prepare stage and read-lock
hold, alongside OS, architecture, and package version.

The pending p95 gates are at most 110% of the prior stage at 10,000 records and
95% at 100,000 records; read-lock hold p95 must be at most 10% of the prior path
at both sizes. The smaller prepare bound tolerates short-run noise; the larger
bound requires a measurable gain from removing the full dirty scheduler scan.
The fixture models a mass-dirty catalog and uses the actual catalog generation
lookup and preview merge code. The lock timer starts after read-lock acquisition
and ends after releasing the guard; it does not include contended waiter latency.
It excludes project scan/import,
catalog details, folder construction, broadcast, retained-host frames, and disk
I/O. It is a catalog prepare-stage comparison, not a product frame benchmark.

## Validation status

Scoped Rustfmt and whitespace checks are the local static gate. The behavior
regression and ignored Release comparison join the combined managed Editor Cargo
batch; no isolated Cargo result is claimed here. Editor04 still needs 100,000
and 1,000,000 asset product corpora, prepare/commit and memory receipts, watcher
storm end-to-end traces, and an explicit UI frame p99 budget before the parent
plan's product performance criteria can be accepted.
