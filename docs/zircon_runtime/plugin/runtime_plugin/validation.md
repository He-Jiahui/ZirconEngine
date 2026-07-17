---
related_code:
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation
  - zircon_runtime/src/plugin/runtime_plugin/module_validation
  - zircon_runtime/src/plugin/runtime_plugin/registration_report
implementation_files:
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/shape/namespace/segments.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/shape/namespace/segments/count.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/shape/namespace/segments/tokens.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/identity.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/shape/namespace/segments.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/versions/segments.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_feature_providers/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_owners/event_catalogs/prefix.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/modules/row/systems.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
tests:
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/shape/namespace/segments.rs::tests::runtime_feature_namespace_validation_streams_segments
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/identity.rs::tests::runtime_feature_owner_prefix_check_does_not_format_a_string
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/identity.rs::tests::runtime_feature_owner_matching_preserves_the_dot_boundary
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/shape/namespace/segments.rs::tests::package_namespace_validation_streams_segments
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/shape/namespace/segments.rs::tests::package_namespace_validation_preserves_segment_diagnostics
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/versions/segments.rs::tests::package_semver_validation_does_not_collect_segments
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/versions/segments.rs::tests::package_semver_validation_preserves_shape_and_component_diagnostics
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_feature_providers/uniqueness.rs::tests::feature_provider_uniqueness_preserves_duplicate_diagnostics
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_owners/event_catalogs/prefix.rs::tests::event_catalog_owner_check_preserves_the_dot_boundary
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/modules/row/systems.rs::tests::module_system_owner_check_preserves_the_dot_boundary
doc_type: module-detail
---

# Runtime Plugin Validation

## Purpose

Runtime plugin validation checks package, module, interface, system and feature registration manifests before they become the frozen runtime catalog. It is a registration boundary rather than a frame/tick path, but its work must still scale predictably for editor discovery, project bootstrap, hot reload and export validation.

## Namespace and identity validation

Feature namespaces require at least one dot and every dot-separated segment must be a lowercase runtime plugin token. The validator checks dot presence first so the existing “at least two segments” diagnostic remains the first failure for a single token, then streams `split('.')` directly into the token predicate. It does not allocate a segment vector.

Feature ids must belong to their declared owner plugin. The owner check borrows both strings and accepts the id only when removing the exact owner leaves a suffix beginning with `.`. This preserves the namespace boundary: owner `rendering` accepts `rendering.deferred`, while owner `render` does not accept it. No formatted owner-prefix string is built.

Package namespace validation uses the same streamed rule. Semantic versions are consumed as exactly three iterator items instead of a temporary segment vector; missing or extra items retain the MAJOR.MINOR.PATCH diagnostic, while valid three-part input continues through component digit, leading-zero and range checks.

Embedded optional-feature and feature-extension provider uniqueness state stores borrowed `(feature_id, provider_package_id)` pairs. The state lives only for the package-validation call, so owning duplicate strings has no lifetime benefit. Event catalog namespaces and module system set/anchor names use borrowed dot-boundary owner checks as well.

## Performance constraints

- Validation diagnostics retain their existing text and ordering; allocation removal must not turn validation into fail-fast behavior.
- Namespace checks may allocate only when emitting an actual diagnostic, not for successful segment traversal.
- Owner identity checks remain allocation-free for both accepted and rejected ids.
- Successful namespace and semantic-version traversal does not allocate segment collections.
- Provider identity and owner-prefix checks do not allocate Strings; only emitted diagnostics own text.
- Registration-scale benchmarks still need to cover 1/100/1000 feature manifests. The broader repeated scans and linear uniqueness checks in registration, module and feature validation remain tracked by the performance plan until a shared ordered validation projection is designed.

## Verification status

The allocation source guards and focused diagnostic/owner-boundary behavior tests have completed static RED-to-GREEN verification. All changed Rust files pass scoped `rustfmt --edition 2021 --check` plus `git diff --check`. Current-source warm Cargo tests and allocation/scale measurements remain pending, so the directory is not accepted in `docs/plans/performance/review.md`.
