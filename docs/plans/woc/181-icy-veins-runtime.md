---
title: WOS181 Icy Veins runtime
status: implemented_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS181 Icy Veins Runtime

## Scope

Replicate the source Mage Frost signature `icy_veins` in the retained offline
world: normal-GCD self cast, Frost-spec admission, two independently
cancellable source aura identities, spell-haste projection, cast-shield
pushback immunity, cooldown, persistence, and typed/slot command routing.

## Source Transaction

- `icy_veins` is Mage/Frost level 12, zero cost, instant, Frost school, and
  has a 180 second cooldown. Its normal GCD is calculated before its auras
  take effect.
- The primary source aura id is `icy_veins` with `buff_spellhaste` value 0.3
  for 10 seconds. The companion id is `icy_veins_cast_shield` with
  `cast_shield` value 1 for 10 seconds. `cancelAura` removes either id alone;
  it neither refunds nor changes the cooldown.
- Active spell-haste auras are additive before the cast/GCD divisor. A live
  cast shield makes `pushbackCast` a no-op. It does not prevent a silence,
  stun, or school lockout from cancelling a cast.

## Delivery Order

1. Add schema 90 retained state for the two Icy Veins aura lifetimes with
   forward-only schema 89 migration defaults and strict column validation.
2. Route Icy Veins through source admission, slot/typed commands, pre-buff
   GCD, cooldown, the shared Mage haste multiplier, cast-shield projection,
   independent cancellation, and fixed-tick expiry.
3. Add a focused `zr_vm:project` package plus source-pinned static guard;
   perform a second review after implementation. Do not substitute another
   runtime when ZrVM is unavailable.

## Status

| Milestone | Scope | Status | Date | Evidence |
|---|---|---|---|---|
| WOS181a | Icy Veins source transaction | implemented_static_validation_pending | 2026-08-01 | `wos181_icy_veins_runtime_static_guard.mjs`; generator checks; `zr_vm:project` pending |

## Secondary Review

Reviewed after implementation against source commit
`5ef9f7cb21cd8875b6d2c49701015dfcd78de35a`. The source's primary and
companion aura identities map one-to-one to schema 90 timers. The retained
spell-haste contribution is additive with existing Arcane Power/state haste,
and cast shielding is derived on every `CastState` load so it cannot survive
expiry or cancellation as a persisted flag. Schema 89 writes reject live Icy
Veins to avoid loss; schema 89 reads migrate the new timers to zero. Slot and
typed routes use the existing Frost school-lockout and catalog/spec admission.
No further static review findings remain.

Dynamic acceptance remains deferred solely because the required
`zr_vm:project` backend is unavailable; no alternate runtime was substituted.
