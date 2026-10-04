---
title: Editor02 History Snapshot Binary Top Lookup
category: zircon_editor
report_id: Editor02-history-snapshot-binary-top-2026-09-14
date: 2026-09-14
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor02 History Snapshot Binary Top Lookup

## Scope

`TransactionHistorySnapshot::from_page` marks the applied prefix by locating
the authoritative history `top` inside the returned page. The page is emitted
by `HistoryStore::detail_window` in transaction-id order, but the projection
previously performed a linear `position` scan on every page. The lookup now
uses `binary_search_by_key`, preserving the existing missing-top semantics for
truncated pages and keeping the public snapshot shape unchanged.

This is a bounded projection optimization. It does not change history routing,
save-token semantics, undo/redo behavior, or the wider Editor02 persistence and
recovery gaps.

## Complexity and deterministic target

For a page of `P` ordered records, locating a visible top changes from `O(P)`
comparisons to `O(log P)` comparisons and does not allocate a side index. A
missing top still returns `None`, so a top on a later page continues to mark the
visible page's prefix exactly as before. The release benchmark uses a 16,384
record tail lookup, alternating legacy and optimized samples, and requires the
optimized P95 to be at least 75% below the linear scan.

## TDD and local evidence

- The source contract was intentionally RED before the implementation because
  the production body contained `records.iter().position` and no binary lookup.
- The focused history/transaction batch passes `40/40`.
- The combined Runtime/Editor pointer, asset-reference, history, and input
  batch now passes `76/76` in one invocation; no per-task Cargo process was run.
- A subsequent single-process Runtime/Editor performance-contract discovery
  loaded both patterns and passed `1823/1823` tests in `22.612s`; this is local
  source/model evidence, not managed Cargo or Release evidence.
- Scoped Rustfmt, Python compilation, and `git diff --check` pass.
- The Rust behavior regression covers a 16,384-record ordered page and verifies
  the tail top and applied-prefix semantics. The ignored release benchmark
  emits `EDITOR02_HISTORY_SNAPSHOT_BINARY_TOP_BENCH_V1` with paired P50/P95
  samples.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/workbench/snapshot/data/transaction_history_snapshot.rs` | `AFC0DD489FD835C35E75CEAB50CB9D6FC057A721DA22644718DF81C285F2E42C` |
| `zircon_editor/src/tests/workbench/transaction_history_snapshot.rs` | `F16BDCB001C9050C42E42E9ACA0720EA6FB3F6A6FB25D0441C53017A9C23650C` |
| `tools/tests/test_editor_transaction_history_snapshot_binary_lookup_performance_contract.py` | `75C4AD5A88A88ABBB5DB61EF7067F35DCAF92078F0E1744DBF9DDCF420B2086F` |

## Managed gate

This slice joins the existing owner-attributed Runtime/Editor Windows Release
batch. Managed compilation, allocator behavior, and Editor history projection
p50/p95/p99 measurements remain pending under the shared external-worktree
admission blocker; no coordinator status polling or standalone Cargo run is
required. Tooling production work remains deferred.
