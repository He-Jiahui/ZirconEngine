---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/206-runtime-asset-registry-project-catalog-index-persistence-rebuild-incremental-query-watch-generation-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/206/2026-08-31-asset-type-posting-index.md
  - docs/plans/optimize/zircon_runtime/206/2026-09-13-secondary-query-postings.md
related_records:
  - docs/plans/astra/features/runtime/695-20260911-runtime-query-registry-builtin-diagnostics-optimization.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
  - docs/plans/astra/features/runtime/743-runtime206-contract-repair.md
  - docs/plans/astra/features/runtime/745-runtime206-referencer-binary-sort.md
  - docs/plans/astra/features/runtime/746-runtime206-registry-build-capacity.md
related_code:
  - zircon_runtime/src/asset/registry/asset_registry_index.rs
  - zircon_runtime/src/asset/registry/query.rs
  - zircon_runtime/src/asset/registry/asset_registry_index/secondary_query_tests.rs
tests:
  - tools/tests/test_runtime206_secondary_query_index_performance_contract.py
---

# Runtime206 Secondary Query Postings

## 计划完成列表

| Work | Status | Evidence |
| --- | --- | --- |
| Tag posting | implemented_pending_validation | `AssetRegistryIndex` maintains `uuids_by_tag` on insert and source removal; missing/empty tags short-circuit without scanning the UUID table. |
| Package posting | implemented_pending_validation | `uuids_by_package` is keyed from `AssetUri::package_id()` and retires empty package buckets. |
| Path-prefix candidate index | implemented_pending_validation | Ordered `uuids_by_path_prefix` range traversal gathers exact-path buckets, preserving existing `starts_with` semantics and labeled subassets. |
| Candidate convergence | implemented_pending_validation | Non-type `get_assets` selects the smallest direct posting when available, uses a borrowed `str` path-range bound, and builds a path-range UUID union only when no direct posting exists before applying the exact predicates and URI ordering. |
| Behavior and source contracts | implemented_pending_validation | Composed-filter, labeled-subasset, source-removal, and empty-bucket tests plus the Runtime206 source/pressure contract are authored; managed Cargo remains pending. |

## 批量验证状态

The focused Runtime206/Runtime85 source-contract batch passes `20/20`. A
single comprehensive non-tooling Runtime/Editor performance-and-pressure
loader covers `551` files and passes `2052/2052` tests in the latest
current-source batch (`80.282s` under the current local load). This is local static/model evidence only;
managed/Release status is not polled here,
per the asynchronous validation policy. Rustfmt parse-only and scoped
whitespace/diff checks pass locally. Tooling production changes remain out of
scope.

The follow-up path-prefix guard also verifies a borrowed `str` range bound,
removing the per-query prefix clone while preserving the same ordered range
semantics.

The refreshed shared Runtime/Editor performance-plus-pressure loader covers
`556` modules and passes `2068/2068` tests in `29.116s`; the focused
Runtime206/Runtime85 plus Editor reference batch passes `37/37` in `111.865s`.

## 性能边界

The deterministic pressure model uses 1,048,576 entries and 32 uniform tag
postings, reducing candidate visits from 1,048,576 to 32,768 (32x fewer,
96.875% reduction). This is a candidate-count model, not a claim about
Windows Release p50/p95/p99, allocator behavior, RSS, or product acceptance.

## 源码指纹

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/asset/registry/asset_registry_index.rs` | `8E1478D5C5C23769F241B2DE57C81806C039BFC1D9EF91B8F8A6FCF07024141E` |
| `zircon_runtime/src/asset/registry/query.rs` | `0A867CFBE7029C1BEB92A2A09D337C04E628230E33D15E8579115ECB8A3E7FDF` |
| `zircon_runtime/src/asset/registry/asset_registry_index/secondary_query_tests.rs` | `B422F76B4A9EDE3F5AE307DB827979B3AB764D56C5F16D72460A093E3E823B51` |
| `tools/tests/test_runtime206_secondary_query_index_performance_contract.py` | `9C964FF6475494E8FB507BFB6CE96143731463890C5C57F145ACA4A97C3CCB1E` |

## 未闭合项

Runtime206 P1-041 remains partial. Compiled query plans, cursor/visitor and
early-stop APIs, result/deadline budgets, generation leases, unified query
dispositions, large-corpus measurements, and the managed Windows Rust gate
remain owned by the parent plan.

Referencer ordering and bulk-build staging follow-ups are recorded in
`745-runtime206-referencer-binary-sort.md` and
`746-runtime206-registry-build-capacity.md`; they remain under the same
deferred managed validation gate.
