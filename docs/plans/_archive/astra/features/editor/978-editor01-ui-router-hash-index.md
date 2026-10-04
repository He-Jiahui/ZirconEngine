---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01/2026-08-26-ui-router-hash-index.md
related_records:
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/binding/core/router.rs
  - zircon_editor/src/ui/binding/core/router/hash_index_tests.rs
tests:
  - zircon_editor/src/ui/binding/core/router/hash_index_tests.rs
---

# Editor978 · UI router exact-path hash index

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Editor01 exact UI event routing | `EditorUiRouter` owns exact `(view, control, event)` routes in `HashMap<UiEventPath, Vec<Handler<T>>>`; the handler vector remains the registration-order owner, so same-path dispatch order and unrelated-route isolation are unchanged. | The focused current-source binary passed the handler-order and no-ordered-iteration contracts (`2/2`). The ignored Debug timing owner was not promoted: it failed its 30% gate under timing noise (`ordered P95 7,341,700ns`, `hash P95 9,730,600ns`). The plan's deterministic workload still removes `4,096` ordered lookups in favor of `4,096` hash lookups with zero handler-order changes; authoritative Release/managed evidence remains pending. | implemented_pending_validation |

## Deterministic boundary

The router exposes no route iterator. Registration appends handlers to the
vector stored at the exact path, and dispatch returns those handlers in
registration order. Unmatched paths continue to produce no route result.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/binding/core/router.rs` | `BC5701284452B3C2A8973001D987C5C55842EB02CE28422295D8A36C101E7672` |
| `zircon_editor/src/ui/binding/core/router/hash_index_tests.rs` | `9876D5DAB09DF176E0B967A9475D5C8F4ACE9E8FD521C6BFDBB237D3EABB4481` |

## Validation handoff

The focused current-source Debug executable was run once with
`optimization_batch_20260826bx_ui_router_hash_index --test-threads 1
--include-ignored --nocapture`. Two behavior/source contracts passed; the
ignored performance owner failed only its Debug timing threshold and is not
treated as a package or Release failure. The current-source grouped managed
Runtime/Editor wave remains the authoritative next validation lane (Runtime
PTY `98618`, Editor PTY `86711`); no coordinator status was read. Managed
Release P50/P95, allocation, and product-scale gates remain pending.
