---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/08c/2026-09-21-skeleton-path-borrowed-segments.md
related_records:
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/animation/manager/pose.rs
tests:
  - zircon_runtime/src/animation/manager/pose/borrowed_path_tests.rs
  - tools/tests/test_runtime877_skeleton_path_borrowed_segments_performance_contract.py
---

# Runtime877 Skeleton Path Borrowed Segments

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime08c animation skeleton path | Keep borrowed ancestor names in the existing ordered collection instead of cloning each name, preserving path and missing-parent behavior. | Combined RED `1/8` -> GREEN `8/8`; adjacent `48/48`. For 4,096 depth-32 paths the deterministic model removes `131072` name clones. Lower parity tests and ignored 101-pair `RUNTIME877_SKELETON_PATH_BORROWED_SEGMENTS_BENCH_V1` are wired. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/animation/manager/pose.rs` | `152BD4CE4E4F8F1DB28C888F605E8FFE114E5660ED5C3B68BD54ED8E57758D12` |
| `zircon_runtime/src/animation/manager/pose/borrowed_path_tests.rs` | `FCE9AF07C9299D3C9258EB1C2F871BD0FC05DB3B515968B4649EE4572136AC77` |
| `tools/tests/test_runtime877_skeleton_path_borrowed_segments_performance_contract.py` | `D8E31CC8CEE98FB8F5B5BA3EB5D160599FA1548C97FB3CE1015A73E556602322` |

## Managed gate

Runtime877 was submitted with Editor896 in combined current-source v24 (PID
`33516`) at `2026-09-21T23:12:59.6906963+08:00`. One bounded receipt read
after independent documentation and tests confirms managed Runtime development
compilation exited `0`; Editor/App admissions failed because the reuse pool
was busy, not because of an attributed Rust error. Rust lower/ignored Release
tests, allocator measurement, and animation-product p50/p95/p99 remain pending.
