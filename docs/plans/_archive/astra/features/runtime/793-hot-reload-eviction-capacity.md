---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/74/2026-09-17-hot-reload-eviction-capacity.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/template/asset/hot_reload_plan.rs
tests:
  - zircon_runtime/src/ui/template/asset/hot_reload_plan/eviction_capacity_tests.rs
  - tools/tests/test_runtime_hot_reload_eviction_capacity_performance_contract.py
---

# Runtime793 · Hot-reload compile-cache eviction capacity

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime74 hot-reload plan | Reserve the saturating sum of template-rebuild and removed-compiled target lengths before cache eviction admission, preserving order and report semantics. | TDD RED→GREEN source contract `3/3`; lower empty/ordinary/overflow regression; ignored `RUNTIME793_HOT_RELOAD_EVICTION_CAPACITY_BENCH_V1`; deterministic 16,384-target model `13→0`; managed Cargo/Release, allocator, and product percentile evidence remain pending. | implemented_pending_validation |

## Implementation evidence

`UiAssetHotReloadPlan::evict_compile_cache` now admits a target vector from the
two already-materialized input lengths with saturating arithmetic, then performs
the same two borrowed `extend` operations. No cache ownership, eviction
ordering, duplicate policy, or report field changed.

The lower module is wired beside the existing hot-reload-plan optimization
tests. Its non-ignored test checks empty, ordinary, and overflow-safe bounds;
the ignored marker models the large joined-target pressure case reserved for
the managed Release lane.

## Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/template/asset/hot_reload_plan.rs` | `01A2E24AE318F9ED4180BF7A7A61BA30A5239F8180D7E2BA31CDD21BA96C1A4F` |
| `zircon_runtime/src/ui/template/asset/hot_reload_plan/eviction_capacity_tests.rs` | `3599A9372A31EB6A57B4959AC2B2F29EB0FD34C1099691D814FFA90C1110C277` |
| `tools/tests/test_runtime_hot_reload_eviction_capacity_performance_contract.py` | `D6CCDC6B0F71A1C18D2C5369DF74DF6CA3EE99A69C9FD6232DBBB905CA135249` |

The focused source contract passes `3/3`; exact-file Rustfmt and Wiki
validation pass. The merged current-source Runtime/Editor performance-contract
batch after Runtime794 covers `557` files and passes `1990/1990` tests in
`71.984s`, with zero failures, errors, or skips. The broad Runtime/Editor
non-tooling regression batch covers `921` files and passes `3740/3740` tests in
`882.184s`, also with zero failures, errors, or skips. These are local
source/model receipts only. A later combined Runtime795/796/797 and Editor798
source-contract rerun passes `2002/2002` across `561` files in `12.503s`.

## 性能与受管验证边界

The deterministic model removes 13 modeled vector-growth events for the
16,384-target fixture (`13→0`). Local source contracts, lower Rust formatting,
and the batched Runtime/Editor checks do not establish Rust Cargo compilation,
Windows Release allocation counts, or product hot-reload p50/p95/p99. Keep this
record `implemented_pending_validation` until the asynchronous managed batch
supplies those gates. This session does not query or poll the coordinator, and
tooling remains intentionally deferred.
