---
title: WOS211 physical talent scaling runtime closure
status: implemented_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS211 Physical Talent Scaling Runtime

## Source Contract

`applyTalentModifiers` derives physical `dmgMult` from
`1 + meleeDmgPct + ability.dmgPct` and `flat` from `ability.flatDmg`.

- `weaponDamage`: `bonus = Math.round(bonus * dmgMult + flat)`.
- `weaponStrike`: `bonus = Math.round(bonus * dmgMult + flat)` and
  `weaponMult = authoredWeaponMult * dmgMult`, with no additional rounding on
  the multiplier.
- `finisherDamage`: `base = Math.round(base * dmgMult + flat)` and
  `perCombo = Math.round(perCombo * dmgMult)`. Variance and attack-power
  contribution remain unscaled.

Queued on-next-swing actions are resolved by `auto_attack.ts` at the swing,
not when queued. Instant weapon strikes and finishers are resolved when their
effect dispatches. A respec before a queued swing therefore changes that
swing; no historical selection snapshot is source-correct for this surface.

## Current WOC Surface

The retained M4 projection has:

- queued `weaponDamage`: Heroic Strike, Raptor Strike and Maul;
- immediate `weaponStrike`: Sinister Strike, Backstab, Stormstrike, Rake and
  Claw; Backstab is the only retained authored `weaponMult` other than one;
- immediate `finisherDamage`: Eviscerate and Ferocious Bite.

WOC currently keeps queued action identity/cost in durable WorldState and
resolves its catalog bonus in `auto_attack_state.zr` on every swing. It uses
the same raw catalog endpoints in five weapon-strike reducers and two
finisher reducers. All nine paths already own hit/crit/RNG/armor/combo/cost,
threat and form behavior.

## Design

1. Add a pure physical talent resolver that returns the exact resolved queued
   bonus, weapon-strike bonus/multiplier and finisher base/per-combo pair. It
   owns no WorldState, target, RNG, attack power, hit, crit, armor, combo or
   threat logic.
2. Extend only the transient `AutoActor` bridge so `prepareOfflineAutoActor`
   resolves a queued `weaponDamage` bonus from the current valid allocation
   immediately before `consumeQueuedSwing`. The durable queued row remains an
   ability/cost intent, preserving source respec-at-swing semantics.
3. Route every immediate `weaponStrike` through the pure resolver before its
   existing `meleeSwing` call. Route both finisher base/per-combo values before
   their existing variance, attack-power, crit and armor sequence. Do not move
   any random draw or target/combopoint transition.
4. Add a `zr_vm:project` fixture and source-pinned static guard covering all
   ten retained ability IDs, physical-only field selection, rounding order,
   Backstab multiplier scaling, queued live-resolution timing, and finisher
   exclusions. Static checks are the only acceptance evidence until dynamic
   ZrVM execution is explicitly authorized.

## Exclusions

- Imbues/Judgement, white attacks, attack power, weapon profiles, crit,
  armor, hit/dodge, procs, target ordering, threat, form rules, resource cost
  and every nonphysical source effect remain with their existing owners.
- No WOS schema, plugin host API, Zircon engine ABI, generated catalog schema
  or native fallback is required.

## Acceptance

- Queued actions use the allocation active at their landing swing and need no
  selection snapshot; an immediate action uses the allocation active at its
  existing dispatch point.
- All resolved physical values match source endpoint rounding, flat handling
  and weapon multiplier composition.
- Existing authoritative RNG count/order and all side-effect ownership remain
  unchanged.

## Status Record

| Milestone | Scope | Status | Date | Evidence |
|---|---|---|---|---|
| WOS211 | Queued weapon damage, immediate weapon strikes and finishers | implementation complete; second review complete; dynamic validation pending | 2026-08-03 | `physical_talent_scaling_state.zr` mirrors the source bonus/multiplier/base/per-combo rules; `prepareOfflineAutoActor` resolves queued intent at swing time and all five strikes plus two finishers use the pure boundary at dispatch. The regression checks active-selection timing and Backstab multiplier precision. `wos211_physical_talent_scaling_runtime_static_guard.mjs` passed; the fixture uses `zr_vm:project`. Canonical plugin execution remains unavailable, so no dynamic result is claimed. |
