---
doc_type: completion-list
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/09h2/2026-08-26-borrowed-active-resource-index.md
  - docs/plans/optimize/zircon_runtime/09h2/2026-08-26-borrowed-executor-metadata.md
  - docs/plans/optimize/zircon_runtime/09h2/2026-08-26-borrowed-pass-resource-index.md
  - docs/plans/optimize/zircon_runtime/09h2/2026-08-26-direct-profile-overrides.md
  - docs/plans/optimize/zircon_runtime/09h2/2026-08-26-direct-visible-snapshot-storage.md
  - docs/plans/optimize/zircon_runtime/09h2/2026-08-26-indexed-volume-registry.md
  - docs/plans/optimize/zircon_runtime/09h2/2026-08-26-inline-camera-history-layers.md
  - docs/plans/optimize/zircon_runtime/09h2/2026-08-26-inline-volume-defaults.md
  - docs/plans/optimize/zircon_runtime/09h2/2026-08-26-owned-volume-layer-masks.md
  - docs/plans/optimize/zircon_runtime/09h2/2026-08-26-sorted-volume-fast-path.md
  - docs/plans/optimize/zircon_runtime/09h2/2026-08-26-stable-terminal-resource-cache.md
  - docs/plans/optimize/zircon_runtime/09h2/2026-08-26-zero-clone-effect-disable.md
related_records:
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
---

# Runtime09H2 · post-process CPU completion list

The Runtime09H2 post-process slices are implemented in the current source and
retain the existing ordering, checksums, resource ownership, and GPU-facing
contracts. This list records the completed optimization boundaries while
keeping managed Cargo, Release, allocator, and renderer-product acceptance
open. Tooling migration is intentionally out of scope.

| Plan slice | Optimization boundary | Local plan evidence | Acceptance status |
| --- | --- | --- | --- |
| Borrowed active-resource index | Replace cloned `BTreeSet<String>` names with one reserved borrowed `HashSet<&str>` for 4,096-effect membership. | Plan records 99.992% allocation reduction and latest P50/P95 reductions of 71.231%/55.026%. | Managed 4/4 source contracts, 19/19 Rust tests, allocation and percentile gates pending. |
| Borrowed executor metadata | Borrow executor-owned metadata strings at the GPU-recording boundary. | Plan records zero metadata allocations for a 29-executor chain and latest P50/P95 reductions of 65.256%/54.138%. | Managed source/test/model gates pending. |
| Borrowed pass-resource index | Use reserved borrowed resource indexes in fallback pass admission. | Plan records 99.942% allocation reduction and latest P50/P95 reductions above the 50% gates. | Managed source/test/model gates pending. |
| Direct profile overrides | Project typed volume overrides directly into final component values. | Plan records 46.429% allocation reduction with P50/P95 gates of 30%/10%. | Managed source/test/model gates pending. |
| Direct visible snapshot storage | Store the public visible snapshot directly instead of allocating an immediately unwrapped outer `Arc`. | Plan records 50% allocation reduction and P50/P95 reductions above 40%. | Managed source/test/model gates pending. |
| Indexed volume registry | Index plugin-scale volume descriptors while preserving deterministic output order. | Plan records plugin-scale P50/P95 reductions above 90%; built-in-only P95 remains explicitly caveated. | Managed source/test/model gates pending. |
| Inline camera history layers | Inline the common one-to-three-layer camera history representation and retain the shared overflow path. | Plan records zero common-path allocations and P50/P95 reductions above 90%. | Managed source/test/model gates pending. |
| Inline volume defaults | Apply the 15 built-in defaults without transient vectors while retaining the one-allocation plugin fallback. | Plan records zero built-in allocations and P50/P95 reductions above 70%. | Managed source/test/model gates pending. |
| Owned volume layer masks | Transfer layer-mask ownership once per emitted volume instead of duplicating masks. | Plan records 50% fewer layer-mask allocations and P50/P95 reductions above 35%. | Managed source/test/model gates pending. |
| Sorted volume evaluation | Use the sorted-volume fast path with early termination and no evaluation allocations. | Plan records zero optimized allocations and P50/P95 reductions above 20%. | Managed source/test/model gates pending. |
| Stable terminal resource cache | Keep warm terminal-cache entries stable and avoid hit-path shifts while preserving LRU behavior. | Plan records zero stable-path shifts and P50/P95 reductions above 30%. | Managed source/test/model gates pending. |
| Zero-clone effect disable | Use one borrowed membership index when pruning disabled effect outputs. | Plan records 99.902% allocation reduction and P50/P95 reductions above 80%. | Managed source/test/model gates pending. |

The Runtime package-wide lib-test submission includes this group together with
the Runtime09c material-index records and the current Editor package. This list
does not infer a pass from asynchronous launch; only authoritative managed
receipts may advance these rows beyond `implemented_pending_validation`.
