---
title: WOS210 drain-tick talent scaling runtime closure
status: planned
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS210 Drain-Tick Talent Scaling Runtime

## Source Contract

`applyTalentModifiers` resolves each `drainTick` endpoint before combat as
`Math.round(endpoint * dmgMult)`. `dmgMult` is `1 + globalDmg +
ability.dmgPct`, selecting `meleeDmgPct` only for the physical school and
`spellDmgPct` otherwise. Unlike `directDamage`, `dot`, and `aoeDamage`, this
path has no `flatDmg` term and no `dotDmgPct` multiplier.

`casting_lifecycle.ts` calls `resolvedAbility` for every channel tick, then
draws once from the already-resolved interval, adds the existing channel spell
power bonus, rounds, deals non-critical damage, and heals the caster by
`Math.round(damage * healFrac)` if alive. A new selection therefore affects a
later tick, but cannot alter a pending tick that has already been scheduled.

## Current WOC Surface

The M4 projection has two retained `drainTick` paths, both shadow channels:

- Drain Life: ranks 1-3, 7/12/17 base endpoint, `healFrac = 1`, five ticks.
- Mind Flay: rank 1, 12 base endpoint, `healFrac = 0`, three ticks.

WOC represents a launched channel tick as an asynchronous projectile. Its
existing WOS107 projectile talent-selection tail already preserves six
selection rows. WOS106 and earlier writers reject a non-empty tail; WOS108
retains the same projectile tail while adding the separate DoT total/selection
tail. The projectile tail must be captured for a `drainTick` tick at launch,
since a respec during travel must not reinterpret its resolved endpoints at
landing.

## Design

1. Add a pure `drain_tick_talent_scaling_state.zr` resolver for school/global
   plus per-ability damage percentages and exact positive endpoint rounding.
   It deliberately does not read WorldState, consume RNG, apply spell power,
   heal, settle threat, or add a flat/periodic modifier.
2. Make `drainTick` eligible for the established projectile selection
   snapshot. At each Drain Life or Mind Flay tick launch, resolve endpoints
   from the live validated allocation, append the existing launch snapshot,
   and retain the current projectile timing/RNG layout.
3. At landing, preserve catalog shape guards (ability, rank, heal fraction,
   school and cast time), recompute the only legal interval from the stored
   snapshot, and reject tampered endpoints. An empty snapshot remains the
   historical base interval for legacy-compatible data.
4. Reuse the WOS107 projectile snapshot in both WOS107 and the current WOS108
   format, preserving the WOS106-and-earlier rejection of non-empty snapshots.
   No state field, engine ABI, generated catalog schema, plugin host API, or
   native fallback is required.
5. Add a `zr_vm:project` fixture and source-pinned Node guard for formula
   parity, no-flat behavior, both launch sites, snapshot restoration and
   source per-tick selection. Static checks are the only acceptance evidence
   until dynamic ZrVM execution is explicitly authorized.

## Exclusions

- `directDamage`, `dot`, `aoeDamage`, `aoeHeal`, `aoeRoot`, weapon damage,
  finishers, absorbs, direct healing, crit/resist, spell-power coefficients,
  threat and cast/channel cadence remain their current owners.
- No new engine feature request is warranted: `zr_vm:project` manifests and
  the WOS107 projectile state tail cover the required runtime boundary.

## Acceptance

- Drain Life and Mind Flay endpoints exactly match source talent rounding and
  exclude `flatDmg` and `dotDmgPct`.
- A selection change before the next channel tick changes that tick's launch
  interval; a selection change after launch cannot change its landing interval.
- Each landing still has one range RNG draw, then the pre-existing channel
  spell-power bonus and exact existing heal/threat semantics.
- WOS107 round-trips required snapshot/endpoints; WOS106 cannot silently
  discard a required snapshot.
