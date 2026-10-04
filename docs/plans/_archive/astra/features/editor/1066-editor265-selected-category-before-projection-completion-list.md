---
doc_type: feature-completion
status: applied_candidate_managed_validation_pending
validation_status: scratch_review_complete_managed_validation_pending
performance_status: authority_projection_boundary_benchmark_unrun_product_gate_open
plan_sources:
  - docs/plans/optimize/zircon_editor/265-editor-settings-preferences-project-settings-scope-schema-overlay-persistence-migration-restart-plugin-window-current-working-tree-review.md
  - docs/plans/optimize/zircon_editor/265/2026-09-29-prefix-category-selection.md
  - docs/plans/optimize/zircon_editor/265/2026-09-29-selected-category-before-projection.md
implementation_files:
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/settings_window/mod.rs
tests:
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/settings_window/mod.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/settings_window/tests/selected_category_refresh_profile.rs
---

# Editor1066 / Editor265 selected-category projection completion list

| Plan item | Source candidate | Evidence and remaining gate | Status |
| --- | --- | --- | --- |
| E-SET-P2-13 retained row projection | Filter borrowed category/domain fields before constructing owned rows. Preserve built-in slash-boundary descendants and near-prefix exclusion, plugin exact-domain/path matching, trimmed string behavior, and no-selection behavior. | The source category filter and extracted benchmark are applied; managed validation is pending. The benchmark now lives in settings_window/tests/selected_category_refresh_profile.rs, declared from the existing tests module. Scratch rustfmt and patch applicability passed for v3; managed Editor tests have not run. | applied_candidate_managed_validation_pending |
| Editor265 scale and product acceptance | Include authority batch materialization and retained row conversion in the selected-category workload. | The ignored boundary benchmark covers 10,000 definitions, 1,000 built-in categories, 100 plugin categories/pages, including a middle parent, high-fanout root, and selected-plugin scenario. It reports boundary P50/P95 but remains unrun, does not call the production bridge/surface or capture allocations, and is not product acceptance. Product P50/P95, allocation/memory, high-fanout virtualization, and broader G30 scale/soak/churn gates remain open; the reviewed plans contain no approved numeric product budgets. | open |

This list records applied source and benchmark changes awaiting grouped managed validation.
No passing test, Release timing or product performance acceptance is claimed.
