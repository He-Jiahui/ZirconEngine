---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01/2026-08-26-projection-composition-hash-index.md
related_records:
  - docs/plans/astra/features/editor/943-editor01-invalidation-scope-hash-index.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/layouts/views/view_projection/projection_composition.rs
tests:
  - zircon_editor/src/ui/layouts/views/view_projection/projection_composition/hash_index_tests.rs
---

# Editor944 Editor01 Projection-Composition Hash Index


| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Source/output row mapping | composed `(node_id, control_id)` identities are admitted once into a capacity-sized borrowed-key `HashMap`; source rows resolve through hash lookup while source order, unmatched rows and first duplicate wins remain unchanged. | Behavior test covers reversed rows, unmatched source and duplicate identity; source contract requires `with_capacity`, `or_insert` and borrowed lookup and rejects nested `.position()` scanning. |
| 性能门禁 | 2,048 reversed source/composed rows change candidate visits from quadratic nested scans to `O(S + C)` indexing and lookup. | ignored marker `EDITOR01_PROJECTION_COMPOSITION_HASH_INDEX_BENCH_V1` requires hash-index P95 ≤60% of nested-linear P95; managed Editor01 Release receipt remains pending. |


- `projection_composition.rs` and hash-index tests pass exact Rustfmt/source checks.
- No tooling changes; the existing optimized source remains pending managed performance evidence.

### Grouped validation submission (2026-09-25)

The Editor01 marker is included in the grouped Editor Release lane PTY
`51515`, alongside Runtime development PTY `8644`, Editor development PTY
`21683`, and Runtime02 Release PTY `11980`. The wrappers remain
intentionally unpolled; no compiler, functional test, or P95 result is inferred.

### Replacement grouped validation submission (2026-09-25)

The narrow `editor01` filter was superseded because adjacent Editor01 markers
use historical batch names. The replacement broad-prefix Editor Release lane
uses PTY `35753`, with Runtime development PTY `76539`, Editor development PTY
`36565`, and Runtime02 Release PTY `51493`. These wrappers remain intentionally
unpolled; compiler, functional-test, and P95 receipts are still pending.
