---
title: WOS236 Arcane Shot choice charge and venom runtime
status: implementation_complete_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS236 Arcane Shot Choice Charge and Venom Runtime

## Scope

Complete the source choice-row semantics excluded from WOS235's base Arcane
Shot projectile. `hun_r5_quick_shots` grants `bonusCharges: 1`, so Fell Shot
has two uses before the six-second recharge gate blocks another cast. It must
be modeled as a general source ability-charge contract, not as a one-off
cooldown exception. `hun_r14_serpents_venom` appends a Nature DoT equal to 50%
of the Arcane Shot's landed direct damage, with three one-second ticks; it
must not apply on a fully resisted or fizzled projectile.

Existing WOC talent snapshots, cooldown state, periodic DoT queues, magic
projectile travel, and serialization are sufficient building blocks. The
missing layer is application-side charge accounting and dynamic source
add-effect dispatch, not a ZirconEngine runtime gap.

## Delivery Order

1. Add source-pinned selection metadata, static guard and `zr_vm:project`-only
   fixture before implementation.
2. Introduce serializable ability charges with a generic expiry/recharge
   contract, then wire Quick Shots only through source-selected modifiers.
3. Add a source-ordered post-impact dynamic-effect reducer for Serpents Venom:
   calculate from the resolved direct damage, create the Nature periodic row,
   and preserve resistance/fizzle/no-extra-RNG behavior.
4. Cover one-versus-two charge use, recharge timing, cooldown display gate,
   respec and snapshot behavior, resisted/fizzled no-DoT paths, critical direct
   damage feeding the 50% total, tick order, serialization, and independent
   second review. Dynamic validation remains `zr_vm:project` only.

## Boundaries

- Do not implement charges as an Arcane Shot-only boolean or add a fallback
  backend.
- Do not create Serpents Venom from cast start, base endpoint values, or a
  resisted projectile; it is derived only from source-resolved landing damage.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| WOS236 | Quick Shots 充能与 Serpents Venom | implementation_complete_static_validation_pending | 2026-08-03 | WOS114 可序列化并行充能状态、重选天赋收敛、飞行快照 addEffect 与 Nature 三跳 DoT 已实现；二次审查发现的周期伤害仇恨遗漏已前向修复并增加每跳仇恨断言，复核无 findings；source-pinned 静态守卫和 111 项 M4 投影检查通过；仅允许 `zr_vm:project` 动态夹具，待协调器唤醒。 |
