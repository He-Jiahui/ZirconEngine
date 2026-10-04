---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/86/2026-09-21-ui-v2-direct-reference-convergence.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/asset/assets/ui.rs
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime/src/asset/registry/dependency_extractors/mod.rs
tests:
  - zircon_runtime/src/asset/tests/assets/ui/references.rs
  - zircon_runtime/src/asset/tests/registry_index/dependency_extractors.rs
  - tools/tests/test_runtime863_ui_v2_direct_reference_contract.py
---

# Runtime863 UI V2 Direct Reference Convergence

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime UI V2 asset dependency references | Give view, component, and style wrappers one typed `direct_references` contract backed by the shared document collector, then route both generic imported assets and registry dependency extraction through it. URI normalization, fragment removal, first-seen deduplication, and order remain unchanged. | v5 reached Cargo and exposed three missing-method `E0599` errors; the support-first wrapper parity regression now covers all three asset kinds, the focused four-contract batch passes `32/32` in `0.020s`, and exact Rustfmt passes. v6 is asynchronous and current-source managed acceptance remains pending. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/asset/assets/ui.rs` | `E4C41B76D9AB8814216E397ADFC2D15E17B7AB7D4331AE6CDEA96A0F934C2B73` |
| `zircon_runtime/src/asset/assets/imported.rs` | `F4006FBB96D44EF7ACD947D0AF2E7275EE58EF9852F5068A354B73953991D0B8` |
| `zircon_runtime/src/asset/registry/dependency_extractors/mod.rs` | `910C3824F1F1B8B4F88573F20408F4C11C911E1B3B735B27F629BB87AF535A2C` |
| `zircon_runtime/src/asset/tests/assets/ui/references.rs` | `334001CABB4D6A9F4D26BFCF329ACACD42AD622932ECFF6321F110B93020708D` |
| `zircon_runtime/src/asset/tests/registry_index/dependency_extractors.rs` | `8A14A772B2A35B9AFE4988C6741671E6E54B6E24FC64BC9F98AD6DC7E3076AC2` |
| `tools/tests/test_runtime863_ui_v2_direct_reference_contract.py` | `BDDE7208D3FF4FD07E4998FFF8BB03C48AFA389775F11A424FEE50CD4F3F5D34` |

## Managed gate

The combined v6 lane was submitted without waiting or polling. Keep this entry
pending until its source fingerprint is reconciled and the current Runtime
tree passes managed Windows compilation plus the required Release, allocator,
and product percentile gates. Tooling production remains deferred.

One later bounded reconciliation proves the repaired Runtime production source
builds under managed Windows Cargo at exit `0` (`651.8218801s` compile/link;
input manifest `77b09f4d…`). The receipt contains no lower-test execution and is
not Release/allocator/product-percentile evidence, so status stays pending.

The later Runtime864 move-append optimization changes only dependency append
scratch ownership; Runtime863's typed wrapper contract remains green in the
combined `36/36` Runtime/asset source-contract batch.
