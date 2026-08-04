---
title: WOS197 Aether Surge charge runtime
status: implemented_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS197 Aether Surge Charge Runtime

## Scope

Integrate the existing source-pinned `combat/aether_surge_state.zr` decisions
into WOC world state for the single-target `arcane_surge` (Aether Surge)
transaction. Retain the source order: a timed Arcane hit completes in place,
reads current charge count for cost/damage/cast speed, resolves its normal
spell impact, stores one refreshed charge (cap four), then consumes exactly
one authoritative random draw for the 25% next-Surge-free proc.

The durable state is restricted to the charge aura and free-cast window. Its
expiration, source ownership, encode/decode migration, rankless M4 profile,
typed/slot command paths, and source Arcane specialization gate belong here.
The Aether Darts channel dump, Perfect Moment, and Temporal Cascade are
separate follow-up milestones because they use different lifecycle and target
transactions.

## Source Contract

- `classes.ts#arcane_surge` is a level-five Arcane-only 2-second, 30-yard,
  projectile-free direct Arcane spell: base cost 16 and base damage 10-13.
- `chronomancy.ts` reads `N` charges before the cast: cost is `16 * 2^N`,
  damage is multiplied by `1 + 0.3N`, and cast time by `1 - 0.05N`, with an
  additional `0.5` cast multiplier while Aether Rush is armed. Charges cap at
  four and refresh a 10-second duration after a successfully resolved cast.
- The source rolls the Aether Rush 25% chance only after Surge damage resolves
  and the charge aura is refreshed. The armed free cast applies only to the
  next Aether Surge, costs zero at completion, and expires after 15 seconds.
- `effect_dispatch.ts` resolves Surge damage in the direct-damage effect and
  invokes `aetherSurgeAddStack` after that hit. Its fixed completion boundary
  prevents a back-to-back cast from observing stale charges.

## Delivery Order

1. Add the source ability to the M4 generated catalog and write a red
   source/transaction guard plus a `zr_vm:project` fixture. Preserve all
   existing ability indexes; append the new row only.
2. Add bounded, source-keyed charge and Aether Rush fields to `WorldState`,
   including schema migration, validation, canonical encoding/decoding and
   fixed-tick expiry. Reuse the existing `aether_surge_state.zr` exact
   integer-charge decision module; do not introduce a second math or combat
   rule owner.
3. Implement the timed hostile cast and completion reducer. Capture the
   pre-resolution charge profile once, use generated cost/range/damage data,
   apply the normal spell critical and Temporal Echo damage paths, then commit
   the refreshed charge and exactly one post-impact free-proc draw.
4. Route exact typed payload and spellbook slot paths; verify Arcane
   specialization, charge-scaled cost/cast-time/damage, free-cost consumption,
   RNG order, expiry, rejection without state reservation, and round-trip
   stability.
5. Run current generator checks, WOS193-196 regressions, static guard and an
   independent second review. Run the fixture only through `zr_vm:project`;
   an absent plugin backend defers accepted closeout but never blocks the next
   non-validation implementation step.

## Exclusions

- `arcane_missiles` / Aether Darts first-landed charge consumption and its
  five-missile full-charge barrage.
- Perfect Moment's preserved full-charge window, Temporal Cascade's group
  marks, and all UI/visual work.
- Any game-local replacement for the engine's reliable ZrVM plugin backend.

## Dynamic Validation

The future package must execute only through `zr_vm:project`; no local VM or
native fallback is acceptable evidence.

## Second Review

2026-08-03: second static review confirmed pre-resolution charge reads,
post-impact charge refresh, the single ordered Aether Rush draw, exact free
cast consumption, persistence and typed/slot routing. The focused fixture
declares `zr_vm:project`, and
`node tools/wos197_aether_surge_charge_runtime_static_guard.mjs` passed from
`examples/woc`. Dynamic ZrVM execution remains pending.
