---
title: WOS240 Startle Shot ranged incapacitate runtime
status: implementation_complete_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS240 Startle Shot ranged incapacitate runtime

## Objective

Project the source-pinned Hunter choice-granted `startle_shot` ability into
WOC M4 and close its offline control loop: level-8 instant cast, 25-resource
cost, normal GCD, 20-second cooldown, hostile 8-35 target/facing admission,
and a four-second damage-breakable incapacitate with no damage, projectile or
RNG branch.

## Source contract

- Source ability: `dev/world-of-claudecraft/src/sim/content/talent_abilities_v2_a.ts`
  at `5ef9f7cb21cd8875b6d2c49701015dfcd78de35a`.
- `startle_shot` is a Hunter ability with learn level `8`, cost `25`, cast time
  `0`, cooldown `20`, range `35`, minimum range `8`, physical school,
  `scalesWith: "ranged"`, and a hostile target requirement.
- The only effect is `{ type: "incapacitate", duration: 4 }`. It neither deals
  damage nor declares `projectile`, so the effect resolves immediately at cast
  admission without armor, critical, threat or RNG handling.
- `choice_rows_classic.ts` grants it only through `hun_r8_startle_shot`; the
  established WOC known-ability partition remains the runtime admission source.
- Source `effect_dispatch.ts` applies a plain, physical-school, break-on-damage
  incapacitate. It has no fear DR, no random facing value and uses immediate
  break-on-any-positive-damage behavior.

## Existing WOC support to reuse

- The generated M4 contract and source-pinned extractor can retain additional
  source abilities and project their effect rows.
- Existing ranged Hunter gate math already enforces target identity, hostility,
  8-35 distance and facing.
- Motion-aura rows already serialize school, kind, source, duration and the
  zero break scale required by a plain incapacitate.
- `clearOfflineBreakableIncapacitateOnDamage` and Eastbrook pursuit already
  clear or suppress a plain incapacitate correctly.

## Delivery order

1. Retain `startle_shot` at the end of the M4 generator scope and update fixed
   projection/coverage cardinalities without renumbering prior ability indices.
2. Add a Hunter-specific exact payload, profile, target gate and instant
   reducer which spends resource, applies normal GCD, arms cooldown, enters
   combat and writes one physical incapacitate motion-aura row.
3. Route slot and typed commands, keeping the ability out of all projectile,
   damage, critical, armor, threat and random-draw paths.
4. Add a `zr_vm:project` fixture covering source profile, resource/GCD/cooldown,
   aura identity/duration/school, no RNG/projectile/damage/threat, snapshot,
   damage break, pursuit suppression, geometry rejection and typed parity.
5. Run source-pinned static guards and independent second review. Dynamic
   fixture acceptance remains coordinator-owned and is not polled here.

## Non-goals

- No ZirconEngine or ZrVM implementation change.
- No talent-choice UI or progression rewrite beyond the existing known-ability
  admission boundary.
- No projectile, combat-damage, crowd-control DR or fear-direction behavior.

## Output record

| Slice | Scope | State | Date | Evidence |
| --- | --- | --- | --- | --- |
| WOS240 | Startle Shot ranged, damage-breakable incapacitate | implementation_complete_static_validation_pending | 2026-08-03 | Source-pinned catalog and generated Zr projection contain Startle Shot at index 114. The Hunter-only 8-35 gate, 25-resource instant admission, normal GCD and 20-second cooldown write one physical, 4-second, zero-scale incapacitate aura without damage, projectile, critical, armor or RNG paths. The generic positive-damage break reducer now recognizes the source ability; snapshot, AI suppression, geometry, slot and typed-command cases are covered by the `zr_vm:project` fixture. A review P2 was forward-fixed: the generated contract and coverage now pin the direct `TALENT_ABILITIES_V2_A.startle_shot` definition rather than the `classes.ts` aggregate. Source guard, generators, Node syntax and diff checks pass; the follow-up independent review found no actionable P1/P2 findings. Dynamic acceptance remains coordinator-owned. |
