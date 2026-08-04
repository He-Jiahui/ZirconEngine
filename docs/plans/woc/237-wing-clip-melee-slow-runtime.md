---
title: WOS237 Wing Clip melee slow runtime
status: implementation_complete_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS237 Wing Clip Melee Slow Runtime

## Scope

Project Hunter `wing_clip` (Fettering Slash): level 10, 20 resource,
instant normal-GCD physical melee target action with no cooldown. The source
resolves direct damage 3-5, then applies `slow` with multiplier 0.6 for ten
seconds. It is not a projectile, ranged attack, or weapon strike; resource and
GCD resolve at admission, while the physical direct branch, armor, threat,
combat and the slow aura retain source effect order.

Existing WOC physical direct settlement, target/facing gates, motion-aura
state, fixed precision codec and `zr_vm:project` fixture support this slice.
No ZirconEngine infrastructure gap is anticipated.

## Delivery Order

1. Add source-pinned M4 metadata and static fixture.
2. Route slot and typed casts through a Hunter melee reducer with exact
   resource/GCD and close-range target admission.
3. Apply source-ordered physical direct damage then the 0.6 slow aura,
   preserving fizzle, armor and snapshot behavior.
4. Cover admitted/rejected target geometry, resource/GCD, hit/armor,
   slow duration, serialization and typed parity; complete static validation
   and independent second review before coordinator-driven ZrVM acceptance.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| WOS237 | Wing Clip 物理近战 directDamage 与 slow | implementation_complete_static_validation_pending | 2026-08-03 | 槽位与 typed cast 均接入；物理 directDamage 解析独立于 `state.zr`，保留近战 AP、暴击、护甲与两次 RNG；既有 motion-aura 保存 0.6/10 秒 physical slow 并通过快照；source-pinned 静态守卫、M4 111 项投影检查和空白检查通过，独立二次审查无 findings；仅允许 `zr_vm:project` 动态夹具，待协调器唤醒。 |
