---
title: WOS233 Aimed Shot ranged projectile runtime
status: implementation_complete_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS233 Aimed Shot Ranged Projectile Runtime

## Scope

Replicate Hunter `aimed_shot` (Long Draw): level 11, cost 50, three-second
cast, six-second cooldown, 8-35 yard target window, physical school,
projectile-enabled, `scalesWith: 'ranged'`, and one 50-62 `directDamage`
effect. Source dispatch resolves the direct hit only when the projectile
lands: its random flat roll gains `round(rangedPower * (3 / 3.5) * 0.15)`,
then follows the physical hit, critical, armor and threat paths.

The source treats this as a physical attack-spell, not an Auto Shot. WOC's
generic ranged landing helper applies the Auto Shot weapon formula and is
therefore deliberately not reused. Existing casting, projectile snapshot,
spell-scaling and physical hit infrastructure already provide the required
engine-facing primitives; no ZirconEngine infrastructure gap is identified.

## Delivery Order

1. Add the generated M4 projection and a source-pinned static guard plus a
   `zr_vm:project`-only runtime fixture.
2. Add exact ability identity, slot/typed routing, Hunter admission, the
   8-35 yard target window, delayed resource spend, three-second cast lock,
   six-second cooldown and projectile launch on successful completion.
3. Add a dedicated physical attack-spell projectile landing path that snapshots
   the source direct-damage endpoints and resolved cast duration at launch,
   reads live ranged AP at impact, and uses the retained physical
   critical/armor/threat lifecycle without the Auto Shot 0.6 weapon multiplier,
   weapon-speed AP formula, or an Auto Shot hit roll.
4. Cover cast interruption and target invalidation, min/max range, cost and
   cooldown timing, ranged scaling snapshot, projectile impact, critical and
   armor handling, serialization, RNG, and slot/typed parity. Run static
   checks and an independent second review; dynamic validation remains
   `zr_vm:project` only.

## Boundaries

- Do not represent Aimed Shot as an Auto Shot, a wand, or a generic spell
  whose source school/miss rules differ from physical attack-spells.
- Do not add a runtime fallback if the ZrVM plugin is unavailable.
- Keep Hunter choice-row cast-time modifiers source-pinned; do not infer them
  from a generic haste adjustment.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| WOS233 | Hunter Aimed Shot hard-cast physical projectile | implementation_complete_static_validation_pending | 2026-08-03 | M4 schema 107、source-pinned static guard、物理护甲接口前向修复与独立二次审查均通过；动态夹具仅允许 `zr_vm:project`。 |
