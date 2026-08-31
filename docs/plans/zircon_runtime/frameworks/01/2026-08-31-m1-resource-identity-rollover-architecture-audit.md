# M1 Resource identity and rollover architecture audit

Status: architecture reviewed; lease-identity hard cut source-complete; projection identity pending
profile and consumer migration; event-order atomicity protocol locked with production not started.

## Scope and reason

The Resource foundation currently uses five unrelated `u64` counters as if they were one kind of
identity:

1. a residency token suppresses release from an old `ResourceLease`;
2. a management sequence identifies an immutable catalog projection;
3. a readiness sequence identifies an immutable readiness projection;
4. a per-row dependency revision identifies dependency state;
5. an event sequence orders one bounded event log and its receiver cursors.

All five advance with `wrapping_add`. A wrap is not merely a diagnostic discontinuity. It can make
an old lease look current, make a stale cache key equal a new projection, make an old render
residency seed match a new dependency state, or make an event receiver confuse new data with a prior
cursor. These contracts have different lifetime and failure requirements, so replacing every call
with the same checked counter would preserve the wrong architecture.

This audit is a correctness and ownership preflight. It does not claim a performance improvement and
does not use build time as runtime evidence. The current ResourceManagement/readiness release profile
is separately blocked by the open managed-sccache lifecycle Failure.

## Current-source evidence

The reviewed source fingerprints are:

- `lease.rs` `ad5fb4e1463ba66813c2435dbdb001ba911c4bac411a2adef2799bc9329c8b26`;
- `event_stream.rs` `636941a86d044ee06c523b9c9c0daed3cb874d8ebcd4c9d1976d8707784e6015`;
- `manager/resource_manager.rs`
  `f4017d2260cae82711a99e015e9fea162d167364cb5478bb235e8d27e038b994`;
- `manager/runtime_slot.rs`
  `5420555d62ecd73451085af6d73f52cfe6796cf9a0829c28aa4578cf96798e42`;
- `manager/lease_ops.rs`
  `6ae575dfc0453b42f6b4ba16ea61ee375302467003be5bac73e1dadef1b105e9`;
- `manager/commit.rs` `a41250387db16276837c94a4047bb988687a2c77a44a48cf1cbc78a38d9d489e`;
- `manager/management_projection.rs`
  `c543c0fe41cf2c3f8ad1d59da2d1ec648354c071f5665f4a3139a12e4dd98fdc`;
- `manager/readiness_projection.rs`
  `9aac00f12030fff53fef7a23e388579c2b8a0099e9787cf300a4bfe4d8c525c9`;
- `readiness_generation.rs`
  `4830e32d4b730ee5ad4f435c1239a3bdc6d397250b3dfa6c0011880cda0045f8`;
- render residency `contract.rs`
  `bd7fbb201e4fc383f6d04efce1a7b6d1dc8a758e27fd3fa0f977c2ed333631a2`;
- render residency `manager.rs`
  `8a155161c91a125654781be06111130750f94d5e9d93cd2308d05bd509e10b54`;
- project asset `management_generation.rs`
  `de11ea80a5da5bd2a2b228b623ff502bf45dace593acd697a3ec24420acc055f`.

The exact defects are:

- `ResourceAuthority::allocate_residency_token` wraps to 1. A payload replacement creates a new
  runtime slot with ref-count zero. If a still-live old lease owns the same reused token, its `Drop`
  can decrement or unload the new payload. The accompanying manual `usize` ref-count also advances
  with unchecked `+= 1`; changing only the token would leave a second unreportable `Drop`-path
  overflow authority.
- management projection publication wraps its sequence in both sparse and rebuild paths.
- readiness publication wraps the generation sequence, and a changed dependency fingerprint wraps
  the row dependency revision.
- `RenderAssetResidencyTicketSeed::matches` compares the raw readiness sequence and dependency
  revision. Simultaneous reuse can accept a stale active ticket as current even though ticket IDs
  themselves already use checked reservation and typed `TicketIdExhausted`.
- event publication and receiver advancement both wrap the cursor used by gap detection.
- `PreparedResourceMutation::commit` is intentionally infallible after prepare, while acquire and
  lease `Drop` can also publish readiness outside the commit serial. A late checked-add error cannot
  simply be inserted into projection publication without creating partial mutation or an error that
  `Drop` cannot return.

## Reference-engine review

The primary reference is Unreal's Streamable Manager:

- `dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/StreamableManager.h`, SHA-256
  `406e4dca4b52153df2927820c8e431c16220579fe9d9f50de8d12229e43c8c94`;
- `dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/StreamableManager.cpp`, SHA-256
  `650d3892b18cf24bd5aad1eff3c57e44570e5f8ef56ae5f441374f0244062fec`.

`FStreamableHandle` is retained by `TSharedRef`/`TSharedPtr`, parents use weak handles, and asset
entries track the exact active handle object. Release removes the exact `SharedThis`/handle pointer
from managed and per-asset collections. It does not recycle a manager-wide integer to decide whether
an old handle may release a new payload incarnation. This is the relevant model for Zircon's
in-process lease lifetime.

Bevy provides a useful counterexample rather than the selected authority. Its strong handle is an
`Arc<StrongHandle>`, but its dense `AssetIndexAllocator` recycles a `u32` generation by ordinary
addition. The local sources are `handle.rs`
`dad84a87ce0369fce480634e71f4615c811189a9513e6d18be26c166dfbfd644` and `assets.rs`
`c05764085b56ffe17c767bd7b2363695a35019e92ba314a8656c364c84406237`.
That design demonstrates generational lookup, but the unchecked finite generation is the same
rollover class being removed here and is not copied as Zircon's lease authority.

## Selected identity model

### Lease lifetime identity

Iteration 1 used the payload `Arc` as incarnation identity. Review rejected that as incomplete
because it preserved the manual `usize` lease count. The accepted model is one private
`Arc<ResourceLeaseIdentity>` per payload incarnation. The runtime slot owns one strong reference and
each live `ResourceLease` owns one. Replacement installs a new identity object; an old lease can
never match the new slot.

`Drop` must move its identity reference into the manager. Under the Resource authority write lock,
the manager first verifies `Arc::ptr_eq`, then drops that moved reference while still holding the
lock. Only after that drop may it inspect the slot's strong count: one remaining owner means the slot
is the sole owner and the current payload may unload. Acquire clones the same identity under the same
lock, so a new lease cannot appear between the final-drop test and unload.

Inspecting `strong_count` without consuming the dropping lease under the lock is forbidden. Two
concurrent drops could otherwise both observe the other lease before either Rust field destructor
runs, both decide they are non-final, and leave a loaded payload with zero leases.

This removes `next_residency_token`, `ResourceRuntimeSlot::residency_token`, and the manual
`ResourceRuntimeSlot::ref_count` completely. `ResourceManager::ref_count` becomes a diagnostic view
of strong owners minus the slot owner. `Arc` supplies its own overflow safety. The design adds one
small allocation per payload incarnation and one atomic clone/drop per lease, while eliminating the
per-lease integer mutation and release-time payload downcast. This is a correctness trade, not a
claimed speedup; allocation and timing must be measured after managed validation recovers.

The hard cut remains internal to the existing public `ResourceLease<TData>` shape. No integer token,
manual lease counter, or compatibility path survives.

### Projection identity

An immutable generation's in-process correctness identity is its `Arc` allocation. Existing UI image
cache code already uses `Arc::ptr_eq` for exactly this purpose. Project asset aggregation and any
other cache that currently persists only `sequence()` must migrate to retain the source generation
handle, or an explicit opaque generation identity derived from it.

Readiness consumers need a per-row object identity, not a hash or a wrapping revision. The render
residency ticket hard cut should carry an opaque cloneable readiness-row identity and stop being
`Copy`; `seed.matches` must compare that identity. Global readiness sequence and dependency revision
may remain only as named diagnostic counters during migration, then be removed from correctness
keys. Hash fingerprints remain accelerators and never semantic equality.

This cross-crate slice is not authorized in the lease patch. It changes Asset and Graphics consumers
and must follow the current readiness profile plus exact ownership transfer.

### Event ordering

Event order is not object identity. The bounded log needs an explicit checked order protocol and a
defined terminal exhaustion state or a reservation that is guaranteed before authority mutation.
Because events are currently published after the Resource authority commits, adding a fallible
increment at publication time would lose the event for an already-committed mutation. Because
`subscribe` and receive take the event lock independently, reserving a range without representing
pending reservations would create false gaps.

The event-stream slice therefore requires a separate atomicity plan covering sequence reservation,
commit lock order, pending visibility, disconnect/exhaustion projection, and receiver gaps. It is not
silently folded into the lease change.

## Execution and tests

### I0: lease object identity

1. Add a structural RED requiring lease-identity `Arc::ptr_eq`, locked identity consumption, and
   forbidding `residency_token`, `next_residency_token`, and the manual slot ref-count.
2. Hard-cut `ResourceLease` release to move its exact incarnation identity into the manager.
3. Remove the token allocator and manual count from Resource authority/runtime slot/commit paths.
4. Preserve behavior where an old lease cannot evict a remove-and-re-registered payload.
5. Add a direct same-resource replacement case proving the old lease cannot decrement or unload the
   new incarnation.
6. Add a concurrent final-drop case proving exactly the last current lease unloads the payload.

I0 is correctness work, not a latency claim. Static source guards, rustfmt, focused tests and the full
`zr_resource` suite are required. Managed Cargo is deferred while the canonical sccache lifecycle
Failure is open; source may advance, but the slice is not accepted or committed without terminal
GREEN.

### I1: projection and render identity

1. Inventory and migrate every scalar generation equality consumer.
2. Add opaque generation/row identities and RED cases that force diagnostic counters to repeat while
   object identity remains distinct.
3. Hard-cut render tickets and asset projection caches; no numeric compatibility overload remains.
4. Run the existing same-source ResourceManagement/readiness paired profile before and after the
   change. Report p50/p95/MAD, allocations/bytes/peak live, affected closure, edge visits and exact
   source hashes. Obtain external RSS/power evidence separately; do not infer it.

The whole current consumer union and exact execution gate are now locked in
`2026-08-31-m1-resource-generation-object-identity-consumer-preflight.md`. The review found one
additional authority defect: management and readiness getters acquire the Resource read lock
separately, so paired Render consumers can observe mixed commit eras. I1 therefore starts with a
single-lock `ResourceProjectionSnapshot`, then publishes opaque generation/row identities to Asset,
Render, resource-streamer and Editor owners. `ResourceManagementPage` and
`ResourceMutationReceipt` are included; project catalog already has an Arc identity and must stop
degrading it to `.sequence()`. The preflight is plan evidence only. Production remains held behind
the unchanged current-source profile and exact cross-crate ownership.

### I2: event order

1. Write the event reservation/atomicity preflight before code.
2. RED covers max-order publication, a receiver waiting across reservation/commit, gap detection,
   coalescing, capacity eviction and disconnect.
3. Preserve bounded memory and O(1) recent-sequence lookup. Never replace checked ordering with a
   larger wrapping integer and call it fixed.

The required preflight is now complete in
`2026-08-31-m1-resource-event-order-exhaustion-atomicity-preflight.md`. It locks a non-mutating
publish permit under `commit_serial`, `Option<u64>` terminal order, typed receiver exhaustion,
optional gap successor, and `Arc/Weak` publisher lifetime. This is design evidence only: no I2
production source has changed, and implementation remains behind the current same-source profile.

### I3: durable I/O artifact identity

1. Replace both durable-I/O `fetch_add` allocators with one private checked nonzero sequence
   authority. `u64::MAX` is issued exactly once, zero is terminal, and there is no reset path.
2. Preserve `create_new`, WAL ordering, owner locking and fail-closed recovery. Exhaustion is typed;
   it is not projected as a collision loop or a wrapped identifier.
3. Hard-cut durable transaction IDs to include the full canonical journal-owner digest, PID and
   nonzero sequence. Bump the journal wire to version 7 and reject version 6/two-component IDs.
4. RED covers the concurrent terminal boundary, final-candidate collision, cross-owner target
   sharing, old-wire rejection and proof that one journal owner cannot delete another owner's
   artifacts.

The whole current atomic-file and transaction path was reviewed before selecting this design. The
owner digest partitions target-adjacent artifacts for distinct journal directories; it does not
replace `PathIdentity` or introduce a global target lock. Full reference, filename-budget,
complexity, validation and ownership evidence is recorded in
`2026-08-31-m1-durable-io-artifact-identity-exhaustion-preflight.md`. I3 is design evidence only and
remains behind the current-source profile.

## Acceptance state

### I0 source result

I0 is source-complete on the reviewed slice. The final production/test fingerprints are:

- `lease.rs` `7bca2f6bdc4d38cc8dce7e3b3792eaafdc0800593681744f4854df5aefd69d19`;
- `resource_manager.rs` `d3c48ce11567a86b88a845175cae3cf2a55636eb80bfe6d192efa9c3628559c5`;
- `runtime_slot.rs` `19f162a0ec80e15b7a99366124cb155fc75abc1332500f7d883e5c3535550448`;
- `lease_ops.rs` `f713a8af183fadadfc65013b7223c792026937c6698ef0752779a7ab3475f624`;
- `commit.rs` `10ce59851595e975b01b7a6b9f24451a1f4f4c5ec0fcff94e2c7100375eb38e5`;
- transaction tests `90d8c172bd9eb9df26bd8b421e783ec80b16fe01e9524164edbd1798ce0adacd`;
- crate static tests `8e4b86524387ae737988060d43b5ad555255ba50643c8dfd3aab0b5622ee9553`.

The structural RED first observed the old token/counter owners and no object identity. It is now
GREEN: the seven production owners contain no `residency_token`, `next_residency_token`, slot
`ref_count` field, or `slot.ref_count` mutation; the release path contains identity `Arc::ptr_eq`,
locked `drop(lease_identity)`, and the sole-slot-owner check. Rustfmt parsed all seven changed Rust
owners with edition 2021 and scoped `git diff --check` is clean. Test owners remain below the 800-line
structure threshold at 771 and 739 lines.

Behavior coverage now includes direct replacement, remove/re-register, and two concurrent final
lease drops. Managed Rust execution is still pending because the canonical sccache temporary-path
Failure rejected R5 and foreign Cargo/rustc jobs currently occupy the shared compiler window. No
Cargo process was started for I0 after that rejection.

Only I0 production editing is complete under this audit. I1 has completed whole-consumer and paired
snapshot architecture preflight; its pre-change ResourceManagement profile is now available, but
cross-crate consumer work remains behind legal Asset/Render/Editor ownership. I2 has completed its
atomicity preflight; I3 has completed its durable-I/O artifact identity, exhaustion and journal-owner
namespace preflight. The shared profile gate is satisfied for Resource support implementation, not
for consumer ownership or post-change acceptance. M1 remains
`source infrastructure materially advanced / milestone_not_accepted`. No commit, WeCom notification,
performance gain, power claim, bottleneck-eliminated claim, or optimal-scale claim follows from this
document.
