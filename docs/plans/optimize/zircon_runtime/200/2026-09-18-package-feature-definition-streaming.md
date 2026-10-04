---
title: Runtime200 Package Feature Definition Streaming
category: zircon_runtime
report_id: Runtime800-package-feature-definition-streaming-2026-09-18
date: 2026-09-18
session_id: root-runtime-editor-async-optimization-20260918
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime800 · Package feature-definition streaming

## Scope

Runtime plugin catalog assembly previously materialized a temporary
`Vec<FeatureDefinition>` for every package and then immediately consumed it in
the merge loop. The merge now visits definitions in-place through a typed
optional-first visitor. The materializing helper remains for callers that need
an owned list, so provider selection, ordering, duplicate diagnostics, and
feature-extension semantics stay unchanged. The enclosing map also reserves
the complete package declaration bound before merging runtime registrations.

## Implementation

- Added `visit_package_feature_definitions` as the canonical package-order
  projection.
- Switched `merge_package_feature_definitions` to consume the visitor, removing
  one short-lived vector per registered package.
- Sized the enclosing definition map/order vectors from the saturating package
  declaration count plus eligible runtime registration count.
- Bound package and runtime registration capacity to product-catalog-eligible
  registrations. Carrier and test-fixture registrations remain inspectable and
  are still skipped by the merge authority, without reserving unreachable
  definition slots.
- Kept `package_feature_definitions` as the compatibility materialization
  boundary for existing callers and tests.

## Deterministic performance model

For 256 registered packages, the retired merge path creates 256 temporary
feature-definition vectors. The visitor path creates zero package-level
temporary vectors; each owned `FeatureDefinition` still retains its required
key, manifest, and provider allocations. The aggregate map/order reservation
also removes geometric growth from the declared package-count bound. This is
allocation-shape evidence, not product CPU, RSS, or percentile evidence.

The eligibility follow-up makes the reservation model match the existing merge
authority: a mixed input with 256 carrier declarations and 64 eligible
declarations reserves 64 slots rather than 320, removing 256 unreachable
slots while preserving the carrier-skip behavior. This is deterministic
allocation evidence, not product timing evidence.

## Local evidence

- TDD source contract is GREEN (`8/8`), after an intentional RED baseline.
- Lower Rust source tests cover materialized/visitor order and provider parity,
  merge visitor wiring, the zero-temporary-vector model, and carrier capacity
  exclusion for package and runtime registrations.
- Exact-file Rustfmt for the six touched Runtime200/205 files, Python
  compilation, and `git diff --check` pass.
- Managed Cargo/Release allocation and product catalog-build p50/p95/p99
  evidence remain pending; tooling production remains deferred.

## 2026-09-21 managed compile repair

The first v5 batch to cross the Windows validation bridge reached Rust and
reported that `product_catalog_feature_registration_count` was re-exported one
module farther than its `pub(super)` definition allowed. The helper now uses
the same `pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog)` scope
as the merge owner. The older Runtime219 structural regression was also
converged from the retired raw registration-count expression to the current
package-declaration plus product-eligible feature-registration bound. The
focused Runtime200/205 contract remains GREEN inside the combined `32/32`
Runtime repair batch. Current-source managed compilation and all product gates
remain pending.

The subsequent bounded v6 receipt compiles `zircon_runtime` successfully with
exit `0` (`651.8218801s` compile/link; input manifest `77b09f4d…`). This closes
the specific visibility compile failure, but the receipt ran no tests and is
not Release/allocator/product-percentile evidence, so the managed validation
status remains pending.

## Acceptance boundary

Keep this record `managed_validation_pending` until Runtime200 is included in
the next batched Windows validation lane. Do not treat the deterministic model
as product acceptance.
