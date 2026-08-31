---
related_code:
  - zircon_runtime/src/core/framework/render/shader/variant_miss_report.rs
  - zircon_runtime/src/core/framework/render/shader/variant_miss_report/tests.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/shader/variant_miss_report.rs
  - zircon_runtime/src/core/framework/render/shader/variant_miss_report/tests.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
tests:
  - rustfmt --edition 2021 --check zircon_runtime/src/core/framework/render/shader/variant_miss_report.rs zircon_runtime/src/core/framework/render/shader/variant_miss_report/tests.rs
  - scoped source ownership scan
  - scoped production-file budget mirror scan
  - git diff --check
doc_type: milestone-detail
---

# Runtime 15 shader variant miss report test owner split

## Status and completed items

| Milestone | Slice | Status | Date | Evidence |
|---|---|---|---|---|
| M3/M4 | Shader variant miss report folder-backed test owner split | `runtime_15_shader_variant_miss_report_test_owner_split_source_complete_static_passed_managed_validation_deferred` | 2026-08-27 | Mixed owner 1013 lines -> production owner 549 lines plus test owner 463 lines; 11/11 tests retained. |

Completed:

- Replaced the inline `tests` module in `variant_miss_report.rs` with an explicit folder-backed path mount.
- Moved all 11 shader variant miss report tests into `variant_miss_report/tests.rs` without changing production data structures, accumulation rules, fallback diagnostics, or public APIs.
- Removed this owner from the Runtime 15 production-file over-budget set: the production owner is now 549 lines, below the 800-line limit.
- Preserved the current uncommitted fallback and pipeline-shape test coverage while keeping the change independent from renderer algorithm and profiling work.

## Review basis

This is a mechanical ownership correction required by `engine-code-structure-convention.md`: production behavior remains in the feature owner and feature-local tests move to a folder-backed test owner. No compatibility module, alias, re-export, duplicate implementation, algorithm replacement, or hotpath instrumentation was introduced.

## Verification

- Scoped `rustfmt --edition 2021 --check` passed for both Rust owners.
- Static ownership scan reported root test attributes `0`, child test attributes `11`, explicit child mounts `1`, and retained child `super::` references `4`.
- The production-file budget mirror scan no longer reports `core/framework/render/shader/variant_miss_report.rs`; it still reports 50 unrelated existing production files at or above 800 lines, so the global Runtime 15 budget remains open.
- Scoped `git diff --check` passed apart from the repository checkout's LF/CRLF notices.
- Managed Cargo validation was not run while bypassing the current validation blocker. Compile and focused behavior tests remain required before accepted milestone closeout.
- No CPU, GPU, energy, or power improvement is claimed because this slice does not change the runtime algorithm.

## Open scope

Runtime 15 and the full runtime architecture remain `in_progress`. This record closes only the source ownership implementation for the shader variant miss report. Managed compile/test, the remaining production-file budget, milestone commit, coordinator integration receipt, and WeCom publication remain open.
