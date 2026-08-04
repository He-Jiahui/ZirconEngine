---
title: WOS227 Evasion dodge aura runtime
status: implemented_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS227 Evasion Dodge Aura Runtime

## Scope

Replicate source Rogue `evasion` (Ghostfoot): a level-8, zero-cost, instant,
targetless, physical, off-GCD ability with a 300-second independent cooldown.
It creates a 15-second self `buff_dodge` aura with value `0.5`.

The world already retains the same motion-aura representation for
`primal_reflexes`. Before registering Evasion, converge its ability-specific
dodge contribution and motion-state validation into an explicit retained
dodge-aura set containing only those two source-pinned abilities. This changes
neither aura storage nor snapshot shape, and preserves Primal Reflexes' own
level-20/60-second behavior.

## Delivery Order

1. Append `evasion` after WOS226 and regenerate the M4 JSON/Zr/coverage
   projections, with a source-pinned guard and `zr_vm:project` fixture.
2. Add exact profile/payload helpers and off-GCD independent-cooldown reducer.
   Reuse the motion-aura append/expiry/cancel path; do not add a dodge column.
3. Generalize the retained dodge-aura resolver and invariant to recognize only
   `primal_reflexes(0.5, 6s)` and `evasion(0.5, 15s)`.
4. Cover slot/typed parity, no-GCD, cooldown, snapshot/expiry/cancel,
   effective-dodge contribution, invalid class/level and zero RNG, then run
   static regression and a second review. Dynamic execution remains only
   `zr_vm:project`.

## Boundaries

- Do not alter Primal Reflexes semantics while generalizing its retained-aura
  lookup.
- Do not introduce generic unknown `buff_dodge` acceptance or a fallback
  runtime.
- Do not change WOS222/WOS223 interrupt and lockout work.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| WOS227 | Evasion M4 投影、受控 dodge-aura 集合、off-GCD reducer、slot/typed 路由与 `zr_vm:project` fixture | 实现完成，静态二次审查完成；动态验证待执行 | 2026-08-03 | `wos227_evasion_dodge_aura_static_guard.mjs` 及 M4 JSON/Zr/coverage `--check` 均通过；Primal Reflexes 保持 6 秒语义；不使用 fallback runtime |
