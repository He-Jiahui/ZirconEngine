---
title: WOS192 Power Echo runtime
status: planned
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS192 Power Echo Runtime

## Scope

Replicate Mage row-14 `power_echo`: an off-GCD, zero-cost, 30-second Arcane
spell grants one 10-second, 0.5-value aura. Its first resolved direct spell
damage or direct spell healing consumes that aura before producing a copy.
The copy is `max(1, round(resolved * 0.5))`, never re-arms Power Echo and adds
no new RNG draw. A damage copy leaves the caster as an actual second projectile
and fizzles if the target dies in flight; a healing copy is immediate and
cannot crit or trigger a second on-heal weapon proc.

This requires WOS-owned durable aura state and a narrow fixed-damage
projectile profile. It must not repurpose the random min/max projectile rows:
the source copies the already resolved post-crit amount exactly. Direct healing
integration covers Temporal Mend and Temporal Echo when those source paths are
reachable, including their independent post-heal effects on only the primary
resolution.

## Source Contract

- `classes.ts` defines `power_echo` at level 14 with cost 0, cast time 0,
  cooldown 30, Arcane school, off-GCD, `selfBuff(power_echo, 0.5, 10)`.
- In `effect_dispatch.ts`, direct spell damage and direct spell healing find
  and remove the aura before copying; the damage copy is scheduled as a real
  projectile using the original critical outcome and amount, while the heal
  uses the non-crit/non-proc apply-heal call.
- A non-direct spell does not consume the aura. No Power Echo operation draws
  an additional random value.
- `mage_choice_rows.test.ts` proves damage amount, direct-heal amount,
  no-recurse semantics and primary-only downstream effects.

## Delivery Order

1. Add a focused `zr_vm:project` entry and red static/source contract for
   aura lifecycle, original amount reuse, RNG neutrality, fixed projectile
   state and direct-heal routing.
2. Add source-pinned ability profile, off-GCD reducer and serialized aura
   state, including expiration and exact talent admission.
3. Extend the bounded projectile row with a canonical fixed-damage profile;
   consume the aura at direct landing/resolve boundaries and route typed and
   slot casts.
4. Cover direct damage, direct healing, target-death fizzles, snapshot restore
   and no recursive/proc/RNG side effects. Run static regression and a second
   independent review. Dynamic execution remains exclusively `zr_vm:project`.

## Dynamic Validation

The WOS192 package must run only through `zr_vm:project`. No alternate runtime
is permitted.
