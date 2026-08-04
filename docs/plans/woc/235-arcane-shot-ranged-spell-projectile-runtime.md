---
title: WOS235 Arcane Shot ranged-spell projectile runtime
status: implementation_complete_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS235 Arcane Shot Ranged-Spell Projectile Runtime

## Scope

Replicate Hunter `arcane_shot` (Fell Shot): level 5, 25 resource, instant
normal-GCD cast, six-second cooldown, 8-35 yard hostile-target window,
Arcane school, `scalesWith: ranged`, and rank-resolved `directDamage`
endpoints (13-17, 24-30, 38-47). Although it has no explicit `projectile`
flag, the source routes every non-physical target spell through projectile
travel. Resource and cooldown resolve on cast; target death/despawn fizzles
the in-flight bolt.

At impact it follows the source spell branch: first one magic-resist draw, then
the direct-damage interval and spell-critical draw if the shot was not fully
resisted. The direct scaling uses live Ranged AP with the instant
`1.5 / 3.5 * 0.15` coefficient, not Spell Power, while the Arcane school uses
the spell critical multiplier and Arcane post-impact hooks. It is neither an
Auto Shot weapon attack nor a physical projectile: no weapon roll, armor
mitigation, physical hit/miss, or attack animation/proc path may be inherited.

The retained runtime already has target validation, cast/GCD/cooldown state,
projectile travel, spell-resist, source-pinned scaling helpers, threat,
serialization, and Arcane projectile effects. No ZirconEngine infrastructure
gap is identified; WOS235 is WOC integration work.

This milestone retains the base ability only. Its source choice rows add a
second charge (`hun_r5_quick_shots`) and a post-hit Nature DoT
(`hun_r14_serpents_venom`); those cross-cutting higher-order effects are
tracked immediately in WOS236 after the base projectile closes.

## Delivery Order

1. Add Arcane Shot to the M4 projection (index 109), a source-pinned static
   guard, and a `zr_vm:project`-only runtime fixture.
2. Add exact identity/profile checks and slot/typed routing. Start-time target
   validation must retain the 8-35 yard, hostile, line-of-sight and facing
   gates; instant resolution must charge resource, arm cooldown and normal GCD
   before the bolt travels.
3. Add an Arcane Shot projectile profile and landing reducer that keeps source
   endpoints and rank in flight, rolls spell resistance first, then applies
   live Ranged AP direct scaling, spell crit, Arcane damage conversion, threat,
   combat and lethal handling in source order.
4. Cover all three base ranks, cost/GCD/cooldown timing, minimum/maximum/facing
   rejection, source/target fizzle, resistant versus landed RNG order, live
   Ranged AP scaling, spell-critical behavior, no Auto Shot/armor path,
   serialization, and slot/typed parity. Run static checks and an independent
   second review; dynamic validation remains `zr_vm:project` only.

## Boundaries

- Do not infer a physical projectile merely because the scaling stat is Ranged
  AP; the source school is Arcane and enters the spell-resist branch.
- Do not snapshot Ranged AP or critical chance at cast time; `runEffects` reads
  them when the projectile lands.
- Do not fold choice-row charges or add-effect DoTs into the base projectile;
  WOS236 owns their shared runtime contracts.
- Do not add a fallback runtime backend.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| WOS235 | Arcane Shot 基础投射物 | implementation_complete_static_validation_pending | 2026-08-03 | M4 index 109；source-pinned 静态守卫通过；补齐 rank、抵抗/RNG、失效、落地 RAP/暴击、护甲独立与 typed 路径状态覆盖；ZrVM 动态验收待独立审查后由协调器唤醒。 |
