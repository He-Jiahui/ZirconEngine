---
doc_type: implementation-design-and-performance-plan
status: approved-for-source-implementation
implementation_status: source-complete-validation-pending
validation_status: isolated-rustc-green; independent-review-green; managed-cargo-validation-pending
source_recheck_required: false
owners:
  - Runtime22: random stream registry, lease lifecycle, checkpoint, eviction
  - Frameworks01: stable random value contracts in zr_contracts
related_code:
  - zircon_runtime/crates/zr_contracts/src/random
  - zircon_runtime/src/core/runtime/random
  - zircon_runtime/src/core/runtime/handle/random.rs
references:
  - docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md
  - docs/plans/zircon_runtime/frameworks/01/2026-08-28-m1-zr-contracts-random-physical-hard-cut.md
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Math/RandomStream.h
  - dev/godot/core/math/random_pcg.h
  - dev/godot/core/math/random_number_generator.h
  - dev/godot/tests/core/math/test_random_number_generator.cpp
---

# Runtime22 Random Stream Registry Architecture And Performance Plan

## Decision

The current PCG32 transition, odd-increment sequence construction, bounded rejection sampling,
draw accounting, and snapshot restore remain unchanged. The structural defect is
`RandomService::stream(key)`: every call derives and returns another mutable stream for the same
stable owner key. That permits divergent state authorities and pays the derivation cost repeatedly.

Runtime22 will hard-cut that API to one authoritative registry owned by `RandomService`:

```text
CoreRuntime
  -> RandomService
       -> Arc<RandomAuthority> (algorithm + master seed + generation + registry)
            -> RandomStreamRegistry (one entry per stable RandomStreamKey)
            -> acquire_stream(key) -> exclusive RandomStreamLease
                 -> lock-free contiguous PCG32 draws
                 -> release/drop commits RandomState through the same complete authority
       -> checkpoint / reseed / eviction (explicit lifecycle boundaries)
```

There is no compatibility `stream(key)` path and no consumer-local cache. A key with an active
lease rejects a second acquire. Checkpoint and reseed reject while any lease is active. Explicit
eviction rejects while its target key is leased. These failures are typed and leave authority and
stream state unchanged.

## Evidence And Deliberate Divergence

| Source | Evidence used | Zircon decision |
|---|---|---|
| Current Zircon Runtime | `RandomStreamKey` already includes world/entity generation, system, purpose and authoring seed; `RandomState` already includes algorithm, generator state, increment and draw index. | Preserve these contracts and make the missing lifetime owner explicit. |
| Unreal `FRandomStream` | Initial/current state live together in the long-lived owner object and draws mutate that local state without a central lock. | Preserve owner-local hot state, but reject Unreal's incidental value-copy and wall-time/name seeding behavior for replay authority. |
| Godot `RandomPCG` / `RandomNumberGenerator` | Seed, PCG state and stream increment are object state; state restore reproduces subsequent draws. The upstream restore-state test is the regression model. | Preserve direct state restore and contiguous draws, but add stable-key admission because independent RNG objects are insufficient for an engine replay owner. |

The divergence is intentional: neither reference engine's low-level value/object API prevents two
mutable owners from representing the same logical simulation stream. Zircon's registry adds that
engine-level invariant while retaining their important property that generator mutation is local to
the stream object.

## Contract And Ownership

- `zr_contracts::random` owns serializable `RandomStreamCheckpoint` and canonical
  `RandomServiceCheckpoint` values. Checkpoint stream entries are strictly ordered by
  `RandomStreamKey`, contain no duplicate key, and must use the service algorithm.
- `core::runtime::random::RandomService` owns the master seed and the only registry.
- `RandomAuthority` keeps algorithm, seed, seed generation and registry in one reference-counted
  lifetime unit. A lease therefore cannot outlive the service while retaining only an orphaned
  cache; it keeps the complete authority alive until its final state is committed.
- `RandomStreamRegistry` owns admission, parked stream state and lifecycle counts. It is not a
  product-facing manager and is not duplicated in plugins.
- `RandomStreamLease` is non-cloneable, owns the mutable `RandomStream` while admitted, commits on
  explicit release or `Drop`, and may move between worker threads.
- `RandomStream` remains the only PCG execution owner. Registry synchronization never enters
  `try_next_u32`, bounded draw, or float draw.
- `RandomService::snapshot` remains the seed-authority snapshot used to reproduce future unseen
  streams. `RandomService::checkpoint` is the replay boundary that also preserves admitted stream
  progress. Callers must not substitute one for the other.

## Lifecycle Semantics

1. First acquire derives one stream from the persisted authority and reserves the key atomically.
2. Release parks the exact progressed state under the same key.
3. Later acquire resumes that state; it does not derive again.
4. A concurrent acquire of the same key fails with `StreamAlreadyAcquired`.
5. Checkpoint requires zero active leases and emits canonical key order.
6. Restore recreates parked registry entries and therefore reproduces each stream's next draw.
7. Reseed requires zero active leases, advances the seed generation, and clears every old-generation
   registry entry atomically with the authority transition.
8. Eviction is explicit. Reacquiring an evicted key re-derives its generation-zero stream state;
   no automatic age/LRU eviction is allowed because wall time and access order are not replay facts.
9. One Runtime authority retains at most 65,536 registered keys under the MVP policy. Capacity
   rejects only unseen keys; parked existing keys remain resumable, and explicit scope eviction
   releases admission without inventing nondeterministic LRU behavior.

## Complexity And Performance Boundary

The existing measured evidence on Ryzen 7 5800H is approximately 426 ns per BLAKE3 derivation and
1.66 ns per contiguous PCG32 draw. Those numbers are historical evidence, not a result of this
change. They show why a mutex or hash on every draw would be structurally wrong.

Expected complexity after the hard cut:

| Operation | Target complexity | Allocation/synchronization |
|---|---|---|
| first acquire of key | `O(log N)` registry admission + one derivation | registry entry allocation; registry lock plus one seed snapshot lock |
| resumed acquire | `O(log N)` | no derivation; one mutex boundary |
| each draw while leased | `O(1)` | zero allocation; zero registry lock/hash |
| release | `O(log N)` | zero allocation; one mutex boundary |
| checkpoint | `O(N)` | one canonical output vector; rejects active leases |
| explicit eviction | `O(log N)` | zero allocation; rejects an active target |

No latency, throughput, power, Unreal-parity, or bottleneck-removal claim is accepted until a
coordinator-managed release benchmark compares first acquire, resumed acquire, contiguous draw,
release and checkpoint at 1/64/1,024/65,536 streams on the same machine and power profile. Required
output is p50/p95, allocations, lock-contention samples, CPU time, package power and checkpoint bytes.

## Validation Matrix

- golden PCG vectors and unbiased bounded draws remain unchanged;
- second acquire of the same key rejects without changing parked or leased state;
- release/reacquire continues at the exact next draw and draw index;
- checkpoint/restore reproduces the next draw for multiple canonically ordered keys;
- malformed, duplicate, non-canonical or algorithm-mismatched checkpoints fail closed;
- active-lease checkpoint, reseed and eviction leave all state unchanged;
- explicit eviction causes a documented re-derivation, not silent state retention;
- multi-threaded same-key admission has exactly one winner;
- a large-key stress case preserves one entry per key and canonical checkpoint order;
- static guards confirm the draw owner contains no registry mutex or key hashing.

## Current Gate

Source implementation is complete for the registry slice. The completed items are:

- stable versioned service/stream checkpoint contracts with canonical-order and algorithm guards;
- complete `RandomAuthority`, bounded stable-key registry, exclusive lease and hard-cut
  `acquire_stream` API;
- checkpoint/restore, reseed, exact-key eviction and world/entity-scope eviction semantics;
- `CoreRuntime` construction from a full random checkpoint and handle-level progression coverage;
- release-path hard invariants and one complete authority lifetime shared by service and leases.

Direct isolated Windows `rustc` validation writes only under
`F:\codex-targets\019ffe-runtime22-random-registry-20260829`: production metadata compilation is
green, `zr_contracts` tests are 8/8 green, and the random kernel tests are 17/17 green. The kernel
suite includes a 4,096-key canonical-order stress case, two-thread same-key admission, capacity and
scope-eviction guards, release/reseed interleaving, checkpoint replay and a static lock/hash-free
draw-owner guard.

Managed Cargo validation remains blocked by the separately owned workspace manifest/lock
integration for `zr_contracts`; this slice does not alter either shared file. Independent re-review
found no remaining Critical, Important or Minor issue and marked the source ready. The milestone
still remains `source-complete-validation-pending`; no milestone commit or WeCom completion message
is allowed until managed Cargo validation completes.
