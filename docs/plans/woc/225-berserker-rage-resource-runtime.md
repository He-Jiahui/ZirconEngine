---
title: WOS225 Berserker Rage resource runtime
status: implemented_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS225 Berserker Rage Resource Runtime

## Scope

Replicate source Warrior `berserker_rage` (Seething Fury): a level-10,
zero-cost, instant, targetless, physical, off-GCD self ability with a
30-second cooldown and one `gainResource` effect of 20 Rage.

The reducer reuses WOS215 source gain-resource talent scaling and WOS221 exact
resource storage. It performs normal known-ability/class/level/resource-bar
admission, applies a capped exact gain, arms its cooldown, leaves normal GCD
unchanged, creates no aura and consumes no RNG. No state schema or engine
capability changes are required.

## Delivery Order

1. Append `berserker_rage` after WOS224 in the generated M4 catalog without
   renumbering earlier entries, regenerate JSON/Zr projections, and add a
   source-pinned static guard plus `zr_vm:project` fixture.
2. Add exact profile/payload helpers and normal generic-slot/typed routing to
   the shared world state. Reuse `resolveGainResourceAmount` and
   `offlineCappedResourceGain`; do not add an ability-local integer write.
3. Cover cap, cooldown, unchanged GCD, wrong level/class/resource bar,
   slot/typed parity, snapshot round-trip and zero RNG in a focused state test.
4. Run static regression and a second review. Dynamic evidence remains solely
   `zr_vm:project`, with no fallback runtime.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| WOS225 | Berserker Rage M4 投影、精确 Rage reducer、slot/typed 路由与 `zr_vm:project` fixture | 实现完成，静态二次审查完成；动态验证待执行 | 2026-08-03 | `wos225_berserker_rage_runtime_static_guard.mjs` 通过；源定义、M4 条目、共享资源精度桥及状态测试已二次审查；不使用 fallback runtime |
