---
title: WOS243 Improved Concussive appended slow runtime
status: implementation_complete_pending_coordinator_dynamic_validation
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS243 Improved Concussive appended slow runtime

## Objective

Project the source-pinned Hunter selection `hun_r8_improved_concussive` into
the existing Concussive Shot projectile landing: retain the base 50% slow and
then dispatch the source-appended 30% slow for four seconds. Both source
effects identify as `concussive_shot_slow` from the same caster, so the latter
replaces the former and the resulting single motion row has `mult: 0.3`.

## Source contract

- Source row: `src/sim/content/choice_rows_classic.ts` at
  `5ef9f7cb21cd8875b6d2c49701015dfcd78de35a`, id
  `hun_r8_improved_concussive` / Pinning Barb.
- It modifies `concussive_shot` with one appended effect
  `{ type: 'slow', mult: 0.3, duration: 4 }`; it neither changes cost,
  cooldown nor the native projectile's base effect rows.
- Source modifier construction appends `addEffects` after native effects in
  `src/sim/content/classes.ts`; the normal effect dispatch resolves each
  resulting slow in that source order. `applyAura` removes same-id,
  same-source rows before appending, therefore the appended row replaces the
  native `0.5` row rather than coexisting with it.
- The existing WOC choice/modifier catalog already retains the nested slow DTO
  as type code `2`, number fields `duration` and `mult`.

## Existing WOC support and gap

- WOS238 already retains Concussive Shot's delayed physical projectile and
  base `slow(0.5, 4)` at impact.
- `combat/talent_added_effect_state.zr` already reads source-selected nested
  DTOs but exposes only the Arcane Shot DoT accessor to its world consumer.
- The Concussive Shot landing path now consumes the typed nested slow tail.
  This closed a WOC game-layer integration gap; no ZirconEngine or ZrVM
  capability was required.

## Delivery order

1. Pin the selection/add-effect source contract and verify the generated
   modifier catalog carries the Hunter row unchanged.
2. Add typed nested-slow accessors without changing the existing encoded
   modifier representation.
3. At Concussive Shot impact, resolve appended slow rows after the base source
   slow, using the existing same ability/kind replacement row so 0.3 replaces
   0.5 exactly as the source same-id aura replacement does.
4. Cover no-selection baseline, selected 0.3 replacement, launch snapshot
   across selection removal, projectile fizzle, slot/typed parity and selection
   removal via a `zr_vm:project` fixture.
5. Run source-pinned static guards and independent second review. Dynamic
   acceptance remains coordinator-owned and is not polled here.

## Non-goals

- No engine or ZrVM change.
- No synthetic direct hit, cooldown reduction or base slow rewrite.
- No broad generic nested-effect dispatcher beyond the typed slow path needed
  for this already-retained Hunter selection.

## Output record

| Slice | Scope | State | Date | Evidence |
| --- | --- | --- | --- | --- |
| WOS243 | Improved Concussive appended landing slow | implementation_complete_pending_coordinator_dynamic_validation | 2026-08-03 | Source choice/modifier, source aura replacement (`concussive_shot_slow` same-source replacement), generated nested slow DTO and current WOS238 projectile snapshot boundaries were mapped before implementation. The typed Talent V2 projection now exposes slow type/duration/mult and verifies source row 42 as `slow(0.3, 4)`. Concussive landing resolves that snapshot tail after native `slow(0.5, 4)` using the existing same ability/kind row replacement, leaving one `0.3` row. The motion projection now takes the minimum valid serialized slow row value, with the historical supplied 0.5 fallback retained only for older zero-value rows. Independent review exposed that generic slow refresh lacked source identity; the forward fix makes slow rows source-scoped, retaining separate casters while refreshing same-caster rows, and adds a two-caster regression. The dedicated `zr_vm:project` fixture covers the unchanged baseline, selected slot/typed landing, launch snapshot after selection removal and snapshot restore, fizzle, exact row identity and resulting 0.3 movement multiplier. Node syntax, M4 generator/coverage checks, CC contract check and source-pinned WOS242/WOS243 static guards pass. A corrected independent second review returned no remaining actionable P1/P2; dynamic acceptance remains coordinator-owned and is not polled here. |
