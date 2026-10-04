---
title: Plugin Particles Current Source Performance Review
date: 2026-08-24
scope:
  - zircon_plugins/particles
status: static_complete_shared_source_preserved_and_dynamic_pending
canonical_owners:
  - docs/plans/optimize/zircon_plugins/09-first-party-particle-vfx-source-runtime-editor-dist-catalog-simulation-render-product-integration-review.md
  - docs/plans/optimize/zircon_editor/15-material-shader-graph-instance-vfx-particle-preview-compiler-diagnostics-authoring-review.md
  - docs/plans/optimize/zircon_runtime/59-runtime-task-execution-job-scheduler-handle-dependency-cancellation-thread-budget-timer-shutdown-diagnostics-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/89-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-product-integration-current-source-review.md
references:
  - dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/Niagara/Private/NiagaraSystemSimulation.cpp
  - dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/Niagara/Private/NiagaraGpuComputeDispatch.cpp
  - dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/Niagara/Private/NiagaraDataSet.cpp
  - dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/Niagara/Private/NiagaraScalabilityManager.cpp
  - dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/NiagaraEditor/Private/NiagaraCompilationTasks.cpp
  - dev/godot/drivers/gles3/storage/particles_storage.cpp
  - dev/Fyrox/fyrox-impl/src/scene/particle_system/mod.rs
---

# Plugin Particles Current Source Performance Review

## 1. Coverage and evidence state

The reviewed package surface after the scoped local corrections in section 7 is **50/50 Rust files**, **8,624 physical / 7,860 non-empty lines**, **304,003 bytes**, **48 tests** and **1 ignored performance test**. Its package-relative `path + LF + raw bytes` SHA-256 is `93391d39a0b72215bb967cc0a177fb450ba5f22457beadf005187f70479b9690`.

Every Dist, Editor and Runtime Rust file was indexed and read. The editor ZUI resources, CPU template, package manifests, first-party catalog/profile references, Runtime prepare/readback admission, RenderGraph declarations/executors, WGPU backend/WGSL, CPU pool/simulation/extract, optional physics/animation paths and all tests were traced. Workspace-wide call-site searches were used to distinguish registered metadata from executable product behavior.

| Area | Rust files | Physical lines | Current execution truth |
|---|---:|---:|---|
| Dist | 1 | 98 | Publishes native registration metadata only; no invocation bridge owns simulation or authoring behavior. |
| Editor | 6 | 512 | Registers 12 disabled operations and three placeholder ZUI surfaces; no operation handler or live preview/compiler is present. |
| Runtime | 43 | 8,014 | Contains CPU and real WGPU implementations, but no product frame clock advances the manager and actual GPU compute bypasses RenderGraph executors. |

Seven Runtime files already had shared changes at review start: `Cargo.toml`, `src/lib.rs`, `src/render/gpu/neutral_buffers.rs`, `src/render/gpu/runtime_owner.rs`, `src/render/runtime_prepare.rs`, `src/service.rs` and `src/simulation/cpu.rs`. They were treated as current source and were not edited or formatted by this review. Per-file `rustfmt --check --edition 2021 --config skip_children=true` passed **49/50** files; only `editor/src/authoring.rs` has a two-import ordering diff.

Managed Rust tests, WPR/ETW and RenderDoc were not run. The current session has no executable managed Windows validator identity and no launchable current-source engine/editor binary. RenderDoc would not answer the CPU scheduling and ownership failures below; it becomes valid only after the same-source executable renders an actual particle scene.

## 2. Structural performance findings

### P0: there is no product simulation clock or exclusive backend owner

`ParticlesManager::tick` is the only path that advances system age, continuous emission, bursts, lifetime, movement and CPU snapshots. A whole-workspace search finds every `manager.tick(...)` caller inside this package's tests; `ParticlesModule` registers only a lazy service descriptor and no frame/update hook. The first-party Runtime catalog can therefore load the plugin while normal product execution leaves particle age at zero. Time-zero bursts may be planned by the GPU owner, but continuous emission and particle aging do not advance.

The backend contract is also unresolved. Instantiating every GPU asset permanently sets `fallback_to_cpu = true` and creates a full CPU instance. Nothing changes that state after the renderer-owned GPU owner is available; GPU feedback only stores counters. If a host or future frame hook calls `manager.tick`, that GPU instance is advanced and snapshotted on CPU while Runtime prepare also executes it on GPU. If no caller advances it, both CPU-derived extract data and GPU delta time remain stale.

MVP needs one scheduler-owned particle phase and an explicit per-instance execution state: CPU active, GPU active, bounded migration, or typed fallback. Backend activation must be acknowledged before simulation; only one backend advances a generation. Preview and game runtime must use the same clock contract with fixed-step/catch-up budgets.

### P0: real GPU work bypasses the declared RenderGraph

The feature declares spawn/update, compact and indirect passes on `AsyncCompute`, but all three executors only validate context. Their declared workload is a fixed `[1, 1, 1]` dispatch. The real `ParticleGpuBackend::execute_frame` is called earlier from Runtime prepare and directly records two capacity-sized compute dispatches plus indirect generation into the command encoder.

RenderGraph therefore cannot schedule the real work, derive barriers from its actual accesses, cull inactive passes, assign queues, profile the dispatches, or reason about resource lifetime. The later graph passes are success-shaped metadata over buffers already mutated out of graph. The indirect executor can additionally publish CPU/extract-shaped feedback, creating two authorities for GPU outputs.

Unreal Niagara's compute dispatcher registers tick/group/free-ID work as RDG passes (`NiagaraGpuComputeDispatch.cpp`), so graph dependencies and execution describe the same workload. Zircon should move real dispatch encoding behind graph executor callbacks and remove the prepare-time compute path after adoption.

### P0: GPU simulation is coupled to debug/readback admission

Runtime prepare calls the real GPU backend only when `RuntimePrepareCollectorContext::gpu_work_admitted()` is true. That value is whether the shared `GpuReadbackQueue` could prepare another completion frame. When readback slots are busy, particle simulation itself is skipped and only previous buffer bindings are retained.

The pending readback queue is bounded indirectly by shared frame slots, so this is not an unbounded-queue finding. The defect is ownership: every admitted particle frame unconditionally requests counters and indirect draw arguments, and readback capacity becomes the simulation cadence. Shipping simulation must run independently of diagnostic readback. Counter readback should be sampled/on-demand and never gate state progression.

### P0: authoring source, compiler artifact and product preview do not form a path

The Editor package declares 12 create/edit/validate/preview operations, but no operation factory/handler/caller exists and all menu items are explicitly disabled. `authoring.zui`, `preview.zui` and `particle_system.drawer.zui` expose named placeholder spaces rather than authoring controls. The CPU template is TOML, while `ParticleSystemAsset` and nested asset types have no Serde contract and no `particles.system` importer. Runtime plugin options (`particles.backend`, `particles.fixed_preview_dt`) are read only by manifest tests.

The package therefore has no versioned source schema, semantic compiler, cooked artifact, dependency/currentness key, preview session or source/native parity. Runtime owns mutable authoring structs directly and compiles WGPU pipelines from them. Editor responsiveness and DDC claims are impossible until a real source-to-artifact path exists.

Unreal separates Niagara authoring graphs, asynchronous compilation tasks, DDC keys/results and installed executable data. Zircon needs the same boundaries in its own Rust contracts: one versioned authoring schema, one deterministic compiler, immutable CPU/GPU execution artifacts and a preview that installs only the matching generation.

### P0: the GPU algorithm scales with reserved capacity and emitter count

The GPU layout reserves up to **1,048,576 slots**. Its 26-word SoA layout consumes **104 MiB per particle buffer** and **208 MiB for the ping-pong pair**, before alive indices, emitter parameters, counters, readback and render state. At maximum capacity, spawn/update and compaction each dispatch **16,384 workgroups**, or **32,768 workgroups / 2,097,152 invocations per frame**. At 60 Hz this is **1,966,080 workgroups / 125,829,120 invocations per second** even when few particles are alive.

Each spawn/update invocation also calls `find_emitter`, a linear scan of emitter ranges. Its upper bound is O(capacity x emitters); with 256 emitters the worst bound is about **268 million range checks per frame** before simulation math. Dead slots copy the full 26-word record into the other ping-pong buffer. Compaction then scans capacity again.

This must be replaced by compiled dense execution data and active counts/ranges. At minimum, encode emitter ownership without a per-slot emitter scan, dispatch update/compact over active lists or bounded occupied ranges, dispatch spawn over spawn requests/free slots, and build indirect arguments from GPU counters. Capacity is a memory budget, not the per-frame work count.

### P1: per-frame GPU preparation rebuilds authoring-shaped aggregate state

`gpu_runtime_instances()` holds the global manager mutex while cloning every GPU component and its complete asset. The owner then clones every emitter again, allocates formatted aggregate IDs and compares a newly assembled `ParticleSystemAsset` against the prior aggregate every admitted frame. A changed instance set or asset reconstructs the entire backend, including layouts, shaders, pipelines and capacity-sized buffers.

Per-instance planners also clone each emitter every frame, and parameter encoding searches the frame emitter vector for every layout emitter, producing O(E^2) host work. Multi-key curves are reduced to first/last endpoints, so CPU and GPU semantics diverge by design. The aggregate top-level `max_dt/max_age` is not consumed by shader encoding, which makes it misleading state rather than an actual synchronization policy.

Runtime should retain immutable compiled system/emitter artifacts by generation and lightweight mutable instance state by dense handle. Dirty asset generations should rebuild only affected program/pipeline/resource groups off the render path, preserving last-good resources until a fence-safe swap.

### P1: CPU simulation and extraction are serialized high-water scans

All instances are advanced while one `ParticlesManagerState` mutex is held. Each emitter scans `0..pool.alive.len()` for update and again for sprite extraction; dead slots remain in that high-water range. `live_count()` scans it again before each spawn decision and while building emitter state. Every live particle linearly searches both size and color curve windows on every tick.

Snapshot rebuild walks every instance/emitter, materializes every sprite and diagnostic payload, then `build_extract` clones the sprites, computes bounds through a `BTreeMap`, and globally sorts transparent sprites O(P log P). `previous_sprites` is always empty. There is no visibility/significance/distance admission before simulation or extract, no dirty range and no job partitioning.

Unreal batches concurrent system/instance ticks with explicit completion/finalize dependencies (`NiagaraSystemSimulation.cpp`) and keeps data buffers dense by replacing a removed instance with the tail (`NiagaraDataSet.cpp`). Its scalability manager owns significance/culling. Bevy's query APIs provide parallel iteration under declared access, while Godot suppresses inactive non-emitting particle updates and supports bounded fixed-step behavior. Zircon should first publish scheduler/ECS access and phase contracts, then use dense live storage, parallel chunks and visibility/scalability admission.

### P1: optional feature names overstate their implementation

`collision_enabled` applies a per-frame velocity damping factor; it performs no scene query, contact generation, collision response or physics-service call. Animation bindings are validated and stored, but `apply_animation_event` ignores their parameter/path/progress data and only performs play/pause/single-spawn commands. `looped` is never consumed, so bursts do not repeat. The first-party capability catalog labels the Runtime particle capability partial, but the nested physics/animation options still appear executable from registration alone.

These features need typed cross-module inputs, scheduling/dependency ownership and deterministic CPU/GPU semantics, or they must fail closed as unsupported. Collision and animation should not add per-particle branches until their providers and data paths exist.

### P1: current tests validate declarations and tiny correctness cases, not scale

GPU tests create offscreen devices opportunistically and silently return when no adapter is available. The CPU/GPU parity test compares one tiny first frame's counts and indirect arguments; it does not compare multi-frame position, lifetime, curves, bursts, reset, backend migration or pixels. Several tests assert fixed one-workgroup graph metadata even though real execution uses capacity-sized direct dispatches. Test-only readback waits indefinitely for the device, which is acceptable only for isolated tests and must not be confused with production behavior.

The sole ignored benchmark measures shared snapshot clone versus deep clone inside a unit-test binary. It does not cover simulation, extract, sort, upload, dispatch, readback, draw, editor preview, frame pacing or power. There are no counters for active/reserved slots, CPU/GPU backend ownership, skipped simulation frames, per-phase queue/wait time, bytes cloned/uploaded/read back, dispatch size, cache/pipeline rebuilds, culling or stale generations.

## 3. Reference-engine constraints

Unreal is the primary architectural constraint:

- `NiagaraSystemSimulation.cpp` separates game-thread preparation, concurrent system work, batched instance ticks and finalize tasks with explicit graph-event dependencies. Zircon's one locked serial loop is not an acceptable target architecture.
- `NiagaraGpuComputeDispatch.cpp` emits Niagara work through RDG passes and groups execution. Zircon's graph declarations and actual encoder work must converge to one authority.
- `NiagaraDataSet.cpp` distinguishes allocated and active instance counts, rounds GPU allocation deliberately, and removes a live CPU instance by copying the dense tail into its slot. Per-frame work should follow active data, not lifetime high-water capacity.
- `NiagaraScalabilityManager.cpp` and component integration own significance, distance and cull policy. Expensive VFX is admitted by visibility/quality/budget rather than simulated and sorted unconditionally.
- `NiagaraCompilationTasks.cpp` builds DDC keys, performs asynchronous system/shader compilation, polls results and reports cache origin. Authoring structs are not rebuilt into pipelines on each render frame.

Secondary references constrain specific behavior without replacing the Unreal baseline. Godot's GLES3 particle storage tracks inactive/non-emitting state and fixed-step accumulation, and owns real collision resources. Fyrox exposes a maximum draw distance and maintains explicit emitter live counts; its remaining high-water scan is a caution, not the target for engine-scale scheduling.

## 4. Dependency-ordered optimization plan

### M0: establish executable product truth and one simulation clock

Wire a scheduler-owned particle update phase into the resolved Runtime profile. Define fixed/variable step, catch-up/substep and paused/preview behavior. Add typed backend activation/migration receipts and guarantee exactly one backend advances an instance generation. Fail GPU activation to CPU before ticking, not after both states exist.

Close the source/native Dist and first-party profile contracts. Unsupported authoring or GPU behavior must fail capability admission rather than publish registration-only success.

### M1: define source, compiled artifacts and instance state

Create one versioned `particles.system` source schema with migrations, stable module IDs, typed parameters, curve/collision/animation semantics and explicit budgets. Compile it deterministically into immutable CPU and GPU execution artifacts keyed by source/dependency/compiler/target/backend/profile generations.

Move emitter layouts, curve tables, spawn schedules, shader IR/WGSL, reflection, resource layouts and pipeline recipes into artifacts. Runtime instances should retain only handles, clocks, dynamic parameters, backend state, compact data ranges and artifact generation. Import, cook, packaging, game Runtime and preview must consume the same artifact.

### M2: converge GPU work into RenderGraph

Move spawn/update, compaction/alive-list production, indirect argument generation and transparent draw behind the declared graph executors. Register actual external/transient resources and dispatch extents; remove prepare-time compute and fixed `[1,1,1]` success metadata. Let graph compilation own barriers, queue selection, pass culling, timestamps and resource lifetime.

Decouple compute admission from readback slots. Read counters only on sampled diagnostics, explicit tooling requests, parity tests or feature-required CPU publication. Simulation must progress when readback is unavailable.

### M3: replace capacity-wide work with active-range execution

Use dense active slots or alive/free index buffers with O(1) active counts. Dispatch updates/compaction over active counts or occupied pages, and spawning over bounded requests/free slots. Encode emitter ownership directly in dense work items or compile per-emitter dispatch ranges; remove shader `find_emitter`.

Track reserved, committed, occupied, alive and visible counts separately. Grow buffers geometrically within budgets, reuse per-artifact layouts and avoid recreating pipelines/buffers on ordinary instance churn. Preserve fence-safe generations during resize/hot reload.

### M4: schedule CPU simulation, extraction and culling by declared access

Move mutable simulation out of one global mutex into scheduler-owned disjoint instance/chunk jobs. Use dense live storage with cached counts, compiled curve lookup data and bounded chunk sizes. Add significance, distance, visibility, paused/inactive and quality admission before simulation/extract; support reduced tick rates with deterministic accumulation.

Publish immutable frame snapshots once per generation. Build bounds and renderer batches in parallel, avoid cloning complete sprite arrays between snapshot/extract, and sort only visible transparent batches using stable renderer-owned keys. Supply real previous-frame data for temporal consumers.

### M5: build functional Editor authoring and preview

Implement the 12 registered operations, real document/drawer controls and an isolated preview session. Use latest-wins background compilation, debounce/coalescing, cancellation, stale-result rejection and last-good artifact installation. Preview play/pause/stop/rewind/warmup must operate on the same clock and artifact as game Runtime.

Expose source/artifact/current preview generations, diagnostics, backend/capability truth and scalability settings. Stable preview frames must perform zero source compilation or pipeline recreation.

### M6: instrument and qualify on fixed workloads

Add deterministic CPU and GPU fixtures covering idle, sparse, dense, many-emitter, burst, continuous, multi-key curve, collision/animation provider, offscreen/culled, backend migration and edit-burst cases. Report p50/p95/p99 CPU phase, queue delay, GPU pass time, frame latency and energy/frame plus active/reserved slots, visits, bytes, dispatches, allocations, readbacks, cache outcomes and stale/cancel counts.

After a managed Windows current-source executable exists, capture the same scenes with WPR/ETW. Use RenderDoc to verify real graph events, dispatch extents, barriers/bindings, indirect draw counts, pixels and CPU/GPU parity. Compare only fixed hardware, driver, build, scene, viewport, quality and power conditions; reference engines provide architecture and order-of-magnitude sanity, not interchangeable numeric promises.

## 5. Acceptance gates

1. A selected Runtime profile advances particles through one scheduler-owned clock; continuous emission, lifetime and reset work without test-only/manual ticks.
2. Every instance has exactly one active backend per generation. CPU fallback is explicit, bounded and disabled after GPU activation; migration has deterministic state policy.
3. The source/native package admits only executable behavior, and `particles.system` import/cook/preview/runtime consume one versioned compiled artifact contract.
4. Real GPU simulation and draw work executes through matching RenderGraph passes; no prepare-time direct compute or fixed fake workload remains.
5. GPU work continues when diagnostic readback slots are unavailable. Shipping readback is sampled/on-demand and never controls simulation time.
6. Update/compact complexity follows active/occupied work, not maximum capacity times emitter count. Shader emitter lookup is O(1), and sparse maximum-capacity fixtures do not dispatch over every reserved slot.
7. CPU simulation uses dense live data, O(1) live counts, bounded parallel jobs and visibility/scalability admission. No global manager mutex encloses all instance simulation.
8. Editor operations and preview are functional, generation-aware, cancellable and last-good; stable frames do zero compile/pipeline recreation.
9. Optional collision/animation/loop semantics are implemented through typed providers on both supported backends or fail closed.
10. Managed correctness, scale and parity tests pass; WPR/ETW and RenderDoc receipts report fixed-scene p50/p95/p99, CPU/GPU timings, bytes/dispatches, frame pacing and energy/frame before protected-ledger promotion, milestone commit or WeCom completion notification.

## 6. Dynamic validation boundary

The static findings above are source-proven and do not require profiler inference. Absolute latency, power and GPU bottleneck rankings remain unclaimed. The next dynamic gate requires a current-source executable produced by the managed Windows validator under an approved non-C target root. WPR/ETW owns CPU scheduling, waits, allocation and power evidence; RenderDoc owns graph/dispatch/resource/draw/pixel evidence. A stale binary or metadata-only plugin registration is not admissible.

## 7. Scoped local corrections

Two local changes were applied after this report first recorded the structural findings. They preserve public behavior and directly remove source-proven redundant work:

1. `CpuParticlePool` now caches its live count and maintains it on spawn/kill/clear, replacing each O(high-water capacity) query with O(1). Two tests cover spawn, duplicate kill, free-slot reuse, clear and the no-scan source invariant.
2. GPU emitter encoding now builds one dense emitter-index lookup, replacing O(layout emitters x frame emitters) repeated linear search with O(layout emitters + frame emitters). A regression test preserves first-match and zero-fill behavior while enforcing removal of the nested `find`.

For equal layout/frame emitter counts, the host lookup changes from O(E^2) to O(E): at 256 emitters the structural comparison bound falls from 65,536 repeated search steps to roughly 512 index/build visits; at 4,096 it falls from 16,777,216 to roughly 8,192. These are operation-count bounds, not measured latency.

The pre-correction package fingerprint was `6ebbdfb671824806a07ea155497d49bbcbbdc218b71f2de23e3a4deb0284914b`. The three edited Rust files pass `rustfmt --check`; the complete package remains 49/50 because of the pre-existing Editor import-order diff recorded in section 1. The new tests were not executed because the managed Windows validator is unavailable. These corrections are regression guards, not evidence that the P0 architecture is accepted.
