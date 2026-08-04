---
title: WOS234 Rapid Fire swing-haste runtime
status: implementation_complete_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS234 Rapid Fire Swing-Haste Runtime

## Scope

Replicate Hunter `rapid_fire` (Fevered Draw): level 20, no resource cost,
instant off-GCD physical self cast, 300-second cooldown, and one
`selfBuff(buff_haste, value: 1.4, duration: 15)` effect. The source's
`swingIntervalMult` folds every active `buff_haste` by division, so the buff
shortens melee and ranged auto-attack intervals only; it is not spell haste,
weapon damage, or a duplicate Auto Shot event.

The existing Blade Flurry representation is intentionally narrowed to one
known haste aura. It must be converged into a source-shaped identity-aware
haste fold before Rapid Fire is admitted: Blade Flurry and Rapid Fire retain
their own duration/value contracts, re-casts refresh same-ability rows, and
different active haste auras compose multiplicatively. The current engine
motion-aura, cooldown, known-ability, input, serialization, and auto-attack
primitives cover this; no ZirconEngine infrastructure gap is identified.

## Delivery Order

1. Add the generated M4 projection, source-pinned static guard, and a
   `zr_vm:project`-only runtime fixture.
2. Refactor the retained `buff_haste` fold so validation identifies supported
   Blade Flurry and Rapid Fire rows independently and composes their swing
   interval divisors without allowing malformed or duplicate same-ability rows.
3. Add Rapid Fire identity, profile, slot/typed routing, Hunter admission,
   off-GCD cooldown handling, same-row refresh, and 15-second expiry.
4. Cover zero-cost/off-GCD behavior, cooldown rejection and expiry, recast
   refresh, composition with Blade Flurry, melee/ranged interval effects,
   serialization, source selection gates, and slot/typed parity. Run static
   checks and an independent second review; dynamic validation remains
   `zr_vm:project` only.

## Boundaries

- Do not represent Rapid Fire as spell haste, an extra attack, or a change to
  direct projectile damage.
- Do not weaken Blade Flurry's source profile while generalizing haste rows.
- Do not add a runtime fallback if the ZrVM plugin is unavailable.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| WOS234 | Hunter Rapid Fire swing-haste cooldown | implementation_complete_static_validation_pending | 2026-08-03 | M4 schema 108、source-pinned static guard、M4 三项生成一致性检查、Blade Flurry 回归与独立二次审查均通过；动态夹具仅允许 `zr_vm:project`。 |
