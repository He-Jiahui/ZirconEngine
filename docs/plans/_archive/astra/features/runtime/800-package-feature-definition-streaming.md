---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/200/2026-09-18-package-feature-definition-streaming.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/package_feature_definitions.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_definition_collection.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_definition_collection/package.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/runtime_feature_definitions/merge.rs
tests:
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/package_feature_definitions/capacity_tests.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_definition_collection/package/capacity_tests.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_definition_collection/capacity_tests.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/runtime_feature_definitions/merge/capacity_tests.rs
  - tools/tests/test_runtime200_package_feature_definition_streaming_performance_contract.py
---

# Runtime800 · package feature-definition streaming

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime200/205 plugin catalog merge and capacity | Consume package feature definitions through an optional-first visitor, reserve only product-catalog-eligible declarations, and preserve provider resolution, duplicate diagnostics, order, and carrier/test-fixture skip authority. | TDD RED→GREEN source contract `8/8`; lower order/provider, merge-wiring, and carrier-capacity regressions; deterministic model `256→0` temporary package vectors and `320→64` eligible slots; managed Cargo/Release and product percentile evidence remain pending. | implemented_pending_validation |

## Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/package_feature_definitions.rs` | `DE37730FA029C445159E1A6DE6DE6F3F052614996366736DDC3BE75B2B6503DC` |
| `zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_definition_collection.rs` | `6C464715FF3C761A898E45C17A1CCC7B045F97A870F36F45D94356108312634B` |
| `zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_definition_collection/package.rs` | `252D307BD9743099DE4F6DC69CC699A655342692CE6A93CABFFE32B2DE7DFAED` |
| `zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/runtime_feature_definitions.rs` | `44FE30B2A3264360144D26763EF42340F6FCBF4BBF089F289AECE1A39BB6BCB4` |
| `zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/runtime_feature_definitions/merge.rs` | `1272877AA888DB5416C50A0F835F68F1B52C193C670AC1F1738434FCE4D01033` |
| `zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/package_feature_definitions/capacity_tests.rs` | `7FED6F28EFE6E7C37B174E508A2660F9B9F8595579858C521B221CA6B2934B94` |
| `zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_definition_collection/package/capacity_tests.rs` | `B5BF7194DD9095378E1EEC537B1F3A2386154003FEA053683975936B387D7ED5` |
| `zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_definition_collection/capacity_tests.rs` | `A4DBDE2AACBD26D121F1E429A0B910349EAE7C7712C04C90D60FC5A453A057DB` |
| `zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/runtime_feature_definitions/merge/capacity_tests.rs` | `86E9CB38202B8981B187BD151422D8A79DD75ACC5277D684B0A49068C3B55652` |
| `tools/tests/test_runtime200_package_feature_definition_streaming_performance_contract.py` | `AE7DC2B4FBFBD7B786D8E7E95932F2CC93DECB1622E27EDA553417FE28E33D98` |

The local source contract, scoped Rustfmt, Python compilation, and diff checks
are green. The historical all-surface `test_*performance_contract.py` discovery
passed `2438/2438` in `64.061s`; the current non-tooling batch passes `2205/2205`
across `598` modules in `6.538s`. These are read-only source/model receipts and
do not establish Cargo compilation, Release allocation counts, or product
catalog-build p50/p95/p99.

## 2026-09-18 capacity-eligibility follow-up

The package and runtime capacity bounds now filter through the same
product-catalog eligibility predicate used by merge. Carrier and test-fixture
registrations remain visible to inspection and are still rejected by the merge
authority, but no longer reserve unreachable definition slots. The focused
Runtime200 source contract is `8/8`; lower Rust regressions cover both carrier
package and carrier feature registration capacity. A deterministic mixed model
of 256 carrier declarations plus 64 eligible declarations changes the bound
from `320` to `64`, removing 256 unreachable slots without changing merge
semantics. Exact-file Rustfmt for the six touched Runtime200/205 files and
`git diff --check` pass. Managed Windows/Cargo/Release and product percentile
gates remain pending.

## 2026-09-21 managed compile repair

The v5 Windows lane reached Cargo and exposed an `E0364`/`E0603` visibility
pair at the eligible-registration count helper. Its visibility now matches the
runtime-plugin-catalog owner that imports it, and the stale Runtime219 source
regression now asserts the exact package plus eligible-feature capacity model.
The combined Runtime repair source-contract batch passes `32/32` in `0.020s`.
The asynchronous v6 submission still requires source-snapshot reconciliation;
managed current-source compilation, Release, allocator, and catalog-build
p50/p95/p99 evidence remain pending.

The bounded v6 Runtime receipt now proves a managed Windows Cargo build at exit
`0` (`651.8218801s` compile/link; input manifest `77b09f4d…`). No tests ran and
the build is not Release evidence, so lower regressions, the ignored marker,
allocator data, and catalog-build p50/p95/p99 remain pending.

## 性能与受管验证边界

The visitor removes one package-level temporary vector per registration from
the merge hot path and keeps the aggregate reservation input-sized. Keep the
status `implemented_pending_validation` until the asynchronous batched lane
supplies lower Rust and product gates. This session did not query or poll the
coordinator, and tooling production remains intentionally deferred.
