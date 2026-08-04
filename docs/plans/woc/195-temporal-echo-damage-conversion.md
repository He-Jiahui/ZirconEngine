---
title: WOS195 Temporal Echo damage conversion
status: implemented_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS195 Temporal Echo Damage Conversion

## Scope

Make WOS193's durable individual Temporal Echo mark perform the source
Chronomancy conversion for every represented WOC Arcane damage endpoint. The
mark's owning Mage converts 35% of each effective single-target Arcane hit and
15% of each effective area Arcane hit into a non-critical, non-recursive heal
on the marked living ally. WOC currently represents this boundary through
Arcane Explosion, Arcane-school ranged/wand projectile landing, and the
school-tagged direct spell projectile dispatcher (including its current
Moonfire Arcane row); later Arcane Surge, Aether Darts and Temporal Cascade
remain separate ability milestones.

The conversion base is the target's actual HP loss after damage shields and
the health floor, not the rolled or pre-absorb amount. It must be rounded only
by the existing resolved-heal path so Mortal Wound, heal absorbs, health caps,
and healing threat retain their WOS193 semantics. It consumes no RNG, rolls no
heal crit, does not invoke an on-heal weapon proc, and cannot feed back into a
damage conversion.

## Source Contract

- `src/sim/combat/damage.ts` invokes
  `chronomancyConvertArcaneDamage(ctx, source, preHp - target.hp, school, aoe)`
  immediately after damage has reduced the target's health. Fully absorbed,
  avoided, and overkill damage therefore contributes zero conversion.
- `src/sim/combat/chronomancy.ts` filters to a living player source, Arcane
  school, and positive dealt damage. It scans marked living allies in stable
  entity order, keeps marks source-scoped, and heals each matching ally without
  drawing random values.
- Individual marks carry `ECHO_CONVERT_SINGLE = 0.35`; area impacts derive
  `ECHO_CONVERT_AOE = 0.15`. The checked generated
  `temporal_echo_contract.zr` is the WOS source of those constants.
- WOS193 only models individual marks. Cascade group mark rates and its
  multi-target cast are expressly out of scope; the conversion helper must not
  reserve state for them or infer a group mark from an individual row.

## Delivery Order

1. Add a red static contract and a focused `zr_vm:project` fixture covering
   source/target ownership, 35% single and 15% area rates, rounding,
   post-absorb/post-overkill base damage, effective-heal threat, snapshot
   preservation, non-Arcane and non-player rejection, and RNG neutrality.
2. Add one narrow post-damage helper that applies shield absorption, clamps HP
   loss, and returns actual dealt damage. Route only the represented Arcane
   endpoints through it; preserve existing non-Arcane behavior and combat/kill
   ownership.
3. Add a source-scoped Temporal Echo conversion reducer using the existing
   WOS193 mark rows and `applyOfflineResolvedDirectHeal`. Scan rows in durable
   order, skip dead or absent parties, and do not mutate the mark, schedules,
   or RNG state.
4. Invoke the reducer after effective damage is committed in Arcane Explosion
   (area), Arcane-school projectile landings (single target), and Arcane wand
   landings (single target). Do not invoke it from a heal, DoT, fixed damage
   copy, kill settlement, or from damage before the final HP delta is known.
5. Run WOS193/194 compatibility guards and an independent second review. The
   dynamic fixture runs only via `zr_vm:project`; lack of that plugin delays
   accepted closeout but never blocks forward implementation.

## Exclusions

- Temporal Cascade group marks, Aether Surge, Aether Darts, Rewind and every
  currently unrepresented Arcane source ability need their own source and
  state milestones.
- PvP/arena conversion is not reachable in WOC's offline MVP and follows the
  source's later PvP-tuning boundary.
- No engine or ZrVM feature is required: bounded world state, authoritative
  fixed-point serialization, source-scoped rows, random-stream ownership and
  resolved-heal application are already present.

## Dynamic Validation

`examples/woc/scripts/woc_game/woc_m4_temporal_echo_conversion_runtime_tests.zrp`
must run only through `zr_vm:project`. No alternate runtime is permitted.

## Second Review

2026-08-03: second static review confirmed source-scoped marks, post-absorb
effective-damage conversion, separate single/area rates, resolved no-crit
healing, and no RNG or recursive conversion path. The focused fixture declares
`zr_vm:project`, and `node tools/wos195_temporal_echo_conversion_static_guard.mjs`
passed from `examples/woc`. Dynamic ZrVM execution remains pending.
