---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/206/2026-08-31-asset-type-posting-index.md
  - docs/plans/optimize/zircon_runtime/04/2026-09-09-manifest-bound-and-root-admission.md
related_records:
  - docs/plans/astra/features/runtime/695-20260911-runtime-query-registry-builtin-diagnostics-optimization.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
related_code:
  - zircon_runtime/src/asset/registry/query.rs
  - zircon_runtime/src/asset/project/package_asset_registry.rs
tests:
  - tools/tests/test_runtime206_asset_type_posting_index_performance_contract.py
  - tools/tests/test_runtime85_project_root_dedup_performance_contract.py
---

# Runtime Asset Contract Robustness Repair

## 计划完成列表

| Batch | Work | Status | Validation evidence |
| --- | --- | --- | --- |
| Runtime206 / type posting | Make the composite type-posting source guard whitespace-stable while retaining the `AssetKind` posting lookup assertion | implemented_pending_validation | Focused Runtime206 contract `4/4` passes; existing posting behavior and deterministic candidate model are unchanged. |
| Runtime85 / project-root dedup | Make the collection-import source guard insensitive to rustfmt import ordering while retaining all required collection types and hash-index assertions | implemented_pending_validation | Focused Runtime85 contract `3/3` passes; root ordering, duplicate precedence, and capacity assertions are unchanged. |

## 批量验证

The comprehensive Runtime/Editor performance-contract and pressure loader covers
548 files and passes `2039/2039` tests in `22.986s`. The six-file focused
Runtime/Editor optimization contract batch passes `18/18`; scoped Rustfmt,
`py_compile`, and `git diff --check` pass. Tooling contracts remain outside
this repair by task direction. Including the two repaired Runtime contracts, the
final eight-file focused set passes `25/25`.

The current focused Runtime206/Runtime85 plus Editor reference invocation
passes `37/37` in `111.865s`; the shared non-tooling performance-plus-pressure
loader covers `556` modules and passes `2068/2068` tests in `29.116s`. The later
all-contract discovery has only the deferred WOC dependency failure.

## 当前源码指纹

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/asset/registry/query.rs` | `16F5AEAE366132941814DB8DF1BA27E1D96A2C5C1E9BBE62B89C0BB1659F8535` |
| `zircon_runtime/src/asset/project/package_asset_registry.rs` | `AD77AEC95D97DCE1947BAD66FD2F83BA7EB9787428A307DAE134D29150674AB5` |
| `tools/tests/test_runtime206_asset_type_posting_index_performance_contract.py` | `A5CB9CFBC0437CBEDFC4E9AA4E769F38140B85AD06EE230CF703BBD9A41C4E57` |
| `tools/tests/test_runtime85_project_root_dedup_performance_contract.py` | `EC8556FF07E5715A534E9CA2F5EAD14D009583D5C1D3DECFE7E70462C160AE7C` |

The source table is the Runtime743 checkpoint; Runtime744–746 subsequently
extended `query.rs`, so their later records carry the current query-source
fingerprint.

## 受管验证边界

These are source-contract repairs for already implemented Runtime optimizations;
they do not claim managed Cargo compilation, Windows Release allocation, or
product p50/p95/p99 measurements. Those gates remain pending under the existing
external `E:\Git\zr_vm` dirty-worktree and overlay admission boundary.
