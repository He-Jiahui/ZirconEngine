---
title: WOS214 judgement talent scaling runtime closure
status: completed
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS214 Judgement Talent Scaling Runtime

## Source Contract

`applyTalentMods` transforms the `judgement` effect at cast resolution as
`dmgMult: (eff.dmgMult ?? 1) * dmgMult` and `flat: (eff.flat ?? 0) + flat`.
For Judgement's Holy school, `dmgMult` is `1 + global.spellDmgPct +
ability.dmgPct`. `effect_dispatch` consumes the active Seal first, then
computes `range(seal.value2, seal.value3) * eff.dmgMult + eff.flat +
directHitBonus`; spell critical and the final rounding remain downstream.

The Seal range itself remains the WOS109 application-time snapshot. The
Judgement effect multiplier and flat are instead evaluated from the caster's
current talent selection because the source resolves the current ability just
before its instant dispatch.

## Current WOC Surface And Gap

WOC's WOS212 path correctly supplies stored Seal endpoints to
`effect_numeric_dispatch_state`, but its Judgement dispatcher currently applies
only the raw generated effect. It has no input for the source cast-time
`dmgMult`/`flat` transform, so later selection changes cannot affect the
Judgement effect while they correctly leave the consumed Seal unchanged.

## Design

1. Add a pure `combat/judgement_talent_scaling_state.zr` resolver for a
   school-aware damage multiplier and flat contribution. It owns no state,
   random draw, Seal lifetime, spell power, critical, threat or target logic.
2. Extend only the transient numeric-dispatch projection with multiplier and
   flat inputs whose defaults preserve existing callers. Apply them after the
   existing Seal range draw and before direct spell-power bonus, critical and
   final rounding, matching the source expression exactly.
3. Populate those inputs at `startOfflineJudgementCast` from the caster's
   current `TalentModifierState`; keep WOS212's stored Seal endpoints and the
   existing consume-before-draw ordering unchanged. No new WOS bytes or schema
   version are required.
4. Add a pure resolver contract, numeric-dispatch regression, WorldState
   respec regression, source-pinned static guard and `zr_vm:project` manifest.
   Static checks cover source ordering and generated effect identity; ZrVM and
   Cargo execution remain out of scope for this session.

## Acceptance

- Judgement applies current Holy spell/ability damage percentage and flat once
  to the retained Seal range before spell power and critical.
- A respec after applying Seal changes the next Judgement transform but not the
  Seal's stored endpoints.
- Seal removal, RNG draw count/order, cooldown, resource, threat and WOS110
  codec compatibility remain unchanged.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| WOS214 | Judgement 施放时天赋缩放 | completed | 2026-08-03 | `wos214_judgement_talent_scaling_runtime_static_guard.mjs` 通过；二次审查确认 source 变换、Seal 消耗顺序、respec 与 RNG 契约 |
