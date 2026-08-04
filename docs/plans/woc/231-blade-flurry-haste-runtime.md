---
title: WOS231 Blade Flurry haste runtime
status: implementation_complete_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS231 Blade Flurry Haste Runtime

## Scope

Replicate Rogue `blade_flurry` (Mirrored Blades): an instant, targetless,
normal-GCD physical self buff at level 10. It spends 25 Energy, has a
120-second cooldown, and applies source `buff_haste` with value `1.2` for 12
seconds. The source's `swingIntervalMult` multiplies one factor of `1 / value`
for every active `buff_haste` row, so Blade Flurry affects melee, offhand, and
ranged automatic swing cadence while composing multiplicatively with the
already retained item and group haste denominators.

The existing WOC motion-aura contract has no `buff_haste` code, and the
auto-attack projection presently fixes `swingIntervalMultiplier` to `1.0`.
This plan owns the source-pinned motion-kind extension, Blade Flurry's closed
self-aura lifecycle, and the player auto-attack projection. It does not add a
generic arbitrary haste-aura API or change non-player cadence behavior.

## Delivery Order

1. Add a source-pinned `buff_haste` motion-kind code and Blade Flurry to M4;
   regenerate JSON, Zr, and projection coverage with a `zr_vm:project`
   runtime fixture and static guard.
2. Add the Rogue identity/profile, normal-GCD/cost/cooldown reducer, slot and
   typed cast routing, snapshot, expiration, and cancellation behavior.
3. Fold only the active source-owned Blade Flurry aura into the player
   `swingIntervalMultiplier` as `1 / value`; retain the existing separate
   melee/ranged haste denominators and dual-wield offhand cadence.
4. Cover slot/typed parity, resource/GCD/cooldown, snapshot, expiry,
   cancellation, zero RNG, and melee/ranged/offhand cadence. Perform
   static validation followed by an independent second review.

## Boundaries

- Do not reinterpret `buff_haste` as additive haste or add it to spell casting.
- Do not admit unknown `buff_haste` motion aura rows or introduce a fallback VM.
- Dynamic execution is only through `zr_vm:project`.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
