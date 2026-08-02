---
title: WOS189 Overload arming runtime
status: implemented_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS189 Overload Arming Runtime

## Scope

Implement the source Mage choice ability `overload` as an off-GCD, zero-cost,
instant self buff. A valid cast arms a durable 40% amplifier for 10 seconds
and starts the source 30-second cooldown. The armed state is separately
cancellable and expires on the normal 50 ms simulation age step.

This slice deliberately stops before consuming and scaling a resolved spell.
The source consumes Overload only in `applyAbility`, after target validation
and before billing/effects; WOC has per-ability reducers rather than a mutable
`ResolvedAbility` object. WOS190 will introduce the one-shot transform at
each appropriate resolver boundary without incorrectly turning it into a
standing spell-damage multiplier or applying it to channels.

## Source Contract

- Mage level 14 `overload`: Arcane, instant, cost 0, off-GCD, 30-second
  cooldown, `selfBuff(overload, 0.4, 10)`.
- Casting the zero-cost buff cannot consume a pre-existing Overload because
  source `consumeOverload` exits for zero cost.
- The armed aura is an ordinary self aura and can expire or be cancelled
  before its next qualifying spell.

## Delivery Order

1. Add source-pinned static and `zr_vm:project` state-test contracts.
2. Add schema 92 entity-aligned remaining/value state with legacy defaults.
3. Route exact `overload` casts and cancellation through the existing command
   surface, preserving off-GCD behavior.
4. Add expiry, snapshot, cooldown and selection-gated regression coverage.
5. Run static regression and independent secondary review; dynamic execution
   remains exclusively `zr_vm:project`.

## Status

| Milestone | Scope | Status | Date |
|---|---|---|---|
| WOS189a | Plan and red source contract | completed | 2026-08-02 |
| WOS189b | Arming runtime | implemented_static_validation_pending | 2026-08-02 |
| WOS189c | Static regression and second review | completed | 2026-08-02 |

## Dynamic Validation

The package introduced by this slice must run only through `zr_vm:project`.
No alternative script runner is permitted.
