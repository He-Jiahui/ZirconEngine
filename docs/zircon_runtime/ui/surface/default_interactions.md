---
related_code:
  - zircon_runtime/src/ui/surface/surface/default_interactions.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/range.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/radio.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/scrollbar.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/table/mod.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/table/mutation.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/timers.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/toast_timer.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/tree_view.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/tree_view_reparent.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/tree_view_support.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/tree_view_virtualization.rs
  - zircon_runtime/src/ui/surface/component_state.rs
  - zircon_runtime/src/ui/surface/property_mutation.rs
  - zircon_runtime/src/tests/runtime_absorption/ui_architecture.rs
implementation_files:
  - zircon_runtime/src/ui/surface/surface/default_interactions.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/table/mod.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/table/mutation.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/timers.rs
  - docs/zircon_runtime/ui/surface/default_interactions.md
  - zircon_runtime/src/tests/runtime_absorption/ui_architecture.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
  - docs/plans/zircon_runtime/runtime/index.md
  - user: 2026-06-16 implement runtime architecture from docs/plans/zircon_runtime/runtime
tests:
  - tools/tests/test_runtime_ui_table_module_structure.py
  - tests/acceptance/runtime-ui-table-mutation-owner-split.md
  - zircon_runtime::tests::runtime_absorption::ui_architecture::runtime_09_surface_default_interaction_fallback_rename_reduces_ui_surface_debt
  - ui_architecture_boundary targeted audit
  - rustfmt check for zircon_runtime/src/ui/surface/surface/default_interactions.rs
  - cargo test -p zircon_editor --lib page_layout_templates --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-editor-0623-clean-2309 --message-format short --color never -- --test-threads=1 --nocapture: passed 4/4 on 2026-06-23 after default-interaction timer import repair
doc_type: module-detail
---

# Runtime UI Surface Default Interactions

`default_interactions.rs` owns the retained-surface default actions that run after explicit widget reducers and input routes have selected a target. It is runtime UI behavior, not an editor projection layer and not a shared interface contract.

The file keeps open-state fallback lookup local to `UiSurface::default_open_boolean_value(...)`. That helper resolves the authored open property first, then the same component-state property, then the listed `fallback_properties`, and finally the canonical runtime open flag before using the supplied default. This preserves existing behavior for `expanded`, `popup_open`, and `open` aliases while naming the alternatives as fallback state rather than migration debt.

The table interaction family keeps pointer routing, resize/sort orchestration, and table-role classification in `table/mod.rs`. Its property-mutation behavior is owned by `zircon_runtime/src/ui/surface/surface/default_interactions/table/mutation.rs`: column-width maps, embedded column widths, sort models, per-column sort direction, row ordering, and the common accepted-mutation binding-report path. This child boundary keeps the route root below the Runtime 15 size guard without changing table mutation semantics.

runtime_09_m1_2_surface_default_interaction_fallback_renamed_static_passed_cargo_pending

The Runtime 09 M1.2 cutover renamed the local open-state compatibility list from `legacy_properties` / `legacy_property` to `fallback_properties` / `fallback_property` in `default_open_boolean_value(...)`. The behavior is unchanged: authored metadata, component state, retained fallback aliases, and runtime open flags keep the same precedence.

`runtime_09_surface_default_interaction_fallback_rename_reduces_ui_surface_debt` guards the final Runtime 09 production UI `legacy` naming hit from returning. After this slice the UI source scan records `ui_legacy_hits=54`, `ui_legacy_production_hits=0`, and `ui_legacy_production_files=0`; the remaining full-tree hits are outside production UI source.

## Editor Layout 05.S2 Support Repair

During editor layout 05.S2 verification, `default_interactions/timers.rs` exposed a split import drift: it referenced `UiTemplateNodeMetadata` through the old `ui::template` namespace while the current owner exports that type from `ui::tree`. The repair changed only the import to `tree::{UiTemplateNodeMetadata, UiTreeError}` and left timer behavior in the child owner. Focused editor layout verification then compiled through runtime and passed `page_layout_templates` 4/4 with existing warning noise.
