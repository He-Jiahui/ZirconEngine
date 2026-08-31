# Frameworks01 M1 `zr_contracts::random` physical hard cut (2026-08-28)

## Status

- `architecture_review_complete`
- `reference_engine_review_complete`
- `tdd_red_captured_missing_physical_crate`
- `physical_contract_source_owner_created`
- `old_runtime_contract_implementation_deleted`
- `runtime_facade_reduced_to_explicit_projection`
- `runtime_kernel_consumers_migrated_to_canonical_contract_owner`
- `release_increment_invariant_bypass_removed`
- `seed_receipt_successor_generation_fail_closed`
- `rust_1_94_const_receipt_compile_fixed`
- `runtime22_support_failure_returned`
- `runtime22_service_checkpoint_exact_blob_delegated`
- `static_owner_guard_green_13_of_14`
- `current_random_guard_layout_refreshed_13_of_14`
- `shared_manifest_wiring_blocked_by_shader06_executable_owner`
- `cargo_lock_not_modified`
- `managed_rust_validation_blocked_by_root_lock_drift`
- `next_contract_domain_wholesale_move_rejected`
- `runtime22_random_registry_handoff_sent`
- `runtime22_atomic_checkpoint_consistency_failure_open`
- `current_source_manifest_preview_refreshed_2026_08_30`
- `random_contract_wire_and_kernel_lock_review_no_new_defect`
- `milestone_not_accepted`
- `service_commit_not_requested`
- `wecom_not_sent`
- `performance_claims_not_admitted`

## Outcome

Frameworks01 M1 now has a real `zr_contracts` crate whose first physical domain is deterministic
Random persistence and identity. Algorithm identity, sequence identity, stable stream keys,
service state, stream state, receipts, validation errors, and their behavior tests moved out of
`zircon_runtime/src/core/framework/random` into
`zircon_runtime/crates/zr_contracts/src/random`.

The old Runtime directory now contains only `mod.rs`, an explicit projection of the approved DTO
surface. No old child implementation, wildcard export, compatibility alias, or copied contract
remains. The Runtime kernel imports canonical DTOs directly from `zr_contracts::random`; the public
Runtime projection remains available to upper product modules without making App, Editor, or
plugins depend directly on the foundation crate.

```text
zr_contracts::random <--- zircon_runtime::core::runtime::random
          |
          +----------> zircon_runtime::core::framework::random projection
```

The hidden `random::assembly` module is not a product API. It lets the Runtime kernel construct a
63-bit sequence ID from a uniformly derived word and commit a `RandomState` after preserving the
odd-increment invariant, while keeping the DTO representations private. The Runtime facade does
not re-export this module.

## Architecture and reference review

The complete current Random contract/kernel module was re-read before this physical cut. No RNG
algorithm or derivation behavior was changed:

- Unreal `Runtime/Core/Public/Math/RandomStream.h` keeps explicit initial/current state beside Core
  execution. This supports a single low-level owner, but Zircon does not copy wall-time/name seed
  conveniences or incidental C++ value-copy semantics.
- Godot `core/math/random_pcg.h` keeps seed, stream increment, and PCG execution in its core math
  layer, confirming the state/execution relationship and PCG's odd-increment stream identity.
- Fyrox persists a deterministic seed while keeping the mutable RNG as hidden runtime execution
  state, supporting the separation between serialized identity and executable state.
- Bevy's low-dependency math crate topology supports the physical crate direction, but is not used
  as the authority for Zircon's random lifecycle.

The structural conclusion is to move only stable DTOs and invariant validation into
`zr_contracts`; BLAKE3 derivation, PCG32 advancement, bounded rejection sampling, master-seed
mutation, and runtime errors remain in `core/runtime/random`. Runtime22 still owns the authoritative
stream registry, replay checkpoints, lifetime/eviction policy, and CPU/GPU parity.

## TDD and static evidence

Before implementation, the focused physical-owner test failed because
`zircon_runtime/crates/zr_contracts/Cargo.toml` did not exist. After the source hard cut, the initial
complete guard reported 12/13 GREEN in 24.622 seconds. Final code review then found that the hidden
assembly constructor accepted a raw `increment: u64` and protected the odd-increment invariant only
with `debug_assert!`; a workspace-internal caller could therefore create malformed release state.
The new invariant guard first failed on that exact signature. The constructor now accepts a
validated `RandomSequenceId`, and draw progress uses hidden assembly
`random_state_with_progress(current, generator_state, draw_index)` to preserve the already-validated
algorithm/increment while changing only generator state and draw index. The helper is not re-exported
by the Runtime facade, and `RandomState` did not gain a public progress mutator. The old constructor
name and raw increment parameter have zero source matches.

After the repair, the complete guard reports 13/14 GREEN in 48.836 seconds. The single remaining
failure is exact and expected: the shared workspace manifest does not yet list
`zircon_runtime/crates/zr_contracts`.

The 13 passing tests prove that:

- the new contract owner contains no BLAKE3, PCG transition, bounded-draw, reseed, or Runtime
  execution implementation;
- all direct Runtime kernel/gateway consumers use `zr_contracts::random` rather than the product
  facade;
- the old Runtime contract owner has no retired child modules and exposes only an explicit curated
  symbol list;
- sequence range, generation exhaustion, non-copyable stream/service authority, borrowed runtime
  accessors, and anti-alias/anti-wildcard scanners retain their earlier behavior.

The contract review also found that public `RandomSeedReceipt::new` accepted arbitrary generation
pairs even though the DTO is immutable evidence for one committed seed transition. The old
constructor now has zero source matches. `RandomSeedReceipt::try_new` returns a typed
`RandomSeedReceiptError` unless `generation == previous_generation.checked_add(1)`; Runtime authority
projects that error through `RandomServiceError` after its existing generation-exhaustion gate. A
focused contract regression covers a rejected generation jump, an accepted single successor, valid
serde round-trip, and malformed serialized generation rejection. `RandomSeedReceipt` uses a manual
`Deserialize` implementation that returns through the same `try_new` validation, so persisted input
cannot bypass the constructor invariant. The current Random boundary guard was refreshed to the
actual folder-backed checkpoint/test layout and `acquire_stream` authority surface; it reports 13/14
with the unwired workspace manifest as its only failure. No Random algorithm, derivation, registry,
checkpoint, or draw-time behavior changed.

No Cargo command was launched. Temporary Python output remains under
`D:\zircon-frameworks01-r12-zr-contracts-random-20260828`; no artifact was written to `C:`.

## Shared manifest coordination gate

The required atomic integration files are `Cargo.toml`, `Cargo.lock`, and
`zircon_runtime/Cargo.toml`. Transfer preview request
`83ccfb36314c407ba9ed47daedbcc09e` at coordinator baseline epoch 525 rejected all three because
Shader06 session `01a019a5-b15f-7461-a1b0-ce4b6aa8e710` remains executable in
`resolving_failure`. Current SHA-256 values at preview time were:

- `Cargo.toml`: `d1de7ecad881433a6a23762319c958316331f903e0372bab5f7976d751dfe3a9`;
- `Cargo.lock`: `33a183616d7aadec95d09762ef4e6302ac650bfd0ad3626c79c831724f115321`;
- `zircon_runtime/Cargo.toml`:
  `239b30e3b81de99ea1c148081dfcba5ad25863554c8e8d00714a09127a019d9b`.

The lock hash has drifted from Shader06's recorded source hash, so Frameworks01 will not apply an
old preview or regenerate the lock. After that owner commits/releases the complete current blobs,
Frameworks01 must take a fresh preview, transfer/lease the exact hashes, add the workspace member
and dependency, update the lock through managed Cargo, then rerun the 13-test guard and managed
`zr_contracts`/Runtime validation.

The workspace HEAD and all three shared manifest blobs continued to move after that preview. The
recorded request and hashes are historical ownership evidence only; they are not valid transfer
inputs. No transfer may be applied until Shader06 becomes non-executable, and the next attempt must
start with a fresh current-source preview.

A fresh 2026-08-28 preview at baseline epoch 530, request
`ef2f4156f31142fc941f90924df643ca`, produced fingerprint
`8eaac4c63075d7627959675f2b3d31a3c5e0cd28501681797a9c1b9bb0075141`. All three paths remain
ineligible with `source_owner_executable`; the current hashes are still the values above, while
`Cargo.lock` attribution still names stale source hash
`b07d72bf844430659c379ec5887bbf446a917222d6a198bdfe1d9be57ead15b2`. The focused guard was
rerun from the repository Python import root and remained 12/13 in 29.969 seconds, with this missing
workspace member as its only assertion failure. After the increment-invariant review added one
guard, the current result is the 13/14 run above. This preview is evidence, not an apply token.

A fresh 2026-08-29 preview at baseline epoch 548, request
`f0c15f2c5f2747afa10076a49a246bb7`, produced fingerprint
`441ee4817b2d65c51a691eeada9ffc526893c666536fcd75777a0c8e28c6c70e`. All three paths remain
ineligible with `source_owner_executable` and still attribute to Shader06 session
`01a019a5-b15f-7461-a1b0-ce4b6aa8e710` in `resolving_failure`. Current hashes are
`d1de7ecad881433a6a23762319c958316331f903e0372bab5f7976d751dfe3a9` for root `Cargo.toml`,
`70a18867be9385416425dbbda9b9adb5692fca2bee887418a224002e58eb1217` for `Cargo.lock`, and
`531f8a844f198b99e0f28991ab6eb30ee816c90f8d105c5bc6edf709127c17ff` for Runtime
`Cargo.toml`. The latter two no longer match Shader06's attributed content hashes; the root
manifest still contains no `zr_contracts` member or workspace dependency, and the Runtime manifest
and lock contain no `zr_contracts` dependency/package. Frameworks01 did not apply, claim, or edit
these shared blobs. The 2026-08-29 preview supersedes every earlier preview as current evidence but
is still not an apply token.

At current HEAD `11cac2d08a891ee92dcc206fd84a2d15f9e1a3f4`, coordinator-managed Windows
validation job `aad41282a276467d9b8760af4913aaf9` used the `D:` target pool. It started at
2026-08-28 20:43:03 +08:00, finished at 20:43:14, and released its Cargo slot at 20:43:23 with no
live Cargo process left behind. `cargo test -p zircon_runtime --locked --verbose --lib
project_asset_manager` exited 101 before compilation because the repository lock file would need an
update.

A structured TOML-to-lock audit of every current workspace member, including normal, development,
build, and target-specific dependency tables plus dependency aliases, found one current graph
mismatch: the `zircon_app` manifest declares the Windows-only `windows-sys` dependency while the
`zircon_app` package entry in `Cargo.lock` does not. The new `zr_contracts` source is also not yet a
workspace member, as the 13/14 static guard records. The App manifest is a mixed blob whose archived
Frameworks05 attribution covers the UI importer/App composition boundary and whose current hash has
subsequently drifted; the shared lock remains attributed to an executable Shader06 session.
Frameworks01 therefore will not rewrite either mixed blob, run Cargo without `--locked`, or create a
partial lock update. The existing Frameworks05 ZUI/App handoff and the current shared-lock owner must
first integrate the complete current dependency graph before this hard cut can enter Rust validation.

### 2026-08-30 current-source coordination and algorithm re-review

At current `HEAD=cc5cadbd597c3707954ebd6109fad0fd5643a152` and coordinator baseline epoch 573,
fresh exact transfer-preview request `076e9a0ce4104c9eb79eba3cd6a028a0` again rejected all three
shared manifest paths solely with `source_owner_executable`. The source session remains Shader06
`01a019a5-b15f-7461-a1b0-ce4b6aa8e710` in `resolving_failure`. The preview fingerprint is
`bc4b5a329c564ea0c7c9db1938aab03d0f64863f40cdd339ab3724b3c109c4e2`; current SHA-256 values are:

- `Cargo.toml`: `d1de7ecad881433a6a23762319c958316331f903e0372bab5f7976d751dfe3a9`;
- `Cargo.lock`: `70a18867be9385416425dbbda9b9adb5692fca2bee887418a224002e58eb1217`;
- `zircon_runtime/Cargo.toml`:
  `531f8a844f198b99e0f28991ab6eb30ee816c90f8d105c5bc6edf709127c17ff`.

The root manifest still matches the source attribution, while the lock and Runtime manifest have
drifted from Shader06's attributed hashes. Shader06's current immutable scope no longer lists these
three paths, but its executable attribution remains authoritative for transfer admission. Frameworks01
did not apply the preview, claim or edit the manifests, regenerate the lock, or absorb their mixed
text/RHI/App dependency changes. The owner must first complete a legal service commit/release or
status rotation; the next Frameworks01 attempt must start from another fresh preview.

The same pass reread the complete current `zr_contracts::random` source and the Runtime random
authority/registry/service/lease/stream chain before considering another optimization. Persisted
algorithm identity already serializes the explicit `u16` stable ID and rejects unknown values;
sequence, state, seed receipt, and service checkpoint deserialization all return through their typed
invariant constructors. Runtime lifecycle operations consistently acquire registry state before the
seed authority when an atomic cross-owner snapshot is required; no seed-to-registry reverse order was
found. First-stream BLAKE3 derivation remains inside admission so reseed cannot publish a new era while
an old-era stream is inserted. The existing measured cost is a first-admission cost amortized by the
parked authoritative registry, not evidence for a frame bottleneck. Moving it outside the lock would
require an epoch-qualified optimistic admission/retry protocol and is not approved without a product
contention profile. Unreal `FRandomStream` and Godot `RandomPCG` continue to support stateful stream
ownership; they do not justify replacing Zircon's checkpoint/reseed atomicity. No Random production
source was changed and no new latency, power, parity, or optimality claim is admitted.

## Next contract-domain admission gate

The 2026-08-28 schema-2 partition audit and a full consumer/reference review rejected another
directory-shaped move into `zr_contracts`:

- `core/framework/events.rs` remains a string-topic/`serde_json::Value` event surface while
  Runtime55 owns the hard cut to the typed event service. Moving it now would make the retiring
  dynamic authority canonical.
- `core/framework/time` mixes immutable policy/domain DTOs with clock advancement, pause/rate, and
  fixed-debt execution. It requires a declaration/behavior partition and Runtime22 owner closure,
  not a whole-directory move.
- `core/framework/tasks` contains 10 files classified as 2 declaration-only and 8 mixed. The
  apparent DTOs are not one coherent task contract: `TaskPollBudget` has no production execution
  consumer, `TaskCancellationPolicy` has no Runtime executor, and `AsyncTaskDescriptor`/
  `AsyncTaskStatus` are used only by dynamic-scene-local state machines rather than admitted by
  `JobScheduler`.
- `core/framework/project` has 197 symbol-consumer files and combines persisted descriptors with
  export policy, plugin path generation, selection mutation, and target resolution. `platform`
  likewise contains 18 declaration-only and 24 mixed files. Neither domain is admitted as an M1
  whole move.

Unreal `Tasks/Task.h` is the primary task reference: launch admission, priority, prerequisites,
handle identity, wait, and result access form one execution model. Godot's WorkerThreadPool also
binds task/group IDs to submission and completion, while Bevy keeps pool construction and thread
assignment in its execution owner. These designs contradict promoting Zircon's disconnected
descriptor/status lane as a final contract.

`ParallelSliceExecutor` is the sole admitted task subset: it has 13 production-like consumer files
and one Runtime implementation, so it is a real dependency-inversion contract. Its current source
also contains Runtime02-owned ordered-map/fast-path work, so Frameworks01 will not rewrite or move
that blob. Runtime59/11 must first converge the canonical task admission/handle/status model and
delete the old DTO lane; only the resulting stable contract plus the independently owned executor
trait may enter `zr_contracts`, without aliases or compatibility projections.

The Random algorithm review found no error in PCG32 advancement, odd increment construction,
unbiased bounded rejection, draw accounting, or snapshot restoration. It did confirm that
`CoreRuntime::random_service().stream(key)` can create multiple mutable streams for the same stable
key. A local cache would hide rather than solve authority, lifetime, replay, and eviction semantics,
so the structural fix remains Runtime22's authoritative stream registry. The finding was sent to
the Runtime primary Session; this slice does not claim registry closure.

Until the current `zr_contracts::random` manifest is atomically wired and validated, Frameworks01
will not create a second unwired contract crate/domain. The next physical source move remains the
already reviewed `zr_kernel::state_machine` cut after the fixed layer-0 order is restored.

## Runtime22 atomic checkpoint consistency handoff

A complete lock-boundary review found a structural replay defect outside Frameworks01 ownership.
`RandomService::checkpoint()` captures registry entries, releases the registry lock, and only then
captures the authority seed/generation. `RandomAuthority::reseed()` holds the registry lock while it
changes that seed/generation and clears streams. The following legal schedule therefore creates a
checkpoint for a state that never existed:

1. checkpoint captures the old parked streams and releases the registry lock;
2. reseed acquires the registry lock, advances seed/generation, and clears the streams;
3. checkpoint captures the new authority snapshot;
4. checkpoint returns old stream entries paired with the new authority snapshot.

`RandomServiceCheckpoint::try_new` validates algorithm identity and canonical stream-key order, but
cannot reject this mixed-era pair. The finding was handed to Runtime22, which owns authoritative
registry, checkpoint, restore, replay, lifetime, and eviction semantics. Frameworks01 did not edit
the Runtime22 source or create a compatibility surface.

The architectural repair must preserve the single `registry -> seed` lock order and capture the
authority snapshot through the registry-locked checkpoint operation before releasing the registry.
It must not add a second lock-order path, a fallback snapshot, an alias, or caller-specific retry.
Acceptance requires a deterministic concurrency regression that pauses checkpoint capture at the
registry/authority boundary, attempts reseed, and proves mixed-era output is impossible, followed by
restore/replay upward validation. Runtime22 has materialized the canonical child-plan Failure and
implemented the single lock-order repair; its managed validation and fixed return remain Runtime22
closeout work, so deterministic Random checkpoint/replay closure remains explicitly unaccepted here.

Runtime22 closeout also requires the canonical contract dependency
`zircon_runtime/crates/zr_contracts/src/random/service_checkpoint.rs`. Frameworks01 cannot commit the
unwired `zr_contracts` crate independently while the three mixed workspace manifests remain under an
executable Shader06 attribution. The current file was therefore delegated unchanged to Runtime22 as
an exact lifecycle dependency: SHA-256
`188b0423df6b14ed10295baf41909ae4336aae4b80531fef5f4bfff000c11917`, transfer preview request
`437243bf16ec4871b9794ccbea2fb16d`, and successful transfer-apply request
`b2382e335a9f4435a0289926faba867c`. This transfers only the named blob for the checkpoint closeout;
it does not transfer the rest of `zr_contracts`, change the file, or accept Frameworks01 M1.

## Rust 1.94 const receipt support repair

Runtime22's approved isolated compiler boundary found that the current
`RandomSeedReceipt::try_new` did not compile on Rust 1.94.1. The successor-generation predicate
called `Option<u64>::PartialEq` inside a public `const fn`; that trait operation is not const-stable
on the locked compiler. A same-toolchain minimal probe reproduced exactly two E0658 diagnostics,
isolating the fault below serde, registry and checkpoint behavior.

Frameworks01 preserved the public const API and changed only the implementation shape: it matches
`previous_generation.checked_add(1)` and compares the resulting primitive `u64` with `generation`.
Overflow and mismatched generations still return
`RandomSeedReceiptError::NonSuccessorGeneration`; manual `Deserialize` still returns through the
same `try_new`. The new `service_state.rs` SHA-256 is
`c10422917a2d8d1896c428c75173d6a7e9821adac2fdadb2af2f75aa93203f21`.

Rust 1.94.1 compiled the real `zr_contracts/src/lib.rs` as a 530,836-byte rlib with exit 0. An
external const/serde harness then compiled and executed with exit 0, covering legal const
construction, overflow and generation-jump typed rejection, valid serde round-trip and malformed
generation rejection. Rlib/harness hashes are
`95556433c15e48fc9f45a7ee7fe1567a6eb64ba79c631fb5adbd7f3191513e39` and
`15ba61d7642f9cff963a55c34e2098c21f3c681ab70997c84fe4055c226ce50e`; all output and temporary
state stayed under `D:/zircon-frameworks01-r12-random-const-compile-20260829`. No second Cargo was
started. `rustfmt` and scoped diff checks passed. The full Random boundary guard remains 13/14 in
30.854 seconds, with only the pre-existing root workspace member assertion RED.

The canonical support Failure was imported and atomically returned to Runtime22. Coordinator return
request `a03d4e3e4dd04fdca37e5b7cde2801cc` completed with delegated proof
`d66c6ed2330e4078a8e6e6144058fee3`. The fixed artifact SHA-256 is
`6178a12dae56a647c7738f21c3600e16ce5d33b3b0f2b3c01b677e9edab66b72`; the Frameworks01 return
record SHA-256 is `9d68a696b9079771ab6c3b76b50d384a951e10f8bd7d324e7bb0b228c62986f9`.
The old fixing-side `failure-*` artifact no longer exists. The repository-wide handoff validator
still reports 33 pre-existing errors owned by Runtime79, Runtime90 and an older 2026-08-25
Frameworks01 handoff, so only this exact lifecycle is closed. This repair reopens Runtime22's
checkpoint RED/GREEN and does not claim its atomicity, workspace manifest, Cargo, milestone,
performance or power gates.

## Performance boundary

This batch is a physical ownership correction, not an algorithm optimization. It preserves the
previously measured Random profile: BLAKE3 stream derivation was roughly 426 ns/op on the measured
Ryzen 7 5800H, while contiguous PCG32 draw was roughly 1.66 ns/op. That evidence identifies repeated
stream construction as the architectural risk and continues to route the optimization to Runtime22's
authoritative registry. The invariant repair adds no draw-time validation, allocation, hash, or
branch: the hidden progress helper copies the existing validated identity and replaces two `u64`
progress fields. Moving DTOs across a crate boundary does not justify latency, throughput,
allocation, power, bottleneck-removal, engine-parity, or optimal-complexity claims.

This slice remains unaccepted until exact manifest integration, managed Rust validation,
independent review, coordinator milestone acceptance, service commit, and quantified WeCom
notification are complete.

## 2026-08-30 combined boundary verification

With `TEMP` and `TMP` pinned under `E:/Git/ZirconEngine/.codex/state/tmp`, the current-source command
`python -B -m unittest tools.tests.test_frameworks_01_random_contract_kernel_boundary
tools.tests.test_frameworks_01_state_kernel_owner_boundary -v` ran 20 tests in 87.912 seconds.
Nineteen tests passed. The sole failure was
`test_random_contract_and_kernel_have_disjoint_physical_owners`, specifically its assertion that the
root workspace already contains `"zircon_runtime/crates/zr_contracts"`; the current root
`Cargo.toml` still omits that member. Random contract fail-closed behavior, implementation-owner
separation, single Random authority, stream non-copyability, old-owner import rejection, state owner
uniqueness, bounded transition observation and transition hook ordering guards all passed.

This is a precise `19/20` preflight result, not a green gate. It confirms that the remaining RED is
the legally blocked physical manifest integration described above; it does not authorize a partial
workspace edit, weaken the guard, accept `zr_contracts`, or substitute for managed Rust/product,
performance, power, independent-review, commit or WeCom evidence.

## 2026-08-30 external mixed-era checkpoint review

A second full DTO-to-Runtime restore review found that the registry/seed lock-order repair closes
only the checkpoint race for snapshots produced by the active process. It does not make the public
checkpoint format self-validating. `RandomStreamCheckpoint` currently carries only a stable key and
`RandomState`; it does not carry the authority generation that produced that state.
`RandomServiceCheckpoint::validate` checks the format version, stream algorithm and strict key
ordering, but cannot prove that a stream belongs to `RandomServiceState::master_seed_generation`.
The manual `Deserialize` implementation returns through that same validation path.

Consequently, an external producer can splice parked streams from generation N into a service
snapshot from generation N+1 while preserving every current validator invariant.
`RandomAuthority::from_checkpoint` and `RandomStreamRegistry::from_checkpoints` then restore the
pair without another generation check. Existing keys resume generation-N progress while unseen
keys are derived from the generation-N+1 authority, so one restored service can expose mixed-era
random behavior even after atomic in-process capture is fixed.

Frameworks01 sent this exact finding to Runtime22 on 2026-08-30. The hard-cut design must review the
service checkpoint DTO, stream checkpoint DTO, capture, restore, serde format version and the
independent `evict_stream`/`evict_world`/`evict_entity` checkpoint-return contracts as one migration.
The minimum candidate is an authority-generation binding on every stream plus fail-closed service
validation; any stronger seed/key/draw-index state verification must remain in the Runtime random
kernel rather than duplicating generator algorithms in `zr_contracts`. The accepted design must
bump the persisted format and must not retain a version-1 compatibility path.

No Runtime22-owned source or delegated `service_checkpoint.rs` blob was edited during this review.
The external mixed-era RED, migration ownership and managed restore/replay validation remain
Runtime22 work. Frameworks01 M1 therefore remains unaccepted even if the current in-process
atomicity validation becomes green.

Runtime22 confirmed on 2026-08-30 that Session
`root-runtime22-checkpoint-atomicity-20260829` remains the exclusive implementation owner for this
schema hard cut. Its canonical manifest covers `stream_checkpoint.rs`, `service_checkpoint.rs`,
`checkpoint_error.rs`, checkpoint contract tests, Runtime `registry.rs` and `service.rs`, and the
restore, replay and eviction regressions; `authority.rs` is added only if the final construction or
restore path requires it through a fresh exact-path preview and claim. Frameworks01 retains no
implementation authority over those files.

The agreed acceptance shape is a version-2-only format. Every `RandomStreamCheckpoint` binds the
`master_seed_generation`; `RandomServiceCheckpoint::try_new`, `validate` and manual `Deserialize`
reject cross-generation entries; restore rejects a checkpoint before installing any state unless
all entries belong to the service generation. `checkpoint`, `evict_world` and `evict_entity` must
capture entries plus generation atomically under the one `registry -> seed` lock order.
`evict_stream -> Option<RandomState>` remains explicitly a non-restorable removal observation unless
the complete API is hard-cut to the generation-bound checkpoint type; no independent recovery
bypass is permitted. Runtime22's RED matrix covers constructor and serde mismatch, version-1
rejection, deterministic paused concurrency, restore/replay and both bulk eviction paths.

## 2026-08-31 physical wiring gate refresh

Fresh coordinator transfer preview `95c197de4b6c46e6a3a35663ed139da4` rechecked the three
atomic wiring inputs: root `Cargo.toml`, `Cargo.lock` and `zircon_runtime/Cargo.toml`. All three remain
owned by Session `01a019a5-b15f-7461-a1b0-ce4b6aa8e710` in `resolving_failure`; every path is
ineligible solely because `source_owner_executable` is still true. Frameworks01 did not apply the
preview, rotate the foreign Session, or rewrite any mixed manifest blob.

The current workspace member/dependency graph and lockfile contain `zr_resource` but not
`zr_contracts` or `zr_kernel`. The unwired `zr_contracts` tree currently contains 15 files: its
manifest and library root plus Random algorithm, assembly, key, state, service state, stream/service
checkpoint, checkpoint error and folder-backed tests. `zr_kernel` does not exist. This is the
required dependency order, not permission to create another disconnected crate: first land the
Runtime22 version-2 checkpoint schema in the existing contracts tree, then atomically wire and
validate `zr_contracts`, and only then materialize the reviewed `zr_kernel` state-machine owner with
its manifests and consumers in one hard cut.
