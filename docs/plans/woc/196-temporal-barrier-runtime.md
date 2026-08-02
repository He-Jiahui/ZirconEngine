---
title: WOS196 Temporal Barrier runtime
status: implemented_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS196 Temporal Barrier Runtime

## Scope

Replicate the source Arcane Mage `temporal_barrier` cast: an instant,
targeted friendly absorb shield with source rank profiles `(55, 10s)`,
`(100, 10s)`, and `(160, 10s)`, resource costs `50/75/105`, 30-yard reach,
and a 12-second cooldown. The ability is Arcane-specialization-only, fills
the source shared personal-barrier slot, and coexists with the existing Mass
Barrier cooldown coupling.

The authoritative numeric contract is the generated M4 ability row, not
duplicated constants in Zr. Mutable shield state stays in the existing
source-keyed `offlineAbsorb*` queue, whose replacement and newest-first
damage-consumption semantics already mirror `ctx.applyAura`.

## Source Contract

- `src/sim/content/classes.ts#ABILITIES.temporal_barrier` specifies an instant
  30-yard Arcane-friendly cast, 12-second cooldown, and one `absorb` effect
  per rank: `55/100/160` for 10 seconds.
- `src/sim/combat/effect_dispatch.ts` resolves an `absorb` effect by calling
  `applyAura` on the selected friendly target (or the caster fallback). A
  matching target/source/ability aura is replaced, not stacked.
- `src/sim/combat/fire_mage.ts` puts `temporal_barrier` in Arcane's shared
  personal-barrier slot. Existing WOC Mass Barrier already starts that
  personal-barrier cooldown for an Arcane caster and must keep doing so.

## Delivery Order

1. Add `temporal_barrier` to the retained M4 source extraction and regenerate
   both JSON and Zr ability contracts. Extend the shared absorb-profile probe
   through its rank-one and rank-three data.
2. Add a narrow Temporal Barrier resolver around the existing friendly target,
   spellbook admission, haste-GCD, ability-cooldown, and absorb-queue
   primitives. It must use the generated rank/cost/cooldown/range/effect
   metrics and must not consume random values.
3. Route the exact typed `cast` payload and spellbook slot command to the same
   resolver. An omitted target follows the source-compatible selected-target,
   then self fallback; invalid/hostile/out-of-range targets reserve nothing.
4. Add a `zr_vm:project` fixture for source-spec rejection, rank transitions,
   typed/slot parity, refresh-not-stack, cooldown/GCD/resource settlement,
   absorb consumption, and encode/decode stability. Dynamic execution is only
   attempted through the ZrVM plugin backend.
5. Run the red/green static contract checks, generator checks, prior
   Chronomancy compatibility guards, then independently inspect each reducer
   and endpoint before coordinator wakeup. An unavailable dynamic plugin only
   defers accepted closeout; it does not block forward work.

## Exclusions

- Temporal Cascade, Temporal Rewind, and other later Chronomancy spells are
  separate milestones.
- General aura rendering, PvP tuning, and unrepresented incoming-damage
  sources do not expand this bounded offline runtime slice.
- No ZirconEngine change is needed: the plugin-hosted ZrVM target, generated
  module imports, fixed-state serialization, bounded array rows, and command
  dispatch surface already support this feature.

## Dynamic Validation

`examples/woc/scripts/woc_game/woc_m4_temporal_barrier_runtime_tests.zrp`
must run only through `zr_vm:project`. Do not substitute a local runtime.

## Second Review

2026-08-02: reviewed the source effect dispatch, personal-barrier slot,
absorbed-damage callback and WOC reducer/codec paths a second time. The
implementation keeps source-keyed replacement and newest-first consumption,
sets resource/GCD/cooldown only after all admission checks, preserves Arcane
specification through `catalogAdmission`, and routes Warded shield-consumed
procs to the shielding Mage while restricting the personal-barrier reduction
to self-owned barriers. No P1/P2 issue was found. Static guard, M4 JSON/Zr
generation checks, WOS193-195 regression guards, JavaScript syntax checks,
manifest JSON parsing and targeted `git diff --check` passed. Dynamic execution
remains pending the unavailable `zr_vm:project` plugin and was not replaced.
