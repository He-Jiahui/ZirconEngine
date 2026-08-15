# Progress Snapshot Selective Lookup Analysis

## Scope

`EditorJobProgressSource::snapshot_for_ids` is the source-side selective lookup
prepared for the retained notification projection. The intended consumer
captures only the job ids currently bound by `ProgressNotificationCenter`,
whose capacity is `MAX_PROGRESS_NOTIFICATIONS = 64`.

## Current Source Evidence

- `zircon_editor/src/core/jobs/progress.rs` now deduplicates requested ids into
  a `BTreeSet`, iterates those ids, and performs `BTreeMap::get`; source-side
  work is O(requested ids * log(active jobs)) rather than O(active jobs).
- The integrated HEAD version of
  `zircon_editor/src/core/notifications/progress/center.rs` still calls
  `jobs.snapshot()`. The shared working tree currently contains a candidate
  `snapshot_for_ids` call, but that file also carries foreign notification
  capacity, identity, and lifecycle changes and is not in Editor14 M3.
- Consequently the end-to-end optimization is not accepted until the
  notification owner commits an exact self-contained consumer manifest.
- `BTreeMap<JobId, ActiveJobEntry>` already indexes the active snapshots by
  the requested key. Iterating the sorted requested-id set and using
  `BTreeMap::get` preserves the prior JobId-ascending, duplicate-free output,
  while excluding terminal entries as before.

## Reference Boundary

Unreal's task graph exposes explicit task submission and named-thread routing
through `FTaskGraphInterface::QueueTask` in
`dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/TaskGraphInterfaces.h`.
It supports Zircon's existing boundary: jobs own authoritative lifecycle
state; retained UI consumes a bounded read model rather than scanning or
owning scheduler state. It does not prescribe a notification-map container,
so this change remains a Zircon-local read-model optimization.

## Change Contract

1. Convert the input into the existing `BTreeSet<JobId>` to retain sorted,
   duplicate-free output semantics.
2. Hold the existing active-state lock once and look up only those ids.
3. Clone a snapshot only for a requested, non-terminal entry.
4. Preserve the empty-request fast path and never alter registration,
   terminal marking, or notification-center ownership.

The source-focused regression requests ids in descending order with duplicates,
alongside unrelated and terminal active jobs. It must observe the same sorted
visible result as the old implementation, demonstrating that the new path is
key-selective without widening the retained UI contract.

## Validation Data

- Static: `rustfmt --edition 2024 --check` for the owned source and scoped
  `git diff --check`.
- Managed validation: M1 owns the source API snapshot. A later notification
  owner manifest must supply the consumer call site and its focused Cargo
  evidence; Editor14 M3 is not that authority. No local Cargo command is used
  for this slice.
- Review: the source lookup received an independent `0/0/0` review, but the
  end-to-end optimization remains pending the foreign consumer closure and
  must not be reported as an M3 deliverable.
