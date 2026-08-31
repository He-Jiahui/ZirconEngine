---
handoff_kind: failure
status: open
failure_scope: local
created_at: 2026-08-28
summary_slug: rhi-lib-test-self-import
origin_plan: docs/plans/optimize/zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md
fixing_plan: docs/plans/optimize/zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md
origin_child_dir: docs/plans/optimize/zircon_runtime/90
fixing_child_dir: docs/plans/optimize/zircon_runtime/90
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/crates/zr_rhi/src/tests/boundary.rs
  - zircon_runtime/crates/zr_rhi/src/tests/device_fault.rs
  - zircon_runtime/crates/zr_rhi/src/tests/device_profile.rs
  - zircon_runtime/crates/zr_rhi/src/tests/handles.rs
tests:
  - .\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zr_rhi -SkipBuild -LibTests -TestFilter surface_handle -VerboseOutput
  - .\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zr_rhi -SkipBuild -LibTests -VerboseOutput
---

# Runtime90: RHI unit tests import their own crate as an external dependency

## Source and reproduction

Runtime90's managed `zr_rhi` surface-handle gate compiled the library test target on
2026-08-28 and failed before executing a test. Rust reported E0432 in `boundary.rs`,
`device_fault.rs`, `device_profile.rs`, and `handles.rs`: each unit-test child used
`use zr_rhi::...`, but a crate's `src/tests` modules must consume the current crate
through `crate::...`.

## Lowest shared cause

The four tests were written with integration-test import syntax even though `lib.rs`
mounts them under `#[cfg(test)] mod tests`. Adding `extern crate self as zr_rhi` would
hide that ownership error behind a permanent alias; the correct boundary is direct
crate-relative imports in the unit-test tree.

## Acceptance

- Replace only the four invalid self-imports with `crate::...`; retain every test body.
- Do not add a self alias, compatibility facade, feature bypass, or integration-test copy.
- The managed surface-handle filter must compile the complete lib-test target and run its
  selected tests successfully.
- The full managed `zr_rhi --lib` suite must pass before this record returns fixed.

## Result

The four unit-test modules now import through `crate::...`; no external self alias was
added. Managed job `18086ba7d85f496b8dda823e9e1be17a` compiled the complete lib-test
target and passed the selected surface-handle tests. Managed job
`b5522b23945e4c70837b8dacb18b145c` passed the complete 78-test `zr_rhi --lib`
suite. Both jobs released with exit code 0.

Open state: `repair and managed validation green / Runtime90 atomic integration pending`.
