---
title: WOS220 Pummel interrupt-result rage runtime closure
status: implemented_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS220 Pummel Interrupt Result And Rage Runtime

## Source Contract

Source `pummel` (`Jawcrack`) is a level-8 Warrior, zero-cost, instant,
physical, targeted, off-GCD melee ability with a 10-second cooldown. Its sole
`interrupt` effect has a four-second lockout and `rageOnInterrupt: 10`.

Normal target admission uses the source five-yard melee range and 2.2-radian
front arc. Resolution is conditional: no target, no cast, fishing, a physical
cast, or an uninterruptible/scripted-invalid cast produces no interrupt reward.
For a valid nonphysical, interruptible cast, source cancels the cast, grants
`10 * warriorAbilityRageMult` Rage only when the caster is a Warrior on the
Rage resource bar, capped at maximum, then asks lockout DR for the aura
duration. A null DR result suppresses only the lockout, not the already-earned
Rage. This path consumes no RNG.

## Current WOC Surface And Gap

WOC already owns Skull Bash's hostile target range/arc predicate, cast
cancellation, school-specific lockout row, off-GCD cooldown model, Warrior
Rage bar and capped resource helper. It lacks Pummel's M4 projection, command
routes and result-dependent Rage grant. No ZirconEngine or ZrVM capability is
missing. The source's fractional Rage storage and rage-generation aura classes
are a separate WOC state-model follow-up; this milestone retains exact default
and integer-multiplier behavior without silently rounding a source value.
Source's `nythraxis_spirit_mending` pseudo-cast is also separate: WOC has the
source exception table but does not persist a scripted cast identity in
`WorldState`; WOS222 owns that encounter-state closure.
The retained state only persists Fear DR today. Source Lockout DR is a separate
PvP-only, target-owned category with its own 18-second reset and immunity
outcome, so WOS223 owns that shared state rather than Pummel silently treating
the first full-duration lockout as a complete DR implementation.

## Design

1. Append `pummel` after `feral_charge` to the retained M4 ability list without
   renumbering prior entries and regenerate both catalog projections.
2. Reuse the Skull Bash hostile melee predicate and lockout lifecycle, while
   keeping Pummel's off-GCD admission, zero cost and ten-second cooldown
   separate from the Druid ability.
3. Resolve any current known nonphysical cast, not merely M4 rows; retain the
   generated `uninterruptible` flag when an M4 row supplies one. After target
   admission, arm the cooldown before effect dispatch, cancel a valid cast and
   apply the saturated base 10-Rage grant before attempting the lockout row.
4. Cover generic slot and typed commands, a non-M4 known spell, no-cast,
   physical/fishing/uninterruptible rejection, 5-yard range/arc, Rage cap and
   zero RNG in one focused state fixture with a `zr_vm:project` backend
   declaration. WOS223 supplies the PvP DR-suppressed lockout case.

## Acceptance

- M4 codegen remains stable at 96 entries and Pummel appends as entry 95.
- A Warrior with a Rage bar gains 10 Rage only after a valid nonphysical cast
  is cancelled; the grant caps and does not consume RNG.
- Lockout duration remains school-specific and is independent of reward
  eligibility; generic and typed routes agree with normal off-GCD cooldown
  admission. WOS223 closes the source's PvP DR-suppressed lockout branch.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| WOS220 | Pummel M4 投影、Zr 生成、无 GCD 打断/怒气 reducer、slot 与 typed 路由、`zr_vm:project` 夹具 | implemented, second review clean | 2026-08-03 | 通用 current-known school projection、冷却先于 effect、non-M4 Blizzard/fishing/range/arc/uninterruptible fixture coverage；静态二审无 P1/P2，未运行 ZrVM/Cargo |
| WOS220 follow-up | Source PvP Lockout DR (suppressed aura but retained cancellation/Rage) | forwarded, WOS223 shared state | 2026-08-03 | `entityFearDrStages`/`entityFearDrResetAt` 不能代表 Lockout 的 18 秒与 immune 语义；`docs/plans/woc/223-lockout-diminishing-returns-state.md` |
| WOS220 follow-up | 源 Rage 浮点存储与 Rage-generation aura 倍率 | implementation forwarded and integrated; validation pending | 2026-08-03 | WOS221 owns the WOS113 exact-resource columns and Rage-generation aura state, including 11.5 Pummel reward and 20.7 stacked-aura coverage; its full Node guard has no result under the no-wait rule, so no dynamic acceptance is claimed. |
| WOS220 follow-up | `nythraxis_spirit_mending` 伪施法身份与可打断遭遇闭环 | forwarded, non-blocking | 2026-08-03 | `combat/scripted_interruptible_channels.zr` 与已持久化的 boss channel timer/ramp 未由 `WorldState` encounter tick 消费；`docs/plans/woc/222-scripted-interruptible-channel-state.md` |
