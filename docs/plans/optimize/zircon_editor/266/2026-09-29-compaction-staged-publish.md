---
title: Editor266 durable journal compaction staged publication
category: zircon_editor
report_id: Editor266-ED-FSIO-P1-023-2026-09-29
date: 2026-09-29
session_id: astra-optimize-20260926-batch-a
implementation_status: blocked_by_active_source_owner
validation_status: not_submitted
performance_status: not_measured
---

# Editor266: staged compaction publication handoff

## Finding and ownership

The journal writer already streams retained records into a sibling staging file and syncs it. The old publication path then read that complete file into a `Vec<u8>` and called `atomic_write`, which wrote the same new payload into another staging file. This added one full-file read, one full-file write, and a journal-sized temporary allocation during every compaction.

Runtime Resource I/O owns the cross-platform replacement, backup, recovery, and durability barriers. A direct publication operation should accept the already-synced sibling and use that owner. Editor should only build and publish the journal payload. The existing backup-free `replace_staged_file` is reserved for transactions with their own backup owner.

## Ownership disposition

The new public export would require `zircon_runtime/crates/zr_resource/src/io/atomic_file/mod.rs`. Its current content is attributed to active Source comment audit M1 Session `1d9a143c-c6a1-4da3-a702-eeeff3e26b8f`. Exact read-only ownership transfer preview fingerprint `12861d0f74c5dcb5eca63b7f176ff824d034849a393c0adaeba5b7e3c6e41321` at baseline epoch 628 returned `source_owner_executable`. A temporary candidate patch was withdrawn from that shared module and the dependent Editor/Runtime facade edits were withdrawn. The existing Editor266 review row stays Open. No Cargo ticket or Release benchmark was submitted for this slice.

## Next eligible slice and gates

- When the source owner completes or an authorized transfer becomes eligible, add a Runtime Resource I/O entry point that accepts a regular, already-synced sibling file, validates the sibling relationship, and reuses the current `PendingAtomicWrite::commit` backup/recovery/durability path. Do not change the backup-free transaction primitive's contract.
- Change `DurableJournal::compact_covered_prefix` to consume the sibling through that entry point while preserving ordered suffix, sequence, and failure-cleanup behavior.
- Run lower replacement, invalid-path, and post-publication failure regressions with the Editor compaction regression in one managed Windows batch. Compare paired large-journal Release p50/p95/p99 and allocation/RSS. The deterministic target is zero additional full-payload reads, writes, and journal-sized allocations during publication; old-target backup fallback I/O remains possible.

## Scope

This handoff is a design and ownership record, not completed implementation. OS writer ownership, multi-document transaction receipts, and journal generation lineage are separate open rows in the Editor266 review.
