---
title: WOS246 Deathless Will big-hit speed proc
status: implementation_complete_pending_coordinator_dynamic_validation
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS246 Deathless Will big-hit speed proc

## Objective

Project the source-pinned Hunter level-11 selection
`hun_r11_survival_instincts` / Deathless Will. A post-mitigation hit of at
least 30% maximum health grants the Hunter 1.4x movement speed for four
seconds, then observes a 30-second internal cooldown.

## Source contract

- The fixed source row declares `hun_deathless_will` as
  `bigHitTaken(hpFrac: 0.3, icd: 30)` and a `buff_speed` aura with value `1.4`,
  duration `4`, name `Deathless Will`, and school `nature`.
- Source `dealDamage` has already completed avoidance, mitigation, absorbs and
  survival substitutions before it stores the resulting hit `amount`, updates
  HP, and invokes `onDamageTaken` while the target has not yet been marked
  dead. The threshold therefore uses the retained post-absorb hit amount, not
  the capped HP delta; a fully absorbed hit is zero and cannot trigger.
- Source `onDamageTaken` is deterministic: it rejects below-threshold hits and
  active ICDs, arms the 30-second ICD before applying the response, and takes
  no random draw. Existing proc state keeps ticking across temporary talent
  deselection.

## Existing WOC support and gap

- The generated current proc catalog already carries selection code `45`, proc
  index `7`, threshold `0.3`, ICD `30`, `buff_speed`, `1.4`, and `4`.
- `talent_proc_state.zr` already implements `onBigHitTaken`, response planning,
  and deterministic ICD aging. WOC already persists source-shaped motion auras
  and uses `buff_speed` in retained movement.
- WOC lacks the owner-scoped durable ICD row and the post-settlement player-hit
  bridge. This is an example-game integration gap only; no ZirconEngine or
  `zr_vm:project` backend capability is missing.

## Delivery order

1. Add bounded Hunter owner/ICD state, validation, historical-schema rejection,
   schema-118 encoding and decoding; schema 117 and earlier decode empty rows.
2. Rehydrate the generic runtime from that row and age its ICD from the existing
   deterministic fixed tick, irrespective of current selection.
3. Add one authoritative `applyOfflineDeathlessWillDamageTaken` reducer that
   gates player targets and row-11 selection, invokes `onBigHitTaken` with the
   post-absorb hit amount, persists the armed ICD, and writes its planned aura
   through the existing motion-aura writer using selection code 45, `nature`,
   `buff_speed`, `1.4`, and four seconds.
4. Invoke the reducer after HP settlement and before death handling in the
   retained hostile-melee, fall-damage and direct-melee post-hit-reflection
   paths; also route the existing effective-damage helper through it for future
   player-target callers. Do not wire player-to-mob projectiles, dots, pets, or
   health-only bookkeeping.
5. Add a `zr_vm:project` fixture and static source guard for threshold,
   pre-armed ICD, no-RNG response, aura expiry, deselection, snapshot migration,
   full-absorb rejection and the actual hostile-melee boundary. Run static and
   generated-catalog checks; coordinator-owned dynamic acceptance is not polled.

## Non-goals

- No engine, ZrVM, input protocol, or ability-catalog change.
- No synthetic Hunter skill and no new damage subsystem.
- No proc from raw pre-mitigation damage, fully absorbed damage, outgoing hits,
  or a target that has already been marked dead.

## Output record

| Slice | Scope | State | Date | Evidence |
| --- | --- | --- | --- | --- |
| WOS246 | Deathless Will big-hit movement proc | implementation_complete_pending_coordinator_dynamic_validation | 2026-08-03 | Source `dealDamage`/`onDamageTaken` ordering, generated proc row and generic big-hit runtime are projected into owner-scoped schema-118 ICD rows. The reducer receives post-settlement damage from effective damage, hostile melee, falling and direct-melee reflected damage before death processing; it adds the source-shaped Nature `buff_speed` aura without RNG. The `zr_vm:project` fixture covers threshold, zero absorbed damage, ICD, aura expiry, temporary deselection, snapshot migration and the real hostile-melee boundary. WOS246 source guard, proc-catalog codegen check, WOS242–245 guards and diff check pass. Dynamic fixture acceptance remains coordinator-owned and is not polled. |
