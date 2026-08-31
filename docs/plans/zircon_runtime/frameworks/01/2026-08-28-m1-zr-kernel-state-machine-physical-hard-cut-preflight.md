# Frameworks01 M1 `zr_kernel::state_machine` physical hard-cut preflight (2026-08-28)

## Status

- `current_source_review_complete`
- `current_source_fingerprint_revalidated_2026_08_29`
- `reference_engine_review_complete`
- `dependency_direction_green_std_only`
- `existing_algorithm_profile_reviewed`
- `first_real_zr_kernel_domain_selected`
- `state_kernel_source_guard_green_6_of_6`
- `product_state_service_not_admitted_runtime48_owned`
- `physical_move_not_started`
- `ordered_after_zr_contracts_manifest_integration`
- `shared_manifest_owner_blocked_by_shader06`
- `manifest_blocker_refreshed_2026_08_30`
- `managed_cargo_not_run`
- `milestone_not_accepted`
- `service_commit_not_requested`
- `wecom_not_sent`
- `new_performance_claims_not_admitted`

## Decision

The first physical `zr_kernel` domain will be the runtime-wide state machine currently at
`zircon_runtime/src/core/runtime/state_machine`. This is not an empty crate placeholder and not a
mechanical move of all `core/runtime`: it is a dependency-complete execution owner already selected
by the earlier Frameworks01 declaration/behavior hard cut.

The current 12-file owner is self-contained and depends only on `std`. It owns:

- `StateSpec`, `State`, `NextState`, transition labels, and the latest transition receipt;
- the type-indexed state registry and per-type machines;
- exit -> transition -> enter hook indexing and deferred dispatch;
- same-state suppression and explicit same-state admission.

It does not depend on Resource, Diagnostics, Asset, Scene, Graphics, UI, manager resolution, the
Runtime facade, or `zr_contracts`. A first `zr_kernel` crate may therefore start with no external
dependency; later Random/lifecycle/scheduler slices may add the already-approved layer-0a
dependencies when their complete physical owners move.

This decision is an implementation-owner relocation, not admission of the current direct state API
as a product service. The current production-consumer scan finds only the
`CoreRuntime -> CoreHandle -> StateRegistry` forwarding chain; App, Editor, plugin SDK, and ZrVM
production code do not consume it. `docs/zircon_runtime/core/state.md` therefore remains authoritative:
the API is a test-only migration baseline until Runtime48 replaces direct mutation, the synthetic
repeated-init DTO, and ownerless permanent hooks with a scheduled service, typed receipts, and
lifecycle-aware subscriptions. Frameworks01 must not invent that Runtime48 contract during the
physical move or describe crate placement as feature completeness.

## Current-source fingerprint

The current-source review was refreshed at HEAD `11cac2d08a891ee92dcc206fd84a2d15f9e1a3f4` and
coordinator baseline epoch 537. The path-sorted manifest uses
`repository-relative-path<TAB>bytes<TAB>lines<TAB>lowercase-file-sha256`; the rows are joined with LF,
without a terminal LF, and the resulting bytes are SHA-256 hashed:

| Files | Lines | Bytes | Manifest SHA-256 |
| ---: | ---: | ---: | --- |
| 12 | 517 | 14,817 | `7683ad0469a95b1656382d02626ea597f6397e53a95adb59c119e0611bdcf0ba` |

Current-source ownership-matrix request `6400f68d4ff34fbe903c99be9d9fc85f` confirmed that all 12 path
hashes still match the active Frameworks01 r12 attribution at epoch 537. The attribution baseline is
nevertheless stale and there is no live source lease. Before any move, the implementing batch must
reclaim and re-attribute every source path and must regenerate this manifest if any hash drifts.

The 2026-08-29 fresh read-only recomputation produced the same 12 files, 517 lines, 14,817 bytes,
per-file hashes, and manifest SHA-256 `7683ad0469a95b1656382d02626ea597f6397e53a95adb59c119e0611bdcf0ba`.
The focused state-kernel source guard then passed `6/6` in 33.949 seconds: the implementation owner
remains singular, retired framework-owner consumption remains absent, and transition observation
remains one bounded latest event. This refresh changes no source, lease, admission order, Cargo
state, performance claim, or milestone status.

## Runtime integration boundary

The direct implementation consumers are bounded:

- `core/runtime/state/core_runtime_state.rs` owns the mutex-protected `StateRegistry` field;
- `core/runtime/handle/states.rs` owns lock recovery, mutation orchestration, and hook dispatch
  outside the registry lock;
- `core/runtime/runtime.rs` owns the `CoreRuntime` forwarding methods.

`core/mod.rs`, `prelude.rs`, and Runtime tests consume the public product surface and may continue
through the curated Runtime projection. App, Editor, and plugins must not gain direct `zr_kernel`
dependencies.

The target physical API is:

```text
zr_kernel::state_machine
  public: StateSpec / State / NextState / labels / StateTransitionEvent
  hidden workspace assembly: StateRegistry / StateTransitionDispatch

zircon_runtime::core::runtime::state_machine
  explicit curated projection of the public list only
```

Cross-crate visibility must not turn registry mutation into a product API. `StateRegistry` and
`StateTransitionDispatch` may become public only through a `#[doc(hidden)] assembly` path used by
Runtime's state owner and handle orchestration; the Runtime facade must not re-export `assembly`.
The old implementation children are deleted in the same batch. A forwarding implementation module,
wildcard export, copied registry, compatibility alias, or dual hook index is forbidden.

## Algorithm and reference review

No algorithm change is approved by this physical move. The earlier state retention review already
identified and removed the structural defect where every transition was appended forever and every
history query cloned the complete vector. Its measured old query path grew from 2.829 us / 40 KiB at
1,000 transitions to 45.498 ms / 40 MiB at 1,000,000 transitions. The current owner retains one
latest event per state type: post-cut transition cost was 95.9-168.5 ns across the measured sweep,
latest query 30-40 ns, and retained payload 40 B in the standalone model. Expected complexity is
O(1) record, O(1) latest query, and O(1) retained transition payload per registered state type.

Unreal's StateTree keeps pending transition work inside instance storage rather than a process-global
history owner. `StateTreeInstanceData.cpp::AddTransitionRequest` caps the queue at 32,
`ResetTransitionRequests` clears it after processing, and `FStateTreeInstanceStorage::Reset` clears
execution state, temporary instances, owned events, and transition requests together. This is
routing evidence for bounded instance lifecycle ownership, not an API copy: Zircon's registry is
runtime-wide rather than a StateTree instance.

Bevy's `bevy_state/src/app.rs` initializes and overwrites state by writing a typed
`StateTransitionEvent` into the `StateTransition` schedule/message stream; enter, transition, and
exit work is schedule-owned. Zircon's repeated `init_state` currently returns an unrecorded synthetic
DTO and its hooks have no unsubscribe token, owner generation, quiescence, or unload boundary. Those
are known product-service gaps, not evidence for optimizing the current callback hash maps. Fyrox
likewise keeps animation machine execution state inside the machine owner rather than a global
unbounded history store. Together the references support one bounded kernel execution owner and a
separate future diagnostics/replay owner, while leaving Runtime48 responsible for scheduled product
authority and subscription lifetime.

The current `TypeId -> Box<dyn Any>` lookup and nested hook hash maps remain unmodified. Existing
profiles did not identify them as the bottleneck. Any change to storage, dispatch ordering,
allocation reuse, or lock strategy requires a new current-source product workload profile first.
This preflight makes no new latency, throughput, allocation, power, parity, bottleneck-removal, or
optimality claim.

The refreshed review reread all 12 implementation files together with the `CoreRuntime`/`CoreHandle`
forwarding chain, tests, local state documentation, Bevy application-state behavior, and Unreal
StateTree references. It found no new Frameworks01 algorithm or correctness defect. `insert_state`
deliberately models installation/overwrite rather than a transition exit, hook lists are snapshotted
under the registry lock and dispatched after unlock in deterministic exit-transition-enter order,
and only the latest transition is retained per state type in O(1) space. The remaining direct
mutation, synthetic repeated-init receipt, and permanent hook-lifetime gaps belong to Runtime48.
Without a current product-workload profile, changing these data structures would be an unjustified
optimization rather than a physical ownership hard cut.

The physical-cut acceptance language must therefore stay narrow: it may prove source uniqueness,
dependency direction, bounded latest-event retention, and preservation of the test-only baseline.
It must not claim a runtime state MVP, plugin-safe hooks, scheduled transition authority, or Unreal
StateTree parity; those require the Runtime48 service and a real product workload before profiling or
optimization is admissible.

## Atomic implementation manifest

After `zr_contracts` workspace integration is complete, the implementing batch must include:

1. New crate paths: `zircon_runtime/crates/zr_kernel/Cargo.toml`, `src/lib.rs`, and the complete
   12-file `src/state_machine` owner, with `publish = false` and `#![forbid(unsafe_code)]`.
2. All 12 old state-machine paths, leaving only the explicit Runtime `state_machine/mod.rs`
   projection and no old implementation child.
3. Direct implementation consumers:
   `core/runtime/state/core_runtime_state.rs`, `core/runtime/handle/states.rs`, and
   `core/runtime/runtime.rs`.
4. Product projections and seals: `core/runtime/mod.rs`, `core/mod.rs`, `prelude.rs`, state tests,
   the existing state owner guard, a new physical crate boundary guard, and module documentation.
5. Shared integration files: root `Cargo.toml`, `zircon_runtime/Cargo.toml`, and `Cargo.lock`.

The batch must prove zero old implementation children, zero Runtime implementation imports through
the product facade, zero direct App/Editor/plugin dependency on `zr_kernel`, and no reverse
dependency from `zr_kernel` to `zircon_runtime`.

## Admission order and current blocker

The parent plan fixes layer-0 order as
`zr_math/zr_resource -> zr_contracts -> zr_kernel -> zr_diagnostics`. Frameworks01 has implemented
the `zr_contracts::random` physical source move, but its workspace/Runtime/lock integration is still
blocked by Shader06's executable ownership of the three shared manifest blobs. Transfer preview
`d3dbc7bade17460e953333b1a1fa2826` at epoch 527 remains ineligible for that reason, and the current
lock hash differs from Shader06's recorded source hash.

Consequently this preflight does not create or partially wire `zr_kernel`. The legal next actions
are:

1. receive a stable Shader06 manifest commit/release and take a fresh exact transfer preview;
2. atomically finish `zr_contracts` manifest/lock wiring and managed validation;
3. refresh the 12-file state-machine fingerprint, leases, ownership matrix, and consumer manifest;
4. capture the physical-boundary TDD RED, execute the no-compat move, then run focused and managed
   `zr_kernel`/Runtime gates;
5. only after review and accepted milestone evidence may the coordinator commit and notify WeCom.

This advances the next infrastructure slice without treating a validation queue as the only work
item and without breaking the parent plan's dependency order.

## 2026-08-30 admission refresh

The prerequisite has not changed at current
`HEAD=cc5cadbd597c3707954ebd6109fad0fd5643a152`. Fresh `zr_contracts` manifest transfer-preview
request `076e9a0ce4104c9eb79eba3cd6a028a0` at baseline epoch 573 remains ineligible solely because
Shader06 session `01a019a5-b15f-7461-a1b0-ce4b6aa8e710` is executable. Its fingerprint is
`bc4b5a329c564ea0c7c9db1938aab03d0f64863f40cdd339ab3724b3c109c4e2`. Root `Cargo.toml` still
matches the attributed source hash, while `Cargo.lock` and `zircon_runtime/Cargo.toml` have drifted to
`70a18867be9385416425dbbda9b9adb5692fca2bee887418a224002e58eb1217` and
`531f8a844f198b99e0f28991ab6eb30ee816c90f8d105c5bc6edf709127c17ff` respectively.

Frameworks01 therefore did not create `zr_kernel`, partially wire either crate, claim the mixed
manifest blobs, regenerate the lock, or reuse an older preview. Current-source review of the 12-file
state machine and the Random authority/registry chain found no new Frameworks01 algorithm defect that
would justify changing the physical-cut manifest: transition retention remains O(1), hook dispatch is
performed after the Runtime registry lock is released, and Random checkpoint/reseed lock ordering is
consistent. The legal order remains `zr_contracts` manifest integration and managed validation first,
then a refreshed exact state-machine fingerprint and the atomic `zr_kernel` move.

The same-source combined Random/State boundary run on 2026-08-30 executed 20 tests in 87.912 seconds:
all six State owner tests and thirteen of fourteen Random boundary tests passed. The only RED was the
intentional physical-owner assertion that `zircon_runtime/crates/zr_contracts` must be a root workspace
member. This evidence narrows the blocker to the shared manifest integration; it does not make
`zr_kernel` admissible before `zr_contracts`, and it is not managed Cargo or milestone acceptance.
