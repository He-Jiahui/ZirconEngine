---
title: WOS217 Life Tap talent scaling runtime closure
status: completed
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS217 Life Tap Talent Scaling Runtime

## Source Contract

`scaleEffect` first resolves only `lifeTap.mana` as
`Math.round(mana * dmgMult + flat)`. The effect dispatcher then applies the
ability `buffPct` as a second `Math.round(resolvedMana * (1 + buffPct))`.
The health payment remains the generated raw `hp` value. The first multiplier
uses the ability school: `1 + spellDmgPct + ability.dmgPct` for the shadow Life
Tap entry. The effect then keeps the existing strict health gate and caps mana
at the normal resource maximum.

## Current WOC Surface And Gap

`startOfflineLifeTapCast` has the source admission, GCD, strict health gate and
resource cap, but writes the raw generated mana field directly. Its current
Warlock loadouts do not expose a modifier for this ability, yet the source
effect contract is generic and the runtime must not silently diverge when a
source-valid modifier is present.

## Design

1. Add a pure Life Tap mana resolver matching source school/global/ability
   ordering, the second `buffPct` phase and positive endpoint rounding.
2. Resolve mana at the existing immediate resource mutation point while leaving
   the generated health payment, GCD, cap and no-RNG behavior unchanged.
3. Add a pure mixed-modifier contract test plus live raw-path regression,
   source-pinned static guard and a `zr_vm:project` fixture. No WOS schema or
   Zircon engine interface change is needed.

## Acceptance

- Synthetic spell, ability and `buffPct` modifiers produce the source two-phase
  rounded mana value; the health payment is not scaled.
- Current no-modifier Life Tap preserves its raw resource result, strict health
  threshold, resource cap, GCD and zero RNG behavior.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| WOS217 | Life Tap 两阶段天赋缩放运行时闭环 | completed | 2026-08-03 | WOS217 静态守卫、M4 与天赋目录生成检查、差异检查均通过；独立二次审查通过。未运行动态 ZrVM/Cargo。 |
