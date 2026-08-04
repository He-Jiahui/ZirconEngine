---
title: WOS175 Fire Mage ability content projection
status: implemented_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS175 Fire Mage Ability Content Projection

## Scope

Add the existing source definitions for Pyroblast, Flamestrike, and Combustion
to the deterministic M4 ability projection. These map cleanly onto the current
generated form: `directDamage`, `aoeDamage`, and `selfBuff` respectively.

Dragon's Breath remains outside this content-only slice. Its `empoweredCone`
shape has angle/stage/fear parameters that the generic M4 effect projection does
not yet own; WOS172's specialized contract remains the correct upstream bridge.

## Delivery order

1. Extend the source-pinned M4 retained ability list and generated catalog
   cardinality from 79 to 82, then regenerate JSON and Zr projections.
2. Add Fire-only admission and cast helpers that reuse the existing projectile,
   ground-area, cooldown and RNG transactions.
3. Connect WOS174's next-cast partition at eligible resource/cast-time and
   success-consumption points, preserving the normal crit draw.
4. Add the Combustion off-GCD state and post-hit Fire proc reducer, then enable
   Dragon's Breath with the specialized cone and fear contracts.

## Boundaries

- Do not generalize `empoweredCone` into M4 merely to admit one ability.
- Generated artifacts are written only through their checked-in generators.
- Runtime acceptance remains deferred to `zr_vm:project`; static checks are not
  a substitute for candidate execution.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| WOS175a | Fire 三能力的受管 M4 内容投影 | implemented_static_validation_pending | 2026-08-01 | `m4_ability_codegen`、生成产物、WOS175 静态守卫 |

## 二次审查

2026-08-03: second static review confirmed that the source-pinned Pyroblast,
Flamestrike and Combustion definitions remain in the 96-entry generated M4
catalog with their intended `directDamage`, `aoeDamage` and `selfBuff` shapes.
`node examples/woc/tools/wos175_fire_mage_ability_content_static_guard.mjs`
passed. Dynamic acceptance remains exclusively pending `zr_vm:project`.
