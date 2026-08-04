---
title: WOS228 Frost Armor rank and spec runtime
status: implemented_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS228 Frost Armor Rank and Spec Runtime

## Scope

Replicate Mage `frost_armor` (Hoarfrost Mantle): an instant, targetless Frost
self buff with three rank-dependent cost/armor values: level 1 `20/30`, level
10 `30/50`, and level 18 `45/70`, each lasting 1800 seconds. The source
excludes Fire and Arcane specs while permitting pre-spec and Frost mages.

It reuses the existing `buff_armor` motion-aura storage. The current
Demon Skin/Barkskin-specific armor contribution and invariant must converge to
an explicit retained armor-aura set that adds Frost Armor and validates each
source-pinned ability/rank profile. No schema or generic unknown-aura admission
is allowed.

## Delivery Order

1. Append `frost_armor` after WOS227 and regenerate M4 JSON/Zr/coverage, with
   a source-pinned guard and `zr_vm:project` fixture.
2. Add rank-aware identity, profile, normal-GCD resource reducer and slot/typed
   routes. Preserve source spec exclusion through catalog admission.
3. Converge retained armor aura contribution/invariants for Demon Skin,
   Barkskin and Frost Armor only, without altering prior effects.
4. Cover all rank costs/values, excluded spec, snapshot/expiry/cancel,
   slot/typed parity and zero RNG, then second-review static evidence.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| WOS228 | Frost Armor M4 投影、三来源 armor-aura 闭集、等级/专精/GCD reducer、slot/typed/cancel 路径与 `zr_vm:project` fixture | 实现完成，静态二次审查完成；动态验证待执行 | 2026-08-03 | `wos228_frost_armor_rank_spec_static_guard.mjs`、M4 JSON/Zr/coverage `--check` 均通过；仅允许 Demon Skin、Barkskin、Frost Armor 的 `buff_armor` 行；不使用 fallback runtime |
