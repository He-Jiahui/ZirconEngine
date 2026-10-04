---
related_code:
  - zircon_editor/src/core/settings/catalog/settings_catalog.rs
  - zircon_editor/src/core/settings/catalog/settings_catalog/optimization_batch_hy_editor608_tests.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/settings_projection.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/settings_window/mod.rs
  - docs/crates/zircon_editor/core/settings.md
plan_source: docs/plans/optimize/zircon_editor/265-editor-settings-preferences-project-settings-scope-schema-overlay-persistence-migration-restart-plugin-window-current-working-tree-review.md
plan_item: E-SET-P2-13
status: partial_source_candidate_managed_validation_pending
---

# Editor265: selecting a built-in parent category shows descendant settings

## Failure and repair

The directory projects every prefix of a built-in setting category as a selectable node.
The value query previously used an exact catalog path, and retained row conversion also
required exact path equality. A parent such as `settings.category.editor` therefore
had no rows or values even though it had nested language, autosave, and shortcut settings.

The immutable catalog now builds a subtree posting index for every full
category and ancestor path. Each posting is a position in the already
key-sorted definition array, so direct and descendant keys appear once in
key order without copying their strings into every ancestor. The value query
iterates borrowed definition keys and resolves them in one authority batch.
The existing direct-path index keeps its exact-match behavior. Retained conversion
shows built-in rows whose path is the selected path or a slash-delimited descendant.
Plugin pages still require their own domain and an exact path. A near-prefix such as
`settings.category.editorial` does not match `settings.category.editor`.
The module contract documents this category behavior.

## Regression and performance gates

A catalog test covers a parent with direct, child, and grandchild keys, sorted
order without duplicates, definition-backed references, exact leaf lookup,
and near-prefix exclusion. A default-authority test covers parent/leaf selection
and the resolved batch. A retained
projection test covers two descendant rows and their values. Rustfmt and scoped diff checks pass. The grouped Editor library
test run is pending; these source-level checks do not establish a passing test result.

Catalog construction appends one `usize` posting per category level and makes
no ancestor key-string clone. Selection and refresh use a B-tree lookup and a
borrowed-key iterator, with no intermediate key vector; the output batch still
owns its keys and values. The single lock and one batch resolution remain. An
ignored Release test prepares 10k definitions across 1k categories, checks
30k integer postings and 100 parent queries, then compares indexed and scan
P95 over 21 alternating pairs with a 20% ratio threshold. That test has not
run. The plan's catalog memory, query allocation, full product P95, and scale
acceptance have not been measured.
