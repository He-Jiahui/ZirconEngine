---
doc_type: completion-list
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/206/2026-09-27-source-removal-referencer-pruning.md
implementation_files:
  - zircon_runtime/src/asset/registry/asset_registry_index.rs
tests:
  - zircon_runtime/src/asset/registry/asset_registry_index/source_removal_pruning_tests.rs
---

# Runtime206 source removal referencer pruning completion list

| Work | Implementation evidence | Remaining acceptance |
|---|---|---|
| Remove the whole UUID reverse-index scan from each source removal. | Only outgoing buckets of removed rows are examined for emptiness. Missing sources perform no reverse scan. Shared, self and incoming references retain their existing meaning. | Independent source review is complete; grouped managed execution remains pending. |
| Preserve complete index and candidate behavior. | Four real regressions cover root/subasset grouping, all secondary postings, missing/repeated/final deletion, reinsertion and actual candidate clone/refresh/diagnostics. Complete prior implementations are frozen; P preserves O as an immutable predecessor and records the exact production delta. | Passing compilation and execution, with exact source attribution, pending. |
| Supply source-removal scale comparisons. | 10K/100K/1M missing/single/128-source cases; 10K/100K real candidate preparation including cloning; current 100K-source removal. Five warmups and 31 samples/pairs report raw p50/p95/p99. | Local removal p95 <= 80% guard and all measured results pending. Full Runtime206 allocation/RSS, I/O, watcher-visible, native and Unreal product gates remain open. |
