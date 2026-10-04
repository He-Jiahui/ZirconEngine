---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-31-editor581-asset-projection-full-reuse.md
related_records:
  - docs/plans/astra/features/editor/945-editor581-badge-overlay-clip-early-exit.md
  - docs/plans/astra/features/runtime/888-runtime581-visible-spatial-hash-index.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/workbench/snapshot/asset/asset_workspace_item_generation.rs
tests:
  - zircon_editor/src/ui/workbench/snapshot/asset/asset_workspace_item_generation.rs
---

# Editor946 Editor581 Asset Projection Full Reuse

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Unchanged asset projection | When every source chunk is unchanged, the projection shares both the chunk array and selected-index array through `Arc`; partial changes retain the existing chunk-level reuse path. | Behavior/source contracts verify pointer sharing for the full-reuse path and preserve changed-chunk behavior. |
| 性能门禁 | 8,192 unchanged items avoid allocating/copying a new chunk array and selected-index array. | ignored marker `EDITOR581_PROJECT_REUSE_BENCH_V1` requires optimized P95 ≤70% of the legacy copy path; managed Editor Release receipt remains pending. |

- `asset_workspace_item_generation.rs` contains the Editor581 behavior/source contracts and marker.
- No tooling changes; this record preserves the managed Release gate and does not infer performance from local inspection.
- Rustfmt-only wrapping in the replacement-capacity source contract was repaired locally; exact-file Rustfmt now passes.

### Grouped validation submission (2026-09-25)

Editor581 is included in the shared `optimization_batch_gz` wave: Editor
development PTY `36565` and Editor Release PTY `35753`, paired with Runtime
development PTY `76539` and Runtime02 Release PTY `51493`. The wrappers remain
intentionally unpolled; compiler, functional-test, and P95 receipts are pending.

The exact `optimization_batch_gz` replacement wave was submitted as Runtime
development PTY `81972`, Editor development PTY `21257`, Runtime02 Release PTY
`20043`, and Editor Release PTY `50976`. The earlier broad-prefix wave did not
select this historical marker; all four replacement wrappers remain unpolled.
