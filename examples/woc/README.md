# World of Claudecraft for ZirconEngine

This project is the one-to-one ZirconEngine reconstruction of
`dev/world-of-claudecraft` at commit
`5ef9f7cb21cd8875b6d2c49701015dfcd78de35a`.

The active protocol, role identity, generated contracts, trace symbols and
native parity suite use `reference/current-head`; its 54 current-head goldens
are materialized and SHA-256-locked inside this project. The older
`7c10f280eec380e9877e66ce16333089e171fe42` contracts remain explicit archive
evidence only. A historical regeneration requires `--historical` and cannot
become an active current-head product path.

Some earlier focused ZrVM modules correctly retain their historical source-slice
provenance. They are partial authored work, not evidence that those behaviors
match current-head or that the project is playable. Current-head parity still
requires their rebase and a real ZrVM transaction.

`reference/current-head/source_manifest.json` pins the rebuilt target
inventory: 165 commands, 248 `IWorld` members across 28 facets, 54 parity
goldens, 949 GLBs with 714 animations and 158 skins, and 14,716 test
registrations. Its sibling catalogs own the per-row source and future WOC
mapping; they are inventory evidence, not a completed gameplay port.

The authoritative gameplay implementation is the ZrVM package under
`scripts/woc_game`. Rust code under `native` owns contract generation,
validation, engine adaptation, transport, persistence and executable roles. It
must not contain a second implementation of gameplay rules.

Runtime clocks are fixed by contract:

- authoritative simulation: 20 Hz;
- default client presentation: 60 Hz;
- one batch ZrVM transaction per simulation tick;
- complete rollback on a VM, budget, decode or command-validation failure.

The ZrVM package and native protocol both target the versioned `WOS71` envelope
documented in `contracts/world-state.md`. Its `stateSchema()` and writer select
WOS71, while the decoder retains WOS2-WOS70 compatibility. WOS71 adds
death-time party loot-recipient snapshots, deterministic fair-split copper and
a rollback-safe poor/common round-robin cursor. WOS70 adds bounded,
rollback-safe pending Need/Greed and Master Loot records, candidate choices,
hidden 1-100 rolls, 60-second resolution and all-Pass corpse return; a 300-second
unassigned Master Loot record converts to a fresh Need/Greed window. The master
looter can assign one candidate directly or turn a selected candidate subset into
that same 60-second vote without changing the retained roll id. WOS69 adds independent
18-second food and drink slots with two-second HP/Mana ticks and source-aligned
standing, damage and death interruption. It also retains party-scoped raid
markers with unique `0..7` symbols, same-target toggling and death/disband
cleanup, plus leader-owned Master Loot settings with member inheritance and
effective-looter fallback. WOS68 previously added a school
discriminator to each motion-aura row so Skull Bash preserves a nonphysical
spell-school lockout across snapshot restoration. WOS63 persists the
rank and selected source-power snapshot for the retained pure DoT rows, so a
restored Serpent Sting or Shadow Word: Pain never recalculates from later gear.
WOS64 applies the same rule to new Rejuvenation and Renew pure HoTs; pre-WOS64
Rejuvenation rows preserve their resolved legacy tick rather than inventing an
unrecoverable historical spell-power value. WOS65 adds the retained, ordered
Power Word Shield absorb rows: the Eastbrook melee bridge consumes them
newest-first before applying residual player damage. It is not a general aura
or damage-runtime claim. Smite and Mind Blast reuse the existing WOS56 projectile
row within the WOS65 envelope, releasing after their timed casts and resolving
Holy or Shadow spell resist, range and critical draws at impact only for retained
Eastbrook targets. Mind Blast additionally retains its source eight-second
cooldown from successful cast completion.
Priest Heal also keeps the WOS65 envelope unchanged: its friendly hard cast
uses the retained direct-heal range, spell-power, critical and healing-threat
path without projectile or spell-resist work.
Flash Heal uses the same WOS65-friendly direct-heal path with its independently
generated 1.5-second cast, cost and range; it adds no persisted state or new
engine runtime dependency.
Mind Flay likewise preserves WOS65: its three Shadow channel pulses reuse the
retained projectile row, one range draw and channel Spell Power rider, while its
source zero self-heal fraction emits neither healing nor healing threat.
Lightning Bolt also preserves WOS65: its Shaman rank-specific casts release a
Nature projectile, then use the retained one-resist, range and spell-critical
landing sequence for Eastbrook hostile targets.
Healing Wave preserves WOS65 as a Shaman rank-aware friendly direct heal: it
uses the generated cast and cost, then the retained range, spell-power, critical
and healing-threat sequence without adding a projectile or spell-resist branch.
Earth Shock preserves WOS65 with an instant Nature projectile and the source
shared Earth/Flame/Frost Shock cooldown, followed by retained resist, damage and
combat settlement at impact.
Frost Shock uses the same shared cooldown and projectile path, then applies the
retained Frost direct-hit and eight-second slow motion aura at landing.
Flame Shock uses that shared cooldown with a Fire projectile, then retains its
direct hit plus same-target replacement DoT for four three-second ticks.
Flametongue Weapon uses the existing mutually exclusive imbue row for its
five-minute source-rank bonus, which the retained physical melee swing adds
before physical critical and armor resolution.
Frostbrand Weapon (displayed in source as Rimebound Weapon) follows the same
single-row replacement and physical-swing treatment, with its rank two learned
at level 20 rather than Flametongue's level 18.
Ghost Wolf (displayed in source as Shadewolf) is a two-second, delayed-cost
toggle that retains a 40-percent movement-speed row for up to one hour; it
combines with slows as source minimum-slow times maximum-speed behavior.
Stormstrike (displayed in source as Ancestral Strike) is the retained Shaman
instant melee strike: it spends 40 resource, arms its 12-second cooldown, and
uses the existing deterministic weapon-hit, threat and combat-settlement path
with its source bonus 26.
Shadow Bolt (displayed in source as Gloom Bolt) is the retained Warlock
four-rank Shadow cast: it defers its 25/38/55/80 resource cost until a
successful 1.7/2.2/2.7/3.0-second completion, then reuses the existing
snapshot-safe projectile, resist, damage, threat and combat-settlement path.
Immolate (displayed in source as Burning Pact) is the retained Warlock
direct-plus-DoT Fire cast: its successful two-second completion launches the
source rank profile, and a surviving target receives exactly five three-second
ticks without a second spell-power rider.
Corruption (displayed in source as Blackrot) is the retained Warlock pure-DoT
Shadow cast: its successful two-second completion defers the 35/55/75 resource
bill, then a successful one-draw projectile impact snapshots spell power for
the source rank's six three-second ticks.
Life Tap (displayed in source as Hard Bargain) is the retained Warlock instant
self-conversion: it first owns the hasted spell GCD, then converts 30/55/85
health into capped mana only when the player remains above that health cost.
Curse of Agony (displayed in source as Hex of Anguish) is the retained Warlock
instant Shadow pure DoT: it spends 25/40/60 resource, then its successful
one-draw projectile impact snapshots spell power for eight three-second ticks.
Searing Pain (displayed in source as Sear) is the retained single-rank Warlock
Fire cast: level 14 delays its 35-resource bill until a successful 1.5-second
completion, then keeps the `30-38` source profile in the projectile.
Shadowburn (displayed in source as Duskfire) is the retained single-rank
Warlock instant Shadow cast: a successful 20-yard cast arms its hasted GCD,
bills 70 resource, starts the source 15-second cooldown and launches its
`56-66` direct-damage projectile.
Demon Skin (displayed in source as Fiendhide) is the retained Warlock armor
self-buff: ranks at levels 1/12/20 spend 20/35/50 resource and persist their
30/55/80 armor snapshot for 30 minutes through the existing WOS65 aura row.
Rain of Fire is the retained Warlock ground-targeted Fire channel: `castAt`
decodes finite x/z coordinates, the authority clamps the point to 30 yards,
and the locked center receives four one-second `14-18` radius-seven pulses
after the channel begins. The cast-aim snapshot round-trips with the existing
state and clears when the channel stops.
Conflagrate is the retained Warlock instant Fire projectile: it spends 55
resource and arms its six-second cooldown before launch, then consumes only the
caster's active Immolate at a non-resisted impact. That row is removed before
the `54-64` direct range/critical resolution; missing or foreign rows consume
no damage/critical random draws.
Siphon Life is the retained Warlock instant Shadow pure-DoT projectile: its
non-resisted impact applies 60 damage over 30 seconds, and every tick heals the
living caster for the resolved periodic damage after damage and lethal handling.
Swiftmend is the retained Druid instant friendly Nature spell: it spends 55
resource and starts its eight-second cooldown, consumes the oldest matching HoT
on the target regardless of source, then resolves its `105-125` direct heal.
When no HoT is present it remains paid and on cooldown without consuming RNG.
Wrath (source display: Wildbolt) is the retained Druid hostile Nature hard cast:
its four ranks preserve `13-16 / 24-29 / 38-45 / 60-71` damage, delayed
`20 / 32 / 48 / 70` resource billing and the source two-second rank-2-plus cast
time. Successful completion queues a rank-aware Nature projectile; its impact
resolves one spell-resist draw before the direct range/critical damage path.
Healing Touch (source display: Wildmend) is the retained Druid friendly Nature
hard cast: its four ranks preserve `37-51 / 68-86 / 115-140 / 175-208` healing,
delayed `25 / 45 / 75 / 110` billing and the three-second rank-2-plus cast time.
Successful completion resolves the existing direct-heal range and critical path,
then distributes effective-healing threat without a projectile or resist draw.
Starfire (source display: Skyfall) is the retained Druid hostile Arcane hard
cast: it spends 80 resource only after its three-second cast succeeds, then
queues an `80-112` Arcane projectile whose impact resolves spell resistance
before the direct range/critical damage path.
Entangling Roots (source display: Gripping Roots) is the retained Druid hostile
Nature hard cast: its two ranks spend `35 / 50` resource only after the
1.5-second cast succeeds, then queue a zero-damage spell projectile. A
non-resisted impact roots the mob for 12 seconds; rank two additionally applies
the source `32 / 12 / 3` spell-power-snapshotted DoT. Rooted Eastbrook mobs stop
moving while remaining able to turn and attack when already in melee range.
Maul (source display: Bonecrush) is the retained Bear-only physical queued
strike. Its level-10 and level-16 ranks queue without cost or GCD after source
form admission, then the next mainhand swing bills 15 Rage and applies the
generated `18 / 27` weapon bonus with `35 / 50` flat threat. The existing
queued-swing row survives state round-trip and typed `maul` input toggles that
single pending strike without creating a second melee runtime.
Growl (source display: Menace) is the retained Bear-only physical taunt. It is
free and off-GCD, admits an 8-yard hostile target only while in Bear form, then
uses the existing Taunt settlement to match top threat, force the target for
three seconds and start its 10-second cooldown. Warrior Taunt and Druid Growl
therefore share target settlement without sharing form admission.
Demoralizing Roar (source display: Craven Roar) is the retained Bear-only,
targetless physical shout. Its level-10 and level-16 ranks spend 10 Rage and
start the haste-adjusted 1.5-second GCD, then refresh a 20-second `20 / 35`
Attack Power drain on each living Eastbrook hostile within eight yards. Each
affected mob enters combat with 10 physical threat, and its existing swing path
reads clamped effective AP, so the applied drain changes subsequent mob melee
damage without introducing a generic aura runtime.
Prowl (source display: Stalk) is the retained Cat-only, out-of-combat physical
self-buff toggle. It spends no resource, applies the haste-adjusted minimum
0.75-second GCD, stores the source one-hour `0.5` stealth movement value, and
uses that same persisted state for idle-mob detection. Repeating Prowl while
out of combat clears the toggle; positive damage, another successful form action
or expiry clears both retained fields.
Rake (source display: Flense) is the retained Cat-only stealth opener. It admits
only a live Prowl state, spends 35 Energy, clears Prowl, starts the hasted
1.5-second GCD, and resolves its physical `8 / 12` weapon bonus. A landed
strike awards one combo point; independently, a surviving target receives the
source three-tick `30 / 48` physical bleed through the existing durable DoT
queue. Rake carries no new state-schema or engine backend requirement.
Claw is the retained Cat-only baseline builder. It spends 45 Energy, uses the
same hasted minimum-0.75-second GCD, resolves its source `12 / 20` physical
weapon bonus, and awards one combo point only on a landed strike. It does not
require Prowl and adds no durable state or backend dependency.
Ferocious Bite (source display: Gorebite) is the retained Cat-only combo
finisher. It spends 35 Energy only with at least one combo point, clears Prowl,
uses the same hasted minimum-0.75-second GCD, and resolves source physical
damage as `10 + 14 * combo + [0, 6) + AP / 14` before physical crit and armor.
It clears the consumed combo points after the effect while retaining their
expiry timestamp, with no new durable state or backend dependency.
Native code treats
the envelope as opaque candidate bytes and never reimplements world rules. The
envelope owns the RNG state, draw count and rolling draw digest so failed
transactions cannot advance the random stream independently of committed state.
WOS47 retains WOS45's bounded
opaque Card Duel snapshot after the entity table,
per-entity dead-target metadata, and a weapon-stowed marker. A dead player, a
lootable corpse, or the viewer's dead mob can be selected; other dead corpses
leave the prior target unchanged. `stow_weapon` toggles the marker only for a
living actor, while a successful hostile attack clears it. Its
typed `card_queue_join`, `card_queue_leave`, `play_card {value}` and
`card_forfeit` commands run through the authoritative command sequence, pair on
the fixed tick and retain live hands, deadlines and queue order across reload.
The Card Master range is resolved from the generated source-pinned NPC id rather
than a duplicated map coordinate.
WOS41 retains only motion-relevant active aura rows as canonical numeric
partitions (source ability, source entity, generated kind and positive remaining
seconds). It adds a source-generated active-form code plus saved mana maximum,
which preserves Bear/Cat resource swaps with the existing resource columns. It
also stores an M5 baseline profile code so source-pinned starting-gear inputs
plus persisted selected talent modifiers recompute form-derived final stats
across rollback and reload. WOS42 appends the authoritative fixed6
`spellHaste` final column, which must match that same retained M5-plus-talent
derivation on encode and decode. WOS43 then persists the current M5 catalog's
helmet, feet and mainhand identities as stable index-plus-one codes. The
identity drives same-order stat and weapon recomputation; an over-level item
remains worn but is inert, matching the source. WOS44 adds a per-player M5
inventory partition of dense item-code/count stacks, four pooled bag sockets and
copper. It retains source stack limits, newest-first removal, overflow-preserving
grants and the 16-slot backpack plus bag capacity rule. Inventory instances,
set counts and ordered live-aura identities remain outside the current envelope.
WOS45 keeps the WOS44 byte rows but assigns mainhand code `255` to an explicit
empty hand while `0` continues to mean the source starting weapon. It appends
the seven missing source class-starting identities after the stable WOS44
14-item catalog prefix, then exposes atomic scalar equip/unequip transfers for
the bounded helmet, feet and mainhand projection. WOS47 appends a bounded,
newest-first vendor buyback partition to every entity row and expands the
source-generated scalar catalog to Trader Wilkes' 17 vendor items. Buy, sell,
buyback and sell-all-junk share the existing command transaction and inventory
authority; non-fungible item-instance attributes remain outside this subset.
WOS48 adds no entity bytes: it makes the existing `q_boars` hide count a
derived projection of the primary player's scalar `boar_hide` inventory. An
active or ready quest is recalculated after every modeled inventory mutation,
so removing a hide can return it to active; turn-in removes five real hides.
When WOS47-or-earlier state is read, at most five persisted progress units are
materialized as hides because the old side ledger cannot reconstruct surplus
items. WOS49 adds no entity bytes and routes the existing `discard` packet
through that same scalar inventory authority: omitted count removes one,
explicit count clamps to stock, and discarding a quest hide immediately updates
the WOS48-derived objective. The current pinned scalar catalog has no
instance payload. Generated `noDiscard`, `noVendorSell` and `soulbound` flags
already preserve source policy for scalar entries; instance-preferred removal
still requires the structured transaction boundary.
WOS50 appends a nullable idle-wander target marker and fixed6 X/Z coordinates
after every persisted wander timer. Earlier rows migrate to no target. Offline
Eastbrook bootstrap also starts the persisted authoritative cursor after the
source constructor's non-dummy camp-spawn draws, so the first live idle-wander
draw shares the target simulation stream rather than replaying construction RNG.
The
retained player tick derives control and cast-mobility facts from
those rows, then expires them after cast progression; it does not claim a full
aura lifecycle or introduce a test-only command path.

The M3 built-in-world collision projection now includes the source 16-yard
spatial cell, fixed props and deterministic tree/rock colliders, plus the
80-step `findSafePos` spiral used before camps and fixed NPCs materialize.
`woc_m3_collision_grid_tests.zrp` contains source-pinned prop, decoration and
safe-position vectors. It is a pure ZrVM package and is not dynamic acceptance
evidence until the project backend can execute it.
`camp_spawn_placement.zr` now composes the source scatter, swim-height rule,
dungeon-door clearance, safe-position correction and final ground query for
every built-in camp spawn; entity allocation remains a separate transaction.
`m3_npc_placement_codegen.mjs` pins all 31 source-order static NPC definitions
and identifies the four dynamic system NPCs. `npc_placement.zr` reproduces the
static constructor path through safe-position, final ground height and authored
facing; entity allocation, market/bank indexes and dynamic systems remain with
the eventual transactional world owner.
`m3_npc_initialization_codegen.mjs` separately records all 35 NPC definitions
used by `createNpc`, including names, colors, quest/vendor lists and service
flags. It pins 87 quest references and 107 vendor rows but does not claim a
live interaction, market, bank or dynamic-event implementation.
`m3_ground_object_placement_codegen.mjs` flattens all 18 source definitions and
55 ground-object positions; `ground_object_placement.zr` supplies their final
ground-height projection. Entity allocation, pickup, respawn and quest-credit
state are still transactional world responsibilities.
`m3_mailbox_placement_codegen.mjs` locks the three Ravenpost positions and
`mailbox_placement.zr` adds the source safe-position and final-ground projection.
Mailbox IDs, message storage and collection state remain outside this scalar
world-construction boundary.
`m3_dungeon_entrance_codegen.mjs` supplies all six index-ordered dungeon
definitions, their five overworld entrances and 24 slots each;
`dungeon_entrance_placement.zr` supplies final entrance ground heights. The
separate door-clearance catalog stays in source `DUNGEONS` insertion order as
required by its loop. Entity IDs, slot claims and instance state remain
transactional world responsibilities.
`m3_spirit_healer_placement_codegen.mjs` pins all seven overworld graveyards
and the dynamic Spirit Healer specification. `spirit_healer_placement.zr`
supplies the constructor's direct final-ground positions, while instance
healers, spirit/corpse state and resurrection remain transactional behavior.
`m3_reserved_npc_placement_codegen.mjs` locks Groundskeeper Bram and FURY,
including their reserved entity IDs and post-roster safe positions.
`reserved_npc_placement.zr` intentionally leaves roster insertion, Vale Cup and
PvP interaction behavior to their transactional feature owners.
`bootstrap_roster.zr` composes every source-pinned world-construction catalog
into the no-player sequential range `1..408`, `nextId = 409`, and the two
separate reserved NPC IDs. It is an input to future atomic roster
materialization, not a mutable entity store.

The current target's 165 command ids are inventoried under
`reference/current-head/command_catalog.json`; its companion
`command_payload_catalog.json` records the current client/server source shapes.
The checked-in `protocol/commands.zr`, `woc_protocol` and
`contracts/command_payloads.json` projections now share that current
165-command catalog. Payload contract v34 gives 128 commands a canonical bounded
layout, including `accept {quest,selection?}` and catalog-backed
`applyTalents {alloc}`, `respec {}`, `setSpec {spec?}`, `switchLoadout {index}`,
`deleteLoadout {index}`, `change_skin {catalog,skinIndex}`, catalog-backed
`selectTalentRow {level,optionId?}`, `resurrect_respond {accept}`,
`releaseEmpowered {ability}`, and empty `release`, `resurrect_corpse`,
`resurrect_healer` and `stow_weapon`. It also preserves every current
`IWorldPet` wire shape: abandon/revive/attack/taunt/heal empty commands,
rename/feed/mode UTF-8 identifiers, and auto-taunt/Water-Jet boolean toggles.
It additionally carries the current `IWorldSocialGraph` shapes: friend, block,
ignore and guild member names, guild invitation/leave/disband empty commands,
`guild_event_remove {id}`, and the bounded
`guild_event_create {day,hour?,title,note}` composite payload. It now also carries
four current `IWorldParty` shapes: `setLootMaster {enabled,looter,threshold}`
with a finite JSON-number looter and closed threshold enum, `setMarker {id,marker}`
and `clearMarker {id}` with finite JSON-number fields, and
`readyrespond {ready}` as one canonical boolean. It also carries all six current
`IWorldDuelArena` shapes: finite-number `duel_req {id}`, empty accept/decline/
leave commands, a closed five-format arena queue enum, and `arena_augment
{augment}` with the source 64 UTF-16-code-unit limit. It also carries four
`IWorldTrade` commands: finite-number `trade_req {id}` and empty accept/confirm/
cancel commands. It now also carries all six current `IWorldValeCup` shapes: closed
bracket, nation, role and betting-side enums; a source boolean guild-banner flag;
and a finite JSON-number bet amount. It also carries the sole `IWorldProgressionXp`
send, empty `prestige {}`, three `IWorldMail` finite-number mail-id actions, and all
three `IWorldBank` sends: deposit/withdraw preserve source JSON-number `slot` plus
optional JSON-number `count`, while `bank_buy_slots {}` is empty. It also carries
all nine `IWorldDungeonFinder` sends: closed role/tag arrays,
activity arrays bounded to 16 source strings of at most 64 UTF-16 code units,
empty leave/close/cancel actions, a proposal boolean, and finite listing/applicant ids.
It also carries five finite world-object-id sends: `loot`, `pickup`, `autoloot`,
`delve_interact`, and `collect_delve_chest_loot`.
It additionally carries four exact empty actions: `sell_all_junk`, `market_collect`,
`leave_dungeon`, and `leave_delve`.
The market domain also carries finite listing-id payloads for `market_buy` and
`market_cancel`; listing ownership, price, inventory, balance and settlement remain
simulation responsibilities.
`delve_rite_choose` carries its exact `easy|medium|hard` source enum as one byte;
the Delve run, reliquary state, sequence generation and rewards remain simulation responsibilities.
`trade_offer {items,copper}` and `mail_send` remain
source-shape inventory because their upstream strings/item-id objects have no source-owned
wire bound. The 29 source-only and 8 unmapped dispatch commands remain source-shape
inventory only. The generic protocol envelope can carry bounded opaque bytes for known ids,
but catalog recognition does not make the remaining 37 rows typed, semantically validated,
or implemented.
The two loadout-index payloads reject values outside the source's ten-slot range
at the native/client boundary. WOS38 retains the WOS16 bounded offline loadout
projection (name bytes, six-row allocation and 22 action-bar ability codes), and
its reducer switches or deletes saved rows with the source active-index fallback.
`saveLoadout` has no canonical typed wire layout yet, so this does not claim a
normal command path that creates or updates those persisted rows.
The fixed `change_skin` payload uses `u8_catalog+u8_skin_index`: WOS38 retains
the WOS15 class catalog values `0..7`, while a mech catalog request is a deliberate no-op
until account cosmetic ownership is represented.
`resurrect_respond` uses one exact `u8_false_or_true` value. WOS38 retains the
WOS15
one-offer-per-dead-player state, clears it at the source's 30-second expiry and
revives an accepting player at a living caster's position or the offer-time
fallback. No offline effect dispatcher invokes this primitive yet, so this is an
exercised durable state primitive rather than a claim of a complete combat
resurrection loop. The current-head Temporal Reversal content contract now drives
the first connected `cast` path: in WOS38, the retained WOS16 state verifies the
generated ability id, known ability, resource, GCD/cooldown, dead group/raid target and start range; it locks
the target through the two-second cast and only bills 60 resource, arms 600
seconds of cooldown and creates the 35% offer after completion-time revalidation.
Payload schema v34 preserves both source `castAbility`/`castAbilityOn` forms, the
thirteen current source party command shapes, all current pet command shapes and all 18
current-source social command shapes, all current-source DuelArena command shapes,
four current-source Trade command shapes, all current-source Vale Cup command shapes,
the current-source empty prestige command, three current-source mail-id commands and all
three current-source bank commands, all nine current-source Dungeon Finder commands, five
current-source finite world-object-id commands, four current-source empty actions, two
current-source market-listing-id commands, the closed Delve Rite intensity command, the
closed dungeon-difficulty command and `lootRoll {rollId,choice}` as finite `f64` plus the
exact `need|greed|pass` enum. This only validates transport input; it does not implement
loot eligibility, winner selection or reward mutation. `claim_event_skin {skin}` is likewise
carried as a finite `f64` only; it does not claim event ownership, availability or cosmetic
state mutation.
`cast`
carries a bounded ability id plus an explicit absent or present `u64` target id;
`pmoveRaid` carries a target `u64` and strict subgroup `1|2`; the four additional
party envelopes preserve only client/server JSON transport types, not party
membership, leader authority, marker-range or ready-check state. `masterAssign`
is now a bounded typed payload and routes the source master-loot assignment and
fallback rules. The pet, social and new party entries are transport-only: pets do not route to the separate mutable-array
`instances/pet_state.zr` model, and social commands do not invent an account-
scoped persistence/reducer; WOS38 also does not fabricate loot, marker or
ready-check reducers. `guild_event_create` preserves its ordered day,
nullable hour, title and note fields with per-field bounds, but remains a
transport payload rather than an account calendar implementation. Duel, arena and
Vale Cup entries are transport-only and do not claim invitation, queue, match, Elo,
fiesta, betting or practice state; the prestige entry does not claim max-level
eligibility or progression mutation; mail-id entries do not claim mailbox proximity,
ownership, attachment transfer or deletion state; bank entries do not claim banker proximity,
inventory slots, stack/count validity, capacity, copper or bank mutation; Dungeon Finder entries
do not claim role eligibility, activity selection, queue matching, cooldown, listing ownership,
application state or party formation. World-object-id entries do not claim spatial range,
ownership, loot eligibility, instance state or reward mutation. WOS38 retains the WOS16 party id, leader, raid/subgroup, source
join order and pending party invite columns, and routes `pinvite` through
`punraid` atomically with command sequences. It reproduces the 5/10-player caps,
30-second invite boundary, leader transfer and subgroup normalization. Cross-
social invite exclusion, party events, loot settings, finder formation, generic
ability resolution, effects/events, source haste/global-GCD modifiers and all
non-Temporal cast paths remain unimplemented.
WOS38 also runs the source overworld spirit loop: a dead player records a corpse,
releases to the nearest of seven source-pinned graveyards, can revive within 35
yards of that corpse at 50% health, or at a nearby spirit healer at 20% health
with a source-level resurrection-sickness duration timer. Arena/delve release
routing, the sickness stat aura/resource recomputation, and events still require
their owning systems.
Command-payload schema 38 covers 135 of 165 source commands. It adds typed
`qlinkaccept` transport (`quest` plus numeric sharer pid) and typed `equip` /
`unequip_item` transport over the source's 12 live paperdoll slots. Native client
mapping and authoritative reducers retain linked-quest same-party availability,
optional aimed equipment slots, class/level admission, source-ordered inventory
transfers and stat recomputation. The WOS71 reducer applies its persisted
`mainhand`, `helmet` and `feet` subset; other legal slot codes remain transport-
valid and state-neutral until their paperdoll rows exist. The remaining 22 source-shaped client sends
and eight dispatch-only commands remain a separate migration. Package and native
identities report both the command-catalog SHA-256 and payload-schema SHA-256.
The VM decoder preserves ordered command payload bytes and rejects unknown ids
before world dispatch.
Every native host identity also reports the pinned source commit and contract
schema fingerprint in addition to the 20/60 Hz clocks.
The four empty-payload target selectors have a ZrVM ordering module at
`scripts/woc_game/src/world/target_selection.zr`. It pins the upstream 40-yard
query boundary, flared facing cone, engaged/visible tiers, near-cluster
wrapping, stable friendly ordering and no-candidate behavior. WOS38 now routes
`tab`, `targetNearest`, `tabFriendly` and `targetNearestFriendly` through the
authoritative command batch using live hostile and non-hostile-player rows.
Arena, Vale Cup, Yumi and owned-pet relationships are not yet projected, and
the current source still needs a fresh ZrVM project-backend run.
Native codecs cover commands, events, saves, network envelopes and RL batches,
but validate only framing, bounds, versions and partition offsets; opaque
payload outcomes remain exclusively owned by ZrVM.

Parity traces use the `WTR1` typed binary envelope. ZrVM supplies authoritative
player/entity values and the ordered event-window digest; `woc_parity` only
resolves the generated symbol dictionary, canonicalizes state, computes the
state digest and formats target-compatible JSON. `reference/trace_symbols.json`
pins 965 golden-visible field/value symbols plus 13 hidden-frame-only values and
their wire fingerprint. Existing symbol IDs never shift; generated state/event
digest strings are deliberately not symbols and cannot be substituted for
gameplay results.

The M3 foundation content slice is documented in `contracts/m3-content.md` and
its four source-ordered scenario drivers in `contracts/m3-scenario-drivers.md`.
Seven source-pinned mob templates and their level-derived health, weapon and
armor values are exposed through one scalar table. The M3 test orchestrator
loads that table once before exercising targeting, locomotion, lifecycle,
roster, the 1,530-draw construction boundary and shared-RNG rules in both
interpreter and newly compiled binary modes. All four M3 WTR1 goldens are exact
in both modes; real-M2 double execution remains an open acceptance gate.

The dependency-independent M4 combat source slice is documented in
`contracts/m4-combat.md`. All sixteen M4 scenarios now have ZrVM source
contracts. The last eight-project interpreter/binary matrix passed before the
latest rule-reuse convergence; current source requires a fresh CLI matrix.
`contracts/m4_abilities.json` extracts the 78 retained source abilities used by
WOC runtime closures from the current pinned Git commit; the scenario factories
are a subset of that runtime catalog. Its generator records LF Git-blob
identities and verifies the source definitions against the known LF or CRLF
source-manifest representation; the current manifest records LF, so newline
representation cannot be mistaken for source-content drift.
The same catalog generates scalar Zr projections in
`scripts/woc_game/src/generated/m4_ability_catalog.zr` and
`m4_ability_effects.zr`. They preserve level-selected ranks, rank effect-array
replacement, target/cast flags, threat and channel metadata without crossing a
custom-class or container ABI. `woc_m4_ability_catalog_tests.zrp` is source-only
until a ZrVM CLI run is available.
`combat/flee_speed_state.zr` carries the current head's mob flee scalar: a 1.4x
base speed multiplied by active movement modifiers and then capped at 65% of the
7-yard player run speed. This keeps hasted fleeing mobs catchable while allowing
slows below the cap. Mob AI, pathfinding and WOS movement application remain
separate work; the focused project is source/static only.
`combat/mob_combat_state.zr` preserves the source default, Nythraxis and
Thunzharr combat-profile selection, including scale-derived reach, Thunzharr's
deliberate scale-5 reach cap, and the moving-only closing-range grace. It does
not claim AI pursuit, pathfinding or authoritative swing mutation.
`combat/stun_dr_state.zr` preserves current stun DR classification: Cheap
Shot/Pounce use the opener bucket, six deliberate stuns use controlled, and all
others use random. The buckets are deliberately independent, but their live DR
timer and aura application still belong to canonical combat dispatch.
`m5_class_baseline_stats` separately projects the current source `Sim` stat
chain for every one of the nine classes at levels 1--20 using start weapon/chest
and no talents or auras. The generated Zr module validates its
`64fe1243cd80bdf329dc51a439c803fb83ea0fd3900de45d992ed8de7e51f032` catalog
hash and representative current-source facts, but it is a baseline lookup only;
leveling, equipment, aura and combat mutations remain their transactional
feature owners.
`progression/talent_allocation_commit_state.zr` adds the generic Talents V2
commit projection over the current 27-spec/162-option catalog. Its generated
`talent_allocation_commit_contract` pins the target ordering for whole-allocation
apply (combat/arena lock, validation, equality short-circuit, recompute and
post-commit cleanup), plus the deliberately different prechecks for `setSpec`,
`selectTalentRow`, and `respec`. The module returns explicit recompute, stat,
known-ability, proc, charge, form, offhand, pet, echo, and log obligations to
the WOS owner rather than duplicating entity mutation; its focused `.zrp` remains
source/static until a reliable ZrVM project backend can execute it.
`progression/talent_modifier_state.zr` now projects all 189 current Talents V2
effects: 27 scaled masteries and 162 row options. It preserves the source order
for 18 stat and 39 global fields (including `cheatDeathIcd`'s max, rather than
sum, rule), 51 ability modifiers, grants, and 55 proc identities. The catalog
also exposes all 12 nested `AbilityEffect` DTOs across the current root, slow,
area-root, absorb, dot, extend-dot, interrupt, and consume-dot variants. WOS
must execute those DTOs through its canonical combat dispatch; this projection
does not claim entity mutation or dynamic ZrVM acceptance.
`progression/talent_world_commit_state.zr` composes the allocation and modifier
reducers at the WOS commit boundary. It refreshes modifier DTOs only after a
changed successful allocation, while source-equivalent equality short-circuits
and rejected transactions leave the previous modifier projection intact.
`combat/talent_added_effect_state.zr` is the narrow M4 bridge from those
selected nested DTOs to an appended, already-scaled effect tail for one resolved
ability. It preserves source tail order and the current `applyTalentMods`
scaling rules for area-root, absorb, and DOT direct-percent riders; the other
current DTO fields pass through losslessly. Its focused project proves the
charge, Sacred Ward, and Serpent's Venom contracts at source/static level. The
native effect list remains outside this module, so canonical M4/WOS dispatch
must still concatenate the tail after native effects and apply target/entity
mutation; no dynamic ZrVM acceptance is claimed.
`combat/threat_state.zr` now projects the target's primitive threat table:
stance/form and Holy Righteous Fury multipliers, clamped stealth detection,
ordered accumulation, forced-target cleanup, and stable rounded top-N meters.
It deliberately leaves threat persistence, target switching, heal fan-out, and
entity mutation to the canonical WOS combat owner; its focused project is
source/static until the reliable ZrVM backend is available.
The focused M5 player-trade fixture now shares the current instance ledger with
inventory: transfer removes fungible copies first, keeps full instance payloads
intact, and counts incoming instances as non-merging slots during capacity
preflight. Its preserved-payload and full-bag rejection cases remain source/static
contracts until a ZrVM project session executes them; this is not the production
Runtime10/Plugins08 trade path.
`combat/ability_admission.zr` consumes that projection in the upstream guard
order and owns the state changes for timed/channel starts, tail queuing,
next-swing queuing/consumption, resource timing, cooldowns, forms and seals.
World relationships, range, line-of-sight and facing enter as a read-only target
projection; the module does not duplicate world queries. Its focused project is
also source-only and is not yet connected to WOS13 dispatch.
`combat/casting_state.zr` additionally owns the real 20 Hz cast/channel/queue
state transitions, including source-order GCD retry, cancellation cleanup,
fixed-channel tail flushing and pushback reduction. Its focused project is also
current-source-only until a ZrVM CLI is available.
This does not accept M4: accepted real-M2 exact-golden coverage remains 0/16.

The first M8 Eastbrook Vale asset closure is materialized under `assets/m8`.
`contracts/m8_asset_selection.json` selects all seven distinct player models
used by the nine-class picker, its 26 base/alternate skin textures, wolf/boar
creatures, village and Vale environment assets, terrain/water inputs, MVP
ability/item icons, fonts and core quest/UI sound effects. The generated
`contracts/m8_assets.json` records 93 pinned files totaling 29,611,202 bytes,
including 26 GLBs with 200 animations and 54 glTF rig skins. Generate and
`--check` read all bytes from commit `7c10f280` through Git, so the built project
never depends on the current nested worktree. This is asset/provenance evidence
only; real Zircon import and rendered-frame acceptance remain open.

`contracts/m8_eastbrook_scene.json` now pins the first Eastbrook source scene
contract: exact player/NPC X/Z/facing values, representative wolf/boar camp
centers, the target town building/stall/well/bonfire formulas and both fence
runs. `m8_scene_codegen.mjs` validates every referenced GLB against the checked
asset manifest, applies normalized integer accessor bounds, expands the glTF
node/mesh/material hierarchy and emits current Zircon project references using
`{kind,guid,path_hint,sub}`. Its checked output has 268 scene entities, 199 glTF
node entities, five actors, 18 prop instances and 11 fence modules. The generated
flat ground and static camera/light composition are dependency-independent MVP
authoring evidence only: the project still starts `bootstrap.scene.toml`, target
terrain-height projection is open, and Runtime 04 must import the original
meshopt/WebP GLBs before this scene can be accepted or rendered.

The native WOC adapter also owns a generic committed-presentation timeline.
It accepts one bulk projection per authoritative snapshot, rejects generation,
tick and monotonic-receipt regressions, treats matching duplicates as
idempotent, resets interpolation history across VM generations, and exposes a
clamped previous/current blend for the three default 60 Hz presentation samples
inside each 20 Hz step. Presentation time is injected by the host and never
enters the VM or committed state. The focused Rust integration tests are
authored, but remain source-only until the managed native workspace Cargo gate
runs.

The first concrete bulk payload is `BulkPresentationProjection`. It carries the
viewer and a canonically ordered actor array with full entity generations,
target-derived transform/animation inputs and appearance identity. Validation
rejects duplicate/out-of-order actors, a missing viewer, non-finite transforms
and negative speed. The 60 Hz path linearly merges two ordered snapshots without
allocating a per-frame actor array, interpolates position and the shortest facing
arc, and always takes discrete animation/appearance state from the current 20 Hz
commit. Five focused Rust tests are authored; this still needs the managed Cargo
gate and the real renderer/animation host connection.

`ClientPresentationProjection` adds the discrete MVP HUD payload without moving
gameplay decisions into the native host. Player, target and target-of-target unit
records carry raw meters, cast state and content identities; the canonical action
array carries authority-computed cooldown/usability/range/queue/proc state; the
quest tracker carries acceptance order and objective counts. Validation binds
every unit to the same actor bulk, requires player/viewer identity, rejects an
orphan target-of-target, and pins action plus quest ordering. Key labels, icons,
localized text, ARIA output and collapse/layout settings remain retained-host
responsibilities. Five focused Rust tests are authored but not yet Cargo-run.

The `woc_client` crate now owns an injected-time `ClientFrameDriver`. It advances
authority only when the 50 ms accumulator reaches a 20 Hz boundary, while every
render frame samples the existing presentation timeline. Pending commands are
delivered to the next successful authoritative commit exactly once; an authority
failure retains both commands and accumulated time. A bounded catch-up slice
defers excess ticks without dropping them, and scheduled receipt time is imposed
by the driver rather than accepted from VM output. The pending queue is allocated
once and rejects a 4,097th command using the protocol's 4,096-command tick limit.
Eight focused driver tests pin the 3:1 cadence, command retry, queue bound,
catch-up, interpolation, atomic projection commit and client recovery behavior.
The real ZrVM authority and Zircon render loop are not connected yet.

`TransactionalClientAuthority` now connects that driver to
`WocTransactionalRuntime` without weakening commit semantics. A VM tick returns
the authoritative `WorldSnapshot` and one bulk presentation payload together;
the client decodes and validates an explicit v1, bounded (16 MiB) JSON projection
before state is committed. Unknown schema versions fail before actor/HUD decode.
Runtime-computed presentation digests join state/event digests in
timeline duplicate identity, so the same tick cannot silently carry another
projection. Projection decode failure enters normal client recovery while the
previous state and queued commands remain intact. The fake VM integration proves
both success and invalid-projection recovery paths; the production ZrVM adapter
and engine render loop remain open.

For validated transactional projections, `visit_presented_actors` is the final
host-side sampling seam: it visits interpolated actor transforms without building
a render-frame actor collection and returns a borrowed current HUD. Discrete unit,
action and quest state therefore never blends across authoritative ticks. The
focused driver suite pins the alpha-zero boundary to previous actor pose plus
current HUD; actual Zircon renderer/retained UI submission is still open.

The same client crate now has one platform-neutral `ClientCommandMapper` for the
first authoritative gameplay input set. Keyboard/mouse, gamepad and touch events
carry a diagnostic device label but map through identical generated payload
contracts. Cast-slot, target/clear-target, hostile/friendly tab and attack edges
share one actor identity and monotonic sequence; invalid target ids and sequence
exhaustion do not advance it. The mapper also carries the source Card Duel
queue join/leave, signed card play and live-match forfeit commands through the
same boundary. Tab lookup additionally requires a generated `ClientSend`
descriptor, so the dispatch-only `targetNearest` command cannot be emitted by a
client adapter. Movement and unported command paths remain open; this source
boundary is not an input-system or product-flow acceptance claim.

The pinned client/server source uses a separate movement stream, not a world
command: every 50 ms it sends a positive sequence, seven held/edge flags and an
optional finite facing value; the server applies each valid packet, echoes the
maximum sequence as its acknowledgement and clears held input after 750 ms of
silence. WOC protocol v2 models that boundary in `FixedTickInput` with actor
identity, non-wrapping sequence, the seven flags, `has_facing + f64 facing`, a
65,536-frame bound and canonical per-actor ordering. The Rust relay matches the
apply/ack/facing/stale-clear rules and the ZrVM decoder validates the same byte
shape. `WorldState` consumes a nonempty canonical batch in its WOS candidate,
retains acknowledgement/facing/held input, clears stale input, and advances the
source-ordered player-motion transition. Movement is not an invented command;
this remains source/static evidence until a managed ZrVM project-backend gate
executes the same transaction.

`CharacterRosterModel` is the first DOM-free client-shell state. It accepts
authority/persistence summaries containing character identity, class, level,
appearance, online/rename state and host-injected recent/playtime metrics. The
four pinned sort modes are deterministic, selection survives re-sort and roster
refresh by id, an empty roster routes to character creation, and a missing
selection falls back to the first sorted row. Primary action preserves the target
precedence: forced rename blocks entry, otherwise an online row offers takeover,
otherwise it enters the world. Candidate refresh validates all rows and duplicate
ids before replacing current state. Character creation also has the pinned shape
normalizer: trim edge whitespace without rewriting internal spaces, then require
2-16 ASCII characters with a leading letter and only letters, spaces, hyphens or
apostrophes. Offensive-name,
duplicate-name and realm-uniqueness decisions remain authoritative. Six focused
tests are authored. `OnlineCharacterFlow` now composes that model into the
target shell order without owning network or server rules: roster Back returns
to login, create Back returns to the roster, sort emits the persisted-refresh
effect, and primary entry emits ordinary enter or an explicit two-step takeover
confirmation. Create/forced-rename/delete requests use normalized names and the
shared nine-class skin catalog; online deletion is disabled and permanent
deletion requires a typed case-insensitive name match. Eight additional tests
cover the effect boundary, first-row reselection on authority refresh and stale
confirmation cleanup. Online create Back retains the draft; successful creation
clears only its name while preserving the selected class and appearance, matching
the source DOM lifecycle. The two matching
retained views expose the dynamic roster/preview/details hosts, exact sort and
class orders, catalog-owned skin sockets, responsive `xs`/`md` stacks and
collapsed takeover/delete modals. Actual API execution, localization, preview
rendering and live retained-host binding remain open.

`ModeSelectionModel` reproduces the target's two-step landing console: Online is
the default, opening the menu highlights the current choice, keyboard movement
clamps across Online/Offline, Escape-style close does not commit, and only Play
emits the chosen destination. Online-only hosts reject Offline without needing a
second implementation. Five focused tests cover pointer/keyboard selection,
commit and availability gates. The retained view keeps the source order,
online-player/offline summaries, 66 px Play action, exact community-token
address/copy route and performance tip in a short-viewport scroll owner. The
online effect deliberately leaves restored-session versus login routing to the
host.

`AuthFlow` is the in-memory client boundary for the target login, registration,
two-factor and password-recovery forms. It preserves the 24/128/254/14 field
limits, trims login usernames, keeps passwords only in one-shot host effects,
requires a shape-valid signup email, classifies whitespace-normalized six-digit
codes as TOTP and other trimmed values as recovery codes. Account existence,
password policy, moderation, verification, rate limits and tokens remain server
authority. Opaque reset-request failures intentionally produce the same sent
state as success; only a host-classified rate limit is distinguishable. The two
retained views keep signup/2FA/provider hosts collapsed until their host
capabilities arrive, and a successful authentication/reset clears secret form
state. Eight focused tests are authored; network/Turnstile/native-attestation,
session persistence, localization and live retained-host binding remain open.

`OnlineShellController` composes the existing authentication, realm-directory
and online-character models without taking ownership of services. A host-proven
restored session goes to realm loading, an unauthenticated entry opens login, a
2FA challenge stays in place, accepted authentication loads realms, and a realm
switch requests its character refresh before the roster appears. Its Back paths
preserve realm -> mode selection, roster -> login and create -> roster. Six
focused tests are authored; API calls, session restoration, URL persistence,
character fetches, world entry and retained-host binding remain host work.

`WocShellController` is the final local routing layer for the current shell
models. Online Play emits only a session-probe effect and waits for a host result;
Offline Play opens the fresh session-only picker. Authentication/realm Back
returns to root mode selection, so a child screen cannot remain active after its
target route leaves the online flow. Picker submit prepares a fresh launch,
Welcome Continue requests the one start-world action, and host readiness enters
the world. Seven focused tests cover those transitions; there is still no API,
storage, VM or renderer implementation at this boundary.

`RealmDirectoryModel` now owns the target's DOM-free post-login world list.
It preserves source directory order and exact Normal/PvP/RP/RP-PvP label keys,
auto-selects an exact remembered world, carries per-world character counts, and
maps live status to the pinned Offline/Full/High/Medium/Low thresholds. After a
status batch completes it recommends the first lowest-population online world,
including source-order tie behavior. Selection emits only the exact world name
and base URL for host persistence/API switching. Six focused tests cover
validation atomicity, remembered routing, population boundaries,
recommendation, selection and unknown-row failures. The matching retained view
keeps loading/empty/error states and a dynamic row host whose projection names
the type, character count, status, population and recommendation fields. Real
directory/status requests and `woc_last_realm` platform storage remain M9 host
work.

`OfflineSessionDraft` reproduces the target's separate offline picker instead of
turning it into an online-style profile. It exposes the nine source-ordered class
ids, defaults to Warrior and skin zero, resets skin on every class activation,
validates the launch name, pins world seed `20061`, and derives the exact
`offline:<class>:<name>` preference scope. Offline gameplay state is intentionally
session-only: the target constructs a fresh simulation every launch and persists
only scoped preferences such as keybinds. Launch now rejects a skin index outside
the selected class catalog without mutating the draft. The current draft also
resolves a complete model/thumbnail/material preview request without requiring a name. Seven
focused tests are authored; there is no native save or resume path to drift from
that behavior.

The matching presentation catalog covers all nine classes in picker order while
keeping authoritative combat numbers out of the client. It pins target role,
armor and weapon localization keys, role types, class colors, visual keys, skin
counts and the three curated signature abilities per class. Its companion
appearance catalog maps those nine rows to the seven materialized player GLBs
and exact base/alternate thumbnail sequence, including the shared mage set. Skin
zero keeps the GLB's embedded material while using `base.png` only as its swatch;
later skins apply their alt thumbnail as the material atlas. Seven
focused catalog tests plus one exhaustive preview-resolution test cover the
catalog; all 33 referenced asset paths exist in the checked M8 manifest. The
retained preview binding remains open.

`OfflineShellController` now owns the target's complete offline entry ordering
around that draft: mode selection -> offline picker -> Welcome -> Loading ->
InWorld. Back clears the name field, while reopening the picker resets
Warrior/skin zero; invalid names or catalog-external skins are atomic failures,
and prepared launch identity survives the Welcome/Loading handoff. Continue and
world-ready commits are one-shot, so a double click or duplicate readiness
notification cannot create a second world. Eight focused tests are authored.
This remains a retained-host-independent state contract; the live UI and real
bootstrap-to-Eastbrook transition are still open.

The DOM-free Welcome Screen model mirrors the target's platform and service
gates before retained UI exists. Armory and claimable-chest tiles require online
desktop web, offline Continue never waits for a connection, Discord status fails
open exactly where the source does, and native/touch hosts select the touch hint.
Release marking preserves caller-owned full records, caps the visible list at
five and advances the last-seen id only forward. Its Armory-open handoff is a
session-storage trait contract consumed once and immediately removed. Eight
focused tests are authored; network reads, focus, localized painting, character
stage mounting and actual HUD/store opening remain host work.

The keyboard-preference core pins the target's serialized chord and
scope identities. Modifier order is always Ctrl/Alt/Shift/Meta, held movement
stores only the physical key while edge actions retain their full chord, Escape
is reserved on every layer, and full/compact keycap labels use the source glyphs.
Profile keys remain `woc_keybinds` for the legacy seed,
`woc_keybinds:char:<id>` online, and the exact offline scope produced above.
The complete 61-action registry includes five categories, five Pet bindings and
all 23 action-bar slots. Its two-slot state resolves defaults, evicts non-shared
conflicts across categories, preserves held/edge semantics, and keeps Attack Move
as the only shared-key action. Stored profiles preserve explicit unbinds, ignore
malformed/reserved slots, resolve duplicate claims in registry order, retain
non-conflicting defaults and repair the target's two exact shipped corruption
signatures before loading. JSON decoding, legacy seed selection and a platform-
neutral storage owner are now source-authored. Valid scoped objects win; missing,
corrupt or non-object values seed from the legacy key; successful mutations
serialize all 61 actions; unavailable storage leaves the in-memory change intact.
The host-neutral options model emits exact category/action order and two-slot
labels, conditionally exposes Attack Move, and owns one-shot capture, modifier
waiting, Escape cancellation, normalized confirmation, reset and panel-leave
state. Fifty-one focused tests are authored. Concrete browser/native/mobile storage
adapters plus retained painting, focus and localization remain later slices.

The complete `woc_settings` core lives under the top-level `preferences` owner.
Its registry matches all 43 target numeric ranges and 41 boolean defaults in
source order, covering graphics first-run marking, controller/touch tuning,
audio, interface, accessibility, party frames and HUD comfort settings.
`ClientSettings` owns defaulting, finite-value clamping, independent snapshots,
reset and click-to-move normalization; `StoredClientSettings` mirrors JSON type
filtering, complete 84-field saves, corrupt/missing fallback and unavailable-
storage degradation. Eighteen focused tests are authored, and the source
extraction guard reports numeric `43/43 diff=0` and boolean `41/41 diff=0`.
The host-neutral options projection now adds the exact Esc menu and Graphics,
Audio, Controller and Interface control trees. It carries live values plus real
ranges, steps, formats, choice labels, rerender/commit policy, native/touch gates
and the target's 41-row Interface order (including its repeated Attack Button
row). Nine focused tests are authored; source guards report Interface `41/41`,
Audio `7/7` and Controller `5/5` in exact order. A separate exhaustive routing
catalog now classifies all 84 settings in registry order as persist-only or as
typed input/audio/renderer/gamepad/touch/HUD/style/platform applications. It
preserves multi-owner order for SFX, UI scale, mobile camera joystick and reduced
motion. Stored changes return the normalized value with that static route, and
startup produces all 84 applications in target order. Nine focused routing tests
cover that boundary. The first-run graphics policy also mirrors the target GPU
family ladder without reading FPS: only recognized software/weak GPUs auto-Low,
unknown/mid devices use Medium, evidenced desktops may reach High/Ultra, touch
devices cap at High and native startup clamps saved Ultra/Advanced to High.
Conclusive automatic choices are marked once; inconclusive Medium remains
unmarked for a later probe. The four exact target runtime budgets all retain
60 Hz presentation; mobile selects a lower render-scale floor rather than a
lower frame rate, and the automatic governor remains enabled below Ultra. Thirteen
graphics-policy/storage/budget tests bring the settings total to 49. Live retained
painting, device-hint collection, governor execution, route execution and concrete
platform storage remain later slices.

Gamepad analog input also has a pure source-equivalent boundary. It applies the
same radial deadzone/rescale and unit-circle clamp, maps the left stick through
the target's strict threshold into four movement flags, scales right-stick look
by host-injected frame time with optional Y inversion, and reports only button
rising edges. Eight focused tests are authored. The flags have a defined
authoritative protocol shape and feed the retained fixed-tick movement stream;
they are never encoded as an invented gameplay command. The real ZrVM authority,
platform polling and retained presentation host remain open.

Touch input shares the directional intent shape but keeps the target's mobile
tuning: 0.22 default/configurable joystick deadzone, separate autorun reveal and
lock bands, explicit Auto/Desktop/Touch override, chat long-press and stationary
recenter windows, 0.8 camera-vector scale, and deadzone-adjusted pinch zoom.
Eight focused tests are authored. Pointer routing, haptics, modal cancellation,
device queries and retained mobile widgets still belong to the host layer.

The standard gamepad button contract pins all 17 W3C indices, excludes only the
OS-owned Guide button, and assigns every other button exactly one target default
with slots 0-8 covered once. Brand detection prefers controller names and then
parses the actual Chrome/Firefox vendor field, avoiding product-id collisions.
Xbox, PlayStation, Nintendo and generic labels match the physical silk screen,
including Nintendo face-button swaps. Six focused tests are authored; polling,
focus, mutable bindings and the retained controller panel remain open.

`GamepadBindings` adds the pure mutable half of that panel contract: all defaults,
bindable-only stored overrides, duplicate actions by design, `none` clearing,
Guide/out-of-range rejection, reset and ordered row export.
`StoredGamepadBindings` shares the top-level `PreferenceStorage` contract and
pins the independent `woc_gamepad` JSON object/array loader, JavaScript numeric
property coercion, complete-map save/reset, duplicate actions, target-compatible
clear/reload behavior and unavailable-storage degradation. The controller model
emits the exact 55-option action catalog, 16 W3C-ordered remap rows and
brand-specific physical labels through one mutable/persistent binding contract.
Seventeen binding, storage and controller tests are authored. Concrete platform
storage, polling/focus, settings controls and retained painting remain open.

The project is not playable at the current foundation milestones. A playable claim is
allowed only after the desktop MVP gate in
`docs/plans/woc/01-woc-zrvm-one-to-one-replication.md` passes with the real
runtime, UI, asset and ZrVM foundations.

Reference catalogs under `reference` pin the upstream product surface. They are
development evidence, not runtime dependencies. The built project must not load
source code or assets from `dev/world-of-claudecraft`.
