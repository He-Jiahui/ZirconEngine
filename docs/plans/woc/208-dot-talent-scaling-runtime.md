---
title: WOS208 periodic-damage talent scaling runtime closure
status: planned
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS208 Periodic-Damage Talent Scaling Runtime

## Source Contract

`applyTalentMods` resolves each non-`directPct` `dot` effect before combat as
`Math.round(total * dmgMult * dotMult + flat)`. `dmgMult` is `1 + globalDmg +
ability.dmgPct`, selecting `meleeDmgPct` only for the physical school and
`spellDmgPct` otherwise. `dotMult` is `1 + dotDmgPct`; `flatDmg` is added once
after both multipliers. A `directPct` rider is unchanged because it consumes
an already resolved direct hit and scaling it again would double apply damage
modifiers.

The combat dispatcher next divides this resolved total over periodic ticks and
then adds the pre-existing cast-time power coefficient only for effects that
are not direct/chain/AOE riders. This includes the current pure DoTs, Rake and
Entangling Roots, while Fireball, Pyroblast, Immolate, Flame Shock and
Moonfire retain their existing no-double-dip rider treatment. Fireball and
Pyroblast apply their one-shot Overload multiplier only after talent total
resolution.

## Implementation Scope

- Add `combat/dot_talent_scaling_state.zr` as the pure owner for global school
  selection, ability percentage, global periodic percentage, flat addition,
  source rounding and the `directPct` exemption. It owns no `WorldState`, RNG,
  combat settlement, power coefficient or threat.
- Extend the existing periodic numeric profile helpers to accept a resolved
  total while retaining their existing power-snapshot coefficient behavior.
  Route all fourteen current catalog `dot` effects through it: Fireball,
  Pyroblast, Moonfire, Serpent Sting, Shadow Word: Pain, Flame Shock,
  Immolate, Corruption, Curse of Agony, Siphon Life, Entangling Roots, Rake,
  Insect Swarm and Rip.
- Capture a talent selection at projectile launch or immediate application;
  transfer the launch snapshot to a landed aura, not the caster's later live
  selection. Persist each active DoT's resolved total, rank and launch
  selection in WOS108, and recompute the exact legal total during validation.
  Fireball/Pyroblast accept only their exact resolved total or its exact
  post-talent Overload transform.
- Normalize WOS107 and earlier periodic rows into their no-talent historical
  source totals. Historical writers reject non-empty DoT talent snapshots so
  they cannot silently discard data required for exact restoration.
- Add a pure resolver contract and a WorldState regression for spell/physical
  selection, `dotDmgPct`, ability percentage and flat ordering, directPct
  bypass, snapshot retention through respec and encode/decode, rejection of a
  tampered total, Fireball/Pyroblast talent-before-Overload ordering, and
  unchanged RNG ownership. Add a source-pinned static guard plus a
  `zr_vm:project` manifest.

## Exclusions

This slice does not change `drainTick`, `aoeDamage`, `aoeRoot`, `weaponDamage`,
`weaponStrike`, `finisherDamage`, `judgement`, `imbue`, `lifeTap`, direct
healing, absorbs, threat formulas, hit/crit mechanics, controls, power
coefficient formulas, or future catalog content. The current source commit has
no M4 `directPct` entry; the pure resolver keeps that source rule explicit
without expanding the generated catalog schema speculatively.

## Deferred Authority

The work remains entirely in the WOC `zr_vm:project` plugin project. WOS108 is
only a WOC-local state-format extension; it adds no Zircon Rust crate, engine
ABI, lifecycle change, host contract or native fallback. Dynamic ZrVM/Cargo
acceptance remains owned by Plugins08.

## Status And Output Record

| Milestone | Scope | Status | Date | Evidence |
|---|---|---|---|---|
