---
title: WOS190 Overload spell resolution
status: implemented_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS190 Overload Spell Resolution

## Scope

Consume the WOS189 armed Overload state at the current Mage spell resolver
boundaries. The source `consumeOverload` runs after the original cast has
passed its target/cast-start checks and before it bills or applies effects.
It consumes one non-physical, positive-cost spell, rounds every positive
output field and empowered-cone stage min/max by 1.4, and bills
`Math.round(cost * 1.5)`. Source resource billing clamps to zero, while its
Overflowing Power reducer receives the transformed billed cost.

The projection has specialised per-ability reducers instead of a mutable
`ResolvedAbility`; this slice introduces a narrow one-shot resolver transform
and applies it to every currently reachable Mage positive-cost, non-channel
spell:

- M4 Fireball, Frostbolt, Arcane Explosion, Pyroblast, Flamestrike and
  Combustion;
- source-profile Cone of Cold, Ice Barrier and Blazing Barrier;
- source-profile empowered Glacial Front and Dragon's Breath.

Channels intentionally remain outside this slice because source bills and
resolves them from the channel branch before `applyAbility`.

## Source Contract

- `casting_lifecycle.ts#consumeOverload` exits for physical or zero-cost
  abilities and draws no RNG.
- It removes the aura once, scales a copied resolved effect by integer
  `Math.round`, and scales only `min`, `max`, `amount`, `bonus`, `total` and
  `value`; duration, radius and nonpositive fields do not change.
- Cast admission uses the original cost. The transformed bill is allowed to
  exceed current mana and clamps remaining mana to zero.
- The transformed bill, not the clamped delta, flows to Overflowing Power.

## Delivery Order

1. Write a red static/source contract over the transform and every resolver.
2. Add the one-shot transform and permissive source-equivalent bill helper.
3. Apply it at each listed resolver, before output construction and billing.
4. Add source-pinned `zr_vm:project` regression coverage for one-shot,
   rounded output, transformed bill, zero-cost preservation and clamped mana.
5. Run static regression and a second independent review. Dynamic execution
   remains exclusively `zr_vm:project`.

## Status

| Milestone | Scope | Status | Date |
|---|---|---|---|
| WOS190a | Plan and red source contract | completed | 2026-08-02 |
| WOS190b | Resolver transform implementation | implemented_static_validation_pending | 2026-08-02 |
| WOS190b.1 | Forward fix: preserve exact Fireball/Pyroblast base or transformed DoT profiles through flight, landing and schema-93 snapshots; normalize pre-93 snapshots | implemented_static_validation_pending | 2026-08-02 |
| WOS190c | Static regression and second review | completed | 2026-08-02 |

## Dynamic Validation

The package introduced by this slice must run only through `zr_vm:project`.
No alternative script runner is permitted.
