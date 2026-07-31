# M4 combat source contract

This contract pins the dependency-independent ZrVM source slice for the M4
combat, aura, casting, effect and auto-attack systems. The behavioral source is
`dev/world-of-claudecraft` at commit
`7c10f280eec380e9877e66ce16333089e171fe42`.

## Temporary scalar boundary

Each module exposes one self-contained scalar function:

```text
metric(kind: int, first: float, second: float, third: float, fourth: float): float
```

This is a ZrVM compatibility shape, not a second gameplay implementation.
Plugins 08 owns the missing reliable helper dispatch and object/container ABI.
Rust must not reproduce any metric or scenario outcome.

## Implemented source slices

- `combat/rules.zr`, kinds 0-14: spell/melee hit tables, directional swing
  miss, armor, rage, threat, effective damage/healing and resource bounds.
- `combat/auras.zr`, kinds 0-14: classic 40-tick regeneration, consumables,
  timers, physical bleed amplification, DoT/HoT/polymorph, aura tick timing and
  the exact 16-kind friendly-NPC hostile-aura rejection set. Kinds 13-14 pin
  non-stunned resource-sap restoration and the 35%-HP Second Wind gate.
- `combat/casting.zr`, kinds 0-13: 20 Hz cast progress, the 0.4s queue
  window, 0.5s timed pushback, 25% channel pushback, cancel/completion gates,
  drain healing and haste/tongues duration.
- `combat/effects.zr`, kinds 0-9: direct/finisher/judgement/AoE damage,
  sunder stacking and threat, ground-pulse draw counts, exclusive forms,
  combo reset and weapon-strike combo award.
- `combat/auto_attack.zr`, kinds 0-12: swing timing, ranged/dead-zone
  selection, the single-roll melee table, Overpower, melee and ranged damage,
  crit suppression, mitigation, cadence, queued cost and melee pre-aggro.
- `combat/affixes.zr`, kinds 0-7: hostile proc guards, rampage, periodic
  damage rounding, Hit-rating conversion and effective spell-hit caps.
- `combat/class_scenarios.zr`, kinds 0-5: effective healing, absorb order,
  healing-threat fan-out, crit vulnerability, ground pulses and combo caps.
- `combat/encounters.zr`, kinds 0-5: phase/final-stand thresholds, split
  damage, ward quorum, Drowned Litany phase gates and pulse cadence.

The scalar compatibility layer is now supplemented by focused state modules:

- `combat/ability_admission.zr` owns the upstream cast guard order, billing,
  cooldown/GCD, queue, next-swing, form, seal and target-projection transitions.
- `combat/casting_state.zr` owns 20 Hz cast/channel progress, tail-queue retry,
  pushback, cancellation and completion cleanup.
- `combat/aura_state.zr` and `combat/regeneration_state.zr` own aura replacement,
  reverse periodic walks, expiry/recalc, classic two-second regeneration and
  timer/combo expiry.
- `combat/spell_scaling.zr` owns direct, channel, DoT/HoT and AoE coefficient
  routing from the pinned source vectors.
- `combat/ground_aoe_state.zr` owns immediate-before-enqueue dispatch, reverse
  20 Hz effect draining, the strict expiry boundary, stable spatial-target
  traversal, source/LoS filtering and immediate-versus-deferred threat inputs.
- `combat/damage_state.zr` and `combat/heal_state.zr` own the ordinary
  post-mitigation damage route and direct/periodic healing routes, including
  modifier/absorb order, combat/threat/tap side effects, cast interruption,
  reflect/Frenzy recursion, healing-threat fan-out and exact RNG guards.
- `combat/auto_attack_state.zr` owns melee and projectile auto-attack state,
  queued Reaver/Gutting strikes, miss/dodge/crit cadence, dead-zone/LoS/facing
  gates, projectile-arrival RNG and thorns/spiked-hide retaliation call order.
- `combat/effect_numeric_dispatch_state.zr`,
  `combat/effect_aura_dispatch_state.zr` and
  `combat/effect_world_dispatch_state.zr` partition every one of the 19 effect
  types present in the 21-ability catalog into exactly one owner. They preserve
  range-before-crit RNG, hybrid DoT/HoT scaling, seal consumption, PvP fear and
  controlled-stun DR, reverse imbue/form exclusion, sunder miss/threat, stable
  AoE traversal, taunt branches, summon requests and synchronous ground-pulse-
  before-enqueue delegation.
- `combat/effect_sequence_state.zr` owns the outer run-effects envelope: stealth
  break, Shadowform removal for heal/HoT, catalog-order dispatch, target
  invalidation and combo reset after the complete effect list.
- `combat/death_state.zr` owns common death teardown, threat/forced-target
  cleanup, player/pet/mob corpse branches, tapper credit and the downstream M5
  reward call points without implementing progression math early.
- `combat/mob_swing_state.zr` and `combat/mob_swing_affix_state.zr` preserve the
  shared miss/dodge roll, hit-only weapon/crit draws and source-ordered affix
  cascade. The corrected six-round contract consumes 30 affix draws, not 18.
- `combat/spell_resist_state.zr` owns the one-draw spell hit/resist gate and
  makes hit rating alter only the threshold.
- `combat/drowned_litany_state.zr` owns the bell-rope, cantor phase, final bell
  and initial Reliquary Rite state used by the pinned scenario.
- `combat/nythraxis_state.zr` owns a 20 Hz full-pull projection for
  Gravebreaker, Raise Fallen, the 70% transition, three no-replacement Soul
  Rend selections, wardstone interruption, Final Stand and death/lockout call
  points. The pinned normal scenario consumes four encounter draws: one weapon
  range draw plus three `rng.int` mark selections.

Most state projects remain source evidence until they integrate with the real
M2 transaction boundary. Direct CLI probes provide selective evidence only:
`mob_swing_state` and `drowned_litany_state` return `1`/exit `0` in fresh
interpreter and binary modes;
the Nythraxis project compiles both reachable modules but stops at the open
Plugins 08 object-field failure before its contract assertions. No selective
probe changes the real-M2 0/16 acceptance count.

The scenario contracts cover all sixteen M4 names and preserve their
statement-order branches. They use scalar event/draw counts because the
production lossless array bridge is still an open Plugins 08 dependency.

## Current evidence

Before the latest scenario-to-rule convergence edits, the following projects
returned result `1`, process exit `0`, in both interpreter mode and a newly
compiled binary mode:

- `woc_combat_rules_tests.zrp`;
- `woc_auras_tests.zrp`;
- `woc_casting_tests.zrp`;
- `woc_effects_tests.zrp`;
- `woc_auto_attack_tests.zrp`;
- `woc_affixes_tests.zrp`;
- `woc_class_scenarios_tests.zrp`;
- `woc_encounters_tests.zrp`.

That last-known-green matrix was eight projects by two execution modes, sixteen
passing runs. `auras`, `casting` and `auto_attack` now reuse their generic rule
entries instead of repeating scenario-local arithmetic, so a fresh current-
source matrix is required when the ZrVM CLI is available again. The old result
is not current-source acceptance. Source coverage remains 16/16, while
accepted real-M2 exact-golden coverage is 0/16. M4 remains open until all named
scenario traces run twice through the real M2 transaction boundary and exactly
match the pinned goldens, including state, event-window and RNG digests.

Current non-Cargo validation parses 61 focused Zr project descriptors and
checks all 15 MJS tools. The seven non-inventory generators remain green from
their latest focused runs, and the generated effect-type owner partition has 19
members with no gap or overlap. The three
dispatch modules are 675, 679 and 603 lines respectively, with balanced braces
and no trailing whitespace. A scoped all-WOC text scan covered 351 files with
zero trailing-whitespace matches. These are static/source checks only; no
current ZrVM execution result is inferred from them.

After correcting `encounters.zr` from two conceptual mechanics to four actual
draws, `woc_encounters_tests.zrp` returned `1` and exit `0` in both interpreter
and binary modes. `woc_m4_nythraxis_state_tests.zrp` independently compiled two
modules from an absent output directory, then failed with
`GET_MEMBER: missing member 'combatData'` even when the single state class had
33 fields, three container references and `combatData` as field zero. The same
CLI also reproduces `K { a: A, b: B }` losing field `b`; the canonical open
Plugins 08 failure records the exact commands and acceptance criteria.

`woc_m4_effect_world_dispatch_state_tests.zrp` now also compiles all five
reachable modules after moving the integer-to-float conversion outside its
helper call. Interpreter and binary execution remain RED at the first
state-field read: `GET_MEMBER: missing member 'source'` in `contractTest`. This
is additional evidence for the same Plugins 08 object-shape failure, not M4
dynamic acceptance.

A current-source focused compile matrix now reaches 11 admission/aura/damage/
death/effect/ground/heal/resist projects. Numeric dispatch also compiles five
modules after its five direct call-casts were split through typed locals, then
hits the same first-field `source` RED in interpreter mode. Ability admission
binds cooldown arrays before indexed writes and passes its complete contract in
fresh interpreter and binary runs (`1`, exit `0`). Its form-gate fixture now
supplies enough resource to pass the pinned source's earlier resource guard,
so the asserted `-18` genuinely tests the later shapeshift restriction.
Effect-sequence ordering and the one-draw spell-resist gate also pass fresh
interpreter and binary runs (`1`, exit `0`). Aura state remains RED before its
contract assertions: the ZrVM C core asserts that both equality operands are
strings when validating an explicitly typed custom-object string field, even
after all `Array<string>` comparisons are explicitly cast.
Casting lifecycle also passes fresh interpreter and binary runs after its
three-condition tail-queue predicate is inlined at the two class-method call
sites that cannot forward-resolve a later free helper. The generated ability
catalog independently returns `1`/exit `0` in fresh interpreter and binary
modes. Regeneration,
damage, healing, auto attack, ground AoE and aura/numeric/world dispatch remain
RED on missing declared fields; mob-affix/aura state hit the object-string C
assertion, and spell scaling exits with Windows access violation
`-1073741819` before a structured result.

The eight dependency-independent scalar contracts now pass a fresh complete
matrix from absent binary outputs: affixes, auras, auto attack, casting, class
scenarios, combat rules, effects and encounters each return `1`/exit `0` in
interpreter and binary modes (16/16 runs). This is current-source focused rule
evidence only; it does not replace the sixteen real-M2 double-run trace and
golden comparisons.

A read-only full-frame probe also recorded all 51 pinned reference scenarios;
projecting each full trace back to the committed golden shape produced 51 exact
matches. The generic `wtr1_encode.mjs` path then encoded all sixteen M4 traces,
recomputed every full-state digest and passed `wtr1_verify.mjs` exactly. That
exercise found 13 values present only in hidden frames, now appended after the
965 stable golden-visible symbol IDs. These binaries are reference-derived wire
fixtures, not ZrVM gameplay output, so they do not change the 0/16 real-M2
acceptance count.
