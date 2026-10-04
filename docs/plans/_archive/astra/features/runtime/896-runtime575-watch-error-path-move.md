---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/575/2026-08-31-watch-error-path-move.md
related_records:
  - docs/plans/astra/features/editor/953-editor575-model-projection-metadata.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/asset/watch/asset_watch_error.rs
tests:
  - zircon_runtime/src/asset/watch/asset_watch_error.rs
---

# Runtime896 Runtime575 Watch Error Path Move

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Watch error conversion | Format the notify error before moving its owned path vector into `AssetWatchError`, eliminating the consumed-error deep clone while preserving root, path order, and message. | Behavior/source contracts cover the complete converted payload and production move path. |
| 性能门禁 | 1,024 errors with four paths per sample avoid cloning path vectors and `PathBuf`s. | ignored marker `RUNTIME575_WATCH_ERROR_PATH_MOVE_BENCH_V1` requires optimized P95 ≤90% of legacy; managed Runtime Release receipt remains pending. |

- `asset_watch_error.rs` contains the Runtime575 behavior contract and marker.
- No tooling changes; the combined Runtime575/Editor575 release gate remains pending.

### Grouped validation submission (2026-09-25)

Runtime575 is included in the exact `optimization_batch_gt` replacement wave:
Runtime development PTY `91895`, Editor development PTY `38484`, Runtime02
Release PTY `63820`, and Editor Release PTY `41285`. The wrappers remain
intentionally unpolled; compiler, functional-test, and P95 receipts are pending.
