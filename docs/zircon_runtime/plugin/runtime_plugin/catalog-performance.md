---
related_code:
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog
implementation_files:
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/registration/constructors.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/runtime_extensions.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_support.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/registration/order.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
tests:
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/registration/constructors.rs::tests::catalog_constructors_do_not_rebuild_after_each_registration
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/runtime_extensions.rs::tests::project_extension_report_does_not_complete_an_already_completed_manifest
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_support.rs::tests::owner_dependency_validation_requires_one_matching_primary
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_support.rs::tests::target_support_streams_runtime_modules
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/registration/order.rs::tests::registration_order_uses_constant_time_seen_membership
doc_type: module-detail
---

# Runtime Plugin Catalog Performance

## Purpose

`RuntimePluginCatalog` owns ordered runtime and feature registration reports, derives diagnostics and bridge dependency closure, completes project selections, resolves feature dependencies and merges the enabled extension registry. These operations run at bootstrap, editor plugin changes, export and hot-reload generations; they are not frame/tick work, but they directly affect MVP startup and editor responsiveness.

## Batch construction

`from_plugins` and `from_descriptors` first preserve module activation order, then build all `RuntimePluginRegistrationReport` values and call `from_registration_reports` once. The catalog therefore rebuilds diagnostics and bridge dependency closure once for a batch rather than once after every inserted plugin. Public `register` and `register_feature` retain incremental mutation behavior and still rebuild after their single mutation.

## Project extension assembly

`runtime_extensions_for_project` completes the incoming project manifest once and passes that completed manifest directly to the internal feature dependency report builder. The public `feature_dependency_report` continues to accept sparse manifests and complete them for standalone callers. This removes one redundant completion without weakening either API contract.

Feature owner validation streams primary dependencies and rejects zero, multiple or mismatched primaries. Target support streams runtime modules and treats a feature with no runtime module as target-independent. Registration ordering records emitted package indices in a fixed bool vector, avoiding repeated linear membership scans while preserving activation order followed by metadata-only order.

## Remaining generation projection

Feature definition maps, package/selection indexes, provider/module indexes and the feature dependency graph are still rebuilt across completion, report, extension and lookup consumers. Fixed-point resolution still rescans and removes from a pending Vec. PERF-MVP-061 requires one immutable ordered projection per catalog generation, exact invalidation on registration/hot reload and O(V+E) feature resolution; local per-helper caches are prohibited.

## Verification status

Source guards and focused owner/target behavior tests completed static RED-to-GREEN verification. The four changed Rust files and the full current session Rust scope pass `rustfmt --edition 2021 --check` plus `git diff --check`. Current-source warm Cargo, constructor build-count benchmarks and project-scale allocation/complexity measurements remain pending, so the catalog stays out of `docs/plans/performance/review.md`.
