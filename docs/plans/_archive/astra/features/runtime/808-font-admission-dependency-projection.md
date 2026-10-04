---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/11b-runtime-text-font-shaping-layout-editing-ime-review.md
  - docs/plans/optimize/zircon_runtime/11a/2026-09-19-font-admission-dependency-projection.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
  - docs/plans/astra/features/runtime/721-host-font-borrowed-dedup.md
implementation_files:
  - zircon_runtime/src/dynamic_api/session/runtime_ui/font_admission.rs
tests:
  - tools/tests/test_runtime_font_admission_dependency_projection_performance_contract.py
---

# Runtime808 · font-admission dependency projection

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime11A/11B cross-surface font dependency merge | Replace the temporary tree-backed dependency set with sorted/deduplicated contiguous storage while preserving lexical order, claim replacement, and admission semantics. | TDD source/model contract `4/4`; the combined eight-slice Runtime/Editor batch is `32/32`; the current one-process non-tooling loader passes `3693/3693` across `877` files in `83.085s` with zero failures, errors, or skips; lower source regression and ignored `RUNTIME808_FONT_ADMISSION_DEPENDENCY_BUFFER_BENCH_V1` marker are wired; deterministic 4,096-entry model changes per-entry tree nodes to one contiguous projection buffer. Managed Windows/Cargo/Release and font-admission product percentile evidence remain pending. | implemented_pending_validation |

## Complexity boundary

This slice changes only the transient cross-surface dependency collection. It
does not alter font asset identity, claim lifetime, prepared admission order,
database generation, failure classification, resident teardown, or profile
counter semantics.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/dynamic_api/session/runtime_ui/font_admission.rs` | `2BE09CE110B94CEC9B66CE512B5C6C2C979304F1FAD6A126AFB416829BF88CFC` |
| `tools/tests/test_runtime_font_admission_dependency_projection_performance_contract.py` | `AEBAD207708166D8D4B015629A5495E003B2B51C31DEDC1F3F0154384D981B6E` |

## Managed gate

No Cargo process is started locally and the external coordinator is not
polled. Local source/model evidence is retained for the merged validation
batch; tooling production remains deferred for the later Rust migration.
