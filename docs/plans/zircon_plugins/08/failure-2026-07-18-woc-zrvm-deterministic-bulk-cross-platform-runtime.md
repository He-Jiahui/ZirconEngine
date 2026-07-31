---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: woc-zrvm-deterministic-bulk-cross-platform-runtime
origin_plan: docs/plans/woc/00-woc-engine-capability-foundation.md
fixing_plan: docs/plans/zircon_plugins/08-zr-vm.md
origin_child_dir: docs/plans/woc/00
fixing_child_dir: docs/plans/zircon_plugins/08
plan_link_mode: child_record_only
related_code:
  - zircon_plugins/zr_vm_language/plugin.toml
  - zircon_plugins/zr_vm_language/runtime/Cargo.toml
  - zircon_plugins/zr_vm_language/runtime/src/backend.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/instance.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/runtime_owner.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/values.rs
  - zircon_runtime/src/script/vm
tests:
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --features backend-zr-vm woc_deterministic_bulk_tick_contract --locked
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --features backend-zr-vm woc_binary_snapshot_round_trip --locked
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --features backend-zr-vm woc_tick_and_hot_reload_transaction_rollback --locked
  - zr_vm_cli.exe examples/woc/scripts/woc_game/woc_world_state_tests.zrp --execution-mode interp --emit-executed-via
  - zr_vm_cli.exe --compile examples/woc/scripts/woc_game/woc_world_state_tests.zrp --run --execution-mode binary --emit-executed-via
  - zr_vm_cli.exe -e "<two nested custom-class fields, then read the second field twice>"
  - zr_vm_cli.exe examples/woc/scripts/woc_game/woc_m4_nythraxis_state_tests.zrp --execution-mode interp --emit-executed-via
  - zr_vm_cli.exe examples/woc/scripts/woc_game/woc_m4_effect_world_dispatch_state_tests.zrp --execution-mode interp --emit-executed-via
---

# Plugins 08: WOC requires a deterministic bulk and cross-platform ZrVM runtime

## 来源执行者

- 来源计划：`docs/plans/woc/00-woc-engine-capability-foundation.md`
- 来源执行切片：WOC ZrVM-only architecture and MVP foundation assessment
- 修复责任计划：`docs/plans/zircon_plugins/08-zr-vm.md`
- 交接原因：The lowest shared cause is the generic ZrVM plugin backend, value bridge, package lifecycle, and platform support. WOC must consume that backend without adding a game-specific VM adapter.

## 失败现象与复现证据

WOC uses one authoritative deterministic simulation package in offline, client-prediction, server, bot, and headless-RL modes. Its fixed simulation clock is 20 Hz, while presentation normally renders at 60 Hz. The target parity corpus currently contains 51 committed golden traces and requires exact replay of state, event, and RNG draw order.

The current ZrVM plugin is not yet a proven reliable backend for that workload:

- `zircon_plugins/zr_vm_language/plugin.toml` declares `maturity = "experimental"`, both capabilities as `partial`, and only Windows, Linux, and macOS.
- The real backend is compiled only with the opt-in `backend-zr-vm` feature; the default plugin build returns `BackendUnavailable`.
- `ScriptHostValue` arguments and returns support only null, bool, integer, float, and string in the real bridge. `Bytes` are lowered through `String::from_utf8_lossy`, so arbitrary binary snapshots and command buffers are not preserved. `HostHandle` is narrowed into an integer rather than a typed capability handle.
- Every call, GC step, and destruction path is serialized by one process-wide lock. This provides safety for the current process-global binding but has no accepted WOC-scale multi-session throughput or isolation evidence.
- State save/restore is JSON-string based. There is no accepted tick transaction contract that couples instruction/time budget, command validation, rollback, and deterministic diagnostics.
- Current feature evidence covers the Windows real backend. It does not prove Android, iOS, or game-runtime WebAssembly execution of the same package artifact.
- Direct Windows CLI reproduction with `examples/woc/scripts/woc_game` shows exported `container.Array<uint>` typed-module metadata is not round-trippable: an unqualified caller tries to load a module named `Array<uint>`; an explicitly qualified caller reaches runtime verification but rejects `parity/wire.fixture` with `expectedHash=0xa0fbae6b9ff0764c actualHash=0x568205f7a9d5ba75`.
- Cross-module enum-to-`int` conversion reaches `RuntimeError: object meta method is not implemented`, so generated raw protocol ids cannot currently use exported enum values as a reliable boundary.
- Same-signature private free functions can be misbound at runtime (`writeU64 -> writeU32` executed the unrelated `writeLength16` body), while module-global `Array.add` mutations disappear even after binding the array to a local. A single high-level call with a local writer and distinct helper arities passes in both interpreter and binary modes, but that is a WOC-local constraint, not an accepted backend fix.
- A WOC production-protocol reproduction narrowed the helper-receiver defect further: a locally built 32-byte payload had the expected length and endpoint bytes, but `ByteReader.readByte -> this.require(...)` reported truncation even after explicit field initialization and distinct arities. Inlining the identical bounds check and first binding `this.bytes` to a local made the complete encode/decode round trip pass in both interpreter and binary modes (18 kernel assertions). This indicates receiver dispatch/field access through a helper method is unreliable independently of the exported-container ABI.
- Windows assertion failures in these paths can leave `zr_vm_cli.exe` waiting after the parent command times out. Seven WOC-owned orphan processes were identified by exact command line and terminated; no unrelated ZrVM processes were touched.
- A later M3 content-table isolation exposed two additional graph/runtime failures. A missing imported module left `zr_vm_cli.exe` running after the 34-second parent timeout instead of returning a module error. After that process was terminated by exact PID, a content contract that directly constructed and inspected one custom class per stage passed stages 1-5 but crashed at the first aggregated helper path with Windows access violation `LASTEXITCODE=-1073741819`. Replacing the public/object helper path with one scalar `metric` entry plus scalar `derived` formulas made all nine content stages and the aggregate contract pass. Locomotion, targeting, and lifecycle each passed when importing that scalar table in isolation, but loading all three through the existing `kernel/tests` dependency graph crashed before the entry output with the same access violation. Removing the repeated shared content imports restored the complete 20-assertion kernel suite. No ZrVM process remained after the access violations. This is evidence that custom-class helper returns and repeated shared dependency-graph linking are not yet reliable even when the externally visible API is scalar.
- A clean WTR1 helper reproduction under a new `bin-wire-tests` project proves the exported built-in array boundary is still inconsistent in both directions. Exporting `buffer(bool) -> zr.container.Array<uint>` compiled three fresh modules but failed at import with `expectedHash=0x297772b1e317952e actualHash=0x77771ba4efd59396`. Moving array construction into the caller then failed on `writeByte(Array<uint>, uint, int) -> void` even though the diagnostic printed identical hashes: `expectedHash=0x45dfb30e2dd5ab5e actualHash=0x45dfb30e2dd5ab5e`. The process exited normally with code 1; no PID was left behind. This blocks modular typed-wire writers: the only proven WOC-local path remains one high-level call with the writer and bytes kept inside the same module.
- WOC M4 combat rules reproduced the same-signature collision entirely within one module: `spellHitChance(int, int, bool)` and `effectiveDamage(int, int, bool)` have different names and return types, but the latter became an unresolved identifier at its call site. Distinct parameter sequences did not make private `int`-returning helpers callable from the public `float`-returning scalar entry: the compiler reported each helper as missing in sequence until the integer calculations were inlined. The resulting six-parameter public scalar entry still compiled but exited 1 without a structured diagnostic on its first external call. Replacing it with one self-contained five-parameter entry made the same rules pass in interpreter and newly compiled binary modes. This blocks natural domain-shaped Zr APIs even before the plugin boundary.
- The first WOS3 authoritative command-reducer slice reproduced both defects on its natural domain API. Zr required the imported decoded batch type to be explicitly qualified as `binary.TickInput`, then rejected `protocol/binary.decodeFixedTickInput` with `expectedHash=0xafc94f9752c22ef7 actualHash=0xafc94f9752c22ef7`. Replacing only that cross-module custom/array boundary with a state-local `CommandBatch` compiled and entered the reducer, but the instance method then failed at its first later-field read with `GET_MEMBER: missing member 'entityHostile'` in `WorldState.applyCommands`, even though the field is declared and initialized and earlier `this` array fields in the same method were read successfully. This leaves the real decoded fixed-tick-to-world-state path unaccepted; the WOC owner will not substitute fixture replay or a native gameplay reducer.
- WOC M4 reproduced the object-shape defect with the current built CLI `0.0.25-win-debug-MSVC-19.44.35228.0` without plugin or engine involvement. A six-line inline program declared `A`, `B`, and `K`, initialized `K.a = new A()` and `K.b = new B()`, then failed on the first `k.b` access with `GET_MEMBER: missing member 'b'`. Separate two-class and four-class repeated-call controls returned 0, so ordinary multiple parameters alone are not the cause. A fresh, non-incremental compile of `woc_m4_nythraxis_state_tests.zrp` then completed `compiled=2`, executed its first 20 Hz tick and allowed the caller to read the raid HP array, but its second tick entered the transition branch with the same raid argument reporting `GET_MEMBER: missing member 'playerIds'`; after packing the raid into `playerCount + playerData`, the failure became `missing member 'playerCount'`. A final single-class kernel reduced the object to 33 fields and only three container references, preallocated all mutable encounter storage, and placed `combatData` as field zero; a fresh two-module compile still failed on the first `addRaidPlayer` call with `GET_MEMBER: missing member 'combatData'`. Renaming the complete output directory and compiling from an absent target reproduced the same results, excluding stale `.zro` reuse. One CLI process from the minimal nested-field failure remained as PID 45728 with no readable image path or command line; both `Stop-Process -Force` and `taskkill /F` returned access denied. No pass is claimed for this project.
- The independent world-effect dispatch state confirms the defect is not specific to the packed Nythraxis layout. After a WOC-owned direct call-cast compile error was reduced to a typed local, `woc_m4_effect_world_dispatch_state_tests.zrp` compiled all five reachable modules. Interpreter and binary execution constructed `WorldDispatchState` and then failed immediately at `area.source.id` with `GET_MEMBER: missing member 'source'`, where `source` is the first declared field and is explicitly initialized in the constructor. No dynamic pass is claimed.
- The focused ability-admission state adds two smaller language/runtime reproductions. Its constructor could not resolve the already imported `catalog` module at `catalog.count()`, while moving that exact call to ordinary module code and passing the resulting integer into the constructor compiled. After that repair, `actor.cooldowns[index] = value` failed with `SET_MEMBER: receiver must be a writable object member`; binding `actor.cooldowns` to a local array before the same indexed writes made the complete admission contract pass in freshly compiled interpreter and binary modes. These WOC compatibility shapes preserve semantics but do not repair constructor import scope or member-container lvalue behavior for engine users.
- The focused aura state reaches a third object-field type failure before any contract result. Its `AuraSpec.id` and `AuraSpec.kind` fields are declared as `string`, initialized to strings and populated from typed string parameters, yet the interpreter aborts in `execution_dispatch.c:7492` on `opA->type == ZR_VALUE_TYPE_STRING && opB->type == ZR_VALUE_TYPE_STRING` during the first equality guard. Explicitly casting every `Array<string>` element read did not change the failure, narrowing it to custom-object string-field type preservation rather than generic array inference. The CLI returned exit 1 and left no new process; no dynamic pass is claimed.
- The mob-swing affix state independently hits the same `execution_dispatch.c:7492` string-operand assertion on explicitly typed object string state. Its parent command timed out after the assertion and briefly projected PID 44588 with the exact CLI image path; the process exited naturally before the verified termination command ran, and only the older inaccessible PID 45728 remains. In the same matrix, scalar `spell_scaling.contractTest()` exited with Windows access violation `-1073741819` before any result or structured diagnostic, while `casting_state` could not resolve a later-declared `canQueue(CastState)` helper from two class methods until the three-condition predicate was inlined. The inlined casting contract then passed in freshly compiled interpreter and binary modes.

These are generic backend limitations. Implementing a second byte codec, per-game lock bypass, fallback language, or platform-specific WOC script path would hide rather than validate the missing engine capability.

## 最低共享层根因

The neutral `VmBackend` lifecycle exists and the real ZrVM binding is connected, but its production contract is still scalar-oriented and desktop-scoped. The plugin does not yet own a lossless bulk value channel, a deterministic budgeted transaction boundary, a scalable process-global-runtime policy, or the target-platform evidence required by an engine-wide reliable backend.

## 架构修复验收

- Preserve arbitrary snapshot and command bytes losslessly across the neutral VM host boundary. The public contract must not encode binary payloads through lossy UTF-8 or a WOC-only filename/module convention.
- Provide a batch-call path suitable for one 20 Hz simulation transaction: canonical input bytes enter once, and validated command/event/state-digest bytes return once. No 60 Hz per-entity VM calls are required.
- Expose deterministic execution, memory, and GC budgets with structured limited/trap diagnostics. Native host callbacks must participate in the same budget or carry an explicit bounded-cost contract.
- Define and test tick rollback plus tick-boundary hot reload: save, deactivate, load, schema migrate, activate, restore, and rollback on any failure without partially mutating the committed world.
- Replace the unqualified process-global serialization assumption with an accepted policy: safe per-runtime concurrency, isolated workers, or measured serialized scheduling that meets the WOC server gate. Multiple sessions must not share mutable script state accidentally.
- Make missing-module resolution and repeated shared-dependency linking terminate deterministically with a structured diagnostic. Add a graph with three modules importing one scalar-only content module and a custom-class helper-return regression; neither interpreter nor binary mode may hang or access-violate.
- Make exported `container.Array<T>` signatures stable for both return values and parameters. A fresh compile must agree on the import hash, and a reported equality of expected/actual hashes must never be rejected as a mismatch. Cover `Array<uint>` mutation across a two-module boundary in interpreter and binary modes.
- Make imported custom-class signatures and declared instance-field lookup stable in interpreter and binary modes. `binary.TickInput` must import without an equal-hash rejection, and a method that reads an initialized later-declared array field must not require a caller-provided duplicate or field-local compatibility rewrite.
- Make class fields remain addressable after construction and across repeated calls, including multiple nested custom-class fields and container fields passed through free functions. The minimal `K { a: A, b: B }` reproduction must read and mutate both fields in interpreter and binary modes; a single-class kernel with `combatData` as its first field must retain that field in `addRaidPlayer`; and the WOC Nythraxis state must retain the identity and complete member set of each state object across distinct 20 Hz tick call sites.
- Cover first-field lookup on a freshly constructed aggregate state: `WorldDispatchState.source` must remain addressable after its constructor in interpreter and binary modes, and the focused world-effect project must reach its contract assertions rather than fail in `GET_MEMBER`.
- Make imported module aliases available consistently inside constructors, ordinary functions and methods, and make `object.containerField[index] = value` a valid writable lvalue when the field and index types are valid. The natural ability-admission source must pass without moving `catalog.count()` to every caller or first copying each cooldown array into a local.
- Preserve declared scalar field types on custom objects. A constructor-initialized and caller-populated `string` field must reach equality as `ZR_VALUE_TYPE_STRING` in interpreter and binary modes; invalid runtime types must produce a structured diagnostic rather than abort through a C assertion. The focused aura-state project must reach and pass its contract assertions.
- Resolve valid later-declared free helpers from class methods or provide a deterministic compile-time rule that rejects the declaration order without cascading prototype errors. Runtime faults in valid scalar helper graphs, including spell-scaling contracts, must return structured diagnostics and must not access-violate or leave the parent waiting after a C assertion.
- Build and execute the same ZrVM package contract on Windows, Linux, macOS, Android, iOS, and game-runtime WebAssembly before those platforms are declared supported. Packaging-only or language-server WASM evidence is insufficient.
- Promote plugin capability and maturity declarations only after the real backend feature matrix, binary round trip, deterministic replay, lifecycle rollback, and platform smoke tests pass.
- Rerun the WOC 51-trace double-execution parity gate and the server fixed-tick p95 gate upward after the generic backend acceptance suite passes.

## 禁止临时方案

- Do not add Rune, CoreCLR, rustclr, or another language as a runtime fallback.
- Do not add a WOC-specific backend selector, source-path branch, binary codec, lock bypass, or platform implementation inside the engine plugin.
- Do not claim mobile/browser support from Rust source portability, export scaffolding, or a successful compile without executing the real game VM lifecycle.
- Do not weaken exact parity, transaction rollback, or server timing gates to fit the current scalar bridge.

## 修复结果与回传

Open state: `待修复`; no pass is claimed.
