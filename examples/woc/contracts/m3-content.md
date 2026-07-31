# WOC M3 authoritative content slice

`scripts/woc_game/src/content/m3_mobs.zr` is the single WOC-owned source for
the mob definitions first exercised by the four M3 foundation scenarios. It
pins target values from `src/sim/content/zone1.ts`, `zone2.ts` and
`dungeons.ts` at source commit
`7c10f280eec380e9877e66ce16333089e171fe42`.

The initial rows are `forest_wolf`, `wild_boar`, `bog_bloat`, `mogger`,
`mogger_lackey`, `korgath_the_bound` and `sister_nhalia`. The derived accessor
uses the target `createMob` formulas: health is rounded after elite scaling,
weapon bounds are rounded after elite damage scaling, and armor starts at zero
at level one.

`world/terrain_content.zr` and `contracts/m3_terrain_content.json` also carry
the pinned `BUILTIN_WORLD.camps` sequence: every row exposes its source
`mob_id`, `mob_is_dummy`, inclusive `mob_min_level`/`mob_max_level`, spawn
`count`, centre and radius. The source has 307 camp entities, of which the one
`training_dummy` is fixed and leaves 306 non-dummy entities that consume five
construction draws each. This is source data for a future world-construction
reducer; it does not itself consume those draws or materialize a live mob roster.
The generated `contractTest()` checks all 67 rows, so the ZrVM data module
also rejects a reordered or altered camp projection.

`world/camp_spawn_layout.zr` replays the source's five non-dummy construction
draws in camp/spawn order and exposes raw scatter, inclusive level, facing and
wander values. Its coordinates intentionally precede dungeon-door projection,
safe-position correction and ground-height resolution; those geometry steps
remain required before an authoritative entity is materialized.

`world/dungeon_door_content.zr` carries the source-order, deduplicated five
overworld dungeon doors and the imported 20-yard aggro-clear radius.
`world/dungeon_door_clearance.zr` applies the target's strict inside-ring test
and deterministic centre-point `+x` fallback before safe-position resolution.

`generated/m3_camp_mob_core.zr` contains all 47 camp-template core rows in
first-camp order. Its stable scalar API supplies identity, presentation text,
level range, base HP/damage, attack/armor/movement/aggro values, scale, color,
elite/boss/rare/dummy/swim flags and explicit respawn-field presence. Loot and
combat-effect payloads remain separate M4/M5 source contracts.

The public ZrVM boundary is deliberately scalar while Plugins 08 repairs the
custom-class and repeated shared-dependency ABI failures. `metric` ids are:

| Id | Value |
| --- | --- |
| 1 | move speed |
| 2-4 | pack frenzy radius, haste multiplier, duration |
| 5-7 | pulse minimum, maximum, radius |
| 8-10 | stomp minimum, maximum, duration |
| 11-13 | terrify radius, interval, duration |
| 14-17 | death-throes minimum, maximum, radius, delay |
| 20-25 | hp base/per-level, damage base/per-level, attack speed, armor per-level |
| 26 | elite flag as `0.0` or `1.0` |
| 27-29 | minimum level, maximum level, aggro radius |

`derived` kinds are `1=maxHp`, `2=weapon minimum`, `3=weapon maximum` and
`4=armor`. `content/rules.zr` separately owns stable simulation thresholds:
50 ms delta, melee/ranged threat multipliers, minimum melee reach, flee HP,
dungeon boundary and idle/respawn wander ranges.

`world/m3_foundation_test_main.zr` imports both content modules exactly once
and then exercises the four foundation system families. System modules remain
pure until the authoritative WOS2 world orchestrator supplies these scalar
values; no system may load target files or a second gameplay implementation.
