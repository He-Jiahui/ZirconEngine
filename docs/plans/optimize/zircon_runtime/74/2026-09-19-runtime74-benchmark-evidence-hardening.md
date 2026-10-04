---
title: Runtime74 Release Benchmark Evidence Hardening
category: zircon_runtime
report_id: Runtime803-runtime74-benchmark-evidence-hardening-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260919
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: evidence_shape_hardened
---

# Runtime803 · Runtime74 Release benchmark evidence hardening

## Scope

The six older Runtime74 UI/template helper probes used 17 alternating samples and reported only
partial percentile data. This slice changes the Rust Release probes—not tooling production—to use
101 alternating samples, balanced 51/50 first-order execution, and explicit nearest-rank P50/P95/P99
fields. Production algorithms and threshold comparisons are unchanged.

## Implementation

- Component-contract hash index reports build and lookup P50/P95/P99 values.
- Dependency-cascade, hot-reload admission, watch invalidation, UI prototype, and UI-v2 prototype
  probes report the same percentile set and sample/order metadata.
- A source contract covers all six markers and rejects regression to 17-sample evidence.
- Existing P95 thresholds remain the only local assertion; P50/P99 are emitted for independent
  managed analysis.

## Validation boundary

The focused Python contract is expected to run as one batch with the surrounding Runtime/Editor
contracts. Rustfmt and source checks are local evidence only; Cargo compilation, ignored Release
execution, complete caller coverage, allocator observations, and product p50/p95/p99 acceptance
remain pending under the external `E:\Git\zr_vm` dirty-worktree admission. Tooling production is
unchanged and remains deferred for the later Rust migration.

The current recent-record Rustfmt batch covers `169` Rust files referenced by
the Runtime/Editor optimize set (`74` Runtime, `84` Editor, and `11` shared
plugin/interface owners) and passes with zero diffs after mechanical formatting
of the two Runtime benchmark owners listed above; production algorithms and
threshold assertions are unchanged.

## Source files

| File | Change |
| --- | --- |
| `zircon_runtime/src/ui/template/asset/component_contract/validation/hash_index_tests.rs` | 101 pairs and build/lookup P50/P95/P99 marker fields |
| `zircon_runtime/src/ui/tests/asset_dependency_index.rs` | 101 samples and cascade P50/P95/P99 marker fields |
| `zircon_runtime/src/ui/template/asset/hot_reload_plan.rs` | 101 samples and balanced admission marker fields |
| `zircon_runtime/src/ui/template/asset/prototype_store/hash_index_tests.rs` | 101 samples and P99/order marker fields |
| `zircon_runtime/src/ui/v2/cache/hash_lookup_tests.rs` | 101 samples and P99/order marker fields |
| `zircon_runtime/src/ui/template/asset/watch_invalidation.rs` | 101 samples and balanced admission marker fields |
| `tools/tests/test_runtime74_benchmark_evidence_contract.py` | Cross-file evidence-shape source contract |
