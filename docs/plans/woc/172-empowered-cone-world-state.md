---
title: WOS172 Empowered cone WorldState integration
status: implemented_static_dynamic_validation_deferred
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS172 Empowered Cone WorldState Integration

## Source contract

Current-head defines two four-stage, 2.4-second hold casts: Frost Mage
`glacial_front` and Fire Mage `dragons_breath`. The client sends
`releaseEmpowered { ability }` without timing data. On release, the
authoritative simulation requires that exact live, non-channel cast and derives
the stage from `castTotal` and `castRemaining`; stun or the applicable
silence/lockout cancels it. A natural completion resolves stage four.

`effect_dispatch` emits the cone effect before target iteration, iterates
eligible hostile targets in the simulation order, skips dead/line-of-sight/
cone failures without RNG consumption, then consumes a spell-crit draw and a
damage-range draw per hit. It applies damage before slow/root/incapacitate,
skips control for newly dead targets, and performs the Fire Hot Streak note once
after the entire hit batch.

## Implementation scope

- WOS172a integrates `glacial_front` only, reusing the existing pure
  `empowered_cone_state` planner. It adds minimal `WorldState` bridges for
  known-ability admission, spell-haste arming, typed id-149 release decoding,
  natural stage-four completion, target iteration, existing damage/threat/death
  writes and the existing motion-aura store, including Frost school identity on
  its slow and root rows.
- Preserve the current bounded offline Eastbrook projection. It must not claim
  source-wide group targeting, generic LoS, event projection, all aura classes,
  or a substitute transaction host.

`dragons_breath` remains deferred: its source semantics require the Fire
Hot-Streak batch aggregation and fear-class diminishing-control state, neither
of which is currently persisted by the WOS candidate. It must be added as a
later state slice, never approximated by dropping those effects.

## Deferred authority

The full command batch only reaches `WorldState` through the real
`zr_vm:project` transactional plugin route. WOS172 must not implement native
gameplay fallback, mutate host state outside the candidate transaction, or
claim dynamic parity while the Plugins08 handoff remains open.

## Verification

- Static source anchors pin the two ability definitions, `releaseEmpowered`
  send, live-clock stage derivation and `empoweredCone` ordering.
- The focused ZrVM WorldState entry covers Glacial Front early release, natural
  completion, wrong-id no-op and stunned interruption. The existing pure
  empowered-cone contract pins per-hit RNG ordering and post-damage control
  gating.
- Regenerate/check any affected generated contracts and defer Cargo and real
  ZrVM transaction acceptance to Plugins08.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
