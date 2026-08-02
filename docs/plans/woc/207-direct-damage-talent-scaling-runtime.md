---
title: WOS207 direct-damage talent scaling runtime closure
status: planned
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS207 Direct-Damage Talent Scaling Runtime

## Source Contract

`applyTalentMods` resolves a `directDamage` range before the combat resolver as
`Math.round(value * dmgMult + flat)`. `dmgMult` is `1 + globalDmg +
ability.dmgPct`, with `globalDmg` selecting `meleeDmgPct` for the physical
school and `spellDmgPct` for every other school. An enemy `consumeAura.deal`
uses the same range rule and receives `flat` once. The resolved integer
endpoints enter the existing range selection; spell power, hit/resist, crit,
damage multipliers, threat, controls and follow-up effects retain their current
owners.

The source talent pass precedes the one-shot Overload transform. A Fireball or
Pyroblast direct range therefore resolves talent endpoints first and then has
each positive endpoint rounded by the Overload multiplier. The existing DoT
total remains outside this direct-only slice.

## Implementation Scope

- Add `combat/direct_damage_talent_scaling_state.zr` as the pure source-range
  owner. It accepts raw endpoints, school, existing `TalentModifierState` and
  ability ID, and owns no `WorldState`, persistence, RNG, damage settlement or
  threat behavior.
- Resolve eligible hostile direct ranges at projectile launch and store the
  resolved endpoints in the existing projectile fields. Generic ability
  projectiles use the shared launch boundary; the Fireball/Pyroblast path
  composes source talent resolution before its existing Overload transform.
- Keep the queue snapshot authoritative across flight and encode/decode.
  Persist the launch talent selection alongside each direct projectile and
  recompute its exact source-resolved range during restore, so a later respec
  neither changes a legitimate payload nor permits an arbitrary bounded range.
  Historical-schema writers reject a queued non-empty launch selection instead
  of silently dropping it; WOS106 remains available only for payloads with no
  such snapshot.
  Conflagrate must consume its queued deal range at impact rather than
  re-reading unmodified catalog values.
- Add a pure contract for physical/spell global selection, ability percent,
  flat addition and endpoint-before-interpolation rounding. Add a WorldState
  regression for Frost's current `frostbolt` modifier, launch-time snapshot
  retention, encode/decode validity, unchanged RNG ownership and the
  Fireball/Pyroblast talent-before-Overload ordering.
- Add a source-pinned static guard and a `zr_vm:project` manifest. Dynamic
  project execution and Cargo acceptance remain assigned to Plugins08.

## Exclusions

This slice does not change `dot`, `drainTick`, `aoeDamage`, `aoeRoot`,
`weaponDamage`, `weaponStrike`, `finisherDamage`, `judgement`, `imbue`,
`lifeTap`, `gainResource`, buff magnitude, costs, cooldowns, cast times,
absorbs, healing, threat formulas or hit/crit mechanics. Those source arms
remain separate follow-up work.

## Deferred Authority

The change remains wholly inside the existing WOC `zr_vm:project` plugin
project. Its WOC-local state format advances to schema 107 to retain launch
talent selections; it introduces no Zircon Rust crate, engine ABI, lifecycle
change, native fallback or host contract. Dynamic ZrVM/Cargo acceptance
remains owned by Plugins08.

## Status And Output Record

| Milestone | Scope | Status | Date | Evidence |
|---|---|---|---|---|
