---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/01/2026-09-13-hierarchy-index-rebuild-reuse.md
  - docs/plans/optimize/zircon_editor/01/2026-09-13-hierarchy-control-id-arc-sharing.md
related_records:
  - docs/plans/astra/features/editor/730-hierarchy-projection-single-pass.md
  - docs/plans/astra/features/editor/731-hierarchy-index-rebuild-reuse.md
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/scene_hierarchy_projection.rs
tests:
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/scene_hierarchy_projection.rs
  - tools/tests/test_editor_scene_hierarchy_control_arc_sharing_performance_contract.py
  - tools/tests/test_editor_scene_hierarchy_generation_index_performance_contract.py
---

# Editor hierarchy control-id Arc backing reuse

`SceneHierarchyProjectionState` keeps two control-id indexes for retained
hierarchy routing. The indexes now share each populated identifier's string
backing through `Arc<str>`: one conversion is made per input control, the
forward map receives a cheap handle clone, and the reverse map owns the same
allocation. Borrowed `&str` lookups and all row/control semantics remain
unchanged.

## Plan completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Editor01 hierarchy control routing | Share control-id backing storage across both reusable indexes, removing the duplicate string allocation on the distinct-ID path. | TDD RED/GREEN source/pressure contract `3/3`, repaired generation-index contract `9/9`, in-file `Arc::ptr_eq` regression, and scoped Rustfmt; managed Cargo/Release allocation and latency evidence remain pending. | implemented_pending_validation |

## Complexity and allocation boundary

With `N` populated distinct controls, string payload allocations fall from
`2N` to `N`; both hash-table entries and borrowed lookup behavior remain. The
model excludes allocator metadata, hash-table growth, and duplicate-key
deduplication, and is therefore a deterministic allocation comparison rather
than a product benchmark.

## Local evidence

- The new source contract was RED before the `Arc<str>` conversion and is GREEN
  at `3/3` after implementation.
- The existing hierarchy generation/index contract is GREEN at `9/9` after its
  key-type assertions were updated.
- The focused source batch passes `33/33` across eight hierarchy, routing, and
  adjacent Runtime hit-grid modules; the single-process non-tooling
  Runtime/Editor performance-plus-pressure loader covers `553` modules and
  passes `2059/2059` tests in `13.480s`. This is a batched source/model receipt,
  not a managed compile or product benchmark.
- Current source fingerprints are also recorded in the companion optimization
  record:

  | File | SHA-256 |
  | --- | --- |
  | `zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/scene_hierarchy_projection.rs` | `49850F245B6677B5E747997D3ED7385026D595634C507BFC6A4A3DC38A4C70B4` |
  | `tools/tests/test_editor_scene_hierarchy_control_arc_sharing_performance_contract.py` | `1FADA62B892B33A00098F807A18C0F4293F6C38E16F49A901A13E6879673F449` |
  | `tools/tests/test_editor_scene_hierarchy_generation_index_performance_contract.py` | `232BCA3EE7017747683FBBBD98507E41BEA95FC69D785BBBF00E028DAF255554` |

- Scoped Rustfmt and Python compilation pass; the final static batch also
  passes Wiki validation (`272/272` pages), scoped diff checks, and target
  whitespace checks.
- No new coordinator request or status query was issued; the existing external
  dirty-worktree gate keeps managed validation pending.

## Managed acceptance gate

Keep this feature at `implemented_pending_validation` until the next
owner-attributed batched Windows Release validation proves compilation,
routing parity, allocation/RSS behavior, and hierarchy latency thresholds.
