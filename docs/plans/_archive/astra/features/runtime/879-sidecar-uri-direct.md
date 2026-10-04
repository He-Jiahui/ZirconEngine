---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/04/2026-09-21-sidecar-uri-direct.md
related_records:
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/asset/migration/sidecar.rs
tests:
  - zircon_runtime/src/asset/migration/sidecar/direct_uri_tests.rs
  - tools/tests/test_runtime879_sidecar_uri_direct_performance_contract.py
---

# Runtime879 Sidecar URI Direct

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime04 migration sidecar resource URI | Append normalized/lossy relative components to one `res://` output instead of collecting vector slots, joining a child, and formatting a wrapper; preserve invalid-Unicode text and existing URI/error authority. | RED two missing-symbol/module errors -> GREEN `3/3`; nearby Runtime/Editor contracts `48/48`. The 4,096-path/32-component model removes `131072` reference slots and `8192` child strings. Lower path parity and ignored 101-pair `RUNTIME879_SIDECAR_URI_DIRECT_BENCH_V1` are authored but not Cargo-run. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/asset/migration/sidecar.rs` | `D196B0D396197B4045A2BDE6F7274DB84CA7276A25D708F460C479AE0EB9327D` |
| `zircon_runtime/src/asset/migration/sidecar/direct_uri_tests.rs` | `3BDA7648DD58876BCCB243D365C02C762FC033A5B77E99E8BCFAC343456DBA99` |
| `tools/tests/test_runtime879_sidecar_uri_direct_performance_contract.py` | `8BF4822168112230A2C8F2E87EDB1B7EDD4A65DBD58FBC74F6E6D1D84607B506` |

## Managed gate

Runtime879 was implemented after v25 source submission. The v25 Runtime
package encountered unrelated foreign `compile_input_changed`, and v24's
Runtime development success predates this change. It belongs to the next
coalesced Runtime/Editor validation wave; current-source Rust compilation,
lower/ignored Release tests, allocator measurements, and migration product
p50/p95/p99 remain pending.

The grouped v26 library-test request failed before Cargo on malformed
temporary command JSON. Its owner-corrected v27 Runtime/Editor batch is
submitted but has no attributable Rust-test receipt yet. The sidecar helper's
ignored Release comparison does not replace a source-bound migration product
fixture or its allocation and percentile gates.
