---
title: WOS187 Overflowing Power runtime
status: implemented_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS187 Overflowing Power Runtime

## Scope

Replicate Mage level-20 choice `mag_r20_overflowing_power`. Every successful
mana debit reduces already-active Mage defensive cooldowns by
`cost / maxMana * 10 * manaDefCdrPer10`; the source caps total reduction at
10 seconds in a rolling 30-second internal-cooldown aura.

## Source Transaction

- The effect runs directly after a normal successful mana debit, so it cannot
  reduce the cooldown created later by that same cast.
- Only existing Blink, Ice Barrier, Blazing Barrier, and Greater Invisibility
  cooldown rows are affected. Other cooldowns and absent rows remain untouched.
- The window records its used reduction and remaining duration. At expiry both
  values reset, making the next successful mana debit open a fresh window.

## Delivery Order

1. Add a focused `zr_vm:project` state test and source-pinned static guard.
2. Add bounded per-entity window state with schema 91 encode/decode migration.
3. Centralize retained ability resource debits so every current Mage mana
   ability reaches one selection-gated post-debit reducer.
4. Run static regression and a second review. Dynamic execution remains solely
   owned by `zr_vm:project`.

## Status

| Milestone | Scope | Status | Date |
|---|---|---|---|
| WOS187a | Test contract and plan | completed | 2026-08-02 |
| WOS187b | Runtime implementation | implemented_static_validation_pending | 2026-08-02 |
| WOS187c | Independent secondary review | completed | 2026-08-02 |
| WOS187d | Temporal Reversal centralized debit forward fix | completed | 2026-08-02 |
| WOS187e | Schema 91 writer-admission forward fix | completed | 2026-08-02 |

## Dynamic Validation

`examples/woc/scripts/woc_game/woc_m4_overflowing_power_runtime_tests.zrp`
must run only with `zr_vm:project`; no alternate backend is permitted.

## Review Follow-up

The first independent review found Temporal Reversal's successful mana debit
outside the centralized bridge. WOS187d replaces that debit with the bridge,
adds a direct successful-cast state fixture, and broadens the static guard to
reject every direct caster-resource subtraction other than the bridge itself.
The same inspection found the historical writer whitelist lagging the schema
91 tail; WOS187e admits schema 91 before the dynamic snapshot test runs.
