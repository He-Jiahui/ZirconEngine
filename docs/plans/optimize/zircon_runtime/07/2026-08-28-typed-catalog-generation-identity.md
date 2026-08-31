---
title: Runtime07 Typed Catalog Generation Identity
category: zircon_runtime
report_id: Runtime07-typed-catalog-generation-identity-2026-08-28
date: 2026-08-28
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Typed Catalog Generation Identity

## Scope

This M1 foundation hard-cuts the runtime plugin catalog's published generation from a bare `u64`
into an opaque non-zero `PluginCatalogGeneration`. It removes value-cloning of the mutable catalog
owner, seals published state behind `Arc<RuntimePluginCatalogSnapshot>`, and adds one compare-exchange
publication owner. This slice does not claim that App bootstrap, dynamic sessions, and native/script
backends already consume this authority, or that the package solver, activation transaction, and
generation lease are complete.

## Current-Source Review

- `registration/reports.rs` and `registration/update.rs` are the two generation producers. Both
  currently use `saturating_add(1)`, so a successful publish at `u64::MAX` can reuse the previous
  identity.
- Projection metrics, update outcomes, compiled project plans, the project-plan cache, module
  composition identity, and the dynamic-session receipt all consume the same bare integer.
- Candidate rows and the derived projection are built before the authoritative fields are changed.
  Safe Rust `&mut self` already makes one catalog publication logically atomic; the hard cut must
  preserve rejected-candidate rollback and cache retention.
- `RuntimePluginCatalog::clone` copies the current generation and immutable projection but creates
  a separate mutable plan cache. Mutating both clones can therefore publish equal generation values
  for different states. No current production caller requires value-cloning this catalog owner.
- A publication API that accepts an independently supplied expected handle and candidate handle is
  insufficient even when generations are consecutive: a caller could pair the current revision-1
  handle with revision 2 derived from another catalog lineage. The prepared-generation type must
  retain the exact base `Arc` used to stage it and be the only value consumed by publication.
- Copying snapshot rows into a mutable catalog and then calling the existing lazy update API would
  clone every production DTO twice. Snapshot staging therefore builds the identity-indexed candidate
  rows directly and owns exactly one copied row set.

## Reference Alignment

Unreal's `FPluginManager` keeps a mutable `Name -> [PluginVersions]` discovery table, separately
tracks enabled and mounted state, and emits separate plugin/module/content events. Zircon must not
copy that split truth. The target is an immutable generation containing selected candidate
identity, resolved enabled closure, deterministic order, contribution plan, and diagnostics, with
one post-publication change event. This slice establishes the typed revision needed by that target;
it does not treat Unreal's container order or mutable `bEnabled` flags as a generation contract.

Primary evidence:

- `dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.h`
- `dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/Interfaces/IPluginManager.h`
- `dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleManager.h`

## Design

- `PluginCatalogGeneration` wraps `NonZeroU64`, starts at revision 1, exposes only `get()` and an
  internal checked successor, and implements value/display/hash/order traits without allocation.
- `RuntimePluginCatalog` remains the only revision owner. `Clone` is removed rather than sharing or
  duplicating a mutable revision clock.
- `RuntimePluginCatalogSnapshot` consumes a completed catalog and is held through `Arc`. The
  snapshot exposes no clone-back-to-owner or mutation API, so consumer clones share one sealed
  generation instead of branching its rows, projection, or project-plan cache.
- `RuntimePluginCatalogCandidate` is staged only from `Arc<RuntimePluginCatalogSnapshot>` and keeps
  that exact base handle. It owns one identity-indexed copy of the cold-path registration rows,
  validates and builds derived state once, then yields a type-state
  `RuntimePluginCatalogPreparedGeneration` whose lineage fields are private.
- `RuntimePluginCatalogAuthority` owns `ArcSwap<RuntimePluginCatalogSnapshot>`. Readers acquire an
  immutable handle without a read lock. Publication consumes a prepared generation and performs one
  pointer-identity compare-exchange; stale candidates return a typed conflict and cannot overwrite
  the winner. Callers cannot supply or recombine the CAS operands, and cannot construct a second
  authority from an existing snapshot handle.
- `RuntimePluginBridgeLifecycleState` is the first hard-cut consumer of the snapshot handle. Its
  existing application/runtime observer clones share the exact catalog generation allocation.
- Initial construction builds revision 1 once. Each accepted batch publishes exactly one checked
  successor. A rejected batch preserves rows, projection, generation, and plan cache.
- Revision exhaustion becomes a rejected update with a structured diagnostic. It must not reuse
  `u64::MAX`, partially publish candidate rows, replace the projection, or clear the last-good cache.
- Public metrics, plans, outcomes, and module-composition provenance carry the typed value. Only the
  fixed-layout dynamic receipt converts it to `u64` at the ABI boundary.

## Complexity And Performance Contract

The newtype is an `O(1)` zero-allocation identity operation. `NonZeroU64` keeps both
`PluginCatalogGeneration` and `Option<PluginCatalogGeneration>` at one machine word. Snapshot
acquisition and publication are `O(1)`; candidate preparation remains `O(P + F + D)` for package
rows, feature rows, and derived projection work. Candidate rows are copied/indexed once per staging
generation, never per frame or per consumer. This is a correctness and ownership repair, so no CPU,
power, or cross-engine speedup is claimed.

Synthetic release microbenchmarks on the current Windows host (11 samples, median, simple 40-byte
row rather than the full production DTO) measured candidate clone + identity indexing at 0.273 ms
for 1,000 rows, 8.418 ms for 10,000 rows, and 64.224 ms for 100,000 rows. This confirms linear cold-
path scaling and also shows why generation construction must not enter a frame path. A separate
1,000,000-iteration single-thread sample measured `ArcSwap::load_full` at 12.03 ns/op and an
uncontended `RwLock<Arc<_>>` read + clone at 11.02 ns/op. `ArcSwap` is therefore not claimed as a
single-thread micro-optimization; it is selected for lock-free readers and exact CAS publication.
These synthetic numbers are not product, ETW, power, or reference-engine acceptance evidence.

Acceptance evidence must prove:

- one accepted batch advances exactly one typed revision;
- rejected and exhausted candidates preserve the last-good state and cache;
- projection metrics, update outcome, compiled plan, composition identity, and ABI receipt agree;
- raw `catalog_generation: u64`, `saturating_add(1)`, and public generation arithmetic are absent
  from the owned catalog path;
- layout remains one word for the value and its `Option`.

## Follow-Up Boundary

M1 now has the typed snapshot, lineage-safe staging type, and one atomic publication owner. It still
needs selected package/version/source identity, resolved dependency closure, contribution plan,
transition leases, and retirement. Multiple runtime, editor, bootstrap, dynamic-session, and native
loader catalog builders remain a higher-level authority gap until they consume this publication.
This slice must not be used to claim P1-1 or P1-2 closed.

## Current Status

Completed implementation:

- added one-word `PluginCatalogGeneration(NonZeroU64)` with revision 1, checked successor,
  display/value projection, and value/`Option` layout tests;
- hard-cut catalog projection metrics, update outcomes, compiled plans, plan-cache keys, module
  composition identity, and the dynamic receipt producer to the typed generation;
- removed mutable `RuntimePluginCatalog` value cloning and replaced terminal saturation with a
  rejected update that retains the last-good rows, projection, and project-plan cache;
- updated catalog consumers and public test fixtures so raw generation arithmetic cannot re-enter
  the Rust runtime contract. The fixed-layout dynamic ABI remains the only `u64` conversion point.
- sealed bridge-lifecycle ownership behind a shared snapshot handle, repairing the remaining
  `Clone` derivation without reintroducing mutable catalog-owner copies.
- added one `ArcSwap` catalog authority, exact-base type-state candidate preparation, stale-writer
  conflict rejection, and single-copy identity-indexed staging rows. Empty or invalid candidates
  cannot consume a generation; a prepared generation is not counted as published before CAS wins.
- retained the dynamic session's locally built catalog as one shared snapshot for the full session
  lifetime, alongside its compiled project plan, with a typed generation provenance assertion. This
  removes the previous build-plan-then-drop-catalog lifetime break but does not yet make dynamic
  sessions consume the process-wide catalog authority.

Completed local evidence:

- source contract: `5/5` tests passed;
- isolated Rust value/layout test compiled and passed `1/1` with its executable under
  the approved `D:\cargo-targets\zircon-engine` tree;
- direct compilation of the real generation and `CandidateRows` modules passed `2/2`, including the
  1,024-row exactly-once clone/index assertion;
- the real production `publication.rs` compiled against a minimal parent-module harness and the
  workspace's existing `arc-swap 1.9.2` rlib, confirming CAS/Guard/type-state signatures; a behavior
  probe also executed winner then stale publication and confirmed the stale base reports generation
  1 expected / generation 2 observed without replacing the winner;
- synthetic release performance evidence is recorded above; it is a complexity guard, not product
  or cross-engine acceptance;
- exact-file Rust formatting: passed with Rust 2021 edition;
- owned-path whitespace/diff check: passed;
- source scan: no raw `catalog_generation: u64`, catalog-generation saturation, or catalog owner
  clone remains in the owned plugin catalog/composition path.

Managed Windows validation was submitted earlier for `zircon_runtime`, `--no-default-features`,
`core-min`, library tests. The coordinator allocated
`D:\cargo-targets\zircon-engine\pool\f9fef644bf8e441a49ad1c139495499657f126cd246ffca80d13868db535561d`,
but Cargo stopped before compilation because the shared dirty `Cargo.toml` set requires a
`Cargo.lock` update while the validator correctly enforces `--locked`. `Cargo.lock` and 14 tracked
manifests already contain foreign changes, so this Session did not overwrite them with
`-NoLocked`. Managed build/test evidence therefore remains pending rather than failed source
evidence.

The current direct `cargo check --locked` attempt still stopped before compilation for the same
shared lockfile drift. Stable and installed nightly Cargo could not redirect the lockfile to D using
the available command/config surface. The Session did not temporarily rewrite the dirty shared lock
or delete foreign data. Managed workspace validation, milestone closeout, Git commit, and WeCom
notification therefore remain pending.
