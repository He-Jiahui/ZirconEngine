---
title: WOS213 absorb talent scaling runtime closure
status: completed
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS213 Absorb Talent Scaling Runtime

## Source Contract

`applyTalentModifiers` resolves `absorb.amount` as
`Math.round(amount * healMult * absorbMult + flat)`, where `healMult` is
`1 + global.healPct + ability.dmgPct`, `absorbMult` is
`1 + global.absorbPct`, and `flat` is the ability `flatDmg`. The effect
dispatcher applies the resolved amount to the aura at cast time. Later damage
only depletes that stored amount; a respec cannot change a live absorb.

## Current WOC Surface And Gap

M4 has `power_word_shield` and `temporal_barrier` absorb effects. WOC's bounded
offline absorb queue currently records target/source/ability/rank/current amount
and duration, but validates the current amount only against the generated raw
profile. It recomputes and applies that raw profile in both cast paths, so it
cannot retain a talent-resolved initial amount or verify it after restore.

## Design

1. Add a pure absorb resolver for source multiplication order and positive
   endpoint rounding. It owns no target admission, resource, cooldown, aura
   replacement, damage-depletion, threat or expiry behavior.
2. Extend the WOC-local state format to WOS110 with an initial resolved amount
   and six-row selection snapshot for every absorb queue row. The queue retains
   its current depletion amount; validation constrains it to
   `0 < current <= resolvedInitial` and reconstructs the initial amount from
   the stored selection.
3. Resolve PWS and Temporal Barrier at their existing cast/application points.
   Replacement keeps source queue ordering; damage depletion reads only the
   stored current amount. WOS109 and earlier normalize rows to generated raw
   initial amounts with empty snapshots, and historical writers reject a
   nonempty snapshot.
4. Keep `zr_vm:project` as the fixture backend. This is a WOC state-extension
   only; no Zircon engine ABI, plugin-host API, generated content schema or
   native-runtime capability is required.

## Acceptance

- Active PWS and Temporal Barrier retain their resolved amount through a later
  respec, partial absorption and encode/decode; a replacement uses the current
  selection.
- Queue replacement order, damage depletion, duration, resource, cooldown and
  target admission remain unchanged.
- WOS109 rows remain readable through forward normalization, and historical
  writers never discard a required selection snapshot.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| WOS213 | Absorb 天赋端点、WOS110 快照与迁移 | completed | 2026-08-03 | `wos213_absorb_talent_scaling_runtime_static_guard.mjs` 通过；二次审查确认 source dispatch 落盘、WOS110 对称编解码及历史归一化 |
