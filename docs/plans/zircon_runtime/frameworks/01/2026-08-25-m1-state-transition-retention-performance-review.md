# M1 State Transition Retention Performance Review

## Status

- Plan owner: Frameworks01
- Session: `frameworks01-shader-invocation-hard-cut-r12-1b2684b4-20260825`
- State: `source_implemented / static_and_profile_validation_green / managed_cargo_foreign_blocked`
- Base HEAD: `1b2684b40ae3eba7abfcdfae3fe7e341b4906ec8`
- Scope: runtime-wide state-machine transition retention and query semantics only
- Excluded: animation graph state machines, scene-local scheduling, EventBus redesign, persistence/replay history

本记录遵循“先复核当前模块、再参考成熟引擎、再量测、最后实现”的顺序。生产 hard cut 已在基线报告
落盘后实施，但不把独立算法微基准当作 Runtime 产品编译、功耗或整机帧耗时结论。

## Current-Source Finding

Current source has one structural retention defect:

- `StateMachine<T>` stores every transition in `Vec<StateTransitionEvent<T>>` for its entire lifetime;
- every `record_transition` appends one clone;
- `state_transition_events()` clones the complete retained history through
  `StateMachine -> StateRegistry -> CoreHandle -> CoreRuntime`;
- a production source scan found no consumer outside these facade definitions; current tests only assert the first
  one or two retained events;
- hooks already provide ordered `OnExit -> OnTransition -> OnEnter` delivery, and each mutation API already returns
  the accepted transition event directly. Permanent history is therefore not required for current execution behavior.

Current input hashes:

- `state_machine/machine.rs`: baseline `a0894b81fe9a7ad0017ff7f95ec94b9181cbd65333d4b41b8786f9445cbc24dc`,
  post-cut `9de9e7a16d045e66bc7c477d3937ef3a9db9bd00e1d691b959d8d22cecd32b65`;
- `state_machine/registry.rs`: baseline `ac24cde40659a3aa46063f72cac44360506f906a86a8bf1e0a190472426c2770`,
  post-cut `e503172da046335ae2620e55b5f93d61c7d867b425399f45800eec9d7a4951f5`;
- `core/runtime/runtime.rs`: post-cut `32f1e225e54d5ed236d3b7572c8a29aafc8a74c2703024d025bf8dfe34888221`;
- `core/runtime/handle/states.rs`: post-cut `81a7bd38e276f291da9c75e115d9b134d5d56a4ce88f76ee57705e46ad64c6bc`;
- `tests/state.rs`: post-cut `ca5a000394e7cc755d821b2485e37e5e2cc468aa12dbd1b972e58b520ce7f695`.

Complexity of the current design after `N` accepted transitions:

- record: amortized `O(1)`, retained payload `O(N)`;
- history query: `O(N)` time plus `O(N)` temporary allocation;
- one query after every transition: cumulative `O(N^2)` copied events;
- no query at all: memory still grows without bound.

This is an owner/lifetime error, not a `Vec` implementation detail. Reserving capacity or changing clone syntax would
not fix the asymptotic behavior.

## Reference-Engine Review

### Unreal Engine

`dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimNode_StateMachine.cpp` keeps transition work as
bounded execution state rather than a permanent event archive:

- `HandledTransitionEvents.Reset()` at the update boundary;
- processed queued events are removed with `RemoveAt(..., EAllowShrinking::No)`;
- `QueuedTransitionEvents.Reset()` clears the per-update queue;
- inactive entries are removed from `ActiveTransitionArray` without shrinking the reusable allocation.

Unreal's animation machine is not the same public contract as Zircon's runtime-wide state registry, so these lines are
routing evidence rather than a copy target. The applicable principle is that execution-owned transition working sets
have an explicit lifetime and reuse bounded storage; permanent diagnostics/replay history belongs to a separate owner.

### Bevy

`dev/bevy/crates/bevy_ecs/src/message/messages.rs` states that buffers grow indefinitely if `Messages::update()` is
not called. Its normal path uses two sequences: `update()` swaps the buffers and clears the oldest one once per update.
State transitions are written as `StateTransitionEvent<S>` messages and consumed by readers, rather than copied from
an ever-growing vector in each state machine.

The applicable Zircon rule is the same: transition delivery and optional replay/persistence are separate concerns.
Framework/kernel state ownership must not silently become an unbounded history service.

## Profile Evidence

Because current shared Runtime compilation is still stopped by foreign RHI current source, the pre-change baseline was
captured with a standalone Rust model that preserves the exact retention operations: append one cloned event to a
default-growing `Vec`, then clone the entire vector for each query. The comparison prototype overwrites one
`Option<Event>`. Both paths were compiled by `rustc -C opt-level=3 -C target-cpu=native`; each timing is the median of
seven trials and uses `black_box` to prevent constant folding.

Environment:

- Windows 11 Pro `10.0.26200`, x86_64;
- AMD Ryzen 7 5800H, 8 cores / 16 logical processors;
- Rust `1.94.1`, LLVM `21.1.8`;
- source and all output artifacts are under `D:\zircon-frameworks01-r12-state-profile`;
- benchmark source SHA-256: `ab4bee67b7baac41e89ec114fc55916f63b626306a3af7ab664d0030d399ca01`;
- sampled CSV SHA-256: `228150d8101884471eff56450d15c1c832c69b2045ef944437f1e62f5e21ec16`;
- process telemetry SHA-256: `2fb097c5bf35c5c183b741b54cbecfba6b4de21c8c427a7a2f7e1cb49ede5ff2`.

| Accepted transitions | Current query median | Current retained payload | Latest query median | Latest retained payload |
| ---: | ---: | ---: | ---: | ---: |
| 1,000 | 2.829 us | 40 KiB | about 1 ns | 40 B |
| 10,000 | 18.194 us | 640 KiB | about 1 ns | 40 B |
| 100,000 | 4.434 ms | 5 MiB | about 1 ns | 40 B |
| 1,000,000 | 45.498 ms | 40 MiB | about 1 ns | 40 B |

The 50 ms Windows process-counter sampling run completed in 9.057 s with 48 samples, 85,721,088 B peak working set,
84,074,496 B peak private bytes, and 4.45 CPU seconds. These process values include both algorithms and harness
overhead; the retained-payload column is the algorithm-specific capacity measurement.

Windows Performance Recorder CPU sampling was attempted but rejected by host policy with `0xc5585011`; `wpr -status`
confirmed that no recording remained active. Therefore this record does not claim sampled call-stack attribution.
The controlled size sweep and source inspection are sufficient to identify the linear clone/retention bottleneck, but
not to claim product frame-time or energy improvement. No energy counter was available, so power remains unmeasured.

After the source hard cut, a second optimized harness directly included the current production
`core/runtime/state_machine/mod.rs` and drove the real `StateRegistry` transition/query methods. Its source SHA-256 is
`d88c9939e0c3374849eb9e48e79b3c882d7bc9c3cfa0c8a39a002b317933f7fc`; sampled CSV SHA-256 is
`b4f3d8dd38861a3c80e9d5050c13f806a7e1b84db07c7e0b642ab4e629d36444`; process telemetry SHA-256 is
`4645e09ab82254bac7e01d4f269e88637fb1e2865210081c8c26e07415fb8dea`.

| Accepted transitions | Post-cut total transition median | Per transition | Latest query median | Retained payload |
| ---: | ---: | ---: | ---: | ---: |
| 1,000 | 95.9 us | 95.9 ns | 30 ns | 40 B |
| 10,000 | 1.041 ms | 104.1 ns | 36 ns | 40 B |
| 100,000 | 13.898 ms | 139.0 ns | 40 ns | 40 B |
| 1,000,000 | 168.466 ms | 168.5 ns | 34 ns | 40 B |

At one million transitions, the controlled query path changed from 45.498 ms to 34 ns (about 1.34 million times
lower), while retained event payload changed from 40 MiB of `Vec` capacity to 40 B (1,048,576 times lower). The exact
post-cut process run peaked at 5,210,112 B working set and 806,912 B private bytes, but it is a smaller harness than the
combined pre-change comparison binary, so those process peaks are reported separately and are not used as a direct
product-memory percentage claim.

## Hard-Cut Design

The production change replaced history ownership rather than adding a cap to the old API:

1. `StateMachine.events: Vec<_>` became `latest_event: Option<_>`.
2. Internal `events()` became `latest_event()` and the singular query propagates through `StateRegistry`.
3. Public `CoreHandle::state_transition_events` and `CoreRuntime::state_transition_events` were deleted.
4. `latest_state_transition<T>() -> Option<StateTransitionEvent<T>>` provides constant-space observation; the old
   plural method has no alias or wrapper.
5. Direct transition receipts and ordered hooks remain unchanged. Replay, audit and persistent history must subscribe to
   an explicit higher-level service in a later owner rather than expanding kernel retention again.

Expected production complexity after the cut is `O(1)` record, `O(1)` retained payload per registered state type, and
`O(1)` latest query without vector allocation. Hash-map lookup by `TypeId` remains unchanged and is not identified as
the bottleneck by this slice.

## Implementation And Acceptance Order

1. Completed: transferred all 12 cancelled r10 state-machine blobs with fingerprint
   `76694588ff58be014ca543fdd2dd2ac61435ac440219b5e5bb657a9061f3e65a`; transferred the stale structure guard with
   fingerprint `da01cebd1bc2cf6afb0f4c6c00aa814d906c78f05db41195d42bb9aa4615fc4a`.
2. Completed: TDD guard failed on the old `Vec` owner, then `6/6` state owner tests passed in 32.119 seconds after the cut.
3. Completed: singular latest-event hard cut and zero product plural-history consumers; no compatibility path.
4. Completed: direct-production-source D-drive benchmark proves constant retained payload and query latency across the
   1,000 to 1,000,000 transition sweep.
5. Completed: two Windows managed focused attempts (`da0685f619264f53a480a6d7ff639695` and
   `46c5552ac4134076ab4f0ae765f5fece`) proved that the workspace lock was stale before compilation. A D-drive shadow
   workspace resolved the same manifests offline; `Cargo.lock` now exactly matches that resolver output and has
   SHA-256 `f8df4d979bd86eb91e58df1031a828a65ca2de43de64a5362ad166ccaa8023de`. The scoped correction adds only the
   missing dependency-array entries for `cargo-zircon` and `zircon_runtime_interface`; it does not rewrite foreign
   lockfile changes.
6. Completed: the next managed state job `673d51016b2d4679842de468abca4ec0` crossed locked resolution and reached the
   lowest shared compile failure in `zircon_runtime_interface/build.rs`: E0106 on `slot_list` because its returned
   string slices were not tied to the input InterfaceSpec value. No Runtime state test binary was generated.
7. Completed under fixing Session `interface08-build-spec-lifetime-fix-r1-1b2684b4-20260825`: the signature binds
   `Vec<&'a str>` only to `spec: &'a serde_json::Value`, while `field: &str` keeps an independent lifetime. Current
   build-script SHA-256 is `36a43263fedaa235ab72b8a803760cd2751bb46c3d9a716c3681d406f90b4b4d`. A direct
   `rustc --test` of the real build script against D-drive managed-pool dependency artifacts is 5/5 GREEN in 0.10
   seconds; executable SHA-256 is
   `e7a2fa792ed95d7b0d319b0186a3ca75c253c93529275113cfbd49b0ddcee0a4`. The real managed
   job `7f8adbb03fdf4616a9d6d887045c74a2` ran the real managed
   `cargo build -p zircon_runtime_interface --locked`, which passed in 3 minutes 12 seconds and proved that the
   original E0106 no longer exists in the package build chain.
8. Completed: the Interface08 handoff returned as
   [`fixed-2026-08-25-interface-spec-slot-list-lifetime.md`](fixed-2026-08-25-interface-spec-slot-list-lifetime.md).
   Its managed filtered lib-test compiled no test binary because nine current-source test errors remained in foreign
   UI text-shape and Project GUID contracts; no InterfaceSpec generator regression was observed or bypassed.
9. Pending: the original Frameworks01 focused Runtime state gate was rerun as managed job
   `29a2e88837ab4890863b3b59ce7fd251` after the Interface job released. Both its build and test invocations crossed
   the repaired build script, then stopped at E0432 in
   `project/manifest_summary/summary.rs` while that file transiently imported `ProjectGuid` from `super`. The current
   file later changed back to `crate::project::ProjectGuid` with SHA-256
   `4302116d5634d08b1ec62156a281d1015b415e32562d9654fc8d328f60f15c4c`, but the coordinator ownership matrix reports
   the Project blob as `attribution_missing`, with no owner or live lease. Frameworks01 will rerun only after the
   Project06 atomic source stabilizes; it does not claim or rewrite that foreign blob.
   Both managed jobs are released with empty process trees; neither was cancelled.
10. Pending foreign-source convergence: the unowned `zircon_runtime_interface/src/project` tree continued changing
    after that compile. The observed snapshots grew from 71 files / 100,823 bytes to 74 files / 112,187 bytes and then
    to 75 files / 123,232 bytes; the latest deterministic tree SHA-256 is
    `e4c2c3da3b57c68d53f3f7ca04844c3f3b4b5ccb2641e6a7a46bec142799d12d`, with the latest write at
    `2026-08-25T13:49:57.2819771Z`. The tree digest hashes sorted `relative-path<TAB>file-sha256<LF>` rows. The current
    blob also cannot be adopted as a valid Project06 baseline merely because the transient import was repaired:
    `CanonicalDescriptorIdentity` serializes a raw `PathBuf` as a cross-process identity and relies on callers to
    perform physical filesystem resolution; `ProjectIdentity` lacks strict unknown-field rejection;
    `ProjectManifestSummary` remains directly constructible/deserializable through public fields; and the compatibility
    product state does not yet encode the planned Open/Copy/Migrate/Safe/Reject decisions. These are Project06
    wire/domain and authority defects, not Framework state defects. Frameworks01 will not claim, rewrite, or validate
    against that moving tree; a stable attributed Project06 atomic snapshot is the entry gate for the next managed
    state run.

Acceptance requires:

- retained transition payload remains constant from 1,000 through 1,000,000 transitions;
- latest query is independent of transition count and allocates no history vector;
- current production plural-history consumers and compatibility paths equal zero;
- hook ordering and same-state suppression behavior remain green;
- exact source hashes, managed validation job IDs and any foreign blocker are recorded before milestone status changes.

Static/source-profile acceptance is met; managed Cargo acceptance is not. Frameworks01 M1 therefore remains
`implementation_in_progress`, and this slice does not yet authorize a milestone commit or WeCom completion notice.
The Interface08 compiler failure is fixed and returned; the current compile frontier is the unowned Project06
current-source transition above, not a state behavior regression. No product compile, power, or whole-engine frame-time
claim is made while that frontier is unresolved.
