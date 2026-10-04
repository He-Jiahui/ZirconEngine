---
related_code:
  - zircon_editor/src/ui/host/editor_manager_plugins_export/mod.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/native_registration/manager.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/package_projection/project_selection.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/enablement/project.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/enablement/capabilities.rs
implementation_files:
  - zircon_editor/src/ui/host/editor_manager_plugins_export/native_registration/manager.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/enablement/project.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/enablement/capabilities.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md
  - docs/plans/optimize/zircon_editor/06/2026-08-27-borrowed-extension-view-validation.md
tests:
  - tools/tests/test_editor12_plugin_manager_contract.py
  - zircon_editor/src/tests/editor_plugin_catalog_consistency.rs
doc_type: milestone-detail
status: implemented_pending_validation
---

# Editor Plugin Manager Contract Maintenance

Current modular owners are reflected by the static contract: manager publication/snapshot/state,
activation effects, native loader authority, and retained plugin panel reads. Stale path/string
assertions were updated without changing production or tooling implementation.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Editor06/Editor12 | Keep plugin manager lifecycle/catalog/activation ownership checks aligned with split Runtime/Editor owners | implemented_pending_validation | `tools.tests.test_editor12_plugin_manager_contract` 24/24 and combined Runtime/Editor bounded contract batch 70/70; scoped Python syntax, Rustfmt, and diff checks pass; managed Editor Cargo/native/release percentile evidence remains pending |
