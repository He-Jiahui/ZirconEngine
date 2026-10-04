---
related_code:
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/settings_window/mod.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/settings_window/tests/selected_category_refresh_profile.rs
  - zircon_editor/src/core/settings/catalog/settings_catalog.rs
  - zircon_editor/src/core/settings/catalog/settings_catalog/optimization_batch_hy_editor608_tests.rs
plan_source: docs/plans/optimize/zircon_editor/265-editor-settings-preferences-project-settings-scope-schema-overlay-persistence-migration-restart-plugin-window-current-working-tree-review.md
related_candidate: docs/plans/optimize/zircon_editor/265/2026-09-29-prefix-category-selection.md
status: applied_candidate_managed_validation_pending
validation_status: scratch_review_complete_managed_validation_pending
performance_status: authority_projection_boundary_benchmark_unrun_product_gate_open
---

# Editor265: filter selected category before owned retained-row projection

## Applied change

The applied source checks borrowed TOML fields first and constructs owned row data only for
matching entries. It preserves trimmed string behavior, built-in slash-delimited descendants and
near-prefix exclusion, plugin exact domain/path matching, and no-selection behavior. The benchmark
fixture reuses each category path across settings by cloning that string when constructing rows.

## Behavioral coverage

The applied tests retain built-in parent/descendant and near-prefix coverage, adds plugin matching
coverage for exact bundle/path selection and cross-domain exclusion, and checks the no-category
case. The benchmark-only fixture also checks 10 rows for a middle built-in parent, 10,000 rows for
the high-fanout built-in root, and one matching page for a selected plugin owner among 100 pages.

## Performance gate

The ignored boundary benchmark lives in settings_window/tests/selected_category_refresh_profile.rs and is declared by a test-only module entry in settings_window/mod.rs. It uses 10,000 definitions across 1,000 built-in categories,
plus 100 plugin categories and 100 plugin-owner pages. It covers a middle parent returning 10
built-in rows, a high-fanout root returning 10,000 rows, and one selected plugin owner whose page
matches among 100 pages. The generated setting, category, plugin-page and resolved-value tables
include the fields emitted by the bridge.

The benchmark times the production built-in/plugin authority branch, test-side bridge-shaped value
payload materialization, and production retained converter. It reports 31-sample P50/P95 by
boundary and in total. This is lower-bound boundary evidence, not full UI refresh: it does not call
the private production bridge helper or a real retained surface, and it does not measure allocator
bytes/count, catalog resident bytes, paint/virtualization, plugin churn, rapid-write soak, or
lifecycle deadlines. The managed product lane must record total versus visible rows and allocation
and memory data. Declare numeric product budgets with the goal owner before closing E-SET-G30.
Keep plugin-owner churn and remaining scale, write-coalescing, soak and lifecycle gates open.

## Validation state

The production category filter and v3 benchmark are applied to the shared checkout. Independent
static review, scratch rustfmt and patch applicability passed. Cargo and the
ignored Release benchmark have not run. Managed Editor validation and all end-to-end product P95,
allocation and memory gates remain open.
