---
title: WOS194 Critical spell impact runtime
status: implemented_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS194 Critical Spell Impact Runtime

## Scope

Complete the source `dealDamage` critical-only post-impact behavior at the
WOC fixed-spell impact boundary. WOS192 already carries an already-rolled
damage amount and the original critical boolean through snapshot-safe projectile
state; this milestone makes that boolean observable to the source-derived
rules without rerolling a critical or replaying source `noteSpellHit` effects.

The initial implementation covers the source behaviors presently represented
by WOC's retained combat modules:

- the largest active target `critvuln` fraction scales each retained hostile
  spell critical impact, after primary-roll damage and before absorbs;
- a committed Fire Mage's hostile Fire ability critical invokes the existing
  `fire_mage_state.zr` Ignite planner, adding/refreshing its source-owned,
  stacking 6-second/2-second DoT;
- Ignite ticks carry `critical = false`, never refresh Ignite and never cause
  a spell-hit/Hot-Streak/weapon-proc replay.

This path owns the current normal and fixed projectile arrivals uniformly. It deliberately
does not call `applyOfflineFireMageSpellHit`: source dispatch invokes that
primary-cast hook once after scheduling the Power Echo projectile, not once per
copied impact.

## Source Contract

- `damage.ts` applies the maximum positive `critvuln` aura value only when
  `crit` is true, source and target differ, and the damage is positive. The
  multiplier happens before absorb shields.
- `damage.ts` calls `igniteOnCrit` for a hostile Fire-school ability critical;
  the burn is based on the post-modifier resolved amount, stacks with the same
  source's existing Ignite and refreshes the source six-second/two-second
  cadence. A tick passes `crit = false`.
- `heal.ts` defines `critVulnBonus` as the largest active `critvuln` aura.
- WOC's existing `fire_mage_state.zr` already pins Ignite eligibility,
  `round(amount * ignitionPct)`, three-tick rounding and stacking plans; it
  has not yet been connected to world-state mutations.

## Delivery Order

1. Add a red source/static contract and focused `zr_vm:project` entry for
   primary and Power Echo fixed arrivals with/without critvuln, non-critical
   rejection, Fire ability eligibility, one-time Ignite creation/stack/refresh,
   and non-recursive tick behavior.
2. Add bounded, versioned world state for target critvuln auras and Ignite
   ownership/value/remaining/timer. Reuse the established DoT tail only when
   it can represent exact source ownership and stacking; otherwise define a
   narrow Ignite row rather than overloading unrelated periodic effects.
3. Move `resolveOfflineFixedSpellImpactDamage` from its current WOS192 relay
   into the shared crit-aware spell impact reducer. Route ordinary spell
   projectiles and WOS192 fixed projectiles through it before absorbs, without
   changing hit-table RNG or invoking `applyOfflineFireMageSpellHit`.
4. Connect `fireMage.igniteOnCritPlan` and `fireMage.ignitePlan` to source
   Fire specialization/talent admission, apply/refesh the durable Ignite row,
   and advance ticks at the source cadence with `critical = false`.
5. Cover snapshot restore, target death, target/source replacement, multiple
   critvuln auras, equal-value ordering, Power Echo amount preservation, exact
   RNG neutrality and no duplicate Hot Streak/weapon proc. Run static
   regression and an independent second review.

## Exclusions

- Critical modifiers not yet represented by a WOC source slice (Berserker
  critical damage, battle-stance mastery, PvP scaling and later encounter
  mechanics) require their own source/payload milestones before being added to
  this common boundary.
- No ZirconEngine or ZrVM feature is missing. This is WOC gameplay state and
  source-rule wiring only.

## Dynamic Validation

`examples/woc/scripts/woc_game/woc_m4_critical_spell_impact_runtime_tests.zrp`
must run only through `zr_vm:project`. No alternate runtime is permitted.

## Second Review

2026-08-03: second static review confirmed the crit-aware shared spell-impact
boundary, maximum crit-vulnerability selection, Fire-only Ignite planning and
non-critical Ignite ticks without duplicate primary spell-hit side effects.
The focused fixture declares `zr_vm:project`, and
`node tools/wos194_critical_spell_impact_runtime_static_guard.mjs` passed from
`examples/woc`. Dynamic ZrVM execution remains pending.
