---
title: WOS223 Lockout diminishing-returns state
status: implementing
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS223 Lockout Diminishing-Returns State

## Source Contract

`diminishedCrowdControlDuration` applies only to hostile player pairs. Lockout
uses the root-style `100/50/25/immune` ladder and an 18-second target-owned
reset. At the fourth active application it returns `null` without changing the
stored stage or reset time. Pummel computes that result first, then cancels a
valid cast and grants Rage; the null result suppresses only the lockout aura.
NPC targets retain the supplied four-second duration and create no DR state.

## Measured WOC Boundary

WOS173 already persists `entityFearDrStages` and `entityFearDrResetAt`, but
Fear has a different 60-second `8/4/2/1` ladder. Reusing it would make Lockout
incorrect. WOC has the entity kind and hostile-pair facts required by the
source rule, but no durable Lockout category rows, so Pummel currently cannot
express the source null-aura branch.

## Design

1. Keep `combat/lockout_dr_state.zr` as the source-pinned pure resolver for
   the player-pair predicate, 18-second reset, three duration stages and
   immunity outcome. This is intentionally separate from Fear and controlled
   Stun state.
2. After WOS221's WOS113 resource migration, append WOS114 entity-aligned
   `entityLockoutDrStages` and `entityLockoutDrResetAt` columns. Older decoders
   initialize both to zero; historical writers reject active Lockout DR state.
3. Add one `offlineLockoutDuration` world adapter using entity kind, target
   hostility and `timeMicros`. It updates only the new target rows. Pummel
   computes the result before cancellation, always preserves its valid-cast
   Rage reward, and adds a school lockout row only when the result is not
   immune.
4. Keep WOS222's scripted cast identity after this shared tail: it advances to
   WOS115 and consumes the same adapter. Cover PvE full duration, hostile PvP
   `4/2/1/immune`, reset, persistence, cancellation/reward on immunity and
   zero RNG with `zr_vm:project` fixtures.

## Acceptance

- A hostile player target receives Lockout durations of 4, 2 and 1 seconds;
  the next valid Pummel still cancels and grants Rage but adds no aura.
- A non-player target always receives the supplied duration and does not alter
  either Lockout DR row.
- WOS113 snapshots upgrade with zero Lockout DR rows, while WOS114 persists
  their stages and reset times deterministically. WOS115 scripted channels use
  the same resolver rather than a second lockout implementation.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| WOS223 | Lockout DR source-pinned pure resolver与 `zr_vm:project` fixture | implemented | 2026-08-03 | `combat/lockout_dr_state.zr` 覆盖 PvE、PvP `4/2/1/immune`、18 秒复位；`wos223_lockout_diminishing_returns_static_guard.mjs` |
| WOS223 | WOS114 持久化、Pummel null-aura bridge、WOS115 消费者 | implementing | 2026-08-03 | `WorldState` 仅有 Fear DR 行；Pummel 不能表示 source null Lockout outcome |
