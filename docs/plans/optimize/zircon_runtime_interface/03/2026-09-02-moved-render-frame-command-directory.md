record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-02
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/surface/render/frame_extract.rs
  - zircon_runtime_interface/src/ui/surface/render/frame_extract/construction.rs
related_tests:
  - zircon_runtime_interface/src/ui/surface/render/frame_extract/construction_performance_tests.rs
  - tools/tests/test_runtime_interface03_frame_command_directory_performance_contract.py
  - zircon_runtime_interface/src/ui/surface/render/frame_extract/construction_performance_tests.rs::runtime_interface03_batch70_71_moved_directory_release_benchmark
---

# Moved render-frame command directory

## Scope

`UiRenderFrameCommands::from_slice` previously grouped each owned leaf level with slice `chunks`
and copied the `Arc` children into every parent directory. Each level therefore performed an
atomic reference-count increment and matching decrement per child even though the input vector was
discarded immediately. The construction module now consumes each node vector, moves the existing
`Arc` values directly into exactly sized parent vectors, and preserves directory order, depth, and
node counts.

The construction and deserialization responsibility was extracted from the 978-line frame module
into `frame_extract/construction.rs`; the orchestration, query, patch, and iteration module is now
939 lines.

## Verification

- TDD RED: the focused contract failed because the construction module and moved-directory helper
  did not exist.
- Focused Batch70-71 static performance contracts after implementation: `4/4` passed.
- Complete RuntimeInterface03 static performance-contract discovery after implementation: `150/150`
  passed.
- The Rust behavior oracle compares the old cloning grouping with the moved grouping across more
  than two directory parents and verifies the directory count.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number, integration, push, or WeCom claim exists yet.
- Initial batched request `runtime-interface03-batch70-71-20260902-r1` was rejected by the
  coordinator's command policy because `--jobs 1` attempted to override coordinator-owned Cargo
  parallelism; it did not create a ticket or execute Cargo.
- Corrected request `runtime-interface03-batch70-71-20260902-r2` was then rejected before ticket
  creation or Cargo execution by `validation_ticket_external_worktree_dirty` for external worktree
  `E:\\Git\\zr_vm`; it produced no compile, behavior, benchmark, integration, push, or performance
  receipt.

## Performance contract

The ignored release benchmark promotes 65,536 leaf nodes over 11 alternating samples. It compares
the former `chunks` plus `to_vec` path with owned-node movement and requires at least 20% P95
improvement. Terminal nanosecond values must come from the managed Windows receipt.
