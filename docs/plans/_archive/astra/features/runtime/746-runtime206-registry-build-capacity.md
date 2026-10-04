---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/206-runtime-asset-registry-project-catalog-index-persistence-rebuild-incremental-query-watch-generation-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/206/2026-09-13-registry-build-capacity-and-streamed-dependency-bootstrap.md
related_records:
  - docs/plans/astra/features/runtime/695-20260911-runtime-query-registry-builtin-diagnostics-optimization.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
  - docs/plans/astra/features/runtime/745-runtime206-referencer-binary-sort.md
related_code:
  - zircon_runtime/src/asset/registry/asset_registry_index.rs
  - zircon_runtime/src/asset/registry/asset_registry_index/build_tests.rs
tests:
  - tools/tests/test_runtime206_registry_build_capacity_performance_contract.py
---

# Runtime206 Registry Build Capacity And Streamed Dependency Bootstrap

## 计划完成列表

| Batch | Work | Status | Validation evidence |
| --- | --- | --- | --- |
| Runtime206 / bulk map reservation | Reserve the iterator lower bound for primary lookup maps before inserting entries | implemented_pending_validation | Source contract checks `size_hint` and all four reservation targets; deterministic build pressure model is green. |
| Runtime206 / streamed dependency bootstrap | Replace the all-at-once `(UUID, paths)` staging vector with UUID-key staging and one path vector per entry | implemented_pending_validation | Rust behavior regressions preserve resolved and unresolved dependency/referencer semantics; transient path staging falls from 4,194,304 to at most 4 records, and reverse-path empty-bucket pruning falls from 1,048,576 full scans to one final pass in the declared model. |

## 批量验证状态

The focused Runtime206 source-contract batch passes `17/17` in one invocation,
including the capacity, referencer binary-sort, secondary-posting, and existing
type-posting guards. Rustfmt parse-only, Python compilation, and scoped
whitespace/diff checks pass locally. The final comprehensive non-tooling
Runtime/Editor loader covers `551` files and passes `2052/2052` tests in
`80.282s` under the current local load. This slice joins the existing deferred
Runtime/Editor validation batch; no new coordinator request or status query was
issued.

The refreshed shared non-tooling loader covers `556` modules and passes
`2068/2068` tests in `29.116s`; the focused Runtime206/Runtime85 plus Editor
reference batch passes `37/37` in `111.865s`. The current all-contract
discovery leaves only the deferred WOC dependency assertion red.

## 性能边界

Bulk rebuilds avoid repeated growth of the four primary hash maps when the
iterator reports a useful lower bound. Dependency-path construction now keeps
only the UUID staging vector plus the persistent published index, rather than
an additional all-path staging copy; reverse-path empty buckets are pruned
once after the stream instead of once per UUID. The model is a deterministic
staging/work bound, not a managed CPU/RSS or product-latency result.

## 源码指纹

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/asset/registry/asset_registry_index.rs` | `8E1478D5C5C23769F241B2DE57C81806C039BFC1D9EF91B8F8A6FCF07024141E` |
| `zircon_runtime/src/asset/registry/asset_registry_index/build_tests.rs` | `A3B33D2A638224EBFEE058D9D7104566760CBBCBF8FB840CC81FBBF9AFC3B23D` |
| `tools/tests/test_runtime206_registry_build_capacity_performance_contract.py` | `C7743CCF5039028250CAF53BF6BA1DCF52EAD5F8AE6EB55174A3C471CE86F72A` |

## 未闭合项

Managed Windows Cargo/Release acceptance remains pending under the existing
external worktree admission boundary. Parent Runtime206 query and generation
contracts are unchanged.
