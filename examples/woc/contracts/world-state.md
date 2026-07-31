# WOC authoritative world state (`WOS71`)

`WOS71` is the current committed ZrVM-owned world-state envelope carried
between fixed-tick transactions. `world/state.zr` is the canonical codec and
`main.zr` publishes `world_state: WOS71`; it writes schema 71 and accepts the
WOS2-WOS70 migration range. Rust treats these bytes as opaque gameplay state:
it checks the FNV-1a digest, enforces transaction budgets and commits or rolls
back the whole candidate. It must not decode the envelope to reproduce gameplay
rules.

## Current WOS71 delta

The WOS18 byte table below is a compatibility baseline, not the current write
layout. WOS19-WOS47 append source-owned state in the codec's canonical order;
WOS48 and WOS49 then refine persisted semantic boundaries without inserting bytes,
WOS50 appends the nullable idle target, WOS51 appends the retained personal
quest-loot slot, WOS52 appends source-visible shared corpse loot, WOS53 appends
the source-ordered in-flight Hunter Auto Shot queue, WOS54 adds its captured
damage-school code so caster wand bolts restore with their source closure, and
WOS55 adds one queued-on-swing row for every entity. WOS56 and WOS57 append
spell-projectile and periodic-effect closures after that entity data; schema
58 then appends the source aura details needed by Fear-family control, and
schema 59 appends Sunder Armor's entity-aligned target-aura state, schema 60
appends the bounded Rejuvenation-friendly HoT queue, schema 61 appends the
bounded Consecration ground-effect queue, schema 62 appends one
application-time imbue row for every entity, and schema 63 appends one
rank-plus-selected-power source fact for every WOS57 periodic row. Historical
Fireball and Moonfire rows use rank/power `0/0`; Serpent Sting and Shadow Word:
Pain carry their generated rank and impact-time ranged or spell-power snapshot.
Schema 64 appends the analogous rank-plus-selected-spell-power source facts for
the schema-60 pure-friendly HoT rows. WOS60-WOS63 Rejuvenation rows backfill
`0/0` and retain their historical resolved tick value; new Rejuvenation and
Renew rows reconstruct the generated pure-HoT profile from those stored facts.
Schema 65 then appends an insertion-ordered Power Word Shield absorb queue with
target id, source id, ability code, generated rank, remaining absorb amount and
remaining seconds. WOS66 then appends one `fixed6` remaining duration and one
`fixed6` generated AP-drain value for every entity. WOS2-WOS65 decode those
Demoralizing Roar rows as zero. WOS67 then appends one `fixed6` Prowl remaining
duration and one `fixed6` stealth movement value for every entity. Active rows
are restricted to a living Cat-form player with the generated source `0.5`
value and at most 3600 seconds remaining. WOS2-WOS66 decode those Prowl rows as
zero; the retained idle-aggro input reads the active row before a mob enters
combat, and the movement fold applies it as the source minimum-slow term before
form and speed-buff multiplication.

WOS68 appends one school-code byte for every flattened source motion-aura row.
Only `lockout` rows may carry a nonphysical school; all other rows persist the
physical/zero discriminator. This preserves Skull Bash's four-second
school-specific interrupt lockout across snapshot restore while WOS2-WOS67
decode with zero school codes.

WOS69 appends two independent per-entity consumable slots. The eating row stores
the M5 food item code, resolved HP gain per two seconds and remaining duration;
the drinking row stores the M5 drink item code, resolved Mana gain per two
seconds and remaining duration. A retained food and drink may run concurrently
for 18 seconds while the player is sitting. The fixed-tick reducer applies each
active gain every 40 ticks, and standing, positive damage or death clears both
slots. WOS2-WOS68 decode with empty consumable rows.

The same WOS69 tail then stores the party-scoped raid-marker map as bounded
`party id / entity id / symbol` rows. Symbols are `0..7`; both the party/entity
pair and party/symbol pair are unique. Any party member may set or clear a mark
on a living, hostile, unowned mob. Reapplying the same symbol to the same mob
toggles it off, while assigning that symbol elsewhere moves it. Entity death and
party dissolution remove the corresponding rows; WOS2-WOS68 decode with no
markers.

WOS69 also appends the party's Master Loot settings to every member row:
enabled, configured looter id (`0` means the current leader), and the
`uncommon`/`rare`/`epic` threshold code. Only the leader may mutate the shared
values. New members inherit them, a departed configured looter falls back to
the current leader without rewriting the stored preference, and party
dissolution clears the rows. WOS2-WOS68 use the source default of disabled,
leader-selected, `uncommon`.

WOS70 appends the source-authoritative pending Need/Greed and Master Loot
records after the WOS69 tail. Each roll stores its monotonic id, corpse entity,
M5 item and quality codes, absolute expiry and nullable master-looter id. A
bounded candidate table snapshots member ids in party order and persists
Pending, Need, Greed or Pass plus the hidden 1-100 result. The nine-byte
`lootRoll` command maps wire choices `0/1/2` to Need/Greed/Pass, consumes no RNG
for Pass, resolves Need before Greed and consumes an extra draw only to break an
actual highest-roll tie. Normal rolls expire after 60 seconds; unanswered
candidates count as Pass and an all-Pass item returns to an open corpse slot.
A Master Loot record gets 300 seconds to be curated. The bounded `masterAssign`
command keeps the window open when no valid target is selected, grants directly
when one target remains, and converts multiple targets into a fresh 60-second
Need/Greed window for that subset. An unassigned record converts in place to the
same fresh window for all original candidates when the 300-second timer expires.
WOS2-WOS69 decode with no pending rolls and `nextLootRollId = 1`.

WOS71 appends two bounded party-loot tables. The first stores each live party's
monotonic poor/common round-robin cursor. The second stores ordered
`corpse mob / recipient player / member ordinal` rows captured at the mob's
death-time reward boundary. Later movement cannot change that candidate set;
respawn removes it. Shared copper uses the source fair-split rule: equal base
shares plus a without-replacement RNG draw for each remainder copper. Poor and
common items advance the party cursor and force-grant to the selected candidate;
premium items, Need/Greed and Master Loot consume the same death-time snapshot.
WOS70 decodes with empty tables, preserving its prior loot-time range fallback.

WOS121 retains Rake without adding wire bytes. Its Cat-form stealth opener uses
the existing WOS57 periodic-damage queue: the persisted row holds the target,
source, Rake identity, resolved `10 / 16` three-second bleed tick, nine-second
duration and three-second timer. The direct weapon strike and one-point combo
award remain immediate transaction state; the DoT row is applied independently
after the strike when the target survives, matching the source effect order.
WOS122 retains Claw without wire changes: its Cat-form 45-Energy weapon strike
and landed one-point combo award are immediate transaction state and reuse no
periodic, aura or schema projection.
WOS123 retains Ferocious Bite without wire changes: its Cat-form 35-Energy
physical finisher snapshots the existing combo value, clears Prowl, and consumes
the combo after source effect resolution while reusing existing immediate combat
state, RNG and threat projection.

The WOS62 Emberkin pet-command closure reuses those existing entity rows and
therefore adds no wire bytes. `pet_attack` assigns the owner's valid hostile
target to the live Emberkin and writes the source one-point opening threat;
`pet_mode` persists passive/defensive/aggressive and clears active combat state
on passive; `pet_revive` restores the retained dead Emberkin beside its owner at
35% HP. The Imp projection has neither Growl nor Water Jet, so its manual and
autocast commands are source-style no-ops that clear both autocast flags. Pet
rename, abandonment, feeding and Demon Heal remain outside this closure because
they require, respectively, string state, generic entity-row deletion, retained
food aura/inventory mapping, and a channel-cast projection.

WOS77 extends that same schema-62 entity/queue projection with the Emberkin
Firebolt combat closure. An already-commanded, living Emberkin holding a valid
Eastbrook target within the source 25-yard reach launches a WOS53 Firebolt
projectile at its two-second base cadence. The homing projectile reads live pet
weapon/AP columns on impact, spends the source's RNG in strict 5% crit then
weapon-range order, bypasses armor, enters combat at impact, and contributes
damage threat under the pet identity while kill XP/quest/loot credit routes to
its owner. It drops a target once the pet is more than the source 40-yard owner
leash away and fizzles without RNG when either endpoint dies. Pet pathing,
nearby-mob pulls, owner-assist/aggressive target selection, pet auras
(pet_damage_pct and pet_spellhaste), generic pet deletion, and the full
source damage/aura pipeline remain outside this retained closure.

WOS78 attaches the existing source-locked pet target predicates to the retained
Emberkin rows without changing schema 62. Passive pets acquire nothing.
Defensive pets choose the nearest strict-distance Eastbrook candidate that is
attacking the pet/owner or that the owner is already attacking or threatening;
the selected target then flows into WOS77 Firebolt. The source aggressive mode
also needs PlayerMeta.lastActiveTick to enforce its anti-AFK rule. That durable
activity timestamp is not in the current WOS envelope, so fresh aggressive
pulls intentionally remain disabled while an aggressive pet still assists an
already-engaged owner or pet. The source spatial grid is represented here by a
complete scan of the bounded retained Eastbrook rows; multiplayer/PvP entities,
generic hostile templates, and the activity-clock column remain future work.

WOS79 completes the no-target Emberkin heel branch only for open ground, again
without changing schema 62. When a live Emberkin has no valid target it clears
combat and, beyond the source 3.5-yard follow distance, advances toward its
owner at the source `max(pet move speed, 7.7)` cadence for one fixed tick. The
reducer records previous/current transform, source heading, a distance-capped
step, and the existing deterministic terrain height sample. The source cached
A* route, path-recalculation cooldown, obstacle/steep-slope movement resolver,
line-of-sight query, waypoint removal, move-speed auras, spatial rebucket, and
the 60/96-yard fresh-path forced-recovery teleport cannot be represented by the
current WOS rows. They remain intentionally absent rather than simulated with
an invented fallback; `entityPetPathCooldowns` stays serialized but is not
claimed as an executable path cache.

WOS80 retains the source `updatePet` scalar maintenance that precedes target
selection, without changing schema 62. A live Emberkin's retained pet timer
decreases by the fixed 50 ms step and clamps at zero. On every authoritative
tick divisible by 40, an Emberkin that was already out of combat restores
`max(1, round(max HP * 2%))`, capped at max HP. Water Jet channel ownership,
pet aspect synchronization, pet aura modifiers, generic control-state queries,
and player-facing pet events remain outside this scalar closure.

WOS81 retains Emberkin's source ranged-pet out-of-range arm. When its valid
Firebolt target exceeds 25 yards, the pet first advances one fixed step at its
source move speed toward that target, then applies the existing no-fire cooldown
decay. The direct open-ground leg uses the same transform/heading/terrain write
as WOS79 and adds no wire bytes. Source root checks, move-speed modifiers,
collision/steep-slope resolution, obstacle slide steering, water handling and
the source spatial rebucket remain unavailable, so they are not represented as
equivalent behavior.

WOS82 connects the existing player-death reducer to the source owned-demon
handoff. When the Emberkin owner dies, the live Emberkin is retired in the same
authoritative transaction: HP is zeroed and target, combat and threat state are
cleared. The target source gives a summoned demon a short corpse interval and
then removes it from its entity Map. Until Plugins08 supplies generic
transaction-safe physical entity-row deletion, WOS retains the existing inert,
dead owner-bound row as that removal projection; no normal pet corpse, loot or
respawn behavior is claimed.

WOS83 makes that demon-death projection time-bounded. `handleDeath` leaves a
summoned demon revivable for its source three-second corpse period before
`updateMob` removes it. WOS writes that existing `entityCorpseMicros` countdown
on owner-driven demon death, accepts `pet_revive` only while it remains positive,
and retains a zero-count inert row after expiry because physical row removal is
still a Plugins08 backend dependency. The owned-pet lookup now prefers a living
Emberkin over an older inert row, preventing a replaced summon from being
shadowed by stale parallel-array data. This does not add hunter corpse behavior,
pet heal/revive cast resource rules, or generic entity deletion.

WOS84 closes two source instant pure-DoT spells: Hunter Serpent Sting and
Priest Shadow Word: Pain. Admission validates the generated current-known rank,
resource, source 1.5-second GCD clamped to 0.7 seconds, hostile Eastbrook
target, range/minimum range, and facing; the retained outdoor slice has clear
line of sight only. A non-physical zero-cast spell still queues a homing
projectile. Its landing consumes exactly one spell-resist draw and, on success,
snapshots the source selected scaling power into a same-target/same-source/
same-ability periodic row without direct impact damage. The bounded queue
replaces that exact source identity, ticks deterministically, and persists its
rank/power facts through WOS63. Generic LOS/collision, set bonuses, proc/aura
observers, player/PvP targets, and a full multi-entity spatial query remain
outside this Eastbrook closure.

WOS85 closes Priest Renew in the same retained friendly player/owned-pet target
scope as Rejuvenation. It is an instant, 30-yard Holy pure HoT: a valid target
directly spends its rank cost, applies the hasted 1.5-second GCD with a
0.75-second floor, and snapshots the source spell power without projectile,
resist or RNG work. The bounded row replaces only the same target/source/ability
identity, so separate casters keep separate Renew auras. Schema 64 persists its
generated rank and application-time spell power; generic friendly NPC targets,
multiplayer ownership, heal crit/absorb and aura-observer behavior remain outside
this Eastbrook closure.

WOS86 closes Priest Power Word Shield for the same retained friendly target
projection. A valid Priest directly spends its generated rank cost, applies the
hasted GCD and source six-second cooldown, then inserts a 30-second absorb row
with the generated `48/90/145` rank amount. Applying the same source/target/
ability removes that row before appending its refresh, preserving the source's
newest-aura order without erasing an independently cast shield. During the
player-aura phase the rows age before Eastbrook melee; that melee bridge
consumes matching rows newest-first, removes fully spent rows, and applies only
the residual damage to incapacity breaks, cast pushback, HP and death. Generic
damage sources, aura proc listeners, pet damage sharing, friendly NPC targets
and universal aura lifecycle behavior remain outside this closure.

WOS87 closes Priest Smite without changing the WOS65 layout. It reuses the
existing in-flight projectile row to retain the successful 2.0/2.5-second cast
release, source rank and generated Holy damage range. On impact the retained
Eastbrook path spends one spell-resist draw, then the range and spell-critical
draws, applies Spell Power and the 1.5x spell-critical multiplier, and enters
combat or resolves target death through the existing projectile path. Generic
spell multipliers, set/proc observers and non-Eastbrook target projections are
not represented by this closure.

WOS88 closes Priest Mind Blast without changing the WOS65 layout. Its three
generated Shadow `directDamage` ranks cast for 1.5 seconds, defer their resource
spend and eight-second cooldown until successful completion, then reuse the
existing projectile row. Impact consumes one spell-resist draw before its range
and spell-critical draws; only retained Eastbrook hostile targets receive the
damage, combat, threat and lethal-settlement bridge. Generic spell multipliers,
set/proc observers and non-Eastbrook target projections are not represented by
this closure.

WOS89 closes Priest Heal without changing the WOS65 layout. Its generated
friendly Holy `heal` ranks use 2.5-second casts and defer the rank resource cost
until successful completion. The retained friendly player/owned-pet bridge then
consumes the source direct-heal range/scaling draw followed by its healing-critical
draw, updates HP and propagates only the existing Eastbrook healing-threat rows.
There is no projectile or spell-resist work. Generic friendly NPC targets,
multiplayer ownership, heal observers and set/proc behavior remain outside this
closure.

WOS90 closes Priest Flash Heal without changing the WOS65 layout. Its generated
friendly Holy `heal` has no cooldown and uses a 1.5-second cast, then follows
the same retained direct-heal range/scaling draw, healing-critical draw, HP and
Eastbrook healing-threat bridge as Heal. It deliberately retains only
player/owned-pet friendly targets; generic friendly NPCs, multiplayer ownership,
heal observers and set/proc behavior remain outside this closure.

WOS91 closes Priest Mind Flay without changing the WOS65 layout. Its generated
Shadow channel costs resources at start, then releases three source-timed
projectiles. Each landing retains exactly one range draw and the channel Spell
Power rider, with `healFrac = 0` preventing both caster healing and healing
threat. Generic channel interruption, multiplayer/PvP target projection, spell
multiplier/proc observers and non-Eastbrook hosts remain outside this closure.

WOS92 closes Shaman Lightning Bolt without changing the WOS65 layout. Its four
generated Nature `directDamage` ranks retain their source cast time and cost at
successful completion, then reuse the in-flight projectile row. Landing spends
one spell-resist draw before direct-damage range and critical draws, then bridges
damage, combat, threat and target death only for retained Eastbrook hostiles.
Generic spell multipliers, set/proc observers, multiplayer/PvP targets and
non-Eastbrook hosts remain outside this closure.

WOS93 closes Shaman Healing Wave without changing the WOS65 layout. Its four
generated Nature friendly `heal` ranks retain their source cast time and spend
cost at successful completion, then use the direct-heal range/scaling draw,
healing-critical draw, HP update and existing Eastbrook healing-threat bridge.
There is no projectile or spell-resist work. Generic friendly NPC targets,
multiplayer ownership, heal observers, set/proc behavior and non-Eastbrook hosts
remain outside this closure.

WOS94 closes Shaman Earth Shock without changing the WOS65 layout. The instant
Nature spell charges cost, starts the shared Earth/Flame/Frost Shock cooldown and
creates its projectile at admission. Landing uses the retained spell-resist,
range/critical damage, combat, threat and lethal paths for Eastbrook hostiles.

WOS95 closes Shaman Frost Shock without changing the WOS65 layout. The instant
Frost projectile uses the same shared Shock cooldown, then applies direct damage
and the generated eight-second slow aura after a successful impact.

WOS96 closes Shaman Flame Shock without changing the WOS65 layout. The instant
Fire projectile shares the Shock cooldown, applies direct damage after spell
resist, and only then appends the generated four-tick 12-second DoT when the
target survives. Its resolved DoT row uses the existing WOS63 fields and
replaces a prior Flame Shock row for that target; Earth, Frost and Flame Shock
projectile profiles are all validated before an in-flight state can encode.

WOS97 closes Shaman Flametongue Weapon without changing the WOS65 layout. Its
instant no-target cast charges cost and starts the generated GCD, then replaces
the existing mutually exclusive imbue row with its source rank and 300-second
remaining duration. At that slice the retained physical auto-attack preparation
recognized Seal of Righteousness and Flametongue Weapon as the legal row
identities and contributed the selected source bonus to the physical swing base.
It did not claim generic aura lifecycle, inventory enchant state or triggered
melee proc support.

WOS98 closes Shaman Frostbrand Weapon (source display: Rimebound Weapon)
without changing the WOS65 layout. It shares the instant no-target, cost/GCD
and single-row replacement behavior, but its second source rank is level 20.
The retained physical auto-attack preparation accepts Frostbrand as the third
legal imbue identity and adds its source bonus before physical critical and
armor resolution. It does not claim generic aura lifecycle, inventory enchant
state or triggered melee proc support.

WOS99 closes Shaman Ghost Wolf (source display: Shadewolf) without changing the
WOS65 layout. Motion kind code 8 is a restricted `buff_speed` row: only the
same-entity `ghost_wolf` source may use it, with value 1.4, no break chance and
at most 3600 seconds remaining. Its timed cast bills on successful completion
and its subsequent successful cast removes that row. Player movement uses the
source product of the smallest retained slow and the largest retained speed or
travel-form multiplier. This is not a generic aura or speed-buff runtime.

WOS100 closes Shaman Stormstrike (source display: Ancestral Strike) without
changing the WOS65 layout. A valid retained hostile Eastbrook melee target
spends 40 resource, receives the source haste-adjusted shared GCD and writes a
12-second expiry to the existing sparse ability-cooldown partition before the
shared deterministic weapon-strike reducer consumes its bonus 26. The reducer
retains weapon damage, physical hit/critical/armor order, combat entry, threat,
lethal settlement and authoritative RNG state; it does not introduce a generic
melee aura, proc or talent-modifier runtime.

WOS101 closes Warlock Shadow Bolt (source display: Gloom Bolt) without changing
the WOS65 layout. Its four generated direct-damage rank profiles persist in the
existing in-flight projectile row, including the source cast time, and a valid
hard cast delays its resource bill until completion. Landing reuses one spell
resist draw followed by direct Shadow damage, threat, combat and lethal
settlement. It does not introduce generic spell multipliers, talent/proc
observers, non-Eastbrook target projection or another spell runtime.

WOS102 closes Warlock Immolate (source display: Burning Pact) without changing
the WOS65 layout. Its three generated direct-plus-DoT ranks persist direct
minimum/maximum and cast time in the existing in-flight projectile row. A
surviving direct impact creates one 15-second, three-second-cadence five-tick
row with its resolved source damage and no second spell-power rider. It does
not introduce generic spell multipliers, talent/proc observers, non-Eastbrook
target projection or another spell runtime.

WOS103 closes Warlock Corruption (source display: Blackrot) without changing
the WOS65 layout. Its three generated pure-DoT ranks persist zero direct
minimum/maximum, the source two-second cast time and the selected rank in the
existing in-flight projectile row. A successful Shadow impact spends one
spell-resist draw, snapshots source spell power through the existing pure-DoT
resolver, and stores one 18-second, three-second-cadence six-tick row using the
existing rank/power persistence fields. It does not introduce generic spell
multipliers, talent/proc observers, non-Eastbrook target projection or another
spell runtime.

WOS104 closes Warlock Life Tap (source display: Hard Bargain) without changing
the WOS65 layout. It projects `hp` and `mana` metrics in the M4 effect catalog
and resolves them directly against existing player HP and resource columns. A
valid no-target cast retains the source hasted GCD before rejecting a health
cost greater than or equal to current HP; otherwise it subtracts the generated
health cost and adds generated mana capped by the existing maximum-resource
column. It does not introduce a new state partition, RNG draw, generic damage
event, error DTO, talent multiplier or another spell runtime.

WOS105 closes Warlock Curse of Agony (source display: Hex of Anguish) without
changing the WOS65 layout. Its three generated pure-DoT ranks share the existing
zero-direct projectile profile and rank/power periodic persistence fields. A
successful Shadow impact spends one spell-resist draw, snapshots spell power,
and creates one 24-second, three-second-cadence eight-tick row. The shared
periodic threat reducer obtains every pure-DoT metric through
`pureDotAbilityIndex`, so this row and Corruption use their generated threat
facts rather than an ability-specific fallback. It does not introduce generic
spell multipliers, talent/proc observers, non-Eastbrook target projection or
another spell runtime.

WOS106 closes Warlock Searing Pain (source display: Sear) without changing the
WOS65 layout. Its generated single Fire direct-damage rank keeps the source
`30-38` range and 1.5-second cast time in the existing in-flight projectile
row; a successful completion delays the 35-resource bill until the retained
projectile is queued. Landing consumes one spell-resist draw before direct
damage, threat, combat and lethal settlement. It does not introduce generic
spell multipliers, talent/proc observers, non-Eastbrook target projection or
another spell runtime.

WOS107 closes Warlock Shadowburn (source display: Duskfire) without changing
the WOS65 layout. Its generated single Shadow direct-damage rank stores the
source `56-66` profile and zero cast time in the existing in-flight projectile
row. A valid instant cast first arms the hasted GCD, then bills 70 resource,
starts the existing sparse 15-second ability cooldown and queues the projectile;
landing consumes one spell-resist draw before direct damage, threat, combat and
lethal settlement. It does not introduce generic spell multipliers, talent/proc
observers, non-Eastbrook target projection or another spell runtime.

WOS108 closes Warlock Demon Skin (source display: Fiendhide) without changing
the WOS65 layout. Its three generated `buff_armor` ranks use a constrained
persisted aura kind with source ability, self source, value and remaining time.
A valid instant cast retains the hasted GCD, spends the generated resource cost
and refreshes the one ability-identity row. Its active 30/55/80 snapshot is
included in physical mitigation before the existing Sunder reduction. It does
not introduce generic aura lifecycle, talent/proc observers, non-Eastbrook
target projection or another spell runtime.

WOS109 closes Warlock Rain of Fire without changing the WOS65 layout. Its
generated position-channel profile uses the existing persisted `castAim` columns:
the `castAt` reducer reads finite little-endian `f64` x/z values, clamps the
center to the generated 30-yard range, and locks it for the four-tick channel.
Each one-second pulse gets the generated channel spell-power coefficient and
uses the retained deterministic radius-seven Fire AoE reducer. There is no
on-cast pulse; the aim is cleared on completion or cancellation. It does not
introduce generic spell multipliers, talent/proc observers, collision/LOS parity,
non-Eastbrook target projection or another spell runtime.

WOS110 closes Warlock Conflagrate without changing the WOS65 layout. Its
generated `consumeAura` profile persists the `54-64` Fire projectile facts in
the existing in-flight row, including its zero cast time. A valid instant cast
retains the hasted GCD, spends 55 resource and starts the six-second cooldown
before launch. A non-resisted impact removes only an active same-source
`immolate` hostile DoT row before the direct range/critical resolution; a
missing or foreign row ends after resistance without damage. This does not
introduce generic aura lifecycle, spell multipliers, talent/proc observers,
non-Eastbrook target projection or another spell runtime.

WOS111 closes Warlock Siphon Life without changing the WOS65 layout. Its
generated pure-DoT profile persists the ordinary ability code, rank, snapshot
power and `60 / 30 / 3` periodic facts already stored in the existing DoT row.
The generated `leechPct: 1` is verified when an in-flight or restored Siphon
row is admitted, then derives a self-heal from each tick's resolved damage. The
tick applies target damage and lethal settlement before healing a living caster
and distributing effective-healing threat. This does not introduce generic aura
lifecycle, spell multipliers, talent/proc observers, non-Eastbrook target
projection or another spell runtime.

WOS112 closes Druid Swiftmend without changing the WOS65 layout. Its generated
`consumeAura` effect selects `auraKind: "hot"` and carries the nested
`105-125` direct-heal range. The world scans the existing ordered WOS64 HoT
queue for the first target-local row, removes it before the direct range and
healing-critical draws, then reuses the established effective-healing-threat
kernel. The row may originate from any caster, matching source helpful-aura
selection; no matching row still spends cost and arms cooldown without RNG.
This does not introduce generic aura lifecycle, spell multipliers, talent/proc
observers, friendly NPC target projection or another spell runtime.

WOS113 closes Druid Wrath (source display: Wildbolt) without changing the WOS65
layout. Its generated direct-damage profile supplies the four source ranks,
including the rank-2 inherited two-second cast time for ranks 3-4. The existing
projectile row persists the resolved rank, `60-71` rank-4 bounds, Nature school
and cast time after successful-cast billing; decode validates those facts before
the landing path consumes one spell-resist draw and the direct range/critical
draws. This does not introduce generic spell multipliers, talent/proc observers,
non-Eastbrook target projection, collision/LOS parity or another spell runtime.

WOS114 closes Druid Healing Touch (source display: Wildmend) without changing
the WOS65 layout. Its generated direct-heal profile supplies four source ranks,
including the rank-2 inherited three-second cast time for ranks 3-4. The
existing cast state retains the selected friendly target and commits cost only
when the cast completes; the direct-heal kernel then consumes range and critical
draws before updating health and effective-healing threat. This does not
introduce generic aura lifecycle, spell multipliers, talent/proc observers,
friendly NPC target projection or another spell runtime.

WOS115 closes Druid Starfire (source display: Skyfall) without changing the
WOS65 layout. Its one-rank direct-damage profile persists the `80-112` bounds,
Arcane school and three-second cast time in the existing projectile row after
successful-cast billing. Decode verifies those facts before the landing path
consumes one spell-resist draw followed by direct range/critical draws. This
does not introduce generic spell multipliers, talent/proc observers,
non-Eastbrook target projection, collision/LOS parity or another spell runtime.

WOS116 closes Druid Entangling Roots (source display: Gripping Roots) without
changing the WOS65 layout. Its two-rank Nature hard cast persists zero projectile
damage bounds, rank and 1.5-second cast time in the existing projectile row.
After one spell-resist draw, impact writes the existing source-ability motion
aura row with a 12-second root before adding rank two's spell-power-snapshotted
`32 / 12 / 3` periodic row. Eastbrook pursuit writes that root into its existing
rooted state, suppressing translation while preserving facing and in-range melee
logic. This does not introduce generic crowd-control DR,
generic aura lifecycle, spell multipliers, talent/proc observers, non-Eastbrook
target projection, collision/LOS parity or another spell runtime.

WOS117 closes Druid Maul (source display: Bonecrush) without changing the WOS65
layout. It records no new state: the existing queued-on-swing row carries Maul's
source identity across encode/decode. Queue admission now passes the active form
to the existing source catalog, which admits Maul only in Bear form and continues
to reject the normal kit while action-locking forms are active. The next retained
mainhand attempt uses the existing generated rank, delayed cost, weapon-bonus and
flat-threat path; rank 2 uses cost 15, bonus 27 and threat 50. This does not
introduce a generic form, melee, threat, aura or cooldown runtime.

WOS118 closes Druid Growl (source display: Menace) without changing the WOS65
layout. It reuses the persisted threat and forced-target columns that already
back Warrior Taunt. Growl's own admission passes the active form to the source
catalog and therefore requires Bear form before the shared taunt settlement
matches top threat, forces the Eastbrook mob for three seconds and writes the
generated 10-second cooldown. This does not introduce a generic form, threat,
crowd-control, cooldown or target runtime.

WOS119 closes Druid Demoralizing Roar (source display: Craven Roar) through the
new WOS66 tail. The targetless Bear-form cast spends 10 Rage, observes the
source haste-adjusted GCD floor, then writes the generated `20 / 35` flat AP
drain and a 20-second duration for every living hostile Eastbrook mob inside
the source eight-yard radius. Each target enters existing combat/aggro state and
receives the source 10 physical-threat opening amount. The retained mob swing
adapter reads clamped effective AP after that debuff; expiry clears both values
before the next hostile lifecycle pass. This does not introduce multiplayer
aura sources, a generic hostile aura partition, talent/proc observers, LOS
geometry or another combat runtime.

- WOS19-WOS21: dungeon-difficulty preference, one party ready-check, and arena
  queue-admission rows.
- WOS22-WOS24: initial combat loadout, idle-wander timer, and combat/aggro
  baselines.
- WOS25-WOS30: mob recovery/flee inputs, taunt-target inputs, player resource
  timers, cast-aim/charge/follow snapshots, resource kind, and presentation
  identity.
- WOS31-WOS35: sitting posture, tap owner, profession-harvest claim owner, and
  finite-or-infinite FFA loot-lock state.
- WOS36-WOS38: pet scalar runtime fields, boss cadence baselines, and remaining
  boss control timers/flags. Mutable paths, full aura lifecycles, and boss
  mechanics remain outside this scalar projection.
- WOS39: ordered per-entity motion-aura partitions. Each row retains the
  source ability code, source entity id, generated control/mobility/speed kind code,
  and positive remaining seconds. The codec derives no persisted `stunned`,
  `rooted`, or `mobile_cast` flags from these rows; the retained player tick
  derives them at use time and expires rows after cast progression.
- WOS40: per-entity active form ability code and saved mana maximum. Together
  with the existing WOS27 resource/saved-mana fields and WOS29 resource kind,
  these preserve Bear/Cat form resource swaps across rollback and reload. Form
  ability codes come from the source-generated contract; full derived-stat
  recomputation and general aura lifecycle remain later work.
- WOS41: per-entity M5 baseline profile code. Zero preserves the pre-WOS41
  final-column behavior. A nonzero code joins the source-pinned class baseline
  and retained entity level to reconstruct the starting-gear pre-form inputs,
  then applies the retained Talent V2 specialization and six row selections.
  Form transition, stripping and retained talent updates therefore recompute
  final stats without applying multipliers to arbitrary preexisting columns.
- WOS42: one per-entity signed fixed6 `spellHaste` final column after WOS41's
  baseline profile. The current source-pinned M5-plus-retained-talent graph
  reconstructs it deterministically; a WOS42 candidate must match that result.
  WOS2-WOS41 snapshots backfill zero for zero profiles or the retained profile
  derivation otherwise.
- WOS43: one byte each for M5 `helmet`, `feet` and `mainhand` item identities
  after `spellHaste`. `0` means an empty armor slot or the class source starting
  mainhand; `1..14` mean the initial pinned M5 content index plus one. The bounded
  identity is class-admitted and rebuilds final stats plus mainhand damage in
  source order. An over-level identity remains stored but contributes no stats
  and uses the source unarmed fallback for mainhand combat values. Inventory
   instances, set counts and ordered live auras remain later state inputs.
- WOS44: per-entity copper (`signed`), four M5 bag `u8` identity codes, then a
  `u16` dense inventory stack count and ordered (`u8` item code, `u16` count)
  rows. Item codes are pinned M5 catalog indexes plus one. The player backpack
  starts at 16 pooled slots; bag identities add their source `bagSlots` value.
  Stackable projected source kinds use 20, while weapon/armor/held-offhand/bag/
  tool rows use exactly one. Nonplayer rows must
  retain zero copper, no bags and no inventory stacks. WOS2-WOS43 migrate to
  those defaults; signer/charges/rolled/enchant/bound item instances remain a
  later non-fungible extension.
- WOS45: no entity bytes are inserted. Instead, the existing mainhand identity
  reserves `255` for an explicitly empty hand; `0` remains the source starting
  mainhand so WOS2-WOS44 snapshots retain their old meaning. The item catalog
  preserves the WOS44 14-item index prefix and appends seven source class-start
  identities at indexes 15 through 21. The bounded scalar transaction can now
  return a source starting weapon on replacement, and return a worn item only
  after an unequip capacity preflight.
- WOS46: command-routing-only milestone. It retains WOS45's row layout while
  binding `equip_bag` and `unequip_bag` to the WOS44 inventory partition.
- WOS47: after the WOS44 inventory stack partition, every entity writes a `u8`
  buyback-entry count (maximum 12) followed by ordered `(item code u8, count
  u16)` rows. Rows are newest-first and merge the same scalar item identity.
  Only players may retain them. WOS2-WOS46 decode to an empty buyback partition.
- WOS48: the pre-existing offline `q_boars` hide field is a derived progress
  projection, not an inventory ledger. While the quest is active or ready it
  must equal the primary player's `min(5, boar_hide)` scalar inventory count;
  inventory removal can therefore revert ready to active. There are no new
  bytes. WOS2-WOS47 active/ready rows materialize at most five hides during
  decode, which preserves the old observable objective state without inventing
  surplus items that the former ledger never represented.
- WOS49: no entity bytes are inserted. Command 24 (`discard`) now removes from
  the same scalar inventory partition with source default-one and requested
  count clamping behavior. The current 35-item scalar catalog contains neither
  `ItemInstanceData`; generated `noDiscard`, `noVendorSell` and `soulbound`
  flags preserve source policy for scalar items, while instance identity remains
  outside the reducer.
- WOS50: after each WOS23 wander timer, an entity writes a target-present byte
  followed by fixed6 X and Z coordinates. A target-absent row must retain zero
  coordinates; WOS2-WOS49 decode to that absent default. The offline Eastbrook
  bootstrap persists the source constructor's post-spawn Mulberry32 cursor
  before the first live idle-mob draw, so construction RNG does not replay in a
  committed tick.
- WOS51: after WOS17's `lootable` byte, every entity writes a scalar personal
  quest-item code and count. Both are zero when no retained personal item exists;
  a nonzero code is a valid M5 item and must have count one. The current MVP
  populates that slot only for an active, still-needed `q_boars` `boar_hide`
  roll on an Eastbrook wild boar. WOS2-WOS50 decode to the empty slot.
- WOS52: after WOS51's personal slot, every entity writes signed corpse copper
  followed by six ordered `(u8 item code, u8 count)` shared slots. Zero code
  requires zero count; a nonzero code is a valid M5 item with count one. The
  retained Eastbrook Wolf/Boar reducer fills those slots in source declaration
  order and preserves the source copper chance plus inclusive `rng.int(ceil(0.6x),
  ceil(1.4x))` draw. WOS2-WOS51 default to zero copper and six empty slots.
- WOS53: after the Card Duel payload, a bounded launch-order trajectory queue
  writes source/target ids, homing X/Z, positive remaining TTL, a wand marker,
  and the captured min/max/speed profile. It persists Hunter Auto Shot's physical
  `5/9/2.3` closure so impact resolves only after the source projectile travel.
- WOS54: each WOS53 queue row inserts a one-byte school code immediately after
  the wand marker: physical, arcane, holy, shadow or nature. WOS53 input rows
  default that code to physical. The current reducer now accepts source wand
  profiles for Mage/Priest/Warlock and Druid in caster or Moonkin form; Bear and
  Cat fall through to the melee path, while Travel and action-locking Fireball
  travel form reject attack arming and cancel auto-attack after their timer decay.
  Wands retain `3/6/1.8`,
  30-yard/no-dead-zone timing and do not apply physical armor mitigation.
  The pre-existing combat-loadout columns now also drive the retained direct
  dual-wield baseline: both timers decay before form gating, each ready white
  swing adds the source `0.1` dual-wield miss penalty, and a valid offhand uses
  its own speed with `0.5` damage multiplier.
- WOS55: after the WOS54 projectile queue, every entity writes a `u16` current
  known ability code, a free-cost marker and fixed6 cost multiplier. `0,false,1`
  is the only empty row; a nonempty row maps to M4 Heroic Strike, Raptor Strike
  or Maul. WOS2-WOS54 decode those defaults. The offline reducer admits a known,
  live hostile target inside five yards and the source melee arc, delegates
  action-locking-form admission to the source catalog, toggles the same queued
  ability off, otherwise arms auto-attack without billing. The next mainhand
  attempt clears all queue fields, bills `ceil(cost * multiplier)` when possible,
  and writes the source-resolved cooldown (including zero) to the sparse
  cooldown table; offhand swings remain white attacks. Player death, ghost
  release and either resurrection path also atomically reset the three queue
  columns, matching the source terminal-state cleanup.

WOS56 is a codec revision: it appends a `u16` ability code, `u8` resolved rank
and fixed6 base cast time to every WOS54 projectile row, before the pre-existing
WOS55 per-entity queued-on-swing rows. WOS2-WOS55 decode those three values as
`0,0,0`. The all-zero closure remains the only white-shot/wand profile. A
Frostbolt row is non-wand Frost school at the source 26-yard-per-second travel
speed, with its generated direct-damage min/max, resolved rank and *unhasted*
source cast time; malformed cross-profile values are rejected before encode or
after restore. Cast completion revalidates only the locked target's source
range/LoS condition, deducts mana and queues that row without consuming RNG or
applying an effect. The launch-order projectile tick follows the live target,
fizzles when either endpoint dies, and at impact consumes spell-resist, damage
range and spell-crit draws in that order. A full resist enters combat but does
not apply Frostbolt's slow or damage threat. A landed hit applies damage,
threat and the source rank's ability-keyed slow row only while the target lives.
The retained subset reads live spell power, spell crit and hit bonus at landing;
it currently has no durable spell-crit-damage bonus, spell damage aura,
rooted-Shatter, set-proc or generalized LoS projection, so those source paths
remain intentionally outside this Eastbrook closure.

WOS57 is a codec revision: after the WOS55 per-entity queued-on-swing rows it
appends a global bounded `u16` count (at most 64) of hostile-spell DoT rows.
Each row stores target and source ids, the ability code, positive per-tick
damage, remaining and original duration, interval and timer. WOS2-WOS56 decode
an empty tail. A landed, unresisted Fireball or Moonfire first resolves its
direct spell hit and threat, then writes one source-profiled row for a living
target; a new instance of the same ability replaces that target's prior row.
The retained mob stage decrements remaining and timer at 20 Hz, applies each due
periodic hit without RNG, advances the timer before removing the row after its
final tick. The bounded projection does not claim a general Aura dispatcher,
stacking families, arbitrary sources, periodic crits, absorbs or damage
modifiers.

Schema 58 appends a motion-aura detail tail after the WOS57 DoT rows. It writes
one `fixed6` value and one `fixed6` graded damage-break scale for every
flattened WOS39 motion-aura row, then one `u32` Fear DR stage and one `fixed6`
Fear DR reset time for every entity. Existing rows decode as value/scale zero,
and WOS2-WOS57 decode with zero DR state. A zero scale retains source classic
break-on-any-damage semantics; a positive scale enables the source probability
`min(1, damage / (scale * maxHp))`. This tail intentionally establishes only
durable data ownership. Fear casting, projectile landing, forced flee movement
and damage-path consumption are owned by the Fear runtime closure below.

Warrior Taunt remains a runtime closure over WOS57's persisted forced-target
id/timer and flattened threat partition. For the retained ordinary Eastbrook
wolf/boar subset it admits the M4 eight-yard/facing target, lifts threat, forces
the target for three seconds, writes the ten-second sparse cooldown and advances
the forced timer at the normal 20 Hz mob update. General map LOS and exceptional
boss/dummy/ignore-taunt templates are not represented by this subset.

Rogue Sinister Strike remains a runtime closure over WOS57's already-persisted
resource, cast-GCD, HP, combat, flattened threat, RNG and combo-point columns.
Its
retained Eastbrook transaction validates the five-yard/facing enemy target,
charges 45 energy, arms the source one-second Rogue GCD and performs exactly one
shared weapon hit. That hit enters combat on every result, invokes idle social
aggro for a live idle target, and adds one capped combo point only on a landed
strike; its physical damage and M4 threat values are also committed to the
flattened mob threat row. `entityComboUntil` restamps to 30 seconds and the 20 Hz player tick
expires only the point pool; the timestamp is intentionally retained after expiry
and death because the source preserves it. General LOS, stealth/poison/talent
modifiers, finishers and non-Eastbrook targets remain outside this runtime slice.

Ability slice WOS58 is a runtime closure (on the then-current schema 57), not
a codec revision: Rogue Eviscerate reuses the
same WOS57 envelope's resource, cast-GCD, position/facing, HP/combat, threat,
RNG and combo columns. A valid retained Eastbrook target spends 35 energy and
the current positive combo pool, then consumes authoritative draws for M4
`range(0, variance)` and physical crit before applying `base + perCombo * spent
+ attackPower / 14`, physical armor reduction and JavaScript rounding. It enters
combat, drives idle social aggro or active target backfill, writes direct-damage
threat and settles lethal target rewards before clearing only combo points. The
source-retained `entityComboUntil` timestamp survives the spend. Auras, stealth,
guaranteed-crit state, damage modifiers/absorbs and non-Eastbrook target classes
remain outside this slice.

WOS59 is a runtime closure, not a codec revision: Rogue Backstab reuses the
same WOS57 resource, cast-GCD, position/facing, M5 main-hand item, HP/combat,
threat, RNG and combo columns. Its source-pinned M4 projection carries the
three rank weapon bonuses, `weaponMult: 1.5` and `requiresBehind`; the existing
M5 item projection supplies the retained main-hand dagger fact. A valid
Eastbrook transaction requires that dagger, normal melee/caster-facing admission,
the source target-back-facing half-plane and its 0.1-yard overlap hold before
charging 60 energy and resolving one shared weapon hit. It preserves the
existing landed-only combo/30-second timestamp behavior and does not change the
state envelope. Stealth, complete weapon catalogs, generalized equipment,
line-of-sight, modifiers and non-Eastbrook targets remain outside this slice.

WOS60 is a runtime closure, not a codec revision: Rogue Gouge reuses the WOS57
resource, cast-GCD, cooldown, position/facing, HP/combat, threat, RNG and combo
columns together with WOS39's motion-aura partition. A valid retained Eastbrook
target spends 45 energy, arms the one-second Rogue GCD and ten-second cooldown,
then consumes the source `range(min,max)` and physical-crit draws before armor
and JavaScript rounding. A live target receives the source ability/caster/
`incapacitate`/positive-remaining row for four seconds; it is refreshed by the
same source ability and removed only after one of the retained positive-damage
paths. On each retained mob tick, forced-target time progresses first, then an
incapacitated mob skips pursuit/melee and its aura ages after its action. This
does not claim a general aura dispatcher, fear diminishing returns, player
crowd-control, stealth/openers, modifier/absorb handling or non-Eastbrook
targets.

WOS61 is a runtime closure, not a codec revision: Rogue Kidney Shot reuses the
same resource, cast-GCD, cooldown, position/facing, HP/combat, threat and combo
columns with the WOS39 motion-aura partition. A current-known primary Rogue
with positive combo points spends 25 energy, records the one-second GCD and
20-second cooldown, then writes an ability-keyed `stun` row for `1 + spent
combo` seconds. The active combo pool is cleared after the effect while its
historical `entityComboUntil` timestamp remains. The source requires stun
diminishing-return history; this retained Eastbrook-only closure represents its
first-category full duration only, and does not claim cross-target/player DR,
immunity, generic aura events or other stun producers.

WOS62 is a runtime closure, not a codec revision: Mage Arcane Explosion reuses
the WOS57 resource, cast-GCD, position, HP/combat, flattened threat and
authoritative RNG columns. Source ability admission keeps it unavailable until
the primary Mage has the committed Arcane specialization. A valid cast spends
60 mana, starts the base 1.5-second GCD and immediately enumerates retained
live hostile Eastbrook entities in stable state order within its ten-yard
caster-centered radius. Each eligible target consumes exactly one source range
draw, receives non-crit Arcane damage including the retained direct spell-power
coefficient, then enters combat and receives damage threat; no cooldown or
additional persistent effect row is created. The retained outdoor projection
has clear line of sight and no spell-haste, talents, auras, absorbs, reflects,
ground aim or non-Eastbrook target classes.

WOS66 is a runtime closure, not a codec revision: Priest Lesser Heal reuses
WOS57's timed-cast target, resource, cast-GCD, HP, owner, combat, flattened
threat and authoritative RNG columns. Its typed and action-bar reducers first
resolve a living non-hostile player or owned pet from explicit target, current
target, then self; an explicitly selected friendly target that is out of the
source `max(range, 5) + 2` horizontal range fails instead of falling back. Cast
completion revalidates that locked target, spends the current generated-rank
mana cost, consumes the generated healing range draw and then the source
spell-crit draw. The heal kernel applies the source post-multiplier rounding and
overheal clamp before splitting 0.5 effective-healing threat over live hostile
in-combat mobs that already have a target or owned-pet threat entry. The WOC
projection has no durable general aura/absorb, healing-modifier, heal-crit
damage, PvP-controller or weapon-proc state; those source branches remain
outside this closure.

The Fear runtime closure is not itself a codec revision: it consumes WOS58's
timed-cast, in-flight projectile, motion-aura-detail, HP/combat and
authoritative-RNG state. A level-14 primary Warlock that knows Fear resolves a
hostile Eastbrook target at the source range/facing gate, arms its 1.5-second
cast without spending, then rechecks the locked target at completion, spends 40
mana and snapshots a zero-damage Shadow projectile. Its landing consumes the
spell-resist draw before the source `[-pi, pi]` direction draw; a landed spell
writes one eight-second `incapacitate` row with `breakChanceScale = 0.1` and
enters combat. In the single-player Eastbrook mob projection, active Fear moves
the affected mob across the retained terrain sample at source capped flee speed
before normal pursuit or melee, then ages the aura. Each retained positive
damage path uses `min(1, damage / (0.1 * maxHp))` and consumes one draw only
while that graded Fear row remains; a chance-one hit still consumes the draw.
WOS58 persists Fear DR stage/reset inputs, but the single-player projection has
no PvP hostility relationship, so PvP hostility diminishing returns are not
active. General collision projection, player-target Fear, generic crowd-control
dispatch and multiplayer target ownership remain outside this closure.

WOS70 Drain Life runtime closure is not a codec revision: it uses WOS58's
existing timed-channel, locked-target, in-flight-projectile, HP/combat,
flattened-threat and authoritative-RNG fields. A level-10 primary Warlock that
knows Drain Life validates a hostile Eastbrook target, spends its rank cost at
channel start, locks that target and arms the source five-second/five-tick
channel. Every live target-validated one-second pulse snapshots a Shadow
projectile; the fixed-count completion flushes all remaining pulses before
clearing the lock. A landing Drain Life projectile consumes exactly one range
draw, including equal min/max ranks, then applies the source channel spell-power
bonus without spell resistance or crit. It commits direct damage/threat before
healing the still-live caster for the rounded, capped damage fraction, and adds
the resulting source healing threat to mobs already aware of that caster. The
single-player Eastbrook closure uses the retained clear-LOS mob subset and its
existing two-yard positional tolerance; player-target hostility, multiplayer
ownership, general LOS/collision and other channel effect families remain outside
this closure. Positive Eastbrook mob melee damage also sends the existing source
channel pushback fraction through the durable cast clock.

WOS71 adds the Sunder Armor tail after WOS58's motion-aura/Fear-DR data: one
`u8` stack count, one `fixed6` remaining duration and one `fixed6` generated
armor display value for every entity. WOS2-WOS58 decode these fields as zero;
an active row is bounded to five stacks and 30 seconds. The runtime admits the
M4 Protection Warrior ability through both action-bar and typed command paths,
bills 15 resource on each instant attempt, consumes exactly one player melee
miss draw and enters combat even on a miss. A landed attempt refreshes the
30-second row, adds one stack through the source cap, and adds the generated
flat threat with WOC's currently representable neutral physical-threat
multiplier. All retained physical player damage paths read `baseArmor *
(1 - 0.02 * stacks)` while the row is active; the rank armor value is retained
for source aura presentation but does not drive source mitigation. This
single-player Eastbrook closure does not yet retain a Warrior defensive-stance
aura, player targets, Expose Armor, Faerie Fire max-combination, or multiplayer
aura ownership; those source features require their own durable combat-aura
projection rather than overloading the Sunder row.

WOS72 adds schema 60 after the WOS59 Sunder rows: a bounded global `u16` count
(at most 64), then target/source ids, ability code, resolved positive per-tick
healing, remaining/original duration, interval and timer for each Rejuvenation
row. WOS2-WOS59 decode an empty queue. The M4 Druid closure accepts the instant
friendly 30-yard spell through action-bar and typed commands, freezes the pure
HoT's base-plus-spell-power tick at application, charges its rank resource and
applies the source hasted 1.5-second GCD with a 0.75-second floor. One
Rejuvenation replaces the prior instance from the same source on its target
rather than stacking.
The player-stage fixed tick first ages the row, applies its every-three-second
effective heal without RNG or heal crit/absorb processing, distributes normal
effective-healing threat when the caster remains live, then removes the row
only after its final 12-second tick. This scoped closure intentionally reuses
the existing friendly player/owned-pet target projection; general friendly NPC
auras, multiplayer ownership and durable arbitrary healing modifiers remain
outside this Eastbrook slice.

WOS73 adds schema 61 after the WOS60 Rejuvenation rows: a bounded global `u16`
count of Consecration ground-effect closures, each storing its source/id,
fixed X/Z center, generated radius/minimum/maximum, application-time spell
power bonus, remaining duration, interval and timer. WOS2-WOS60 decode an
empty queue. The Paladin M4 command is instant, bills 60 resource, writes its
eight-second sparse cooldown and performs the source immediate 8-yard Holy
pulse before appending its ten-second/two-second-cadence row. The fixed tick
walks rows in reverse creation order before projectiles, decrements time before
each due pulse and resolves every living Eastbrook hostile through the existing
GroundAoE range/damage projection. Concurrent casts remain separate source
zones. Ground targeting, generalized LoS, aura damage modifiers and targets
outside the retained Eastbrook subset remain future work.

WOS74 retains M4 Warlock `summon_imp` without changing schema 61 because an
Emberkin uses the existing fully serialized entity row. The action-bar and
typed routes arm the source 5-second spell-haste-scaled cast and 1-second-floor
GCD; completion then bills 50 resource, retires the previous live owned pet,
and creates the level-matched friendly Emberkin at the source `(+2,+1)` ground
position. Its HP, weapon, armor, movement speed and presentation identity are
the pinned `WARLOCK_PET_MOBS.emberkin` values. The source removes a replaced
pet from its entity Map. Until Plugins08 supplies transaction-safe generic
mutable-array entity removal, the WOS projection keeps that replacement as an
inert, dead, owner-bound row while admitting only one *live* owned pet. Pet
follow/assist, ranged Firebolt, generalized pet targeting and the exact physical
row-removal contract remain later source slices.

WOS75 adds schema 62 after the WOS61 Consecration queue. Every entity receives
`u16 imbue ability code`, `u8 application rank`, and fixed-six-decimal remaining
seconds. An all-zero row means no imbue; a live row is currently the one
mutually-exclusive Paladin `seal_of_righteousness` aura. Persisting its rank
captures the source Aura's resolved `value`, `value2`, and `value3` at cast
time, so a level-up cannot rewrite an already-active Seal. WOS2-WOS61 decode
all-zero rows. The instant Seal command costs its generated rank amount, applies
the source spell-haste GCD with a one-second floor, replaces the active imbue,
and gives every retained direct melee swing the generated flat bonus before its
normal physical crit and armor stages. `judgement` requires that live Seal
before cost/GCD/cooldown admission, consumes it before its exact range and spell
crit RNG draws, resolves through the existing M4 numeric dispatcher as Holy
damage, and writes its ten-second cooldown. Imbue expiry runs after player auto
attack, matching the source player update tail. Generalized multi-aura state and
future non-Paladin imbues remain outside this Eastbrook projection.

Exact field order, defaults, bounds, encoder checks, and legacy migrations are
owned by `scripts/woc_game/src/world/state.zr`. This document records the
cross-module version boundary only; it must not become a second codec.

## Historical WOS18 baseline

`WOS18` was the committed envelope before the WOS19-WOS38 additions. The
following table and historical schema notes preserve its compatibility details.

All fields are little-endian. Schema version 18 contains:

| Field | Type | Initial value |
| --- | --- | --- |
| magic | `4 x u8` | `WOS2` |
| schema | `u16` | `18` |
| authoritative tick | `u64` | `0` |
| simulation time | `u64` microseconds | `0` |
| next entity id | `u64` | `408` |
| package generation | `u64` | `0` |
| authoritative RNG state | `u32` | `1` |
| authoritative RNG draw count | `u32` | `0` |
| authoritative RNG draw digest | `u32` | FNV-1a offset `2166136261` |
| authoritative world seed | `u32` | offline target seed `20061` |
| offline-session active | `u8` boolean | `0` |
| offline player identity when active | class `u8`, skin `u16`, name length `u16` plus 2-16 ASCII bytes | absent |
| offline quest ledger | two state `u8` values, two objective `u16` values, derived q_boars hide-progress `u16`, copper/xp `u32`, lifetime XP `u64` | all zero |
| offline talent loadouts | `u16` count, `u16` active index (`65535` means none), then bounded name bytes, specialization and six row codes, plus 22 action-bar ability codes per row | empty / no active row |
| next party id | `u64` | `1` |
| entity row count | `u32` | `0` |

Entity rows are strictly ordered by nonzero entity id and currently contain:

| Field | Type |
| --- | --- |
| id / generation | `u64` / `u32` |
| kind / template code / level | `u8` / `u16` / `u16` |
| position / previous position | six signed fixed6 values |
| facing / previous facing / velocity x,y,z / fall-start y | six signed fixed6 values |
| spawn position | three signed fixed6 values |
| movement speed | signed fixed6 |
| hp / max hp | signed-magnitude integers |
| hostile/dead flags and AI state | `u8` bitset / `u8` |
| on-ground / jumping flags | bits 0 and 1 of a separate `u8` movement bitset |
| target id | `u64` |
| auto-attack / command-sequence-initialized flags | bits 2 and 3 of the entity flag byte |
| last accepted command sequence | `u32` |
| held movement flags | `u8`: forward/back/turn-left/turn-right/strafe-left/strafe-right/jump |
| movement acknowledgement / accepted tick | `u32` / `u64` |
| respawn / corpse timers | two `u64` microsecond values |
| threat entries | `u16` count followed by ordered (`u64` target, signed fixed6 value) pairs |
| cast lifecycle | WOS8-added ability ids, fixed6 timers, compact `u16` boolean flags, bounded channel/queue/empower counters, lifecycle totals and cancellation reason |
| ability runtime | WOS15 signed resource/max-resource values, locked timed-cast target `u64`, then an ordered per-entity `u16` ability-code / future-expiry-`u64` cooldown partition |
| known abilities | WOS9-only `u16` source-catalog codes, partitioned per entity and kept in source class-kit order |
| talent allocation | WOS10-only `u16` specialization code plus six `u16` level-row option codes; only the offline primary player can retain nonzero codes |
| resurrection offer | WOS12-added presence byte, caster `u64`, hp fraction and fallback x/y/z as fixed6, then expiry `u64` microseconds |
| spirit state | WOS13-only ghost and corpse-presence bytes, corpse x/y/z as fixed6, and resurrection-sickness `u64` microseconds |
| party/raid state | WOS14 party id and leader `u64`, raid byte, subgroup byte, member-order `u16`, then pending invite sender and expiry `u64` values |
| dead-target metadata | WOS17 owner `u64` (`0` is source `null`) and lootable `u8` after party state; only a dead player, lootable corpse, or viewer-owned dead mob is selectable |
| weapon stow state | WOS18 `u8` after dead-target metadata; `stow_weapon` toggles it only for a living actor and a valid `attack` clears it |
| motion aura partition | WOS39 `u16` count followed by (`u16` source ability code, `u64` source entity id, `u8` generated kind code, fixed6 positive remaining seconds) rows |
| active form transition | WOS40 `u16` source ability code and signed saved-mana maximum after the motion-aura partition; `0` is no active form |
| baseline stat profile | WOS41 `u8` after active-form state; `0` is no M5 baseline and `1..9` select source class order. A nonzero profile derives from the retained Talent V2 row; it does not imply unpersisted equipment, set or aura inputs. |
| spell haste | WOS42 signed fixed6 after the baseline profile; must equal the current source-pinned M5-plus-retained-talent derivation for a nonzero profile, otherwise `0` |
| M5 equipment identities | WOS43 three `u8` codes after spell haste: helmet, feet, mainhand. `0` is empty/start-mainhand; nonzero is the pinned M5 item index plus one. Only the current scalar M5 three-slot projection is represented. |
| M5 inventory / bags / copper | WOS44 signed copper, four bag `u8` codes, and an ordered `u16`-bounded dense `(item code u8, count u16)` stack partition. Player capacity is the 16-slot backpack plus equipped M5 bags; nonplayers retain the empty/zero default. |
| WOS45 mainhand semantic | WOS45 keeps the WOS43/WOS44 three equipment bytes but reserves mainhand `255` for explicitly unarmed. `0` remains the source starting mainhand; WOS44's 14 catalog indexes stay fixed and seven class-start identities append after them. |
| WOS47 vendor buyback | WOS47 appends a `u8` count (at most 12) and newest-first `(item code u8, count u16)` buyback rows after WOS44 inventory stacks. The scalar subset supports source-generated vendor stock plus buy/sell/buyback/sell-all-junk; item instances remain unrepresented. |
| WOS48 q_boars projection | WOS48 retains WOS47 bytes but derives the WOS7 q_boars hide-progress field from the primary player's scalar `boar_hide` stacks while active/ready. It consumes five real stacks on turn-in and migrates an older active/ready progress field to at most five hides. |
| WOS49 scalar discard | WOS49 retains WOS48 bytes and attaches existing command 24 to scalar tail-stack removal. Missing count means one; a present count clamps to available scalar stock. Generated scalar `noDiscard`, `noVendorSell` and `soulbound` flags retain source policy; instance identity still needs a structured state authority. |
| WOS50 idle-wander target | WOS50 writes an explicit target-present `u8` and fixed6 target X/Z after the WOS23 wander timer. WOS2-WOS49 rows default to no target and zero coordinates. Offline Eastbrook initialization restores the post-constructor camp-spawn RNG cursor before its first idle-wander draw. |
| WOS54 pending projectile queue | After the entity table and bounded Card Duel snapshot, WOS54 writes a `u16` launch-order count (at most 64), then source/target `u64`, horizontal x/z and ttl fixed6 values, a wand marker, school code, and captured ranged min/max/speed fixed6 values. Hunter retains physical `5-9/2.3`; Mage/Priest/Warlock/Druid wand rows retain arcane/holy/shadow/nature `3-6/1.8`. WOS53 rows default the new code to physical; attacker stats and target position remain live landing-time reads. |
| WOS55 queued-on-swing rows | After the WOS54 queue, one `u16` ability code, `u8` free marker and fixed6 multiplier are emitted per entity. WOS2-WOS54 default to `0,false,1.0`; present rows must identify a current-known M4 `onNextSwing` ability. |
| WOS56 Frostbolt projectile closure | Every WOS54 queue row additionally writes `u16 ability code`, `u8 rank` and fixed6 base cast time before the WOS55 per-entity rows. WOS2-WOS55 default those fields to `0,0,0`; nonzero values identify exactly a generated Frostbolt rank with Frost school, original direct-damage profile and standard projectile speed. |
| WOS57 hostile-spell DoT tail | After the WOS55 per-entity rows, a bounded `u16` count (at most 64) writes source/target `u64`, ability `u16` code, positive `u16` per-tick damage and fixed6 remaining/duration/interval/timer values. WOS2-WOS56 default this tail to empty; rows must match one generated Fireball or Moonfire periodic profile. |
| Card Duel snapshot | WOS16 `u32` byte length plus a bounded opaque `CDS1` service snapshot after the entity table |

The existing `attack` command now applies the source initial melee pull only
when its live hostile mob target is ownerless, idle, inside five yards and the
player is not casting. It stands the player, arms auto-attack, records the
primary's chase/aggro/combat/leash state, seeds its threat and applies the
second source `startAutoAttack` threat. Same-template idle social allies within
the family pull radius receive chase/aggro/combat, their own current position as
leash anchor, and one threat. Source affix cascades, revenge, full death
teardown, retargeting and combat exit remain separate M4 transaction work.
Offline direct-melee profiles now consume that
armed target through the source mainhand white-melee timing and hit table:
warrior, rogue, paladin and shaman have no source ranged white-hit profile, while
druid is admitted only in Bear/Cat form. Hunter uses this same path at five yards
or closer. Mainhand/offhand white sequence, range/facing, swing timers,
weapon/AP/crit/armor and Mulberry cursor writeback are state-owned, and a lethal target reaches
the ordinary wild corpse branch in the same tick. The retained wild boar also
applies its source fixed two-damage Bristled Hide reaction after a landed melee
hit, including a killing hit while the attacker is still alive. WOS54 adds the
source ranged launch/flight/landing subset: Hunter Auto Shot launches at 8-35
yards and caster wand at 0-30 yards without pre-aggro, records a homing projectile
without consuming RNG, and advances it before player swings. Each uses live target
x/z, forces landing at the source three-second ttl, and fizzles with no damage,
credit or RNG draw when an endpoint dies. Hit/miss, range and crit draws occur
only at impact; captured profile plus physical/arcane/holy/shadow/nature school
survive rollback while live ranged power, hit, crit and target armor remain
landing-time reads. The five-yard Hunter mainhand fallback reuses the direct-
melee bridge, while the 5-8 yard dead zone and out-of-range state advance its
swing timer without a hit. Bear/Cat druid forms use the melee fallback; Travel
and action-locking Fireball travel form cancel auto-attack after timer decay, while
Moonkin retains nature wand. Dual-wield plus Heroic Strike/Raptor Strike queued
mainhand consumption are state-owned; aura reflection and set/talent/weapon
proc producers remain separate transactions.
A direct Eastbrook white-melee lethal hit first commits its local swing RNG
cursor, then evaluates the source `mobXpValue` curve before quest credit and
corpse loot. The retained single-player subset carries the source base
`45 + 5 * mobLevel`, `zeroDiff` gray threshold and level-difference rounding;
it deliberately excludes party division/bonus, elite multipliers and rested XP.
A killed Forest Wolf then invokes the retained `q_wolves` kill-credit reducer;
its active-state/eight-kill bounds remain owned by that existing ledger. The
source-order corpse table follows: chance for copper, its separate range draw,
active-and-needed quest-drop chance, and the remaining ordinary entries. WOS52 persists the resulting
Eastbrook corpse copper, the personal `boar_hide` slot, and the ordered ordinary
item slots. Its single-player `loot` reducer transfers copper before capacity-
gated personal and ordinary items; full bags leave items on the corpse. Party/
FFA rights, group rolls and broader death-credit selection remain later source
transactions.

Before the retained idle-wander arm, an offline Eastbrook wolf or boar now runs
the source idle nearest-player selection against its generated `aggro_radius`,
level/trivial-con and strict-radius rules. The current offline subset has no
stealth or delve detection modifier, so it supplies their source-neutral values
and routes a selected player through the same primary/social aggro transition.
The resulting chase rows skip idle wandering. The retained direct live-player
target also advances the source melee-pursuit profile at 20 Hz: it commits the
pre-step transform as previous position, moves and turns only while outside the
source melee distance, decrements/rearms the existing swing timer and labels the
row `attack` at contact or `chase` otherwise. Each emitted swing resolves the
source one-roll miss/dodge/parry/block table, weapon/AP damage, fixed 5% crit,
armor reduction and HP update through the committed Mulberry cursor; the cursor
state, draw count and digest write back with the candidate. The retained
single-player death handoff clears HP, cast/movement/combat posture and releases
Eastbrook pursuers to idle. It does not own ranged, dual-wield or unsupported-
form player auto profiles, revenge, aura or affix cascades, full death teardown,
multi-player retargeting, pull-over, leash/flee or combat exit.

The same ownerless Eastbrook wolf/boar subset now has the source ordinary wild
mob corpse branch: a nonpositive HP row enters `dead`, receives the source
60-second corpse and 30-second respawn clocks, clears aggro threat plus forced
target, and decrements both clocks at 20 Hz. A lootable corpse holds an expired
respawn until its corpse window ends; otherwise the row returns to its stored
spawn transform at full HP, hostile/idle and consumes the source `range(2, 8)`
Mulberry draw for its next wander timer. This reuses existing WOS timer and
spawn columns, while WOS51 appends the retained personal quest-loot slot and
WOS52 appends the Eastbrook copper/shared-item corpse payload. General credit
selection, party/elite/rested XP, corpse harvesting, pets, instances, rares/world
bosses, auras and the full damage dispatcher remain their separate source owners.
The direct player Forest Wolf white-melee death is the narrow exception: it awards
the source-shaped single-player mob XP, then invokes the existing `q_wolves`
active-task ledger before this lifecycle begins.

The decoder caps the entity section at 100,000 rows and the retained WOS16 Card Duel
snapshot at 16 MiB, rejects unsorted or
duplicate ids, invalid kind/AI/flag values, noncanonical signs and trailing
bytes. The current WOS57 decoder accepts schemas 2 through 57. WOS2-WOS54
default every queued-on-swing row to empty, while WOS2-WOS55 default every
projectile closure row to zero and WOS2-WOS56 default every hostile-spell DoT
tail to empty. Threat targets are nonzero and strictly ordered within each entity;
threat values are nonnegative and partition offsets must cover the flattened
arrays exactly. Every parallel state column must have exactly the declared row
count before encoding.
Schema-2 defaults auto-attack and command sequence state; schemas 2 and 3
default previous position to current position, both facings and all velocities to
zero, on-ground to true, jumping to false and fall-start y to current y. This
matches the target entity constructor before a legacy candidate is re-encoded as
WOS52. In schemas 5 through 52, a zero movement acknowledgement requires zero held flags
and a zero accepted tick; a positive acknowledgement requires a positive accepted
tick no later than the containing authoritative tick. This prevents a persistently
held input from existing without the positive sequence receipt that created it or
from postponing its stale-clear window beyond state time. Aura, event and
durable-player sections will extend the same versioned envelope with their own
explicit counts and bounds.

When the schema-6 through schema-17 offline marker is present, class uses the
source-content order, skin uses the source class-change range `0..7`, and name
bytes use the target 2-16 ASCII rule.

An empty committed state initializes this envelope. Each accepted transaction
must advance exactly one tick and 50,000 microseconds. Its generation must stay
unchanged or increase by exactly one after an accepted hot reload; the candidate
absorbs that new generation. The VM rejects digest drift, skipped/repeated
ticks, generation rollback and generation skips before producing a candidate.
Every gameplay RNG draw updates the three RNG fields on the candidate state.
Rejecting a tick therefore rolls random-stream progress back together with all
other gameplay state; process-global RNG state is not authoritative.

The fixed-tick decoder retains every command id, actor id/generation, sequence
and payload byte in deterministic batch order. The first authoritative reducer
slice implements `target`, `attack`, `stopattack` and `stow_weapon` using the generated
contract in `contracts/command_payloads.json`. Target uses one little-endian
`u64`, where zero clears the target; the attack and weapon-stow toggles have
empty payloads. A dead actor cannot change its stow state, while a successful
attack clears a stowed weapon before enabling auto-attack.
The same contract now pins `castSlot` as signed `i32_le`, matching the
upstream `msg.slot | 0` conversion. Its payload is transport-ready, but the VM
continues to reject its gameplay dispatch until spellbook/casting state lands.
`tab`, `targetNearest`, `tabFriendly` and `targetNearestFriendly` are empty
payloads routed through `world/target_selection.zr`. The current WOS projection
supplies live hostile rows and non-hostile player rows, including source-facing
ordering, range limits and stable friendly ties. PvP, arena, Vale Cup and pet
relationship rows are still absent from that projection, so those source
selector cases remain unported rather than being synthesized by the reducer.
The payload contract has its own generated SHA-256 in both the package schema
identity and every native host identity, so matching command ids cannot conceal
different byte layouts.
Actor generation, strictly increasing per-actor sequence, exact payload
partitioning and command support are validated before the candidate can be
committed. Unported commands are rejected explicitly and never discarded.
Player, aura and event state will extend this versioned envelope as M4
converges; any incompatible layout change requires another schema increment and
hot-reload migration.

Schema 7 added the session-only offline quest ledger. Its current reducer accepts
the source-pinned `q_wolves` and `q_boars` identifiers only through the generated
payload contract, checks the target seven-yard quest-NPC tolerance, and records
accept/abandon/turn-in state atomically. The source-shaped single-player kill-XP
hook, `q_wolves` kill credit, the WOS51 personal `boar_hide` corpse-loot grant
and WOS52's copper and ordinary corpse-item slots together produce the generated
objectives; the latter is recomputed from the retained hide count on accept and
consumes five hides on turn-in. Generic combat-death credit and the remaining
inventory/loot integration are still separate M4/M5 work.
Offline bootstrap materializes Marshal Redbrook and Trader Wilkes as non-hostile
kind-3 rows, with template codes equal to their generated M5 NPC catalog index
plus one. The quest reducer scans those rows instead of accepting a static map
coordinate with no NPC entity present; the complete target NPC roster remains
later M8/M10 world-content work.
After strictly decoding an offline WOS6 snapshot, the schema-7 through 18 reader
idempotently materializes either missing fixed quest NPC row before returning
the state. Re-encoding therefore writes WOS18 with the same player plus those
source-pinned NPC identities; WOS12 snapshots that already contain them are
not duplicated.
Account rewards, daily rewards, mail and network quest resynchronization remain
server-owned M9 responsibilities.

Schema 8 appends one compact cast-lifecycle row after each entity's threat
partition. It persists the complete `combat/casting_state.zr` transition state:
cast/channel timers, delayed queue, control/lockout flags, pushback, empowered
and Ice Floes counters, lifecycle totals and the two Mass Resurrection rejection
reason codes. Schemas 2 through 7 append a zero-valued row during decode.
`WorldState` copies each living player row into `CastState`, runs the
source-order `advanceFixedTick`, then writes the result into the same candidate.
Ability admission, cast arming and effect resolution remain separate WOS work.
When the currently wired landing path marks a row dead, WOS then mirrors
`handleDeath`'s direct teardown: it clears only the active ability and retained
cast-target lock. It does not call the ordinary cancellation helper, increment a
cast-stop total, clear the queued ability, or manufacture an event. Generic
combat death, respawn and cast-target identity remain later WOS work.

Schema 9 appends a `u16` count and that many `u16` ability codes after the
Schema-8 cast row. Codes refer only to the generated current-head ability
catalog; the list order is the source `PlayerMeta.known` base-list order. The
offline bootstrap fills its freshly spawned player from the source class kit at
the bootstrap level with no grants and no committed specialization. When the
existing offline quest-XP reducer changes that player's level, it replaces that
entire partition using the same source eligibility gates while preserving all
other entity partitions. A base rank is derived from each catalog code and the
entity level by the source-declared rank order; it is intentionally not stored a
second time. Schemas 2 through 8 decode to empty partitions. Talent modifiers,
grant state, action-bar bindings, cast admission and resolved ability effects
are not represented by this row.

Schema 10 appends a specialization `u16` and six fixed-row option `u16` codes
after the Schema-9 known-ability partition. The six rows correspond to source
levels 5, 8, 11, 14, 17 and 20. The decoder gives schemas 2 through 9 an all-zero
allocation, then rejects nonzero allocation on non-player rows. For the offline
primary player it checks the current generated class/spec/row catalog and that
each nonzero choice has reached its source level. A successful state-local
replacement atomically rebuilds only that player's known-ability partition in
source order: specialization signature, selected row grants, base-plus-grants
dedupe, then normal eligibility. Numeric modifiers, effects and action-bar
execution remain outside this allocation row. The bounded offline command
reducer consumes typed allocation, specialization, respec and row-selection
payloads but remains source-only until the open lossless transactional Plugins
08 boundary permits dynamic proof.

Schema 11 inserts the offline saved-loadout projection after the quest ledger
and before the entity count. It stores at most ten rows. Each row carries a
1-96-byte raw name representation, one specialization `u16`, six fixed
row-option `u16` codes, and 22 action-bar ability `u16` codes. The active index
is a `u16`, with `65535` representing no active loadout. Schemas 2 through 10
migrate this section to an empty list and no active index. The decoder validates
the flattened name/row/action-bar partitions, source class and level eligibility
for allocations, and catalog membership for nonzero action-bar codes.
`switchLoadout` and `deleteLoadout` consume their typed `u32_le` index payloads
only for the offline primary player; they reject invalid/missing indices and the
reachable combat lock without mutation. Deleting the active row applies the
source fallback row before removal, while deleting a preceding row decrements
the active index. The source `saveLoadout` command still has no canonical typed
wire payload, so the WOS11 loadout projection does not expose an invented creation/update operation.
Action-bar codes are retained data only until a live action-bar runtime exists.
The raw-byte name bound is a deterministic storage bound, not a claim that the
source JavaScript UTF-16 truncation policy has been reproduced.

Schema 12 appends a source-shaped pending-resurrection projection after each
entity's talent row: presence, caster id, health fraction, offer-time fallback
position and absolute expiry. Exactly one offer is retained per dead player.
WOS2 through WOS11 decode these columns as canonically empty. At the end of each
fixed tick, rows are cleared when the target has revived or the authoritative
microsecond clock reaches the 30-second expiry. `resurrect_respond` resolves the
command actor as the target, consumes an existing offer before evaluating decline,
expiry or death, and on acceptance restores `max(1, Math.round(maxHp * hpFrac))`
at the live caster's ground position when that caster is a living player, otherwise
at the stored fallback position. The WOS slice clears retained movement, targeting,
auto-attack and active/queued casting projection with that revival. The effect
dispatcher, resource pools, auras and events remain separate M4 work, so this
is durable response state rather than a claim of complete resurrection-system
parity. The current-head `Temporal Reversal` contract now drives WOS15's direct
`cast` reducer: its exact `resurrectAlly` 35% fraction reaches this offer path
only after the source's dead group/raid target, cast, range, resource and
cooldown checks have completed. A generic M4 effect dispatcher remains separate
work.

Schema 13 appends spirit state after Schema 12's offer fields. `release` records
the death location as a corpse, marks the dead player as a ghost and places that
ghost at the nearest source-pinned overworld graveyard (stable first-entry tie
break). Ghost rows retain normal movement input at 125% speed but cannot take
landing damage. `resurrect_corpse` requires that ghost to be within 35 horizontal
yards of its corpse and restores 50% health at its current position.
`resurrect_healer` requires the current ghost position to be within eight yards
of a static source graveyard healer, restores 20% health, and writes the
source-level resurrection-sickness duration. The `u64` sickness countdown drops
by 50,000 microseconds per committed fixed tick. Revivals clear ghost/corpse
data but preserve sickness; legacy schemas decode these columns as empty. The
current WOS slice records that timer only: the source's `-75%` all-stat aura and
its resulting health/resource recomputation require the missing durable aura and
stat pipeline. This offline slice intentionally also excludes source arena/delve
routing and event ownership.

Schema 14 writes `nextPartyId` after the pre-existing offline-loadout header and
appends a durable PartyMachine projection to every entity row. A party member
retains its party id, leader id, raid marker, subgroup, source join-order ordinal,
and one pending incoming party invitation. `pinvite`, `paccept`, `pdecline`,
`pleave`, `pkick`, `ppromote`, `praid`, `punraid`, and `pmoveRaid` mutate that
same decoded candidate. The reducer preserves the source's 5-player party and
10-player/two-subgroup raid caps, exact 30-second inclusive invitation boundary,
leader handoff to the earliest remaining join, and subgroup normalization after
a raid member leaves. Schemas 2 through 13 append canonical empty social columns
and retain `nextPartyId = 1` when decoded. Trade/duel invitation exclusion,
social events, loot strategy, ready checks, finder formation and client-facing
party views remain outside this compact WOS owner.

Schema 15 appends resolved-ability runtime state immediately after the Schema-8
cast lifecycle row: nonnegative current/max resource, the `u64` target locked
at timed-cast start, and a canonical per-entity ability-code/cooldown-expiry
partition. Schemas 2 through 14 append zero resources, no cast target and an
empty cooldown partition. The first connected consumer is `cast temporal_reversal`:
its exact generated id, level-gated known ability, 60 resource cost, active cast
or GCD, running cooldown, dead same-party target and 30-yard corpse-or-body
range are checked at admission. Its two-second cast locks that target; completion
rechecks only dead same-party membership, then creates the existing 35% offer,
spends the resource and records the 600-second cooldown. A changed target state
fizzles before billing. Expired cooldown entries are removed after simulation time
advances, before the candidate is encoded. Other cast ids intentionally remain
outside this reducer. The generated source contract supplies Temporal Reversal's
mage base GCD of 1.5 seconds; spell-haste and the 0.75-second minimum-GCD path
remain outside the present stat/aura projection.

Schema 18 appends one source-visible weapon-stowed byte after WOS17 dead-target
metadata. Schemas 2 through 17 decode that byte as false. The generated empty
`stow_weapon` command toggles it only while the actor is living; a successful
hostile auto-attack clears it, matching `toggleWeaponStow` and `drawWeapon`.

The package `stateSchema()` advertises `WOS18`, matching the current writer.
Card Duel restores its Mulberry32 cursor from the enclosing WOS RNG fields and
copies that cursor back before writing the `CDS1` payload. The embedded CDS1
cursor is therefore a compatible redundant representation, never a separate
minigame RNG authority; all target-Sim random draws stay in the single
transactional stream.

Fixed-tick protocol version 3 additionally carries a separate bounded movement
frame batch after the package generation. Each frame has actor id/generation, a
positive non-wrapping sequence, forward/back/turn-left/turn-right/strafe-left/
strafe-right/jump flags, and an explicit `has_facing` plus finite `f64 facing`
through +/-1000. Rust canonicalizes frames by actor identity before encoding;
both decoders reject duplicate/noncanonical actors, zero identity or sequence,
invalid booleans, non-finite/out-of-range facing and trailing bytes. The
transport relay applies every valid received frame, keeps the acknowledgement as
the monotonic maximum sequence, retains facing when a later frame omits it, and
clears held directional flags only after more than fifteen 20 Hz ticks (750 ms).
This is intentionally not a command.

After the movement batch, the same fixed-tick envelope carries a bounded
optional `OfflineSessionBootstrap` byte payload. It is allowed only on the first
empty-state transaction and contains bootstrap version, fixed seed `20061`,
canonical source class index, target-valid name bytes and class-valid skin. Rust
retains it across a rejected candidate and removes it only after a successful
Tick 1; the ZrVM decoder independently verifies and consumes it to construct the
fresh player from generated source content. Every later transaction carries an
empty bootstrap field.

The WOS14 envelope retains each player's held movement flags, acknowledgement high-water mark
and accepted tick. The authoritative reducer applies every valid canonical
frame, keeps acknowledgement at the maximum sequence even when a lower sequence
later updates held state, retains facing on an omitted-facing frame, and clears
held flags only when `current_tick - accepted_tick > 15`. It then routes every
live player through `world/player_motion_transition.zr` at 20 Hz, committing
previous/current transform, facing, velocity, jump and ground fields together.
The current WOS damage column receives landing damage directly; combat events,
auras, crowd control, casts and durable death handling still belong to later
M4/M5 authoritative integration.

The static built-in terrain inputs consumed by this reducer are generated
from the pinned target source into `contracts/m3_terrain_content.json` and
`world/terrain_content.zr`. They cover the three zone boundaries/hubs, five lake
circles, water/world/instance constants, 67 camps, the built-in terrain edit,
two docks, 14 road polylines, Sowfield flatten/shell and decoration-exclusion
inputs. `hasWaterAt` is deliberately an explicit predicate, preserving the
target's no-water `-Infinity` behavior without replacing it with a finite value.
`world/terrain_height.zr` composes the
source-pinned builtin `terrainHeight` sequence, while `world/terrain_ground.zr`
adds the strict flat-instance threshold, Sowfield tiers and source dock-plank
surface maximum. Dynamic custom-content selection and complete decoration and
active-run collision coverage remain required before movement can be accepted as
target-parity evidence.

`world/terrain_gradient.zr` supplies the pure four-sample steepness function
and JavaScript-compatible rounded-cell query consumed by the motion transition.
`world/decoration_candidate.zr` mirrors the fixed-world
candidate/filter/radius rule, but it is not a cached decoration collider field.

`world/player_motion_world.zr`, `world/player_motion_vertical_world.zr` and
`world/player_motion_wall_standoff.zr` remain the pure ports. WOS14 composes
them only through `world/player_motion_transition.zr`, so no duplicate movement
math or presentation-side position authority is introduced.

`world/terrain_wall_standoff.zr` supplies the source body-width terrain-wall
setback, and `world/player_motion_wall_standoff.zr` supplies its compensation
and second resolve inside that atomic transition.

`world/collision_geometry.zr` supplies the exact scalar circle/OBB push-out
primitive used by the swept resolver. The fixed builtin prop and Vale
Cup set is generated into `m3_collision_content.json` and resolved in source
order by `world/collision_static.zr`, with source-style substeps/fence crossing
in `world/collision_sweep.zr`; seeded decorations, custom maps and active-run
Delve routing remain deferred from this built-in-world reducer.

`world/instance_collision_content.zr` and
`world/instance_collision_static.zr` now cover five source dungeon/arena local
layouts. `world/instance_collision_routing.zr` covers standard dungeon/arena
band, slot and local/world selection; active-run Delve context and complete
unified collision coverage remain required before movement can be accepted as
target-parity evidence.

The schema-2 local state codec and fixed-tick protocol known vectors passed in
interpreter and binary modes before this extension. Schema 8 added source-level
round-trip and cast-lifecycle tick assertions, WOS9 adds the known-ability
partition, WOS10 adds allocation round-trip/static validation, WOS11 adds
loadout migration/reducer state coverage, WOS12 adds resurrection-offer state
coverage, WOS13 adds the offline spirit-loop state coverage, WOS14 adds
party/raid command and storage coverage, WOS15 adds Temporal Reversal's
resource/target-lock/cooldown completion coverage. WOS16 adds the bounded
Card Duel `CDS1` snapshot, typed command ingress and fixed-tick pairing/expiry,
and WOS17 adds dead-target owner/lootable persistence plus target admission
coverage. WOS18 adds `stow_weapon` state persistence, dead-gate and successful
attack-draw coverage, but no
dynamic pass is claimed: the WOS8 `CastState` copy-in/copy-out
intentionally exercises the same natural
decoded-batch and instance-field boundary blocked by the open Plugins 08 ZrVM
handoff. None of this is evidence of an accepted end-to-end engine bridge.
