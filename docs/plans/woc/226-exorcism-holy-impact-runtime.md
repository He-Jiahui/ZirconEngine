---
title: WOS226 Exorcism holy impact runtime
status: implemented_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS226 Exorcism Holy Impact Runtime

## Scope

Replicate source Paladin `exorcism` (Rite of Expulsion): a level-5,
55-resource, instant, 30-range Holy spell against a hostile target. It has a
15-second independent cooldown and one `directDamage` effect of 46--56.

The slice uses the retained normal-GCD instant-spell projectile lifecycle:
resource is paid at cast start, one Holy projectile is created, normal
spell-resist/crit/talent impact resolution applies on arrival, and the
independent cooldown starts before flight. It must retain source rank one,
hostile/range checks, target selection, threat/combat effects and deterministic
RNG draw order. It has no source spec, charge, aura, shared-cooldown or schema
requirement.

## Delivery Order

1. Append `exorcism` after WOS225 without renumbering earlier M4 entries;
   regenerate JSON/Zr/coverage projections and add a source-pinned static guard
   plus `zr_vm:project` fixture.
2. Add the exact identity/payload/profile and hostile-target reducer; model
   normal GCD, cost, independent cooldown and an immediate Holy projectile.
3. Register its projection validation and impact dispatch. Reuse fixed spell
   impact/talent resolution rather than copying damage, hit or resistance math.
4. Cover slot and typed casts, cost/cooldown/projectile profile, snapshot
   flight/landing, wrong level/class/target rejection and deterministic RNG in
   a focused state test. Then run static regression and a second review.

## Boundaries

- Do not route through the Shaman Shock shared cooldown.
- Do not turn the source instant spell into a timed cast, aura or fallback
  native runtime path.
- Do not change WOS222 scripted channel or WOS223 lockout work.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| WOS226 | Exorcism M4 投影、独立冷却 Holy 投射、落点分派、slot/typed 路由与 `zr_vm:project` fixture | 实现完成，静态二次审查完成；动态验证待执行 | 2026-08-03 | `wos226_exorcism_holy_impact_static_guard.mjs` 及 M4 JSON/Zr/coverage `--check` 均通过；不使用 fallback runtime |
