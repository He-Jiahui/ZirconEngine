---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/160/2026-09-26-runtime-asset-root-single-metadata-probe.md
implementation_files:
  - zircon_runtime/src/asset/runtime_asset_path.rs
tests:
  - zircon_runtime/src/asset/runtime_asset_path.rs
---

# Runtime asset root single metadata probe completion list

| Completed slice | Evidence | Remaining acceptance |
| --- | --- | --- |
| Non-verbose candidate admission uses one metadata lookup per existing root. Verbose path-list diagnostics and the later selection pass each probe the candidate once, for two lookups total. | Added missing/file/directory selection regression and retained existing product-root ordering tests. | Grouped managed Runtime test filter `asset::runtime_asset_path::tests`; ignored Release marker `RUNTIME_ASSET_ROOT_SINGLE_METADATA_PROBE_BENCH_V1` must meet the 95% P95 target for the non-verbose probe. |

The source and benchmark are ready for the batch validation lane. Dynamic performance and product
acceptance stay open until terminal results are available.
