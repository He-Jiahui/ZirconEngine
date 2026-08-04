---
title: WOS205 HoT talent scaling runtime closure
status: implemented_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS205 HoT Talent Scaling Runtime

## Source Contract

`applyTalentMods` resolves every `hot` effect before aura creation as
`Math.round(total * healMult * hotMult + flat)`, where `healMult` contains the
global `healPct` and the ability `dmgPct`, and `hotMult` contains global
`hotHealPct`. `effect_dispatch` then converts that resolved total to a per-tick
base with `Math.max(1, Math.round(total / (duration / interval)))`. Pure HoTs
add their spell-power tick bonus after that conversion; hybrid HoTs do not add
spell power. The resulting per-tick value is stored on the aura, so later ticks
and restored snapshots do not re-evaluate talents.

The retained source mastery regression fixes the concrete baseline: rank-one
Rejuvenation total is 116 without Restoration and 145 with its fully scaled
`hotHealPct: 0.25`. Under the WOC rank-four/150 spell-power profile this changes
the stored tick from 59 to 66: scale and round the source total first, divide
and round next, then add the unchanged spell-power bonus.

## Implementation Scope

- Add `combat/hot_talent_scaling_state.zr` as the narrow pure owner for the
  source total-resolution formula. It reads the existing
  `TalentModifierState`, including global heal/HoT modifiers and per-ability
  modifiers, and contains no `WorldState`, queue, command, snapshot, or RNG
  behavior.
- Extend the existing pure and hybrid HoT profile modules with an explicit
  resolved-total entry point. Their existing public profile functions retain
  the unmodified catalog-total behavior for callers that have no resolved
  talent state.
- At the two normal aura-creation paths in `world/state.zr`, obtain the
  caster's existing talent modifiers, resolve the source total once, and store
  the resulting profile tick. Rejuvenation and Renew use the pure profile;
  Regrowth uses the hybrid profile. The historic already-resolved Rejuvenation
  migration helper remains unchanged.
- Do not change snapshot schema or queue shape: `offlineHotHeals` already owns
  the resolved per-tick snapshot value. Do not modify direct heals, periodic
  tick behavior, threat, spell-power bonus, or native/plugin transport.

## Regression Design

- Add module tests for no-modifier identity and the source order of global
  healing plus HoT scaling.
- Extend the WorldState HoT regression with Restoration Rejuvenation (66 at
  the retained rank-four/150-power fixture), Restoration Regrowth, and Holy
  Renew. Each case checks that the queued value is resolved at application,
  survives encode/decode unchanged, ticks without RNG, and does not retroactively
  change after the talent selection changes.
- Add a focused static guard anchoring the source `hot` scale/dispatch order,
  the new module boundary, the two WOC application sites, and the absence of
  direct-heal rewrites. Use the existing `zr_vm:project` WorldState test route;
  dynamic ZrVM/Cargo acceptance remains owned by Plugins08.

## Deferred Authority

This is deterministic project-side combat behavior and requires no new engine
ABI. It must continue to use `zr_vm:project` rather than adding a native
fallback. Real plugin execution and Cargo acceptance remain deferred under the
existing Plugins08 runtime handoff.

## Status And Output Record

| Milestone | Scope | Status | Date | Evidence |
|---|---|---|---|---|
| WOS205 | HoT talent total scaling, application-time snapshot and retained ticks | implementation complete; second review complete; dynamic validation pending | 2026-08-03 | `hot_talent_scaling_state.zr` resolves `total * healMult * hotMult + flat` before profile subdivision; pure/hybrid profiles preserve source spell-power ordering and `state.zr` snapshots the resulting tick. `hotTalentRuntimeStateTest` covers Restoration Rejuvenation 66, Regrowth 9 and Holy Renew 64 across respec, restore and zero-RNG tick. `wos205_hot_talent_scaling_runtime_static_guard.mjs` passed; its fixture explicitly declares `backend: "zr_vm:project"`. Canonical plugin execution is unavailable, so no dynamic result is claimed. |
