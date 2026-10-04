---
title: Runtime04 Pack Export Delta Reader Ownership Transfer
category: zircon_runtime
report_id: Runtime04-pack-export-delta-reader-move-2026-09-26
date: 2026-09-26
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_copy_elimination_release_and_product_pending
---

# Runtime04 pack export delta reader ownership transfer

The real `zircon_export_pack` delta path writes a `ZrPackDeltaWriteReport` to disk, then
cloned its entire `bytes` vector to construct `ZrPackDeltaReader` for verification. The
report keeps its manifest and asset lists independently of that vector, so the reader now
takes ownership of the written bytes. Delta decoding, changed-asset verification,
base application, byte-for-byte target comparison, and JSON report fields are unchanged.

| Evidence | Before | After / acceptance target |
| --- | ---: | ---: |
| Extra full-delta allocation and copy at reader handoff | One delta-sized copy | Zero; move the existing vector |
| Changed/reused/removed report and reconstructed target bytes | Existing behavior | Exact parity in the real `run()` delta export regression |
| Release isolated delta-sized clone P50/P95/P99 | Pending | Report measured copy cost separately from reader validation |
| Release reader-handoff P95 on 1,024 changed 16 KiB assets | Pending | At most 95% of legacy clone P95 |
| Production delta-stage P50/P95/P99 | Pending | Report actual timings; no product latency pass claimed |

The ignored `RUNTIME04_PACK_EXPORT_DELTA_READER_MOVE_BENCH_V1` runs five warmups, then
31 alternating old-clone/new-move reader handoff pairs on a real delta generated from
1,024 changed 16 KiB assets. It times the isolated full-delta clone separately from
reader validation, and also times 31 calls to the production delta export stage, including
base-pack file read, delta generation/write, reader verification, reconstruction, and
target-byte comparison. Input construction is outside the timed samples. The production
stage data has no old full-stage comparator, so it is diagnostic rather than proof of
end-to-end speedup or the broader Runtime04 1/10/100 GiB pack qualification.
The benchmark retains both source and cloned vectors after timing, checks their buffers
are distinct, and prints all 31 raw samples in acquisition order with OS, architecture,
and package version. The managed receipt must supply compiler, build profile, machine,
and cache-state metadata before this can count as a performance result.

Grouped managed Runtime test and Windows-native Release results, machine/build/cache
metadata, memory peak, and product P99 are pending. No dynamic result is claimed by
this source slice.
