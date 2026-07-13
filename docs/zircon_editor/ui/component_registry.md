---
related_code:
  - zircon_editor/src/ui/component_registry.rs
  - zircon_editor/src/ui/template_runtime/retained_adapter.rs
  - zircon_editor/src/ui/retained_host/ui/component_contract_metadata.rs
  - zircon_editor/src/ui/retained_host/ui/workbench_window_projection.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_component_family/roles.rs
implementation_files:
  - zircon_editor/src/ui/component_registry.rs
  - zircon_editor/src/ui/template_runtime/retained_adapter.rs
  - zircon_editor/src/ui/retained_host/ui/component_contract_metadata.rs
  - zircon_editor/src/ui/retained_host/ui/workbench_window_projection.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_component_family/roles.rs
plan_sources:
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/zircon_editor/editor_layout/17-text-rendering-and-typography.md
  - docs/plans/zircon_runtime/text/index.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - retained_registry_includes_material_text_input_contracts (passed 1/1 on current-source Windows binary)
  - search_field_projection_preserves_placeholder_and_text_input_identity (passed 1/1 on current-source Windows binary)
  - extension_workspace_search_field_paints_surface_glyph_and_placeholder_text (passed in componentized painter 4/4 group)
doc_type: module-detail
---

# Retained Component Registry

`ui/component_registry.rs` is the single editor owner for component descriptors consumed by retained projection and native host contract metadata. It starts with `UiComponentDescriptorRegistry::editor_showcase()` and then registers `material_editor_foundation()` descriptors. Registry replacement by component id is intentional: when both catalogs describe the same id, the Material foundation descriptor is the final typed contract rather than a legacy showcase string fallback.

Both `RetainedUiHostAdapter` and `component_contract_metadata` call `retained_component_registry()`. They must not assemble private registry variants, because a split catalog can preserve geometry while silently dropping category, layout-role, and semantic role before native painting.

## SearchField Projection

`SearchField` is classified as `search-field / input / leaf`. Workbench native projection preserves the authored placeholder as display text when no value or explicit text exists, emits the native `SearchField` role, chooses the shared `inset` surface, and applies the standard 5 logical-pixel radius plus 1 logical-pixel border defaults. `template_component_family` maps the role to `TextInput`, so glyph measurement and placeholder painting continue through the retained Runtime text interface instead of a component-local bitmap or ASCII fallback.

The defaults are component semantics, not absolute screen placement. The authored `.zui` layout remains responsible for relative frame calculation, while design-token resolution and the native text family own color, border, padding, font metrics, and interaction-state appearance.

## Validation State

The typed registry exact test passes 1/1 on the 2026-07-12 current-source Windows binary; SearchField DTO projection also passes 1/1 and the earlier E0308 has been returned as fixed. The active-workspace-root painter group passes 4/4, including SearchField surface/glyph/placeholder pixels and inactive sibling-host isolation. Real Blend Space screenshots pass at 900×620 and 1260×780 and are stored only under `docs/tests/editor`; registry metadata and native painting now have end-to-end visual evidence.
