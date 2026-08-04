---
title: WOS193 Chronomancy direct-heal runtime
status: implemented_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS193 Chronomancy Direct-Heal Runtime

## Scope

Replicate the source Mage Arcane Chronomancy direct-heal entry points that
make WOS192 Power Echo healing reachable: `temporal_mend` and
`temporal_echo`. Temporal Mend is a friendly 30-yard, 2-second cast with
ranked costs and base ranges of 45 / 62-74 (level 5), 70 / 105-125 (level 12)
and 95 / 150-178 (level 18). Temporal Echo is a friendly 30-yard instant
GCD spell with ranked costs and base ranges of 40 / 24-30 (level 8),
60 / 40-50 (level 12), and 85 / 58-70 (level 18); it applies its 15-second
per-caster mark after its primary direct heal.

Both spells reuse the existing WOC direct-heal numeric, crit, health-cap and
healing-threat path. Their primary result is the actual post-absorb,
post-cap healed amount, not the nominal spell roll. An armed Power Echo is
consumed after a spell's primary direct-heal resolution and before the copy;
the copy is `max(1, round(actualHealed * 0.5))`, cannot crit, cannot call a
second on-heal weapon proc, and consumes no extra random value. It still uses
the source-compatible healing-threat path for its actual effective healing.

The Temporal Echo mark is durable state in this milestone so refresh/move,
expiration and snapshots are well-defined. Converting later Mage Arcane damage
through that mark is deliberately deferred to the following Chronomancy damage
conversion milestone; this plan must not invent that unrelated combat rule.

## Source Contract

- `classes.ts` declares Temporal Mend as a Mage/Arcane level-5 friendly heal
  with three source ranks and Temporal Echo as a Mage/Arcane level-8 friendly
  instant direct heal plus a `temporalEcho` effect, also with three ranks.
- `effect_dispatch.ts` resolves a direct `heal` with direct cast-time Spell
  Power scaling, then passes the actual result returned by `applyHeal` to the
  Power Echo tail. The aura is removed before deciding whether a positive copy
  can be applied; the copy calls `applyHeal(..., false, false)`.
- The source applies Temporal Echo's own mark only for the primary cast.
  Recasting moves that caster's mark; its 15-second duration is independent
  of the direct-heal result and Power Echo copy.
- `heal_state.zr` already returns actual effective healing and owns the first
  critical draw, health cap, healing threat and one on-heal weapon-proc call.
  WOS193 must add a narrow no-crit/no-proc resolved-heal path there rather than
  recreating random or threat math in a spell reducer.

## Delivery Order

1. Add WOS193 red source/static contracts and a focused
   `zr_vm:project` entry. Pin all six rank values, friendly/range admission,
   Temporal Echo's instant GCD behavior, its 15-second mark, and Power Echo's
   primary-only tail semantics.
2. Extend `contracts/m4_abilities.json` and the M4 generator's fixed catalog
   cardinality from 83 to 85, regenerate `m4_ability_catalog.zr` and
   `m4_ability_effects.zr`, and add codegen drift checks. Do not hand-edit
   generated files.
3. Add source-pinned Mage/Arcane profile helpers, exact typed-payload matching,
   slot routing, target/range/resource/GCD/cast completion checks, and the two
   reducers. Temporal Mend uses the timed-cast path; Temporal Echo resolves
   immediately but remains on the normal GCD.
4. Make the existing direct-heal bridge return the effective primary result and
   add one shared resolved-copy operation that omits only crit RNG and
   on-heal weapon proc. Route WOS192 Power Echo to it for these two spells;
   leave non-spell heals, HoTs, chain hops and later Temporal damage conversion
   outside the trigger boundary.
5. Store Temporal Echo's per-caster target and remaining duration in validated,
   versioned world state. Define move, refresh, dead/out-of-range rejection,
   expiration and snapshot behavior without changing its later damage-to-heal
   conversion policy.
6. Cover all rank gates, typed/slot casts, friendly/hostile/range rejection,
   resource and GCD behavior, exact primary-plus-copy amount, overheal aura
   consumption, no extra RNG/crit/proc, mark move/expiration, snapshot restore
   and no recursive mark/copy effects. Run static regression and a second
   independent review.

## Exclusions

- Arcane-damage conversion through the Temporal Echo mark, group Temporal
  Cascade, Temporal Barrier, resurrection and rewind mechanics are separate
  source behaviors and require later milestones.
- WOC must not use the untracked standalone
  `combat/power_echo_heal_state.zr` as an implicit contract or alter it without
  its owner's coordination. The accepted reducer belongs at the established
  `heal_state.zr` and world-state boundaries.
- No engine/runtime change is requested: existing WOC ZrVM modules already
  provide the required numeric, state, serialization and command facilities.

## Dynamic Validation

`examples/woc/scripts/woc_game/woc_m4_chronomancy_direct_heal_runtime_tests.zrp`
must run only through `zr_vm:project`. No alternate runtime is permitted.

## Second Review

2026-08-03: second static review confirmed rank-gated direct-heal admission,
effective-primary result forwarding, the no-crit/no-proc Power Echo copy
boundary, and source-scoped Temporal Echo mark persistence. The focused
fixture declares `zr_vm:project`, and `node tools/wos193_resolved_heal_static_guard.mjs`
passed from `examples/woc`. Dynamic ZrVM execution remains pending.
