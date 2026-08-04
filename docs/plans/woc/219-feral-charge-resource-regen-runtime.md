---
title: WOS219 Feral Charge resource-regen runtime closure
status: implemented
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS219 Feral Charge And Resource Regen Runtime

## Source Contract

Source `feral_charge` (`Primal Surge`) is a level-10 Druid, zero-cost,
instant, physical, targetless, off-GCD ability with a 90-second cooldown and
`usableInForm: true`. Its sole `feralCharge` effect has no numeric payload.
The effect resolves after ordinary ability admission:

- In Cat Form it applies or refreshes the self aura
  `feral_instinct_energy` (`buff_energyregen`, value `1`, duration `10`).
- In Bear Form with the Rage resource bar it immediately adds 50 Rage, capped
  at the live maximum.
- In all other forms it has no effect after its normal cast/cooldown admission.

The source update phase runs after movement, casting and auto-attacks. Every
40 fixed ticks (two seconds), an Energy bar gains `round(20 * product(1 +
buff_energyregen.value))`, capped at its maximum. Within a source tick,
resource regeneration runs before aura aging. Therefore a positive final-tick
duration contributes to that tick's regeneration; the first later eligible tick
sees the expired aura. This path consumes no RNG.

## Current WOC Surface And Gap

WOC already owns Cat/Bear form identity, live resource kind/max/resource rows,
fixed 50ms state ticks, cast/cooldown admission, generated M4 ability metadata,
and historical schema migration. It does not project `feral_charge`, and it has
no authoritative passive Energy regeneration or durable Feral Instinct timer.
No ZirconEngine or ZrVM capability is missing.

## Design

1. Append `feral_charge` to the M4 retained ability list as entry 94, keeping
   the existing 94 indices unchanged, and regenerate the JSON and Zr catalog.
2. Add a small pure combat helper for the two-second Energy restoration
   calculation and saturated Bear Rage grant. In `WorldState`, append a
   per-entity Feral Instinct remaining-duration row, defaulting to zero.
3. Advance the schema from WOS111 to WOS112: writer appends the new aligned
   row; WOS111 and older readers synthesize zero; invariants, bootstrap, entity
   append paths, decoder defaults and the world-state contract remain aligned. This is a
   forward migration only.
4. Route generic-slot and typed `feral_charge` commands through normal
   off-GCD/cooldown admission. At dispatch, Cat refreshes its 10-second timer;
   Bear with Rage receives the capped +50; neither branch consumes RNG.
5. Place the passive resource step in `stepRetainedPlayerTicks` after retained
   casting and before consumables. On every `state.tick % 40 == 0`, an Energy
   bar receives base 20 multiplied by the active Feral Instinct modifier; the
   timer then ages in the ordinary tick path. This preserves the source's
   resource-before-aging order within WOC's retained player phase.

## Acceptance

- M4 codegen is stable at 96 abilities, with `feral_charge` retained at entry
  94 and all
  prior ability indices unchanged.
- Cat casts refresh a persisted ten-second Feral Instinct duration, have no
  immediate resource grant and yield 40 Energy on every eligible tick whose
  pre-aging timer is positive; the first later eligible tick yields 20. Energy
  is capped at its maximum.
- Bear casts with the Rage bar grant exactly 50 Rage, capped at the maximum;
  caster/no-form paths perform no branch effect but retain normal off-GCD
  cooldown admission.
- Generic and typed command routes agree, no branch changes RNG, WOS111 input
  restores with zero Feral Instinct duration, and WOS112 round-trips it.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| WOS219 | M4 投影、WOS112 计时器、资源 tick、命令路由、夹具 | implemented, second review clean | 2026-08-03 | 96 条 M4 生成检查、`wos219_feral_charge_resource_regen_static_guard.mjs` 及受影响历史守卫通过；复审 P2 已补正 GCD 下 off-GCD 回归并复审通过；`woc_m4_feral_charge_resource_regen_runtime_tests.zrp` 指定 `zr_vm:project`，但本会话未运行 ZrVM/Cargo。 |

## Cross-Layer Follow-Up

The native `woc_protocol` identity remains WOS83 while the script-owned
authoritative envelope is WOS112. This is recorded in
`.codex/sessions/20260803-woc-native-wos111-protocol-identity-drift.md` for the
native protocol owner. It is a forward reconciliation task; WOS219 does not
downgrade the Zr state or alter native code.
