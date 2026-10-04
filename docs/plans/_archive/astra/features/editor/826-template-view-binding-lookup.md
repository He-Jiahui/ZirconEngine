---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/123/2026-09-19-template-view-binding-lookup.md
  - docs/plans/optimize/zircon_editor/123-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-current-source-review.md
related_code:
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/core/extension/store/batch.rs
tests:
  - zircon_editor/src/core/editor_extension/optimization_batch_editor826_template_binding_tests.rs
  - zircon_editor/src/core/extension/store/batch/optimization_batch_editor826_template_binding_tests.rs
  - tools/tests/test_editor_template_view_binding_lookup_performance_contract.py
---

# Editor826 Template-to-View Direct Lookup

The two Editor extension contribution owners now bind plugin templates by
directly probing the template map for each view ID. The temporary template-ID
`BTreeSet` and its full pre-scan are removed while explicit bindings, missing
views, and non-plugin documents retain their prior behavior.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Editor826 | Replace temporary plugin-template ID sets in `EditorExtensionRegistry` and `ContributionBatch` with keyed `ui_templates.get(view.id())` probes | implemented_pending_validation | TDD source/model contract `4/4`; lower parity regressions and ignored `EDITOR826_TEMPLATE_VIEW_BINDING_LOOKUP_BENCH_V1` markers are wired in both owners; the current one-process non-tooling Runtime/Editor batch passes `2218/2218` across `621` modules with zero failures/errors/skips, and the latest shared batch passes `2227/2227` across `624` modules; the 4,096-template/4,096-view deterministic model removes one temporary set allocation and a full template pre-scan; managed Cargo/Release and Editor product p50/p95/p99 evidence remain pending. |

## Complexity boundary

This is a local contribution-replacement projection change. It preserves
template URI validation, explicit view bindings, missing-template checks,
source ordering, and the `plugins://` admission predicate. It does not change
plugin ownership, generation, capability filtering, or publication authority.

## Managed gate

The source/model contract is locally green and joins the shared batched
Runtime/Editor validation. No standalone Cargo command was started. Managed
Windows Release compilation and Editor allocation/latency evidence remain
pending behind the external dirty `E:\Git\zr_vm` admission gate; no coordinator
status was polled.
