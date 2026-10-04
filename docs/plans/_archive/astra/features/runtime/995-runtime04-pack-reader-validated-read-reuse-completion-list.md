---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/04/2026-09-27-pack-reader-validated-read-reuse.md
implementation_files:
  - zircon_runtime/src/asset/pack/reader.rs
tests:
  - zircon_runtime/src/asset/tests/pack/basic.rs
  - zircon_runtime/src/asset/tests/pack/reader_validation.rs
  - zircon_runtime/src/asset/tests/pack/delta_pack.rs
  - zircon_runtime/src/asset/pack/reader/optimization_tests.rs
---

# Runtime04 pack reader validated read reuse completion list

| Completed slice | Evidence in source | Remaining acceptance |
| --- | --- | --- |
| Reuse the full open-time chunk validation for later reads of immutable pack bytes. | `from_bytes` still validates manifest, extent and every chunk hash; `read_asset` and `read_chunk_by_hash` retain range, size and owned-copy behavior without hashing the same payload again. The preexisting `into_bytes` method is preserved. | Grouped managed Runtime build/tests must pass for the exact Batch K source snapshot. |
| Add direct pack and delta behavior regressions. | Borrowed-input snapshot, repeated reads, cloned reader, alias, empty, missing, corruption-at-open, and repeated delta byte parity cases. | Run the relevant `asset::tests::pack` selection in the grouped managed Runtime test batch. |
| Add a Release comparison in the existing reader optimization test owner. | Test-local pre-change read path and public optimized path use the same 64 x 256 KiB pack and 128 reads per sample; five warmups and 31 alternating pairs print raw P50/P95/P99. | Execute ignored `runtime04_pack_reader_validated_read_reuse_bench` in one managed Release batch; optimized P95 must be <=95% of legacy P95. Seal environment and source. |

The real 1K delta export CLI old/new binary comparison remains pending and must
confirm pack/delta output parity and report product wall time and memory. No
dynamic speedup or Runtime04 M7 random-read/RSS acceptance is claimed yet.
