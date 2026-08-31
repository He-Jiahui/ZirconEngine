---
related_code:
  - zircon_runtime/src/ui/surface/frame_hit_test.rs
  - zircon_runtime/src/ui/surface/frame_hit_test/tests.rs
implementation_files:
  - zircon_runtime/src/ui/surface/frame_hit_test.rs
  - zircon_runtime/src/ui/surface/frame_hit_test/tests.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
tests:
  - rustfmt --edition 2021 --check zircon_runtime/src/ui/surface/frame_hit_test.rs zircon_runtime/src/ui/surface/frame_hit_test/tests.rs
  - scoped source ownership and post-module production-retention scan
  - scoped production-file budget mirror scan
  - git diff --check
doc_type: milestone-detail
---

# Runtime 15 frame hit-test owner split

## Status and completed items

| Milestone | Slice | Status | Date | Evidence |
|---|---|---|---|---|
| M3/M4 | Frame hit-test folder-backed test owner split | `runtime_15_frame_hit_test_owner_split_source_complete_static_passed_managed_validation_deferred` | 2026-08-27 | Mixed owner 1066 lines -> production owner 757 lines plus test owner 308 lines; 6/6 tests and 2/2 post-test public debug functions retained. |

Completed:

- Replaced the middle-of-file `projected_grid_tests` module with an explicit folder-backed path mount.
- Moved all six projected-grid incremental patch, affine projection, rebuild, clip refresh, and popup ordering tests into `frame_hit_test/tests.rs`.
- Updated the moved source guard to read `../frame_hit_test.rs`, preserving its original target after relocation.
- Kept both public debug hit-test functions and all following rejection diagnostics in the production owner.
- Removed this owner from the Runtime 15 production-file over-budget set: the production owner is now 757 lines, below the 800-line limit.

## Review basis

This is a mechanical ownership correction required by `engine-code-structure-convention.md`. Projected hit-test state, patch/rebuild policy, ordering, cell mapping, public debug entry points, and rejection behavior remain in the production feature owner. No compatibility module, alias, re-export, duplicate implementation, algorithm replacement, or profiling change was introduced.

## Verification

- Scoped `rustfmt --edition 2021 --check` passed for both Rust owners.
- Static ownership scan reported root test attributes `0`, child test attributes `6`, explicit child mounts `1`, corrected parent-source includes `1`, and retained post-module public debug functions `2`.
- The production-file budget mirror scan reports none of the three owners closed in this session as oversized; 49 unrelated current production files remain at or above 800 lines, so the global Runtime 15 budget remains open.
- Scoped `git diff --check` passed apart from the repository checkout's LF/CRLF notices.
- Managed Cargo validation was not run while bypassing the current validation blocker. Compile and focused behavior tests remain required before accepted milestone closeout.
- No CPU, GPU, energy, or power improvement is claimed because this slice does not change the hit-test algorithm.

## Open scope

Runtime 09, Runtime 15, and the full runtime architecture remain `in_progress`. This record closes only the source ownership implementation for projected frame hit-test tests. Managed compile/test, the remaining production-file budget, milestone commit, coordinator integration receipt, and WeCom publication remain open.
