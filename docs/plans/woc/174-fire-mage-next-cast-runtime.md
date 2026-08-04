---
title: WOS174 Fire Mage next-cast runtime foundation
status: implemented_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS174 Fire Mage Next-Cast Runtime Foundation

## Source contract

Current-head `combustion` (Phoenix Trance) is a Fire Mage, off-GCD self buff:
it costs 100, lasts 10 seconds and has a 120-second cooldown. While active it
overrides Fire spell crit results but does not skip the pre-existing crit RNG
draw; its builder crits still advance Hot Streak. A builder crit outside the
window reduces the live Combustion cooldown by one second.

Two sequential Fire builder crits consume Heating Up and create a 12-second
pair: `hot_streak` (`next_cast_free`) and `hot_streak_instant`
(`next_cast_instant`). Both target only Pyroblast and Flamestrike. The source
spends the pair when the eligible cast resolves through its normal
`empower_next` handling. The two source aura identities have matching duration
but separate semantics, so the candidate must preserve free and instant as
independent modifier flags even when Hot Streak creates both together.

## Architecture note

This remains a WOC script-state leaf. `combat/fire_mage_state` owns deterministic
Fire decisions and `combat/empower_next_state` owns modifier lookup/consume
rules. `WorldState` must own a compact, entity-partitioned next-cast modifier
store, durability, expiry and the cast-admission bridge. Combustion and Hot
Streak then become ordinary producers of that store; Dragon's Breath is an
ordinary post-batch producer/consumer, never a special-cased Fire route.

The persisted row is canonical by opaque nonzero `identityCode` within an
entity, `kindCode` (`free`, `instant`, or cost multiplier), remaining seconds,
and value. A second sorted ability-code partition is its explicit scope; an
empty scope applies globally. Reapplying an identity replaces the entire row.
A successful scoped free/instant/cheap cast consumes the entire modifier row,
not just one target code. This preserves Hot Streak's two independent rows
while also supporting non-Fire next-cast producers without text identities.

## Implementation order

1. Define one compact persisted next-cast modifier row shape: source identity,
   modifier kind (free/instant/cheap), scalar value, an explicit target-ability
   partition and expiry. Add default rows, validation, snapshot v85 tail and
   old-snapshot zero migration. Do not encode target names or add one field per
   ability.
2. Make normal cast admission and cost/cast-time resolution query and consume
   those rows via the existing pure `empower_next_state` contract. Verify an
   eligible Pyroblast/Flamestrike consumes both Hot Streak rows together; an
   unrelated spell consumes neither.
3. Add Combustion as an off-GCD Fire self buff with real cost/cooldown/duration
   state, preserving the normal crit draw before its override and applying the
   one-second cooldown reduction only for resolved builder crits outside the
   active window.
4. Let one post-batch Fire helper update Heating Up/Hot Streak and then enable
   Dragon's Breath through the existing empowered lifecycle and fear resolver.

## Boundaries

- No hardcoded `hasCombustion = false`; until step 3 is present, Dragon remains
  disabled.
- PvP fear DR requires a real enemy-player relation. The current offline
  `entityHostile` flag represents NPC hostility and must not be repurposed for
  duels or arena teams. NPC targets keep full fear duration exactly as source.
- The authoritative dynamic path remains `zr_vm:project`; static completion
  never constitutes candidate transaction acceptance.

## Verification

- Source anchors pin Combustion's ability data, `fireMageOnSpellHit` and
  `empower_next` application/consumption ordering.
- Pure tests cover modifier eligibility and Fire transitions with no RNG.
- WorldState tests cover v85 migration, pair consumption, off-GCD timing,
  cooldown reduction, snapshot restore and final Dragon batch ordering.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| WOS174a | 通用 next-cast 规则层和 v85 持久化分区 | in_progress | 2026-08-01 | `combat/next_cast_modifier_state`、`WorldState` v85、静态守卫 |

## 二次审查

2026-08-03: second static review confirmed the canonical next-cast row shape,
strict target partitions, free/instant consumption on an eligible successful
cast, v85 default migration, Combustion's off-GCD state and post-hit ordering.
The focused Fire-proc and next-cast world-state packages now explicitly bind
`zr_vm:project`; `node examples/woc/tools/wos174_fire_mage_next_cast_static_guard.mjs`
passed. Dynamic ZrVM execution remains pending and is not replaced by a host
runtime.
