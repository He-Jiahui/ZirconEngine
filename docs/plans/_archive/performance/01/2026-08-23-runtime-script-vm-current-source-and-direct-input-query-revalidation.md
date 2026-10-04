---
title: Runtime Script VM Current Source and Direct Input Query Revalidation
date: 2026-08-23
scope:
  - zircon_runtime/src/script
status: static_complete_dynamic_pending
source_fingerprint: 74dcbdaedf919dfc42830fd3dab559a9080aa0508e43cf351c357dc322b72c48
canonical_owners:
  - docs/plans/optimize/zircon_runtime/21-zr-language-parser-type-system-semir-bytecode-package-loader-vm-runtime-review.md
  - docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md
  - docs/plans/optimize/zircon_plugins/16-first-party-zr-vm-language-source-runtime-dist-catalog-reflection-callsite-host-interface-gc-hot-reload-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/99r-runtime-input-device-event-frame-state-action-map-focus-gamepad-recording-replay-host-product-integration-current-source-review.md
references:
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/VerseVM/VVMContext.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/VerseVM/VVMHeap.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/VerseVM/VVMCollectionCycleRequest.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/VerseVM/VVMBytecodeAnalysis.cpp
  - dev/godot/modules/gdscript/gdscript_vm.cpp
  - dev/godot/modules/gdscript/gdscript_cache.cpp
---

# Runtime Script VM Current Source and Direct Input Query Revalidation

## 1. Current-source coverage

`zircon_runtime/src/script/**` currently contains **102/102 Rust files, 18,879 physical lines / 17,309 non-empty lines, 654,022 bytes, and 156 test markers**. The VM subtree accounts for 101 files / 18,837 physical lines / 650,914 bytes; root `script/mod.rs` is a 42-line public re-export surface and was reread directly. The ordered SHA-256 over each workspace-relative path, NUL, raw bytes, and NUL is `74dcbdaedf919dfc42830fd3dab559a9080aa0508e43cf351c357dc322b72c48`.

Runtime21 already covers the current 102-file language/compiler/loader/runtime surface. Plugins16 froze the immediately preceding 101-file VM snapshot and reviewed the vertical Zr language provider, real backend, host/reflection/GC, catalog, App and product chain. From its `25e09a23178000f2e783ce2143cf70a8b118d404` source revision to the current tree, only `scene_system.rs` import ordering and `gameplay_host/input.rs` changed. The script tree is currently clean. This report therefore combines the two full reviews with direct rereads of the root, both deltas, the input manager implementation, Core manager resolution, Vampire consumer, and the relevant Unreal sources; it does not create a fourth VM issue ledger.

No Rust test, real ZrVM backend, Vampire product, WPR/ETW or current-source executable was run. The 156 test markers are inventory, not pass results.

## 2. Confirmed deterministic improvement

Commit `08094b9b9e17f6c80372e15c17b01204038b305b` replaced `gameplay.key_pressed`'s per-query `InputSnapshot` materialization with `InputManager::button_pressed`. `DefaultInputManager` overrides the compatibility default and performs one locked `ButtonInputState::pressed` lookup. With P pressed buttons and Q queries, the old path copied P retained values Q times before searching; the current concrete path performs Q ordered-set lookups and copies zero retained button values. Runtime56's release workload therefore reduces 2,048 snapshot vectors and 2,097,152 retained-button clones per sample to zero. This is deterministic work reduction, not measured latency.

The change should be preserved, but it does not close the input/script architecture. `ButtonInputState` is a `BTreeSet`, so lookup remains O(log P). Named keys still allocate `InputButton::Key(String)` on every query. Each host call upgrades the runtime Core weak handle, resolves the manager identity by name, resolves the typed service generation, clones the service `Arc`, and locks the complete input state. Vampire calls the raw string API four times in every update for A/D/W/S. Runtime99r correctly marks `INP-P1-005` Partial and routes the product to compiled typed actions and one frame context.

## 3. VM performance findings in architecture order

| Priority | Current fact | Performance consequence | Canonical owner |
|---|---|---|---|
| P0 | external Zr toolchain/source is not reproducibly fixed; `.zro` admission, artifact transaction, and type/codegen/object/GC semantics remain unqualified | timing an untrusted or semantically divergent backend cannot establish an engine baseline | Runtime21 M0-M5 |
| P1 | real backend uses one process-wide mutex as runtime/raw-pointer owner across package lifecycle, calls and GC | one long package call, collection or drop serializes unrelated packages/worlds; lock wait can dominate frame time | Runtime07 / Plugins16 M4 |
| P1 | source workspace open, incremental compile and session start occur synchronously during runtime package load | startup/reload pays compiler and I/O work on the activation path and cannot reuse a qualified immutable BuildSet | Runtime21 M1-M4 / Plugins16 M1 |
| P1 | ordinary calls lack enforced fuel, deadline, cancellation, host-call, allocation and per-package quota | runaway scripts and host-call storms can monopolize an owner thread or frame | Runtime21 M5 / Runtime07 |
| P1 | scene systems declare conservative world access and gameplay behavior uses repeated scalar/JSON host calls | scheduler cannot safely parallelize independent worlds/entities; boundary work scales with calls and serialization | Plugins16 M3-M4 / Runtime07 |
| P1 | GC has cooperative host scheduling, deadline and telemetry, but no qualified per-world context/root model or live-byte memory enforcement | pause and contention cannot be attributed or bounded per world/package; global serialization remains | Plugins16 M4-M5 |
| P1 Partial | `ScriptCallTable` already resolves names at preparation and dispatches by dense `u32` site over immutable `Arc` storage | this is the correct hot-path foundation; do not reintroduce name lookup or a registry lock per prepared call | Plugins16 M3 |
| P1 Partial | direct button lookup removes snapshot copying but retains raw string compilation, manager resolution and input lock per host call | four Vampire movement queries still repeat boundary work; typed action/frame transaction is the correct cut | Runtime99r M117.4 / App06 |

`HostExportRegistry` rebuilds the immutable call table under its state mutex only when modules register; prepared `ScriptCallTable::call` indexes the vector without that registry lock, validates arity/capability, installs the borrowed runtime frame and invokes the callback. Registry rebuild cost and two-level name maps are control-plane work, not the current per-call bottleneck. Optimizing them before execution ownership, typed frame transactions and product scheduling would target the wrong layer.

## 4. Unreal and Godot source constraints

Unreal VerseVM's `VVMContext.h` models I/O, heap access, running, allocation and handshake as distinct context capabilities. It explicitly says the type of context documents what a function may do to the heap, uses per-thread context claim/release, and identifies soft handshake as the basis of concurrent GC. `VVMCollectionCycleRequest` forbids waiting for GC from a running context because that can deadlock. `VVMHeap.h` separates marking, census, destruction, sweeping and completion, owns collector-thread execution, and coalesces cycle requests by requested/completed generation rather than launching duplicate collections.

`VVMBytecodeAnalysis.cpp` builds basic blocks and successor/predecessor edges from opcode branches, checks that branch classification and analysis remain synchronized, and requires every incoming edge to a block to agree on the failure-context stack. This supports Runtime21's bounded CFG/type/layout verifier before materialization or execution. It does not support loading partially checked `.zro` and attempting to recover inside the interpreter.

Godot's GDScript VM/cache remains useful secondary evidence for language-owned execution plus parser/dependency invalidation. Zircon should absorb the owner/generation boundary, not its global-singleton shape. For concurrency, GC and bytecode admission, Unreal VerseVM remains the primary standard.

## 5. Structural optimization plan

### M0: qualify the artifact before timing the VM

Freeze a clean Zr source/build receipt, host/type/opcode schema and product BuildSet. Implement bounded `.zro` header/section/checksum/CFG/type/layout verification before allocation, registration or execution. Shipping runtime must consume an immutable receipt and must not open/compile developer source.

### M1: establish world execution domains

Replace the process mutex safety assumption with the real native thread/context/root contract. Each `ZrVmWorld` or isolated worker generation owns instance shards, scheduler, allocator/heap and GC cooperation. If the native runtime is genuinely process-global and non-reentrant, expose that as an isolated actor with bounded queue/deadline rather than claiming in-process parallelism.

### M2: compile the script/world boundary

Generate immutable callsite/interface tables bound to package, world, schema and generation. A `ScriptWorldTransaction` supplies typed bulk input/reflection state once, records command/event/presentation effects, and commits once at a schedule barrier. Conservative world access becomes a compiled read/write plan; independent shards may then run in parallel and merge deterministically.

For input, compile Vampire and shipping scripts to typed action handles. The frame context resolves input/action service generation once and exposes a borrowed immutable action view or batched query. Raw `key_pressed(String)` remains only a named compatibility path; adding a faster string hash or another snapshot cache is not the target architecture.

### M3: enforce execution, memory and lifecycle budgets

Every call/tick/package receives fuel, wall deadline, cancel, host-call/effect count, cross-boundary bytes and allocation quota. GC pressure uses live/committed/native bytes and allocation rate; safepoints support bounded continuation. Reload closes admission, drains generation leases to deadline, activates a preverified artifact, migrates typed bounded state, publishes atomically and retires old roots after leases release.

### M4: product and tooling qualification

Editor31 consumes the same workspace/build/debug generations for diagnostics, source maps, breakpoint/step/stack/local/evaluate and CPU/allocation/GC/host-call traces. Plugins16 takes ownership of the ten currently ignored Vampire behavior tests and runs them against the real provider. App06 cuts Vampire from four raw WASD calls to compiled action state and validates gameplay/HUD/menu/reload results.

## 6. Dynamic measurement matrix

1. Scale packages, active slots, worlds and instances at `1/100/10k`; run `1/4/16` worker/world owners and report frame p50/p95/p99, throughput, ready/backlog/oldest age, lock wait/hold, context switches and determinism digest.
2. Run `1M` prepared host calls for scalar, named lookup, dense callsite, 1 MiB bytes and typed batch variants. Record callsite resolution, allocations/copied bytes, runtime-context/TLS lookup, Core/service resolution, input locks, callback time and host failures.
3. Compare input queries at `1/4/1k` per frame and pressed controls at `1/64/1,024`. Require snapshot vectors and retained-button clones to remain zero on the concrete path; separately measure raw string parse/allocation, manager resolution, BTree visits, lock wait and compiled-action view cost.
4. Sweep heap/live data, allocation rate and roots at `1/100/10k` packages/worlds; record cycle reason, requested/completed generation, mark/sweep/total pause, overrun, heap/committed/live/fragmentation, RSS and cross-world interference.
5. Exercise compile/load/reload/shutdown, active-call reload, infinite loop, recursion, allocation/host-call storm, backend crash/hang and cancellation. Every operation must end in a typed receipt with bounded queue/bytes/time and last-good recovery.
6. Use WPR/ETW on a current-source Windows product executable for CPU sampling, thread lifetime, context switches, wait/lock, file I/O, working set and energy. RenderDoc is used only when a script transaction changes a visible current-source frame, to correlate presentation correctness; it is not evidence for VM CPU, GC, contention or power.

## 7. Current result

- The 102/102 script Rust files have current-source composite static coverage and a reproducible fingerprint.
- The direct input query is accepted as deterministic work reduction: per-query full pressed-button snapshot copying is gone from `DefaultInputManager`.
- No new production/test code was changed because the remaining hot path belongs to artifact qualification, world execution ownership, typed frame transactions, scheduling and GC/lifecycle control.
- Dynamic acceptance remains pending: current-source Cargo tests, real backend, Vampire, WPR/ETW and product executable evidence are all zero for this pass.
