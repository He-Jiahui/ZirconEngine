---
title: WOS222 scripted interruptible channel state closure
status: implementing
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS222 Scripted Interruptible Channel State

## Source Contract

`nythraxis_spirit_mending` is a scripted, non-ability channel with school
`shadow`. Interrupt resolution first looks up an ordinary resolved ability;
when none exists it consults `SCRIPTED_INTERRUPTIBLE_CHANNELS`. A valid Pummel
therefore cancels the cast, grants its Rage reward, and applies a Shadow
lockout. The Nythraxis encounter observes that lockout, clears the channel bar
and resets the mending ramp before it can be armed again.

## Measured WOC Boundary

WOC already has `combat/scripted_interruptible_channels.zr`, including the
exact `nythraxis_spirit_mending -> shadow` table. It is not imported by
`WorldState`. `entityCastingAbility` stores only a generated numeric ability
code; Pummel correctly rejects unknown codes, so it cannot identify a scripted
cast. WOC already persists `entityBossChannelTimers` and
`entityBossChannelRamps`, but no world tick consumes them as a channel lifecycle
and neither provides a cast identity, target or cancellation state.
Every currently retained M8 spawn initializes those channel fields to zero, so
the source Nythraxis channel-heal content projection is absent as well.

## Design

1. After WOS221's WOS113 resource migration and WOS223's WOS114 Lockout DR
   tail, append only a WOS115
   entity-aligned scripted-cast code. Use a generated numeric vocabulary rather
   than persisting arbitrary strings; zero means absent and code one is the
   source Nythraxis Mending identity. Reuse the existing durable
   `entityBossChannelTimers` and `entityBossChannelRamps` rows for remaining
   time and escalating amount. WOS113 and older decode zero, and historical
   writers reject any active scripted cast.
2. Add one authoritative cast-view adapter which resolves either the existing
   M4 `entityCastingAbility` or a scripted code into `(present, fishing,
   interruptible, school)`. Pummel and every later interrupt use it; ordinary
   noncatalog pseudo-casts remain immune.
3. Project the source Nythraxis content before arming a lifecycle: select the
   living same-faction, non-pet, largest-health protectee in radius and hold a
   six-unit caster standoff. Mirror the current scripted cast, retain its ramp,
   clear it on matching Shadow lockout/stun/silence, reset the timer to its
   source cadence, and do not overload `entityBossMendTimers` with cast
   identity.
4. Cover regular M4 Pummel, scripted Shadow Pummel, unknown pseudo-cast
   rejection, WOS113 migration, WOS115 round trip, lockout cancellation and
   zero RNG under a `zr_vm:project` fixture. Advance the native protocol
   identity handoff together with the WOS113/WOS115 script envelope.

## Acceptance

- Pummel cancels `nythraxis_spirit_mending`, grants Rage before the lockout
  outcome, and applies a four-second Shadow lockout; an unknown pseudo-cast is
  unchanged and earns no Rage.
- Nythraxis cannot retain or immediately re-arm its mending ramp while the
  matching lockout is active, and it round-trips the scripted cast state.
- WOS113/WOS114 snapshots decode with no scripted cast; WOS115 writers reject an
  active scripted cast when emitting any older schema while preserving existing
  channel timer/ramp values.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| WOS222 | 数值伪施法词汇 | implemented | 2026-08-03 | `combat/scripted_interruptible_channels.zr` 以 `0=absent, 1=nythraxis_spirit_mending` 固定双向映射 |
| WOS222 | Nythraxis channel content、目标选择与 tick 纯状态 | implemented | 2026-08-03 | `combat/nythraxis_channel_state.zr` 固化 45/6/4/320/240/1440、最大生命 protectee、standoff、Ramp 与 Shadow interruption reset；尚未接入 `WorldState` |
| WOS222 | 源伪施法身份、打断桥接、Nythraxis channel lifecycle | planned, after WOS114 | 2026-08-03 | Source `mob/healer_channel.ts`; retained M8 channel fields are all zero; WOC `combat/scripted_interruptible_channels.zr` has no `WorldState` consumer |
