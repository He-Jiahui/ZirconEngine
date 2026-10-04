---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/04/2026-09-26-pack-export-cli-product-baseline.md
implementation_files: []
tests:
  - zircon_runtime/tests/runtime04_pack_export_cli_performance.rs
---

# Runtime04 pack export CLI product baseline completion list

| Completed slice | Evidence in source | Remaining acceptance |
| --- | --- | --- |
| Added real Windows Release `zircon_export_pack` process workloads at 1K and 100K assets. | Deterministic 90% unique corpora, report and pack-hash checks, five warmups, 31 raw wall/working-set/commit samples, P50/P95/P99. Child and suite deadlines bound the expensive 100K run. | Grouped managed Release test `runtime04_pack_export_cli_current` must run with `--locked` and `--test-threads=1`, then produce raw 1K/100K markers with source, machine, storage and cache-state receipt. |
| Added opt-in old/new full CLI comparison and 1K delta workload. | Distinct executable and claimed matched-source inputs are required; 31 alternating AB/BA pairs check pack/delta byte hashes, report semantics and provisional latency/memory ratios. Delta base-pack preparation uses a bounded real CLI child with stderr and temporary-corpus cleanup on failure. | Seal both source snapshots and binaries, then run selected comparison tests serially through a managed lane that can supply the three required environment variables. No missing-variable fallback is accepted. |

The new test only adds a performance fixture. It has not been compiled or run
in this slice. The 1K/100K pack export M0 product data, comparison thresholds,
peak memory, and overall Runtime04 performance target are pending. Pack mount,
random section reads and 1/10/100 GiB workloads remain open.
