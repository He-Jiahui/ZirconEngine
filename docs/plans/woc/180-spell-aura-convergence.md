---
title: WOS180 spell aura convergence
status: implemented_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS180 Spell Aura Convergence

## Scope

Complete the next Arcane Power source boundary that WOS179 deliberately did
not hide: its independently cancellable `buff_spelldmg` and
`buff_spellhaste` aura ids, plus spell-damage projection for retained periodic
and non-`TimedSpell` direct magic effects.

## Source Transaction

- `effect_dispatch` emits Arcane Power's primary `arcane_power` spell-damage
  aura and its distinct `arcane_power_buff_spellhaste` companion. Generic
  `cancel_aura` may remove either one without refunding its cooldown or
  restoring the other.
- The source reads active spell-damage auras for direct and AoE magic hits.
  It snapshots pure DoT base/spell-power values into the aura, so later
  periodic ticks do not reapply `buff_spelldmg`. Expiry/cancellation changes
  only the affected direct/AoE contribution; spell haste continues to govern
  future cast/GCD arming.

## Delivery Order

1. Split WOS179's paired duration into snapshot-safe source aura identities
   and route exact primary/companion cancellation.
2. Audit retained periodic and non-`TimedSpell` magic damage boundaries;
   retain the source's unmodified DoT snapshots and project only direct/AoE
   magic hits. This slice covers Glacial Front and Arcane Explosion without
   changing physical effects or random-draw order.
3. Add focused ZrVM tests and source-pinned static guards, then perform a
   second review. Dynamic evidence remains exclusively `zr_vm:project`.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| WOS180a | Independent Arcane Power aura and periodic convergence | implemented_static_validation_pending | 2026-08-01 | `wos180_spell_aura_convergence_static_guard.mjs`; command/M4/empowered-cone generator checks; `zr_vm:project` pending |

## Secondary Review

Reviewed against source commit `5ef9f7cb21cd8875b6d2c49701015dfcd78de35a` after
implementation. `cancelAura` removes the primary `arcane_power` and the
companion `arcane_power_buff_spellhaste` by separate identities; the retained
state now preserves that separation through schema 89 and leaves cooldowns
unchanged. The audit also found and forward-fixed the two remaining retained
non-`TimedSpell` spell-hit projections: Glacial Front now supplies the shared
spell multiplier and spell critical-damage bonus, while Arcane Explosion
passes the multiplier to its existing non-physical AoE reducer. Both retain
their existing random-draw order. No further static review findings remain.

Dynamic acceptance remains deferred solely because the required
`zr_vm:project` backend is unavailable; no alternate runtime was substituted.
