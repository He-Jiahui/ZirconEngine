---
title: WOS206 direct-heal talent scaling runtime closure
status: planned
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS206 Direct-Heal Talent Scaling Runtime

## Source Contract

`applyTalentMods` resolves every `heal` range before numeric dispatch as
`Math.round(value * healMult + flat)`, where `healMult` is `1 + global.healPct
+ ability.dmgPct`. A heal-only `consumeAura` uses the same range rule; its
flat modifier is added once. Range selection then consumes its one existing
authoritative draw, and `heal_state` owns critical healing, absorbs, caps and
threat without receiving another talent multiplier.

Holy's current `healPct: 0.2` turns rank-two `heal` from `230..270` into
`276..324`. The range endpoints are rounded before the existing random
interpolation, rather than scaling a post-roll result.

## Implementation Scope

- Add a pure `combat/direct_heal_talent_scaling_state.zr` owner for the
  source range-endpoint formula. It accepts only raw endpoints, the existing
  `TalentModifierState` and an ability ID; it does not own WorldState, RNG,
  critical hits, persistence or threat.
- In the shared `applyOfflineDirectHeal` path, obtain the caster modifiers
  once after numeric dispatch has consumed the original range draw. Replace
  only the raw range component with the same interpolation over the resolved
  integer endpoints, retaining its direct-heal spell-power contribution and
  leaving the crit draw count unchanged.
- Cover generated `heal` effects and the existing friendly `consumeAura`
  healing path. Do not alter damage-only consume-aura behavior, direct damage,
  dots, HoTs, absorbs, cooldown/cost modifiers or added-effect dispatch; they
  remain separately scoped follow-up work.
- Add focused module and WorldState regression entrypoints plus a static guard
  anchored to the source `heal`/`consumeAura` scale rules, one-draw range
  preservation and the `zr_vm:project` test manifest.

## Deferred Authority

This is project-side deterministic combat behavior. It uses only
`zr_vm:project`; no native fallback, ABI change or snapshot schema migration is
permitted. Dynamic project execution and Cargo acceptance remain assigned to
the existing Plugins08 runtime owner.

## Status And Output Record

| Milestone | Scope | Status | Date | Evidence |
|---|---|---|---|---|
