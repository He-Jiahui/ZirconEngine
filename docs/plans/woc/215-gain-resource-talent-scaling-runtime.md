---
title: WOS215 gainResource talent scaling runtime closure
status: completed
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS215 Gain-Resource Talent Scaling Runtime

## Source Contract

`scaleEffect` resolves `gainResource.amount` as
`Math.round(amount * dmgMult + flat)`, where `dmgMult` uses the ability school:
`1 + global.meleeDmgPct + ability.dmgPct` for physical abilities. The current
M4 `enrage` entry is a physical Druid ability with one `gainResource` effect
of 20. Feral currently supplies `global.meleeDmgPct: 0.15`, so its resolved
instant gain is 23.

## Current WOC Surface And Gap

`startOfflineEnrageCast` admits the existing Bear-form off-GCD ability and
writes the generated raw amount directly to the resource pool. It therefore
misses Feral's source-resolved immediate resource gain. The effect is instant;
it does not create an aura, projectile, queue entry or codec obligation.

## Design

1. Add a pure gain-resource resolver using the source multiplier, flat and
   positive endpoint rounding order.
2. Resolve Enrage from the caster's current `TalentModifierState` at its
   existing resource mutation point, preserving form admission, cap,
   cooldown and off-GCD behavior.
3. Add a Feral-to-unselected respec regression, source-pinned static guard and
   `zr_vm:project` fixture manifest. No Zircon engine ABI, plugin-host API or
   WOS schema change is required.

## Acceptance

- Feral Enrage grants the resolved 23 rage and an unselected recast grants the
  raw 20, bounded by the existing resource cap.
- No extra random draw, aura, cooldown, form, target or state-codec behavior
  is introduced.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| WOS215 | `gainResource` 天赋缩放运行时闭环 | completed | 2026-08-03 | `gain_resource_talent_scaling_state.zr`、WOS215 静态守卫、`zr_vm:project` fixture；独立二次审查通过，未运行动态 ZrVM/Cargo。 |
