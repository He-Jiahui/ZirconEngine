# Frameworks01 M1 deterministic random contract/kernel partition (2026-08-24)

## Status

- `source_implemented`
- `contract_kernel_hard_cut_complete`
- `pcg32_sequence_identity_fail_closed`
- `seed_generation_exhaustion_fail_closed`
- `implicit_stream_copy_hard_cut_complete`
- `master_seed_authority_copy_hard_cut_complete`
- `static_owner_guard_green_13_of_13`
- `exact_source_random_tests_green_12_of_12`
- `independent_review_green_no_critical_or_important`
- `cancelled_r10_ownership_transfer_complete_10_of_10`
- `runtime_gateway_current_source_ownership_pending`
- `runtime_product_revalidation_blocked_by_foreign_zr_rhi_wgpu`
- `stream_registry_and_replay_open`
- `blake3_performance_measurement_complete`
- `blake3_local_micro_optimization_rejected`
- `runtime22_stream_registry_entry_gate_confirmed`
- `milestone_not_accepted`
- `service_commit_not_requested`
- `wecom_not_sent`
- `whole_product_performance_claims_not_admitted`

## Architecture Result

The physical owner split remains unchanged. Stable algorithm, sequence, key, state, service-state,
and receipt contracts live in `core/framework/random`; BLAKE3 derivation, master-seed authority,
PCG32 execution, and operation errors live in `core/runtime/random`. No implementation alias,
forwarding module, or duplicate generator was added.

This correctness atom closes four defects found after the owner split:

- PCG32 represents a stream with an odd 64-bit increment, leaving exactly 63 identity bits. The old
  crate-private constructor accepted an arbitrary `u64`, so selectors that differed only in bit 63
  silently produced the same increment. `RandomSequenceId` now validates that range, fails closed on
  deserialization, and is required by `RandomStream::from_seed`. Uniform BLAKE3 output is reduced to
  its low 63 bits before construction. This produces the same increment as the previous left shift,
  so `Pcg32XshRrV1` output is not silently versioned.
- `RandomService::reseed` previously used `saturating_add(1)`. At `u64::MAX` it changed the seed while
  reusing the terminal generation. It now returns `Result<RandomSeedReceipt, RandomServiceError>`,
  computes the next generation with `checked_add` before mutation, and leaves the complete authority
  unchanged on exhaustion. No infallible compatibility overload remains.
- `RandomStream` previously derived `Clone + Copy`, so assignment or helper argument passing could
  fork mutable draw state without naming a stream key or crossing a replay-visible boundary. The
  execution owner is now non-cloneable and non-copyable, and read-only state accessors borrow it.
  `RandomState` snapshot/restore is explicit state reconstruction for checkpoints and tests, not a
  completed fork policy: the same stable key can still recreate an initial stream through the service.
- `RandomService` previously derived `Clone + Copy`, and both `CoreHandle::random_service()` and
  `CoreRuntime::random_service()` returned it by value. A caller could therefore create a detached,
  mutable master-seed authority without crossing the explicit persisted-state boundary. The service
  is now non-cloneable and non-copyable, its query and stream-derivation methods borrow `&self`, and
  both Runtime accessors return `&RandomService`. Tests that require an independent authority must
  name the `RandomServiceState` snapshot/restore operation explicitly. Runtime exposes no mutable
  service accessor. Follow-up review found that the backing `CoreRuntimeInner.random_service` field
  was still `pub(crate)`, allowing crate-internal code to combine `Arc::get_mut` with direct reseed,
  and that the guard did not reject additional by-value or mutable accessors. The field is now
  private, runtime construction is owned by `CoreRuntimeInner::new`, and Handle reaches it only
  through the state owner's shared-borrow method. Mutation tests reject a re-exposed field plus
  additional `RandomService` value and `&mut` return surfaces. The final guard scans every product
  Rust candidate that names `RandomService`, rejects `Clone`/`Copy` and authority-returning methods
  in any other module, and requires exactly one shared-borrow accessor in each of Runtime, Handle,
  and `CoreRuntimeInner`. Any `type` or `use ... as ...` authority alias is rejected at its source,
  so a second file cannot hide a value/copy escape behind a renamed authority.

The review also found and fixed a compile error hidden behind the current RHI failure: the
entity-absent arm of `append_stream_key` returned `&mut blake3::Hasher` while the entity-present arm
returned `()`. Both arms now update the hasher and return `()`.

## Reference Review

- Unreal `FRandomStream` keeps initial and current seeds together in Runtime Core, which supports a
  single explicit execution owner but does not provide Zircon's stream hierarchy or replay identity;
  C++'s incidental value-copy ability is not imported into the stricter Rust contract.
- Godot `RandomPCG` keeps seed, state, and increment together and delegates initialization to the
  vendored PCG implementation. `dev/godot/thirdparty/misc/pcg.cpp` constructs the increment as
  `(initseq << 1) | 1`, confirming that the high selector bit cannot be part of PCG32 stream identity.
  Its public `RandomNumberGenerator` is a `RefCounted` mutable owner rather than a copied state DTO.
- Runtime22 remains the semantic owner for stream registry, replay manifest, checkpoint, CPU/GPU
  parity, and cross-platform determinism. This Frameworks01 atom only hardens the lower contract and
  kernel implementation needed by that plan.

## Performance Preflight (2026-08-27)

The previously open BLAKE3 measurement is complete against the exact current production modules.
An optimized standalone harness includes `core/framework/random` and `core/runtime/random` by path,
uses `rustc 1.94.1` with `-C opt-level=3 -C target-cpu=native`, and links the existing release
dependencies from the managed F-drive target pool. The output executable and all temporary files are
under `D:\zircon-validation\frameworks01-random-profile-20260827`; no Cargo job or C-drive artifact
was created. The host is an AMD Ryzen 7 5800H (8 cores / 16 logical processors) on Windows 11 build
26200. Each case uses two warmups and nine measured samples; the table reports the second complete
release run's median and nearest-rank p95. A first run linked against debug BLAKE3 and is explicitly
invalid for performance comparison.

| Case | Operations/sample | Median ns/op | p95 ns/op |
| --- | ---: | ---: | ---: |
| same stable key derivation | 100,000 | 426.430 | 933.143 |
| unique stable key derivation | 100,000 | 426.462 | 505.509 |
| unique derivation plus first draw | 100,000 | 425.691 | 443.005 |
| PCG32 seed initialization only | 100,000 | 2.559 | 7.045 |
| contiguous PCG32 draw | 1,000,000 | 1.657 | 1.911 |
| bounded PCG32 draw, upper bound 17 | 1,000,000 | 5.118 | 5.542 |

The same-key and unique-key cases are the same order, so the current API performs no implicit
reuse. One stable-key derivation is about 166.7 times the cost of PCG32 seed initialization and
257.4 times one contiguous draw. Re-deriving 100,000 streams costs about 42.65 ms on this host,
whereas 100,000 persistent-stream draws cost about 0.166 ms. This identifies repeated stream
construction as the architectural risk; it does not identify PCG32 execution as a bottleneck.

Frameworks01 therefore rejects a local hash substitution or hidden cache. BLAKE3 owns the stable,
versioned derivation boundary and its fixed cost is paid only when a stream is admitted. The correct
optimization is Runtime22's authoritative stream registry: admit a `RandomStreamKey` once per owned
lifetime, retain the non-copyable stream and draw index, and make snapshot/fork/replay transitions
explicit. That design removes repeated derivation from the frame loop without weakening identity or
creating detached mutable authorities. No registry is implemented in this Frameworks01 record,
because Runtime22 owns its lifecycle, replay, eviction, and checkpoint semantics.

The final harness source SHA-256 is
`606317f561953c4919de403cc7a668e1b2233673606591804a07d7e5066c7c03`; the final release executable
SHA-256 is `1364ae119d449a0004cdacd33daea0ee44162991dfbb4c2ade6073c9e74f0ce2`. The measured production
service, stream, and key source hashes remain respectively
`e5a73a61c30c4571c187a543336d4a94328e603192482b962b1a6cc9c09379b1`,
`ba78fc3956e34155e2ed77c905ec33b0d6b13706ee44b63188281f14f2d41021`, and
`269c0b85ca3ed3c8598d8349d53077827e75b6341c988d4e9c5e0f30504aeb8b`.

## Validation Evidence

The owner guard is `13/13` GREEN in 20.955 seconds after adding explicit checks for the 63-bit typed
sequence, checked reseed generation, absence of saturating generation, glob-import migration
escapes, raw `u64` stream constructors under renamed selector parameters, and the absence of
implicit mutable-stream or master-seed-authority `Clone`/`Copy`. It pins borrowed service queries
and stream derivation, including a bounded-draw rejection vector that returns `8` only after three
PCG32 draws, scans all product candidates for extra authority surfaces, locks the three
shared-accessor owner files, and requires the backing runtime-state authority field to remain the
only `CoreRuntimeInner` authority field and remain private. Field visibility, generic value-return,
renamed/qualified field, multiline field type, external impl, and external Clone mutations all
failed before their corresponding guard changes and are GREEN as detectors afterward. A split-file
authority-alias mutation also failed before alias-source rejection and is GREEN afterward. Exact
rustfmt, Python syntax, and scoped diff checks are GREEN. The explicitly named post-closure suite
combining camera-controller owner (15), random contract/kernel (13), and scene-animation boundary
(9) is `37/37` GREEN in 104.517 seconds. This record does not claim the unrelated full Frameworks01
boundary discovery is GREEN; current foreign Physics/Diagnostics guards have two failures outside
this atom.

A fresh Windows managed `zircon_runtime` build plus filtered lib-test job used coordinator ephemeral
lane `bffbcc35cb1e48fe98e46697df13bd81` under `F:\cargo-targets`, from
2026-08-24 23:33:04 +08:00 through 2026-08-25 00:08:34 +08:00 (35 minutes 30.175 seconds).
The cold lane compiled `zr_math`, `zircon_runtime_interface`, and `zr_rhi`, proving the historical
`zr_rhi/src/surface.rs` E0499 shape is no longer the current blocker. Build and test then both stopped
before `zircon_runtime` on the same 14 foreign `zr_rhi_wgpu` errors. The lowest error is E0432 at
`zircon_runtime/crates/zr_rhi_wgpu/src/production/device/context.rs:6`, where the root no longer
exports `wgpu_device_features`; the set also includes missing types/traits, completion-order Default,
surface collection/result typing, a stale bind-layout field, and missing `FifoRelaxed` coverage.
The job released with exit code 1 at 00:08:37 and its F-drive target was deleted by coordinator
policy. Frameworks01 did not modify or adopt those foreign RHI files, so product validation remains
blocked rather than GREEN.

An exact-source `rustc --test` harness under ignored `.codex/state` linked only the existing D-drive
serde/serde_json/thiserror/blake3 artifacts and wrote its post-review executable to
`D:\zircon-validation\frameworks01-random-contract-20260824\random-tests-rejection-vector.exe`.
The executable is 1,233,408 bytes, SHA-256
`863872e670d9d37332b837ab503043f6ca8e9a3b22aa252a06e0ff2ada1c3a0e`, and ran 12/12 tests GREEN.
Coverage includes the published PCG32 seed/sequence vector, full service-derivation vector,
sequence range/serde rejection, seed-generation failure atomicity, explicit service-state
reconstruction, stream snapshot/restore, draw-index exhaustion, bounded-draw range, and the fixed
bound-10 rejection vector `Some(8)` after exactly three PCG32 draws. The locked service vector remains:

```text
state      = 0x0de07104618494d6
increment  = 0xceddaca06cc34e29
sequence   = 0x676ed6503661a714
draw_index = 0
```

On 2026-08-25 the ten unchanged contract/kernel implementation blobs left by cancelled Session
`frameworks01-review-corrections-r10-1b2684b4-20260825` were transferred atomically to current
Frameworks01 r12. Transfer preview request `1102cbf209764560b1c72abde14cf91a` proved all ten current
hashes equal their source content hashes with no blocking reason; apply request
`9b9931b9b0664362837f44c73b8cd044` accepted fingerprint
`ce98561b713ab6784d030145e8e9ad7d01731a4898eca55ad8417bbecbd0161c`. This changed coordinator
ownership only and did not rewrite source.

Key current blob SHA-256 values are contract algorithm
`e7c351d8ec0c1c6ec9345df691d96adf133439995c2143f6063ca4e6ebf4d064`, runtime service
`e5a73a61c30c4571c187a543336d4a94328e603192482b962b1a6cc9c09379b1`, runtime stream
`ba78fc3956e34155e2ed77c905ec33b0d6b13706ee44b63188281f14f2d41021`, runtime tests
`fd5b2e863cd300a51dacf7c898d3d360c990851067d9e65a8d55fef397347c77`, handle accessor
`64a6f846854ded9baed91ea29f45c7de3b4dff09d6a634fe3d30697855ba8f3d`, Runtime facade
`32f1e225e54d5ed236d3b7572c8a29aafc8a74c2703024d025bf8dfe34888221`, runtime Handle gateway
`4cfb5ceec8e2bdaa7490714e690a2b23b854a6ecba4627c0212ea475ae04bc1c`, runtime state owner
`da89f936fb8f5c160df5d8c8e2283ef7b8afb2732979356e6b2875af40531129`, and owner guard
`b81fcd725bba4dbf0995ce5f77af9fddc185bce5d76ce83cdc15b43684ef5d86`.

The first independent review reported no Critical or Important findings; its two Minor findings were
closed by the deterministic rejection-path assertion, precise plan wording, glob-import mutation,
and renamed raw-selector constructor mutation. The stream-ownership reviews then found two rounds of
Important boundary defects. The first covered `cfg_attr`/explicit or aliased copy implementations and
an overstated final fork policy; structured trait scanning, four copy mutations, and the corrected
Runtime22 boundary closed those. The second found that extra value/mutable `RandomService` accessors
were not rejected and that the backing field remained crate-visible. The RED field assertion,
accessor mutations, private field, owner constructor, and shared-only state method close those
concrete escapes. The third found that file-local enumeration still allowed an authority-returning
impl or `Clone` implementation in another product module. The all-product authority-surface scan,
three-owner allowlist, exact-one checks, and external-impl mutation close that cross-file escape.
The fourth found that the second module could spell only an authority alias and avoid the literal
candidate scan. Hard rejection of all RandomService type/use aliases plus the split-file mutation
closes that route. Final independent review reran the focused and complete guard and reported no
Critical, Important, or Minor findings. Text guards do not expand generated macros; no current
product macro generates this authority surface.

## Remaining Boundary

This is a correctness, ownership, and performance-entry result, not a product optimization. There is no
authoritative stream registry, stable-key fork policy, replay/checkpoint integration, CPU/GPU parity
matrix, or managed product GREEN yet. BLAKE3 stream-creation cost is now quantified, but the Runtime22
registry/lifetime optimization has not been implemented or qualified. No cache, alternative hash,
whole-product throughput, latency, power, bottleneck-removal, parity, or optimality claim is admitted.
M1 remains unaccepted and no commit or WeCom notification is requested.

The transfer does not close the current-source gateway. Canonical Runtime01 node `2493698` is
recorded in `failure-2026-08-25-random-runtime-handle-gateway-ownership.md`.
`core/runtime/handle/mod.rs` contains the
required single `mod random;` declaration, but the whole blob remains attributed to an archived
Runtime core lifecycle Session with a drifted hash and is outside r12's immutable scope.
`core/runtime/runtime.rs` and `core/runtime/state/core_runtime_state.rs` also combine Random authority
construction with Time, State, and lifecycle changes. Runtime gateway scope rotation or an exact
ownership transfer, followed by managed Runtime validation, is required before this atom can become
an integration candidate; no partial gateway migration is accepted.

The Runtime-owned master-seed service is now a single non-copyable authority exposed only by shared
borrow. Runtime22 still owns admission of stable stream keys, the authoritative stream registry,
checkpoint/replay integration, and any explicit fork capability; this atom does not claim those
higher-level policies are complete.
