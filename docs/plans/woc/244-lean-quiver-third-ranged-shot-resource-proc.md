---
title: WOS244 Lean Quiver third ranged shot resource proc
status: implementation_complete_pending_coordinator_dynamic_validation
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS244 Lean Quiver third ranged shot resource proc

## Objective

Project the source-pinned Hunter selection `hun_r11_efficiency` / Lean Quiver:
every third completed Hunter ranged-shot cast restores 20 mana, without adding
an extra RNG draw or making the proc depend on projectile impact.

## Source contract

- Source choice row `hun_r11_efficiency` declares proc `hun_lean_quiver` with
  `castNth(n: 3, abilities: hunterRangedShotAbilityIds)` and one
  `resource(amount: 20)` response.
- The trigger runs in source `onCastCompleted`, after the cast's successful
  completion and not at projectile impact; eligible shots include the source
  Hunter ranged-shot allowlist and free casts do not feed the counter.
- The generated WOC talent-proc catalog already retains Lean Quiver and the
  generic `combat/talent_proc_state.zr` already implements deterministic
  `castNth`, counter, response-plan and resource response semantics.

## Existing WOC support and gap

- WOC has serializable resource, known-ability, command, projectile and
  talent-selection state, plus a generic talent proc state machine.
- The world reducer currently consumes that generic proc system for selected
  Mage paths, but no Hunter cast boundary invokes the retained Lean Quiver
  definition. This is a WOC integration gap, not a ZirconEngine or ZrVM
  capability gap.

## Delivery order

1. Pin source proc timing, eligibility, free-cast counter exclusion and
   response contract against the fixed source commit and generated proc catalog.
2. Add a bounded, owner-scoped WOC runtime row for the Lean Quiver `castNth`
   counter. The source definition has no internal cooldown, so no synthetic
   ICD state is introduced.
3. Invoke the generic `castNth` evaluator exactly once after successful
   eligible Hunter ranged-shot completion, apply its 20-resource plan via the
   ordinary capped-resource writer, and retain no extra RNG path.
4. Cover first/second/third cadence, allowlist rejection, failed/free cast
   exclusion, cap clamping, snapshot, selection removal and projectile-in-flight
   independence through a `zr_vm:project` fixture. Existing WOS238/WOS243
   slot/typed command coverage reaches this same shared completion boundary.
5. Run static source/contract guards and independent second review. Dynamic
   acceptance remains coordinator-owned and is not polled here.

## Non-goals

- No ZirconEngine or ZrVM change.
- No projectile-impact-triggered resource refund.
- No generic expansion of every dormant source proc before its owning WOC
  gameplay slice is implemented.

## Output record

| Slice | Scope | State | Date | Evidence |
| --- | --- | --- | --- | --- |
| WOS244 | Lean Quiver third ranged cast resource response | implementation_complete_pending_coordinator_dynamic_validation | 2026-08-03 | Source choice and `onCastCompleted` timing, generated proc catalog and generic proc state support were mapped. WOC now persists an owner-scoped `castNth` counter in schema 116, rejects that new state from historical encoders and decodes historical snapshots empty. Successful projectile release and Counter Shot both call the generic evaluator exactly once; rejected casts and projectile impact do not. The dedicated project fixture covers first/second/third cadence, rejection, temporary deselection, snapshot restore, cap, allowlist and free-cast exclusion. Second review required two forward fixes: the fixture now drives a third Counter Shot through its real cast completion path and proves its one-time +20 response, and it decodes an actual schema 115 snapshot with empty Lean Quiver rows. The forward fixes passed static revalidation and the independent revision review reported no remaining actionable P1/P2. Dynamic acceptance remains coordinator-owned. |
