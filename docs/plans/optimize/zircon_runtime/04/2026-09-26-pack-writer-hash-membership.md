---
title: Runtime04 Pack Writer Single Hash Membership
category: zircon_runtime
report_id: Runtime04-pack-writer-hash-membership-2026-09-26
date: 2026-09-26
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_work_reduction_met_dynamic_pending
---

# Runtime04 pack writer hash membership

The pack assembler used a `BTreeMap<hash, offset>` to detect duplicate payloads. Its offset value
was never used to build the pack; unique assets first performed `get` and then `insert`, walking the
tree twice. The writer now keeps only a `BTreeSet<hash>` and admits each payload with one `insert`.
Chunk offsets remain in `chunk_entries`. Sorted input order, canonical chunk ordering, duplicate
reporting, and the second file read/hash check are unchanged. A failed second read discards the
private assembler without publishing a pack.

| Evidence | Before | After / acceptance target |
| --- | ---: | ---: |
| Tree lookups per unique payload | 2 | 1 |
| Stored deduplication value | hash + offset | hash only |
| Pack bytes and deduplication | Existing memory/file parity test | Same test in grouped Runtime gate |
| Release P95, 8192 unique + 2048 duplicate hashes | pending | at most 95% of legacy |

`RUNTIME04_PACK_WRITER_SINGLE_HASH_MEMBERSHIP_BENCH_V1` is an ignored Release benchmark. It uses
five alternating warmup pairs followed by 31 alternating measured pairs and eight iterations per
side of each pair. The legacy map lookup and new set admission use the same prebuilt keys. It
retains and prints both raw series in acquisition order, P50/P95/P99, OS, architecture, and package
version. The managed receipt must supply compiler, build profile, machine, and cache-state metadata.
The grouped Runtime validation must run this benchmark and the pack writer parity tests before any
dynamic performance claim. This helper measures hash admission only; real pack export throughput,
peak RSS, and Runtime04 product-scale targets remain pending. No Cargo or Release timing result is
claimed by this source slice.
