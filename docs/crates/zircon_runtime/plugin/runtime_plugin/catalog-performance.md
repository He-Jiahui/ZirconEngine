---
related_code:
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog
implementation_files:
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/derived_projection.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/registration/update.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/registration/update/candidate_rows.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_resolution.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/runtime_extensions.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
tests:
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/registration/update/tests.rs
  - zircon_runtime/src/tests/plugin_extensions/runtime_plugin_catalog_features.rs
  - zircon_runtime/src/tests/plugin_extensions/runtime_plugin_catalog_features/feature_dependency_reports.rs
doc_type: module-detail
---

# Runtime Plugin Catalog Performance

## Purpose

`RuntimePluginCatalog` owns ordered runtime and feature registration reports, derives diagnostics and bridge dependency closure, completes project selections, resolves feature dependencies and merges the enabled extension registry. These operations run at bootstrap, editor plugin changes, export and hot-reload generations; they are not frame/tick work, but they directly affect MVP startup and editor responsiveness.

## Batch construction

`from_plugins` and `from_descriptors` first preserve module activation order, then build all `RuntimePluginRegistrationReport` values and call `from_registration_reports` once. Structural mutation uses `RuntimePluginCatalogUpdate`: each changed registration domain lazily clones and identity-indexes the published rows once, applies replace/remove through stable slots and tombstones, then materializes one candidate. A successful transaction builds diagnostics and the ordered projection once and publishes one generation; rejection retains the previous rows, projection, plans and generation. An empty transaction does not clone or index registration rows.

## Project extension assembly

`complete_project_manifest(manifest, target)`, `feature_dependency_report(manifest, target)` and `runtime_extensions_for_project(manifest, target)` resolve through one target-specific `CompiledProjectPluginPlan`. The plan stores its catalog generation, structural manifest fingerprint, source manifest equality guard, and shared completed manifest, feature report and extension report. Public completion/report methods return `Arc` snapshots, so a stable cache hit performs only the structural fingerprint/equality check and cheap `Arc` copies; it does not serialize the manifest or deep-clone a report/registry. One plan is retained per target, and publishing a new catalog generation invalidates all cached plans while existing in-flight `Arc` snapshots remain valid.

Feature owner validation streams primary dependencies and rejects zero, multiple or mismatched primaries. Target support streams runtime modules and treats a feature with no runtime module as target-independent. Feature resolution performs one declaration-order first pass and then wakes only rows subscribed to newly available capabilities. An earlier available provider may affect a later row; an immediate blocker is frozen when first visited, is excluded from the unresolved-provider set and is emitted before final capability waiters. This preserves diagnostic order without repeated pending-Vec scans or removals.

## Generation authority

The catalog generation owns one immutable ordered derived projection for package, module/provider, feature/provider and capability dependency lookup. Project plans are consumers of that projection rather than independent catalog caches. Registration order and project declaration order remain observable authority; hash maps are lookup-only and never determine report or extension order.

## Verification status

Source guards cover stale owned APIs, per-mutation full-Vec scans and manifest serialization/report deep clones. Focused tests cover first-pass dependency ordering, false-cycle exclusion, atomic last-good publication, 1/100/10,000 row indexing counters and shared snapshot identity. Current-source managed Cargo and product-scale allocation/latency traces remain acceptance requirements; static evidence alone does not move this subsystem into `docs/plans/performance/review.md`.
