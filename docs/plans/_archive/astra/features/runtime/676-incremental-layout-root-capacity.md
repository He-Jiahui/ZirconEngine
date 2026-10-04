---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
related_code:
  - zircon_runtime/src/ui/layout/pass/incremental.rs
tests:
  - zircon_runtime/src/ui/layout/pass/incremental.rs
---

# Incremental Layout Root Capacity

Incremental layout root filtering now allocates its output vector from the
already-built candidate-set length, and root-size arrangement reserves the
additional retained-tree root bound before extending and deduplicating. Empty
incremental passes remain zero-capacity; root ordering, ancestor suppression,
resize sorting, deduplication, and all layout traversal behavior are unchanged.

## Plan Completion List

| Batch | Work | Status | Validation evidence |
| --- | --- | --- | --- |
| Runtime11A / incremental layout roots | Bound filtered-root and root-resize arrangement vector growth from existing inputs | implemented_pending_validation | Focused Rust source regression locks both reservations and the retained sort/dedup path. Scoped Rustfmt, scoped diff check, and source invariants pass. Combined Runtime/Editor static-contract validation and managed Rust remain pending; no coordinator state was polled. |

## Complexity Boundary

Root discovery remains `O(D log D)` for the existing ordered candidate set and
ancestor checks. This slice removes geometric growth of the final root vectors
without changing the selected root set or traversal complexity. It does not
claim product CPU, RSS, allocation, p50, p95, or p99 improvement before managed
release evidence.

## Source Snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/layout/pass/incremental.rs` | `D411F08F4E833796203DE2E2E1CF2138F86313622180CF44058043CC977BEBCF` |

## Managed Gate

No direct Cargo command or coordinator status query was run for this slice.
The next immutable multi-task validation input must compile the focused
incremental-layout regressions with the Runtime/Editor batch and compare a
Windows Release root-resize allocation/time capture. Until that batch succeeds,
this record remains `implemented_pending_validation` and makes no product
performance claim.
