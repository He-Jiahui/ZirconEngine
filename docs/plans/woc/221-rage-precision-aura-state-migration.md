---
title: WOS221 fractional resource and Rage-generation aura migration
status: implementation_complete_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS221 Fractional Rage And Rage-Generation Aura State

## Source Contract

Source entities retain `resource` and `maxResource` as JavaScript numbers.
Pummel awards `rageOnInterrupt * warriorAbilityRageMult` after a valid
interrupt. `warriorAbilityRageMult` is `(1 + abilityRagePct)` times the sum of
active `buff_rage_gen`, `buff_reckless`, and `battle_stance` aura modifiers.
For example, Anger Management permits an exact `10 * 1.15 = 11.5` Rage award.
The reward is capped only after that multiplication, and cancellation remains
independent of a null PvP lockout DR result.

## Measured WOC Boundary

`WorldState.entityResources` and `entityMaxResources` are `Array<int>`. Their
historical WOS15 row is encoded with `writer.signed` and decoded with
`reader.signed`; the same integer rows feed cast admission, costs, form
transitions, passive restoration, auto-attack adapters and state invariants.
The WOC talent projection already contains `abilityRagePct`, but the retained
motion-aura kind contract does not represent the three source Rage-generation
auras. Therefore changing only Pummel would either round a source number or
drop part of its multiplier.

## Design

1. Advance the script envelope from WOS112 to WOS113 without altering the
   historical WOS15 resource/max-resource bytes. Append entity-aligned
   `fixed6` authoritative resource and maximum-resource override columns;
   WOS112 and older decoders initialize each exact value from the existing
   signed integer rows. Historical writers reject a nonintegral override.
2. Introduce one small resource-state helper that owns exact load, compare,
   capped add, spend and legacy integer-mirror rules. Replace direct combat
   resource access by domain batches: generic cast admission/costs, form
   transitions, passive regeneration, auto attacks, consumables and test
   builders. Do not convert a call site opportunistically without using the
   helper.
3. Extend the durable aura vocabulary with the source Rage-generation kinds
   and a source-keyed value/remaining representation. Compute the multiplier
   in deterministic source order from the entity's current talent snapshot and
   active aura rows. Keep PvE target lockouts at full duration; model PvP DR
   only when the retained player-vs-player surface exists.
4. Upgrade Pummel to consume the exact multiplier and write the capped exact
   resource result. Add round-trip, WOS112 migration, 11.5-Rage, stacked aura,
   cap, cost-after-fraction and zero-RNG fixtures under `zr_vm:project`.
5. Update `world-state.md`, package identity, static guards and the native
   protocol owner handoff. Native currently declares WOS83, so it must
   reconcile through WOS113 rather than reintroducing an older identity.

## Acceptance

- A WOS113 Pummel with Anger Management produces and round-trips exactly 11.5
  Rage; the next resource cost observes that fractional value correctly.
- `buff_rage_gen`, `buff_reckless` and `battle_stance` stack by the source
  multiplier and cap only after multiplication.
- WOS112 snapshots migrate losslessly from their integer resource values, and
  a WOS112 writer rejects any nonintegral WOS113 resource override.
- Existing integer-only ability fixtures remain behaviorally unchanged while
  generic resource admission is served by the shared exact accessor.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| WOS221 | 源 Rage 倍率与精确资源纯状态 | implemented | 2026-08-03 | `combat/rage_generation_state.zr` 保留 `11.5` Pummel 契约；`combat/resource_precision_state.zr` 固化封顶/扣除/历史整数写回门槛，尚未接入 `WorldState` |
| WOS221 | 浮点资源、Rage aura、WOS113 前向迁移 | implementation complete; second review complete; validation pending | 2026-08-03 | `state.zr` appends/reads four WOS113 `fixed6` resource columns and forces historical writers through `resourcePrecisionStateCanEncodeVersion`; Pummel covers 11.5, 1.5 spend, 20.7 aura stack, cap and zero RNG. Second review confirmed tail order and consumer ownership; `cc_contract_codegen.mjs --check`, Node guard syntax, targeted writer-boundary assertion and `git diff --check` passed. The complete WOS221 Node guard had no result before it was stopped under the no-wait rule; canonical `zr_vm:project` execution remains unavailable, so no dynamic result is claimed. |
