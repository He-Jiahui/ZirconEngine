---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/editor/31-20260901-optimization-batch-completion.md
implementation_files:
  - zircon_editor/src/core/runtime_event_consumer/registration.rs
  - zircon_editor/src/ui/sample_grid/generation.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/sample_grid.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_sample_grid.rs
tests:
  - zircon_editor/src/core/runtime_event_consumer/registration/optimization_batch_hx_editor607_tests.rs
---

# Editor Registration And Sample-Grid Slice Ownership

This slice keeps editor contribution publication atomic and removes avoidable
allocation layers from registry extension and retained sample-grid generation:

- runtime-event consumer batches preflight all incoming IDs and publish only
  after the complete batch succeeds, then move registrations with
  `BTreeMap::append` instead of cloning the live registry;
- sample-grid ticks and points are stored as immutable `Arc<[T]>` slices,
  allowing projections and painters to share one compact generation without a
  separately owned `Vec` header/capacity.

No contribution ordering, duplicate diagnostics, sample values, or generation
identity semantics changed.

## Plan completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Editor runtime-event consumer registry | Preflight incoming IDs, preserve rejected-batch atomicity, and append disjoint registrations without cloning the live registry. | Move-only extension behavior/source regressions (including late-duplicate atomicity) plus bounded-pump contracts pass; scoped Rustfmt and diff checks pass. | implemented_pending_validation |
| Retained Sample Grid | Share ticks/points as `Arc<[T]>` immutable slices. | Sample-grid source contract passes; scoped Rustfmt and diff checks pass. | implemented_pending_validation |

## Post-change local batch (2026-09-12)

- Editor performance contracts: `581/581` passed in `1.042s`.
- Full Editor pressure batch: `129/129` passed in `2.216s`.
- Runtime-event consumer and inspector contracts: `16/16` passed in `0.691s`.
- Deduplicated unified Runtime/Editor performance, pressure, accessibility,
  rich-text, and Editor607-focused loader: `2041/2041` passed in `10.835s`.
- Current single-invocation Runtime/Editor performance-plus-pressure batch:
  535 matching modules, `1991/1991` passed in `8.659s`.
- Current `registration.rs` SHA-256:
  `3388BF2E58C73DD7906176523599111777CB8D73B72FA71A68FD4B4490D61198`.

These are local source/contract checks; they do not replace the managed Cargo
compile or Windows Release product measurements.

The legacy tooling module `test_editor02_plugin_registration_atomicity_contract`
was intentionally not changed: its three checks still require the retired
clone-candidate text and an old monolithic registration module. Running it
against the current source reports `2 failures/1 error`; the authoritative
Editor607 Rust source regression instead verifies preflight plus move-only
`BTreeMap::append`. Tooling migration remains deferred.

## Validation boundary

The local source-contract evidence is not a managed Cargo compile or Windows
Release product-performance receipt. The external `E:\\Git\\zr_vm` dirty-worktree
admission gate still prevents owner-attributed Cargo validation; keep this
record pending until the asynchronous multi-task ticket supplies compile/test
and product p50/p95/p99 evidence. Tooling changes remain deferred by request.
