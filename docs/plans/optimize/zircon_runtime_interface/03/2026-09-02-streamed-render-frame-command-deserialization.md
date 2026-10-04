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
  - tools/tests/test_runtime_interface03_frame_command_deserialization_performance_contract.py
  - zircon_runtime_interface/src/ui/surface/render/frame_extract/construction_performance_tests.rs::runtime_interface03_batch70_71_streamed_deserialization_release_benchmark
---

# Streamed render-frame command deserialization

## Scope

Deserialization previously decoded every `UiRenderCommand` into one temporary flat `Vec`, then
called `from_slice`, cloning every command and owned payload again into persistent leaf segments.
The custom serde visitor now decodes directly into final 64-command leaves and builds the directory
from those owned segments. Exact sequence hints reserve final capacities; unknown hints retain the
bounded segment capacity. The command-range index is rebuilt from the final retained iterator, so
non-contiguous owner ranges keep the existing fail-closed semantics.

## Verification

- TDD RED: the focused contract failed because the old `Vec::<UiRenderCommand>::deserialize`
  staging path remained and no streamed benchmark existed.
- Focused Batch70-71 static performance contracts after implementation: `4/4` passed.
- Complete RuntimeInterface03 static performance-contract discovery after implementation: `150/150`
  passed.
- The Rust behavior oracle compares streamed decoding with the former decode-then-clone path for
  a multi-segment command sequence.
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

The ignored release benchmark decodes 1,024 commands carrying 1 KiB owned text eight times per
sample over 11 alternating samples. It compares the former temporary-Vec plus command-clone path
with direct segmented decoding and requires at least 10% P95 improvement. Terminal nanosecond values
must come from the managed Windows receipt.
