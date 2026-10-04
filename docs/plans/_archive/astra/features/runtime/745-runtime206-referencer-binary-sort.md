---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/206-runtime-asset-registry-project-catalog-index-persistence-rebuild-incremental-query-watch-generation-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/206/2026-09-13-referencer-binary-sort.md
related_records:
  - docs/plans/astra/features/runtime/695-20260911-runtime-query-registry-builtin-diagnostics-optimization.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
  - docs/plans/astra/features/runtime/744-runtime206-secondary-query-postings.md
related_code:
  - zircon_runtime_interface/src/resource/asset_uuid.rs
  - zircon_runtime/src/asset/registry/query.rs
  - zircon_runtime/src/asset/registry/asset_registry_index/secondary_query_tests.rs
tests:
  - tools/tests/test_runtime206_referencer_binary_sort_performance_contract.py
  - tools/tests/test_runtime206_secondary_query_index_performance_contract.py
---

# Runtime206 Referencer Binary Sort Key

## 计划完成列表

| Batch | Work | Status | Validation evidence |
| --- | --- | --- | --- |
| Runtime206 / P1-047 | Add a borrowed UUID byte key and use it for deterministic referencer sorting | implemented_pending_validation | TDD source contract `3/3`; behavior regression covers binary/display order equivalence; deterministic pressure model removes `65,536` owned sort keys. |
| Runtime206 / adjacent postings | Keep the tag/package/path-prefix candidate postings and source-removal coverage in the same query batch | implemented_pending_validation | Combined Runtime206 focused set now passes `17/17`; managed Cargo remains pending. |

## 批量验证状态

The focused Runtime206 source-contract batch passes `17/17` in one invocation
after the referencer change. Rustfmt parse-only, Python compilation, and
scoped whitespace/diff checks pass locally. The final comprehensive
non-tooling Runtime/Editor loader covers `551` files and passes `2052/2052`
tests in `80.282s` under the current local load; tooling production changes
remain out of scope.

No new coordinator request or status query was issued. The managed Windows
Cargo/Release gate remains deferred under the existing external dirty-worktree
admission boundary.

The current shared non-tooling loader covers `556` modules and passes
`2068/2068` tests in `29.116s`; the focused Runtime206/Runtime85 plus Editor
reference batch passes `37/37` in `111.865s`. The all-contract discovery's
only failure is the deferred WOC dependency assertion.

## 性能边界

Referencer ordering no longer formats UUIDs or allocates display keys in the
query path. For `R` referencers, sort-key construction changes from at least
`R` owned strings to zero; comparison work remains `O(R log R)`. This is a
deterministic allocation/work model, not a product latency or RSS claim.

## 源码指纹

| File | SHA-256 |
| --- | --- |
| `zircon_runtime_interface/src/resource/asset_uuid.rs` | `03FBB77678F8AB4DF0C3BB9D8B27DB09C79C865D78A267DD535611BE73430B73` |
| `zircon_runtime/src/asset/registry/query.rs` | `0A867CFBE7029C1BEB92A2A09D337C04E628230E33D15E8579115ECB8A3E7FDF` |
| `zircon_runtime/src/asset/registry/asset_registry_index/secondary_query_tests.rs` | `B422F76B4A9EDE3F5AE307DB827979B3AB764D56C5F16D72460A093E3E823B51` |
| `tools/tests/test_runtime206_referencer_binary_sort_performance_contract.py` | `3D074E2C704F85B8F36163B2CA49AF83FC1F95398C5F95B809FFE3C4D56E5F5D` |

## 未闭合项

P1-047 remains `implemented_pending_validation` until managed Windows Cargo
tests and the next Release allocation/latency batch pass. Parent-plan work on
compiled queries, cursor/visitor early-stop, generation leases, and unified
availability dispositions is unchanged.
