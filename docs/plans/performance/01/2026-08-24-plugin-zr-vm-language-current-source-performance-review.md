---
title: Plugin Zr VM Language Current Source Performance Review
date: 2026-08-24
scope:
  - zircon_plugins/zr_vm_language
  - zircon_plugins/first_party_runtime_catalog
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_runtime/src/script/vm
status: static_complete_shared_source_preserved_dynamic_pending
canonical_owners:
  - docs/plans/optimize/zircon_plugins/16-first-party-zr-vm-language-source-runtime-dist-catalog-reflection-callsite-host-interface-gc-hot-reload-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md
  - docs/plans/optimize/zircon_runtime/21-zr-language-parser-type-system-semir-bytecode-package-loader-vm-runtime-review.md
  - docs/plans/optimize/zircon_editor/31-script-source-code-editor-build-compiler-hot-reload-debugger-visual-script-class-component-authoring-review.md
  - docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md
  - docs/plans/optimize/zircon_runtime_interface/05-runtime-host-foreign-output-safe-api-ownership-admission-budget-fuse-observability-review.md
references:
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/VerseVM/VVMContext.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/VerseVM/VVMHeap.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/VerseVM/VVMCollectionCycleRequest.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/VerseVM/VVMBytecodeAnalysis.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/UObject/ScriptCore.cpp
  - dev/UnrealEngine/Engine/Source/Editor/AnimGraph/Private/AnimBlueprintExtension_PropertyAccess.cpp
  - dev/UnrealEngine/Engine/Source/Editor/Kismet/Private/BlueprintCompilationManager.cpp
  - dev/godot/modules/gdscript/gdscript_cache.cpp
  - dev/bevy/crates/bevy_reflect/src/type_registry.rs
---

# Plugin Zr VM Language Current Source Performance Review

## 1. Coverage and execution truth

This review covers **37/37 Rust files**, **4,708 physical / 4,317 non-empty lines**, **169,035 bytes**, **44 test markers** and **1 ignored performance test**. The package-relative `path + NUL + LF-normalized bytes + NUL` SHA-256 is `0f9529a4dc56705410b50be41ec566bd9ebb1bd167a900af7cc0261f67c86145`.

| Module/folder | Rust files | Physical lines | Current execution truth |
|---|---:|---:|---|
| `dist` | 1 | 98 | Native descriptor/registration-manifest carrier only; no VM command, bridge, state, unload or host-ready behavior. |
| `runtime` root | 6 | 460 | Optional provider/backend registration and seven conservative scene systems; disabled by default and Partial. |
| `runtime/call_site` | 6 | 974 | Immutable dense reflected type/field table with generation-qualified direct tokens and one ignored microbenchmark. |
| `runtime/host_interface` | 5 | 87 | Thin capability-gated registration adapters for systems, behavior nodes, RPC and Editor operations. |
| `runtime/real_backend` | 10 | 1,194 | Feature-gated external ZrVM binding, synchronous source compile/session startup, global lock, native modules and value lowering. |
| `runtime/reflection_host` | 4 | 702 | Package-local table/catalog locks, runtime World access and JSON reflection ABI. |
| `runtime/tests` | 5 | 1,193 | Registration and feature-gated real-backend fixtures; fixtures default to the OS temporary directory. |

All 37 files pass per-file `rustfmt --check --edition 2021 --config skip_children=true`. `runtime/src/call_site/script_call_table.rs` and `runtime/src/call_site/tests.rs` already contain shared uncommitted work and were preserved. Rust tests were not run: the managed Windows validator is unavailable, and real-backend fixtures would write to the C-drive OS temporary directory unless the harness first relocates `TEMP`/`TMP` to an approved D/E/F root.

`wpr.exe`, `renderdoccmd.exe` and a launchable current-source `zircon_editor.exe`/`zircon_runtime.exe` were not found in the active tool/path scan. WPR or RenderDoc therefore cannot produce honest current-source evidence in this review. RenderDoc is also not a CPU/VM contention tool; it becomes relevant only after a script-driven render product exists and needs frame/pixel/pass parity.

## 2. Foundations that must be retained

- `ScriptCallTable` resolves public names during package preparation, stores dense type/member slots and rejects wrong-generation tokens in O(1). Runtime reflection does not repeat field-name dispatch.
- Reflection catalog prepare/commit and hot-reload rollback already prevent a prepared schema from dispatching before exact commit.
- Host interfaces bind callbacks to slot generation and capability, then refresh stable handles against the active generation.
- Slot instances are removed from the coordinator map before external calls, so coordinator locks are not held while user VM code executes.
- Cooperative GC has a host-owned frame deadline, deterministic slot queue, per-slot elapsed time and bounded diagnostics history.
- Script binding projection is keyed by the dynamic-component generation rather than rebuilt after every unrelated World mutation.

These are useful local mechanisms. They do not close the product, concurrency, artifact or ABI architecture, and should not be discarded during the hard cut.

## 3. Structural performance findings

### P0: the declared NativeDynamic product cannot execute the real backend

The runtime plugin is disabled by default and both capabilities are Partial. Linking `first-party-zr-vm-language-runtime-plugin` only registers the provider; real execution additionally requires `backend-zr-vm`. The `dist` crate depends on the runtime crate without that feature and exports an empty behavior table while its package advertises NativeDynamic as the default carrier and provides `script.behavior.v1`.

This is not a startup-cost optimization problem. Product preflight needs a target-specific `LanguageActivationPlan` proving provider, carrier, native SDK, compiler/artifact, ABI and policy closure. NativeDynamic must either expose an equivalent executable VM service or stop advertising the runtime/interface capability. Enabling the feature by default would only hide the missing reproducible toolchain and distribution contract.

### P0: source compilation occurs inside nested process-wide control paths

`HotReloadCoordinator::load_package` holds the global lifecycle guard while calling the backend. The real backend then holds the process-wide ZrVM mutex across runtime construction, host-module registration, `ProjectWorkspace::open`, incremental compile and session startup. Hot reload first removes/deactivates the active instance and only then repeats this compile/start path for the candidate.

The effective critical path is:

`lifecycle guard -> process VM guard -> open source workspace -> compile -> start session -> schema query -> activate -> commit`

One slow compile blocks every package lifecycle operation; the old generation is unavailable throughout candidate construction. `incremental: true` does not establish a cross-session content-addressed cache because every load creates a new runtime/workspace and no compiler/source/options/dependency receipt is handed to Runtime.

Shipping Runtime must never compile source. Editor/cook owns dependency closure, compile, bytecode verification and immutable artifact publication. Runtime load/reload should verify and stage an artifact off the active generation, quiesce at commit, then atomically switch a generation lease with last-good rollback.

### P0: one mutex limits all ZrVM work to concurrency one

Activate, deactivate, save/restore/schema, every export, GC and destruction acquire the same static `Mutex<()>`. `ZrVmRuntimeOwner` uses that mutex as the complete safety proof for `unsafe impl Send/Sync`. The three registered extension-system dispatchers invoke callbacks sequentially; both scene-binding systems iterate entities sequentially; all seven plugin systems declare conservative World access and join one system set. Adding ECS workers cannot overcome a backend whose maximum process-wide ZrVM concurrency is **1**.

Do not replace this with unproven per-instance locks. First certify the native VM thread/context/root/safepoint model. If instances are independent, use owner-affine per-world/per-domain runtimes with explicit context roles and deterministic command merge. If the binding is process-global and non-reentrant, move it behind a bounded worker actor/process and report that backend's latency/concurrency limit. Trusted in-process and isolated untrusted policies need separate qualification.

### P0: O(1) call-site lookup still ends in allocation-heavy scalar ABI work

The dense token improvement removes string lookup but not the dominant boundary cost:

- Reflection read serializes every `ReflectedValue` to tagged JSON and allocates a ZrVM string.
- Reflection write copies the guest string and parses JSON before a single field write.
- Any ZrVM Array is treated as bytes, so a generic typed array can be misclassified.
- Host-to-VM bytes perform one `Value::new_int` plus one `array_push` per byte.
- VM-to-host bytes perform one `array_get` and integer conversion per byte before filling an owned `Vec<u8>`.

For a 1 MiB payload, the current lowering performs **1,048,576 integer constructions and 1,048,576 pushes** in one direction, or **1,048,576 indexed VM reads** in the other, before useful work. Reflection similarly pays serialization/parsing and copies per scalar field. No buffer-byte, call-count or allocation budget protects these paths.

Compile a versioned tagged value/layout ABI from the shared type registry. Use borrowed spans or owned buffer leases with bulk copy, typed aggregate/optional/result/handle tags, and a batched `ScriptWorldTransaction`. JSON remains an authoring/debug compatibility boundary, never a per-frame gameplay ABI.

### P0: global catalog revisions turn unrelated schema changes into synchronous rebind work

Each package table is tied to the process-wide reflection catalog revision. A changed revision makes existing dispatch fail closed; a later `resolve` synchronously recompiles the complete public table under its write lock. When reached through a VM native callback, that rebuild occurs while the process VM mutex is already held. Existing stored tokens become stale and guest code must re-resolve them.

Preserve stale-token rejection, but replace global revision invalidation with stable schema/type/field IDs and affected-owner segments. Build new immutable segments before publication, retain old segments for active generation leases, and atomically rebind only dependent packages. Runtime calls should never discover that they need to rebuild a full schema table.

### P0: ordinary calls and GC are not enforceably preemptible

The coordinator measures a GC deadline and passes remaining microseconds to one backend call, but it cannot preempt a backend that overruns. Ordinary exports have no fuel, wall deadline, cancellation, host-call count, recursion or allocation quota at all. Because every call owns the process VM mutex, one runaway export or GC step blocks all packages and can stall the main/runtime thread. Mutex poison recovery only recovers the Rust lock; it does not prove the C VM remains valid after panic or memory corruption.

Execution needs per-tick/per-operation budgets enforced at VM safepoints, bounded host-call admission, cancellation and typed fault/quarantine. GC needs heap/live/native/allocation/fragmentation metrics and resumable cursor receipts, not only pause/root counts.

### P1: scheduling snapshots and context setup remain per-frame work

Every dispatcher resolves `VmPluginManager`, snapshots the active systems and sequentially invokes each callback. Each scene binding builds two owned host arguments, a weak core handle and a cloned level handle before invoking one entity callback. The projection cache avoids full-scene reconstruction, but the execution model is still one host transition per binding/export and has no compiled read/write access plan, batch input columns or atomic command commit.

Compile Script Class/Component bindings into stable ECS layouts. Prepare immutable input snapshots by access plan, batch calls by `(world, package generation, module, export)`, execute on certified VM owners, then deterministically merge commands/events. Conservative access remains the fallback for unknown host calls, not the permanent schedule for all seven systems.

### P1: the only benchmark excludes every structural bottleneck

The ignored benchmark performs 4,096 x 32 token resolutions and compares a token `HashMap` with direct indexing. It does not load/compile a package, wait for the global mutex, cross the VM ABI, access World state, serialize JSON, transfer bytes, run GC, reload, render or measure power. It is valid as a local lookup regression gate, not as evidence for the scripting product.

## 4. Reference-engine evidence and adopted boundaries

Unreal is the primary source reference:

- `VVMContext.h` makes thread/heap authority explicit through IO, access, running, allocation and handshake contexts. The API documents root/stack scanning and concurrent-GC handshakes rather than treating a global lock as a transferable `Send/Sync` proof.
- `VVMHeap.h` tracks live native bytes, exposes concurrent collection-cycle requests and separates heap spaces by destruction/census behavior. `VVMCollectionCycleRequest.h` explicitly forbids waiting from a running context because it would deadlock.
- `VVMBytecodeAnalysis.cpp` derives control-flow blocks and validates jump/failure/task edges from the opcode stream before execution. Zircon's artifact owner must perform equivalent admission before Runtime loads bytecode.
- `AnimBlueprintExtension_PropertyAccess.cpp:34-177` compiles property access handles and selects worker-thread or game-thread cached batches from endpoint thread safety. This supports a compiled access plan, not per-field JSON calls.
- `BlueprintCompilationManager.cpp:173-230` separates queued compilation from later reinstancing; `677+` gathers dependencies, filters work, orders stages and emits CPU trace scopes. This supports staged compile/publish with currentness and instrumentation.
- `ScriptCore.cpp:107-120` enforces script time limits at bounded instruction intervals, while `568-572` dispatches bytecode through a fixed opcode table. A host wall timer observed after an unbounded call is weaker.

Godot's `gdscript_cache.cpp:69-142, 208-262, 419-440` provides monotonic parser phases, forward/inverse dependency tracking and recursive invalidation. Bevy's `TypeRegistry` keeps `TypeId`, full type path and short-name ambiguity separately. These reinforce dependency-directed rebuilds and stable typed identities; they do not replace the Unreal concurrency, compilation and instrumentation model.

## 5. Required optimization sequence

| Milestone | Owner result | Acceptance gate |
|---|---|---|
| M0 Product/toolchain truth | Freeze clean external toolchain and target/carrier/provider matrix; correct misleading NativeDynamic capability. | Clean clone reproduces toolchain/artifact digests; missing provider fails in preflight. |
| M1 Offline artifact pipeline | Editor/cook compiles and verifies source into atomic immutable BuildSets. | Shipping Runtime performs zero source parses/compiles; failed build preserves last-good generation. |
| M2 Typed host ABI | Generated tagged values, buffer leases, field codecs and stable schema IDs. | Frame path has zero reflection JSON; 1 MiB bytes uses bounded bulk operations and zero per-byte VM value allocations. |
| M3 VM ownership | Certified contexts, per-world/domain owners or isolated worker actor, leases and quiescence. | Unrelated worlds/packages can progress concurrently; no process-global VM mutex is the safety model. |
| M4 Script transaction/schedule | Compiled access plans, batch input, worker execution and deterministic command/event commit. | Work scales with active bindings and transferred fields; failure/stale/cancel publishes no partial World mutation. |
| M5 GC/reload/isolation | Enforced safepoint budgets, memory policy, staged reload and crash/hang quarantine. | Deadline/fuel/heap violations are bounded typed outcomes; old generation stays live until atomic commit. |
| M6 Editor product | Workspace/LSP/build/hot-reload/debugger/profiler consume one source/artifact generation. | Edit-build-play-debug-reload-stop/reopen uses the same BuildSet and source map. |
| M7 Dynamic acceptance | Current-source Windows product, fixed workloads and WPR/ETW CPU/allocation/power captures; RenderDoc only for script-driven render parity. | 1/100/10k instances and 1/4/16 worlds report compile/load/call/host/GC/heap/RSS/lock-wait P50/P95/P99 with correctness parity. |

Complexity gates are part of acceptance: package preparation scales with the changed dependency closure; call-site dispatch remains O(1); bulk payload bridge work is O(bytes) memory copy with O(1) ABI calls rather than O(bytes) VM objects/calls; reflection catalog publication scales with affected schemas; frame scheduling scales with active bindings and declared accesses, not all packages/types.

## 6. Instrumentation contract

Record BuildSet/toolchain/source/dependency/artifact identities; target/carrier/provider; world/domain/slot/generation; compile queue/wall/CPU/cache-hit reasons; VM queue and lock-wait time; instruction/fuel/host-call counts; ABI calls, copied bytes and allocations by value kind; reflected fields/batches/JSON compatibility calls; active bindings/batches/tasks/commands; GC requested/actual pause, heap/live/native/fragmentation/allocation rate; reload prepare/quiesce/commit/rollback; faults/quarantine/restart; frame and power counters.

The external dependency is currently `E:/Git/zr_vm` commit `4af30a6c911407a2a852a1ca5f5f3ef385d0fe81` with **32** worktree changes. Existing canonical documents that name another revision/change count are stale and must set `source_recheck_required` until a clean receipt is frozen.

## 7. This review's implementation decision

No production source was changed. The first safe implementation is not a local allocation tweak: M0/M1 must remove runtime compilation and establish artifact/product truth before the global-lock, typed-ABI and scheduling hard cuts can be validated. Changing the shared call table, widening lock scope, enabling the backend by default, or adding more JSON host functions would optimize or legitimize a temporary architecture.

Static review is complete for `zircon_plugins/zr_vm_language`; dynamic Runtime/Editor/backend acceptance remains pending. This is not a milestone-completion claim and does not warrant a Git milestone commit or quantified WeCom message.
