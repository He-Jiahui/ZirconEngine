---
title: Runtime UI V2 Direct Reference Convergence
category: zircon_runtime
report_id: Runtime863-ui-v2-direct-reference-convergence-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: semantic_repair
---

# Runtime863 UI V2 Direct Reference Convergence

## Finding

The asset-registry dependency extractor had been widened to consume the typed
`direct_references` contract for UI V2 view, component, and style assets, but
those three wrappers exposed only parse and serialization methods. The first
managed v5 Runtime compile therefore reached Rust and failed with three
`E0599` errors. `ImportedAsset` also bypassed the typed boundary and invoked the
shared document collector directly, leaving two reference-extraction paths.

## Repair

- Added `direct_references` to `UiV2ViewAsset`, `UiV2ComponentAsset`, and
  `UiV2StyleAsset`; all three delegate to the existing
  `ui_v2_asset_references` collector.
- Routed `ImportedAsset::direct_references` through those typed methods, so the
  registry extractor and generic imported-asset projection share one wrapper
  contract.
- Kept URI normalization, fragment removal, first-seen deduplication, and
  import/resource order in the existing shared collector.
- Added a lower parity regression across all three wrappers plus a source
  contract covering both callers and the lower-test wiring.

## Local validation boundary

- The v5 managed Runtime attempt is a valid compile failure receipt: it entered
  Cargo and exposed the missing typed methods after `654.54s` of compile/link
  work. It is not acceptance evidence.
- The combined Runtime863, UI reference visitor, Runtime200/205 capacity, and
  Runtime87 reference-resolution source-contract batch passes `32/32` in
  `0.020s`, with zero failures or errors.
- Exact-file Rustfmt passes for both UI asset owners, the registry extractor,
  and the two lower regression owners.
- The subsequent Runtime864 move-append optimization changes only the shared
  dependency append scratch shape; the typed UI V2 extraction contract remains
  covered in the combined `36/36` Runtime/asset source batch.
- The Runtime→Editor→App v6 batch was submitted asynchronously with the other
  current Runtime/Editor repairs. Its result must be reconciled against the
  exact source snapshot before it can supply managed evidence.
- The later bounded v6 Runtime receipt builds the repaired production source
  successfully at exit `0` (`651.8218801s` compile/link; input manifest
  `77b09f4d…`). It ran no lower tests and is not Release evidence.
- Tooling production remains deferred for the later Rust migration.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/asset/assets/ui.rs` | `E4C41B76D9AB8814216E397ADFC2D15E17B7AB7D4331AE6CDEA96A0F934C2B73` |
| `zircon_runtime/src/asset/assets/imported.rs` | `F4006FBB96D44EF7ACD947D0AF2E7275EE58EF9852F5068A354B73953991D0B8` |
| `zircon_runtime/src/asset/registry/dependency_extractors/mod.rs` | `910C3824F1F1B8B4F88573F20408F4C11C911E1B3B735B27F629BB87AF535A2C` |
| `zircon_runtime/src/asset/tests/assets/ui/references.rs` | `334001CABB4D6A9F4D26BFCF329ACACD42AD622932ECFF6321F110B93020708D` |
| `zircon_runtime/src/asset/tests/registry_index/dependency_extractors.rs` | `8A14A772B2A35B9AFE4988C6741671E6E54B6E24FC64BC9F98AD6DC7E3076AC2` |
| `tools/tests/test_runtime863_ui_v2_direct_reference_contract.py` | `BDDE7208D3FF4FD07E4998FFF8BB03C48AFA389775F11A424FEE50CD4F3F5D34` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
an owner-attributed current-source Windows lane compiles Runtime, executes the
lower regressions, and completes the required Release/allocator/product gates.
The local source contracts and the failed v5 compile do not establish product
performance acceptance.
