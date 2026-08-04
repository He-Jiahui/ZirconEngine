---
title: WOS230 Cold Blood next-attack critical runtime
status: implementation_complete_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS230 Cold Blood Next-Attack Critical Runtime

## Scope

Replicate Rogue `cold_blood` (Killer's Calm): an instant, targetless,
zero-cost, physical normal-GCD ability at level 10 with a 120-second cooldown.
It applies the source `next_attack_crit` self aura with value `1` for 60
seconds. The first eligible attack forces a critical result and consumes the
aura exactly once; retained direct damage, finishers, Judgement, and white
melee/ranged attacks are eligible. Expiry and `cancel_aura` remove it without
a hit.

The current WOC CC contract has no source-pinned code for this aura kind, and
the retained direct-damage, finisher, Judgement, and auto-attack paths
independently roll their critical result. Source `directDamage` consumes this
aura without a physical-school filter, while the source auto-attack owner
consumes it only after a landed swing reaches its critical roll. WOC must cover
every retained eligible impact path rather than applying a physical-only
shortcut. This plan owns the game-projection extension required to encode the
source aura and centralize one-shot consumption before those paths select their
critical outcome. It does not claim an engine or ZrVM defect.

## Delivery Order

1. Extend the source-pinned CC contract with `next_attack_crit`; regenerate
   JSON/Zr, add Cold Blood to M4, and bind the fixture to `zr_vm:project`.
2. Add the Cold Blood identity/profile/normal-GCD reducer and retain only its
   self-owned 60-second motion aura; cover cooldown, snapshot, expiry, and
   cancellation.
3. Introduce one source-order critical override/consumption boundary used by
   every retained `directDamage`, finisher, Judgement, and white auto-attack
   path; weapon strikes enter the same retained melee boundary. Preserve their
   existing random draw accounting: a forced crit consumes the existing
   critical-roll slot rather than adding a draw.
4. Cover a forced direct-damage spell, physical hit, finisher, and Judgement
   impact; also cover one non-eligible path, exact one-shot consumption,
   slot/typed parity, zero additional RNG, and static second review.

## Boundaries

- Do not implement a skill-specific damage shortcut or consume the aura on
  cast; source consumption happens when an eligible attack resolves.
- Do not generalize unknown motion-aura kinds or add a fallback runtime.
- Dynamic execution is only through `zr_vm:project`.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
