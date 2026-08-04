---
title: WOS229 Hunter Aspect exclusive aura runtime
status: implemented_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS229 Hunter Aspect Exclusive Aura Runtime

## Scope

Replicate Hunter `aspect_of_the_hawk`, `aspect_of_the_monkey`, and
`aspect_of_the_cheetah`: instant, targetless Nature self buffs sharing source
`exclusiveGroup: aspect`. Hawk has rank-dependent costs and `buff_ap` values
of `20/20`, `30/35`, and `40/50` at levels 4, 12, and 18. Monkey costs 20 at
level 5 and grants `buff_dodge` `0.08` for 1800 seconds. Cheetah costs 20 at
level 14 and grants `buff_speed` `1.3` for 1800 seconds.

The source removes only different active Aspect auras before applying a new
one, while a recast refreshes its own row. WOC must retain this exact
exclusive-set behavior and extend each existing derived-stat aura boundary
without admitting arbitrary `buff_ap`, `buff_dodge`, or `buff_speed` rows.

## Delivery Order

1. Append all three Aspect definitions to M4 and regenerate JSON, Zr, and
   projection coverage with source-pinned static guard and `zr_vm:project`
   fixture.
2. Add rank-aware profiles, normal-GCD resource reducers, and slot/typed
   routes. Reuse motion-aura storage without a new schema.
3. Converge retained AP, dodge, and speed aura validation/contribution paths
   to an explicit source-pinned set, preserving Tiger's Fury, Primal
   Reflexes, Evasion, Ghost Wolf, and Dash behavior.
4. Remove different active Aspect rows in descending index order before apply;
   cover recast refresh, all transitions, rank costs/values, snapshot, expiry,
   cancel, slot/typed parity, and zero RNG. Then perform static second review.

## Boundaries

- Do not alter the WOS222/WOS223 interrupt and lockout work.
- Do not add a generic aura schema or admit unknown derived-stat aura rows.
- Dynamic execution is only through `zr_vm:project`; no fallback runtime is
  accepted.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| WOS229 | Hunter Aspect M4 投影、互斥 aura 清理、AP/闪避/速度闭集、等级/GCD reducer、slot/typed/cancel 路径与 `zr_vm:project` fixture | 实现完成，静态二次审查完成；动态验证待执行 | 2026-08-03 | `wos229_hunter_aspect_exclusive_aura_static_guard.mjs` 及 M4 JSON/Zr/coverage `--check` 均通过；不同 Aspect 倒序移除、同能力重施刷新；不使用 fallback runtime |
