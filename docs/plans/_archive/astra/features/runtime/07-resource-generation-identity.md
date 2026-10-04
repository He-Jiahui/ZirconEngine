---
status: in_progress
plan_sources:
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/64-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-review.md
---

# Resource publication cache identity

## Diagnosis and repair

Batch run `4e5e00e217074a20be419299442bfafa` reports 129 runtime lib-test
compile errors, down from 278 in the previous diagnostic batch. Tests and
performance samples have not executed; this is not performance acceptance.

`ProjectAssetManagementGeneration` still compared the removed numeric resource
sequence. It now retains `ResourceManagementGenerationIdentity` and compares
the immutable publication together with the project generation. Closed-project
projections contain no resource identity. The public accessor is
`resource_generation_identity`; numeric diagnostic counters are not cache keys.

The regression covers a shared publication, a changed project generation, two
distinct publications with equal diagnostic counts, and release of the retained
publication when its asset projection is dropped. No compatibility sequence API
is restored.

The captured resource publication is also passed through record construction to
the ID scan. A second regression publishes a resource after capture and verifies
the old scan remains empty while the new publication contains the resource.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| M10 | Resource publication cache identity and lifetime regression | implemented_pending_validation | Include `astra_m10` with the next multi-task release batch |
