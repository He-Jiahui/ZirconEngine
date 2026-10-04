---
title: Runtime04 Pack Reader Validated Read Reuse
category: zircon_runtime
report_id: Runtime04-pack-reader-validated-read-reuse-2026-09-27
date: 2026-09-27
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: release_and_product_pending
---

# Runtime04 pack reader validated read reuse

`ZrPackReader::from_bytes` parses and validates the manifest, payload extent,
and BLAKE3 hash of every unique chunk before it owns the pack bytes. Its
private byte vector is immutable throughout subsequent reads. Previously,
`read_asset` and `read_chunk_by_hash` each repeated the full payload hash on
every read, then returned an owned copy. Delta writing reads changed target
chunks, and delta application reads reused base chunks through these methods.

The read path now reuses the completed open-time hash validation. It still
checks asset size and chunk range, and still returns an independent `Vec<u8>`.
The existing `into_bytes` ownership transfer was already present in the shared
checkout and remains unchanged. This slice does not defer or remove any
open-time format, manifest, extent, or chunk-hash rejection.

## Regression and measurement

The focused pack regressions cover a borrowed input snapshot after its source
changes, repeated reads from a cloned reader, deduplicated aliases, isolated
owned output, empty and missing assets, and corruption of every unique chunk
at open. A repeated real delta write/apply case checks changed and reused
aliases, removed paths, exact target bytes, and stable results across reads.

The existing `reader/optimization_tests.rs` owner now also contains ignored
`runtime04_pack_reader_validated_read_reuse_bench`. Its test-local legacy path
reproduces the prior lookup, size/range check, BLAKE3, and owned copy against
the same already-opened reader. A 64-asset, 256 KiB per asset pack is validated
for exact old/new bytes and chunk hashes before timing. Each sample reads a fixed sequence of
128 paths; five warmup pairs precede 31 alternating old/new pairs. Marker
`RUNTIME04_PACK_READER_VALIDATED_READ_REUSE_BENCH_V1` emits both raw series,
nearest-rank P50/P95/P99, pack fingerprint, OS, architecture, and package version. The focused
Release gate requires optimized P95 to be at most 95% of legacy P95. Run it in
the grouped managed Runtime Release batch with filter
`runtime04_pack_reader_validated_read_reuse_bench`, `--ignored --nocapture
--test-threads=1`, and a sealed source, compiler, target, CPU, and cache state.

The existing `runtime04_pack_export_cli_delta_product_1k` is the product path
for a later matched-binary comparison, since it executes real export, delta
write, and apply. It requires two independently sealed Release binaries and
the fixture's three explicit environment inputs; the current-binary baseline
alone cannot prove a before/after speedup. Its P95 <=105%, P99 <=110%, and
working-set P95 <=110% ceilings are provisional regression checks, separate
from this reader's <=95% focused performance target.

No Cargo, Release timing, or product measurement ran in this source slice.
Runtime04 M7 pack mount, random section reads, 1/10/100 GiB bundles, and
bounded runtime RSS remain open. The local Release gate does not close them.
