---
title: WOS177 Flamestrike position runtime
status: implemented_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS177 Flamestrike Position Runtime

## Scope

Implement the generated WOS175 `flamestrike` definition as a two-second,
ground-targeted Fire Mage cast through the existing `castAt` ABI. It costs 80,
has a 12-second cooldown and resolves one Fire `aoeDamage` burst in radius 7
at a range-clamped point.

## Source transaction

- The cast locks its clamped target point, bills at successful completion, and
  does not create a persistent ground effect or channel.
- A whiff draws no RNG and produces no Hot Streak event.
- If one or more targets qualify, the cast draws exactly one shared crit unit,
  then one damage unit in target traversal order for each target. Combustion
  forces that shared crit outcome but does not suppress its normal draw.
- Every struck target shares the crit result. The Fire hit reducer runs once
  after the burst, not once per target.
- Hot Streak's instant modifier consumes when it reduces the positive cast
  time to zero; the free modifier consumes only when the instant burst or a
  completed hard cast successfully bills its cost.

## Boundary

`combat/ground_aoe_state` continues to own ordinary non-critical instant and
persistent pulse arithmetic. It cannot express a cast-wide conditional crit
draw or a one-per-cast Fire proc signal, so no speculative generic API is
introduced for this sole source behavior.

## Formula Projection

Schema 87 carries the source spell-damage multiplier and spell critical-damage
bonus as persistent per-entity combat inputs. Flamestrike applies the spell
multiplier before `1.5 + critDamageSpellBonus` and rounds only after both,
matching `effect_dispatch`. The same fields now feed the existing Fireball and
Pyroblast projectile resolver and the pending Dragon's Breath cone. Arcane
Power is the first source producer of `buff_spelldmg`; its command/aura route
remains a separate source slice, while this plan owns the consumers and
snapshot-safe input boundary.

## Delivery order

1. Add source-pinned profile, exact payload, aim clamping and WOS177 static
   test entry.
2. Add timed and instant cast paths plus completion dispatch and cooldown.
3. Add the dedicated shared-crit burst transaction, target traversal, threat
   and exactly-once Fire reducer call.
4. Perform static checks and a second review; defer candidate execution only
   to `zr_vm:project`.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| WOS177a | Flamestrike position and shared-crit runtime | implemented_static_validation_pending | 2026-08-01 | WOS177 guard, schema 87 formula input/restore coverage |

## 二次审查

2026-08-01: independent review confirmed castAt aim routing, clamped-point
snapshot symmetry, whiff RNG preservation, shared crit sequencing and delayed
free-cost consumption. It found that the direct burst had bypassed source
`spellDamageMultFromAuras` and `critDmgSpellBonus`; the forward schema-87
projection and source-order formula were added before this record. The focused
ZrVM package remains pending `zr_vm:project` availability.
