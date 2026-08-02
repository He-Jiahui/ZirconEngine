---
title: WOS173 Dragon's Breath state foundations
status: implemented_static_dynamic_validation_deferred
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS173 Dragon's Breath State Foundations

## Architecture note

This is a leaf WOC script-state change, not a new Zircon runtime subsystem.
`combat/fear_dr_state` owns the source-pinned, deterministic fear decision;
the existing `WorldState` owns every persistent row, its snapshot lifecycle and
the candidate transaction. The Dragon command route will consume those two
boundaries and must not obtain host objects or establish a parallel gameplay
runtime. The existing `combat/` folder already owns Fire and empowered-cast
pure rules, so no new umbrella module or cross-crate facade is introduced.

## Source contract

Fire Mage's four-stage `dragons_breath` is an `empoweredCone` with a Fire
school, 90 resource cost, 20-second cooldown, one post-batch `noteSpellHit`,
and a damage-breakable `incapacitate` effect. Each hit supplies its already
resolved crit to that one batch note. `noteSpellHit` advances Fire's Heating Up
and Hot Streak auras without rolling new RNG.

The source reduces an incapacitate in the `fear` DR category only when a player
hits a hostile player. It stores the target's category stage plus reset time;
successive fear durations are selected from the source ladder and a non-player
pair receives the supplied duration unchanged. Damage is applied before DR and
incapacitate; a newly dead target receives neither control nor combat entry,
but still contributes to Dragon's Breath's one Hot Streak note.

## Implementation order

1. Add a focused `fear_dr_state` pure module that models the current-head fear
   ladder, reset-time lookup and player-pair applicability. Keep it separate
   from `stun_dr_state`: fear is a distinct category and Dragon's Breath does
   not share stun buckets.
2. Reuse the already-versioned, per-entity `entityFearDrStages` and
   `entityFearDrResetAt` rows by wiring them through the canonical resolver.
   Add only Fire's Heating Up/Hot Streak pair as new persistent rows, with
   defaults, validation, encode/decode, copy/removal and fixed-tick expiry
   before any Dragon command route uses them.
3. WOS173a completes the pure fear resolver plus the durable Fire-proc/fear
   rows, their v84 snapshot tail, old-snapshot zero migration and fixed-tick
   expiry. It deliberately does not enable a candidate command.
4. WOS174 must first add the missing Combustion world-state/ability and
   `empower_next` spender-consumption bridge, then call
   `fire_mage_state.hotStreakOnSpellHit` once after the Dragon batch and finally
   integrate `dragons_breath` with Fire-spec admission, id-149 release,
   2.4-second arm/release/natural completion, stage-four guaranteed crit,
   cooldown/cost, damage-first feared incapacitate, and source-school aura
   rows.

## Boundaries

- The fear DR row is target-owned and applies only to hostile player pairs;
  Eastbrook NPC targets retain the full source duration. It is not a generic
  replacement for root, polymorph, lockout or stun DR.
- Dragon's Breath must never be enabled by omitting Hot Streak, fear DR,
  damage-break behavior, or the source's post-batch ordering.
- The real candidate transaction remains the `zr_vm:project` plugin path.
  WOS173 adds no host-side gameplay fallback and defers dynamic transaction
  acceptance to the open Plugins08 handoff.

## Verification

- Source anchors pin `fireMageOnSpellHit`, the `fear` branch of
  `diminishedCrowdControlDuration`, and the Dragon's Breath dispatch ordering.
- Pure tests cover fear's source `8/4/2/1` fixed-duration ladder (the fourth
  and later applications remain one second), reset behavior and Fire batch
  note behavior without RNG.
- WorldState tests cover serialized state, Fire batch application, target-pair
  applicability, damage-break incapacity and full Dragon release once the
  state foundations are integrated.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
