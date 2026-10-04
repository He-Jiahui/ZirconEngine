---
record_kind: milestone
status: implemented_pending_validation
created_at: 2026-09-12
plan: docs/plans/astra/optimize/01-review-and-repair.md
milestone: PLUGIN-A1 runtime catalog alias convergence
session: astra-plugin-catalog-convergence-20260912
---

# Runtime catalog alias convergence

This bounded runtime slice closes the remaining base-selection and feature-ID
alias gaps after the App explicit product-admission repair. Catalog matching
resolves project and registration IDs through `RuntimePluginId::parse_key`,
keeps one eligible registration per canonical ID (preferring an exact persisted
row so a disabled alias cannot re-enable a second package), and rejects
carrier-role base reports from product selection. Required rows therefore
produce an explicit fatal admission diagnostic when no eligible target provider
remains. Feature owner/dependency, definition, and provider-registration
matching use the same alias contract.

## Evidence and scope

| Finding | Status | Lowest owner and evidence |
|---|---|---|
| Runtime catalog compares canonicalizable IDs by raw string | `implemented_pending_validation` | `zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project/selection.rs` now canonicalizes matching while preserving the manifest spelling passed to feature dependency context and selecting one eligible registration per canonical ID. |
| Required alias selection can be silently dropped | `implemented_pending_validation` | Focused regression `required_alias_selection_matches_canonical_catalog_registration` in `selection.rs` covers `audio -> sound`, selected index, and source-spelling preservation. |
| Carrier-role base report can satisfy a required product row | `implemented_pending_validation` | `selected_registration_indices` applies `PluginPackageRole::is_product_catalog_eligible`; regression `required_carrier_role_is_not_a_product_catalog_provider` covers `TestFixture`. |
| Feature owner/dependency and provider aliases disagree | `implemented_pending_validation` | `feature_support.rs` precomputes canonical selection/enabled sets and uses O(1) membership; `feature_definitions/lookup.rs` and `feature_registration_match.rs` use symmetric alias matching. Regressions cover owner `sound`/dependency `audio`, `plugin_is_enabled_for_target("sound", alias audio)`, alias provider registration, and unknown/malformed IDs. |
| Feature-only report role provenance | `blocked_dependency` | `RuntimePluginFeatureRegistrationReport` carries `provider_package_id` but no typed `PluginPackageRole`. Existing valid external feature-provider behavior (`zircon_runtime/src/tests/plugin_extensions/runtime_plugin_catalog_features/compiled_selection.rs:179-228`) means catalog-side rejection by provider ID alone would reject valid feature-only providers. A coordinated report/loader provenance field is required before feature reports can be role-filtered safely. |

## Contract notes

- The project manifest remains source-spelled for diagnostics and persistence;
  `FeatureDependencyContext` retains that raw map while precomputing canonical
  selection and enabled-ID sets once. Feature status/dependency checks then use
  O(1) canonical membership, without adding alias-duplicated project rows.
- Definition lookup and feature-provider registration matching prefer exact
  persisted spelling, then apply symmetric `RuntimePluginId` matching. Exact
  valid dynamic package IDs remain valid through the same parser; malformed
  IDs are rejected even when both sides use the same raw spelling.
- Feature registrations remain role-neutral. The explicit base selection path
  filters `PluginPackageRole::{Production,DeveloperTool}` for product catalog
  admission, while feature-only providers continue to be selected by their
  explicit feature/provider identity until a typed report/loader provenance
  field is available.

## Validation boundary

- `rustfmt --edition 2021 --check` passed for the ten touched Rust sources: selection, feature support/context/resolution/status, feature definitions lookup, feature registration matching, catalog feature tests.
- Scoped `git diff --check` passed for those ten sources (Git emitted only the existing LF/CRLF warning).
- Focused static regressions added: `required_alias_selection_matches_canonical_catalog_registration`, `required_carrier_role_is_not_a_product_catalog_provider`, `feature_owner_enablement_matches_alias_without_duplicating_selection_rows`, `feature_owner_enablement_uses_precomputed_canonical_membership`, `selection_provider_alias_resolves_the_canonical_definition`, `provider_lookup_keeps_raw_manifest_spelling_while_matching_alias_registration`, `provider_match_rejects_unknown_and_malformed_ids`, `canonical_alias_selection_resolves_owner_and_dependency_feature`, and `canonical_alias_feature_provider_registration_matches_project_provider`.
- Final per-file SHA-256 values after the last static check:
  - `selection.rs`: `864f590e9051dbe491e55d0558ff49034b63e6f85cc760336e42c6767b31df9a`
  - `feature_support.rs`: `52afc2fd71ab325a1d09a7dd48d46dc17a822fcb4118475d64012aa8bc22bf76`
  - `features/context.rs`: `58b88f1215ed16136900828c4a039b2fde35f9edc00b525ca347f72ded980c12`
  - `features.rs`: `684f89f526ef0db0bcd81eae962ae6692712e10eba83a9e5d9995e4f598cd55d`
  - `feature_resolution.rs`: `3105deb9bfbb3bea74c3ac25acbd7afc60c56b6ed5f76a1d546c704b64c99313`
  - `feature_status.rs`: `60efdfb0aa7c8a165724619ebeab896b41b7ec06c5807ffebc17477b15884714`
  - `feature_status/dependencies.rs`: `5db8aff3148f39ad356df842d958ad7d9dafb382d4b17de5b87fc09223b15d6c`
  - `feature_definitions/lookup.rs`: `cd8b7fffc3d10d2592b6a9cd668421f59f09e2892356ae8f8f9e539c2c71fd3f`
  - `feature_registration_match.rs`: `5cc2d77d3dacd81f40573138cc730dde7a203f4d14197b9f2193b455c3e88f62`
  - `compiled_selection.rs`: `ed2806220230a5bcc1f4aa1d2de64549d043e03f82a6bb7b15fa1bb3fad1abc2`
- The final record hash is reported in the coordinator handoff after this
  content is frozen (the record intentionally does not embed a self-hash).
- No Cargo, native, DLL, or product command was run. Managed validation remains deferred by the dirty external `E:\Git\zr_vm` dependency; this record is not an acceptance claim.

No commit was created. The exact source and record leases are released in the
session handoff after this static check.
