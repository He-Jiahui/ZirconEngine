# Frameworks01 M1 `zr_kernel` current-source preflight (2026-08-30)

## Status

- `preflight_complete`
- `physical_hard_cut_not_started`
- `kernel_algorithm_runtime_assembly_boundary_locked`
- `diagnostics_dependency_cycle_rejected`
- `current_source_profile_required_before_algorithm_optimization`
- `milestone_not_accepted`

This is an architecture and measurement-admission record. It does not claim a crate migration,
managed compile result, performance improvement, power reduction, reference-engine parity, or
milestone acceptance. Coordinator authorization request
`b3ec31696c044bcdadbeaa4e5c446f04` allowed this child-plan path; the parent numbered plan remains
protected from ordinary business-session writes.

## Current Source Snapshot

The review used shared main HEAD `cc5cadbd597c3707954ebd6109fad0fd5643a152`. The current
`zircon_runtime/src/core/runtime` tree contains 335 Rust files, 51,573 lines, and 1,804,735 bytes.
The dominant subtrees are diagnostics (80 files/13,593 lines), tasks (75/13,248), tests
(64/10,989), handle (40/6,260), random (17/1,636), and descriptors (13/1,494). The separate
`zircon_runtime/src/engine_module` tree contains eight Rust files and 571 lines.

A path/line/domain audit was written outside the C drive to
`D:/zircon-validation/frameworks01-kernel-preflight-20260830/runtime-domain-audit.json`. Its SHA-256
is `cb95413a1afc561257d5260e142657987b00da6c170c5c9f48076b8634103340`; the 920,128-byte schema-1
record contains 3,421 production references and 75 domain edges. A focused lexical audit of
`core/runtime` finds 72 files consuming `core::framework`, 17 consuming `core::diagnostics`, and
five consuming `engine_module` for built-in module assembly. It finds no production dependency on
asset, scene, editor UI, render graph, WGPU, Naga, or Glyphon code.

Neither `zircon_runtime/crates/zr_kernel` nor `zircon_runtime/crates/zr_diagnostics` exists. The
physical hard cut must follow the planned foundation order and current ownership rotation; a new
crate shell without deleting the old implementation owner would not satisfy the plan.

## Structural Review

The parent plan's short label `core/runtime + engine_module` describes the capability set, not a
literal directory move. Current source combines four different ownership domains:

1. Kernel algorithms and contracts: module lifecycle states/descriptors and dependency ordering,
   event-bus delivery policy, task graph/admission/shutdown, state-machine transitions, frame-clock
   and fixed-step algorithms, deterministic random service/stream algorithms, and stable
   module/service traits.
2. Runtime object assembly: `CoreRuntime`, `CoreHandle`, `CoreWeak`, `CoreRuntimeInner`, context and
   factory helpers, built-in module wiring, registry mutation, service resolution, and lifecycle
   observation.
3. Diagnostics capability: diagnostic DTO/store/profiling/render-stat state, which is reviewed for
   `zr_diagnostics` rather than kernel ownership.
4. Product facade and tests: curated public exports, product policy, integration tests, and manager
   adapters that intentionally sit above the lower crates.

`CoreRuntimeInner` currently owns the task graph, clock, random authority, modules, services,
configuration, events, diagnostic store, and devtools plugin catalog in one facade assembly object.
Moving that object into `zr_kernel` would force kernel to depend on diagnostics state. Moving
diagnostic DTO/store into `zr_diagnostics` would then require diagnostics to depend back on kernel
contracts. That cycle is structural, not a Rust import-detail problem. `CoreRuntimeInner` and its
handle/facade must remain above both crates.

The same split is visible in `engine_module`: `EngineModule`, `EngineService`, driver/manager/plugin
traits, names, and dependency specifications are stable contracts; `contexts.rs` and
`service_factory.rs` construct facade-owned `CoreWeak`, `PluginContext`, `ServiceObject`, and
`CoreError` values and must remain Runtime assembly.

## Reference-Engine Findings

Unreal Engine is the primary reference:

- `Core/Public/Modules/ModuleInterface.h` defines a small lifecycle contract. Dependencies loaded
  during startup remain available through reverse-order shutdown.
- `Core/Public/Modules/ModuleManager.h` separately owns dynamic loading, registry lookup, callbacks,
  unloading, and abandonment. Contract and manager assembly are related but not one physical owner.
- `Core/Public/Async/TaskGraphInterfaces.h` exposes explicit queueing, named-thread attachment,
  processing, completion waiting, and shutdown-facing behavior through the task graph interface.
  Execution policy is an engine-core capability rather than a scene/editor subsystem concern.

These references support placing lifecycle/task algorithms and stable contracts in a low-level
kernel while leaving loading, object construction, registry resolution, and product assembly in the
Runtime facade. They do not support moving the whole current `core/runtime` directory into one
crate. Bevy's separation of app/world assembly from task executors is a secondary cross-check, not
the primary design source.

## Locked Hard-Cut Shape

After the required `zr_contracts` prerequisites are present and accepted, the `zr_kernel` batch
must:

1. Move kernel-owned lifecycle/descriptors, dependency ordering, event bus, task execution,
   state-machine, time/clock, random execution, and stable module/service traits into
   `zircon_runtime/crates/zr_kernel` as their only implementation owner.
2. Keep `CoreRuntime`, `CoreHandle`, `CoreWeak`, `CoreRuntimeInner`, built-in module assembly,
   registry/service resolution, context/factory helpers, lifecycle observers, and manager-facing
   adapters in the Runtime facade. The facade may depend on `zr_kernel`, `zr_diagnostics`, and
   `zr_contracts`; the reverse edges are forbidden.
3. Keep diagnostic store/DTO/profiling/render-stat capabilities out of kernel. Move them only under
   the independently reviewed `zr_diagnostics` hard cut; keep `CoreHandle`-reading collectors and
   devtools assembly above both crates.
4. Partition `engine_module` by semantic owner in the same atomic migration. Do not retain an old
   module implementation, wildcard projection, compatibility alias, duplicate trait, or forwarding
   wrapper.
5. Preserve `zircon_runtime` as the curated product facade. App and Editor consumers must not gain
   direct dependencies on the internal foundation crates unless a separately reviewed public
   contract requires it.
6. Regenerate the complete move/consumer manifest and update Cargo manifests, lockfile, facade
   exports, tests, static guards, and architecture documentation atomically through the coordinator.

## Algorithm And Profiling Gate

No kernel algorithm optimization is admitted from source inspection alone. Before changing task
queues, module ordering, state transitions, fixed-step scheduling, or random-stream storage, select
one subsystem, reproduce an exact current-source Windows managed baseline outside the C drive, and
record its source manifest and workload.

For task execution, the minimum baseline includes submit/ready/start/complete latency P50/P95/P99,
throughput by producer/worker count, queue depth, steals/wakeups, idle CPU, shutdown latency,
allocations, context switches, RSS, and ETW scheduling evidence. For lifecycle ordering, record graph
size/edge density, validation and topological-sort cost, activation/rollback/shutdown latency, and
typed failure behavior. For time/random/state, record deterministic replay equality and algorithmic
scaling before elapsed-time comparison. Power or reference-engine parity requires comparable
workloads and measured system counters; this record makes neither claim.

Only a measured dominant cost may select an optimization. Worker-count tuning, lock replacement,
queue replacement, graph caching, or arena allocation remain hypotheses until the baseline proves
the bottleneck and a same-source post-profile demonstrates its removal without weakening shutdown,
ordering, or deterministic behavior.

## Admission Boundary

No production or test file was changed for this preflight. Physical `zr_kernel` work remains behind
accepted `zr_contracts`, fresh exact ownership, a complete current-source move/consumer manifest,
and managed validation capacity. The immediate executable Frameworks01 validation remains the final
GREEN for the implemented `zr_resource` path-normalization correction.

## 2026-08-30 Shared-manifest and cross-plan coordination refresh

The fresh ownership transfer preview for the three shared integration files was request
`94cf8ede78e1402b88727346c3dbc7b1` at coordinator baseline epoch `573`. It remains ineligible
solely because Shader06 primary session `01a019a5-b15f-7461-a1b0-ce4b6aa8e710` is executable with
status `resolving_failure`; its write scope no longer lists these mixed files, but the executable
source attribution is still authoritative. The preview fingerprint is
`bc4b5a329c564ea0c7c9db1938aab03d0f64863f40cdd339ab3724b3c109c4e2`. Current hashes were
`Cargo.toml=d1de7ecad881433a6a23762319c958316331f903e0372bab5f7976d751dfe3a9`,
`Cargo.lock=70a18867be9385416425dbbda9b9adb5692fca2bee887418a224002e58eb1217`, and
`zircon_runtime/Cargo.toml=531f8a844f198b99e0f28991ab6eb30ee816c90f8d105c5bc6edf709127c17ff`.
Frameworks01 did not claim, edit, regenerate, or transfer any of the three files. The legal next
step remains a Shader06 owner-side current-source commit/release or coordinator status rotation,
followed by a new preview before `zr_contracts` wiring.

Runtime22's clean-copy report also remains cross-plan. The mixed
`zircon_runtime/src/dynamic_api/session/state.rs` blob is attributed to
`root-runtime22-checkpoint-atomicity-20260829`; no active mvp00 integration owner or Frameworks01
scope covers its pre-existing host-request/UI changes, so it must not be repaired or absorbed here.
The compile-time `zircon_runtime/crates/zr_rhi_wgpu/src/command_validation/copy_commands.rs` input
is owned by Runtime90 primary `root-runtime90-copy-command-resource-closure-20260829` in
`resolving_failure` and must be committed with Runtime90's command-validation union. Frameworks01
will not claim, transfer, or edit either path; Runtime22 can rerun its exact copy only after the
Runtime90 source is present in HEAD and the mixed Runtime22 integration surface is resolved by its
own owner.

## 2026-08-30 Current-source epoch drift reconciliation

After this preflight was first captured, the coordinator advanced shared HEAD to
`399f2318150ae4fa0df3a2543133b03b80099288` through an external `runtime_interface` diagnostic-id
performance commit. Its three changed paths are limited to
`zircon_runtime_interface/src/plugin_diagnostics.rs`, the corresponding RuntimeInterface04 plan
record, and its static performance contract. A fresh state-machine manifest was regenerated at
`D:\zircon-validation\frameworks01-kernel-state-manifest-20260830\state-machine-manifest.json`:
`12 files / 517 lines / 14,817 bytes`, artifact SHA-256
`132647339017960c118b1cfbca8f24331f8e50f7586cfb2f7543f3370b7f9b61`. The state-machine source
rows remain unchanged from the prior 12-file review; this artifact is the current move input, not
permission to start a partial crate move. The shared manifest transfer preview was refreshed at
the new epoch as request `ac37b32feb104fe597ab636457169547` and still reports Shader06 executable
ownership for all three integration files. No Frameworks01, Runtime22, or Runtime90 source path
was edited or absorbed during this epoch transition.

## 2026-08-30 Runtime22 time and Runtime90 RHI ownership reconciliation

The latest Runtime22 request for `zircon_runtime/src/core/runtime/time.rs` no longer requires an
owner-side Frameworks01 status rotation. `session show` request
`f734615717dc4071a0adc7164d173d09` confirms the exact path is already present in
`root-runtime22-checkpoint-atomicity-20260829`'s immutable write scope. Fresh transfer-preview
request `28461dcb8d2449889b20750a88589396` at baseline epoch `574` records fingerprint
`2fd2c14b79876420639ccf8d535707ac07f68001acb227e87587c3dc25b0b272`, current/content hash
`6f594f01b6f7974fb11664344b88883526fddc07c7df2b054360ba2a5f30e4fd`, source and target both
Runtime22, and the sole blocking reason `path_already_owned_by_target`. Repeating transfer-apply or
rotating Frameworks01 would be incorrect; Runtime22 must refresh its own lease and attribution
before validation or closeout.

The RHI validation-copy input remains a Runtime90 responsibility. `session show` request
`18ee854ea1ab471a9cb59ac68661dde7` confirms
`zircon_runtime/crates/zr_rhi_wgpu/src/command_validation/copy_commands.rs` is in the exact write
scope of `root-runtime90-copy-command-resource-closure-20260829`. Fresh ownership-matrix request
`7bb2bc7eb56c49048f0e8e4e98673218` observes current hash
`493da66a570a4230ce39b2804a838727a434dc2e72f0a7556c50c7888e77b124`, replacing the earlier
reported `fcd154...` snapshot, with `attribution_baseline_stale` and `live_lease_missing` as the
remaining ownership blockers. The exact evidence and required Runtime90 re-claim, re-attribution,
validation, and coordinator commit were returned to task
`01a04c62-01c7-7ea1-8328-87fa4eb2b125`. Frameworks01 did not edit, claim, transfer, or submit the
RHI source; Runtime22's mixed `dynamic_api/session/state.rs` remains untouched as required.
