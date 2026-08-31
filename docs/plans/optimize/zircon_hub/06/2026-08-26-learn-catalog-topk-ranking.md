---
title: Hub 06 Learn Catalog Bounded Top-K Ranking
category: zircon_hub
report_id: Hub06-learn-catalog-topk-ranking-2026-08-26
date: 2026-08-26
session_id: root-hub06-three-task-performance-batch-r2-20260831
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Hub 06 Learn Catalog Bounded Top-K Ranking

## Scope

- Parent gate: Hub06 `D01`, specifically Learn catalog ranking after discovery.
- Baseline: `94f86015d0da980d6c93ef3cf3fcd9d759d0e477`, epoch `430`.
- Owners: `zircon_hub/src/learn/catalog.rs`, the source/performance contract, and this record.
- This slice preserves full Markdown discovery and the existing 128-result API while reducing ranking work. It does not claim depth/time/entry/byte scan budgets, cursor/truncation/freshness metadata, bad-document isolation, Asset/Plugin changes, or complete `D01` qualification.

## Change

- Extracted the existing source/root/source/category/title/path total order into one comparator shared by selection and final sorting.
- For more than 128 candidates, partitioned the vector at the catalog limit with `select_nth_unstable_by`, discarded the suffix, and sorted only the retained prefix.
- Preserved selected-project priority, fallback-root rank, every tie-break, output length, and returned ordering.
- Added a filesystem-backed Rust regression that creates 150 reverse-named Markdown documents and requires the exact sorted `Guide 000` through `Guide 127` prefix.

Ranking complexity changes from O(N log N) full sorting to average O(N + K log K), with fixed `K = 128`. Discovery, Markdown reads, and candidate allocation remain O(N) and are explicitly outside this slice.

## TDD and Local Evidence

- RED: `python -m unittest tools.tests.test_hub06_learn_catalog_topk_performance_contract -v` failed `4/4` against the full-sort implementation.
- Current-source benchmark RED: the two new native-evidence contract cases failed while the four
  existing algorithm cases remained green.
- GREEN: the current contract now passes `6/6`; the complete Hub06 three-task contract batch passes
  `15/15`.
- An isolated `rustc --test` harness runs the actual Learn catalog module and passes `5/5`, including the new 150-to-128 exact-prefix regression.
- Rust 1.94.1 targeted formatting and scoped `git diff --check` pass, apart from Git's existing LF/CRLF checkout notice.

The standalone optimized Rust model compares the same six-key total order over 100,000 deterministic candidates, retains 128 entries, first proves exact vector equality, then measures 21 alternating sample pairs with nearest-rank percentiles. Input cloning is outside timed intervals.

| Metric | Full sort + truncate | Partial select + prefix sort | Change |
|---|---:|---:|---:|
| P50 | 12.7228 ms | 0.8269 ms | -93.501% |
| P95 | 14.0535 ms | 1.1014 ms | -92.163% |

These numbers isolate ranking cost and do not claim full filesystem or Markdown discovery latency.
The historical model is now embedded in the actual Learn module: its optimized side calls
`retain_top_ranked_entries`, first proves exact output parity, emits
`HUB06_LEARN_CATALOG_TOPK_BENCH_V1`, and enforces at least 35% lower P50 and P95 over 100,000
entries.

## Async Validation

One coordinator batch runs the focused Asset/Learn/queue behavior and benchmarks under Rust 1.94.1:
`cargo test --manifest-path zircon_hub/Cargo.toml --bin zircon_hub --locked --release --jobs 1 --
hub06_ --include-ignored --nocapture --test-threads=1`. Acceptance requires exact top-K parity, the
filesystem regression to pass, and ranking P50/P95 improvements of at least 35%.

The current-source successor Session owns the exact three-task union. Integration remains
coordinator-owned after managed validation, required rebase handling, and independent review.
Automatic WeCom publication must include the managed ranking row and label it as ranking-only
evidence.
