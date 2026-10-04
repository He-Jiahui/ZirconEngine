---
title: Runtime04 Delta Reader Validated Read Reuse
category: zircon_runtime
report_id: Runtime04-delta-reader-validated-read-reuse-2026-09-27
date: 2026-09-27
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: release_and_product_pending
---

# Runtime04: reuse validated delta payloads on repeated reads

## Source finding and repair

`ZrPackDeltaReader::from_bytes` validates the document, payload extent and
every changed chunk's BLAKE3 hash before returning a reader. Its owned byte
buffer and manifest are private and immutable. `read_changed_asset` repeated
the payload hash on every read, including delta apply's changed-asset branch.

The read path now retains lookup, size and range checks and returns the owned
copy without hashing the same immutable payload again. Opening a corrupted
delta remains a typed `ChunkHashMismatch` failure before any consumer receives
a reader. The patch preserves the preexisting delta HashSet/manifest changes.

## Behavior and performance evidence

`delta/validated_read_tests.rs` adds behavior coverage for borrowed-input
snapshot isolation, cloned readers, independent owned outputs, deduplicated
aliases, repeated reads, missing assets and corruption of every unique chunk
at open. Existing delta format, extent, base/target and apply tests remain
applicable. The companion Batch K pack-reader tests exercise exact repeated
delta generation/application parity.

The ignored `runtime04_delta_reader_validated_read_reuse_bench` uses a real
writer/base/target/delta fixture with 64 unique 256 KiB changed chunks plus an
alias. Open, fixtures and byte-equivalence checks precede timing. Each sample
performs 128 fixed shuffled public reads; the test-local legacy route retains
the original lookup, size/range checks, BLAKE3 and owned copy. Five alternating
warmup pairs and 31 alternating raw measured pairs emit delta hash, environment
and nearest-rank P50/P95/P99. The local P95 gate is new <=95% of old.

Submit with the other Batch K Runtime/Editor filters through managed Windows
Release validation, with serial test execution. The exact ignored filter is
`runtime04_delta_reader_validated_read_reuse_bench` and marker is
`RUNTIME04_DELTA_READER_VALIDATED_READ_REUSE_BENCH_V1`. No Cargo or benchmark
result exists yet. Static checks alone do not establish the ratio.

## Acceptance boundary

This reduces read-time byte scans in the existing in-memory delta/export path.
Runtime mount/section streaming, 1/10/100 GiB random read and bounded RSS,
authenticated installation, and matched-binary CLI/product percentiles remain
open under the Runtime04 main plan. This record does not close those gates.
