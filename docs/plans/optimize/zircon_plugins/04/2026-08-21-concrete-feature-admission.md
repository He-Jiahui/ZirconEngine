# Plugins04 concrete feature admission optimization record

- Date: 2026-08-21
- Owner plan: `docs/plans/optimize/zircon_plugins/04-rendering-umbrella-feature-bundles-solari-native-provider-product-integration-review.md`
- Finding: `PLUGIN-RENDERING-P0-001`
- Status: `validation_pending`

## Scope

- Index concrete runtime feature providers by qualified feature/provider identity in the immutable catalog projection.
- Require that concrete registration before a selected feature can become available or publish capabilities.
- Surface missing providers through the public feature block and fatal product-extension diagnostics.
- Preserve the existing fixed-point dependency order and concrete provider extension merge behavior.

## Contract

- Package optional-feature metadata remains declaration-only.
- A selected feature is admissible only when a matching `RuntimePluginFeatureRegistrationReport` exists for its resolved provider package.
- Missing concrete providers produce `provider_missing = true`, zero feature availability, zero feature capability publication, and a stable diagnostic naming the feature and provider.
- Each selected feature performs exactly one O(1) provider-registration membership check from the catalog projection.

## Performance Gate

- The release workload resolves 1,000 enabled metadata-only feature selections.
- The previous path falsely admitted 1,000 feature ids and published 1,000 feature capabilities; the current path admits and publishes zero, a deterministic 100% reduction for both false states.
- The current path performs exactly 1,000 provider registration checks and produces exactly 1,000 provider-missing blocks, preserving linear resolver work with O(1) admission lookups.
- The release marker is `PERF-MVP-PLUGINS04-CONCRETE-FEATURE-ADMISSION`.

## Validation

- The scale regression asserts zero available features, 1,000 structured provider-missing blocks, 1,000 fatal extension diagnostics, and no feature runtime modules.
- The existing immediate-blocker regression now installs concrete registrations for its two real feature definitions, so it continues to lock fixed-point capability cleanup rather than relying on metadata-only admission.
- Catalog dependency/cycle fixtures now install concrete feature registrations explicitly, while the built-in Rendering catalog regression asserts that dependency-complete metadata-only features remain provider-missing.
- Cargo compilation, behavior tests, and release marker validation are queued in the multi-task Plugins aggregate; no standalone Cargo run or measured pass is claimed here.

## Remaining Plan Work

- This slice closes the first fail-closed admission gate for concrete registration presence.
- Artifact kind/digest, detached materialization, runtime initialization receipt, health generation, and qualification state remain Plugins04 milestones.
- Editor/Play/cook/export canonical provider graph equivalence and Solari native contribution replay remain separate Plugins04 findings.
