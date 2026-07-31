# WOC engine capability foundation assessment and ZrVM design

> Status: current-head source audit complete; ZrVM-only design approved 2026-07-18; implementation plan active; MVP foundation handoffs open
> Source: `dev/world-of-claudecraft` at `5ef9f7cb21cd8875b6d2c49701015dfcd78de35a`
> Delivery root: `examples/woc`
> Authoritative VM backend: `zircon_plugins/zr_vm_language`
> Clock contract: 20 Hz authoritative simulation, 60 Hz default client presentation
> Handoff link mode: `child_record_only`

## Purpose

This record captures the engine-foundation findings and the durable architecture decision for rebuilding World of Claudecraft one-to-one on ZirconEngine. The authoritative gameplay implementation runs on ZrVM through the existing plugin boundary. Rune, CoreCLR, rustclr, and runtime language fallback are outside the design.

The replication must expose missing engine capabilities. It must not hide them with WOC-only runtime branches, decompressed duplicate assets, hardcoded HUD renderers, or browser/mobile shells that do not execute the game.

## Current decision

ZirconEngine cannot yet complete an end-to-end one-to-one WOC replication. The
current project is valid source and contract work, but not a retained playable
session. A complete replication remains architecturally feasible only after the
generic Runtime 04, Runtime 09, Runtime 10, Runtime 13, Plugins 08, and
Plugins 09 contracts named below are delivered and accepted. The WOC effort must
continue to port source-owned deterministic behavior and schemas in parallel;
it must not substitute a WOC-local VM, math implementation, asset conversion,
UI renderer, or host loop for those foundations.

The WOC-owned current-head execution identity has now been hard-cut in source:
active native protocol, inventory validation, role inspection, trace generation
and parity loading use `reference/current-head` at
`5ef9f7cb21cd8875b6d2c49701015dfcd78de35a` and report `WOS39`. The historical
`reference/` catalog remains archive evidence and historical tools require an
explicit `--historical` opt-in. The cutover materializes and hash-locks all 54
current-head goldens inside `examples/woc`; it never reads a live product role
from `dev/world-of-claudecraft`.

This is static source identity, not current-head gameplay parity. Several
earlier ZrVM modules correctly retain comments recording their historical
source slices, and every such module still needs a behavior-level current-head
rebase plus a real-ZrVM transaction before it contributes acceptance evidence.

## Audit evidence

- WOC current-head manifest: 3,163 TypeScript/JavaScript/Svelte source files, 56,451,702 characters, 1,331 test files, 14,716 test registrations, 67 parameterized test generators, and 54 authoritative parity golden traces.
- ZirconEngine evidence is maintained by module-level source inspection across app, runtime, editor, hub, runtime interface, plugins, examples, and tools; the capability dispositions below name the concrete owning gaps rather than treating prior aggregate file counts as a compatibility claim.
- WOC assets: the current-head catalog records 949 GLBs with 714 animations and 158 skins. Target GLB requirements cover 869 `EXT_meshopt_compression`, 803 `EXT_texture_webp`, and 868 `KHR_mesh_quantization` entries. Optional material extensions include unlit, emissive strength, IOR, and specular.
- WOC product surface: deterministic offline simulation, authoritative WebSocket multiplayer, PostgreSQL persistence, desktop/mobile/browser clients, Svelte administration, headless RL, bots, and external platform integrations all share the same simulation and command vocabulary.

## Authoritative architecture

### Runtime topology

- One WOC gameplay package owns deterministic simulation logic and is compiled for `backend = "zr_vm:project"`. Offline play, client prediction, authoritative server, bots, and headless RL consume the same simulation exports and data schemas.
- The authoritative simulation advances at exactly 20 Hz. A tick receives one canonical input/command batch, executes one budgeted ZrVM transaction, validates the resulting command/event batch, and commits once.
- The client normally renders at 60 Hz from committed snapshots. Interpolation, camera presentation, animation sampling, particles, audio, UI painting, and local visual prediction do not call gameplay scripts per entity or per render frame.
- The VM boundary is bulk and language-neutral: versioned binary snapshots, inputs, commands, events, state digests, and structured diagnostics cross once per tick. ZrVM code does not receive direct mutable access to the engine world.

### Project and ZrVM module ownership

- `examples/woc` is now an ordinary Zircon project rooted by `zircon-project.toml`, with a project-owned ZrVM package, native adapter workspace, generated contracts, reference inventories, and focused test packages. It remains source/static evidence only until the engine foundations below make a retained runtime session possible; it is not an engine builtin, a `zircon_app` special case, or a copy of the original web application.
- The project selects `zr_vm_language` for client, server, editor, and headless roles and discovers one authoritative startup package under `scripts/woc_game` using the existing `plugin.toml -> .zrp -> source/binary/entry` contract.
- The `.zrp` source root is `src/`. `src/main.zr` owns only package lifecycle, state migration dispatch, and the project-level batch tick export. Gameplay behavior is split into imported modules aligned with the target source domains: world/state, commands/events, RNG, movement, combat, casting/effects, mobs/pets, quests, loot/inventory/economy, progression/professions, party/social/PvP, instances/dungeons/raids/delves, and content catalogs.
- Shared protocol and state schemas have one canonical definition and generated/validated projections for ZrVM, Rust host code, network payloads, persistence, and parity traces. Hand-maintained duplicate enums or numeric command tables are forbidden.
- Native Rust code owns engine adaptation, capability registration, binary validation, persistence/network transports, launch roles, and test orchestration. It does not contain a second implementation of combat, progression, quests, economy, encounter rules, or any other authoritative gameplay outcome.

The Vampire example proves the current project manifest, ZrVM package discovery, compilation, and lifecycle call chain. Its entity-by-entity `onUpdate` gameplay and four-active-script-entity hot path are compatibility evidence only; WOC must not inherit that callback topology as its simulation architecture.

### Deterministic tick transaction

1. Freeze the ordered input batch, fixed tick number, deterministic seed state, and previous committed world snapshot.
2. Invoke the ZrVM fixed-update export under explicit execution, memory, host-call, and GC budgets.
3. Decode and validate the returned command/event batch against schema version, entity generation, capability, bounds, and finite-number rules.
4. Apply commands to a candidate world and canonicalize state, event, and RNG draw-order digests.
5. Commit the candidate only when every stage succeeds. A VM trap, budget exhaustion, invalid command, decode error, or digest failure discards the entire candidate and retains the previous committed snapshot.

The simulation may use only injected deterministic time, seeded RNG, canonical entity ordering, ordered or explicitly sorted collections, and fixed protocol versions. Wall clock, ambient randomness, filesystem enumeration order, and asynchronous completion order are not gameplay inputs.

### Failure and recovery

- Client prediction failure requests an authoritative full snapshot and replays retained inputs from the accepted tick. It never commits a partial script result.
- Server tick failure retains the last committed state, records a structured fault with package/build/tick/digest identity, marks the affected world/session as faulted, and stops advancing it. The host supervisor restores a replacement from the last durable snapshot; the failed process does not retry the same tick or continue serving from a stale world.
- Offline tick failure pauses simulation on the last committed state and surfaces the structured fault. The player may restart a fresh offline session or load a corrected package, but the current world does not advance past the failed tick; target-compatible offline world state is session-only.
- Network commands rejected by schema, authority, rate, range, or state precondition produce deterministic rejection events and do not reach mutable simulation state.
- Persistence writes are derived from committed ticks only. Database or external-integration failure cannot retroactively change the simulation result.

### Hot reload and package lifecycle

- Development hot reload occurs only between committed ticks. The coordinator performs `save -> deactivate -> load -> schema migrate -> activate -> restore` and publishes the new generation atomically.
- Any compile, load, migration, activation, or restore failure reinstates the previous generation and state. Save data carries package identity, state schema version, protocol version, and deterministic migration diagnostics.
- Shipping and parity runs pin the ZrVM package artifact and disable source watching. No client or server silently substitutes another language or backend.

### Platform contract

- Windows, Linux, macOS, Android, iOS, and game-runtime WebAssembly must execute the same ZrVM package contract. Host adapters may differ, but gameplay source, bytecode semantics, tick protocol, and committed digests do not.
- A platform is supported only after a real host retains the VM/session, advances fixed ticks, renders or serves the product role, routes input/network events, and shuts down cleanly. Successful packaging or source compilation alone is not acceptance.
- The current ZrVM manifest is `experimental`/`partial`, enables the real binding only through an optional feature, and declares desktop platforms only. Cross-platform reliability remains a Plugins 08 responsibility, not a WOC-local workaround.

## Parity and performance acceptance

- Each of the 54 committed WOC scenarios runs twice from the same seed and input stream. Canonical state, ordered events, RNG draw order, and final digests must match exactly between both runs and the committed reference.
- Trace serialization sorts keys/maps/sets, rounds finite trace floats to `1e-6`, represents non-finite values with explicit sentinels, and uses the target-compatible FNV digest contract. Runtime simulation values are not rounded merely to make traces pass.
- The authoritative server must keep fixed-tick p95 at or below 40 ms, preserving 20 percent headroom inside the 50 ms step. Measurement includes the ZrVM transaction and host bridge, not only native command application.
- The default client presentation target is 60 Hz. Rendering may interpolate across 20 Hz states, but it must never change authoritative outcomes or become a substitute for fixed-step parity.
- Product parity includes behavior, protocol, persistence, UI interaction/accessibility, original asset ingestion, audio, administration, bots/RL, integrations, and real desktop/mobile/browser/server hosts. A desktop-only simulation fixture is an intermediate milestone, not completion.

## Capability disposition

| Capability | Current Zircon evidence | Disposition |
|---|---|---|
| Fixed-step deterministic simulation | ECS schedules, fixed clocks, input recording/replay, clonable world snapshots, dynamic-scene serialization | Host foundation available; authoritative WOC simulation belongs to the ZrVM package and bulk transaction bridge |
| Current-head WOC build identity | Active native protocol/codegen, role inspection, trace symbols and parity fixtures use the materialized `reference/current-head` root, 165 commands, 248 world members, 54 goldens and `WOS39`; historical inputs need explicit archive tooling | Static identity cutover is source-complete; each earlier source-sliced ZrVM module still needs a current-head behavioral rebase and real transaction evidence |
| Reliable ZrVM gameplay backend | Neutral VM lifecycle, state schema/hot-reload coordinator and GC bridge exist; the real binding is an optional feature, all lifecycle/export/GC calls take one process-wide mutex, and the current value bridge accepts primitives, strings and byte arrays only. WOC's current `releaseEmpowered` string and `resurrect_respond` boolean have source-pinned static contracts, but neither may cross as a project command envelope yet. | Blocking foundation gap: lossless structured `CommandValue` bridge, per-session concurrency/isolation, transactional rollback/budgets, enabled reliable backend, and proven mobile/WASM execution. WOC must not reinterpret its archival byte payload as a plugin-boundary substitute. |
| Deterministic scalar math host | Current builtin host modules expose only vector length/dot helpers needed by existing samples | Blocking foundation gap: WOC requires stable target-defined `abs`, `atan2`, `ceil`, `cos`, `exp`, `floor`, `sin`, `sqrt`, and exponent semantics through the plugin host, not a WOC-local approximation |
| Desktop scene rendering | Real wgpu scene renderer, PBR, shadows, post processing, animation pose extraction, skinning, morph and particle paths | Foundation available, subject to target-asset importer handoff |
| Target GLB ingestion | The glTF runtime plugin calls the ordinary `gltf::import` reader with only `KHR_texture_transform`/generic extension features enabled; no meshopt or WebP decoder/projection path is present | Blocking foundation gap for the 869 meshopt and 803 WebP target entries; quantization support also requires direct target-asset proof |
| Runtime retained UI | Rich `UiSurface`, layout, input, accessibility, templates, and rendering exist | Blocking integration gap: project runtime does not own or submit this UI |
| Client networking | Real TCP/UDP/HTTP/WebSocket/RPC/replication/reliable-UDP implementations | Foundation available; WOC protocol and authority logic remain game code |
| Authoritative server host | Fixed-step runtime exists, but dynamic headless uses client target mode and app headless returns after bootstrap | Blocking foundation gap |
| Custom native gameplay package in normal runtime loop | `woc_runtime` defines the neutral transactional VM trait, but its role binaries currently emit only project identity reports and production code has no `WocProjectVm` implementation; current implementations are test doubles | Blocking foundation gap: Runtime 10 host wiring and Plugins 08 reliable project-VM execution are both required |
| Audio assets | Symphonia audio importer and Sound runtime can decode the target MP3 family | Available, with Plugins 02 current Kira validation still external |
| Fonts and localization | WOFF2 decode, shaping, fallback, SDF/MSDF and rich text paths exist | Foundation available |
| Android/iOS/WebGPU/WASM | Export files and packaging contracts exist | Blocking foundation gap: generated lifecycle/input functions are no-ops and no live render/tick host exists |
| Persistence/admin/integrations | No engine database layer is present | WOC-owned application/server code, not a required core-engine abstraction |

## MVP dependency order

1. WOC must retain the active `reference/current-head` build, protocol, identity, trace and golden inputs; historical `reference/` inputs may be read only through explicit archive tooling. Rebase every earlier source-sliced module against current-head before it is counted as behavior evidence.
2. Runtime 04 must import the target GLBs directly, including required compression and texture extensions.
3. Plugins 08 must provide the lossless, budgeted, transactional, and cross-platform ZrVM backend contract consumed by WOC.
4. Runtime 13 must expose the deterministic target-compatible scalar-math host contract through the ZrVM plugin boundary.
5. Runtime 10 must provide a real client/server host contract for the shared ZrVM gameplay package.
6. Runtime 09 must connect project-authored retained UI to runtime input, rendering, and accessibility.
7. WOC may then close a desktop offline MVP: deterministic session picker, one zone, movement, combat, quest, inventory, HUD, session-only world state, persisted settings/keybind preferences, and parity traces.
8. WOC-owned authoritative server/network/persistence code follows without waiting on mobile/browser hosts.
9. Plugins 09 closes actual browser and mobile execution before cross-platform parity is claimed.

## Open foundation handoffs

- Runtime 04: [WOC glTF meshopt/WebP import](../zircon_runtime/runtime/04/failure-2026-07-17-woc-gltf-meshopt-webp-import.md)
- Runtime 09: [WOC project runtime UI bridge](../zircon_runtime/runtime/09/failure-2026-07-17-woc-project-runtime-ui-bridge.md)
- Runtime 10: [WOC runtime host client/server extensibility](../zircon_runtime/runtime/10/failure-2026-07-17-woc-runtime-host-client-server-extensibility.md)
- Plugins 08: [WOC deterministic bulk and cross-platform ZrVM runtime](../zircon_plugins/08/failure-2026-07-18-woc-zrvm-deterministic-bulk-cross-platform-runtime.md)
- Runtime 13: deterministic target-compatible scalar math is an open WOC-originating foundation requirement; the owning plan must publish its canonical child record before MVP acceptance.
- Plugins 09: [WOC mobile/browser host no-op](../zircon_plugins/09/failure-2026-07-17-woc-mobile-browser-host-noop.md)

## Independent WOC work while handoffs are open

The originating WOC effort may continue dependency-independent source work: the current-head hard cutover, exact data schemas, seeded RNG and deterministic ordering contracts, ZrVM gameplay modules, command/event/snapshot codecs, golden-trace adapters, server domain models, protocol fixtures, content catalogs, localization catalogs, and tests for those owners. In particular, WOC may continue complete pure cast/effect/state reducers and source-pinned command contracts while `releaseEmpowered` and `resurrect_respond` remain held at the lossless `CommandValue` boundary; it must not invent a local encoding or accept a no-op command. Rust owns the neutral host adapters, validators, and test harness; it must not become a second authoritative gameplay implementation. WOC must not claim a playable engine MVP until its current-head build identity plus Runtime 04, Plugins 08, Runtime 13, Runtime 10, and Runtime 09 provide executable foundation contracts.

## Approved implementation owner

The user approved this ZrVM-only design on 2026-07-18. The dependency-ordered implementation owner is [01-woc-zrvm-one-to-one-replication.md](01-woc-zrvm-one-to-one-replication.md). It keeps one authoritative gameplay implementation, preserves all 54 reference traces, and separates dependency-independent WOC work from open engine handoffs.

## 状态与产出记录

每个里程碑测试通过后记录一次；实现切片不单独写入产出记录。

| 里程碑 | 范围 | 状态 | 完成日期 | 验证批次 / 残余风险 |
|---|---|---|---|---|
