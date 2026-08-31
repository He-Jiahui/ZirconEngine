---
handoff_kind: failure
status: open
created_at: 2026-08-25
summary_slug: random-runtime-handle-gateway-ownership
origin_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
fixing_plan: docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/01
fixing_child_dir: docs/plans/optimize/zircon_runtime/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/runtime/handle/mod.rs
  - zircon_runtime/src/core/runtime/handle/random.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/state/core_runtime_state.rs
  - zircon_runtime/src/core/runtime/random/mod.rs
tests:
  - tools/tests/test_frameworks_01_random_contract_kernel_boundary.py
  - cargo check -p zircon_runtime --lib --locked
---

# Runtime01: RandomService handle gateway has no executable current-source owner

## Source executor

- Origin plan: `docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
- Origin slice: deterministic random contract/kernel owner hard cut
- Fixing plan: `docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md`
- Routing reason: `core/runtime/handle/mod.rs` is a Runtime lifecycle gateway owned by an archived
  Runtime01 Session and is outside Frameworks01 r12's immutable scope.

## Current evidence

At HEAD `0fd7df4ecdd157f9505cd51013780e3225cfb83c`, coordinator baseline epoch 436:

- `core/runtime/handle/mod.rs` SHA-256 is
  `4cfb5ceec8e2bdaa7490714e690a2b23b854a6ecba4627c0212ea475ae04bc1c`;
- `core/runtime/handle/random.rs` SHA-256 is
  `64a6f846854ded9baed91ea29f45c7de3b4dff09d6a634fe3d30697855ba8f3d`;
- `core/runtime/runtime.rs` SHA-256 is
  `32f1e225e54d5ed236d3b7572c8a29aafc8a74c2703024d025bf8dfe34888221`;
- `core/runtime/state/core_runtime_state.rs` SHA-256 is
  `da89f936fb8f5c160df5d8c8e2283ef7b8afb2732979356e6b2875af40531129`.

The Random contract/kernel implementation itself has a current executable owner. Transfer preview
request `1102cbf209764560b1c72abde14cf91a` proved 10/10 cancelled-r10 blobs unchanged, and apply request
`9b9931b9b0664362837f44c73b8cd044` transferred fingerprint
`ce98561b713ab6784d030145e8e9ad7d01731a4898eca55ad8417bbecbd0161c` to Frameworks01 r12 without
rewriting source. The current owner guard is 13/13 GREEN in 30.915 seconds.

`core/runtime/handle/mod.rs` contains the required single `mod random;` declaration. Its whole blob
is still attributed to archived Session `runtime-core-lifecycle-m0-veto-atomicity-20260815`, whose
plan family is Runtime01. The coordinator reports a stale content hash, stale baseline, no live
lease, and an owner that cannot execute. Frameworks01 r12 does not include this path in its immutable
write scope and therefore does not edit or claim it.

The neighboring Runtime construction blobs cannot be treated as substitutes. `runtime.rs` and
`core_runtime_state.rs` combine RandomService construction with Time, State, and module-lifecycle
changes. Moving only the `mod random;` line elsewhere, adding a forwarding module, or exposing the
implementation through a compatibility facade would break the approved hard-cut topology.

## Required closure

- Register or rotate an executable Runtime01 Session whose immutable scope includes the complete
  current `core/runtime/handle/mod.rs` blob and the focused Runtime handle tests.
- Re-review the current blob against Runtime01 lifecycle/module-boundary rules. Preserve exactly one
  private Random handle module declaration and do not add root or framework implementation exports.
- Confirm that `CoreHandle::random_service`, `CoreRuntime::random_service`, and
  `CoreRuntimeInner::random_service` remain the only three shared-borrow authority accessors, with
  the backing field private and no `Clone`, `Copy`, value return, or mutable return escape.
- Run the 13-test Frameworks01 Random owner guard and a coordinator-managed Windows
  `cargo check -p zircon_runtime --lib --locked`. If a foreign lower-layer compile error intervenes,
  return its exact current-source fingerprint rather than changing another owner.
- Return a canonical fixed record to Frameworks01 with current hashes and validation receipts.

## Forbidden shortcuts

- Do not move `handle/random.rs` back into framework contracts or duplicate its accessor in another
  gateway.
- Do not use `pub use`, an alias module, shim, feature-gated legacy path, or compatibility wrapper.
- Do not submit the transferred Random implementation without the current gateway blob and managed
  Runtime validation.
- Do not claim BLAKE3 performance, power, replay, registry, CPU/GPU parity, or algorithm optimality;
  those remain Runtime22 work with separate profiling entry gates.

## Return state

Open. Root-cause and owner routing are complete. Runtime01 scope rotation, implementation-owner
review, managed validation, fixed return, Frameworks01 milestone acceptance, commit, and WeCom
synchronization remain pending.
