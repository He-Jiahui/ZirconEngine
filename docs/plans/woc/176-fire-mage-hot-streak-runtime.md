---
title: WOS176 Fire Mage Hot Streak runtime
status: implemented_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS176 Fire Mage Hot Streak Runtime

## Scope

Close the first executable Fire Mage loop using the generated WOS175 ability
definitions: Pyroblast casts as a Fire projectile with its direct hit and
twelve-second damage-over-time effect; Combustion is an off-GCD self state;
and a committed Fire critical hit applies the source `Heating Up` / `Hot Streak`
reducer. The reducer creates the WOS174 free-cost and instant-cast modifiers
scoped to Pyroblast and Flamestrike.

## Source invariants

- A Fire builder non-critical hit removes `Heating Up`.
- A critical builder hit without `Heating Up` starts its ten-second window.
- A second critical builder hit removes `Heating Up` and applies both scoped
  modifiers for twelve seconds, including while Combustion is active.
- While Combustion is inactive, each critical builder reduces a positive
  Combustion cooldown by one second; Combustion itself grants its ten-second
  guaranteed-critical state and has a 120-second cooldown.
- Pyroblast's free-cost modifier is billed only at successful cast completion;
  its instant modifier is consumed when it changes a positive cast time to
  zero. The cast must not consume either modifier on validation failure.
- The existing projectile hit still performs exactly its normal resist draw
  before its normal damage/crit draw. The Fire reducer runs after that result,
  and performs no RNG work.

## Design and persistence

`WorldState` retains a per-entity `entityCombustionRemaining` clock. Schema 86
appends that clock after schemas 84 (Heating Up/Hot Streak) and 85 (next-cast
modifiers); schema 85 and earlier decode to zero Combustion time. Combustion
cooldown remains in the existing per-ability cooldown partition, so its
source CDR modifies that partition rather than introducing a duplicate clock.

The runtime applies the pure `combat/fire_mage_state` plan only after the
projectile transaction has committed its critical result. Modifier identity
codes are stable WOS-owned opaque values supplied by the generated Fire
contract; only the source-defined ability scope is observable gameplay state.

## Delivery order

1. Extend the checked-in Fire contract generator with the two opaque modifier
   identities and add schema-86 Combustion state, migration, expiration and
   static snapshot coverage.
2. Add exact Pyroblast validation, hard-cast completion, projectile landing and
   DoT scheduling by reusing Fireball's existing path without changing its
   RNG ordering.
3. Apply the Fire hit reducer and WOS174 modifier consumption at the existing
   successful-cast boundaries; add Combustion's off-GCD command path.
4. Add static guard coverage and defer candidate execution only to
   `zr_vm:project`.

## Boundaries

- Do not make generic M4 ownership of the Fire proc reducer or an arbitrary
  next-cast aura protocol.
- Do not broaden `castAt` for Flamestrike in this slice; it requires its own
  source-positioned area transaction.
- Do not generalize Dragon's Breath `empoweredCone` behavior.
- No local Cargo, native host, or alternate VM is dynamic validation evidence.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| WOS176a | Combustion durable state and source migration | implemented_static_validation_pending | 2026-08-01 | schema 86 encode/decode/migration and WOS176 static guard |
| WOS176b | Pyroblast / Hot Streak runtime transaction | implemented_static_validation_pending | 2026-08-01 | content generator, WOS176 static guard, world-state VM entrypoint |

## 二次审查

2026-08-01: reviewed the completed WOS176 implementation. The schema-86
decoder validates Fire state only after the historical Combustion migration;
Combustion leaves the normal GCD untouched; and both Fire projectiles preserve
one resistance draw plus two timed-spell draws before their no-RNG proc reducer.
The executable test package remains pending `zr_vm:project` availability.
