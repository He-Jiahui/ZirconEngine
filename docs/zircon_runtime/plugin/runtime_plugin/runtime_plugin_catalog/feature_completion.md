---
related_code:
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_completion.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_completion/owner_selection.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/package_feature_definitions.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_status.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/enablement/features.rs
implementation_files:
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_completion.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_completion/owner_selection.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/enablement/features.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
  - docs/plans/zircon_runtime/frameworks/02/failure-2026-07-11-editor-m1-plugin-provider-lookup.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_runtime/src/tests/plugin_extensions/runtime_plugin_catalog_features.rs
  - zircon_runtime/src/tests/plugin_extensions/runtime_plugin_catalog_features/feature_dependency_reports.rs
  - zircon_editor/src/tests/host/manager/minimal_host_contract/optional_features.rs
doc_type: module-detail
---

# Runtime Plugin Feature Completion

## Purpose

`RuntimePluginCatalog` completes sparse project plugin manifests into the canonical selection graph consumed by runtime status, editor plugin management, dependency validation, and export planning. The graph must preserve one provider identity from package declarations through feature selections and standalone external provider packages.

## Behavior Model

Feature definitions are keyed by `feature_id@provider_package_id`. An owner package can declare a feature whose provider is the owner itself or a standalone external package. Completion first projects every feature under its owner plugin selection, preserving runtime/editor crates, target modes, packaging, and an external `provider_package_id` when present.

For every external provider definition, completion also materializes a disabled `ProjectPluginSelection` for that provider package. The provider selection uses the same feature manifest projection for target modes, packaging, runtime crate, and editor crate. It is not a second feature definition and does not enable itself by default; it is the package-level activation record required by `feature_status(...)` and export materialization.

## Dependency Enablement

Editor dependency enablement operates on the completed manifest. It recursively enables declared plugin dependencies and unique feature providers, then enables the requested feature's external provider package selection. The requested feature remains disabled until the user explicitly enables it. The update report includes the external provider in `enabled_dependency_plugins`, making the writeback and UI report reflect the actual runtime dependency graph.

The runtime status path continues to reject an enabled feature when its owner, declared dependency, or external provider package is disabled. No fallback lookup, alias provider key, or editor-only exception bypasses that rule.

## Edge Cases And Constraints

- Existing provider selections are preserved; completion never duplicates or overwrites user choices.
- Owner-provided features do not create a second package selection.
- External providers without a runtime or editor module retain `None` for the corresponding crate field.
- Provider completion happens after owner feature projection so mutating the selection vector cannot invalidate the owner iteration.
- A missing completed provider is an architecture error; editor enablement returns a typed diagnostic instead of manufacturing a path-specific substitute.

## Test Coverage

The runtime catalog projection test asserts that an external feature provider appears as a disabled package selection with the declared runtime and editor crates. The editor optional-feature regression covers the upward path: initial block, dependency enablement of owner/dependency/provider packages, available-but-disabled status, then explicit feature enablement.

Current acceptance remains open until the rebuilt current-source exact and the full Editor M1 library gate pass.
