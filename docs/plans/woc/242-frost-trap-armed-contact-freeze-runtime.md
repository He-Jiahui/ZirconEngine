---
title: WOS242 Frost Trap armed contact freeze runtime
status: implementation_complete_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS242 Frost Trap armed contact freeze runtime

## Objective

Project the source-pinned Hunter choice-granted `frost_trap` ability into WOC
M4 as a self-placed, 1.5-second arming, 60-second-lived trap that consumes on
the first hostile movement contact and applies a three-second Frost controlled
stun.

## Source contract

- Source ability: `src/sim/content/talent_abilities_v2_a.ts` at
  `5ef9f7cb21cd8875b6d2c49701015dfcd78de35a`: Hunter level 10, 35 cost,
  instant, 30-second cooldown, Frost, no target, one `aoeRoot` effect with
  `duration: 3`, `radius: 3`, `stun: true`, and trap `{ armTime: 1.5,
  lifetime: 60 }`.
- Source choice row `hun_r8_frost_trap` grants the ability.
- Source runtime: `src/sim/combat/hunter_trap.ts`. A trap is created at the
  owner position; a new trap replaces that owner's prior hunter trap; it
  cannot trigger while arming; its first hostile `segmentTouchesAnnulus`
  contact after arming enters combat, consumes the trap, and applies the
  `controlledStun` DR-bucket duration. It draws no RNG.

## Existing WOC support and gap

- WOC already owns fixed-tick ground-effect lifecycle, prior/current movement
  coordinates, the source-equivalent segment/annulus geometry helper, motion
  aura serialization and controlled-stun handling.
- The retained ground-effect rows do not yet carry the source hunter-trap
  rider fields (owner replacement identity, arming timer, freeze duration and
  consumed flag). This is a WOC game-projection gap, not a ZirconEngine or
  ZrVM capability gap.

## Delivery order

1. Retain Frost Trap at a new M4 tail index and regenerate source-pinned
   catalog, Zr projection, coverage and any affected cardinality checks.
2. Add a bounded, separately serialized hunter-trap rider table alongside the
   existing damage-only ground-effect rows. Preserve source-compatible owner
   replacement, arm/lifetime/contact transitions and a transient triggered
   transition without
   widening Consecration's row contract.
3. Add the Hunter reducer and slot/typed routes; enforce zero range, 35 cost,
   normal GCD and a 30-second cooldown, with replacement of only the owner’s
   previous Frost Trap.
4. Cover schema-114 empty-rider compatibility, arm delay, crossing contact,
   stationary in-radius contact, first-only consumption, owner replacement
   including full-table replacement, lifetime expiry, snapshot, invalid
   class/level/admission and typed parity through a `zr_vm:project` fixture.
5. Run source-pinned static guards and independent second review. Dynamic
   acceptance remains coordinator-owned and is not polled here.

## Non-goals

- No ZirconEngine or ZrVM change.
- No synthetic projectile or generic instant target-area nova fallback.
- No client visual/shimmer event implementation beyond the authoritative
  retained trap state.

## Output record

| Slice | Scope | State | Date | Evidence |
| --- | --- | --- | --- | --- |
| WOS242 | Frost Trap armed first-contact controlled stun | implementation_complete_static_validation_pending | 2026-08-03 | Source-pinned M4 catalog retains Frost Trap at tail index 116 with its direct `TALENT_ABILITIES_V2_A` owner. The generator now accepts the source `stun` flag and preserves nested trap timing as scalar read-only metrics. WOC schema 115 serializes a bounded, separate Hunter-trap rider table without widening Consecration's damage row. The Hunter-only level-10 reducer applies normal GCD/resource/cooldown semantics, replaces only the caster's prior trap, then the fixed ground phase applies source arm/lifetime/first swept-contact ordering and a Frost stun before immediate consumption. After independent review identified two P2 test gaps, the fixture now also proves schema-114 empty-rider decode and full-capacity own-owner replacement while retaining every other owner row and normal resource/GCD/cooldown effects. M4 generators, syntax checks and the source-pinned static guard pass. The revised independent second review reported no actionable P1/P2; dynamic acceptance remains coordinator-owned and is not polled here. |
