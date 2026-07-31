# WOC ZrVM One-to-One Replication Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan milestone by milestone in the existing `main` checkout. Do not create worktrees or feature branches. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rebuild `dev/world-of-claudecraft` at commit `5ef9f7cb21cd8875b6d2c49701015dfcd78de35a` one-to-one as a real ZirconEngine project under `examples/woc`, with ZrVM as the sole authoritative gameplay backend and no language fallback.

**Architecture:** One multi-module ZrVM package owns every authoritative gameplay outcome. A project-local native workspace owns generated protocol projections, the lossless batch VM adapter, transports, persistence, platform entry roles, and parity orchestration without duplicating gameplay rules. The simulation commits transactional 20 Hz ticks; the client presents committed snapshots at 60 Hz.

**Tech Stack:** ZirconEngine project/runtime/plugin contracts, ZrVM project packages and `%import` modules, Rust 2021/Serde/Bincode/PostgreSQL/WebSocket adapters, retained `.zui` runtime UI, wgpu assets/rendering, Windows-native milestone validation, Playwright/device/product evidence where required.

## 2026-07-26 current-source rebaseline

The prior `7c10f280eec380e9877e66ce16333089e171fe42` pin is retained only as the
historical source of earlier WOC slicing. The requested target directory now
resolves to `5ef9f7cb21cd8875b6d2c49701015dfcd78de35a`, 440 commits later.
This is a semantic rebaseline, not a count-only refresh: every prior manifest
identity, parity digest, golden result and source-complete statement is
non-acceptance evidence until regenerated against the new pin.

M0 first recreated one versioned inventory at `reference/current-head` with
these audited anchors: 3,163 source files, 245 tracked `src/sim` files, 1,331
`.test.ts` files containing 14,716 test registrations and 67 parameterized
generators, 54 parity goldens, 949 GLBs with 714 animations and 158 skins, 650
UI-flow source rows, 165 dispatch commands (156 client-send and nine
dispatch-only), and 248 `IWorld` members (67 data, 181 methods, 28 facets).
Historical `12,842`/`1,581`/`1,043` totals and the older manifest's
`702`/`157` figures must not be copied forward by hand.

All existing WOS2-WOS14 modules, catalogs, focused ZrVM runs and native
adapter tests remain valuable authored source. They are classified only as
`source-authored` or `static-checked` until a production `zr_vm:project`
transaction and the matching current-head golden prove them dynamically. An
accepted milestone requires, in order: source authored, static contract
checked, real-ZrVM transaction, exact current-head double-run golden, and the
relevant product/platform journey. A later state never follows from an earlier
one by description alone.

The current-head delta is assigned explicitly rather than hidden in older
milestones: M4 adds fire/frost/chronomancy, empowered casts, damage history,
group targeting, resurrection and warrior/hunter mechanics; M5 adds Talents
V2 row choice, loadouts and save migration; M6 adds Card Duel; M5/M10 add
profession identity, combo eligibility, masterwork and attunement; M8 adds the
domain HUD-controller structure, Welcome, stance/proc/target-of-target UI and
software-render/fail-soft behavior; M9/M12 add 165-command interaction-result
DTOs and the headless level/talent reset plus dynamic quest observation.

`reference/current-head/delta_from_7c10.json` is the canonical migration
queue. It records nine added commands (`card_forfeit`, `card_queue_join`,
`card_queue_leave`, `pet_auto_water_jet`, `pet_water_jet`, `play_card`,
`releaseEmpowered`, `resurrect_respond`, `selectTalentRow`), 15 added world
members, one added facet, three added goldens (`card_duel`,
`professions_craft`, `warrior_row_capstones`) and 11 added GLBs. It also proves
that all 51 historical goldens changed, so no historical trace result may be
carried into current-head acceptance.

### Current-head execution identity hard cutover (implemented source configuration; dynamic acceptance pending)

`reference/current-head` is now the active static authority:
`woc_protocol::REFERENCE_COMMIT`, `woc_contract_codegen::REFERENCE_COMMIT`,
`woc_runtime::inspect_project`, the materialized native parity suite and the
trace-symbol generator all use the current root. `woc_protocol` now reports
`WOS65`, matching `main.zr::stateSchema()`. This was not a constant swap: the
implementation rebuilt the seven inventory catalogs, regenerated command-payload
and trace-symbol projections, materialized and SHA-256-locked all 54 current
goldens, and made legacy inventory generation opt in through `--historical`.

The delta still changes nine command rows, 99 command positions/metadata rows,
15 world members, 90 existing world-member source signatures/locations, all 51
historical golden records, and adds three new current-head goldens. Earlier
source-sliced ZrVM modules retain their historical provenance comments until
their behavior is actually rebased; they are not current-head parity evidence.

The future cutover must (1) retain the historical catalog as explicitly named
archive evidence, (2) make a single current-head manifest root the source for
native protocol/code generation, ZrVM projections, role identity and parity
orchestration, (3) regenerate every affected projection and schema fingerprint,
and (4) reject a mixed-pin artifact before any VM invocation. It must then prove
that all active command/world/trace identities resolve to 165/248/54 against
`5ef9f7cb21cd8875b6d2c49701015dfcd78de35a`. This is WOC-owned M0/M1 work, not
an engine workaround, and cannot be accepted from source inventory alone.

| Active path | Current observed state | Required hard-cutover state |
|---|---|---|
| Reference inventory and Rust codegen | `reference/current-head` and `woc_contract_codegen` validate 3,163/165/248/54 at the requested pin; legacy generation requires `--historical` | Keep generated inventories and their fixed-current checks fresh; behavior-bearing ZrVM modules still rebase independently |
| Native protocol identity | `woc_protocol` reports the current commit and `WOS65`, matching the package writer; generated commands carry 165 current-head rows | Compile-time identity tests must reject any future mixed row set |
| Project role inspection | `woc_runtime::inspect_project` reads the current-head manifest and compares it to the current Rust identity | A real client/server/bot/headless host must execute this identity through ZrVM once Runtime10/Plugins08 land |
| Native parity suite | `woc_parity` loads the materialized, hash-locked 54-file current-head suite | No runtime path reads `dev/world-of-claudecraft`; exact double-run output remains unproven until a real ZrVM transaction exists |

The source cutover materialized pinned current-head trace inputs inside
`examples/woc`, made current-head the default reference-root descriptor,
regenerated protocol/ZrVM/native projections, aligned native WOS metadata with
the package writer, and added focused identity and fixture guards. Historical
data remains readable only through explicitly named historical tools and tests,
never through an active current-head product role. A real transaction and
current-head double-run are the next, separately blocked, acceptance gate.

Five owner fixes jointly gate desktop MVP: Runtime 04 asset import, Runtime 09
project-owned retained UI, Runtime 10 live project client/server host, the
Runtime script-host scalar-math surface, and Plugins 08 reliable lossless
transactional ZrVM. Plugins 09 additionally gates Android, iOS and
browser/WebGPU. WOC may continue independent schemas, catalogs, ZrVM modules
and fixtures, but may not replace any of these with a game-local workaround or
call an MVP/playable result complete while they remain open.

The scalar-math handoff is a distinct current-source execution blocker. The
runtime registers only `zr.zircon.math` and currently exposes
`vec3_length`/`vec3_dot`; it does not expose scalar `sqrt`, `sin`, `cos`,
`floor`, or `atan2`. WOC migrated its 36 provisional `zr.math` import sites to
the registered `zr.zircon.math` identity, but all scalar calls observed there
remain unavailable: `abs`, `atan2`, `ceil`, `cos`, `exp`, `floor`, `sin` and
`sqrt`. Chronomancy Aether Surge also has an exact exponentiation requirement
that must be served by a returned public API or a real-ZrVM-proved equivalent.
The shared Runtime/ZrVM owner must publish a stable, deterministic scalar-math
host API (including its source-level module identity and cross-platform
precision contract). WOC will not create a project-local math module,
approximate the functions, or introduce a second gameplay-rule owner.

---

## Authoritative scope

The approved architecture lives in [00-woc-engine-capability-foundation.md](00-woc-engine-capability-foundation.md). This plan does not redefine completion around an intermediate sample. Completion requires the full target product surface:

- deterministic offline simulation and client prediction;
- authoritative multiplayer server, realm/session lifecycle, reconnect and persistence;
- desktop, Android, iOS, WebGPU/WASM, headless server, bot and RL roles;
- login, character creation/selection, gameplay HUD, inventory, quests, talents, map, chat, mail, bank, market, guild/social, party/raid, PvP, settings and accessibility;
- Svelte-admin-equivalent operations UI, content/editor workflows, guide/wiki-facing content and external integrations;
- original content, localization, audio, fonts, shaders and 949 GLB assets ingested from their checked-in bytes;
- all 54 parity scenarios, 165 command paths, 248 `IWorld` members across 28 facets, and a disposition for every current-target test case computed by M0.

## Fixed contracts

- **Backend:** `zr_vm:project` only. Rune, CoreCLR, rustclr, embedded JavaScript and runtime backend fallback are forbidden.
- **Clock:** authoritative simulation is exactly 20 Hz (`dt = 0.05`). Default client presentation is 60 Hz and never changes authoritative results.
- **Boundary:** one canonical input batch enters ZrVM and one command/event/state batch leaves per tick. Per-entity 60 Hz VM calls are forbidden.
- **Identity:** every package/host identity pins the source commit, structural contract fingerprint, command-catalog SHA-256, command-payload-schema SHA-256 and world-state schema before a tick can be considered compatible.
- **Transaction:** trap, budget exhaustion, invalid bytes, invalid command, non-finite gameplay value or digest mismatch rolls back the complete tick.
- **Determinism:** injected fixed time, seeded RNG, canonical entity order, explicit collection ordering and exact protocol versions are the only accepted inputs.
- **Parity:** every scenario runs twice and both traces must exactly equal each other and the pinned reference. Trace-only finite floats use `round6`; simulation values are not rounded to manufacture parity.
- **Source ownership:** Rust may validate, transport, persist and present results. Combat, movement, progression, quests, economy, encounter outcomes and other gameplay rules exist only in ZrVM.
- **Foundation gaps:** WOC publishes and consumes cross-plan fixes. It never repairs Runtime 04/09/10, the Runtime script-host scalar-math surface, Plugins 08 or Plugins 09 in game-local code.
- **Current-head ownership:** `IWorld` rows must be classified as authoritative simulation, client projection, service/online-only or presentation. Promise-shaped target interactions cross the boundary as deterministic command-result/event DTOs, never as a host Promise inside ZrVM.

## Planned project ownership

```text
examples/woc/
  README.md
  LICENSES.md
  zircon-project.toml
  contracts/
    manifest.json
    commands.json
    events.json
    snapshot.json
    save_state.json
    network.json
    rl.json
  reference/
    current-head/{source_manifest,command_catalog,world_api_catalog,test_catalog,parity_scenarios,asset_catalog,ui_flow_catalog}.json
    command_catalog.json
    world_api_catalog.json
    test_catalog.json
    asset_catalog.json
    ui_flow_catalog.json
  scripts/woc_game/
    plugin.toml
    woc_game.zrp
    src/main.zr
    src/generated/contracts.zr
    src/protocol/{codec,commands,events,snapshot}.zr
    src/sim/{clock,entity,world,rng,systems}.zr
    src/sim/{movement,combat,casting,effects,mobs,pets}.zr
    src/sim/{quests,loot,inventory,economy,progression,professions}.zr
    src/sim/{party,social,pvp,instances,dungeons,raids,delves}.zr
    src/content/{classes,items,skills,talents,zones,npcs,quests,encounters}.zr
  native/
    Cargo.toml
    crates/woc_contract_codegen/
    crates/woc_protocol/
    crates/woc_parity/
    plugins/woc_runtime/
    apps/woc_client/
    apps/woc_server/
    apps/woc_bot/
    apps/woc_headless/
  assets/{audio,data,fonts,models,scenes,shaders,textures}/
  ui/{components,screens,themes}/
  tests/parity/golden/
  tests/parity/scenarios/
  tests/product/
  tests/platform/
```

`contracts/*.json` is the single editable protocol truth. `woc_contract_codegen` writes checked-in `src/generated/contracts.zr` and `woc_protocol/src/generated.rs`; a drift test rejects manual edits or stale projections. `reference/*.json` records the pinned upstream surface and hashes rather than becoming a second implementation.

The nested `examples/woc/native/Cargo.toml` is an independent workspace. It uses path dependencies on ZirconEngine packages but is not added to the fixed engine root workspace and does not introduce a WOC package into `zircon_runtime` or `zircon_app`.

## Coordination rules

- At every milestone start, query the six WOC-origin handoffs and scan the owning child plan for returned `fixed-*` artifacts.
- A handoff blocks only the dependent testing gate. Continue source, fixtures, schemas, content and other independent implementation slices.
- Do not modify another plan's owner modules from this session. New shared failures receive a canonical `failure-*` handoff in the lowest fixing child.
- During implementation slices use formatting, source guards, JSON/TOML parsing and `git diff --check`. Cargo compile/test runs belong to the named milestone testing stage.
- All Cargo validation runs Windows-native through the coordinator/validator and an approved `D:`, `E:` or `F:` target root. Linux and device validation occurs only in the milestones that explicitly require those platforms.
- Each accepted milestone adds exactly one concise row to `## 状态与产出记录`; do not add per-slice rows or command transcripts.

## M0 - Project identity and pinned reference inventory

**Goal:** Establish a valid, standalone WOC project and a machine-readable inventory that prevents scope shrinkage.

**Dependencies:** Approved design only; no shared foundation handoff is required.

**Implementation slices:**

- [ ] Create `README.md`, `LICENSES.md` and `zircon-project.toml` with client/server/editor/headless role selections, `zr_vm_language`, project-local `woc_runtime`, original source attribution and the pinned commit.
- [ ] Create the independent `native` workspace with empty owner crates/apps named in the project tree; each crate root documents its allowed responsibility and forbids gameplay rules in Rust.
- [ ] Create `scripts/woc_game/plugin.toml`, `woc_game.zrp` and a multi-module `src/main.zr` lifecycle shell using `source = "src"`, `binary = "bin"`, `entry = "main"` and `backend = "zr_vm:project"`.
- [ ] Generate and byte-check the already-materialized `reference/current-head/source_manifest.json`, sibling catalogs and `delta_from_7c10.json` for `5ef9f7cb21cd8875b6d2c49701015dfcd78de35a`, including SHA-256 identities for the target package manifest, parity sources and golden directory. They supersede, but do not erase, the historical `7c10` catalogs.
- [ ] Materialize catalogs for all 165 commands, 248 `IWorld` members/28 facets, current-target test cases, 54 parity scenarios, 949 GLBs and product UI flows. Every row records upstream owner, WOC owner and one of simulation/client/service/presentation ownership classes.
- [ ] Add reference-inventory tests that reject missing, duplicate, renamed or count-drifted rows and verify `examples/woc` contains no symlink or runtime dependency on the original web application.

**Testing stage - M0 structure and provenance:**

- Parse every new TOML/JSON file with structured parsers, run the reference inventory tests in `woc_contract_codegen`, and run package discovery against the ZrVM manifest without requiring a live game tick.
- Diagnose count/hash failures from the pinned target first. Update the catalog only when the target evidence proves the previous inventory wrong; never change expected totals to fit incomplete extraction.

**Exit evidence:** Valid project/package manifests; exact current-head inventory counts; pinned source hashes; no target-source runtime dependency.

## M1 - Canonical contracts, generated projections and parity harness

**Goal:** Define the neutral bytes and trace semantics used by every later gameplay, network, persistence and RL milestone.

**Dependencies:** M0.

**Implementation slices:**

- [ ] Define versioned contract manifests for entity ids/generations, fixed tick input, commands, events, world snapshots, save state, network envelopes and RL observation/action batches.
- [ ] Implement deterministic code generation into ZrVM and Rust with stable field ids, enum ids, bounds, finite-number policy, schema fingerprints and reserved-id rejection.
- [ ] Implement `woc_protocol` binary framing, decode limits, canonical ordering, schema/version checks and structured errors. The codec must preserve arbitrary bytes and may not use lossy UTF-8.
- [ ] Implement target-compatible Mulberry32 known vectors, FNV-1a event/RNG folding, sorted map/set projection, `round6` trace formatting and explicit non-finite sentinels.
- [ ] Copy the 54 current-head golden JSON files byte-for-byte into `tests/parity/golden` and create one scenario manifest row per golden with source scenario, coverage assertions and file digest. All 51 historical golden copies must be rehashed because each changed in the current-head range.
- [ ] Implement `woc_parity` trace reading, canonical comparison, first-difference diagnostics, duplicate double-run comparison and guarded golden update mode disabled by default.
- [ ] Add contract drift, malformed length, unknown id, oversized payload, non-finite value, ordering, hash-vector and golden-inventory tests.

**Progress evidence (not exit evidence):** The source-pinned
the historical `7c10` `reference/command_payload_coverage.json` now accounts for all 156 target
commands against the neutral transport schema: 22 commands have an explicitly
bounded descriptor (21 observed client sends and one dispatch-only command),
126 observed client sends retain only their verified source field shape, and
eight dispatch-only commands remain unmapped. The generator rejects source,
catalog or descriptor drift; this is an admission ledger, not proof that a
command has a ZrVM reducer or a real-M2 execution path.

**Testing stage - M1 protocol and trace foundation:**

- Run one Windows-native package check for `woc_contract_codegen`, `woc_protocol` and `woc_parity`, followed by their focused contract/vector/golden suites.
- Generate both language projections twice and require byte-identical output. Intentionally stale one generated projection in a test fixture and require the drift gate to fail.

**Exit evidence:** Reproducible generated Rust/ZrVM contracts; lossless bounded codec; target-compatible RNG/hash vectors; exactly 54 immutable current-head reference goldens and a functioning comparison harness.

## M2 - Transactional ZrVM package and neutral runtime adapter

**Goal:** Execute one empty-but-real 20 Hz world transaction through the production ZrVM plugin and commit or roll back atomically.

**Dependencies:** M1 and the returned Plugins 08 WOC handoff.

**Implementation slices:**

- [ ] Implement `main.zr` lifecycle exports, state schema identity, save/restore envelope and one project-level `fixedTick(input_bytes)` export that delegates to imported protocol/world modules.
- [ ] Implement `woc_runtime` as a project-local native plugin that registers a versioned batch tick host contract, owns candidate/committed snapshots and exposes no direct mutable world handle to scripts.
- [ ] Implement execution, memory, host-call and GC budgets; map limited/trap/decode/validation failures into structured `WocTickFault` values.
- [ ] Implement candidate apply/commit, complete rollback, faulted server session, paused offline session and client full-snapshot recovery state machines.
- [ ] Bind tick-boundary hot reload to the existing coordinator with save/deactivate/load/migrate/activate/restore/rollback generation semantics.
- [ ] Add real-backend tests for empty commit, command rejection, malformed bytes, VM trap, every budget limit, rollback equality, generation rollback and state schema migration.

**Testing stage - M2 real VM transaction:**

- Run the nested native workspace check plus focused `woc_runtime`/`woc_protocol` tests, then the real `backend-zr-vm` transaction suite with the coordinator-provided binding environment.
- Run identical input twice from identical snapshots and require byte-identical output, diagnostics and digest. Inspect the committed snapshot before and after every injected failure.

**Exit evidence:** A real package loads through `zr_vm:project`, executes one batch tick, preserves arbitrary bytes, enforces budgets and proves commit/rollback/hot-reload atomicity.

## M3 - World kernel, entity roster, RNG and locomotion parity

**Goal:** Port the minimal deterministic world kernel and close the four foundation scenarios.

**Dependencies:** M1; M2 is required for the full testing stage but not for module-local source work.

**Scenario gate:** `entity_roster`, `mob_locomotion`, `mob_targeting`, `mob_lifecycle`.

**Progress evidence (not exit evidence):** Roster, locomotion, targeting and
lifecycle focused trace projects pass from absent binary outputs in interpreter
and freshly compiled binary modes. Their stable payload length/digest pairs are
`6680/3966473180`, `19162/2666418349`, `24900/986896405` and
`19803/971948515`. The separate target-selection state reaches
`GET_MEMBER: missing member 'near'`. The four real-M2 double-run golden
comparisons remain open, so M3 is not complete.

**Implementation slices:**

- [ ] Implement entity allocation/generation, player metadata, component/state stores, ordered query sets, spawn/despawn/respawn and snapshot copy semantics in ZrVM modules.
- [ ] Implement fixed clock, shared RNG observer, construction seed handling, movement integration, bounds, facing, target selection, aggro/leash and mob lifecycle systems.
- [ ] Port the four scenario drivers and their coverage assertions without translating expected results into special-case branches.
- [ ] Add module tests for generation reuse, sorted iteration, snapshot isolation, RNG draw order, boundary movement, target ties, respawn and despawn cleanup.

**Testing stage - M3 kernel parity:**

- Run module tests and execute all four scenarios twice through M2. Compare complete canonical traces to their pinned goldens and run the corresponding target coverage assertions upward.
- Debug from entity/RNG/clock support before changing scenario drivers or trace formatting.

**Exit evidence:** Exact double-run and golden parity for all four named scenarios with no harness exclusions added.

## M4 - Combat, casting, effects and class parity

**Goal:** Port shared combat resolution and class/encounter mechanics covered by sixteen historical scenarios plus the current-head fire, frost, chronomancy, empowered-cast, damage-history, group-targeting, resurrection, warrior-stance and hunter-trap contracts.

**Dependencies:** M3.

**Scenario gate:** `c3_aura_runner`, `c4a_casting_lifecycle`, `c4b_effect_dispatch`, `c5_auto_attack`, `affix_mob`, `mob_swing_affixes`, `hit_rating_heroic_geared`, `hit_rating_heroic_ungeared`, `solo_mage`, `solo_rogue`, `solo_warrior`, `multi_class_frenzy`, `multi_class_heal`, `paladin_consecration`, `drowned_litany`, `nythraxis_full_pull`.

**Implementation slices:**

- [ ] Implement stats, hit/crit/avoidance, damage/heal, threat, resources, cooldowns, global cooldown, auto attack and death credit.
- [ ] Implement cast lifecycle, interruption, channels, auras, periodic effects, effect dispatch, ground AoE, dispel and deterministic event ordering.
- [ ] Port class, skill, talent, equipment, affix and encounter content required by the current-head scenario manifests, including fire/frost/chronomancy, frozen orb, ring of frost, temporal hourglass, rewind, heroic leap, hunter trap and warrior stances.
- [ ] Port scenario drivers/coverage assertions and add edge tests for simultaneous lethal events, mid-cast death, target invalidation, periodic expiry, hit caps and effect recursion limits.

**Progress evidence (not exit evidence):** All sixteen M4 scenario names now
have dependency-independent ZrVM source contracts. A current-source matrix from
absent binary outputs passes all eight scalar projects in interpreter and
freshly compiled binary modes: 16/16 runs return `1`/exit `0`. This includes the
converged `auras`, `casting` and `auto_attack` generic rule entries. Natural
object/container execution still exposes the open Plugins 08 field-visibility
defect described below.
Accepted real-M2 exact-golden coverage remains 0/16, so none of the M4
implementation slices or exit gates are complete.

### M4 scalar regeneration kernel

**Goal:** retain the source's deterministic two-second regeneration ordering as
an executable Zr leaf without misrepresenting it as full WOS aura authority.

- [x] Model per-aura, live-resource Lifesap with source per-value clamping and
  stun suppression; apply it before mana/energy/rage recovery.
- [x] Apply the source global `manaRegenPct` multiplier and post-passive-health
  `secondWindPctPerSec` threshold/heal order, before independent food/drink
  consumption slots.
- [x] Add fixed-source order and scalar contract coverage for the new fields.
- [ ] Integrate regeneration into WOS only with the complete ordered aura,
  control, talent and event-state transaction. The current leaf remains
  independent and does not make the M4 scenario gate accepted.

### M4 deferred projectile travel kernel

**Goal:** preserve renderer-aligned, homing projectile arrival as an executable
world-level leaf without pretending that it is yet wired into ability resolution.

- [x] Retain the current source constants, horizontal `sqrt` homing step and
  `max(reach, tickStep)` snap-to-target arrival rule.
- [x] Model a pending projectile's source/target identity, position and TTL;
  emit only `continue`, `impact` or `fizzle` so the eventual world transaction
  remains the sole owner of damage, RNG and event mutation.
- [x] Preserve source ordering: dead/despawned endpoint fizzle, live-target
  homing, immediate arrival, then TTL decrement and forced impact. The focused
  project covers axis/off-axis movement, moving targets, both fizzle sides and
  an outrunning target's deadline impact.
- [ ] Connect the pending collection to the authoritative WOS cast/effect
  dispatcher only after structured entity lookup, ordered pending rows and
  transactional impact resolution are available. This leaf does not make the
  M4 scenario gate accepted.

### M4 world-boss participation core

**Goal:** retain the deterministic non-loot core of the current world-boss
source without fabricating a local item, scheduler or combat authority.

- [x] Project Thunzharr's spawn/HP constants and the shared `worldboss:`
  lockout identity with `i64`-backed Zr `int` reset instants.
- [x] Preserve pet-owner threat credit, leaving-player filtering, entity-id
  sort order, permanent damager union for death-time personal-loot eligibility,
  full evade/reset cleanup and source-correct only-increasing HP expansion.
- [x] Cover source lockout boundary semantics, an owned-pet threat row,
  contributor ordering, dead-threat-style retention through the permanent
  damager roster, cap behavior and a reset wipe.
- [x] Model the per-boss scheduler's boot cadence, corpse-expiry removal and
  single-interval advancement before an empty-slot spawn request. Entity
  allocation/drop and broadcast remain explicit host-owned requests.
- [ ] Integrate source scheduler/spawn, boss abilities, corpse interaction and
  per-contributor roll groups only when the full live-entity, M5 item content,
  personal-slot and WOS transaction authorities are available. This core does
  not claim a playable world-boss encounter.

The WOS9 candidate-state slice persists auto-attack and per-actor command
sequence state and implements the first real authoritative `target`, `attack`
and `stopattack` reducer, including melee pre-aggro and threat mutation. It also
persists the bounded movement-frame acknowledgement, accepted tick and held
input, then applies the existing source-ordered player-motion transition inside
the same candidate transaction. Its payload schema is generated into Zr and Rust
from one pinned contract. This is not accepted evidence: the natural
`binary.TickInput` import is rejected with equal expected/actual signature hash,
and the state-local compatibility probe then reaches the separate `GET_MEMBER`
receiver defect recorded under Plugins 08. The native host still owns only
atomic commit/rollback and does not contain a fallback gameplay reducer.

The package-facing `stateSchema()` now reports `world_state: WOS65`, matching the
state writer's schema-65 envelope while retaining WOS2-WOS64 decoder compatibility.
WOS41 retains WOS40's source-generated active-form code and parked mana maximum,
then adds the M5 baseline profile used by source-ordered derived-stat recomputation;
WOS42 appends its deterministic spell-haste final-column owner, WOS43 appends
bounded M5 helmet/feet/mainhand identities that rederive the equipped contribution,
WOS44 appends persistent M5 stack, bag and copper state, WOS45 reserves mainhand
`255` for explicitly unarmed while retaining `0` as the source-start weapon identity,
and WOS47 appends the source vendor/buyback scalar partition after the WOS44 stacks.
WOS48 derives `q_boars` progress from scalar `boar_hide` inventory, WOS49 routes
the existing `discard` command through that same authority, WOS50 persists the
nullable Eastbrook idle-wander target after the WOS23 timer while restoring the
post-constructor camp-spawn RNG cursor before live wander draws, WOS51 adds a
per-corpse personal quest-item slot for the retained Eastbrook boar-hide loop,
WOS52 adds source-visible corpse copper plus six ordered shared item slots, WOS53
appends the launch-order Hunter Auto Shot queue after the entity/Card Duel
sections, and WOS54 preserves the pending projectile's one-byte school code.
Each row retains source/target identity, homing x/z, ttl, wand marker and
captured profile across rollback. This closes the source Mage/Priest/Warlock
wand profiles plus Druid caster/Moonkin nature wand while Bear/Cat fall through
to melee; Travel and Fireball travel form reject attack arming and cancel
auto-attack. WOS53 Hunter rows decode as physical. WOS55 appends one durable
queued-on-swing row per entity after the projectile section, allowing the direct
offline Heroic Strike/Raptor Strike path to survive encode/decode, toggle and
consume its source-equivalent billing state on the next mainhand attempt.

### WOS55 queued-on-swing closure (delivered)

**Goal:** make the already-pure Heroic Strike/Raptor Strike branch a real world
transaction rather than a local `AutoActor` demonstration. The source queues an
on-next-swing ability without billing it, clears the queue on the next mainhand
swing whether it resolves or fizzles, applies a free/cheap modifier at billing,
and arms its cooldown only after a paid weapon-damage effect resolves. Offhand
swings remain ordinary white attacks.

**Delivered scope:**

- [x] Append a WOS55 tail after the WOS54 projectile section, one canonical
  `{abilityCode, free, costMultiplier}` row per entity. WOS2-WOS54 must decode
  the empty queue (`0`, `false`, `1.0`); a nonempty row must name a current-known
  M4 `onNextSwing` ability.
- [x] Project that row into the direct offline `AutoActor`, use source-equivalent
  `ceil(cost * multiplier)` at mainhand consumption, clear every queue modifier
  on that attempt, and write the resolved cooldown back to the authoritative
  sparse ability-cooldown table before the next command is admitted.
- [x] Route only source-admitted, known, in-range, hostile-target Heroic Strike
  and Raptor Strike slot/cast commands through the world reducer. Preserve the
  source toggle-off behavior and its `startAutoAttack` side effects without
  creating a second aggro implementation.
- [x] Keep talent/aura producers of free or cheap casts, proc chains, and the
  general ability dispatcher as later owners. WOS55 carries their exact durable
  values but does not invent those producers.
- [x] Reset the queue atomically on player death, spirit release and resurrection,
  with the retained global death sweep repairing every terminal row. This prevents
  source one-shot billing modifiers from surviving a lifecycle transition.

**Evidence:** the focused state self-test covers queue, toggle, WOS55
encode/decode, billing, cooldown, offhand exclusion, WOS54 migration and terminal
cleanup; the WOS55 guard compares command, swing, death and spirit source paths,
and all existing WOC static guards pass. This remains implementation evidence only
until the Plugins08 `zr_vm:project` backend executes the package transaction.

### WOS56 Taunt runtime closure (delivered, no schema bump)

**Goal:** connect Warrior Taunt's already-persisted threat and forced-target inputs
to the retained offline Eastbrook world without changing the WOS55 envelope. The
source raises the caster to the current threat-table maximum, forces that target for
three seconds, wakes an idle mob without a social pull, resumes a fleeing mob's
attack, and consumes neither resource nor global cooldown.

**Delivered scope:**

- [x] Admit only the current-known, source-admitted level-5 Warrior `taunt` for
  the offline primary player, a live ordinary Eastbrook hostile, M4's eight-yard
  range, and the source-facing admission. Preserve its zero resource cost and
  off-GCD state while writing the source ten-second cooldown after a valid effect.
- [x] Route both known-ability slot selection and an exact typed `cast` identifier
  with its explicit target through the same reducer. The idle branch invokes the
  existing aggro state with `social=false`; chase/attack retarget directly; flee
  clears its timers and becomes attack.
- [x] Advance the persisted forced-target timer once at the 20 Hz Eastbrook mob
  update. A live forced player owns aggro during the positive window; expiry clears
  only the forced target, matching the source's retained aggro behavior.
- [x] Add a focused state test for idle no-social activation, top-threat lift,
  encode/decode persistence, typed target dispatch, cooldown rejection, flee
  recovery, and forced-target expiry. `wos56_taunt_static_guard.mjs` pins the
  source definition, effect dispatch, reducer and timer paths.

**Boundary:** retained Eastbrook wolf/boar rows have no static-occluder state, so
their source outdoor line of sight is trivially clear. General map line-of-sight
queries, target switching/pull-over, pets, bosses, dummies and `ignoreTaunt`
templates remain later world-combat owners; this slice does not claim them.

**Evidence:** all WOC static guards, including WOS56, pass against the pinned
source. It remains source/static implementation evidence until Plugins08 provides
the real `zr_vm:project` transaction backend.

### WOS57 Sinister Strike runtime closure (delivered, no schema bump)

**Goal:** make Rogue Sinister Strike an authoritative retained-world weapon
transaction instead of leaving its M4 `weaponStrike` definition in generated data
only. The source admits a live hostile in melee range and facing arc, charges 45
energy, applies the Rogue one-second GCD, resolves one shared weapon hit, enters
combat even on a miss or dodge, and grants one character-bound combo point only
when that hit lands.

**Delivered scope:**

- [x] Admit only the current-known, source-admitted primary Rogue against the
  retained ordinary Eastbrook wolf/boar target subset. Use the source zero-range
  melee fallback (five yards), facing admission, zero cast time, 45 resource cost
  and one-second Rogue GCD; reject every failed gate before any state mutation.
- [x] Reuse `autoAttackState.meleeSwing` exactly once rather than advancing a
  white-swing timer. Commit its authoritative RNG cursor, combat/HP result and
  boar post-hit reaction in the existing target-death/loot/XP order, including
  the source physical-damage threat increment. An alive idle target starts the
  source social aggro transaction; a live active target receives only the
  source's missing-aggro-target backfill.
- [x] Persist source character-bound combo state with the existing
  `entityComboPoints`/`entityComboUntil` columns: landed strikes add M4's one
  point with a cap of five and restamp 30 seconds; the retained player tick clears
  an expired point pool after player actions but deliberately retains the stale
  timestamp. Player terminal cleanup also clears the point pool, matching source
  death behavior.
- [x] Route both known-ability slot selection and exact typed `sinister_strike`
  casts, including explicit typed targets, through the same reducer. Add focused
  state coverage for damage/cost/GCD, idle social aggro, combo persistence,
  typed dispatch, GCD rejection, expiry and death cleanup. The
  `wos57_sinister_strike_static_guard.mjs` pins the source definition, casting
  gates, weapon dispatcher and combo reducer.

**Boundary:** this closure does not generalize line-of-sight geometry beyond the
retained open Eastbrook rows and does not claim stealth, poisons, gear/talent
modifiers, parry/block, finishers, combo-spender behavior, pets, PvP or arbitrary
mob/template support. Those paths require their own persistent aura/combat and
targeting owners.

**Evidence:** all retained WOC static guards, including WOS57, pass against
source `5ef9f7cb21cd8875b6d2c49701015dfcd78de35a`. Dynamic ZrVM execution remains
an infrastructure validation dependency: no executable `zr_vm:project` backend
is available to this session, and the Cargo lane is owned by a concurrent session.

### WOS58 Eviscerate runtime closure (delivered, no schema bump)

**Goal:** make Rogue Eviscerate consume the retained character-bound combo pool
through a real authoritative damage transaction. The source admits the same
melee enemy gate as Sinister Strike, charges 35 energy, applies the one-second
Rogue GCD, calculates a physical finisher from the points actually spent, and
clears that pool only after its damage effects complete.

**Delivered scope:**

- [x] Admit only a current-known, source-admitted primary Rogue with at least
  one combo point against a live ordinary Eastbrook wolf/boar in the source
  five-yard/facing gate. Failed target, GCD, resource, form, known-ability and
  empty-combo gates do not mutate resource, RNG, GCD or combo state.
- [x] Resolve M4 rank data as `base + perCombo * spent + range(0, variance) +
  attackPower / 14`, consume variance then physical-crit RNG draws, apply the
  source physical critical multiplier and armor reduction, and use the shared
  JavaScript rounding helper. The existing authoritative RNG state/draw/digest
  is consumed directly, not recreated in a side stream.
- [x] Commit source direct-damage combat timers, idle social aggro or active
  target backfill, physical damage threat, HP and normal Eastbrook lethal
  XP/quest/loot settlement. Clear `entityComboPoints` after that effect path but
  retain `entityComboUntil`, matching the source's post-effect combo cleanup.
- [x] Route known-ability slot and exact typed `eviscerate` casts through one
  reducer. The focused state test covers three-point slot damage, typed
  two-point damage, GCD rejection, empty-combo rejection, threat, persistence
  and timestamp preservation. `wos58_eviscerate_static_guard.mjs` pins the
  source definition, cast gate, finisher effect, direct-damage threat and combo
  reset paths.

**Boundary:** the closure covers only the retained no-aura Rogue and ordinary
Eastbrook targets. It does not claim stealth/openers, next-attack guaranteed
crits, damage-modifying auras/talents, armor debuffs, absorbs, other target
kinds, full combat events or additional Rogue finishers. Those paths require
their corresponding aura, talent, combat-log and world-targeting state owners.

**Evidence:** all retained WOC static guards, including WOS58, pass against
source `5ef9f7cb21cd8875b6d2c49701015dfcd78de35a`. Dynamic ZrVM execution remains
an infrastructure validation dependency because this session has no executable
`zr_vm:project` backend and must not take the concurrent Cargo lane.

### WOS59 Backstab runtime closure (delivered, no schema bump)

**Goal:** make Rogue Backstab an authoritative retained-world weapon transaction
instead of an unavailable known ability. The source requires a dagger, a live
enemy in melee range and arc, a strict behind-target angle and non-overlapping
positions; it charges 60 energy, applies the Rogue one-second GCD, deals 150%
weapon damage plus the rank bonus, and awards one combo point only on a landed
strike.

**Delivered scope:**

- [x] Add `backstab` as the explicit WOC-only non-parity row in the M4 source
  generator. It is extracted from the pinned source rather than hand-copied,
  remains index 21 after the scenario-derived rows, and carries all three ranks
  with `weaponMult: 1.5` and `requiresBehind: true` in the fingerprinted
  `m4_abilities.json` contract.
- [x] Extend the generated M4 effect projection with the strict numeric
  `weaponMult` field and the strict boolean `requiresBehind` flag. Unknown
  effect fields remain validation errors. Backstab's retained-world admission
  consumes that flag, the M5 catalog-projected main-hand dagger query, source
  five-yard fallback, caster-facing arc, target back-facing half-plane and the
  source 0.1-yard overlap hold.
- [x] Charge resource and arm the GCD only after all gates pass. Reuse one
  authoritative `autoAttackState.meleeSwing` with M4's rank bonus and weapon
  multiplier, commit its RNG/HP/combat result, boar reaction, social aggro or
  active target backfill, physical threat, ordinary Eastbrook lethal settlement
  and landed-only capped 30-second combo state.
- [x] Route slot selection and exact typed `backstab` through the same reducer.
  The focused state test covers M5 default-dagger admission, behind success,
  typed success, threat, persistence, front rejection and explicit unarmed
  rejection. `wos59_backstab_static_guard.mjs` additionally pins source
  definition/cast geometry, M4 content generation, generated multiplier/flag
  and the WOC runtime route.

**Boundary:** this uses the already-retained M5 item catalog and its generic
main-hand dagger flag; it does not materialize every source weapon, stealth or
opener state, aura/talent damage modifiers, non-Eastbrook targets, complete
line-of-sight geometry, PvP or a general equipment/combat system. Those remain
separate projections with their owning state and runtime boundaries.

**Evidence:** the retained WOC static guard suite, plus both M4 generators'
staleness checks, pass against source `5ef9f7cb21cd8875b6d2c49701015dfcd78de35a`.
Dynamic ZrVM execution remains an infrastructure validation dependency: this
session has no executable `zr_vm:project` backend and must not take the
concurrent Cargo lane.

### WOS60 Gouge runtime closure (delivered, no schema bump)

**Goal:** close Rogue Gouge as a source-pinned direct-damage and break-on-damage
incapacitate transaction in the retained Eastbrook world. The source charges 45
energy, uses the one-second Rogue GCD and a ten-second cooldown, rolls direct
physical damage in the 8-9 (rank two 15-17) interval before the physical-crit
draw, then applies a four-second `gouge_incap` aura and awards one combo point
only while the target remains alive.

**Delivered scope:**

- [x] Add `gouge` as the second explicit WOC-only source-extracted M4 row. It
  follows Backstab at index 22 in the fingerprinted 23-row contract, with its
  direct-damage and incapacitate effects and rank-two values generated into the
  Zr catalog/effect modules.
- [x] Admit a known primary Rogue against the retained live hostile Eastbrook
  melee target gate, then commit source-order resource/GCD/cooldown, two
  authoritative RNG draws, physical crit/armor/JavaScript rounding, combat,
  social aggro or target backfill, threat, lethal settlement and capped
  30-second combo state. Slot and exact typed `gouge` payloads route through
  that same reducer.
- [x] Reuse WOS39's persisted motion-aura partition without a schema revision.
  The dedicated producer records the source ability code, caster id,
  generated `incapacitate` kind and remaining seconds; it refreshes the
  source-equivalent `gouge_incap` identity rather than introducing a generic
  aura dispatcher. Every retained positive-damage writer clears only that
  break-on-damage row.
- [x] Preserve source mob ordering: forced-target time still advances while the
  mob is incapacitated, pursuit and melee are skipped, and its aura ages after
  the mob action. The focused state test covers slot/typed commands, cooldown,
  source RNG consumption, persistence, incapacitated pursuit suppression,
  aura expiry decrement and damage break. `wos60_gouge_static_guard.mjs` pins
  source definition/dispatch/locomotion order, generated M4 data and runtime
  ownership.

**Boundary:** this is only Gouge's plain, instant-break incap on ordinary
Eastbrook targets. Fear DR, generic aura removal, player-target crowd control,
stealth/openers, talents, absorb/modifier layers, combat-log events and other
crowd-control producers remain with their respective state owners.

**Evidence:** all 19 retained WOC static guards and both M4 generator staleness
checks pass against source `5ef9f7cb21cd8875b6d2c49701015dfcd78de35a`.
Dynamic ZrVM execution remains an infrastructure validation dependency: this
session has no executable `zr_vm:project` backend and must not take the
concurrent Cargo lane.

### WOS61 Kidney Shot runtime closure (delivered, no schema bump)

**Goal:** close the already scenario-projected Rogue Kidney Shot finisher on
the retained Eastbrook world. Its source requires positive character-bound combo
points, charges 25 energy, uses the one-second Rogue GCD and 20-second cooldown,
then applies `1 + spent combo` seconds of stun before clearing only the active
combo pool.

**Delivered scope:**

- [x] Reuse the source-generated M4 index-13 `finisherStun(base: 1,
  perCombo: 1)` projection and the existing current-known ability identity; no
  new source content row or codec revision is needed.
- [x] Extend the internally ability-keyed WOS39 motion-aura writer so distinct
  source effect identities refresh their own rows. Gouge retains its dedicated
  instant-break-on-damage removal, while Kidney Shot writes the generated `stun`
  kind and is not removed by that narrower path.
- [x] Admit only a current-known primary Rogue with positive combo points and a
  live hostile retained Eastbrook target in the normal melee/facing gate. The
  shared reducer commits resource/GCD, current combo-derived duration, combat
  and idle aggro or target backfill, then clears combo points while preserving
  `entityComboUntil`, and records the generated cooldown.
- [x] Route slot selection and exact typed `kidney_shot` through the reducer.
  The focused state test covers a three-point four-second stun, persistence,
  forced-target progression with suppressed pursuit/melee, typed two-point
  refresh, cooldown, no-RNG behavior and zero-combo rejection.
  `wos61_kidney_shot_static_guard.mjs` pins source admission/effect cleanup,
  M4 projection and retained runtime ownership.

**Boundary:** the current Eastbrook slice has no prior stun diminishing-return
history, so it retains only the source's first-category full-duration result.
Cross-target/player DR histories, immunity, generic aura events and all other
stun producers remain separate state owners.

**Evidence:** all 20 retained WOC static guards and both M4 generator staleness
checks pass against source `5ef9f7cb21cd8875b6d2c49701015dfcd78de35a`.
Dynamic ZrVM execution remains an infrastructure validation dependency: this
session has no executable `zr_vm:project` backend and must not take the
concurrent Cargo lane.

WOS38 introduced the current-source Arena queue admission rows: five protocol formats
(`1v1`, `2v2`, Fiesta, Yumi3 and Yumi5), party-leader/size/death/instance gates,
an atomic premade unit order used for both queue ordering and whole-team leave,
and the source's frozen team-average rating with both PlayerMeta ladder columns.
It deliberately stops before match allocation, combat reset, score/wipe resolution,
ratings, Arena/Fiesta/Yumi UI events and augment selection, because those require the
remaining PvP lifecycle state rather than a project-local approximation. WOS20
continues to persist the source's single active party ready-check as aligned member rows
(`partyId`, initiator, deadline and pending/ready/not-ready response), including
the existing one-byte `readyrespond` command, all-answers early completion,
member-removal cleanup and the 30-second end-of-tick expiry. The exact `/ready`
UTF-8 command route and prompt/counts-only event delivery remain pending Plugins 08;
the candidate does not invent a replacement wire command or a retained global map.
WOS19 persists the source `normal`/`heroic` dungeon selection, including party-first
resolution and the party-leader mutation gate. WOS18's weapon-stowed marker remains
compatible: living actors toggle it and a valid hostile `attack` clears it. This does
not make generic loot, pet ownership, combat death or presentation attachment state
complete. This updates the public package metadata; it does not claim dynamic ZrVM or
playable-M8 acceptance.

### WOS39 motion-relevant aura projection (source implementation complete)

**Goal:** make the retained player-motion reducer consume the source's control
and cast-mobility state without inventing per-entity `stunned`, `rooted`, or
`mobile_cast` flags. This is a source-authored M4/M3 bridge only; it does not
claim a full aura lifecycle, dynamic ZrVM execution, or locomotion parity.

**Architecture:** `world/state.zr` remains the sole WOS38/WOS39 codec owner,
but only owns aligned numeric columns and calls narrow motion helpers. A
generated motion-aura contract owns stable numeric aura-id/kind codes and the
base/talent `castWhileMoving` lookup. The future full effect dispatcher is the
only producer of these rows; WOS39 must not introduce a test-only command or a
second aura authority. This boundary uses numeric codes rather than a mutable
array of string/object auras, so it does not depend on a project-local escape
from the outstanding Plugins08 class/array ABI work.

**Implementation slices:**

- [x] Extend `tools/known_ability_catalog_source_extract.mjs` and
  `tools/known_ability_catalog_codegen.mjs` so the current-head catalog carries
  each base ability's `castWhileMoving` flag. Add a generated numeric resolver
  that combines that flag with the selected Talent V2 ability modifier already
  represented by `progression/talent_modifier_state.zr`; do not infer mobility
  from cast time or ability name.
- [x] Extend `tools/cc_contract_codegen.mjs` (or its generated contract) with
  stable numeric codes and predicates for source motion-relevant aura rows:
  `stun`, `stasis`, `incapacitate`, `polymorph`, `root`, and the `ice_floes`
  cast-mobility protection. Keep source aura id, source entity id, kind code and
  remaining time in aligned WOS variable partitions so future refresh/expiry
  work can use the source identity instead of a derived boolean.
- [x] Add a focused `world` motion-control helper that derives stunned/rooted
  and cast-mobility facts from those numeric rows, delegates stand-up and
  cancellation decisions to `world/player_motion_effects.zr`, and invokes the
  existing `combat/casting_state.zr::cancelCast` path. `world/state.zr` only
  marshals its entity columns and commits the returned values.
- [x] Add WOS39 encode/decode/default migration columns in `world/state.zr`.
  WOS2-WOS38 snapshots must decode with empty motion-aura partitions, and all
  rows must preserve canonical entity/partition ordering and finite 20 Hz
  expiry fields.
- [x] Replace the separate fixed-tick player loops with one per-player source
  order: movement side effects and transition, cast progression, then
  motion-aura ageing/expiry. Preserve the existing dead/ghost eligibility and
  do not fold auto-attack, charge, follow, fear, DoT/HoT, regen, or full
  `updateAuras` into this slice.

**Testing stage:** add a dedicated WOS39 source guard plus focused Zr projects
for stunned turn suppression, rooted planar/jump rejection, sit-before-root
stand-up, mobile/non-mobile cast cancellation, Ice Floes protection, expiry
after the player cast phase, WOS38 migration, and WOS39 encode/decode ordering.
Run generator freshness and focused static projects first. The production
`zr_vm:project` transaction and current-head trace remain gated by Plugins08;
the engine host is not replaced by WOC code.

### WOS40 form-state and authoritative cast bridge

**Goal:** reproduce the current-head form-toggle state machine through the
existing authoritative `cast` and `castSlot` commands: the six source forms
must have one mutually-exclusive active-form projection, source-compatible
resource-bar transitions, admission and known-ability stripping. This slice
does not approximate the forms' derived combat stats; that is WOS41.

**Architecture:** extend the pinned known-ability projection with the source
cost, cast time, cooldown and primary self-buff kind, then derive the six
form ability codes from `classes.ts`, `forms.ts` and `types.ts`. Persist one
active form ability code plus the mana maximum parked while a Bear/Cat form
owns the live resource bar. `world/state.zr` remains the only snapshot and
command authority; a narrow `combat/form_transition_state.zr` owns only
numeric transition arithmetic. It must not repurpose WOS39 motion-aura rows,
decode arbitrary command values, or create a second generic aura lifecycle.

**Implementation slices:**

- [x] Extend the current-head ability catalog extractor/code generator with
  base cost, cast time, cooldown and the primary `selfBuff` kind. Generate a
  numeric form resolver for `bear_form`, `cat_form`, `travel_form`,
  `fireball_form`, `moonkin_form` and `shadowform`; reject catalog/source
  cardinality or kind drift.
- [x] Add a pure form-transition module and a focused project test entry. It
  must distinguish same-form exit from a cross-form cast, park and restore
  mana plus its maximum for Bear/Cat, set Bear rage to zero and Cat energy to
  100, debit live versus parked mana using the source's `formShiftKind` rule,
  and leave non-resource forms on mana.
- [x] Add WOS40 aligned columns, writer/decoder validation and WOS2-WOS39
  default migration for active form and parked mana maximum. Preserve one
  source form per entity and canonical snapshot order; no boolean shadow
  flags may become the authority.
- [x] Generalize only the existing Temporal Reversal command dispatch enough
  to route the six exact source ability identifiers and their action-bar codes
  to the form reducer. Retain the current explicit unimplemented boundary for
  every other known ability and do not introduce a byte-string substitute for
  Plugins08's structured `CommandValue` bridge.
- [x] Apply source admission against the current active form, honor dead/cast/
  GCD/resource/cooldown gates, arm Fireball Form as its two-second cast and
  ten-second cooldown, and complete only form casts through the existing
  retained cast lifecycle. Travel and Fireball form application must retain
  the source action-locking and cast/auto-attack cancellation semantics.
- [x] Call form-orphan stripping immediately after every offline known-ability
  partition replacement used by talent row/spec/respec/loadout updates. A
  form whose granting code left the refreshed partition must be removed before
  later command admission can observe it.

**Testing stage:** run the form-contract and known-ability generator freshness
checks, the WOS40 source guard and focused Zr form-transition project. Cover
same-form free exit, every cross-form payment path, Bear/Cat resource values,
Fireball timed completion/cooldown, typed and slot command routing, action
locking, talent-orphan removal, WOS39 migration and WOS40 snapshot ordering.
Then batch the native protocol/runtime identity suites. Real project execution
and current-head trace comparison remain Plugins08-gated and are not replaced
by static checks.

**Static validation (2026-07-29):** WOS43 source guards cover the append order,
native identity, Hunter Survival's nonzero persisted spell-haste path and
Warrior's source-exact gnarled-staff replacement; the
world-state test package reaches `state.selfTest()`, and the touched diff is
whitespace-clean. The coordinator rejected the managed native Cargo batch before
Cargo started because this active Session's immutable write scope cannot be reused
by `validate-matrix`; this is a coordinator ownership issue, not a test result.
No `zr`, `zrvm`, or `zr-cli` executable is available here, so focused Zr projects,
a project transaction, and trace comparison remain unrun acceptance work.

### WOS41 form-derived combat contribution graph

**Goal:** make active forms contribute exactly where the source's derived
combat, movement, threat and damage owners read them: Bear armor/AP/max HP and
threat, Cat AP/agility/threat and swing identity, Travel/Fireball speed,
Moonkin armor/spell damage and Shadow school-only damage.

**Dependencies:** WOS40 plus a source-ordered, recomputable player-stat graph.
The current WOS scalar combat columns are final values without their base,
gear, talent and aura contribution graph; directly multiplying those values
would be an incorrect approximation.

**Implementation slices:**

- [x] Add a source-pinned pure six-form contribution kernel and generated
  contract for the reference entity, movement, threat, spell, damage and
  form-swing rules, plus the M5 starting-gear pre-form baseline adapter. The
  generated M5 lookup remains no-talent/no-aura, while the adapter applies the
  separately persisted source talent selection through the current modifier
  projection.
  It preserves source order for Bear/Cat armor/AP/HP, Moonkin spell damage,
  Shadow's non-final school-only multiplier and Travel/Fireball max movement
  composition, and proves the adapter rebuilds all 9 x 20 unformed generated
  baselines.
- [x] Persist a WOS41 M5 baseline profile code and recompute its final
  armor/AP/SP/crit/dodge/max-HP columns with source health-fraction semantics
  on form transition and stripping. The codec accepts WOS2-WOS40 with the
  zero profile default and rejects a nonzero profile whose final columns do not
  match the source-pinned baseline output.
- [x] Introduce a source-pinned pure player-stat contribution graph that
  retains the source `recalcPlayerStats` ordering for base equipment values,
  flat set/aura/talent values, all-stat drain, talent and raid percentages,
  armor/AP/max-HP stages, combat ratings and form modifiers. Its M5 adapter
  starts from the no-aura/no-gear-contribution baseline and applies retained
  talent selections; no non-M5 WOS final scalar is replaced until the
  corresponding equipment, set and aura identities are persisted.
- [x] Feed the WOS40 active form into pure movement, threat, form-swing and
  school-damage helpers without duplicating form predicates or applying form
  values to unrelated schools. The retained auto-attack rule consumes active
  form state for Cat's 1.8 swing speed and Bear/Cat/Travel wand suppression;
  it is not yet the WOS world combat dispatcher.
- [ ] Extend snapshot-compatible derived-state recomputation from its current
form transition/strip and retained-talent update paths to equipment, set and
live-aura updates, preserving source health and resource fractions. This
requires bounded, source-identity-preserving WOS inputs for equipment instances,
set counts and ordered live auras. WOS43 owns the current M5 catalog's bounded
helmet/feet/mainhand identities plus its M5-plus-talent `spellHaste` final
column; it must not stand in for those still-unpersisted source identities.

### WOS42 authoritative spell-haste snapshot

**Goal:** make the source-derived M5-plus-retained-talent spell-haste result
part of the candidate world state without changing any WOS41 or earlier bytes.

- [x] Append one fixed6 per-entity `spellHaste` column after the WOS41 baseline
  profile, write WOS42, and require its value to match the same pure graph used
  by the other derived combat columns.
- [x] Migrate WOS2-WOS41 by deterministically adding zero for a zero profile or
  rebuilding the retained M5-plus-talent output for a nonzero profile. The
  self-test round-trips Hunter Survival's nonzero `0.15` contribution.
- [ ] Persist equipment, set and ordered live-aura identities before extending
  this column beyond the explicitly retained M5-plus-talent input scope.

### WOS43 bounded M5 equipment identities

**Goal:** retain the current M5 catalog's source item identity instead of
persisting only its final combat consequences.

- [x] Materialize every class's source starting-mainhand contribution beside the
  existing M5 starting-gear baseline, then expose the contribution through the
  generated Zr catalog.
- [x] Add a pure equipment state adapter for the current M5 helmet, feet and
  mainhand catalog slots. Its `0` code means empty/start-mainhand; nonzero
  codes are stable catalog indexes plus one. It applies source level gates, so
  an equipped over-level item stays visible but is stat-inert.
- [x] Append three WOS43 identity bytes after `spellHaste`, migrate WOS2-WOS42
  to the empty/start-mainhand defaults, and normalize profiled legacy mainhand
  combat values from the retained identity graph.
- [x] Recompute form/talent-derived stat columns and mainhand combat values when
  a bounded identity changes. The focused world self-test replaces Warrior's
  starting sword with `gnarled_staff`, asserts the source `3..6 @ 2.9` weapon
  profile and `spellPower` increase from `5` to `6`, then round-trips WOS43.
- [ ] Extend this bounded identity layer to inventory instances, every source
  slot, set bonuses and ordered live aura rows before claiming full equipment
  parity.

### WOS44 bounded M5 inventory stacks and bags

**Goal:** retain the target's source-order player inventory as authoritative
world state without pretending that mutable item-instance payloads are plain
stacks.

- [x] Append a per-entity M5 inventory partition after WOS43: signed copper,
  four bag identity codes, and dense item-code/count rows. WOS2-WOS43 candidates
  migrate to zero copper, no bags and an empty partition; nonplayer rows are
  required to remain empty.
- [x] Preserve target stack semantics for the projected catalog: backpack 16,
  pooled bag slots, 20-item stackable rows, one-item weapon/armor/bag/tool
  stacks, existing-stack first grants, forced-grant overflow and newest-first
  removal. The self-test exercises a 21-count food grant, bag capacity change,
  tail-stack removal and WOS44 round trip.
- [x] Keep the primitive catalog/stack/bag rules in
  `progression/m5_inventory_rules.zr`; `world/state.zr` retains only the local
  mutable partition, atomic mutation and codec boundary because the current
  ZrVM cross-module Array/class ABI is not a safe ownership transfer.
- [ ] Persist `ItemInstanceData` signer, charges, rolled data, enchant,
  `boundTo` and manual slot identity before using a same-id inventory row for
  market, trade, bank, mail, equip or buyback authority.

### WOS45 bounded M5 equipment/inventory transactions

**Goal:** close the scalar source equip/unequip loop without reinterpreting
existing WOS44 item codes or pretending structured item instances are fungible.

- [x] Keep the WOS44 14-item catalog index prefix immutable and append the seven
  class starting identities (`startWeapon`, `startOffhand`, `startChest`) after
  it, with source-derived provenance and generator checks.
- [x] Reserve mainhand stored code `255` for explicit empty/unarmed. Code `0`
  still means the source starting mainhand, so WOS2-WOS44 migration remains a
  byte-preserving semantic migration with no new entity field.
- [x] Add atomic scalar `equipM5InventoryItem` and `unequipM5InventoryItem`
  transitions for the M5 helmet/feet/mainhand projection. Replacement removes
  the inventory tail match then returns the old source/concrete item without a
  capacity gate; unequip checks capacity before clearing the slot and returning
  its item. Both recompute the same baseline/talent/form-derived columns.
- [x] Add WOS45 static and lifecycle coverage for initial weapon return,
  explicit unarmed combat values, WOS44-compatible decoder admission and the
  WOS45 opaque native identity.
- [ ] Persist item instances, remaining source equipment slots, two-hand/offhand
  displacement, manual inventory cells and set/aura inputs after Plugins 08
  supports reliable structured values across the project transaction boundary.

### WOS46 source bag command bridge

**Goal:** route the existing typed source bag commands through the persistent
WOS44 inventory partition without adding a parallel inventory reducer.

- [x] Dispatch protocol `equip_bag` (126) and `unequip_bag` (127) inside the
  existing candidate command transaction after the standard sequence and
  payload-partition checks.
- [x] Decode the native `u32 utf8 length + id + optional socket` and fixed-u32
  socket shapes locally at the byte boundary. The current scalar catalog admits
  only `wolfhide_satchel`; a well-formed unknown ID and an out-of-range socket
  leave authoritative state unchanged like the source command body.
- [x] Reuse `equipM5InventoryBag` / `unequipM5InventoryBag` for source pooled
  capacity, replacement and shrink preflight rules. A lifecycle batch covers
  equip, unequip, returned item and strict actor command sequencing.
- [x] Route the source-generated scalar vendor subset for `buy`, `sell`,
  `buyback` and `sell_all_junk` through WOS47's item-code partition. Exact
  packet bytes stay at the command boundary and scalar UTF-8 matching uses the
  generated catalog rather than a second item-id table.
- [ ] Route generic `use`, market/trade/bank/mail flows and all
  instance-bearing vendor paths only after their source item/instance state has
  an equally complete persistent authority boundary; do not reinterpret a
  UTF-8 packet as a scalar item selection for an instance-bearing operation.

### WOS47 scalar vendor and buyback loop

**Goal:** establish the smallest complete vendor loop for the source-pinned M5
catalog without manufacturing `ItemInstanceData` from scalar rows.

- [x] Derive Trader Wilkes' vendor stock from `NPCS.vendorItems`, expand the
  catalog from 21 to 35 stable entries without changing the WOS44/WOS45 prefix,
  and expose generated scalar UTF-8 byte queries for packet matching.
- [x] Add generated `buy` (25), `sell` (26), `buyback` (27) contracts and native
  typed payloads. The native layer validates bytes only; all vendor policy stays
  in the ZrVM candidate reducer.
- [x] Persist a per-player newest-first buyback partition after WOS44 inventory
  stacks, bound it to 12 identities, merge repeated scalar ids, and migrate
  WOS2-WOS46 snapshots to empty partitions.
- [x] Route buy, sell, buyback and sell-all-junk through actor sequencing,
  alive/range/copper/capacity checks and the authoritative inventory partition;
  a state self-test covers the full loop plus WOS47 round-trip.
- [x] Charge food and drink stacks by `buyValue * vendorStackSize`, matching
  `sim/items.ts` in both the authoritative WOS reducer and the independent
  inventory/vendor contract model. The latter's scenario now rejects 124
  copper for a five-unit, 25-copper bread stack instead of retaining a
  single-click-price shortcut.
- [ ] Preserve source `ItemInstanceData` signer/charges/rolled/enchant/bound
  attributes, `noVendorSell`/soulbound flags when they occur outside this
  scalar catalog, and per-instance buyback order after Plugins08 provides the
  structured transaction boundary.

### WOS48 inventory-derived `q_boars` collect progress

**Goal:** make the existing M5 inventory partition the sole authority for the
source `q_boars` collect objective instead of maintaining an independent hide
counter.

- [x] Recompute the active/ready objective state from the primary player's
  actual scalar `boar_hide` count after every M5 inventory grant and removal;
  cap progress at the source requirement of five and allow a removal to move a
  ready quest back to active.
- [x] On accept, derive progress from hides already in the inventory. On
  turn-in, verify and remove exactly five authoritative hides before applying
  the existing source-pinned copper/XP completion path.
- [x] Keep the persisted WOS7 hide field only as a derived compatibility
  projection. WOS2-WOS47 reads materialize at most five scalar hides for an
  active/ready `q_boars` row, because the prior side ledger cannot reveal any
  surplus inventory; WOS48 validates the derived projection before encode and
  after decode.
- [x] Keep the small derived-progress leaf in `world/state.zr` as a documented
  temporary boundary exception: its authority is the existing mutable parallel
  inventory arrays, and the current Plugins08 cross-module Array/Class ABI is
  not reliable enough to move that mutation safely. Extract it when the plugin
  transaction boundary returns lossless structured values.
- [x] Add WOS48 static and lifecycle coverage for ready-to-active regression,
  re-ready after a replacement hide, real turn-in consumption and state
  round-trip.
- [ ] Generalize inventory-change quest credit only after all item mutation
  owners (use/discard/loot/trade/bank/mail and non-fungible instances) share a
  persistent source-equivalent authority boundary. Do not infer an unmodeled
  mob drop or item instance from this scalar collect path.

### WOS49 scalar `discard` command bridge

**Goal:** route the already-generated source `discardItem(item, count?)`
command through the same bounded M5 inventory authority without manufacturing
an item instance or an error/event side channel.

- [x] Parse command 24's canonical `utf8_id_optional_u32` payload locally;
  absent count means one, explicit `u32` count clamps to the available source
  scalar stack total, and malformed optional tails reject the candidate.
- [x] Reuse the source-order tail remover. This is the exact current-catalog
  continuation of source `removePreferFungible` because every projected M5
  row is plain; it deliberately permits discard while dead and automatically
  re-evaluates an active `q_boars` collection objective through WOS48.
- [x] Add WOS49 lifecycle coverage for default-one removal, excess-count
  clamping, ready-to-active boar-hide regression, command sequencing and
  snapshot round-trip.
- [x] Generate and apply the source scalar `noDiscard`, `noVendorSell` and
  `soulbound` flags in both WOS and the independent M5 inventory contract, so
  a future scalar catalog row preserves its existing source policy without a
  second handwritten item table.
- [ ] Add instance-preferred removal only when Plugins08 can persist and
  transact `ItemInstanceData`; scalar flags do not represent signer, charges,
  rolls, enchantment or binding-instance identity.

### M5 independent consumable timing leaf

**Goal:** keep the scalar M5 fixture's consumable behavior source-compatible
without claiming a generic `use` command or a second authority path in WOS.

- [x] Preserve separate food/drink slots, source-rounded per-two-second values,
  nine 2-second consumption ticks, resource-kind gating, max-value clipping
  and the source rule that completing consumption does not stand the player up;
  an explicit stand-up transition cancels both unfinished consumption slots.
- [x] Model potion HP-or-mana admission, shared 120-second cooldown decay, and
  elixir expiry that removes its derived stamina before recomputing stats.
- [x] Repair the source checker to locate both repositories from its own file,
  so it can run from the ZirconEngine root rather than relying on an ambient
  working directory.
- [ ] Route generic `use` only when WOS can persist ordered consume/aura state,
  healing modifiers, shared simulation time and instance-preferred removal in
  one Plugins08 transaction. This fixture remains a direct test leaf, not
  authoritative gameplay state.

### M5 simulation money fragment

**Goal:** retain the source simulation's compact English `g/s/c` text for
loot, quest, vendor and market event owners without leaking retained-UI locale
formatting into gameplay rules.

- [x] Mirror source floor/remainder order for gold, silver and copper, omit
  zero-valued leading fragments and retain the mandatory `0c` fallback.
- [x] Cover ordinary mixed values and the reference function's observable
  negative-input floor/remainder boundary; the project currency domain itself
  remains integral.
- [ ] Route this fragment into committed WOS loot/vendor/quest/market event
  text only after structured event ownership exists. It does not mutate a purse,
  replace UI localization or create a text-event compatibility shim.

### M5 public supply observation

**Goal:** expose the source's fixed-width consumable, economy and capacity
facts to decision consumers without giving an observation encoder permission to
mutate inventory or infer vendor range.

**Source status:** `src/sim/supply.ts` is currently an untracked file in the
reference project's worktree and is absent from the pinned `5ef9f7cb` tree.
`m5_supply_source_check.mjs` therefore pins its exact byte SHA-256
`220ee4ff05ede7b402c1fa0412b15ea4fa7fa47c79da0a4950a1a96509fd92ff` until
the reference owner commits it; this leaf is source-worktree compatible, not
commit-pinned evidence.

- [x] Project food, drink, healing-potion and mana-potion counts from ordered
  inventory rows using the generated current M5 item catalog, with source
  availability bits and a 16-count cap.
- [x] Preserve potion readiness, capped copper, free-bag fraction and each
  fixed restock's independent copper-and-capacity feasibility. The four source
  restocks retain their item identities and `5/5/1/1` quantities.
- [x] Add focused contracts for a stocked player, partial-stack/full-bag
  capacity boundary, active potion cooldown and insufficient-copper boundary.
- [ ] Inject observations into WOS only when one transaction can read the
  canonical inventory/bag/cooldown state. This is read-only and does not imply
  a vendor is in range or that a purchase command is available.

**Testing stage:** compare source-pinned vectors for every six form with and
without relevant talent/gear/aura combinations, then verify movement, threat,
physical/spell/shadow damage, swing and state round trips. This stage is not
accepted until it runs through the real ZrVM transaction.

The decoder's offline WOS6-to-WOS7/8/9/10 migration idempotently materializes the two
fixed source quest NPC rows, so adding the live-NPC interaction gate cannot make
an existing offline snapshot permanently unable to accept or turn in its quests.

The Vale Cup audit now has four independent executable leaves:
social/vale_cup_ball_state.zr ports the pinned source's 20 Hz ball
kinematics, speed cap, gravity/bounce, ground and pocket friction, mouth-first
goal crossing, all six Sowfield board reflections, dribble carry, body trap and
capped launch. vale_cup_ball_contract_codegen.mjs hashes
vale_cup_ball.ts, vale_cup_layout.ts and the shared tick source into a
checked JSON/Zr contract, and the dedicated .zrp test entry covers both goals,
a goal-flank reflection, pocket settling, carry, trap and launch. It deliberately owns no cup
queue, player/member/guild/deserter map, match scheduler, score, bet or event
state. Dynamic execution remains unavailable until the already-notified
zr.zircon.math scalar host functions are present in the reliable ZrVM backend;
this is static-source evidence only, not a Vale Cup or M6 acceptance claim.

The same pinned layout contract also supplies
social/vale_cup_layout_state.zr. It retains the source's inclusive Sowfield
presence shell, full decoration shell and pitch predicates plus unbounded
numeric practice-pitch origin mapping (eight is the separately exposed slot
policy). It reuses layout constants only and deliberately does not duplicate
the existing terrain height/stand lift or collision owners. Its dedicated test
entry covers boundaries and positive, final and negative practice slots; this
is likewise static-source evidence rather than a live match claim.

The dependency-independent numeric core of Vale Cup bot policy is now isolated
in social/vale_cup_bot_policy.zr: source-ordered seat roles use explicit Zr
codes, and the same pinned source contract fixes the no-RNG aim triangle wave,
keeper guard position, charged-shot aim/range and ball interception clamps.
The future cup match owner must map role codes to canonical roster state and
issue movement/casts; this module owns neither entity collections, names,
queues, bots nor simulation actions. Its code generator, JSON/Zr output and
test project pass static freshness/closure checks. The full upstream bot
integration suite exceeded this session command limit, so no upstream
full-bout or ZrVM dynamic result is claimed.

The queue-only portion of the same current source is now isolated in
social/vale_cup_queue_state.zr. It accepts already-admitted numeric premade
units, retains their snapshot member/role order, mirrors whole-unit leave,
normalizes roles, deterministically ensures a 3v3+ keeper, and selects a
source-shaped first-fit match by oldest joining tick with the source lower
bracket tie break. Its generated contract pins the source bracket/nation
catalog and queue/matchmaking statements. Player liveness, party leadership,
duel/trade/instance exclusion, string-keyed deserter/guild state, live match
allocation, kit swapping, ball/entities and events remain future WOS or
runtime-bridge owners. The dedicated project currently has static generator
and closure evidence only; no ZrVM queue execution or Vale Cup acceptance is
claimed.

Dungeon Finder now has one independent numeric core in
social/dungeon_finder_role_state.zr. It projects the source's ordered Kuhn
augmenting-path role assignment with explicit tank/healer/dps codes, including
flexible-member reseating, exact five/ten-player composition checks and
partial-roster open capacity. The generated source contract pins the role
order, capacity tables and source algorithm statements. Activity strings,
class/spec/level eligibility, party snapshots, FIFO unit search, proposals,
cooldowns, board/listing data and PartyMachine formation remain future
transaction-bridge owners; static contract/closure evidence is not a Dungeon
Finder or M6 acceptance claim.

The current-head Master Loot decisions are now in
progression/master_loot_state.zr. The module preserves QUALITY_RANK ordering,
the source missing-quality-to-common default, threshold comparison, disabled
no-looter result, leader selection for zero configuration and departed-looter
fallback. Its source-pinned contract prevents silent quality-order drift. The
future inventory/party loot owner remains responsible for item definitions,
party MasterLootSettings, threshold routing, corpse rights, assignment
transactions and events; this static leaf is not current loot-pipeline or M5
acceptance evidence.

The current-head WARFARE rating leaf is now in combat/pvp_power_state.zr and
integrated at the existing damage projection's hostile player-versus-player
boundary. It pins the ten-rating-per-percent conversion, independent 20 percent
offense/defense caps, custom-cap computation and the post-mitigation
(1 + offense) * (1 - defense) multiplier. The integration applies before
shield absorption and leaves friendly, self, pet and PvE paths unchanged. Gear
rating aggregation, hostility resolution, all PvP lifecycle rules and live
world/transaction wiring remain their own owners; this source-pinned leaf is
not dynamic M4/M6 or arena acceptance evidence.

The current-head PvP honor reward policy is now in
social/pvp_honor_policy_state.zr. It pins ranked 1v1/2v2 awards, the
first-only Arena repeat decay, Fiesta's 1/half/quarter/zero repeat curve, the
10/15-win daily taper and source-compatible nonnegative reward flooring. It
does not substitute numeric keys for honorTeamIdentity, nor create a partial
daily record/event/persistence adapter: UTC reset, character/name identity,
per-opponent counters, spendable/lifetime saturation and emitted honor events
remain a source-shaped social transaction owner after the generic ZrVM
string/object and rollback ABI is returned. This static policy leaf is not
M6 honor or multiplayer acceptance evidence.

The bounded M5 inventory projection now applies current-source equipment weight
rules to its materialized helmet/feet/mainhand rows. Its generated content
catalog exposes the source requiredClass array as a structured itemAllowsClass
query plus weapon.hand; the inventory state uses mail, leather and cloth ranks
instead of its former warrior-only comma-string shortcut. The focused source
state covers typed-armor rejection for rogue/mage and mail/leather admission
for paladin/warrior. This is deliberately not full equipment parity:
target-slot selection, ring/offhand/twohand displacement, spec revalidation and
authoritative equipment persistence remain outside the bounded M5 transaction.
The scoped rows now normalize finite explicit levels at content generation and
apply full-catalog source-derived level admission after class eligibility but
before the swap mutates inventory; ring/offhand/twohand and spec routing remain
separate source transactions.

The current-head item-level equip gate is now isolated in
progression/item_level_requirement_state.zr. Its source-pinned contract keeps
the explicit-level override, rare/epic/legendary source-level derivation,
12/18/cap fallback bands, missing-quality common default and inclusive 1--20
clamp. Its catalog entry point now resolves itemSourceLevel from the full pinned
content table before applying the rule; content decoding still normalizes
non-finite JavaScript input before the typed Zr boundary. This leaf does not
claim persistence or a full equipment-acceptance path.

The current-head equipment slot-routing policy is now in
progression/equipment_routing_state.zr. It retains ring1/ring2 empty-first
resolution, targeted ring acceptance, one-hand defaults and Warrior Fury's
two-hand offhand routing. Its input accepts the normal preceding content
eligibility result rather than scalarizing the source ItemDef.requiredClass
array; full class/proficiency evaluation remains coupled to the catalog and
generic ZrVM collection ABI. This is a policy leaf, not a swap, displacement or
spec-revalidation transaction implementation.

The current-head shared-loot FFA policy is now in
progression/loot_ffa_state.zr. It pins the 60-second owner-lock window, lapse
at zero, free-for-all, untapped, tapper and tapper-party access paths. The
typed party member array remains explicit; empty and absent parties are
equivalent only for this membership predicate. Corpse countdown ownership,
personal loot, rolls, events and inventory transactions remain the full loot
pipeline's work, so the leaf is not M5 loot acceptance evidence.

The source-shaped world/corpse_loot_rights_state.zr now composes that policy
with first-positive-damage tap ownership, pet-owner attribution, leaving-player
rejection, loot-time party membership, finite microsecond owner-lock state, and
manual versus passive FFA admission. It remains an isolated world-state
projection until the interaction command and inventory transaction owners can
compose it; it does not alter the historical M5 loot-distribution fixture or
claim a live loot pipeline.

The current-head fleeing social-aggro leaf is now projected in
world/fleeing_social_aggro_state.zr. It uses the caller-provided local spatial
candidate list to keep the source's same-family, idle, ownerless, living,
hostile and strict-radius predicate; each pulled ally enters chase, copies the
leash anchor and receives one target-threat point. The caller still owns grid
query ordering and ends flee after the first nonzero rally, so this is static
source evidence rather than M3 combat/AI acceptance.

The current-source town-focus leaf is now projected in
progression/town_focus_state.zr. It keeps generic string component keys as
ordered Record pairs, including source-preserving previous allocation on
not-in-town, invalid, or over-budget rejection; it also retains additive yield
bonus, capped harvest-tier shift, inclusive town-hub circle and all three
reallocation cost tiers. Position/zone selection, payment, elapsed real time,
player persistence and harvest execution remain the owning transaction
boundaries. This is static source evidence, not profession or M5 acceptance.

The current-source profession action XP leaf is now projected in
progression/profession_action_xp.zr. It carries the gather/craft base curves,
the shared green/gray falloff, source `zeroDiff` bands, four-level bonus cap
and nearest-integer award rule. It does not mutate craft skill or progression
state: action dispatch and XP-event commit remain the owning M5 boundaries.
This is static source evidence, not a completed gather/crafting progression
transaction or M5 acceptance.

The current-source salvage transaction is now projected in
progression/salvage_state.zr. It retains weapon/armor and non-poor eligibility,
the source quality-material table, required-level tier bonus, one RNG bonus
unit, and removal-before-grant order. The focused inventory state deliberately
models only `removePreferFungible`; complete item-definition indexing and
instance ownership stay with the M5 inventory transaction, so this is static
source evidence rather than full economy acceptance.

The current-source Battlefield Experience self-observation reducer now composes
the existing crafting transaction rather than carrying a second craft-skill
store. Its pinned contract preserves the rare-or-better gate, legacy rolled
quality before static definition fallback, signer-equals-observer attribution,
recipe-profession resolution boundary, active-or-paired-major gate and fixed
0.25 additive award. The generic cross-module mutable-state ABI is still a
Plugins 08 dependency, so this is source/static evidence rather than dynamic
ZrVM or M5 acceptance.

The current-source crafting-hub gate is now projected from the Highwatch
content circle and the level-20 minimum. `crafting_hub_state.zr` owns the
inclusive squared-distance and level checks, then synchronizes only the
existing crafting transaction's `atHub` precondition. This restores the source
location gate without duplicating recipe, reagent, fee or RNG logic; its
cross-module transaction composition remains static evidence until Plugins 08
returns the generic mutable-state ABI.

The current-source Enchanting profession now has a composed state projection.
It preserves disenchant eligibility, fungible-first unenchanted-copy selection,
one-draw arcane-material yields, all-or-nothing enchant validation, persistent
instance merge semantics and additive enchanting skill gain. The shared item
instance projection now carries explicit rolled-stats presence and armor so a
legacy enchant remains distinguishable from a masterwork payload. The complete
current-head `ENCHANTS` table is generated into the projection; item-definition
and required-level lookup remain at the WOS owner boundary. Generic cross-module
arrays and live transactions remain Plugins 08 static-only evidence.

The current-source gathering-tool policy is now projected with its complete
three-effect catalog. The module retains tool-tier admission, confirm-or-skip
behavior, effect application, zero-durability draw consumption, rarity-scaled
durability loss, original-crafter and specialized recharge discounts, and
integer-ceiling costs. Harvest target resolution and inventory material removal
remain at their WOS call sites; its composed crafting-skill state remains
static-only until Plugins 08 returns generic mutable-state execution.

The full current-head gathering leaf is now separately source-generated from
the 24-node placement catalog, gathering content and `gathering.ts`. Its state
projection preserves player-local node ready times, one-draw readiness ordering,
three independent queued proficiency counters, material-rarity clamping and
weights, component mapping, first-claim corpse ownership, source-order focus
selection, concentration tier shifts, and rare-or-better material signing. The
world command still owns range/death/bag admission, item/quest/Xp/event writes
and durable player storage. Its ordered Zr arrays deliberately retain the
source's dynamic record/list boundary, so this is static source evidence until
Plugins 08 supplies reliable generic container/object execution, not gathering
or M5 acceptance.

The current-head archetype leaf is now source-generated from the canonical
ten-craft ring, combo recipes, wheel threshold and `archetype.ts`. Its state
projection preserves canonical adjacent-pair ids, combo-aware default majors,
skill/ring-order hobby selection, history/mode transition rules, the `5 + 3n`
amends ramp, and common/rare/unlimited ceilings. It writes only active/paired
identity and the selected recipe craft's `-1`-encoded ceiling into the existing
crafting transaction; recipe execution, quest admission, player lookup and
persistence remain their owners. That mutable cross-module composition is
static-only pending Plugins 08 and is not M5 acceptance.

The current-source mobile-crafting-station leaf is now projected in
progression/mobile_crafting_station_state.zr. It preserves the caller-owned
specialization gate, source-derived 12,000-tick lifetime and strict expiry
boundary, while retaining player, craft and placement coordinates in the
returned state. Storage, station removal and any future crafting-location
override remain outside this inert source-policy slice and are not M5
acceptance evidence.

The current-head comparable item score is now in
progression/item_score_state.zr. It preserves primary-stat aggregation,
12-armor-per-point conversion, half-weight average weapon DPS and stable
one-decimal rounding for the source catalog's nonnegative, positive-speed item
domain. Full item-source indexing, derived item levels, malformed JavaScript
number normalization and equipment transactions remain their owning boundaries;
this is not a full tooltip or M5 inventory acceptance claim.

The full current-head item_level.ts source index is now materialized in
progression/item_level_catalog_state.zr. Its generator executes the pinned
TypeScript module graph and emits all 580 item records, including the 395
derivable source levels, 331 visible item levels and 14 raid flags, without a
second handwritten index algorithm. The generated table is a current-content
input for later tooltip/equipment/loot transactions; runtime item mutations,
full item objects and dynamic ZrVM string/module acceptance remain open.

The movement-frame protocol and WOS13 reducer retain facing, previous pose,
horizontal/vertical velocity, on-ground/jump and fall state. Every valid
canonical frame updates held input and a monotonic acknowledgement, stale flags
clear after fifteen silent 20 Hz ticks, and the candidate applies the existing
turn, slope, water, terrain-height, swept-collision, vertical and wall-standoff
transition. Dynamic custom-content, decoration and active-run collision context,
plus M4 combat/aura side effects, remain required for full locomotion parity.

`m3_terrain_content_codegen.mjs` now reads the pinned `zone1/2/3.ts`,
`data.ts`, `world.ts` and `vale_cup_layout.ts` source blobs to generate the
built-in three-zone/five-lake projection and source-identity catalog. It
preserves the strict lake-radius boundary and explicit no-water predicate without
using a finite `-Infinity` substitute. The catalog now also fixes the 67
source-order camp records with mob identity/count, dummy marker, inclusive
level range, centre and radius. It fixes the source's 307 camp entities and 306
non-dummy construction draw participants, alongside the built-in terrain edit,
two docks and the Sowfield rectangle/height/falloff, together with 14 road
polylines, the Sowfield shell and decoration exclusion inputs. This is a
builtin world-query and future world-construction data prerequisite only, not
locomotion acceptance evidence or a live spawn loop.

`camp_spawn_layout.zr` now replays the exact five-draw construction schedule
for every non-dummy camp spawn, including the inclusive level roll, facing and
  wander timer. It intentionally exports the raw post-RNG scatter point only;
  dungeon-door clearance, source-projected ground-height query and the built-in
  open-world safe-position correction now exist as separate scalar modules,
  while actual entity materialization remains separate M3 work.

The source-order `dungeon_door_content` projection now supplies all five
deduplicated overworld doors and their imported 20-yard clear radius.
`dungeon_door_clearance.zr` ports the strict inside-ring projection and
deterministic centre fallback, without consuming RNG. `terrain_ground.zr`
  also now supplies the source-projected final ground-height query.
  `collision_grid.zr` now ports the source 16-yard collision-cell insertion for
  the 170 fixed records and deterministic tree/rock colliders, including the
  0.8-yard broad-phase expansion and source insertion order. `safe_position.zr`
  uses it with the exact 80-step golden-angle spiral for built-in overworld
  positions. Instance/Delve routing and entity materialization remain open.

`m3_camp_mob_core_codegen.mjs` now projects all 47 first-seen camp mob
templates into a scalar ZrVM catalog: names/families, level ranges, base combat
and movement values, presentation color/scale, core flags and optional respawn
field presence. Loot and combat-effect payloads remain separate M4/M5 work;
this core catalog does not claim a completed world spawn loop.

`content/camp_mob_spec.zr` is the one generic current-source projection of
`createMob`'s initial scalar derivation for all 47 camp templates: bounded
level, elite-only `2.3x` health and `1.5x` damage multipliers, rounded weapon
range, armor and move speed. It now also exposes the source `canSwim ||
family == "mudfin"` construction predicate. `camp_spawn_placement.zr` composes
the existing source-order scatter replay with the swim-dependent height
threshold, door-ring projection, safe-position spiral, final door-ring
projection and final ground query. It has no RNG ownership or world mutation;
entity materialization remains blocked behind the transactional-world
foundation.

`m3_npc_placement_codegen.mjs` pins the 31 source-order static NPC definitions
and separately identifies the four system-owned dynamic NPC ids. The generated
catalog and `npc_placement.zr` reproduce the constructor's static-NPC path:
`waterLevel() + 0.6` safe-position projection, final ground height and authored
facing. This remains a scalar placement result only; entity-id allocation,
market/bank indexes and dynamic NPC systems stay with the transactional world
owner.

`m3_npc_initialization_codegen.mjs` separately locks all 35 current-head NPC
definitions needed by `createNpc`: names, titles, greetings, poses, colors,
quest references, vendor inventories and service flags. It preserves the 87
quest references, 107 vendor rows, World Market/banker/heroic/Card-Duel entry
identities and dynamic marker. This is source-owned initialization data, not a
claim that interaction, vendor/market/bank admission or dynamic event systems
are materialized in the world transaction.

`m3_ground_object_placement_codegen.mjs` flattens the 18 source-order
`GroundObjectDef` records into all 55 constructor placements. Its companion
`ground_object_placement.zr` preserves the source raw X/Z coordinates and final
`groundPos` height sampling without consuming RNG. Entity IDs, lootability,
collection/respawn state and quest-credit mutation remain WorldState work.

`m3_mailbox_placement_codegen.mjs` locks the three Ravenpost source positions.
`mailbox_placement.zr` composes their required `waterLevel() + 0.6` safe-position
projection and final ground height, including the Highwatch collision adjustment.
Mailbox entity IDs, PostOffice registration, message persistence and collection
remain owned by their transaction and service layers.

`m3_dungeon_door_content_codegen.mjs` is now re-pinned to current head and
retains the `Object.values(DUNGEONS)` order required by door-clearance, which is
intentionally distinct from entity construction. `m3_dungeon_entrance_codegen.mjs`
supplies the `DUNGEON_LIST` index order instead: six dungeon definitions, five
overworld doors and 24 preallocated slots each. `dungeon_entrance_placement.zr`
projects only the five real door entities through final ground height. Door IDs,
instance-slot state, party claims and internal dungeon objects remain
transactional-world work.

`m3_spirit_healer_placement_codegen.mjs` pins the seven source-order overworld
graveyards plus the dynamic Spirit Healer definition. `spirit_graveyards.zr`
now reads that generated catalog for nearest-graveyard and range queries, while
`spirit_healer_placement.zr` reproduces the constructor's direct ground-height
placement for every angel. Instance healers, spirit/corpse state, resurrection
offers and penalties remain their owning transaction modules.

`m3_reserved_npc_placement_codegen.mjs` pins Groundskeeper Bram and FURY after
the RNG-driven world roster, including their source reserved IDs
`1_000_000_000` and `1_000_000_001`. `reserved_npc_placement.zr` reproduces
their safe-position and final-ground construction inputs without allocating
them. Vale Cup and PvP behavior, idempotent roster insertion and all NPC
interaction state remain WorldState and their feature owners.

`bootstrap_roster.zr` composes the current source-pinned construction catalogs
into the exact no-player sequential entity schedule: 31 static NPCs, 307 camp
mobs, 55 ground objects, three mailboxes, five dungeon doors and seven Spirit
Healers occupy `1..408`, with `nextId` at `409`; Bram and FURY remain separate
reserved IDs. It supplies materialization order only, not live entity storage
or mutations.

The source-pinned `terrain_noise.zr` now carries exact scalar `hash2`, value
noise and FBM transforms, including JavaScript-compatible negative-grid
two's-complement coercion. Its fixed vectors come from pinned
`src/sim/rng.ts` blob `2d9015bb82c901b8c6aba67b60abc9455b29a786`; it is only
the deterministic noise prerequisite and is not itself a terrain-height or
collision implementation.

`terrain_shape.zr` now ports the built-in `shapeAt/baseHeight` branch with
source-verified vectors for open terrain, zone hubs and lake centres. It also
contains the public terrace and Mirefen crater transforms. The module explicitly
excludes the later `terrainHeight/groundHeight` layers for camps, ridges/rims,
terrain edits, Sowfield, dock surfaces and collisions, so no consumer may treat
`builtinBaseHeight` as a final movement ground query.

`terrain_shape` applies the source camp-flattening loop in array order.
`terrain_mountains.zr` then ports the exact ridge/rim profile, crest noise and
terracing expressions; the checked ZrVM math registry exposes the required
`exp` and `floor` primitives. `terrain_sowfield.zr` and `terrain_height.zr`
complete the pinned built-in `terrainHeight` order: camps, Sowfield level pull,
ridges/rim, Mirefen crater and terrain-edit layer. Fixed source samples cover
each layer, including the level-stamp centre and crater/ridge combinations.
This is static/source evidence only: no ZrVM execution is available, and target
`terrain_ground.zr` also ports the fixed built-in `groundHeight` branches:
strict dungeon threshold, Sowfield stand tiers and continuous dock planks. The
dynamic/custom-content selection and collision/swept-resolution integration
remain required before movement may run.

`terrain_gradient.zr` mirrors the pure four-sample terrain steepness query and
the target's JavaScript half-value rounding before `terrainSteepnessAt`; its
source cache is intentionally omitted because it changes only cost, not the
result. This is sufficient for future slope gates and decoration filtering, but
not a player-motion reducer.

`player_motion_world.zr` now composes the source horizontal coordinate branch
over the generated world query set: no-water/deep-water distinction, steep-ground
slide, grounded and airborne uphill gates, jump fence-clearance policy and
source-style swept collision. Its sweep now selects fixed overworld, standard
instance, arena and Yumi collision routing rather than silently applying the
overworld subset at instance coordinates. It remains a pure field producer and
WOS8 consumes it only through `player_motion_transition.zr`; the seeded
decoration cache, custom-map data and a persistent active-Delve run context
binding remain required for full target-parity movement.

`player_motion_vertical_world.zr` adds the source vertical state transition as
a pure field protocol: deep-water tread/hop, ground jump, same-tick gravity,
water impact, landing reset/fall damage and ledge departure. WOS8 consumes it
through the atomic transition and writes the current scalar HP consequence;
event emission, aura interaction and durable death handling remain later
authoritative integration, so this is not movement acceptance evidence.

`terrain_wall_standoff.zr` now carries the source eight-direction body-radius
terrain-wall setback, and `player_motion_wall_standoff.zr` composes the source
wish-direction compensation, second swept resolve through the same generated
world collision router, and final walkable-slope gate. WOS8 now commits these
pure results atomically; complete collider coverage and the ZrVM
state/collection ABI still block dynamic target-parity acceptance.

`collision_geometry.zr` mirrors the target circle/rotated-OBB `pushOut`
primitive, including exact tangent/no-contact boundaries, centre fallback and
tie-breaking. `m3_collision_content_codegen.mjs` generates the 170 fixed
built-in prop/Vale-Cup records (134 circles, 36 OBBs and six fences), and
  `collision_static.zr` applies their source-order, three-pass resolver with
  fence skipping. `collision_grid.zr` then adds the exact built-in spatial-cell
  selection and deterministic procedural decorations, rather than treating the
  fixed global scan as movement-equivalent. `collision_sweep.zr` adds the same 0.2-yard substeps,
fence-line crossing guard and remaining-vector stop rule. Pinned
`resolvePosition` vectors match this subset exactly. Seeded decoration
colliders, custom-map data and active-module Delve routing remain unported, so this
is still not movement acceptance evidence.

The fixed-world, instance, Delve-collision and Delve-run-layout generators are
all re-pinned to `5ef9f7cb21cd8875b6d2c49701015dfcd78de35a`. Their current-head
extractions retain the existing geometry, routing and selected-run vector
hashes; the rebaseline updates provenance, not the resolved behavior.

`m3_instance_collision_content_codegen.mjs` now extracts the five source
`layoutColliders` sets—crypt, sanctum, temple, nythraxis and arena—plus the
fixed-seed Yumi maze (170 local colliders total). `instance_collision_static.zr`
provides their source-order three-pass local resolver.
`instance_collision_routing.zr` maps the six standard dungeon bands/24 slots,
four arena slots and four Yumi slots to their local layouts before resolving and
restoring world coordinates.
`world_collision_router.zr` composes that route with the fixed overworld
resolver and its source 0.2-yard swept pass. Its declared support set is the
space below `DELVE_BAND_X_MIN` (4773) plus the fixed-seed Yumi band
`[8000, 12000)`: the active-module Delve band and unrelated far-east fall-through
region are explicitly rejected rather than treated as open-world geometry. This
is still pure rule coverage, not movement-acceptance evidence.

`pathfind_state.zr` now projects the source local A* over that same generated
world-query boundary: one-yard cells, eight-yard margin, source 64-cell default
span/direct fallback, eight-way octile heuristic, blocked diagonal corners,
ride-height uphill gating and 0.25-yard string-pull samples. It preserves the
raw destination as the last waypoint, applies player deep-water rules only
inside authored lake footprints, keeps swimmers on the water surface and retains
the 24-ring destination-to-shore scan. The focused contract covers open direct
routes, static detours, walker/swimmer clicks, fence policy and oversized-window
fallback. It is static source coverage only: dynamic execution awaits the
Runtime13 scalar math backend, while procedural decorations and active Delve
selection remain outside `world_collision_router`'s declared support set.

`instance_line_of_sight.zr` now mirrors the source half-yard intermediate-sample
line-of-sight query for the generated standard-dungeon, arena and Yumi layouts.
It uses the same strict `pushOut` overlap boundary and has source-regenerated
blocked/clear vectors for all three routes. It deliberately rejects open-world
and Delve queries: those paths need the decoration/camera-top cache and active
module-state selection respectively, so this is a precise combat-world primitive
rather than a complete LoS substitute.

`m3_delve_collision_content_codegen.mjs` now extracts all eleven source Delve
layouts (473 colliders across four Reliquary and seven Litany modules), their
spans, routing constants and the two source default chains. The generated table
and `delve_collision_static.zr` provide an exact source-order local three-pass
resolver for a caller-selected module. `delve_collision_routing.zr` can adapt an
explicit active module origin/index to world coordinates. The companion
`m3_delve_run_layout_codegen.mjs` locks the source run selector, mulberry32 RNG,
layout table and both Delve definitions; its generated scalar projection applies
the source Fisher-Yates draw to `delveIndex`, `slot`, `seed`, source-order tier
route and module offset, then derives the selected module and its world origin.
It covers 42 pinned normal/heroic/unknown-tier vectors, including the target
zero-seed normalization. `DelveState` now binds the immutable scalar inputs
(`delveIndex`, slot, seed and tier route), then re-derives the source module
list/index and origin as progression advances. This removes no-active-run
fallback from the active selection path, but is not yet an authoritative
`WorldState` integration: the current ZrVM custom-object/collection ABI cannot
validate the target's full mutable `DelveRun` object and its atomic lifecycle
truthfully. Within that state owner, the active context now composes the
explicit collision resolver, 0.2-yard sweep and half-yard LoS primitive for
source-selected Reliquary/Litany module vectors. `delve_default_collision_router.zr`
separately ports the source's
exact no-active-run fallback, including unknown-x Reliquary fallback, slot
selection and default-chain module transitions. It is not wired into
`WorldState`: using it for an active run would be source-incompatible.
`m7_delve_module_content_codegen.mjs` additionally locks the exact 11-module
spawn/interactable/hazard catalog (46 initial mob spawns, 16 interactables and
45 hazard zones). The source-run `DelveState` path now uses its real selected
module's initial mob and puzzle counts instead of the former fixed `3/2`
placeholder. It also maps source puzzle offsets to their actual interactable
indices, rejects duplicate triggers and opens a non-finale exit only after all
current-module puzzles fire. Its source puzzle entry gate now uses the target
2.5-yard walk-on boundary for pressure plates, sluice valves, grave tablets and
corpse candles; bell ropes alone retain the 6-yard deliberate-interaction
boundary. Mob
entities, object identities/link state, other object interactions, Litany room
waves and rewards still require the complete run lifecycle and are not claimed
by this scalar projection. `m7_litany_dry_ground_codegen.mjs` now pins the
seven authored Litany dais and 55 island safe-ground regions from the same
source commit, including their source-order module indices. The separate
`delve_blackwater_rules.zr` composes that data with the existing 45-zone catalog
to match the source's inclusive ellipse boundary, deep-over-shallow precedence,
airborne and dry-ground exemptions, normal/heroic `4%`/`8%` damage, `high_water`
`1.35` multiplier, positive rounding/minimum damage, and one-second timer reset.
It remains a pure scalar rule: party/death filtering, entity damage emission and
the persisted `DelveRun.blackwaterTimer` have not been connected to the mutable
run lifecycle, so this is neither a live hazard tick nor dynamic ZrVM evidence.
`m7_bad_air_content_codegen.mjs` AST-extracts the source's eight-second Bad Air
clock and full self-sourced nature-DoT specification (`4` seconds, value `3`,
two-second periodic cadence). `delve_bad_air_rules.zr` preserves the source
order: an inactive affix leaves the timer untouched; a due clock resets before
the party/alive checks; only then can the caller apply the aura. The pinned
JavaScript source reaches the raw `>= 8` predicate on the 161st repeated `0.05`
addition because it does not use an epsilon or fixed-tick replacement; this is
captured as a source boundary, not asserted as dynamic ZrVM evidence. Aura state
mutation and per-member iteration still belong to the future `DelveRun` owner.
`m7_delve_raise_dead_content_codegen.mjs` now locks the source's five-second
Raise Dead channel, `cracked_grave` start/interrupt gate, and its completion rule:
the channel always clears on expiry, but boss adds spawn only when the recorded
boss remains alive. `delve_raise_dead_rules.zr` exposes a caller-side
`canStartOn` query and projects that state boundary; grave/boss IDs, range
validation, log emission and actual add entities remain
with the future `DelveRun` owner.
`m7_delve_restless_graves_content_codegen.mjs` locks the death-hook contract:
eligible non-boss/non-elite, non-affix-spawned mobs enqueue a
`reliquary_bonewalker` at their death coordinate for `now + 3` seconds.
`delve_restless_graves_rules.zr` preserves multiple pending entries and source
order when ready entries are consumed. Concrete mob construction, rebucketing
and run ownership remain outside this scalar queue projection.
`m7_delve_affix_selection_codegen.mjs` now locks the source-order implemented
affix pools and all 32 Normal/Heroic seed vectors. Its Zr projection uses the
same Mulberry32 shuffle and `seed ^ 0x5a11c0de` route: Crypt selects from
`restless_graves`, `bad_air`, `candleblind`; Ruin selects from `high_water`,
`lively_choir`, `belligerent_dead`; Normal returns none and Heroic returns one.
This covers deterministic selection only. The selected effects still require
their corresponding mutable run/entity integrations.
`m7_baptistry_content_codegen.mjs` also AST-extracts and source-locks the
Sinkhole Baptistry room source without depending on its unresolved aggregate
import: three waves with exact `6/6/3` spawn order/coordinates, the three
egg-sac spots, burst radius/percent/despawn constants and hatchling spawn
limits. `delve_baptistry_state.zr` models the source's initial wave, clear-gated
wave advancement, egg-sac enablement after wave three, once-only burst handling,
two hatchlings per burst, normal/heroic level bonus and the
`belligerent_dead` bulwark 1.1x scale. It intentionally carries only scalar
room state. Entity IDs/deaths, collision-aware randomized hatchling positions,
per-party logs and actual damage emission remain unconnected, so it cannot be
treated as a live Baptistry run or M7 dynamic evidence.
The Bad Air and Baptistry generators are re-pinned to current source
`5ef9f7cb`: their extracted content hashes remain respectively
`fc4ba3bdc9f01043cd62a679acd208f2e6844f6c212fb5656f84c56b123159fd` and
`7e47e5ee102ba924fd64ea149d0aa72befc830310bf81e65480ec20dc2ef377d`, while
the shared `runs.ts` blob now records
`374bf2a609668a5a0db62e7835bb76749ff85072509de2f96eb874fc05af65ff`.
The matching Candleblind contract is also current-source-pinned with unchanged
content hash `85abb00978edb529be9c0aee0a9c997375ecf9730a06680e72bc5ddb00ba6dc1`;
its generated projection carries both the inactive/active detection multipliers
and the active-affix requirement, which `delve_candleblind_rules.zr` now
exposes. These are scalar rule inputs, not a live visibility system.
This is current-source content provenance only, not mutable run integration or
dynamic acceptance.
`delve_collision_sweep.zr` additionally mirrors the source 0.2-yard scan,
fence guard and remaining-vector stop rule inside one explicit module context;
its pinned Reliquary and Litany vectors are regenerated by the same source
extractor. A movement crossing into another module still requires the active
run to select and supply the next context; `DelveState` can now supply it
locally, while the future WOS8 Delve-run extension must preserve it atomically.
`delve_default_line_of_sight.zr` separately ports the source no-active-run LoS
fallback: it fixes the module context from the start point and samples the ray
at half-yard intervals in that module's local space. Its `delve_line_of_sight`
primitive accepts an explicit active-module origin/index for the future run
state owner. The Litany LoS layouts were verified equal to their movement-
collider layouts at the pinned commit; choosing an active module remains
state-dependent and is not inferred from the fallback.

The pinned decoration filter inputs are present, but a source-compatible
decoration collider cache requires persistent, ordered ZrVM collections.
`decoration_candidate.zr` now ports the pure candidate grid, source hash
offsets, biome density/mix, road/shell/hub/camp/water/slope filters and collider
radius rule. Its seed-20061 target source evidence is 989 decorations (453
tree, 244 tree2, 292 rock), with fixed representatives for all collider cases.
The current Plugins 08 collection/instance ABI defect prevents proving the
cache; recomputing thousands of candidates per 20 Hz player tick is rejected as
a behaviorally and operationally invalid substitute.

The `tab`, `targetNearest`, `tabFriendly` and `targetNearestFriendly` payloads
now also have a dependency-independent ZrVM target-ordering module and focused
project. It preserves the source 40-yard boundaries, flared facing cone,
engaged/visible tier order, near-cluster wrap, distance/ID tie breaks and stable
friendly traversal. This is source progress only: `WorldState` dispatch still needs the
relationship candidate projection, and the module cannot cross the current
custom-class/container ABI or receive a fresh CLI run yet.

The casting slice now also has a stateful 20 Hz lifecycle projection rather
than only scalar rules. It preserves casting-before-GCD-timer tick order,
single-slot tail queue replacement/retry, fixed-count channel tail flushing,
cancel cleanup, fishing completion isolation and distinct timed/channel
pushback formulas. WOS8 introduced its compact per-entity lifecycle columns and
WOS9 preserves them while
performs the single `CastState` copy-in/tick/copy-out inside the candidate
transaction; ability admission, cast arming and effect resolution remain open.
This wiring is static only because it deliberately crosses the open Plugins 08
custom-object/field ABI boundary.

The WOS9 movement path now also observes the source distinction between death
teardown and an ordinary cast interruption: after landing damage marks a row
dead, it clears the active cast and retained cast-target lock without creating a
failed `castStop`, clearing the queued press, or changing lifecycle counters.
This is only the narrow landing-death handoff; generic combat death, respawn,
events and cast-target identities remain unimplemented M4 work.

`combat/known_ability_state.zr` now separately pins the current
`abilitiesKnownAt` selection boundary needed by `castSlot`: preserve class-kit
order, append only new granted abilities, skip missing definitions, apply the
level/spec/exclusion gates to non-grants, and apply talent modifiers only after
selection. It accepts catalog-owned ability codes and availability facts rather
than creating a partial class kit. The current-head catalog generator now supplies
all nine class-kit orders and every ability's level/spec/passive visibility facts,
and WOS9 now persists a per-entity source-order known-list partition. The
offline bootstrap fills only the newly spawned player from its current source
class kit at bootstrap level, with no grants or committed specialization; the
existing offline quest-XP reducer rebuilds that partition whenever it dings,
preserving other entity partitions. WOS2 through WOS8 decode to empty
partitions. Catalog codes now derive their base source rank from entity level in
the declared rank order. WOS10 adds a fixed six-row allocation and
specialization code only for the offline primary player; a source-valid
allocation atomically refreshes its WOS9 known partition with specialization
and row grants while preserving every other partition. This deliberately does
not claim the full effect/cost/cast/modifier resolver, combat/arena allocation
locks, loadouts or wire-command dispatch. `castSlot` therefore remains
explicitly rejected instead of mapping a source slot to an invented M4 index.

`talent_selection_catalog_codegen.mjs` now locks all 27 current specialization
identities/signature abilities, 162 six-row option identities and 54
row-granted abilities into scalar codes. `sourceKnownAbilityCodes` mirrors the
selection half of `abilitiesKnownAt`: valid specialization signature first,
then selected row grants in source row order, followed by the existing
base-plus-grants dedupe and eligibility filter. WOS10 consumes those scalar
codes only for direct offline state replacement and persists the resulting
allocation. It deliberately stops before talent numeric/proc/effect modifiers.
The current `applyTalents {alloc}`, `respec {}`, `setSpec {spec?}` and
`selectTalentRow {level,optionId?}` commands
now use generated scalar catalog codes within the offline reducer; reliable
cross-module ZrVM transport still depends on the Plugins 08 transactional boundary.

The generated current ability catalog is intentionally one 2,777-line
machine-produced lookup/data owner, rather than a hand-maintained mixed module.
Ranks remain part of that same ability metadata boundary. Any later generated
cost/cast/effect projection must use a separate `current_ability_effect_catalog`
owner instead of appending another subsystem to this table.

The sixteen scenario factories now also have a generated, source-pinned catalog
of the 21 abilities they actually invoke. The generator evaluates the current
`5ef9f7cb` Git blobs, records their LF identities and verifies the scenario blob
against the source manifest under its known LF or CRLF representation. The
current manifest records the LF blob identity. The catalog is source inventory
only; ability execution and real-M2 exact-golden acceptance remain open.

That catalog now also generates two scalar Zr modules: one resolves learned
rank, cost, cast time, cooldown, range, threat, channel and targeting metadata;
the other exposes the exact rank-replaced effect list and its typed scalar/text
fields. A focused source contract covers all 21 ID round trips plus rank/effect
vectors for warrior, mage, priest, warlock, paladin, druid and hunter content.
This is still not accepted dynamic evidence: the current CLI can compile the
projects, but natural object/container execution remains unreliable and the
modules are not yet wired into `WorldState` casting/effect dispatch.

A separate authoritative admission reducer now consumes those scalar modules
in the upstream `castAbility` guard order. It preserves delayed timed-cast cost
and cooldown, immediate channel billing, tail-window single-slot replacement,
next-swing toggle/consume behavior, cost-tax rounding, forms/seals, combo gates
and the target/range/LoS/facing checks supplied by a read-only world projection.
The reducer is source-complete for these admission transitions but remains
unaccepted until its focused ZrVM project runs and `WorldState` integration is possible.

The lower combat transforms are also stateful now: spell coefficient routing,
aura replacement/periodic expiry, classic regeneration/timers, ordinary
damage, direct/periodic healing, melee/projectile auto attack and the ground-
AoE lifecycle each have focused ZrVM projects. Damage/heal preserve modifier
and absorb order, threat/tap/combat side effects, reflect/Frenzy recursion,
healing-threat fan-out and their exact guarded RNG draws. Auto attack preserves
queued next-swing attacks, single-table melee resolution, hunter/wand range
differences, projectile-arrival RNG and retaliation tail order.

The auto-attack projection now exposes its actor, target and event carriers for
the future WOS-owned transaction and can restore/consume/re-export the exact
`kernel/rng` state, draw count and digest on the branch that actually draws.
Focused vector fixtures remain supported ahead of that authority path. This is
a reusable source/static seam only: no `WorldState` combat integration, custom
object/container runtime result, or dynamic M4 acceptance is claimed.

The WOS command path now consumes the source initial melee `startAutoAttack`
transition for an ownerless idle mob: it records the primary chase/aggro/combat
and leash state, applies the source's two initial threat additions, and invokes
the same-template social pull owner for nearby idle allies. The command-state
regression proves strict radius, template exclusion, primary versus ally leash
placement and fixed6 threat preservation. This intentionally stops before the
per-tick swing/projectile/damage/death/retarget/exit chain; that chain must land
as one shared M4 transaction rather than as command-local partial logic.

The same shared primary transition now receives the current offline Eastbrook
idle proximity selector before idle wander. Its generated `aggro_radius`,
level/trivial-con and strict nearest-player rule activate wolves and boars
without RNG draws; converted rows then naturally bypass the idle-wander arm.
This deliberately supplies neutral detection inputs for the unrepresented
stealth/delve subset and does not claim melee damage or exit parity.

The retained Eastbrook ordinary-wild lifecycle now closes the independent
corpse/respawn tail for those same ownerless wolf/boar rows. It mirrors the
source dead-state label, 60-second corpse timer, 30-second default respawn
timer, 20 Hz countdown, lootable-corpse deferral, spawn-transform/full-HP reset,
threat/forced-target reset and one `range(2, 8)` Mulberry wander-timer draw.
This is deliberately scheduled after the narrow idle/wander passes, so a
revived row does not receive an extra idle-aggro scan in its respawn tick. It
does not claim the upstream swing/projectile/damage/death-credit chain, loot,
pet/instance/boss lifecycle, aura reset or dynamic ZrVM acceptance.

The same source-audited Eastbrook subset now runs the narrow melee-pursuit
profile after its lifecycle pass. Given the already-owned direct live player
target, it preserves the previous transform, source move/turn and melee-distance
rules, the pre/post swing-timer ordering and the resulting `chase`/`attack` AI
label. Each recorded swing now enters the source-pinned base `mobSwing` shell:
one hit-table draw, weapon-range draw and fixed-5%-crit draw flow through the
committed Mulberry cursor, then base physical damage applies target dodge,
warrior front-arc parry/block, armor reduction and HP/death-posture handoff.
The retained wolf/boar templates have no on-hit mob-swing affix, so their base
branch is closed without inventing cascade draws. It does not yet cover
unsupported-form auto profiles, revenge, auras, full death
teardown, retarget threats, pull allied mobs, leash/flee/evade, or dynamic-ZrVM
acceptance. The already-armed offline direct-melee profile now also crosses its
live wolf/boar target through the exact mainhand/offhand white timing/hit table
and shared cursor: both weapon timers decay before the Travel gate, each normal
dual-wield swing adds the source `0.1` miss penalty, and a valid offhand applies
its independent speed plus `0.5` damage multiplier. Warrior, rogue, paladin and
shaman have no source ranged white-hit profile, and druid is admitted in Bear/Cat
form; Travel and action-locking Fireball travel form cancel auto-attack after timer
decay and reject an attack command without seating, drawing or aggroing the target.
A lethal result feeds the existing ordinary-wild
lifecycle immediately after player updates. The retained
wild boar also applies its source fixed two-damage Bristled Hide reaction after
a landed melee hit, including a killing hit while the attacker remains alive.
WOS54 adds the retained ranged-auto closure: Hunter Auto Shot's 8-35 yard profile
and caster wand's 0-30 yard/no-dead-zone profile launch without pre-aggro, capture
wand marker, school and min/max/speed in a persistent source-order queue, home on
live target x/z before player swings, and perform their hit/range/crit
draws only when the projectile lands. A dead source or target fizzles with no
draw, while the five-yard Hunter fallback reuses the direct melee bridge. Queued
swings, aura reflection, and set/talent/weapon
procs remain outside that narrow player bridge.
Direct Eastbrook white-melee deaths commit the local swing cursor, award the
source `mobXpValue` result, then run the existing active-only, eight-kill-bounded
Forest Wolf `q_wolves` ledger before consuming the pinned corpse table in source
order. The single-player projection retains the source base `45 + 5 * mobLevel`,
level-difference cap, gray threshold and half-up rounding, but leaves party bonus/
division, elite multipliers and rested XP to their owning systems. WOS51 retains
the active-and-needed `q_boars` personal `boar_hide` slot, while WOS52 retains
generated copper and ordinary item slots. Native `loot` transfers copper first,
then the personal and shared entries after dead/range/capacity checks and
recomputes the existing collect ledger. Party/FFA rights, group rolls and general
death-credit selection remain separate transactions.

Three additional generated-catalog consumers partition all 19 effect types
present in the 21 scenario abilities into numeric, aura/CC/form and world/pet/
ground owners with no gap or overlap. They preserve range-before-crit draws,
hybrid DoT/HoT scaling, seal consumption, PvP fear/controlled-stun DR, reverse
imbue/form exclusion, sunder miss/threat, stable AoE traversal, taunt branches,
summon requests and synchronous ground-pulse-before-enqueue delegation. Ground
AoE itself preserves reverse 20 Hz drain, strict 10-second expiry, stable
target/LoS filtering and the fact that only the immediate pulse carries
ability-specific threat inputs. This review corrected the old scalar
Consecration contract from six pulses to the source-verified five at 0/2/4/6/8
seconds.
`combat/threat_state` separately ports the target's primitive threat table
semantics without a mutable entity bridge: multiplicative stance/form and Holy
Righteous Fury modifiers, clamped stealth detection, ordered accumulation,
forced-target cleanup, and stable rounded top-N meters. It is intentionally not
a claim that WOS has wired target switching, healing-threat fan-out, or combat
entity mutation; that integration remains source/static pending the reliable
ZrVM object/container boundary.
`combat/flee_speed_state` also preserves the current pure flee-speed invariant:
base movement times the 1.4 flee multiplier and active speed multiplier is
capped only at the final value, at 65% of the 7-yard player run speed. This
prevents haste from making a fleeing mob uncatchable while retaining slow effects
below the ceiling. AI/pathfinding and WOS movement mutation remain separate.
`combat/stun_dr_state` now carries the current split stun categorization: Cheap
Shot/Pounce are opener stuns, Kidney Shot/Hammer of Justice/Bash/Charge/Bear
Charge/Faultline are controlled stuns, and unknown proc stuns use randomStun.
The independent buckets prevent an opener from diminishing the next controlled
stun; live DR timers and aura mutation remain canonical dispatch work.

The remaining focused combat owners now cover the run-effects envelope, common
death teardown, the base mob swing plus its source-ordered affix cascade, the
one-draw spell-resist gate and the two scripted encounters. Review of the pinned
`nythraxis_full_pull` source corrected the old encounter scalar from two
conceptual mechanics to four actual draws: Gravebreaker draws once and normal
Soul Rend selects three marks without replacement. The corrected scalar
encounter project currently returns `1`/exit `0` in interpreter and binary
modes. Drowned Litany and base mob swing each return `1`/exit `0` in fresh
interpreter and binary probes.

`combat/nythraxis_state.zr` now models the 20 Hz phase-one/transition/phase-two
path, three Soul Rend marks and expiry split, three distinct ward channels,
Deathless interruption/self-stun, Final Stand and death/lockout call points.
Its project compiles two reachable modules from an absent output directory, but
execution remains RED: even a single 33-field state with only three container
references and `combatData` declared as field zero reports
`GET_MEMBER: missing member 'combatData'` in `addRaidPlayer`. A six-line inline
`K { a: A, b: B }` reproduction separately loses `b`. This is appended to the
canonical Plugins 08 open failure and is not a WOC acceptance result.

The world-effect dispatch project has separately advanced from a WOC-owned
compile error to `compiled=5`: its ground-AoE path now stores the integer helper
result before converting it to `float`. Interpreter and binary execution then
fail on the first read of the freshly constructed state's field zero with
`GET_MEMBER: missing member 'source'`. That runtime result is recorded under the
same Plugins 08 object-shape gate and is not counted as a combat pass.

The same current-source audit compiles 11 focused admission/aura/damage/death/
effect/ground/heal/resist projects. Numeric dispatch's five imported call-casts
were reduced through typed locals and now compile five modules before the
interpreter reaches the same first-field `source` RED. Ability admission also
passes a fresh compile plus interpreter and binary contract runs (`1`, exit
`0`) after constructor module lookup is moved to its caller and cooldown arrays
are bound before indexed writes. Its cat-form rejection fixture now satisfies
the pinned source's earlier resource guard before asserting the later form
gate. This is focused rule evidence, not one of the 16 real-M2 golden passes.
Effect-sequence ordering and the one-draw spell-resist state independently pass
fresh interpreter and binary runs (`1`, exit `0`). Aura state remains RED in
the ZrVM C dispatch core before its contract assertions: a custom-object string
field reaches equality without a string runtime type even after every generic
string-array comparison is explicitly cast. That assertion is a Plugins 08
runtime failure, not a WOC expected-value mismatch.
Casting lifecycle had fresh interpreter and binary evidence before the WOS8
copy-in/copy-out update; that historical focused result does not validate the
current module or WOS8 integration. The generated ability catalog independently
passes in fresh interpreter and binary modes. Regeneration, damage, healing, auto attack,
ground AoE and
aura/numeric/world dispatch remain RED on lost declared fields; mob-affix and
aura state hit the object-string C assertion, while spell scaling exits with
Windows access violation `-1073741819` before a structured result. These are
Plugins 08 runtime/compiler gates, not real-M2 progress.

Current static validation parses 61 focused Zr projects, checks all 15 MJS
tools, retains the latest green seven-generator evidence, proves the 19-type
owner partition, and finds no trailing whitespace across 351 WOC text files.
None of these
source checks is dynamic or exact-golden acceptance; real-M2 M4 remains 0/16.

**Testing stage - M4 combat parity:**

- Run combat/casting/effect module suites, all sixteen scenarios twice, exact golden comparison and target coverage assertions.
- Attribute the first differing RNG draw/event/state field to the lowest shared system; never regenerate a golden or widen trace rounding.

**Exit evidence:** Exact parity for the sixteen named scenarios and stable combat event/RNG ordering across repeated runs.

## M5 - Progression, quests, inventory, loot and economy parity

**Goal:** Port durable character progression and transactional item/economy systems covered by eleven maintained scenario names plus current-head Talents V2 and profession contracts.

**Dependencies:** M4.

**Scenario gate:** `g1b_xp_prestige`, `talents_progression`, `inventory_vendor`, `l1_loot_distribution`, `party_loot`, `bank_round_trip`, `market_round_trip`, `player_trade`, `quest_collect_turnin`, `quest_kill_credit`, `quest_link_abandon`.

**Progress evidence (not exit evidence):** The 11 pinned factories now drive a
Git-blob-generated M5 content catalog pinned to
`5ef9f7cb21cd8875b6d2c49701015dfcd78de35a`. Its exact scope is 14 items, two
quests, one mob, six NPCs, three current `talent_option` rows (`Double Charge`,
`Die by the Sword`, `Victory Rush`), three scoped abilities and two specs, plus
the 20-level XP table, prestige threshold, market cut and bank expansion prices.
Quest-objective closure derives `boar_hide` and the q_wolves `forest_wolf`
ownership, while the selected mob's real loot table derives `milepost_boots`
and `wolfhide_satchel`, rather than hardcoding those indirect references. The
selected Arms/Fury definitions also derive their signature abilities, adding
`bloodthirst` beyond the two saved action-bar IDs. The Talent V2 subset carries
its row level, grant ability and Double Charge's `bonusCharges: 1` scalar. The
catalog SHA-256 is
`9ff52d9d80e5c269cb06850c7ad73da4ce6b18f741cb2f88f9de00fce1b730b6`, and a
second extraction passes byte-for-byte `--check`. No real-M2 M5 scenario is
accepted yet.
The checked JSON also generates a scalar Zr catalog for item, quest, mob/loot,
NPC/vendor, Talent V2 option, ability and progression/economy lookups. It has
generation and byte-for-byte evidence only until a managed ZrVM project session
can execute its focused tests; no old dynamic result is promoted to this
current-source projection.

The separate current-head `m5_class_baseline_stats` contract evaluates the
source `Sim` stat chain for the nine picker-order classes at every level 1--20,
with each class's start weapon/chest and without talents or auras. It cross-checks
the current M8 bootstrap and fresh-player contracts before projecting the 180
profiles into scalar Zr lookups. Its catalog SHA-256 is
`64fe1243cd80bdf329dc51a439c803fb83ea0fd3900de45d992ed8de7e51f032`; a
second source evaluation passes byte-for-byte `--check`. It is a derived
baseline input, not acceptance of leveling, equipment, aura or combat updates.

The separate source-pinned `m5_camp_mob_loot` catalog now records every one of
the 47 camp-template loot and component-tag rows in camp first-appearance
order: 177 entries (131 item, 46 copper, 23 quest-gated and 24 roll-group
entries) plus 29 component tags. Its Zr projection retains presence flags for
every optional source field. This is a data prerequisite only: source-order
rolls, personal quest recipients, heroic substitutions and corpse/inventory
transactions remain unaccepted M5 reducer work.

Eight dependency-independent M5 state owners now cover all eleven named
scenario factories at the source-contract layer. XP owns 20 Hz rested accrual,
kill-only consumption, level/lifetime/virtual-level transitions and prestige
anti-abuse; the current Talent V2 owner validates whole `{ spec, rows }`
candidates, preserves rows across Arms/Fury switches, clears only rows on
respec, derives known abilities from the full current catalog, projects Double
Charge's bonus charge, and stores ten 22-slot loadouts; its 5-level
specialization gate is derived from `ROW_LEVELS[0]`, not hardcoded. Full row
effect dispatch remains open. The separate generic
`talent_allocation_commit_state` covers all 27 current specs and 162 current
row options after canonical code decoding. Its current-head contract fixes the
whole-allocation lock/validation/equality/mutation order and the intentionally
different `setSpec`, `selectTalentRow`, and `respec` precheck orders; it returns
the source recompute, stat, known-ability, proc, charge, form, offhand, pet,
echo, and log obligations to WOS without duplicating world mutation. That
projection is source/static until the reliable ZrVM project backend executes
its focused test. `talent_modifier_state` now applies all current 27 mastery
and 162 row `TalentEffect` records in the target's accumulation order: 18 stat
fields, 39 global fields (including max-only `cheatDeathIcd`), 51 ability
modifier rows, grants and the 55 current proc identities. Its generated
catalog also carries all 12 nested `AbilityEffect` DTOs from 11 modifier rows,
so the WOS effect dispatcher has complete current-source input for root, slow,
area-root, absorb, dot, extend-dot, interrupt and consume-dot handling. The
DTO application itself remains WOS-owned and source/static until the reliable
project backend executes the focused test. `talent_world_commit_state` composes
the allocation transaction and modifier reducer only after a changed successful
commit; equality short-circuits and rejected candidates retain the prior scalar
projection. `combat/talent_added_effect_state` now consumes the selected nested
DTOs for one resolved ability and emits the source-order appended tail after the
same current `applyTalentMods` scalar pass: area-root min/max use damage scaling,
absorb uses heal/absorb plus flat scaling, and a DOT `directPct` rider is left
unscaled to avoid a second direct-hit modifier. Root, slow, extend-dot,
interrupt, and consume-dot payloads remain lossless. The pure bridge does not
receive native effects or mutate entities, so canonical M4/WOS composition,
dispatch and reliable ZrVM execution remain open. Its Talents V2 save migration now uses the
current full ability catalog to retain only known, non-duplicate main-bar
skills, preserves valid form/stealth-restricted skills already on the bar,
excludes passives and warrior stances, and fills empty slots from the
spec-only non-form/non-stealth baseline in declaration order. Exact
string-slice name truncation and dynamic execution remain open. Inventory/vendor owns pooled capacity,
equipment stat refresh, consumables, buy/sell/junk/buyback ordering and
pre-commit rejection. Its current-head scenario still proves the helmet exchange path;
the bounded 14-item content projection also exercises projected `feet` and `mainhand`
slots through that same swap/unequip transaction. Current-source rings, `held_offhand`,
specialization-sensitive offhand revalidation and Titan Grip remain outside that bounded
item catalog and are not represented as accepted parity. Bank owns partial/whole pooled moves, indivisible item
instances, atomic capacity refusal and the 24-plus-6-slot expansion schedule.
Market owns escrow, query, the floored 5% cut, buy/cancel, 20-tick expiry and
collection; player trade owns offer clamping, double confirmation, final
ownership/currency/capacity revalidation, atomic swap and distance
cancellation. Its current source fixture now uses the shared instance ledger:
each capacity preflight removes outgoing fungible stock first, treats incoming
instance copies as non-merging slots, and the atomic transfer preserves the
specific signer/charge/rolled/enchant/binding payload. Focused contracts cover
both a preserved instance and the full-bag same-id rejection; they remain
source/static evidence until the ZrVM project session can run them. Quest state covers collect recomputation, ready demotion,
kill credit, linked-party acceptance, abandon and turn-in rewards across its
three scenarios. Loot distribution covers the remaining two scenarios with
death-time candidate order, fair-split remainder selection, Need-over-Greed,
all-Pass return-to-corpse, common/personal/open visibility and conservation.
Its three-player contract consumes exactly the three loot-owned draws; the
golden's fourth draw belongs to a later unrelated world tick. All eight focused
projects remain source-only pending a reusable ZrVM CLI. A separate scalar M5
scenario matrix now routes the eleven fixed scenario names, in gate order, to
the eight owners; quest and loot expose distinct `3 + 2` scenario entrypoints
instead of reusing one aggregate result. This matrix is import/routing evidence,
not an authoritative-world or WTR1 driver, and real-M2 M5 remains 0/11.

**Implementation slices:**

- [ ] Implement XP/level/prestige, Talents V2 row choices/loadouts/save migration, currencies, item instances, stack/equipment/bag rules and derived stat refresh.
- [ ] Implement loot tables, personal/group loot, rolls, vendor buy/sell, bank, player trade and market listing/buy/cancel/collection transactions.
- [ ] Implement quest prerequisites, objectives, shared/link state, kill/collect credit, abandon and turn-in rewards.
- [ ] Port required item/talent/quest/vendor/profession content, including profession identity, combo eligibility, masterwork and attunement, scenario drivers and coverage assertions; add conservation, capacity, duplicate-delivery, migration and rollback tests.

**Testing stage - M5 durable-system parity:**

- Run progression/economy/quest module suites and all eleven exact double-run golden comparisons.
- Verify every failed transaction leaves item/currency/quest state unchanged and every successful transaction conserves ownership and currency.

**Exit evidence:** Exact parity for all eleven scenarios and atomic durable-state transitions suitable for later persistence.

## M6 - Social, party/raid, PvP and fiesta parity

**Goal:** Port multiplayer coordination and competitive rules covered by nine historical scenarios plus the current-head `card_duel` scenario.

**Dependencies:** M5.

**Scenario gate:** `chat_social`, `party_raid`, `targeting_markers`, `arena_1v1`, `arena_2v2_wipe`, `duel_to_winner`, `fiesta`, `fiesta_powerups`, `fiesta_midcast_kill`, `card_duel`.

**Progress evidence (not exit evidence):** Six dependency-independent state
owners cover all nine named scenarios at the source-contract layer.
Chat
preserves the 8-message/2-per-second token bucket, world/lfg opt-in routing,
General's single broadcast event, say/yell ranges, party delivery,
whisper/reply state, emote fan-out, inspect/help readouts and overhead-emote
sequencing. Its pinned drive produces 59 ordered business/delivery events and
ends with A/B/C token balances `0/1/0`; C's final two sends are the only
throttled operations. Party/raid preserves invite expiry, one-party ownership,
leader-only mutation, five-player conversion, deterministic two-subgroup fill,
cross-group movement, join-order leadership handoff, post-removal subgroup
normalization and final disband. The exact seven-player drive records 22 ordered
authority actions without consuming RNG. Both focused projects are source-only;
the targeting/marker owner composes the existing deterministic enemy/friendly
selection contract with party-scoped, unique-symbol marker storage and
death/disband cleanup. Ranked Arena owns stable equal-rating matchmaking, 5 s
countdown, first-down versus team-wipe distinction, symmetric Elo/flooring,
duplicate-result rejection and 5 s return; its exact base-rating results are
`1516/1484` for both 1v1 and 2v2. Duel owns 30 s invitations, 3 s countdown,
60-unit forfeit, 1-HP finishing protection and decided-only win/loss counters.
Fiesta owns the 15-point score bout, growing 3-14 s respawns, three deferred
augment waves, 22-to-6 ring contraction, twice-per-second 6%-per-second ring
damage, and the 12/16/5/18 s power-up spawn/interval/telegraph/TTL lifecycle.
Its per-match random stream is counted separately from shared combat RNG;
augment offers use Fisher-Yates and a power-up spawn consumes exactly three
private draws. The mid-cast contract preserves normal-cast `+0.5 s` pushback,
fishing cancellation, lifesteal-before-lethal handling and damage-event-before-
takedown order. A fixed matrix routes all nine historical gate names, in order,
to these six owners. The current-head Card Duel source port begins with the
deterministic deck/hand, FIFO queue and best-of-three match core: it preserves
shared Mulberry32 Fisher-Yates draws, first-held-card removal, discard
recycling, pre-score voids, scored forfeits and the inclusive 90-second
deadline. WOC now composes that core into the WOS38 candidate with typed
 command ingress, source-id Card Master range lookup, liveness sweep, bounded
 `CDS1` persistence and the enclosing WOS RNG cursor; the Card Duel stream is
 not a second local authority. Player-name projection, source events/deed
 credit, full static roster materialization and real ZrVM transaction evidence
 remain open. All M6 projects are source-authored/static-checked only. The
 composed selector still carries the known ZrVM `near` field RED, and M6
 dynamic/real-M2 acceptance remains 0/10.

### M6 holder-tier pure projection

**Goal:** retain the current source's cosmetic/analytics holder ladder as a
shared pure rule without treating a wallet balance as simulation state.

- [x] Project all eighteen stable tier keys and thresholds from Ember at one
  whole token through Sovereign at the 1,000,000,000-token maximum supply.
- [x] Preserve highest-qualifying-rung selection, the source's no-wallet result
  and threshold-share calculation. The scalar boundary represents `null` as
  `hasBalance=false` and invalid index lookup as an empty key/zero threshold.
- [x] Add focused source-pinned coverage for both ladder regions, maximum
  supply, metadata lookup, shares and unavailable/out-of-range scalar values.
- [ ] Keep wallet acquisition, presentation delivery and any token economy out
  of WOS unless a future source contract explicitly makes them gameplay state.

### M6 text-rule prerequisites

**Goal:** preserve source text semantics rather than silently reducing chat and
aura behaviors to exact ASCII-string comparisons.

- [x] Audit current `/assist` and aura-classifier source contracts against the
  available Zr script/runtime text surface.
- [ ] Provide reliable Unicode-aware `trim`, case folding and prefix testing in
  the ZrVM plugin. These are required for `/assist` exact-case precedence plus
  unique case-insensitive matching, and for the open-ended `buff_` branch in
  `isDebuffAura`.
- [ ] Port those source rules only after the text APIs are executable; do not
  replace the source's open string semantics with a finite hand-maintained
  identifier list.

**Implementation slices:**

- [ ] Implement chat channels, ignore/friend/social state, party/raid membership, leader/assistant roles, ready state, target markers and group event ordering.
- [ ] Implement duel, Card Duel, arena teams/rounds/wipe/winner rules, fiesta lifecycle, powerups, augments and mid-cast cleanup.
- [ ] Port all ten scenario drivers/coverage assertions, including `card_duel`, and add disconnect, membership mutation, simultaneous wipe, invalid marker, duplicate-finish, pre-score-forfeit and deadline tests.

**Testing stage - M6 multiplayer-rule parity:**

- Run social/group/PvP/Card Duel module suites and all ten scenarios twice with exact trace comparison.
- Verify ordering is independent of hash iteration or connection arrival order after commands enter the canonical tick batch.

**Exit evidence:** Exact parity for all ten named scenarios and deterministic group/PvP/Card Duel lifecycle transitions.

## M7 - Pets, delves, dungeons and raid-instance parity

**Goal:** Port owned companion AI and instanced content covered by the final eleven scenarios.

**Dependencies:** M6.

**Scenario gate:** `pet_ai`, `pet_commands`, `hunter_pet`, `warlock_pet`, `delve_companion`, `delve_death`, `delve_lockpick`, `delve_lockpick_fail`, `delve_progression`, `dungeon_instances`, `dungeon_raid_lockout`.

**Progress evidence (not exit evidence):** Three dependency-independent state
owners cover all eleven named scenarios at the source-contract layer, with a
fixed gate-order matrix. Pet state owns tame/summon/replace, passive cleanup,
feed/revive/taunt, beast and demon stash/restore, aggressive acquisition,
melee/ranged cadence, owner combat linger and heel/teleport. It keeps despawn
retarget and attack draws on an explicit shared-combat counter. Delve state owns
multi-module progression, pressure-plate/portal advance, 50%-health first-death
recovery, second-death failure/ejection, rank-scaled companion combat/heal/heel,
the rank-3 once-per-run revive, Marks shop/upgrades/daily reset, and the 1/2/3
ante mapping to `3/2/1` pages, `1/2/3` tries and `60/120/180` tick deadlines.
Both lockpick success and terminal failure open the surface exit. Dungeon state
owns the exact 13-spawn Hollow Crypt claim draws, same-party claim reuse,
two-member leave and 300-second empty reset, plus raid/attunement/active-lockout
gate order for Nythraxis. These are focused source contracts only; M7 dynamic,
real-M2 and exact-golden acceptance remain 0/11.

**Implementation slices:**

- [ ] Implement pet ownership, summon/dismiss, command modes, target/assist behavior, threat, abilities, death and cleanup.
- [ ] Implement instance identity, membership, reset, encounter state, dungeon/raid lockout and deterministic transfer boundaries.
- [ ] Implement delve runs, companion progression, seeded lockpick board/visibility/path, success/failure, death and rewards.
- [ ] Port the eleven scenario drivers/coverage assertions and add sub-stream draw-order observers for pet/delve/lockpick RNG where the reference harness documents gaps.

**Testing stage - M7 final simulation parity wave:**

- Run pet/instance/delve module suites, the eleven exact scenario comparisons, then all 54 current-head scenarios as one double-run wave.
- Run the target parity and coverage suites unchanged as the reference baseline. A target or WOC red trace blocks promotion.

**Exit evidence:** All 54 current-head scenarios exact, deterministic and coverage-active; one authoritative ZrVM simulation implements every golden-covered subsystem.

## M8 - Eastbrook Vale desktop offline MVP

**Goal:** Close the first playable engine loop without reducing the final product scope.

**Dependencies:** M7 plus returned Runtime 04, Runtime 09, Runtime 10 and Plugins 08 WOC handoffs. The current-head HUD uses domain controllers/pure views/painters rather than a monolithic HUD port.

**Progress evidence (not exit evidence):** The first source-pinned Eastbrook
asset closure is materialized without a runtime dependency on `dev/`. A checked
selection manifest maps 93 target blobs into `assets/m8`, totaling 29,611,202
bytes. Its 26 GLBs retain the pinned asset-catalog identities plus 200 animation
clips and 54 glTF rig skins; the same closure includes all seven distinct player
models used by the nine-class picker and their 26 base/alternate skin textures,
plus wolf and boar, Vale foliage/village/environment/terrain/water inputs, MVP
ability/item icons, UI fonts and core quest/UI sound effects. Every row records
role, source path, output path, SHA-256, byte length and license id. Pinned
`CREDITS.md` and `LICENSE` copies travel with the closure, while the KayKit row
points to the official Adventurers and Character Animations CC0 sources. This
proves bytes and provenance only; no Zircon importer, scene, material, animation,
sound, retained UI or rendered frame is accepted by this evidence.

The first dependency-independent Eastbrook scene authoring contract is also
generated and byte-checked. It preserves the pinned player/NPC horizontal
positions and facing, representative wolf/boar camp centers, deterministic town
building/stall/well/bonfire formulas and the two fence runs. The generator reads
the checked GLBs, normalizes quantized accessor bounds, expands their glTF scene
graphs, and emits the current discriminated Zircon project-reference schema. The
result contains 268 entities, including 199 glTF node entities, five actors, 18
prop instances and 11 fence modules, plus a four-vertex flat MVP ground, camera,
ambient/sun and bonfire light. This is source/schema evidence, not a completed
implementation slice: `bootstrap.scene.toml` remains the default; original
terrain heights, navigation, materials and presentation wiring remain open; and
all ten referenced GLBs still require the Runtime 04 meshopt/quantization path,
with several also requiring WebP. No importer or rendered-frame pass is claimed.

The offline-world bootstrap now composes one source-pinned Eastbrook encounter
slice rather than starting with only the player and two quest NPCs. Its generator
creates a temporary archive from the exact reference commit, runs the target
`Sim` constructor at seed `20061` before any tick, and records four ordered
wolf/boar camps plus all 24 resulting entity snapshots: source entity order,
camp/member index, position/height, level, max HP, movement speed, facing and
wander timer. The WOS creates its own local IDs after the player/NPC rows, maps
the source mob identities through the M3 catalog, and persists the 24 hostile
rows through the existing snapshot codec; its self-test round-trips the 27-row
offline world and pins representative wolf/boar values. Keeping this small
orchestration in the oversized WOS is deliberate: the state codec and its
parallel mutable entity columns cannot yet cross the experimental ZrVM Array
ABI, while the independent source data lives in the generated encounter module.
WOS50 persists the source-null wander target and runs the retained offline
Eastbrook wolf/boar idle arm after player updates, using the ordered source
target-selection draws, post-constructor Mulberry32 cursor, point-collision
mobile transform and source arrival timing. The same composition now includes
the narrow initial aggro/social-pull, direct-profile white-melee
(warrior/rogue/paladin/shaman plus Bear/Cat druid and Hunter inside five yards),
deferred Hunter Auto Shot at 8-35 yards plus Mage/Priest/Warlock/Druid
wand at 0-30 yards, melee-pursuit, base enemy
melee-damage, source-shaped single-player kill XP, direct Forest Wolf `q_wolves`
credit and ordinary wild corpse/respawn branches. WOS51 additionally retains the one active-and-needed
wild-boar personal `boar_hide` slot and WOS52 retains source-generated copper
plus ordinary shared item slots through the same capacity-gated `loot` command;
WOS54 retains the source-order pending Hunter and caster-wand trajectory and landing
closure; unsupported-form auto profiles, on-hit effects, party/FFA
loot strategy, group rolls, party/elite/rested XP, general credit, non-wolf quest dispatch, pet/instance/boss
lifecycle and the shared dynamic transaction remain separate authoritative
mob-system work. This is a material world-composition slice, not playable-MVP
or dynamic-ZrVM acceptance evidence.

The project-local native adapter now has the dependency-independent half of the
client clock boundary. `PresentationTimeline<T>` accepts one bulk projection per
committed snapshot and exposes previous/current interpolation without knowing
entity or gameplay semantics. The default cadence is locked to 20/60 Hz and
three presentation subframes per authoritative step. Matching duplicates are
idempotent; conflicting digests, generation/tick regressions and non-monotonic
host receipt times are rejected; a new VM generation resets interpolation
history. Four focused integration tests cover those rules. This remains
source-reviewed only until the next managed Cargo run, and it is not the real
desktop host/render-loop connection required by the first M8 implementation
slice.

The timeline now has a concrete dependency-independent actor payload boundary.
`BulkPresentationProjection` carries one canonically ordered array for the whole
visible roster, using full `(id,generation)` identity plus target-derived
transform, animation inputs and appearance. Ingest validation rejects invalid
ordering, missing viewer identity, non-finite transform/motion values and
negative speed. Its render visitor performs an allocation-free ordered merge,
interpolates position and the shortest facing arc, holds new/reused-generation
actors at their current pose, and exposes current committed discrete state. Five
focused Rust tests are authored. This proves the host-side bulk/interpolation
shape only; ZrVM projection encoding, production host wiring, animation binding
and rendered-frame evidence remain open, and no Cargo pass is claimed yet.

`ClientPresentationProjection` schema v2 now composes that actor bulk with the
discrete HUD plus inventory/quest-window payload from the same committed 20 Hz
transaction. Raw player/target/target-of-target meters and cast state,
authority-computed action availability, dense inventory order with optional
manual cell hints, bag/currency state and complete acceptance-ordered quest
details cross the boundary together. Validation requires player/viewer identity,
actor membership for every unit, target before target-of-target,
finite/nonnegative numeric presentation inputs, canonical action identity,
derived bag capacity/four sockets, unique item instances and exact equality
between the HUD tracker and quest-log quest/objective state. The native layer
does not recompute resource, range, cooldown, inventory or quest rules; key
labels, localization, accessibility text and layout/collapse preferences remain
host-owned. Six HUD and six window-projection tests are authored. This is still
source evidence: no production ZrVM encoder, retained UI binding or Cargo/product
pass is claimed.

The client-side inventory and quest-log pure models consume only that immutable
projection plus a local presentation catalog. Inventory mirrors the target's
exact six categories and three stable sorts, case-insensitive trimmed search,
real-cell/manual-order gate, empty/no-match/overflow states and tolerant filter
preference JSON. Unknown local item definitions disappear from derived lists but
remain in the real-cell layout, while duplicate, missing or out-of-range legacy
cell hints deterministically occupy the first free cell without deleting a
stack. Quest selection remains painter-owned, preserves valid selection and
falls back to the first accepted quest when stale; objective completion and raw
reward/turn-in fields are projected rather than recomputed. Ten focused client
tests are authored and rustfmt-clean, without a managed Cargo claim.

`ClientWindowController` now supplies the DOM/engine-neutral interaction half of
those views. It owns only window visibility, persisted inventory filter state,
quest selection and the active target options subpanel. Static asset routes are
parsed before mutation; filter/search/sort return a persistence request, an
equipped bag socket returns a typed `UnequipBag` authority intent, quest share
returns the resolved id, and abandon returns a confirmation request rather than
calling authority early. Settings Back returns to the target options menu,
reset remains delegated to `StoredClientSettings`, and Bug Report cannot open
without the online capability gate. Six focused tests pin atomic unknown/missing
input rejection and prove the immutable ZrVM projection is not mutated.

The inventory interaction core now mirrors the target's mode priority exactly:
owner-bound transfer refusal precedes trade, mail, market, vendor, bank and pet
feed, followed by quest discard, bag equip and ordinary use. It also pins the
matching tooltip translation keys, vendor/bank shift-link exclusion,
transaction-mode destroy suppression, protected-item refusal and the
splittable-stack bank prompt. Immutable snapshot clicks re-resolve by dense
inventory index plus the complete projected row, so a stale count/replacement
cannot act on a different stack; partial deposit submission keeps the target's
same-item guard and count clamp. Five focused tests cover these branches without
calling native or ZrVM business logic.

The desktop client crate now owns the dependency-independent frame scheduler.
`ClientFrameDriver` consumes host-injected elapsed nanoseconds, advances its bulk
authority only at 50 ms boundaries and samples presentation on every render
frame. Commands remain queued across a failed step and clear only after a commit;
catch-up is bounded per frame while all excess authoritative ticks remain in the
accumulator. The driver overwrites projection receipt time with its own scheduled
boundary, so wall time cannot enter authority or be forged by the backend. Its
pending vector is allocated once and enforces the generated 4,096-command tick
limit before encode. Eight focused tests pin the 60:20 cadence, command
delivery/retry, queue bound, backlog, interpolation, atomic projection commit
and recovery shape. This is not
implementation-slice exit evidence until the real ZrVM adapter, engine loop and
managed Cargo/product gates pass.

The first atomic client transaction boundary is also authored.
`VmTickResult` carries authoritative world bytes and one bulk presentation payload
from the same backend call. `tick_with_projection` prepares and validates the
world candidate, decodes an explicitly versioned, structurally validated 16
MiB-bounded JSON client projection, and only then replaces committed state.
Unknown projection schema versions fail before actor/HUD validation. A runtime-computed
presentation digest is part of timeline identity beside state/event digests;
same-tick projection drift is a conflict, not an idempotent duplicate. The
`TransactionalClientAuthority` adapter feeds the resulting projection into the
frame driver, and a fake `WocProjectVm` integration exercises the full source
path, including invalid projection -> `Recovering` with tick and pending commands
unchanged. JSON is the first MVP bulk codec, not a final full-roster
wire-performance claim. Production ZrVM byte output and engine/product gates
remain open.

The validated transactional driver also exposes `visit_presented_actors`, an
allocation-free host seam that applies timeline alpha only to actor transforms
and returns the current committed HUD by reference. A two-tick fixture pins the
alpha-zero boundary to previous pose `x=1` while the discrete HUD is already
`Hero 2`. This closes the dependency-independent host sampling shape, not the
real renderer or retained UI binding.

The canonical command-payload manifest is now schema v11 with twenty-nine generated
rows against the current 165-command catalog. In addition to the original
cast-slot/target/tab/attack contracts it pins
the target's `castAt {ability,x,z}` at id 1, `cast {ability}` at id 2,
`cancel_aura {aura}` at id 3, empty `interact {}` at id 11 and
`accept {quest,selection?}` at id 16 and `turnin {quest}` at id 17,
`abandon {quest}`, `use {item}`,
`discard {item,count?}`,
`equip_bag {item,socket?}`, `unequip_bag {socket}` and bounded lockpick trio:
`lockpick_engage {objectId,ante}`, `lockpick_action {sid?,action}` and
`lockpick_abort {sid?}` at ids 18/23/24/126/127/120/121/122, it now pins
`applyTalents {alloc}` at id 95 as `u16_le_spec_code+6*u16_le_row_option_code`,
`respec {}` at id 96 as a generated empty payload and
`setSpec {spec?}` at id 97 as a generated class-scoped specialization code
(`u16_le_spec_code`, with zero for source `spec:null`) and
`switchLoadout {index}` and `deleteLoadout {index}` at ids 99/100 as fixed
`u32_le` indices, plus
`change_skin {skin,catalog}` at id 31 as fixed `u8_catalog+u8_skin_index`
(`class=0`, `mech=1`; class values are bounded to `0..7`), plus
`selectTalentRow {level,optionId?}` at id 163 as a generated catalog option
code (`u8_row_level+u16_le_option_code`, with zero for source `optionId:null`)
and `resurrect_respond {accept}` at id 164 as `u8_false_or_true`.
Identifier payloads
use the protocol's canonical `u32_le` UTF-8 byte-length prefix with a 256-byte
transport bound; optional integers use an explicit presence byte plus `u32_le`,
never a sentinel. Ground casts append finite `f64_le` x/z coordinates after the
ability identifier and normalize `-0` to the source JSON wire's `0`. Generated
Rust and ZrVM metadata carry fixed or min/max
lengths and schema SHA-256
`1889a9adf787ab59f60115ae82dbabecad4aac4ab6549de9f1eb2ef676a1f5ad`.
The active Zr/native command directory and v12 typed subset are generated from
the current catalog at `5ef9f7cb21cd8875b6d2c49701015dfcd78de35a`, including
`change_skin` at id 31, `applyTalents` at id 95, `respec` at id 96, `setSpec` at id 97,
`switchLoadout` at id 99, `deleteLoadout` at id 100, `selectTalentRow` at id 163
and `resurrect_respond` at id 164. The remaining 127 client-send and eight
dispatch-only rows are source-shape inventory only
until each has a canonical bounded encoding and reducer owner. The generic
command envelope can carry bounded opaque bytes for known ids; this transport
fact is not typed-payload validation or gameplay support.
Focused Rust payload tests cover exact vectors, absent/present optionals,
UTF-8, truncation, trailing bytes, range bounds and stable catalog ids. The
WOS8 offline-world reducer now applies typed `accept`, `abandon` and `turnin`
rows for its source-pinned `q_wolves`/`q_boars` ledger. Well-formed unavailable
quest ids leave state unchanged rather than rolling back their enclosing Tick;
target error projection remains later UI/event work. The reducer checks the
target's seven-yard quest-NPC tolerance. Source-pinned `q_wolves` kill and
`boar_hide` post-loot credit hooks complete their eight-kill/five-collect
objectives and grant the generated 75/120 copper and 250/350 XP rewards. The
generic combat-death and collect-inventory integration that calls those hooks,
`interact` dialogue, quest resync, daily reward and account-complete services
remain open, and no authoritative gameplay or Cargo pass is inferred from this
codec slice.

The WOS13 offline reducer consumes the typed `applyTalents`, `respec`,
`setSpec` and `selectTalentRow` commands and applies source-order class-spec or
row/level/class-option checks before its current
reachable combat lock, then delegates the full six-row allocation to the shared
atomic replacement path. Its generated option code is a transport surrogate for
the source string id, with zero preserving clear. The offline MVP locks when a
live hostile mob is in the active AI state and targets the player; source arena
membership and the five-second post-combat linger are not yet represented, so
this remains an explicitly bounded offline slice rather than a claim of full
talent-state parity.

`switchLoadout` and `deleteLoadout` now have an exact generated `u32_le` wire
shape and native/client validation that rejects indices outside the source's ten
loadout slots. WOS13 persists a bounded offline `SavedLoadout` projection: raw
name bytes, a specialization and six row codes, 22 action-bar ability codes, and
the active index. Schemas WOS2-WOS10 migrate to an empty list. The reducer uses
the source's combat lock, missing-index no-op, active-delete fallback and
automatic allocation replacement. It deliberately does not invent `saveLoadout`:
that source command remains untyped, and the stored action-bar codes are not yet
bound to a live action-bar runtime.

`change_skin` now has an exact generated two-byte wire shape and native/client
validation. The source class path normalizes the skin index to `0..7`, so WOS13
persists a valid class selection in its existing offline-player identity field
and preserves it through encode/decode. Mech chromas remain account-owned in the
source; without an account cosmetic service, a well-formed mech request consumes
its command sequence but intentionally leaves the WOS13 class skin unchanged.

`resurrect_respond` now has its exact one-byte boolean wire shape and is emitted
by the platform-neutral client mapper. WOS13 persists one proposal per dead
player with the source caster id, hp fraction, offer-time fallback position and
absolute 30-second expiry. The response consumes a proposal before decline,
expiry and dead checks, then restores `Math.round(maxHp * hpFrac)` at the living
caster location or stored fallback; the state-level regression covers no-offer,
decline, fallback, expiry and WOS round-trip. The current offline effect
dispatcher remains unable to create a proposal, and resources, ghost/corpse,
auras and events remain M4 work, so this advances the durable state loop without
claiming full combat-resurrection parity.

**WOS13 overworld spirit-loop extension (current-source slice):** Payload schema
v13 introduced generated empty payloads for `release` (id 35),
`resurrect_corpse` (id 135), and `resurrect_healer` (id 136), initially raising
the typed command total to 33/165. The current payload schema v15 additionally
preserves the source `castAbilityOn` optional target id as
`u32_le_utf8+u8_presence+u64_le_target` and the nine party commands, raising
the checked total to 42/165; its generated fingerprint is
`166415fb09b91c4eac976ac5e100fe5674d358136b0a4bf57c3dec6a21293d85`.
The current payload schema v16 adds the source-empty `stow_weapon` command (id
162) with a generated Zr/Rust payload descriptor, raising the typed total to
48/165; its generated fingerprint is
`5be262ca0d7178a85e1b24173a97d2a627a32a79b4b57d0a525c7145da858fc4`.
Current payload schema v17 adds the remaining nine current-source `IWorldPet`
wire shapes: abandon/revive/attack/taunt/heal empty commands, rename/feed/mode
bounded UTF-8 identifiers, and the auto-taunt boolean. The generated Zr/Rust
descriptors and platform-neutral client mapper now cover 59/165 commands
(58 client-send and one dispatch-only) with fingerprint
`5237597dc7bd91b9d7ec0a21a5162a94c67bbc622a498cba6a0f6c883991d712`.
This is transport completion only: WOS38 does not silently route these commands
to the separate mutable-array pet model until Plugins 08 can prove the required
cross-module state ABI, so pet reducer ownership remains explicit.

Payload schema v18 added 17 current-source `IWorldSocialGraph` transport
shapes: friend/block/ignore and guild-member names, guild invitation/leave/
disband empty commands, and `guild_event_remove {id}` as a strict little-endian
`u32`. Generated Zr and Rust descriptors plus the platform-neutral client mapper
now cover 76/165 commands (75 client-send and one dispatch-only), with fingerprint
`0930ad2815aaa15cdeaedec684cb4880cdd0f0db017cb486fbbee77a24660cc5`.
This is intentionally transport-only: source social actions are account-scoped
persistence services and WOS38 does not fabricate a social aggregate, snapshot
route, or durable reducer.

Current payload schema v19 adds the final current-source `IWorldSocialGraph`
shape, `guild_event_create {day,hour,title,note}` (id 132). Its canonical layout
preserves source order with separate length-prefixed day/title/note fields, a
nullable finite `f64` hour, and bounds of 10/192/640 UTF-8 bytes. The title and
note bounds retain the full source-normalized 48/160 UTF-16-code-unit prefixes.
Generated Zr/Rust descriptors and the platform-neutral client mapper now cover
77/165 commands (76 client-send and one dispatch-only), with fingerprint
`5d72df22581d1dc80169f7fc56db78dd10a414f93321eab6722314f526e39bf0`.
The payload rejects malformed presence markers, non-finite hours, overlong
fields and trailing bytes; it does not implement the source guild calendar
persistence, permission checks, snapshot updates or moderation path.
Payload schema v20 adds four source-pinned `IWorldParty` transport shapes:
`setLootMaster {enabled,looter,threshold}` at id 48, `setMarker {id,marker}` at
id 50, `clearMarker {id}` at id 51 and `readyrespond {ready}` at id 52. The
canonical layouts are respectively `u8+f64+u8`, `f64+f64`, `f64` and `u8`;
all JSON-number fields reject non-finite values and the master-loot threshold is
closed to `uncommon|rare|epic`. Generated Zr/Rust descriptors and the
platform-neutral client mapper now cover 81/165 commands (80 client-send and
one dispatch-only), with fingerprint
`b7d7e746e5447e6d283233e7cdba31a212263711df492add2fef72e02ab41d29`.
This is transport-only for loot-master and marker commands: WOS38 does route the
existing `readyrespond` boolean into the persisted active ready-check reducer, but
does not claim the source `/ready` UTF-8 route, prompt/result event delivery,
marker-range or loot policy. `masterAssign` remains source-only because its arbitrary
nonempty numeric array has no source-owned transport bound.
`WorldState` routes its implemented commands through the existing atomic command
batch and persists WOS14 ghost/corpse coordinates plus resurrection-sickness microseconds.
The reducer reproduces the source overworld branch: first-entry-tie nearest
selection across seven source-pinned graveyards, a 35-yard corpse gate with 50%
health revival, an 8-yard healer gate with 20% health revival, and the source
level-derived sickness duration. Release, corpse and healer state round-trips
are covered by a focused WOS state test. This remains intentionally bounded:
arena/delve routing, living resource restoration, aura/event lifecycle and the
combat effect producer for `resurrect_respond` still require their owning M4/M7
systems and the Plugins 08 mutable cross-module ABI.

The current-head Temporal Reversal contract is separately regenerated from the
pinned `classes.ts` and `effect_dispatch.ts` blobs. It fixes the ability's level
16, 60-cost, two-second cast, 600-second cooldown, 30-yard friendly dead-target
shape, `resurrectAlly { hpFrac: 0.35 }`, and `temporalGlyph` spell-fx branch.
WOS15 consumes the generated contract at the actual `cast` boundary: it checks
known ability, 60 resource, GCD/cooldown, source party/raid dead-target selection
and start range, locks the target for the two-second cast, then rechecks target
membership/death before offering resurrection, billing resource and arming the
600-second cooldown. Event projection and generic resolved-ability effects remain
explicit M4 work rather than hidden direct world mutation.

The current group-targeting projection now also carries Temporal Reversal's
pure target selector: explicit mouseover overrides current target, a missing or
invalid override never falls back, and only a dead player retained in the
caster's party/raid is selected. WOS15 now owns the durable party id, leader,
raid/subgroup and join-order projection required for that membership test and
implements the same scalar selection in its state-local reducer. The existing
cross-module target-selector import remains blocked by the Plugins 08 mutable
Array ABI, so this does not prove the preferred modular handoff.

The isolated M6 `social/party_raid_state.zr` projection is now rebased to
current-head `PartyMachine`: it models the ten-member cap, 30-second invites,
accept/decline, leader-only kick/promote, party-to-raid and bounded
raid-to-party transitions, plus subgroup normalization. WOS15 now mirrors that
state inside `world/state`: it stores pending party invitations, stable join
order, party leaders and two raid subgroups, and reduces the real `pinvite`
through `punraid` command family in the transactional candidate. This is still
source-authored/static-checked only; source trade/duel invite exclusion, social
events, loot/ready-check/finder state, the target-selector ABI handoff, generic
ability resolution, source haste/GCD modifiers, and all non-Temporal combat
effects remain explicit work.

The first platform-neutral gameplay input boundary is also source-complete.
`ClientCommandMapper` accepts keyboard/mouse, gamepad, touch and confirmed window
intents through one type, then emits only generated, typed command payloads with
one actor identity and non-wrapping sequence. Ground-target, ability-id and
cast-slot, aura-cancel, empty interact, quest accept, set/clear target,
hostile/friendly tab, attack edges, catalog-backed full talent allocation, respec,
spec, row select/clear, class/mech skin selection and bounded loadout switch/delete indices and
all five inventory/quest commands above
are covered; invalid ids and sequence exhaustion do not consume sequence. Every
mapped row must resolve to `ClientSend`, so the dispatch-only `targetNearest` id
cannot leak through a client host. Focused tests prove cross-device command
equality, typed field round trips and failure rules. The equipped-bag retained
route now returns the same `ClientGameplayIntent::UnequipBag` consumed by that
mapper rather than a second window-only command type. Movement remains a
separate planned stream. The retained touch HUD route
`woc.hud.touch.interact` now queues that typed empty command for the next 20 Hz
commit; it does not claim a ZrVM `sim.interact` reducer, engine input binding,
focus, accessibility or product-flow acceptance. Likewise, the typed `accept`
and `turnin` intents deliberately do not mirror the target's
`pendingQuestCommands` UI state and have no NPC-dialogue route; WOS8 owns the
limited session-only offline task-world reducer. `turnin` also has no quest
resync, daily-reward service or
account-completion special-case implementation.

Read-only source audit also identifies the next WOC-owned protocol requirement.
The target streams movement every 50 ms outside the command channel: a positive
sequence, forward/back/turn-left/turn-right/strafe-left/strafe-right/jump flags
and optional finite facing. Server authority applies each valid frame, advances
acknowledgement to the maximum sequence and clears held movement after 750 ms of
silence.
`FixedTickInput` protocol v3 carries a bounded movement-frame batch after
generation: actor identity, non-wrapping sequence, those seven flags and
explicit `has_facing + f64 facing`, with canonical per-actor ordering, duplicate
rejection and the target stale-clear boundary. The fingerprinted projection is
mirrored in Rust and ZrVM, and the Rust relay pins applied-frame acknowledgement
and facing retention. `WorldState` now consumes a nonempty canonical batch
inside its WOS8 candidate transaction and routes it through the source-ordered
movement transition. Movement is not a command, and no M8 gameplay or
engine-bridge acceptance is claimed until a managed Cargo/ZrVM gate validates it.

The first dependency-independent client-shell state is also authored.
`CharacterRosterModel` consumes authority/persistence summaries for identity,
class, level, appearance, online/forced-rename status and host-injected recent
and playtime values. It implements the target's four roster sorts, stable
name/id ties, selection preservation across sort/refresh, first-row fallback and
empty-roster transition to creation. Its primary action keeps the pinned order:
forced rename disables entry before online takeover, then ordinary enter-world.
Roster replacement validates every candidate and duplicate id before atomic
replacement. The creation-name shape normalizer mirrors target edge trimming
without rewriting internal spaces and its 2-16 ASCII-character rule;
offensive-name, duplicate-name and
realm-uniqueness gates remain authoritative. Six focused tests cover the state
and name shape. `OnlineCharacterFlow` now owns the pure shell transition/effect
boundary around it: select/create Back behavior, persisted four-mode sort,
ordinary entry, two-step takeover confirmation, class-catalog creation,
forced-rename submission and offline-only typed-name deletion. It emits typed
effects instead of calling the API or moving any realm rule into Rust, and it
drops takeover/delete confirmations when a roster refresh removes their
character. Every authority refresh reselects the first sorted row; create Back
retains its draft, while successful creation clears only the name and preserves
class/skin. Eight additional focused tests bring `woc_client` to 220 authored
tests. No Cargo execution is inferred from this source slice.
This remains a DOM/i18n/network-free state and effect layer; profile storage,
API execution, live retained binding, 3D preview and real enter/takeover flows
remain open.

`ModeSelectionModel` now pins the landing console before either character path.
Online is the default; opening the menu activates the current choice; two-option
keyboard navigation clamps at both ends; closing does not commit; and only Play
emits the chosen online-flow or offline-picker effect. Online-only hosts reject
Offline explicitly, while restored-session versus login routing remains a host
decision. Five focused tests cover that boundary.

The target's post-login world list is now mirrored by `RealmDirectoryModel`.
Directory replacement is atomic, preserves source order and exact realm-type
label keys, exposes character counts and auto-selects only an exact remembered
`woc_last_realm` name. Live status uses the pinned offline/full/high/medium/low
edges (`cap`, 80 and 15), and the completed status batch recommends the first
lowest-population online world with source-order ties. Selection emits the exact
name/base URL as a host effect rather than mutating transport or storage in the
model. Six focused tests cover that boundary; real directory/status calls and
platform preference storage remain M9 work.

The target offline path has now been audited separately from online character
persistence. It creates a new simulation from fixed seed `20061` after the player
chooses one of the nine source-ordered classes, a name and a skin. The target
explicitly states that offline characters are not persisted; only preferences
such as keybinds use the stable `offline:<class>:<name>` scope. The new
`OfflineSessionDraft` preserves those rules: Warrior/skin zero defaults, skin
reset on every class-card activation, exact class ids, target name validation, fixed seed
and a preference scope. It deliberately exposes no save-state or resume path.
Launch also rejects a skin index outside the selected class catalog without
mutating the draft. It also resolves the current class/skin to a complete preview
request without requiring a name. Seven focused tests are authored. This corrects the earlier M8 plan assumption
that offline gameplay should survive restart; durable authoritative character
persistence belongs to M9 online realms.

The client-side class presentation catalog now covers the same nine classes in
picker order without copying authoritative combat numbers. It pins the target
role/armor/weapons localization keys, role types, class colors, renderer visual
keys, skin counts and exactly three curated signature ability ids per class.
Its companion appearance catalog binds those nine rows to all seven materialized
player GLBs and the exact base/alternate thumbnail sequence, including the shared
mage set for Priest/Mage/Warlock. Preview resolution keeps the target distinction:
skin zero uses `base.png` only as its swatch and retains the GLB's embedded material,
while later variants apply their alt atlas. Eight focused tests cover order, signature
triples, assets, lookup identity and every valid/invalid preview index; a source comparison reports
`target=9 rust=9 equal=True`, and every one of the 33 referenced asset paths is
present in the checked M8 manifest. Live class stats and ability details must
come from the ZrVM content projection, while retained preview wiring remains open.

The offline Shell now composes that draft into the target product-entry order.
`OfflineShellController` starts at mode selection, opens an editable offline
picker, prepares one fixed-seed session, then advances through Welcome, Loading
and InWorld. Back clears a typed name, and reopening the picker restores
Warrior/skin zero; picker edits outside that state are rejected; invalid submission
or a catalog-external skin leaves both draft and screen unchanged; and Continue/world-ready commits are
one-shot. The exact launch identity and preference scope survive every forward
transition. Eight focused tests are authored. This is still a pure state
contract, not retained painting or a real runtime scene transition.

The first authoritative offline tick now has a versioned, bounded
`OfflineSessionBootstrap` envelope rather than an invented command. It carries
the source standard seed `20061`, canonical source-content class index, normalized
name bytes and selected skin. `WocTransactionalRuntime` retains that envelope
through a failed candidate and consumes it only after a successful Tick 1;
subsequent ticks encode an empty field. The ZrVM decoder independently checks
the standard-session contract, initializes the first player from the generated
M8 content/stat tables, persists the selected identity and its initial source
known list in WOS9, persists the WOS10 all-zero allocation and WOS11's empty
loadout list, and still decodes WOS2-WOS11. The bootstrap and `change_skin` protocol both use the source class
catalog's `0..7` range; materialized preview skin counts do not constrain saved
offline identity. This is a protocol and source-contract connection only:
the open Plugins 08 byte/class ABI still blocks dynamic ZrVM proof, and the
open Runtime 09/10 bridges still block live UI-to-scene execution.

Runtime inspection confirms the existing open Runtime 09 failure
`woc-project-runtime-ui-bridge` still owns project-authored retained UI: the
dynamic session only recognizes a hardcoded single-button Start/Game Over menu
and text HUD. A separate Runtime 10 failure,
`project-script-scene-transition-host-request`, now records that project scripts
and live UI cannot request an atomic `res://` scene change. WOC therefore keeps
`bootstrap.scene.toml` as the default rather than bypassing the picker/Welcome
flow by directly loading Eastbrook; both product integration gates remain open.

The target's post-selection Welcome Screen pure view model is now mirrored by
`woc_client` without DOM, network or localization ownership. Its gating matrix
shows Armory and daily-reward tiles only on online desktop web, keeps offline
Continue ready without a connection, preserves the Discord strip's explicit
disabled/linked-member/fetch-failure behavior, and selects touch hints for native
or touch hosts. News retains full caller release records while marking at most
five rows NEW, and the last-seen id advances monotonically. The Armory handoff is
an exact one-shot session-storage intent that clears on first successful consume.
Eight focused tests are authored. This is source-only state: live retained
painting, focus trap, async reads, desktop character stage and localized output
remain open.

The per-character keybind core is also source-authored. It pins canonical
`Ctrl+Alt+Shift+Meta+code` order, exact combo parsing, physical modifier codes,
Escape reservation across modifier layers, and the target distinction between
edge chords and held movement keys. Full and compact keycap labels match the
source, while storage identities remain `woc_keybinds`,
`woc_keybinds:char:<id>` and `woc_keybinds:offline:<class>:<name>`. The complete
61-action source registry now covers all five categories, five Pet actions and
the contiguous 23-slot action bars. `Keybinds` owns both slots, exact/default
dispatch, cross-category conflict eviction, held-key modifier stripping,
edge-chord preservation, Escape rejection, clear/reset, and Attack Move's sole
shared-key exemption. The pure stored-profile loader now preserves explicit
unbinds, rejects malformed/reserved slots, resolves duplicate claims in registry
order, retains non-conflicting defaults and applies the two exact target repair
signatures before loading. JSON decoding, legacy-blob seed selection and the
platform-neutral storage owner are also source-authored: valid scoped objects
win, missing/corrupt/non-object values seed from the legacy key, successful
mutations serialize all 61 actions, and storage errors degrade without discarding
in-memory changes. The host-neutral options model emits the exact category/action
order and two-slot labels, conditionally exposes Attack Move, and owns one-shot
capture, modifier waiting, Escape cancellation, normalized confirmation, reset
and panel-leave state. Fifty-one focused tests are authored. Concrete
browser/native/mobile storage adapters plus retained painting, focus and
localization remain open.

The complete client-settings preference core is now source-authored under the
top-level `preferences` owner. Its registry matches all 43 target numeric ranges
and 41 boolean defaults in source order, including graphics first-run marking,
controller/touch tuning, audio, interface, accessibility, party frames and HUD
comfort settings. `ClientSettings` owns defaulting, finite-value clamping,
independent snapshots, reset and click-to-move button normalization;
`StoredClientSettings` mirrors `woc_settings` JSON type filtering, complete
84-field saves, corrupt/missing fallback and unavailable-storage degradation.
Eighteen focused tests are authored, and an extraction guard reports numeric
`43/43 diff=0` plus boolean `41/41 diff=0` against the target source. Retained
painting, live subsystem application and concrete platform storage remain open.

The matching host-neutral options projection is now source-authored as well.
It emits the exact Esc menu routes and the target Graphics, Audio, Controller
and Interface control trees, with current values, registry ranges, slider step
and format, choice labels, rerender flags and `uiScale` release-only commit. Its
native-shell gate removes Ultra/Advanced and Interface Mode; touch gates reveal
the source-ordered mobile rows. The full Interface sequence matches all 41
setting rows, deliberately retaining the target's repeated Attack Button row;
Audio and Controller match `7/7` and `5/5`. Nine focused tests are authored.
An exhaustive application-routing catalog also covers all 43 numeric and 41
boolean settings in registry order. Empty routes explicitly mean the target only
persists or reads the value later; nonempty routes use typed input, audio,
renderer, gamepad, touch, HUD, style and platform actions, retaining target
multi-owner order for SFX, UI scale, mobile camera joystick and reduced motion.
Stored mutations now return the normalized committed value with that static
route, and startup generates all 84 applications in target order. Nine route and
commit tests cover that boundary. The first-run graphics policy also mirrors the
target GPU family ladder without consulting FPS: only recognized software/weak
GPUs auto-Low, unknown/mid devices use Medium, evidenced desktops can reach
High/Ultra, touch devices cap at High and native startup clamps saved
Ultra/Advanced to High. Conclusive choices are marked once; inconclusive Medium
remains unmarked for a future probe. The four exact runtime budgets all preserve
60 Hz presentation; mobile changes only the render-scale floor, and the automatic
governor remains enabled below Ultra. Thirteen graphics-policy/storage/budget tests
bring the settings total to 49. This closes the pure controls, application-plan,
startup-policy and budget projections, not retained painting, localization,
focus/input routing, device-hint collection, governor/route execution or concrete
platform storage.

The target's host-agnostic gamepad math is now mirrored beside the command
mapper. Radial deadzone removal rescales the surviving vector and clamps square
corners to unit magnitude; the left stick keeps the source's strict movement
deadzone boundary and 85% per-axis threshold; the right stick emits injected-
frame-time yaw/pitch deltas with optional Y inversion. Button snapshots produce
only up-to-down edges, including a newly observed button when the previous array
is shorter. Eight focused tests are authored. These values are input intent only:
they do not invent a movement command or bypass the planned authoritative 20 Hz
movement payload/ack/stale-clear contract.

The matching touch-input core now reuses the same directional flag shape while
pinning mobile-only target behavior: default joystick deadzone `0.22`, custom
deadzone override, autorun reveal/lock bands `1.45/2.05`, Auto/Desktop/Touch
interface override precedence, 420 ms chat long-press, 300 ms stationary double-
tap recenter, 0.8 analog camera scale and 12 px/0.035 pinch-zoom filtering.
Eight focused tests are authored. Device detection, pointer ownership, haptics,
window-open cancellation and retained mobile controls remain host integration
work; the resulting movement flags remain local intent until the protocol gate.

The W3C Standard Gamepad layout is now explicit as well. All 16 non-Guide
buttons have one default action, action slots 0-8 occur exactly once, and the
target jump/interact/target/menu/map/autorun/friendly-target defaults are pinned.
Controller identification gives product names precedence, then reads only the
Chrome or Firefox vendor field so a colliding product id cannot misclassify the
pad. Xbox, PlayStation, Nintendo and generic labels follow each physical button,
including Nintendo's swapped face glyphs and shared D-pad arrows. Six focused
tests are authored. Polling/focus, remapping persistence and retained controller
options remain host work.

`GamepadBindings` now owns the matching mutable, storage-codec-independent
layout state. It starts from all 16 defaults, accepts stored overrides only for
bindable indices, deliberately permits multiple buttons to target one action,
uses `none` to clear a button, ignores Guide/out-of-range writes, restores the
full default map on reset and exports panel rows in W3C button order.
`StoredGamepadBindings` shares the top-level `PreferenceStorage` boundary and
pins the independent `woc_gamepad` JSON object/array loader, JavaScript numeric
property coercion, complete-map save/reset, duplicate actions, target-compatible
clear/reload behavior and unavailable-storage degradation. The controller model
emits the exact 55-option action catalog, 16 W3C-ordered remap rows and
brand-specific physical labels through one mutable/persistent binding contract.
Seventeen binding, storage and controller tests are authored. Concrete platform
storage, polling/focus, settings controls and retained painting remain host work.

Project-authored retained UI source now covers in-app authentication/password
recovery, mode/realm selection, online character select/create, the session-only
offline picker and post-selection Welcome screen. The eight schema-v2 views plus
their shared theme preserve the target's username/password/email caps, signup-only
email field, challenge-gated 2FA input, enumeration-safe reset-request status,
token-gated new-password view, Online-first two-stage Play console, exact
contract-address copy route, dynamic realm list loading/empty/error anchors, four
roster sorts, dynamic character rows, first-row preview/details, explicit
takeover and typed-name delete confirmations, nine-class create order and
class-catalog-owned four skin sockets. The existing offline and Welcome views
retain their full/compact preview and gated news/Discord/Armory/chest behavior.
The four composite views switch at the engine's `md` breakpoint, keep their
scroll owner explicit and expose at least 40 px touch targets for the new online
actions; the realm view uses one centered scrollable panel at every size, while
the mode and auth consoles scroll on short viewports. Their 257-node graph
exposes 221 unique control ids and 97 host events through one shared WOC shell
theme; a structured
`tomllib` checker rejects TOML, graph, order, route, duplicate-id, capability
gate, modal-default and style-token drift. Turnstile, Apple and Discord provider
hosts stay collapsed until their capability adapters exist; authentication host
responses remain M9 work rather than a client-side account implementation. Every
root imports the pinned Vale WebP as an optional nonblank backdrop until its live
3D host attaches. The combined asset digest is
`5b95b0ad2bf717c790ce75ec554b2ee4b45cdf01b86c18dbfa8049cb551a640b`.

`AuthFlow` is the matching pure state boundary. It trims login usernames without
rewriting passwords, applies the target's 24/128/254/14 UI bounds, requires a
nonempty shape-valid signup email but leaves account existence, password policy,
name moderation, rate limits and 2FA verification to the service. A
`twoFactorRequired` response reveals the follow-up field, whitespace-normalized
six-digit codes route as TOTP and any other trimmed value routes as a recovery
code. Password-reset requests collapse opaque failures into the same sent status
and expose only a host-classified rate limit. Reset tokens and secret fields are
in-memory only and are cleared after completion or reset Back. Eight focused
tests are authored; live transport, Turnstile/native attestation, token parsing,
session persistence, localization and retained-host input/focus wiring remain
host work.

`OnlineShellController` is the client-leaf composition point for the accepted
online path: host-provided restored-session state goes straight to realm loading,
otherwise it activates login; a 2FA challenge remains on authentication, an
accepted authentication requests the realm directory, and remembered/manual
realm selection requests characters only after the host switches base URL.
It preserves target Back order (realm -> mode selection, roster -> login,
create -> roster) and forwards credential and character effects without
inspecting or persisting them. Six focused tests cover those transitions. The
controller is not a network/session implementation: API calls, token restore,
realm/character fetches, world entry and Runtime 09 retained-view binding remain
host work.

`WocShellController` is the root client-leaf router. It retains the selected
Online/Offline mode, emits a host session probe only for Online Play, maps that
host result into `OnlineShellController`, and opens the fresh session-only
offline picker directly for Offline Play. Realm Back and authentication Back
return the active root branch to mode selection rather than leaving a stale
child controller visible. Offline picker submit emits only a prepared launch;
Welcome Continue emits the single start-world effect, and host readiness emits
the final in-world effect, so no pre-Welcome action can create a second fresh
world. Seven focused tests cover root transitions, online-only rejection and the
offline sequence. It has no API, persistence, VM or renderer dependency; Runtime
09 remains responsible for dispatching retained routes to this reducer.

The first project-authored in-world retained HUD is also source-complete. Its
schema-v2 view and theme add the target's desktop player/target/target-of-target
frames, health/absorb/resource/cast/XP surfaces, minimap, quest tracker,
12-slot primary plus 11-slot secondary action bars, low-health layer and exact
nine-entry Esc menu. The compact layer keeps five actions over two source-pinned
pages for source slots 1 through 10,
separate attack/target/interact/jump controls, six carried-consumable slots,
movement/camera zones and touch-sized top actions. Its 137-node graph exposes
136 unique control ids and 54 host events; the structured checker pins event
namespace, layer order, breakpoint gates, target ownership and every slot route.
The view/theme digest is
`9f5a20463729181869923e03b5c5a11fb377934ab89a113a220b766b658dc772`.
Only the four currently supported ZUI event kinds are authored; target drag,
right-click clearing and long-press behavior remain explicit Runtime 09 input
bridge work rather than invented asset events.

Three additional schema-v2 retained views now cover inventory, quest log and
settings. Inventory fixes the exact `all/weapon/armor/consumable/material/quest`
category order, `recent/quality/name` sorts, search, backpack plus four sockets,
dynamic real-cell grid, empty/no-match state and money/capacity anchors. Quest
log has an `md+` list/detail split and an `xs/sm` list-before-detail stack with
separate dynamic list/objective/reward hosts plus share/confirmed-abandon route
anchors. Settings uses the target's main-menu-to-subpanel navigation rather than
inventing tabs: one scrollable host declares the exact Key Bindings/Controller/
Graphics/Interface/Audio/Performance/Bug Report panel discriminator set, with
Back/reset/close routes; the main HUD menu keeps Bug Report online-gated and the
existing 84-setting model remains the data owner. The
three graphs contain 74 unique controls and 24 events, every interactive node
meets the 40 px touch floor, and the structured digest is
`ad3c146c079c5e92ed2b4d8220137b0726b2af0dde0c5aab199d1487270c29cf`.
The unified M8 checker now validates all fifteen project UI assets beside the 93
materialized blobs and 268-entity scene.

These retained assets are source closure only: Runtime 09's open project-runtime
UI bridge still prevents live loading, dynamic projection, input/focus/
accessibility routing and rendered acceptance, so the retained-client slice and
M8 exit evidence remain unchecked.

**Implementation slices:**

- [ ] Build the desktop client role around the shared tick transaction and 60 Hz snapshot interpolation/prediction without per-entity VM presentation calls.
- [ ] Import the original Eastbrook Vale zone, player/class subset, mobs, NPCs, props, animations, audio, fonts and textures from target bytes with source/license inventory links.
- [ ] Author Zircon scene, camera, lighting, navigation, material and animation bindings that preserve target coordinates, gameplay bounds and readable visual identity.
- [ ] Implement retained login/online character selection plus the session-only offline picker, HUD, action bar, target/unit frames, inventory, quest log, settings and pause screens with input, focus, accessibility and touch-aware layout contracts.
- [ ] Connect movement, combat, one kill quest, one collect quest, loot/equip/vendor and death/respawn to the already-parity-accepted ZrVM systems; keep offline world state session-only while persisting target-compatible settings/keybind preferences.
- [ ] Add product fixtures for fresh offline sessions, online profile/resume, keyboard/mouse/gamepad/touch flows, resize/DPI, accessibility tree and nonblank rendered frames.

**Testing stage - M8 playable MVP:**

- Run project/package/client checks, focused runtime plugin/UI/asset integration suites and all 54 current-head simulation traces.
- Launch the real desktop client, complete login/character selection and the Eastbrook Vale loop, capture 1280x720 and 640x520 evidence, verify input/accessibility, and require a stable 60 Hz presentation diagnostic on the acceptance scene.

**Exit evidence:** A real playable desktop offline client using original target assets and live retained UI; every launch creates a fresh target-compatible offline world while settings/keybind preferences survive restart; no engine/game special case or fallback asset path.

## M9 - Authoritative realms, networking and PostgreSQL persistence

**Goal:** Reproduce the target multiplayer authority and durable service loop using the same ZrVM simulation.

**Dependencies:** M7, returned Runtime 10 and Plugins 08 handoffs; Runtime 09/04 are not required for headless server work.

**Implementation slices:**

- [ ] Implement the complete versioned 165-command wire catalog, deterministic command-result/event DTOs for target interaction outcomes, authentication/session envelopes, authoritative input queue, snapshot/delta stream, reconnect/resume and protocol rejection responses.
- [ ] Implement `woc_server` fixed 50 ms scheduler, realm/world/session ownership, faulted-world supervisor restart, backpressure, controlled shutdown and deterministic/manual stepping.
- [ ] Implement PostgreSQL migrations and repositories for accounts, characters, inventories, quests, social state, mail, bank, market, guild/party durable metadata, lockouts and audit records.
- [ ] Commit persistence only from accepted ticks, add idempotency keys and make database/external failures unable to mutate committed simulation state.
- [ ] Implement client online mode, prediction/reconciliation, full-snapshot recovery, latency/jitter handling and reconnect without duplicating server rules.
- [ ] Add protocol compatibility, authority, malformed packet, rate/range, reconnect, persistence round-trip, crash recovery and multi-realm isolation tests.

**Testing stage - M9 server product gate:**

- Run protocol/persistence/server/client focused suites and all 54 current-head traces in offline and authoritative-host modes.
- Launch PostgreSQL fixtures and real WebSocket clients; exercise join/play/reconnect/save/restart. Run the target-compatible load/jitter profile and require server fixed-tick p95 at or below 40 ms including ZrVM and bridge time.

**Exit evidence:** Live authoritative server and online client share one simulation artifact, persist committed state, recover sessions and meet the 20 Hz timing gate.

## M10 - Complete client content, UI, assets, audio and localization

**Goal:** Expand the MVP into the full target-facing game client and content catalog.

**Dependencies:** M8 and M9; returned Runtime 04/09 remain mandatory.

**Implementation slices:**

- [ ] Port every target zone, camp, road, graveyard, gathering node, NPC, mob, quest, item, skill, talent, profession, class, encounter, dungeon, raid, delve and PvP/fiesta content definition into generated/validated ZrVM content modules.
- [ ] Ingest all 949 GLBs and verify inventory-derived animation and skin totals, meshopt, WebP, quantization and material extension projections through artifact-cache round trips.
- [ ] Port the complete target UI flow catalog: action bars, unit frames, cast/aura displays, inventory/equipment, character/talents/professions, quests/map, chat/social/guild/party/raid, mail/bank/market/trade, dungeon/PvP/delve views, modals, tooltips, settings, accessibility and mobile controls.
- [ ] Port original audio/music/voice manifests, font families, rich text, localization catalogs and locale fallback without shipping generated placeholder media.
- [ ] Reproduce target animation state, particles, ground effects, lighting, post-processing, camera behavior and performance budgets with inspectable original assets.
- [ ] Add content-reference, missing localization, asset dependency, UI action, accessibility, visual screenshot and save-compatibility suites for every catalog row.

**Testing stage - M10 complete client wave:**

- Run full project asset import, content/schema, retained UI, audio/font/localization and client product suites plus all 54 current-head traces.
- Execute representative complete flows for every UI/catalog family and compare approved screenshots/video against the target behavior and visual inventory at desktop and mobile-sized viewports.

**Exit evidence:** All target gameplay content and client-facing workflows exist; all original assets are accounted for and render/play through Zircon rather than placeholders.

## M11 - Administration, authoring, guide and external integrations

**Goal:** Reproduce the non-client operational product surfaces that are part of the target repository.

**Dependencies:** M9 and M10.

**Implementation slices:**

- [ ] Implement retained administration screens for accounts, characters, realms, moderation, grants, mail, market, guild/social state, metrics, audit records and controlled server operations.
- [ ] Implement project-local world/content authoring flows equivalent to the target editor for zones, camps, entities and content catalogs while preserving generated contract ownership.
- [ ] Materialize guide/wiki content, search/navigation and model/media inspection from the same checked-in content source used by the client.
- [ ] Implement capability-scoped adapters for email, Steam and wallet/chain integrations with deterministic offline fixtures, explicit unavailable states, credential isolation and audit logging.
- [ ] Reproduce update/version metadata, privacy/terms/credits/third-party notices and packaging-facing product identity.
- [ ] Add role/permission, audit, destructive-operation confirmation, content round-trip, guide navigation and integration contract tests.

**Testing stage - M11 operations wave:**

- Run admin/authoring/guide/integration suites against fixture services and a real local realm/database.
- Drive high-risk administration and authoring flows through the live retained UI, verify accessibility and require audit/persistence evidence for every mutation.

**Exit evidence:** Target operational, authoring, guide and integration capabilities are usable without Svelte/Electron embedding or hidden web-client fallback.

## M12 - Bots, headless RL and observability

**Goal:** Reproduce target bot and RL roles on the authoritative deterministic runtime.

**Dependencies:** M7 and M9.

**Implementation slices:**

- [ ] Implement `woc_bot` through the same public command/network contract used by real clients; no direct world mutation or privileged gameplay branch is allowed.
- [ ] Implement `woc_headless` reset/step/observation/action lifecycle with deterministic seeds, `player_level` and normalized talent allocation reset inputs, dynamic quest-objective observations, manual stepping, vectorized environments and bounded binary RL batches.
- [ ] Port target benchmark/load scenarios and observation/reward/termination semantics with recorded known vectors.
- [ ] Implement metrics, structured logs, traces, CPU/tick/queue/VM/GC diagnostics and fault correlation without adding nondeterministic inputs.
- [ ] Add bot fairness, reset reproducibility, parallel environment isolation, observation/action bounds, reward and benchmark regression tests.

**Testing stage - M12 headless product gate:**

- Run bot/RL/observability suites and compare repeated seeded episodes byte-for-byte.
- Run the target-compatible environment benchmark and server load profile while enforcing tick p95, memory, GC pause and queue bounds declared by the reference catalog.

**Exit evidence:** Network bots and headless RL environments execute the same ZrVM rules, reproduce seeded episodes and expose actionable bounded diagnostics.

## M13 - Desktop, Android, iOS and WebGPU/WASM hosts

**Goal:** Prove the same project/package contract on every target platform.

**Dependencies:** M10, M12 and returned Plugins 08/09 WOC handoffs.

**Implementation slices:**

- [ ] Configure Windows x64/arm64, Linux x64/arm64 and macOS universal desktop packages with project assets, ZrVM artifact, native adapter and update metadata.
- [ ] Configure Android and iOS hosts with persistent session ownership, real render surface, lifecycle, touch/pointer/keyboard/text/IME, audio, accessibility, suspend/resume and bounded asset access.
- [ ] Configure WebGPU/WASM host with persistent VM/session, canvas/surface lifecycle, complete pointer/text/IME input, allowlisted asset fetch and browser persistence/network behavior.
- [ ] Add platform package manifests, capability reports, startup/shutdown tests, input recordings, nonblank frame evidence and cross-platform save/protocol compatibility fixtures.
- [ ] Run identical deterministic trace inputs on every platform and compare state/event/RNG digests to the Windows reference.

**Testing stage - M13 real platform matrix:**

- Execute real desktop packages, Android emulator/device, iOS simulator/device and supported browsers. Compile-only and generated-source string checks are not acceptance.
- Require startup, at least two presented frames or two server ticks, one gameplay input, save/reload, background/foreground where applicable and clean shutdown on every host.

**Exit evidence:** Six platform families run the real game role and the same ZrVM artifact contract with matching deterministic digests; no host is a no-op shell.

### WOS74-WOS116 Eastbrook combat closure ledger

This ledger records source-owned implementation work completed after the
earlier plan slices. It is not release acceptance evidence: all rows still
require the canonical `zr_vm:project` execution path owned by Plugins08.

| Slice | Source-implemented closure | Persistent boundary |
|---|---|---|
| WOS74 | Warlock Summon Imp command and bounded owned Emberkin identity | No generic entity spawn/deletion abstraction |
| WOS75-76 | Paladin Seal of Righteousness and Judgement | Source aura/proc system remains unprojected |
| WOS77 | Emberkin Firebolt projectile, landing damage and owner credit | Pet pathing and generic pet combat remain absent |
| WOS78-81 | Passive/defensive target selection, open-ground follow, scalar maintenance and ranged chase | No spatial grid, cached path, collision or obstacle resolver |
| WOS82-83 | Owner-death demon retirement, three-second corpse and revivable retained row | Physical row deletion remains a Plugins08 backend requirement |
| WOS84 | Serpent Sting and Shadow Word: Pain action-bar/typed admission, nonphysical projectile/resist landing, pure DoT replacement/ticks and WOS63 rank/power snapshot persistence | Outdoor clear LOS only; no generic LOS/collision, aura observers, set bonuses or PvP target projection |
| WOS85 | Priest Renew action-bar/typed friendly admission, direct pure-HoT application, same-source replacement/ticks and WOS64 rank/power snapshot persistence | Player/owned-pet target projection only; no generic friendly targets, heal crit/absorb or aura observers |
| WOS86 | Priest Power Word Shield action-bar/typed friendly admission, six-second cooldown, same-source row refresh, WOS65 persistence and ordered Eastbrook melee absorb | No generic aura lifecycle, proc observers, pet damage sharing, friendly NPC targets or damage-source bridge |
| WOS87 | Priest Smite action-bar/typed hostile admission, timed cast, delayed cost, WOS56 projectile and Holy resist/range/critical impact | No generic spell multiplier, set/proc observer, multiplayer/PvP target or non-Eastbrook host projection |
| WOS88 | Priest Mind Blast action-bar/typed hostile admission, 1.5-second cast, successful-completion eight-second cooldown, WOS56 projectile and Shadow resist/range/critical impact | No generic spell multiplier, set/proc observer, multiplayer/PvP target or non-Eastbrook host projection |
| WOS89 | Priest Heal action-bar/typed friendly admission, 2.5-second cast, delayed cost and existing direct-heal range/critical/threat resolution | No generic friendly NPC, multiplayer ownership, heal observer, set/proc or non-Eastbrook host projection |
| WOS90 | Priest Flash Heal action-bar/typed friendly admission, 1.5-second cast, delayed cost and existing direct-heal range/critical/threat resolution | No generic friendly NPC, multiplayer ownership, heal observer, set/proc or non-Eastbrook host projection |
| WOS91 | Priest Mind Flay action-bar/typed hostile admission, three-tick Shadow channel, start-time cost and zero self-heal projectile landings | No generic interruption, spell multiplier/proc observer, multiplayer/PvP target or non-Eastbrook host projection |
| WOS92 | Shaman Lightning Bolt action-bar/typed hostile admission, rank-aware cast, delayed cost, WOS56 projectile and Nature resist/range/critical impact | No generic spell multiplier, set/proc observer, multiplayer/PvP target or non-Eastbrook host projection |
| WOS93 | Shaman Healing Wave action-bar/typed friendly admission, rank-aware cast, delayed cost and existing direct-heal range/critical/threat resolution | No generic friendly NPC, multiplayer ownership, heal observer, set/proc or non-Eastbrook host projection |
| WOS94 | Shaman Earth Shock instantaneous action-bar/typed hostile admission, shared Shock cooldown, WOS56 projectile and Nature resist/range/critical impact | No generic spell multiplier, set/proc observer, multiplayer/PvP target or non-Eastbrook host projection |
| WOS95 | Shaman Frost Shock instantaneous action-bar/typed hostile admission, shared Shock cooldown, Frost impact and retained slow aura | No generic spell multiplier, set/proc observer, multiplayer/PvP target or non-Eastbrook host projection |
| WOS96 | Shaman Flame Shock instantaneous action-bar/typed hostile admission, shared Shock cooldown, Fire direct impact, same-target DoT replacement and 12-second periodic damage | No generic spell multiplier, set/proc observer, multiplayer/PvP target or non-Eastbrook host projection |
| WOS97 | Shaman Flametongue Weapon instantaneous action-bar/typed no-target admission, mutually exclusive 300-second imbue and source bonus on retained physical melee swings | No generic aura lifecycle, triggered proc system, weapon-enchant inventory model or elemental melee damage bridge |
| WOS98 | Shaman Frostbrand Weapon (source display: Rimebound Weapon) instantaneous action-bar/typed no-target admission, mutually exclusive 300-second imbue and source bonus on retained physical melee swings | No generic aura lifecycle, triggered proc system, weapon-enchant inventory model or elemental melee damage bridge |
| WOS99 | Shaman Ghost Wolf (source display: Shadewolf) action-bar/typed no-target delayed cast, delayed cost, 3600-second self-toggle and source min-slow/max-speed movement composition | No generic aura lifecycle, broader speed-buff catalog, travel-form projection, obstacle/collision parity or multiplayer host projection |
| WOS100 | Shaman Stormstrike (source display: Ancestral Strike) instantaneous action-bar/typed melee admission, source resource/GCD/cooldown order and deterministic `weaponStrike` resolution | No generic melee aura/proc/talent modifier system, non-Eastbrook targets, collision/LOS parity or multiplayer host projection |
| WOS101 | Warlock Shadow Bolt (source display: Gloom Bolt) action-bar/typed hostile hard-cast admission, delayed billing, rank-aware in-flight profile and Shadow direct-hit resolution | No generic spell multiplier/talent/proc observer system, non-Eastbrook targets, collision/LOS parity or multiplayer host projection |
| WOS102 | Warlock Immolate (source display: Burning Pact) action-bar/typed hostile hard-cast admission, delayed billing, rank-aware Fire direct hit and five-tick hybrid DoT | No generic spell multiplier/talent/proc observer system, non-Eastbrook targets, collision/LOS parity or multiplayer host projection |
| WOS103 | Warlock Corruption (source display: Blackrot) action-bar/typed hostile hard-cast admission, delayed billing, rank-aware Shadow pure-DoT projectile and impact-time spell-power snapshot | No generic spell multiplier/talent/proc observer system, non-Eastbrook targets, collision/LOS parity or multiplayer host projection |
| WOS104 | Warlock Life Tap (source display: Hard Bargain) action-bar/typed no-target instant health-to-mana conversion, hasted GCD and capped resource restore | No talent modifier, emitted combat/presentation event, generic error event or multiplayer host projection |
| WOS105 | Warlock Curse of Agony (source display: Hex of Anguish) action-bar/typed hostile instant Shadow pure-DoT, one-draw projectile landing and 24-second periodic threat | No generic spell multiplier/talent/proc observer system, non-Eastbrook targets, collision/LOS parity or multiplayer host projection |
| WOS106 | Warlock Searing Pain (source display: Sear) action-bar/typed hostile 1.5-second Fire direct cast, delayed billing and source single-rank in-flight profile | No generic spell multiplier/talent/proc observer system, non-Eastbrook targets, collision/LOS parity or multiplayer host projection |
| WOS107 | Warlock Shadowburn (source display: Duskfire) action-bar/typed hostile instant Shadow direct cast, source GCD/billing/15-second cooldown order and in-flight profile | No generic spell multiplier/talent/proc observer system, non-Eastbrook targets, collision/LOS parity or multiplayer host projection |
| WOS108 | Warlock Demon Skin (source display: Fiendhide) action-bar/typed instant three-rank armor self-buff, WOS65 aura persistence and physical player-defense projection | No generic aura lifecycle/talent/proc observer system, non-Eastbrook target projection or multiplayer host projection |
| WOS109 | Warlock Rain of Fire action-bar/typed `castAt` position channel, authoritative 30-yard aim clamp, four one-second Fire AoE pulses and cast-aim snapshot cleanup | No generic spell multiplier/talent/proc observer system, collision/LOS parity, non-Eastbrook targets or multiplayer host projection |
| WOS110 | Warlock Conflagrate action-bar/typed instant Fire projectile, six-second cooldown and same-caster Immolate consumption before direct damage | No generic aura lifecycle/spell multiplier/talent/proc observer system, non-Eastbrook targets, collision/LOS parity or multiplayer host projection |
| WOS111 | Warlock Siphon Life action-bar/typed instant Shadow pure-DoT projectile, same-caster replacement, 30-second periodic damage and post-damage self-leech including a lethal tick | No generic aura lifecycle/spell multiplier/talent/proc observer system, non-Eastbrook targets, collision/LOS parity or multiplayer host projection |
| WOS112 | Druid Swiftmend action-bar/typed instant friendly consume-HoT direct heal, eight-second cooldown and source-order first-matching-HoT removal | No generic aura lifecycle/spell multiplier/talent/proc observer system, friendly NPC targets, multiplayer ownership or host projection |
| WOS113 | Druid Wrath (source display: Wildbolt) action-bar/typed hostile hard-cast admission, delayed billing, rank-aware Nature projectile profile and direct-hit resolution | No generic spell multiplier/talent/proc observer system, non-Eastbrook targets, collision/LOS parity or multiplayer host projection |
| WOS114 | Druid Healing Touch (source display: Wildmend) action-bar/typed friendly hard-cast admission, delayed billing, rank-aware direct-heal and effective-healing threat resolution | No generic friendly NPC targets, spell multiplier/talent/proc observer system, multiplayer ownership or host projection |
| WOS115 | Druid Starfire (source display: Skyfall) action-bar/typed hostile hard-cast admission, delayed billing, one-rank Arcane projectile profile and direct-hit resolution | No generic spell multiplier/talent/proc observer system, non-Eastbrook targets, collision/LOS parity or multiplayer host projection |
| WOS116 | Druid Entangling Roots (source display: Gripping Roots) action-bar/typed hostile hard-cast admission, delayed billing, two-rank Nature root projectile and rank-two periodic profile | No generic crowd-control DR, generic aura lifecycle, spell multiplier/talent/proc observer system, non-Eastbrook targets, collision/LOS parity or multiplayer host projection |

WOS84 extends the M4 retained ability generator from 23 to 25 entries, adding
source fields `minRange`, `scalesWith`, and generated rank-count access. Its
numeric resolver selects ranged power for Serpent Sting and spell power for
Shadow Word: Pain, snapshots it on successful impact, and validates the stored
profile during WOS63 decode. The focused static guard covers source definition,
projected catalog, command routing, projectile dispatch, WOS codec identity and
native protocol identity. It does not substitute for a ZrVM execution result.

WOS85 extends the retained M4 generator to 26 entries and adds a compact,
source-pinned pure-HoT profile module. Rejuvenation and Renew now resolve their
per-tick amount from generated rank and application-time spell power, with WOS64
persisting those facts. The WOS64 decoder preserves legacy WOS60-WOS63
Rejuvenation rows as resolved rank-zero records, but defers validation of a new
Renew row until its source-fact tail is loaded. Friendly casts run directly with
no projectile, resist draw or RNG mutation, and replace only the same
target/source/ability row. Static guards retain WOS72/WOS84 coverage under the
new envelope identity; they do not substitute for `zr_vm:project` execution.

**Current evidence:** the WOS39 motion-aura source guard, all 75 WOS44-WOS116
static guards, the CC generator and both M4 generator staleness checks pass
against `5ef9f7cb21cd8875b6d2c49701015dfcd78de35a`.
No direct Cargo or substitute VM execution was used.
Canonical `zr_vm:project` execution remains a Plugins08-owned acceptance item.

### WOS86 Power Word Shield implementation

The next independently reproducible source closure is Priest Power Word Shield.
The pinned source declares it as an instant friendly 30-yard Holy spell with a
six-second cooldown and one `absorb` effect: `48/90/145` for ranks 1-3, each
lasting 30 seconds. Friendly spell dispatch spends cost then applies the aura
directly. `applyAura` removes only conflicting same-id/same-source auras before
pushing the replacement; the damage core consumes `absorb` auras in reverse
insertion order, reduces their value, and removes a fully spent row before HP
loss or damage-derived follow-up work.

WOS86 advances the envelope to schema 65 with one bounded, insertion-ordered
absorbed-damage queue: target id, source id, ability code, generated rank,
remaining amount and remaining seconds. WOS2-WOS64 decode an empty queue.
Recasting one source's PWS removes and reinserts only that row, which makes the
source's refreshed shield newest without erasing independently cast shields.
The fixed tick ages this queue in the player-aura phase before Eastbrook mob
updates. Its focused damage bridge is deliberately limited to the retained
Eastbrook mob swing path: only damage remaining after that ordered queue breaks
incapacitate, pushes back a cast, changes HP or can cause death. The reference's
generic proc listeners, pet-damage sharing, NPC friendly-aura rules and every
other damage source remain separate work; no generic aura runtime is being
claimed. The required state is representable using existing ZrVM arrays and
does not expose a new ZirconEngine foundation gap. The source implementation
includes generated absorb-profile validation, slot and typed cast reducers,
same-source removal then append, WOS65 encode/decode, expiry and a focused
post-absorb Eastbrook mob-swing bridge. All 52 WOS static guards pass; canonical
`zr_vm:project` execution remains the separate Plugins08-owned evidence item.

### WOS87 Smite implementation

The next source-audited closure is Priest Smite. The pinned source defines a
30-yard Holy `directDamage` spell with four ranks: 2.0-second casts at ranks
one/two (`15-20`, cost 20; `26-33`, cost 32) and 2.5-second casts at ranks
three/four (`42-52`, cost 48; `64-78`, cost 70). The source releases every
nonphysical spell as a projectile at successful cast completion, spends the
cost then, and resolves one fully-resisted spell check at impact before drawing
the direct-damage range and critical outcome.

WOS87 adds Smite to M4 and reuses the existing WOS56 projectile facts, so it
does not change the WOS65 envelope. Its retained Eastbrook-only bridge uses
the existing timed-spell profile for source range, Spell Power, 1.5x spell crit
and Holy threat, with target death and combat entry following the existing
Frostbolt/Fireball path. Generic spell multipliers, set/proc observers,
multiplayer/PvP targets and all non-Eastbrook hosts remain separate work. The
implementation includes slot and typed admission, target revalidation at cast
completion, post-completion cost spend, source-rank projectile facts, landing
dispatch and WOS65 round-trip coverage. All 52 static guards pass; canonical
`zr_vm:project` execution remains a Plugins08-owned evidence task.

### WOS88 Mind Blast implementation

The next source-audited closure is Priest Mind Blast. The pinned source defines
a 30-yard Shadow `directDamage` spell with a 1.5-second cast and eight-second
cooldown: rank one is level 5, costs 50 and deals `42-46`; rank two is level 14,
costs 70 and deals `60-66`; rank three is level 20, costs 95 and deals `86-94`.
As with every source nonphysical spell, a successful cast completion spends the
cost, arms the cooldown and releases a projectile. One spell-resist draw happens
only at impact, followed by the generated direct-damage range and spell-critical
draws.

WOS88 adds Mind Blast as M4 entry 28, taking the generated catalog to 29
entries, while preserving the WOS65 layout. Its retained Eastbrook bridge adds
slot and typed admission, cooldown re-entry rejection, completion target
revalidation, post-completion billing/cooldown, source-rank Shadow projectile
facts, landing dispatch and WOS65 round-trip coverage. It deliberately excludes
generic spell multipliers, set/proc observers, multiplayer/PvP target projection
and non-Eastbrook hosts. All 52 static guards pass; canonical `zr_vm:project`
execution remains a Plugins08-owned evidence task.

### WOS89 Heal implementation

The next source-audited closure is Priest Heal. The pinned source defines a
30-yard friendly Holy `heal` with no cooldown: it is learned at level 14, costs
95, casts for 2.5 seconds and heals `165-195`; rank two is level 20, costs 130
and heals `230-270`. Source friendly targeting resolves the explicit friendly
target, then the current friendly target, then self, before a range gate. A
successful hard-cast completion spends cost and invokes the direct-heal effect:
the range plus Spell Power draw occurs before the healing-critical draw.

WOS89 adds Heal as M4 entry 29, taking the generated catalog to 30 entries,
while preserving WOS65. It reuses the retained friendly player/owned-pet target
projection and a state-private direct-heal resolver. The implementation supplies action-bar and typed admission, target
revalidation, successful-completion billing, deterministic range/crit order,
HP application, healing-threat projection and WOS65 round-trip coverage. It
deliberately excludes generic friendly NPCs, multiplayer ownership, heal
observers, set/proc behavior and non-Eastbrook hosts. All 52 static guards
pass; canonical `zr_vm:project` execution remains a Plugins08-owned evidence
task.

### WOS90 Flash Heal implementation

The next source-audited closure is Priest Flash Heal. The pinned source defines
a level-20, 30-yard friendly Holy `heal`: it costs 75, casts for 1.5 seconds,
has no cooldown and heals `120-142`. Source friendly targeting resolves the
explicit friendly target, then the current friendly target, then self before its
range gate. Completion spends cost and follows the direct-heal range plus Spell
Power draw before the healing-critical draw.

WOS90 adds Flash Heal as M4 entry 30, taking the generated catalog to 31
entries while preserving WOS65. The closure supplies action-bar and typed
admission, target revalidation, successful-completion billing, independent
cast-time coverage, shared direct-heal resolution and WOS65 round-trip coverage.
Generic friendly NPCs, multiplayer ownership, heal observers, set/proc behavior
and non-Eastbrook hosts remain separate work. All 52 static guards pass;
canonical `zr_vm:project` execution remains a Plugins08-owned evidence task.

### WOS91 Mind Flay implementation

The next source-audited closure is Priest Mind Flay. The pinned source defines
a level-14, 20-yard Shadow channel: it costs 45, lasts three seconds, emits
three ticks and uses `drainTick 12-12` with `healFrac = 0`. Channel admission
bills the resource at start. Each live target pulse validates target/range,
releases a projectile and, at landing, consumes its one range draw plus the
channel Spell Power rider. No spell-resist, critical, self-heal or healing-threat
branch applies to this source ability.

WOS91 adds Mind Flay as M4 entry 31, taking the generated catalog to 32 entries
without changing WOS65. The closure supplies action-bar and typed admission,
source channel clocks, live target gate, individual tick projectiles, zero-heal
profile validation, landing damage/threat and WOS65 round-trip coverage. Generic
interruption behavior, spell multiplier/proc observers, multiplayer/PvP targets
and non-Eastbrook hosts remain separate work. All 52 static guards pass;
canonical `zr_vm:project` execution remains a Plugins08-owned evidence task.

### WOS92 Lightning Bolt implementation

The next source-audited closure is Shaman Lightning Bolt. The pinned source
defines a 30-yard Nature `directDamage` spell with four ranks: `15-17` for 15
resource and 1.5 seconds at level 1, `26-30` for 25 and 2.0 seconds at level 8,
`45-51` for 40 and 2.5 seconds at level 14, and `75-85` for 60 and 3.0 seconds
at level 20. Like every source nonphysical hostile spell, a successful cast
completion spends its cost and releases a projectile. Landing performs one
spell-resist draw before direct-damage range and critical draws.

WOS92 adds Lightning Bolt as M4 entry 32, taking the generated catalog to 33
entries while preserving WOS65. The closure supplies Shaman action-bar and typed
admission, target revalidation, rank-aware cast facts, delayed billing, Nature
projectile landing and WOS65 round-trip coverage. Generic spell multipliers,
set/proc observers, multiplayer/PvP target projection and non-Eastbrook hosts
remain separate work. All 52 static guards pass; canonical `zr_vm:project`
execution remains a Plugins08-owned evidence task.

### WOS93 Healing Wave implementation

The next source-audited closure is Shaman Healing Wave. The pinned source
defines a 30-yard Nature friendly `heal` with four ranks: `36-44` for 25
resource and 1.5 seconds at level 1, `56-68` for 40 and 2.0 seconds at level 6,
`92-110` for 65 and 2.5 seconds at level 12, and `138-164` for 90 and 2.5
seconds at level 18. The source resolves a valid friendly target before it
spends cost at successful cast completion, then applies the direct-heal range,
Spell Power, critical and healing-threat sequence.

WOS93 adds Healing Wave as M4 entry 33, taking the generated catalog to 34
entries without changing WOS65. The closure supplies Shaman action-bar and typed
admission, source-valid friendly target revalidation, rank-aware cast facts,
delayed billing and the shared direct-heal resolver. Generic friendly NPCs,
multiplayer ownership, heal observers, set/proc behavior and non-Eastbrook hosts
remain separate work. All 52 static guards pass; canonical `zr_vm:project`
execution remains a Plugins08-owned evidence task.

### WOS94 Earth Shock implementation

The pinned source defines Shaman Earth Shock as an instant 20-yard Nature
`directDamage` spell: `19-22` for 30 resource at level 4, `33-38` for 45 at
level 10, and `54-61` for 65 at level 16. Every nonphysical targeted spell
still creates a projectile; successful admission spends cost, starts the 1.5
second GCD and arms the six-second shared Earth/Flame/Frost Shock cooldown.

WOS94 adds Earth Shock as M4 entry 34, taking the generated catalog to 35
entries. Its retained closure creates a Nature projectile immediately, resolves
one spell-resist draw then range/critical damage at landing, and preserves the
existing Eastbrook combat, threat and lethal paths. Generic spell multipliers,
set/proc observers, multiplayer/PvP targets and non-Eastbrook hosts remain
separate work. All 53 static guards pass; canonical `zr_vm:project` execution
remains a Plugins08-owned evidence task.

### WOS95 Frost Shock implementation

The pinned source defines Shaman Frost Shock as an instant 20-yard Frost spell
that costs 50 at level 8, deals `36-42` direct damage, slows movement by 50
percent for eight seconds, and shares the six-second Shock cooldown. Its
nonphysical cast creates a projectile, so resist occurs before direct damage and
the source slow effect at impact.

WOS95 adds Frost Shock as M4 entry 35, taking the generated catalog to 36
entries. It reuses the persistent Shock cooldown keys, the WOS56 projectile
landing path and the existing motion-aura slow representation. Generic spell
multipliers, set/proc observers, multiplayer/PvP targets and non-Eastbrook hosts
remain separate work. All 54 static guards pass; canonical `zr_vm:project`
execution remains a Plugins08-owned evidence task.

### WOS96 Flame Shock implementation

The pinned source defines Shaman Flame Shock as an instant 20-yard Fire spell:
rank one is level 8, costs 35, deals `25` direct damage and applies `28` total
damage in four three-second ticks over 12 seconds; rank two is level 16, costs
55, deals `42` direct damage and applies `48` total damage on the same cadence.
It shares the six-second Earth/Flame/Frost Shock cooldown. As a nonphysical
targeted spell it spends cost and launches a projectile at admission; at impact
one spell-resist draw precedes direct range/critical resolution. The source
applies the DoT only if that direct damage leaves the target alive, and the
hybrid direct-plus-DoT shape does not add a second spell-power rider to the DoT.

WOS96 adds Flame Shock as M4 entry 36, taking the generated catalog to 37
entries. It persists the resolved DoT profile in the existing WOS63 row and
replaces a prior Flame Shock row for the same target. The implementation also
adds source-profile snapshot validation for Earth and Frost Shock projectiles,
so all three shared-cooldown spells survive an in-flight encode/decode cycle.
The state regression covers flight round-trip, successful direct impact, four
periodic ticks, shared cooldown keys and typed command admission. Generic spell
multipliers, set/proc observers, multiplayer/PvP targets and non-Eastbrook hosts
remain separate work. All 55 static guards pass; canonical `zr_vm:project`
execution remains a Plugins08-owned evidence task.

### WOS97 Flametongue Weapon implementation

The pinned source defines Shaman Flametongue Weapon as an instant no-target
Fire imbue: rank one is learned at level 5, costs 25 and supplies an `8` bonus
for 300 seconds; rank two is learned at level 18, costs 40 and supplies `13`
for the same duration. Source imbue dispatch removes a prior, different imbue
before applying the new one. The source auto-attack implementation adds each
imbue value to its physical melee base before its physical critical and armor
resolution, which is the behavior retained here despite the ability's Fire
school presentation.

WOS97 adds Flametongue Weapon as M4 entry 37, taking the generated catalog to
38 entries. It reuses the WOS62 single mutually exclusive imbue row, now
validates its source rank and duration for both Seal of Righteousness and
Flametongue Weapon, and supplies the source flat bonus to retained offline auto
attack preparation. The regression covers action-bar and typed no-target
commands, exact payload bytes, immediate cost/GCD behavior, replacement of a
Seal row, WOS65 round-trip and expiry. Generic aura lifecycle, triggered melee
procs, inventory enchant state and an elemental melee-damage bridge remain
separate work. All 56 WOS44-WOS97 static guards pass; canonical
`zr_vm:project` execution remains a Plugins08-owned evidence task.

### WOS98 Frostbrand Weapon implementation

The pinned source identifies this ability as `frostbrand_weapon`, displayed as
Rimebound Weapon: an instant no-target Frost imbue learned at level 5 for 25
resource, supplying an `8` bonus for 300 seconds. Its rank two is learned at
level 20 for 40 resource and supplies `13` for the same duration. The shared
source imbue dispatcher removes any different imbue before it applies this one.
As with Flametongue, source auto attack includes the stored bonus in physical
melee base damage before physical critical and armor mitigation rather than
emitting a separate Frost hit.

WOS98 adds Frostbrand Weapon as M4 entry 38, taking the generated catalog to
39 entries. It reuses the WOS62 single mutually exclusive imbue row, extending
profile and encode validation to the three recognized row identities: Seal of
Righteousness, Flametongue Weapon and Frostbrand Weapon. Its regression covers
action-bar and typed no-target commands, exact payload bytes, immediate
cost/GCD behavior, replacement of a Flametongue row, WOS65 round-trip, physical
auto-attack bonus projection and expiry. Generic aura lifecycle, triggered
melee procs, inventory enchant state and an elemental melee-damage bridge
remain separate work. All 57 WOS44-WOS98 static guards pass; canonical
`zr_vm:project` execution remains a Plugins08-owned evidence task.

### WOS99 Ghost Wolf implementation

The pinned source identifies `ghost_wolf` as Shadewolf: a no-target Nature
self-buff learned at level 16, costing 35 with a two-second cast and no
cooldown. It applies one `buff_speed` row with value `1.4` for 3600 seconds.
The source self-buff dispatcher treats this ability as a toggle, so a later
successful cast charges cost and removes the existing row rather than
refreshing it. Source movement takes the smallest slow multiplier and multiplies
it by the largest speed buff or travel-form multiplier.

WOS99 adds Ghost Wolf as M4 entry 39, taking the generated catalog to 40
entries. It gives `buff_speed` stable motion-kind code 8 and reuses the existing
WOS65 motion-aura row, including its ability identity, owner source id, value
and expiry encoding. World-state validation permits that code only for a
same-entity Ghost Wolf row with the generated 1.4 and 3600-second profile.
The timed reducer defers cost until successful completion, and the regression
covers action-bar admission, in-flight and active-row round trips, typed toggle,
speed composition and expiry. It does not claim generic aura lifecycle, a
broader speed-buff catalog, travel-form projection, collision parity or
multiplayer host projection. The historical WOS39 guard now correctly checks
the current WOS65 envelope rather than its obsolete WOS54 header. At that
point, WOS39 and all 58 WOS44-WOS99 static guards passed; canonical
`zr_vm:project` execution remained a Plugins08-owned evidence task.

### WOS100 Stormstrike implementation

The pinned source defines `stormstrike` (displayed as Ancestral Strike) as a
level-20 Shaman physical melee ability: it costs 40, casts instantly, has a
12-second cooldown, requires a hostile target and executes one `weaponStrike`
with bonus 26. Its shared cast path arms the ordinary haste-adjusted GCD before
the resolution path bills cost and arms cooldown. The effect dispatcher then
passes that bonus, the default 1.0 weapon multiplier and resolved threat terms
to the shared melee hit table.

WOS100 appends Stormstrike as M4 entry 40, taking the generated catalog to 41
entries. It reuses the existing hostile Eastbrook melee range/facing gate,
deterministic `autoAttackState.meleeSwing` projection, sparse ability-cooldown
partition, combat entry, threat, lethal settlement, RNG state and WOS65
encode/decode behavior; no codec revision is necessary. The regression covers
direct resolution and round trip, cooldown expiry, typed-target admission and a
GCD/cooldown-blocked retry. It deliberately excludes generic melee aura, proc
and talent modifiers, non-Eastbrook targets, obstacle/collision parity and a
multiplayer host projection. At that point, all WOS39 and 59 WOS44-WOS100
static guards passed; canonical `zr_vm:project` execution remained a Plugins08-owned
evidence task.

### WOS101 Shadow Bolt implementation

The pinned source defines `shadow_bolt` (displayed as Gloom Bolt) as a Warlock
30-yard Shadow `directDamage` spell with no cooldown. Its four ranks are level
1 `13-18` for 25 resource and 1.7 seconds, level 8 `24-31` for 38 and 2.2
seconds, level 14 `42-53` for 55 and 2.7 seconds, and level 20 `68-84` for 80
and 3.0 seconds. The generic source casting path creates a nonphysical
projectile at successful completion; landing performs one spell-resist draw,
then direct range and critical draws with the resolved cast time.

WOS101 appends Shadow Bolt as M4 entry 41, taking the generated catalog to 42
entries without changing WOS65. It supplies Warlock action-bar and typed
admission, exact identifier bytes, rank-aware hard-cast facts, delayed billing,
in-flight profile validation, Shadow projectile landing and a WOS65 round-trip
state regression. The retained scope deliberately excludes generic spell
multiplier, talent and proc observers, non-Eastbrook targets, obstacle/collision
parity and multiplayer host projection. All WOS39 and 60 WOS44-WOS101 static
guards pass; canonical `zr_vm:project` execution remains a Plugins08-owned
evidence task.

### WOS102 Immolate implementation

The pinned source defines `immolate` (displayed as Burning Pact) as a Warlock
30-yard Fire hybrid spell with no cooldown. Rank one costs 25, casts in two
seconds, deals `11` direct damage and applies `20` total damage in five
three-second ticks; rank two is level 10, costs 40, deals `22` and applies 35;
rank three is level 16, costs 60, deals `38` and applies 60. The source creates
a nonphysical projectile after successful cast completion. At landing it spends
one spell-resist draw, then direct range/critical draws; the DoT is applied only
when that hit leaves the target alive. Its hybrid shape deliberately contributes
no second spell-power rider to periodic damage.

WOS102 appends Immolate as M4 entry 42, taking the generated catalog to 43
entries without changing WOS65. It supplies Warlock action-bar and typed
admission, exact identifier bytes, rank-aware cast and in-flight profiles,
delayed billing, Fire landing and same-target hybrid-DoT replacement. The state
regression covers cast and projectile round trips, three landing draws, exact
five-tick cadence and typed admission. The retained scope excludes generic spell
multiplier, talent/proc observer, non-Eastbrook target, obstacle/collision and
multiplayer host projection. All WOS39 and 61 WOS44-WOS102 static guards pass;
canonical `zr_vm:project` execution remains a Plugins08-owned evidence task.

### WOS103 Corruption implementation

The pinned source defines `corruption` (displayed as Blackrot) as a Warlock
30-yard Shadow pure DoT with no cooldown. Rank one is learned at level 4,
costs 35, casts for two seconds and deals 40 total damage over six three-second
ticks; rank two is level 12, costs 55 and deals 72; rank three is level 18,
costs 75 and deals 85. Successful cast completion creates a nonphysical
projectile. Its landing consumes exactly one spell-resist draw; on success the
shared pure-DoT reducer snapshots source spell power and persists the generated
rank, duration, interval and resolved tick amount.

WOS103 appends Corruption as M4 entry 43, taking the generated catalog to 44
entries without changing WOS65. It adds exact identifier-byte routing, rank to
learn-level validation for retained pure-DoT projectiles, delayed billing,
Shadow projectile landing and same-target pure-DoT replacement. The state
regression covers cast and projectile encode/decode, completion billing, the
single impact draw, source-power snapshot, periodic tick, and typed admission.
While closing the shared profile validation, it also corrects the pure-DoT GCD
floor from `0.7` to the pinned source's `0.75` seconds. The retained scope
excludes generic spell multiplier, talent/proc observer, non-Eastbrook target,
obstacle/collision and multiplayer host projection. All WOS39 and 62
WOS44-WOS103 static guards pass; canonical `zr_vm:project` execution remains a
Plugins08-owned evidence task.

### WOS104 Life Tap implementation

The pinned source defines `life_tap` (displayed as Hard Bargain) as a Warlock
instant, no-target Shadow ability with no cost or cooldown. Its rank-one
level-6 effect converts 30 health into 30 mana; rank two is level 14 and
converts 55; rank three is level 20 and converts 85. The normal spell cast path
arms the hasted GCD before effect dispatch. The effect then rejects the
conversion when health is less than or equal to the health price; otherwise it
subtracts health and restores mana capped at maximum, with no RNG draws.

WOS104 appends Life Tap as M4 entry 44, taking the generated catalog to 45
entries without changing WOS65. The M4 Zr generator now projects the source
`hp` and `mana` effect metrics rather than discarding them. The retained
closure adds exact no-target identifier routing, rank-profile validation,
source GCD-before-health-gate ordering and capped resource restoration. Its
state regression covers action-bar and typed commands, maximum-resource cap,
health-gate GCD retention, snapshot encoding and zero RNG mutation. Improved
Life Tap's talent multiplier, source damage/spellfx events, generic error DTOs
and multiplayer host projection remain separate source owners. All WOS39 and
63 WOS44-WOS104 static guards pass; canonical `zr_vm:project` execution
remains a Plugins08-owned evidence task.

### WOS105 Curse of Agony implementation

The pinned source defines `curse_of_agony` (displayed as Hex of Anguish) as a
Warlock instant 30-yard Shadow pure DoT with no cooldown. Rank one is level 8,
costs 25 and deals 36 total damage over eight three-second ticks; rank two is
level 14, costs 40 and deals 72; rank three is level 20, costs 60 and deals 78.
The generic nonphysical spell path bills the instant cast and sends a
projectile; its impact takes one spell-resist draw, then the pure-DoT reducer
snapshots spell power and creates the periodic row.

WOS105 appends Curse of Agony as M4 entry 45, taking the generated catalog to
46 entries without changing WOS65. It adds exact identifier-byte routing and
the 8/14/20 rank mapping to the existing pure-DoT profile, admission,
projectile and same-target replacement path. The shared periodic reducer now
uses `pureDotAbilityIndex` for threat metrics, which closes the same
post-impact threat path for Corruption and future pure DoTs. The state
regression covers action-bar/typed casts, codec boundaries, one impact draw,
source-power snapshot, the 24-second tick and periodic threat. Generic spell
multiplier, talent/proc observer, non-Eastbrook target, obstacle/collision and
multiplayer host projection remain separate work. All WOS39 and 64 WOS44-WOS105
static guards pass; canonical `zr_vm:project` execution remains a Plugins08-owned
evidence task.

### WOS106 Searing Pain implementation

The pinned source defines `searing_pain` (displayed as Sear) as a single-rank
Warlock Fire direct-damage cast: level 14, 35 resource, 1.5 seconds, 30 yards
and `30-38` damage, with no cooldown. WOS106 appends it as M4 entry 46 and
takes the generated catalog to 47 entries without changing WOS65. Successful
cast completion bills the source cost and retains a Fire projectile whose
profile, one resist draw, direct range/critical resolution, threat and typed
admission are source-pinned. Generic spell multipliers, talent/proc observers,
non-Eastbrook targets, collision/LOS parity and multiplayer host projection
remain separate work.

### WOS107 Shadowburn implementation

The pinned source defines `shadowburn` (displayed as Duskfire) as a single-rank
level-14 Warlock instant Shadow direct-damage spell: 70 resource, 15-second
cooldown, 20 yards and `56-66` damage. WOS107 appends it as M4 entry 47 and
takes the generated catalog to 48 entries without changing WOS65. The retained
instant path first arms the hasted GCD, then bills cost, starts the sparse
ability cooldown and queues the nonphysical Shadow projectile; landing keeps
the source one-resist then direct-hit random order. Generic spell multipliers,
talent/proc observers, non-Eastbrook targets, collision/LOS parity and
multiplayer host projection remain separate work.

### WOS108 Demon Skin implementation

The pinned source defines `demon_skin` (displayed as Fiendhide) as a Warlock
instant Shadow self-buff with no cooldown: ranks at levels 1/12/20 cost
`20/35/50` and snapshot `30/55/80` armor for 1800 seconds. WOS108 appends it
as M4 entry 48, taking the generated catalog to 49 entries without changing
WOS65. The retained aura contract adds source `buff_armor` as a constrained
kind, preserves its ability/source/value/remaining facts in the existing row,
and includes the active value in player physical mitigation after Sunder Armor.
Generic aura lifecycle, talent/proc observers, non-Eastbrook hosts and
multiplayer target projection remain separate work.

### WOS109 Rain of Fire implementation

The pinned source defines `rain_of_fire` as a level-18 Warlock Fire position
channel: 85 resource, zero cast time, 10-second cooldown, 30-yard authoritative
aim clamp, and four one-second `14-18` radius-seven pulses. WOS109 appends it
as M4 entry 49, taking the generated catalog to 50 entries without changing
WOS65. The existing `castAt` command payload now decodes its finite little-endian
`f64` coordinates through the shared binary reader, captures the clamped center
in the already-persisted cast-aim fields, and arms the existing fixed-count
channel lifecycle. Every consumed channel tick uses the generated channel
spell-power coefficient and retained deterministic ground-AoE projection; no
instant pulse is emitted. Completion or cancellation clears the captured aim.
Generic spell multipliers, talent/proc observers, collision/LOS parity,
non-Eastbrook targets and multiplayer host projection remain separate work.

### WOS110 Conflagrate implementation

The pinned source defines `conflagrate` as a level-10 Warlock instant Fire
spell: it costs 55 resource, has a six-second cooldown and a 30-yard hostile
target gate. Its one `consumeAura` effect searches only consumable hot/DoT
auras, and a hostile DoT must have been applied by the same caster. A successful
cast still spends cost, arms cooldown and launches the Fire projectile when no
matching Immolate exists. At a non-resisted impact it removes that matching
Immolate before drawing the `54-64` direct range and critical result; a missing
or foreign Immolate ends after the resistance draw without damage.

WOS110 appends Conflagrate as M4 entry 50, taking the generated catalog to 51
entries without changing WOS65. The generated Zr projection now exposes the
source nested `consumeAura.deal` metrics and its ordered `auraIds`. The
retained projectile row records the generated Fire damage profile and zero cast
time, so the existing codec validates it across reload. The state regression
covers own-DoT consumption, typed command admission, snapshot round-trip, and
the no-own/foreign-DoT branches' one-resistance-draw boundary. Generic aura
lifecycle, spell multipliers, talent/proc observers, non-Eastbrook target
projection, collision/LOS parity and multiplayer hosts remain separate work.

### WOS111 Siphon Life implementation

The pinned source defines `siphon_life` as a level-10 Warlock instant Shadow
spell. It costs 45 resource, has no cooldown, targets a hostile within 30 yards
and sends the ordinary nonphysical projectile. A successful non-resisted impact
applies one 30-second, ten-tick DoT for a total of 60 damage. Every periodic
tick deals its resolved damage first, then heals the living caster for
`round(tickDamage * leechPct)` up to maximum health and applies the source
effective-healing threat. The heal still occurs when that tick kills the target.

WOS111 appends Siphon Life as M4 entry 51, taking the generated catalog to 52
entries. The generated effect projection exposes the source `leechPct: 1`,
while WOC derives this fixed source behavior from the persisted ability code,
rank and existing DoT snapshot rather than adding another WOS65 column. The
state regression covers action-slot and typed admission, snapshot round-trip,
periodic damage, self-leech threat, and the lethal-tick healing order. Generic
aura lifecycle, spell multipliers, talent/proc observers, non-Eastbrook target
projection, collision/LOS parity and multiplayer hosts remain separate work.

### WOS112 Swiftmend implementation

The pinned source defines `swiftmend` as a level-10 Druid instant Nature
friendly spell: it costs 55 resource, has an eight-second cooldown and a
30-yard target gate. Helpful spells resolve before the source projectile branch.
After cost and cooldown, its `consumeAura` effect scans target auras in insertion
order, consumes the first matching `hot` regardless of aura source, then draws
the `105-125` direct-heal range and the shared healing-critical result. A target
with no matching HoT still pays and starts cooldown but draws neither value.

WOS112 appends Swiftmend as M4 entry 52, taking the generated catalog to 53
entries without changing WOS65. The generated projection now exposes
`consumeAura.auraKind` and nested `consumeAura.heal` metrics. The world reuses
the existing friendly player/owned-pet target gate, ordered WOS64 HoT queue and
direct-heal/healing-threat kernel; its regression covers cross-source HoT order,
resource/cooldown behavior, two-draw positive resolution, empty-HoT behavior
and typed command admission. Generic aura lifecycle, spell multipliers,
talent/proc observers, friendly NPC targets, multiplayer ownership and host
projection remain separate work.

### WOS113 Wrath implementation

The pinned source defines `wrath` (displayed as Wildbolt) as a level-1 Druid
hostile Nature spell with four ranks: `13-16 / 24-29 / 38-45 / 60-71`, learned
at levels `1 / 8 / 14 / 20`. Costs are `20 / 32 / 48 / 70`; cast time changes
from `1.5` to `2.0` seconds at rank 2 and remains two seconds for ranks 3-4.
It has no cooldown and requires a target within 30 yards. The source hard-cast
path validates target/facing at admission, rechecks range on completion, bills
only after a successful cast, then queues a nonphysical Nature projectile. Its
impact consumes one spell-resist draw before direct range and spell-critical
draws, combat/threat mutation and lethal settlement.

WOS113 appends Wrath as M4 entry 53, taking the generated catalog to 54
entries without changing WOS65. The world stores rank, resolved damage bounds,
school and cast time in the existing projectile row, validates that snapshot on
decode, and covers action-slot/typed admission, delayed billing, state
round-trip and projectile settlement. Generic spell multipliers, talent/proc
observers, non-Eastbrook targets, collision/LOS parity and multiplayer host
projection remain separate work.

`world/state.zr` remains the reducer boundary for this slice despite its size:
its `WorldState` and WOS codec own parallel mutable projectile columns, and the
existing source records that the experimental cross-module `container.Array`
ABI cannot safely carry those columns. The smallest future split is a hostile
spell cast/projectile reducer once the canonical `zr_vm:project` backend accepts
that mutable-array boundary; WOS113 must not manufacture a second state owner
before that capability is proven.

### WOS114 Healing Touch implementation

The pinned source defines `healing_touch` (displayed as Wildmend) as a level-1
Druid friendly Nature hard cast with four ranks: `37-51 / 68-86 / 115-140 /
175-208`, learned at levels `1 / 8 / 14 / 20`. Costs are `25 / 45 / 75 / 110`;
cast time changes from `2.5` to `3.0` seconds at rank 2 and stays three seconds
for ranks 3-4. The source resolves the friendly target inside `max(range, 5)+2`,
locks it during the cast, spends only at successful completion, then performs
the direct-heal range draw followed by shared healing-critical handling and
effective-healing threat. Helpful casts have no projectile or spell-resist path.

WOS114 appends Healing Touch as M4 entry 54, taking the generated catalog to 55
entries without changing WOS65. The world retains the source rank, cost and
cast-time facts through the existing direct friendly-cast reducer, keeps
source-specific target/range identity, and reuses the authoritative numeric and
healing-threat kernel. Its regression covers action-slot/typed admission,
delayed billing, state round-trip, both random draws, health restoration and
threat attribution. Generic aura lifecycle, spell multipliers, talent/proc
observers, friendly NPC targets, multiplayer ownership and host projection
remain separate work.

### WOS115 Starfire implementation

The pinned source defines `starfire` (displayed as Skyfall) as a level-14 Druid
hostile Arcane hard cast. It costs 80 resource, casts for three seconds, has no
cooldown, requires a 30-yard target and deals direct `80-112` damage with no
ranks. Admission validates target/facing and completion rechecks range before
delayed billing. The nonphysical Arcane projectile spends one spell-resist draw
on impact before its direct range and spell-critical draws, combat/threat
mutation and lethal settlement.

WOS115 appends Starfire as M4 entry 55, taking the generated catalog to 56
entries without changing WOS65. The existing projectile row persists the
one-rank source bounds, Arcane school and cast time; decode validates that
profile and the regression covers action-slot/typed admission, delayed billing,
round-trip and impact ordering. Generic spell multipliers, talent/proc
observers, non-Eastbrook targets, collision/LOS parity and multiplayer host
projection remain separate work.

### WOS116 Entangling Roots implementation

The pinned source defines `entangling_roots` (displayed as Gripping Roots) as a
level-8 Druid hostile Nature hard cast. Rank 1 costs 35 resource and Rank 2,
learned at level 16, costs 50; both take 1.5 seconds, have no cooldown and
require a target within 30 yards. A successful completion bills cost and queues
the normal nonphysical spell projectile. Impact first consumes one spell-resist
draw. A non-resisted result applies a 12-second root and enters combat; Rank 2
then applies its ordered `32` total, 12-second, three-second-interval DoT.
Because the retained target is an Eastbrook mob rather than a player, the source
does not enter its PvP root diminishing-return branch.

WOS116 appends Entangling Roots as M4 entry 56, taking the generated catalog to
57 entries without changing WOS65. The existing motion-aura row persists root
identity, source and duration; Eastbrook pursuit now reads its already-generated
root kind to suppress translation while retaining its in-range melee progression.
The existing DoT row carries Rank 2's source power snapshot, resolved periodic
amount and codec validation. The regression covers action-slot/typed admission,
delayed billing, round-trip, one-draw impact, root persistence and Rank 2 DoT
application. Generic crowd-control DR, aura lifecycle, spell multipliers,
talent/proc observers, non-Eastbrook targets, collision/LOS parity and
multiplayer host projection remain separate work.

## M14 - One-to-one completion audit and release candidate

**Goal:** Prove the original objective requirement by requirement instead of inferring completion from selected passing tests.

**Dependencies:** M0-M13 and all six WOC-origin handoffs returned or explicitly superseded by stronger accepted evidence.

**Implementation slices:**

- [ ] Close every row in `reference/test_catalog.json` with an executable Zircon test/evidence owner or a precise platform/tooling equivalence rationale; missing and `not_run` rows are release blockers.
- [ ] Close all 165 command rows and all 248 `IWorld` member/28-facet rows with schema, implementation and exercised-test evidence.
- [ ] Recompute the 54-scenario, 949-GLB, inventory-derived animation/skin, content, UI, localization, audio and platform inventories from current files and reject count/hash drift.
- [ ] Run behavior, protocol, persistence, UI/accessibility, rendering/audio, admin/authoring, integration, bot/RL and platform product journeys from fresh installations and upgraded saves.
- [ ] Run release security, malformed input, capability isolation, deterministic replay, performance, soak, packaging and license/notice gates.
- [ ] Produce one completion matrix that links each explicit objective requirement to authoritative current-state evidence and records no unsupported completion claims.

**Testing stage - M14 release acceptance:**

- Run the nested WOC workspace, affected Zircon packages, target reference suites, all 54 parity traces, real product journeys and platform matrix as a dependency-complete execution wave.
- Re-run any repaired lower-layer focused batch before the complete wave. Treat indirect or missing evidence as incomplete work, not accepted residual risk.

**Exit evidence:** Every catalog row and explicit objective requirement is proven; all required tests/product/platform gates pass; the full WOC project is usable from `examples/woc` without the target web runtime or a fallback language.

## 状态与产出记录

每个里程碑测试通过后记录一次；实现切片不单独写入产出记录。

| 里程碑 | 范围 | 状态 | 完成日期 | 验证批次 / 残余风险 |
|---|---|---|---|---|
