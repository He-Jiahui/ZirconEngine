# WOS209: AoE Talent Scaling Runtime

Status: planned

## Goal

Bring the retained WOC `aoeDamage` subset into agreement with
`dev/world-of-claudecraft/src/sim/content/classes.ts`. The work must use
`zr_vm:project` fixtures and must not add an engine ABI solely for this game
projection.

## Reference Contract

`applyTalentModifiers` transforms each `aoeDamage` and `aoeHeal` effect as:

```text
min = Math.round(min * (aoeHeal ? healMult : dmgMult) + flat)
max = Math.round(max * (aoeHeal ? healMult : dmgMult) + flat)
```

`dmgMult` includes the physical-or-spell global and per-ability `dmgPct`.
`healMult` includes the global healing modifier and per-ability `healPct`.
`flat` is the per-ability `flatDmg` or `flatHeal` value selected by the source
effect path. Scaling occurs before target enumeration and random interval
selection. Existing spell-power, channel-tick and threat handling remain WOS
owners.

## Current WOC Surface

The current M4 projection has exactly five `aoeDamage` effects:

- Arcane Explosion (instant, arcane)
- Flamestrike (aimed delayed/instant burst, fire)
- Rain of Fire (position channel, fire)
- Swipe (instant, physical)
- Hurricane (position channel, nature)

It has no retained `aoeHeal` effect. The paths are split across immediate
`groundAoE.castInstantAoE`, `resolveOfflineGroundAoEPulse`, and position
channels. The source switch does not transform `groundAoE`, so Consecration and
serialized ground-effect rows remain outside this plan.

`effect_dispatch.ts` receives the resolved ability for an immediate cast.
`casting_lifecycle.ts` calls `resolvedAbility` again before every channel tick.
Accordingly, immediate endpoints resolve at the existing burst/command point;
Rain of Fire and Hurricane resolve from the current allocation per tick. No
selection snapshot and no WOS field is source-correct for this surface.

## Design

1. Add a pure `aoe_talent_scaling_state.zr` module with exact independent
   damage/heal endpoint resolution, ordering tests and no state ownership.
2. Add a WorldState endpoint helper that resolves an ability/rank/effect from
   the validated current talent selection. Invoke it at each existing immediate
   resolution and at every Rain of Fire/Hurricane channel tick.
3. Do not add a WOS fact: there is no persisted AoE endpoint in scope, and a
   channel snapshot would contradict the source's per-tick resolution.
4. Route each of the five retained `aoeDamage` callers through the
   helper before `groundAoE` target selection. Do not alter the established
   spell-power coefficient, authoritative RNG count/order, hit resolution,
   threat or lifecycle code.
5. Add a `zr_vm:project` runtime fixture and Node static guard for source
   formula parity, endpoint rounding, immediate resolution, and subsequent
   channel ticks after a respec. Run static checks only until dynamic ZrVM
   execution is explicitly authorized.

## Exclusions

- `aoeHeal` (no current M4 entry), `groundAoE`, `aoeRoot`, `drainTick`,
  `finisherDamage`, `weaponStrike` and
  unrelated non-area effects are separate contracts.
- No generated schema expansion or engine-side builtin is planned.
- This plan does not modify the coordinator, validation queue or existing
  accepted snapshots.

## Acceptance

- Each retained AoE endpoint follows the source rounding order.
- A selection change before an immediate resolution affects that burst; a
  selection change mid-channel affects subsequent ticks only.
- RNG and target ordering stay byte-for-byte within their current owners.
- No WOS schema or historical payload changes are introduced.
