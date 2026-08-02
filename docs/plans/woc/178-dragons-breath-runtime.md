---
title: WOS178 Dragon's Breath runtime
status: implemented_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS178 Dragon's Breath Runtime

## Scope

Finish the Fire Mage `dragons_breath` path that WOS172 intentionally deferred
and WOS173 made durable. The existing empowered-cast command (`releaseEmpowered`),
four-stage cast clock, Fire Heating Up/Hot Streak state, and fear DR rows remain
the only owners of their respective concerns.

## Source transaction

- Fire Mage, level 14, 90 resource, 2.4-second four-stage empowered cast and
  20-second cooldown.
- The source stage data is `(range, angle, min, max, incapacitate)` =
  `(6,55,32,40,1)`, `(8,65,48,60,1.5)`, `(10,78,68,82,2)`, and
  `(12,90,90,110,3)`.
- Eligible hostile Eastbrook targets are traversed in retained entity order;
  invalid, dead, out-of-cone and unavailable-line-of-sight targets consume no
  RNG. Each struck target consumes one normal spell-crit draw and one damage
  draw. Stage four forces the resolved crit but retains that normal crit draw.
- Damage happens before damage-breakable incapacitation. Targets killed by the
  hit receive no incapacitation, combat/threat stays on the existing hit path,
  and retained Eastbrook targets keep the full source duration because fear DR
  only applies to hostile player pairs.
- The Fire spell reducer receives exactly one post-batch critical result: a
  Dragon's Breath with one or more hits advances Heating Up/Hot Streak once;
  a whiff does not advance it or consume a crit/damage draw.
- Although it is outside the generic M4 table, Dragon's Breath retains its
  Fire school identity for cast admission, release interruption and active
  school-lockout cancellation.

## Delivery order

1. Add exact source-profile and known-ability helpers, then route cast slot,
   cast command, release command and natural empowered completion by ability
   identity without changing the existing Glacial Front route.
2. Reuse `empowered_cone_state` for stage geometry and hit arithmetic, apply
   the existing `fear_dr_state` decision and durable breakable-incapacitate
   rows after damage, and call the WOS176 Fire reducer once per resolved batch.
3. Add a focused project ZrVM test package and a source-pinned static guard.
   Candidate execution remains exclusively `zr_vm:project`.
4. Perform static checks and a second review. Deferred ZrVM availability delays
   acceptance only; it does not stop the next source-owned implementation slice.

## Boundaries

- Do not alter `combat/empowered_cone_state`, `combat/fear_dr_state`, or the
  generic M4 catalog merely to absorb this one ability.
- The retained Eastbrook outdoor projection has no dynamic occluders, matching
  the source's clear line-of-sight result for that world; delve and arena
  geometry remain outside this bounded runtime slice.
- No local Cargo, native host, alternate VM, or handcrafted fallback counts as
  dynamic validation evidence.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| WOS178a | Dragon's Breath command and resolved Fire cone | implemented_static_validation_pending | 2026-08-01 | WOS178 guard; WOS172/176/177 regression guards; payload and generated-contract checks |

## 二次审查

2026-08-01: post-implementation source review compared command entry, release,
natural completion, per-target RNG, formula order, death/control sequencing,
DR and Hot Streak batching with current-head. It found that this non-M4 ability
would otherwise resolve as Physical in the generic school mapper, bypassing a
Fire lockout. The forward identity mapping and a focused denied-cast assertion
were added before this record. The ZrVM package remains pending
`zr_vm:project` availability; no alternate runtime was used.
