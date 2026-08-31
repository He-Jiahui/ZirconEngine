# Frameworks01 M1 state-machine kernel-owner hard cut (2026-08-24)

## Status

- `source_implemented`
- `old_owner_deleted`
- `static_owner_guard_green_5_of_5`
- `contracts_kernel_guard_green_3_of_3`
- `runtime_product_gate_blocked_by_foreign_rhi_errors`
- `milestone_not_accepted`

This record is an M1 declaration/behavior partition slice. It does not accept M1 or claim a Runtime
product GREEN.

## Current-Source Review

At implementation-start HEAD `79f64878f3b9526517644c055ad3bf5cadfccd0f`,
`zircon_runtime/src/core/framework/state` contains 12 Rust files, 519 lines, and 14,817 bytes. The
path-sorted `path<TAB>file-sha256<TAB>bytes` manifest SHA-256 is
`c084cd29e09c339bdf57407b57f5ad93c393c4d4f6272be194b1e349f301f1db`.

The folder is not a pure contract owner. It contains `StateMachine`, `StateRegistry`, hash-indexed
hook storage, transition recording, dispatch ordering, and deferred hook execution. Current tracked
consumers are confined to the runtime kernel, its public `core`/prelude projection, and Runtime
state tests. No App, Editor, or plugin production source imports the old state path directly.

## Reference-Engine Decision

- Unreal keeps StateTree as a Runtime plugin module that depends on Core/Engine and owns execution,
  transition, tracing, and validation behavior. It is not placed in a generic declaration-only
  contract layer.
- Bevy keeps state values, pending transitions, transition schedules/events, conditions, and state
  execution in the independent `bevy_state` implementation crate. The public types and the
  transition machinery share an implementation owner rather than depending on an application
  facade.
- Fyrox has no closer standalone foundation contract owner that would justify moving executable
  transition machinery into Zircon's `zr_contracts`.

Zircon does not need another `zr_state` crate for the current MVP surface. The state machine is used
only by the runtime kernel, so its canonical owner is `core/runtime/state_machine` now and
`zr_kernel` after the M1 physical kernel cut.

## Locked Hard-Cut Shape

1. Move the complete 12-file state folder to `zircon_runtime/src/core/runtime/state_machine`.
2. Publish the seven existing product types from `core::runtime::state_machine` and keep
   `StateRegistry`, `StateMachine`, hook storage, and dispatch internals crate-private.
3. Update runtime, core/prelude projections, and tests to the new owner in the same change.
4. Delete `core/framework/state` and its module declaration. Do not leave a forwarding module,
   `pub use` compatibility bridge, alias, copied implementation, or old-path test fixture.
5. Add a static hard-cut guard that proves the old owner is absent, all tracked Rust consumers use
   the kernel owner, and the Runtime public projections remain explicit.

## Validation Boundary

Required focused evidence is the static owner guard, its mutation controls, scoped formatting, and
diff-check. A coordinator-managed Runtime product build remains required for acceptance, but is not
repeated while the four unchanged foreign `zr_rhi_wgpu` hashes continue to block compilation. This
slice makes no runtime latency, allocation, throughput, energy, bottleneck-removal, parity, or
optimality claim because it changes ownership only and has no same-fingerprint profile sample.

## Implementation Result

- All 12 files and 519 lines now live under `core/runtime/state_machine`; the new path-sorted
  manifest SHA-256 is `f3154a973b8adc041a4e8a95d6c4b45c46afa9519589f9552bc7961d20d24b56`.
  File contents were moved mechanically; transition, hook-index, event-order, and dispatch algorithms
  were not rewritten.
- `core/framework/state` no longer exists. Runtime, `core`/prelude projections, and state tests use
  `core::runtime::state_machine`; no forwarding module, alias, wildcard projection, or copied owner
  remains.
- TDD owner guard RED reported the old directory plus 10 old-path consumer lines. A chained module
  alias mutation then produced a second focused RED before alias-graph normalization was added.
  Final owner guard is `5/5` GREEN; the combined partition/contracts/state suite is `13/13` GREEN in
  25.392 seconds and covers grouped use, item alias, chained module alias, multiline-qualified path,
  and comment/literal negative controls. The existing contracts-to-kernel guard remains `3/3` GREEN.
- Exact Rust formatting with `skip_children=true` and scoped `git diff --check` are GREEN. The first
  read-only rustfmt invocation followed root modules into foreign animation/render children and
  reported their pre-existing format drift; it wrote no files and is not acceptance evidence.
- Windows-native standalone `rustc --edition 2021 --crate-type lib` for the complete moved module is
  GREEN. The D-drive rlib is 90,042 bytes with SHA-256
  `d8e562036c32b30d6c0d5925a0e981c1f4dd62b47b38118fb5b1a95263b7ecf8`; its 11 warnings are expected
  dead-code/unused-import warnings because the isolated crate root has no `CoreRuntime` consumer.
- The reproducible partition audit now reports 595 Framework production files, 52,108 nonempty code
  lines, 3,579 function bodies, 2,532 product-public function bodies, 144 restricted-visibility
  function bodies, and 47 public traits. File classes are 204 declaration-only, 369 mixed, and 22
  behavior-only. The D-drive report SHA-256 is
  `a6b9712bc02c32f3d0c0b394c5b8276cf786c322ddf2b68d0b16f1fa66bd514d`.
- No Cargo command was launched. The four Runtime90 `zr_rhi_wgpu` blocker hashes are unchanged, so a
  repeated product build would reproduce the same foreign stop rather than validate this slice.
- Coordinator attribution marks all 12 new kernel-owner files `integration_ready`. The 12 old
  paths hold exact live deletion leases and remain `deletion_requires_explicit_candidate`; a future
  service candidate must list them explicitly. No candidate, commit, or WeCom notification was
  created because there is no passed managed validation ticket matching this current-source
  manifest.

This slice remains `source_implemented / static_green / managed_rust_gate_pending`; it is not a
milestone acceptance or commit candidate by itself.
