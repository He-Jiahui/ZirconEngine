---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-30-runtime521-gpu-scene-id-allocator.md
implementation_files:
  - zircon_editor/src/core/plugin/panel_source.rs
tests:
  - zircon_editor/src/core/plugin/panel_source.rs
---

# Editor958 Editor521 plugin-panel single search

Plugin-panel row projection now performs one manager binary search and reuses
that index for the generation-paired catalog projection. Package ordering and
borrowed-generation semantics remain covered.

Marker `EDITOR521_PLUGIN_PANEL_SINGLE_SEARCH_BENCH_V1` reports the one-search
versus two-search model across the panel catalog. Managed Editor Release
validation remains pending.
