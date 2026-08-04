---
title: WOS241 Deterrence off-GCD dodge and damage-reduction runtime
status: implementation_complete_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS241 Deterrence off-GCD dodge and damage-reduction runtime

## Objective

Project the source-pinned Hunter choice-granted `deterrence` ability into WOC
M4: level-10, zero-cost, instant, 300-second, off-GCD self cast that applies
both a 25 percentage-point dodge buff and a 30% all-damage reduction for ten
seconds.

## Source contract

- Source: `dev/world-of-claudecraft/src/sim/content/talent_abilities_v2_a.ts`
  at `5ef9f7cb21cd8875b6d2c49701015dfcd78de35a`.
- `deterrence` is a physical Hunter ability: learn level `10`, cost `0`, cast
  time `0`, cooldown `300`, range `0`, no target and `offGcd: true`.
- Its ordered effects are `selfBuff(buff_dodge, 0.25, 10)` and
  `selfBuff(buff_dr, 0.3, 10)`.
- The source selection row `hun_r17_deterrence` grants the ability. Existing
  known-ability admission remains the WOC runtime source of that grant.
- Source `selfBuff` emits two same-ability aura identities: the first carries
  the bare ability id and the companion receives a kind-suffixed id. WOC's
  motion-aura rows preserve the two effect kinds independently.

## Existing WOC support and gap

- `retainedDodgeAuraBonus` and `effectiveOfflineDodgeChance` already project
  self-owned `buff_dodge` rows and serialize/expire their duration.
- The M4 effect generator already projects both self-buff rows, and off-GCD
  admission already permits casts while an existing GCD remains active.
- `buff_dr` is present in generated contracts but has no retained motion-aura
  profile or shared offline incoming-damage reduction reader. This is a WOC
  game-layer gap to close in this slice, not a ZirconEngine capability gap.

## Delivery order

1. Retain Deterrence in the M4 source-pinned contract and update fixed
   projection/coverage cardinalities without renumbering prior ability indices.
2. Add the `buff_dr` retained aura profile and a shared live incoming-damage
   multiplier, then apply it to the current hostile direct/projectile damage
   paths that already consume other defensive mitigation.
3. Add an off-GCD Hunter reducer that preserves an active GCD, arms the
   300-second cooldown and writes independent Dodge and DR aura rows.
4. Route slot and typed commands; cover snapshot, expiry, off-GCD behavior,
   dodge/DR values, reduced damage and invalid class/level admission through a
   `zr_vm:project` fixture.
5. Run source-pinned static checks and independent second review. Dynamic
   acceptance remains coordinator-owned and is not polled here.

## Non-goals

- No ZirconEngine or ZrVM change.
- No party, pet, projectile or target-selection behavior.
- No UI talent-selection rewrite beyond existing known-ability admission.

## Output record

| Slice | Scope | State | Date | Evidence |
| --- | --- | --- | --- | --- |
| WOS241 | Deterrence off-GCD dodge and all-damage reduction | implementation_complete_static_validation_pending | 2026-08-03 | Source-pinned catalog and generated Zr projection contain Deterrence at index 115 and retain its direct `TALENT_ABILITIES_V2_A` owner. The Hunter-only level-10, zero-cost off-GCD reducer preserves an active GCD, sets the 300-second cooldown, and writes ordered 25% dodge plus 30% damage-reduction aura rows. The source-style reduction sums self-owned `buff_dr`, rounds before retained absorbs, and is wired into the live Eastbrook hostile melee path; state covers snapshot, expiry, real matching-RNG incoming damage, slot/typed parity and invalid level/class admission through a `zr_vm:project` fixture. All generators, CC contract, source guard, Node syntax and diff checks pass; independent second review found no actionable P1/P2 findings. Dynamic acceptance remains coordinator-owned. |
