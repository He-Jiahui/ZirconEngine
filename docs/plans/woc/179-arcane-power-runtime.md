---
title: WOS179 Arcane Power runtime
status: implemented_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS179 Arcane Power Runtime

## Scope

Implement Mage `arcane_power` (Aether Surge) as the first source producer for
the schema-87 spell-damage multiplier and the existing Mage spell-haste
consumer. The bounded offline projection retains only the single player and
does not broaden the generic M4 or motion-aura contracts.

## Source Transaction

- A level-five Mage may cast Arcane Power with no resource cost, zero cast
  time, 90-second cooldown, Arcane school, and a normal GCD.
- The source computes that GCD from the pre-cast `spellHasteMult`, then applies
  both self auras for ten seconds: `buff_spelldmg = 0.2` and
  `buff_spellhaste = 0.1`.
- The live multiplier is `1.0 + spell-damage bonuses`; the retained current
  owner is Arcane Power, so its activation projects `1.2` and expiry restores
  `1.0`. All retained `TimedSpellSpec` magic-hit paths read that shared
  damage/critical modifier boundary. Haste is read dynamically by
  `mageSpellHasteMultiplier`, preserving the baseline stat column and
  affecting only casts/GCDs armed while active.
- The remaining duration is snapshot-persistent. Existing snapshots migrate
  to an inactive default, and the normal fixed tick expires the projection
  without creating a second timer or consuming RNG.

## Delivery Order

1. Add source-profile helpers, durable Arcane Power duration, schema migration
   and deterministic expiry projection.
2. Route typed and slot casts through normal school-lockout, cooldown, GCD and
   known-ability admission without changing existing Fire paths.
3. Add a focused ZrVM package plus source-pinned static guard that checks
   pre-buff GCD order, activation, expiry and snapshot restoration.
4. Perform static checks and a second review. Dynamic evidence remains
   exclusively `zr_vm:project`; unavailable execution delays acceptance only.

## Boundaries

- Do not treat a current explicit source ability as a generic aura-system
  refactor; future producers may converge only when a second source case needs
  their shared representation.
- Do not use Cargo, native host execution, a local replacement VM or a
  handcrafted fallback as test evidence.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| WOS179a | Arcane Power activation, persistence and expiry | implemented_static_validation_pending | 2026-08-01 | WOS179 guard; WOS176-178 regressions; command/M4/cone contract checks |

## 二次审查

2026-08-01: post-implementation review verified the source's pre-aura GCD
calculation, normal Arcane lockout admission, typed/slot routing, 90-second
cooldown, schema-88 tail ordering, v87 migration and fixed-tick expiry. It
found that the schema-87 multiplier had previously been consumed by only the
four Fire paths. The existing common `TimedSpellSpec` boundary was therefore
forward-wired to all fourteen retained magic-hit resolvers, including the
shared spell critical-damage input, before this record. Multi-aura individual
cancellation and non-`TimedSpell` periodic/generic effects remain the next
source-owned convergence slice. The focused ZrVM package remains pending
`zr_vm:project`; no alternate runtime was used.
