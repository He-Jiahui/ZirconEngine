---
title: WOS245 Redline Draw third ranged shot cooldown refund
status: implementation_complete_pending_coordinator_dynamic_validation
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS245 Redline Draw third ranged shot cooldown refund

## Objective

Project the source-pinned Hunter level-20 selection `hun_r20_rapid_killing` /
Redline Draw: every third completed eligible ranged shot reduces Fevered Draw
(`rapid_fire`) cooldown by five seconds, with an eight-second proc ICD.

## Source contract

- The source row declares `hun_redline_draw` as
  `castNth(n: 3, abilities: hunterRangedShotAbilityIds, icd: 8)` and a
  `cooldownRefund(rapid_fire, seconds: 5)` response.
- Source `onCastCompleted` excludes a cast that consumed an empower aura,
  ignores matching casts during the active ICD without banking a counter, resets
  the third-count before a response, and has no chance draw for this proc.
- `onCastCompleted` is reached by source auto attacks and casting lifecycle;
  this WOC slice covers the already-retained player ranged cast boundaries.

## Existing WOC support and gap

- WOS244 supplies the source-equivalent retained completion boundaries for
  projectile releases and Counter Shot, and the generated proc catalog retains
  Redline Draw as option code `53`, nth `3`, ICD `8`, and cooldown response
  `rapid_fire/5`.
- WOC already has authoritative cooldown expiries, Rapid Fire, deterministic
  time and generic `TalentProcRuntimeState` ICD evaluation.
- WOC lacks only owner-scoped persistent Redline counter/ICD state, its
  authoritative tick, and the response-plan-to-absolute-expiry bridge. This is
  a game-layer integration gap, not a ZirconEngine or ZrVM capability gap.

## Delivery order

1. Pin source selection, complete-cast/ICD ordering and generated catalog
   fields at the fixed source commit.
2. Add bounded owner-scoped Redline counter and remaining-ICD rows to a new
   schema version, retaining source proc state across temporary deselection.
3. Tick the ICD from the existing deterministic world tick; invoke generic
   `onCastCompleted` at WOS244's completion boundaries; apply only the planned
   5-second `rapid_fire` refund to the existing absolute cooldown expiry.
4. Cover threshold, active-ICD ignored casts, expiry, zero/short cooldown,
   selection removal, snapshot and no-RNG behavior with a `zr_vm:project`
   fixture.
5. Run source-pinned static guards and independent second review. Dynamic
   acceptance remains coordinator-owned and is not polled here.

## Non-goals

- No engine or ZrVM change.
- No projection of unimplemented Hunter auto shot, Wyvern Sting, Multi Shot or
  Volley solely to manufacture a proc event.
- No cooldown refund on projectile impact or on an active Redline ICD.

## Output record

| Slice | Scope | State | Date | Evidence |
| --- | --- | --- | --- | --- |
| WOS245 | Redline Draw third ranged-shot cooldown refund | implementation_complete_pending_coordinator_dynamic_validation | 2026-08-03 | Source choice and `onCastCompleted` timing, generated proc catalog and generic proc state were mapped. The proc projection now retains source `cooldownRefund.seconds`, so all generated cooldown-refund plans use the source duration instead of the unrelated resource amount field. WOC persists owner-scoped Redline counter/ICD rows in schema 117, rejects new rows from historical encoders and decodes schema 116 with empty rows. The authoritative fixed tick ages the retained ICD; successful projectile release and Counter Shot both invoke the generic evaluator once; the planned Rapid Fire response mutates the existing absolute expiry or clears a short cooldown. Second review first found missing Counter Shot fixture execution, then found the initial guard could borrow Concussive assertions. Forward fixes add a real Counter Shot third-cast path and a Counter-only guard slice for the 300-to-295-second refund, counter reset, eight-second ICD and zero RNG. WOS245 static source guard, proc codegen check, WOS242–244 guards and diff check pass; final independent replacement review reports no actionable P1/P2. Dynamic `zr_vm:project` acceptance is coordinator-owned. |
