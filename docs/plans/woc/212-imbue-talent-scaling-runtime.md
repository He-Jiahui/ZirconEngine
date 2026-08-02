---
title: WOS212 imbue talent scaling runtime closure
status: implementing
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS212 Imbue Talent Scaling Runtime

## Source Contract

`applyTalentModifiers` resolves `imbue` fields as
`Math.round(bonus * dmgMult + flat)` and, when present,
`Math.round(judgeMin * dmgMult + flat)` / `Math.round(judgeMax * dmgMult +
flat)`. `dmgMult` uses physical `meleeDmgPct` only for physical schools and
`spellDmgPct` otherwise, plus per-ability `dmgPct`; `flatDmg` is added once to
each endpoint.

The effect dispatcher writes those resolved values into the active `imbue`
aura (`value`, `value2`, `value3`) at cast time. White swings consume `value`;
Judgement consumes the stored `value2`/`value3` range and then removes the
aura. A later respec cannot reinterpret an already-active aura.

## Current WOC Surface And Gap

The M4 projection has three imbues: Seal of Righteousness (holy bonus plus
Judgement range), Flametongue Weapon (fire bonus) and Frostbrand Weapon
(frost bonus). WOC currently persists only active imbue ability/rank/remaining
and recomputes raw catalog bonus/Judgement values on swing or Judgement. That
incorrectly changes an active imbue after respec and cannot verify restored
endpoints exactly.

## Design

1. Add a pure imbue resolver for source rounding, school selection and optional
   Judgement endpoints. It owns no aura replacement, RNG, attack, spell power,
   crit, threat or cooldown behavior.
2. Extend the WOC-local format to WOS109 with per-entity resolved imbue
   bonus/Judgement values and the six-row allocation snapshot that produced
   them. On applying any imbue, resolve and persist all fields at the existing
   cast point; clearing/replacing it clears the values and snapshot.
3. Route `offlineImbueBonus` and Judgement’s numeric input through persisted
   resolved values. Modern validation recomputes values from the stored
   snapshot and rejects altered rows. WOS108 and earlier normalize active
   imbues to their generated raw endpoints with empty snapshots; older writers
   refuse a non-empty WOS109 imbue snapshot.
4. Keep `zr_vm:project` as the fixture backend. This is a game-project state
   extension only: no Zircon engine ABI, plugin host API, generated catalog
   schema or native runtime capability is needed.

## Acceptance

- Active Seal, Flametongue and Frostbrand values remain fixed through a later
  respec and encode/decode; the next cast uses the new selection.
- Judgement rolls from the stored resolved Seal range before its existing
  spell-power/crit steps and consumes the imbue once.
- Legacy base rows remain decodable, while historical encoders never discard a
  required selection snapshot.
- Existing RNG, hit, spell-power, crit, resource, aura replacement and threat
  ownership remain unchanged.
