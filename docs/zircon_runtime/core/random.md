---
related_code:
  - zircon_runtime/crates/zr_contracts/src/random/mod.rs
  - zircon_runtime/crates/zr_contracts/src/random/algorithm.rs
  - zircon_runtime/crates/zr_contracts/src/random/assembly.rs
  - zircon_runtime/crates/zr_contracts/src/random/key.rs
  - zircon_runtime/crates/zr_contracts/src/random/state.rs
  - zircon_runtime/crates/zr_contracts/src/random/service_state.rs
  - zircon_runtime/src/core/framework/random/mod.rs
  - zircon_runtime/src/core/runtime/random/mod.rs
  - zircon_runtime/src/core/runtime/random/service.rs
  - zircon_runtime/src/core/runtime/random/stream.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/random.rs
  - zircon_runtime/src/core/runtime/state/core_runtime_state.rs
implementation_files:
  - zircon_runtime/src/core/runtime/random/mod.rs
  - zircon_runtime/src/core/runtime/random/service.rs
  - zircon_runtime/src/core/runtime/random/stream.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/random.rs
  - zircon_runtime/src/core/runtime/state/core_runtime_state.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
  - docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Math/RandomStream.h
  - dev/godot/core/math/random_pcg.h
  - dev/godot/core/math/random_number_generator.h
tests:
  - zircon_runtime/crates/zr_contracts/src/random/tests.rs
  - zircon_runtime/src/core/runtime/random/tests.rs
  - zircon_runtime/src/core/runtime/handle/random.rs::tests
  - tools/tests/test_frameworks_01_random_contract_kernel_boundary.py
doc_type: module-detail
---

# Core Random Contracts and Runtime

Zircon separates deterministic-random persistence contracts from generator execution. Frameworks01
M1 gives those contracts their first physical low-dependency owner in `zr_contracts`; the Runtime
random kernel remains the executable owner and is a precursor to the later `zr_kernel` cut.

## Ownership Boundary

`zr_contracts::random` is the canonical owner of cross-domain contracts:

- `RandomAlgorithmId`, fail-closed stable-ID decoding, and the validated 63-bit
  `RandomSequenceId` used by PCG32;
- stable World/entity/system/purpose stream keys;
- serializable `RandomState` and `RandomServiceState` snapshots;
- immutable `RandomSeedReceipt` data and invariant-preserving DTO accessors.

`zircon_runtime::core::framework::random` is an explicit product projection of that list. It has no
child implementation modules, wildcard re-export, copied DTO, or compatibility implementation.
The projection deliberately omits `zr_contracts::random::assembly`, which is a hidden workspace
construction boundary used only by the Runtime kernel after it has established the relevant
invariants.

`zircon_runtime::core::runtime::random` owns executable behavior:

- `RandomService` master-seed authority and versioned BLAKE3 stream derivation;
- `RandomStream` PCG32 XSH-RR state advancement, bounded rejection sampling, and unit-float draws;
- typed seed-generation/draw-index exhaustion and restoration of a runtime stream from a validated
  contract snapshot.

The old `framework::random::{RandomService, RandomStream, RandomStreamError}` surface does not
exist. Runtime kernel owners import canonical DTOs directly from `zr_contracts::random`; product
consumers may use the curated `core::framework::random` projection. There is no forwarding alias or
copied implementation.

## Determinism Contract

A stream derivation includes the algorithm stable ID, master seed, master-seed generation, World
identity/generation, optional entity identity/generation, compiled-system key, purpose key, and
authoring seed. It deliberately excludes wall time, frame completion order, pointers, and thread
identity. Snapshots store the algorithm, generator state, odd PCG increment, and draw index.

Changing the behavior of an existing `RandomAlgorithmId` is forbidden. A behavior change requires a
new stable algorithm variant plus migration and replay qualification. Unknown stable IDs and even
PCG increments fail during deserialization instead of being interpreted as the current algorithm.
PCG32 sequence IDs are limited to 63 bits because the increment's low bit is reserved. BLAKE3
derivation reduces a uniform word to those 63 bits before stream construction; external or persisted
out-of-range values are rejected rather than silently folded.

`RandomStream` is deliberately neither `Clone` nor `Copy`, so assignment or helper argument passing
cannot duplicate mutable draw position silently. `RandomState` snapshot/restore remains an explicit
state-reconstruction boundary for checkpoints and tests; it is not the engine's final fork policy.
Calling `RandomService::stream` with the same key can still reconstruct the same initial position.
Runtime22 must define which stable-key derivations are admitted and register them before Zircon can
claim an authoritative fork contract.

`RandomService::reseed` is a fallible transaction. It computes the successor generation before
changing the seed, returns `RandomServiceError::SeedGenerationExhausted` at `u64::MAX`, and preserves
the complete pre-call service state on rejection.

`RandomService` is deliberately neither `Clone` nor `Copy`. `CoreHandle` and `CoreRuntime` expose the
runtime-owned authority only as `&RandomService`, and service queries plus stream derivation borrow
that authority. `CoreRuntimeInner` keeps the service field private and owns runtime construction, so
crate-internal callers cannot recover a mutable service through `Arc::get_mut` and reseed the live
authority directly. Constructing an independent mutable service requires an explicit
`RandomServiceState` snapshot/restore operation; Runtime has no mutable or by-value service accessor.
The owner guard scans every product Rust candidate that names `RandomService`; only the Runtime,
Handle, and `CoreRuntimeInner` owner files may expose their single shared-borrow accessor, and no
product module may add `Clone`, `Copy`, by-value, or mutable authority access. Renaming the
authority through a Rust `type` alias or `use ... as ...` is forbidden, including an alias defined
in one file and consumed from another.

## Current Limits

The physical source move and consumer migration are present, but workspace integration is not yet
complete. The shared root `Cargo.toml`, `Cargo.lock`, and `zircon_runtime/Cargo.toml` remain owned by
the active Shader06 session. Frameworks01 therefore has not edited them: `zr_contracts` is not yet a
workspace member or Runtime dependency, and managed Rust validation is not admissible until those
three exact blobs can be transferred or committed by their current owner. The focused static guard
currently passes 13/14 tests; its only failure is the intentionally unmet manifest wiring assertion.
This is an integration gate, not a substitute for product validation.

This slice establishes owner direction and closes master-seed generation exhaustion plus explicit
63-bit PCG sequence identity; it does not complete Runtime22 replay architecture. The current source
still needs an authoritative stream registry, stable-key fork policy, checkpoint/replay integration,
and CPU/GPU parity. The existing data-structure probe does not authorize a cache, hash replacement,
or product-throughput claim from this document.

The master-seed authority copy boundary is closed, but Runtime22 must still define stable-key
admission, registry ownership, replay checkpoints, and any explicit fork capability before the
engine can claim a complete deterministic-random product lifecycle. The existing local release
probe measured BLAKE3 stream derivation at roughly 426 ns per stream and contiguous PCG32 draws at
roughly 1.66 ns per draw on the measured Ryzen 7 5800H. This only identifies repeated derivation as
the architectural optimization entry; it is not product-frame, allocation, energy, replay, or
cross-engine parity evidence and does not authorize a local cache outside Runtime22.

The previous Windows managed product validation job `bffbcc35cb1e48fe98e46697df13bd81` compiled `zr_math`,
`zircon_runtime_interface`, and `zr_rhi`, then stopped in the foreign `zr_rhi_wgpu` crate on 14
current-source errors before compiling `zircon_runtime`. Static owner guards and formatting passing
therefore do not establish product startup, performance, energy use, parity with another engine, or
algorithmic optimality.
