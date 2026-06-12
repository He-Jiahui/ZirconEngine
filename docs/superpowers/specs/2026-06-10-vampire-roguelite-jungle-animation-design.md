# Vampire Roguelite Jungle And Animation Design

## Goal

Build `examples/vampire` into a playable third-person vampire roguelite that runs in `zircon_runtime`, then use it to drive the missing runtime capabilities for HUD output, gameplay spawning, jungle navigation content, and skeletal animation playback.

## Approved Direction

The work is split into two milestones.

1. Gameplay and content first: make the example visibly playable with WASD movement, random enemy spawning around the player, automatic weapons, XP, drops, upgrades, boss timing, HUD text, and a denser jungle terrain/navigation scene.
2. Animation second: replace the current scale-based action cues with real animation state-machine and skeletal playback support that can drive imported character clips.

## Current Evidence

The current example already has player movement, camera follow, automatic Blood Bolt damage, enemy chase/attack behavior, imported GLB models, a flat `main.navmesh.toml`, and screen-space UI rendering in the renderer. The gaps are:

- gameplay script has no spawn/drop/XP/upgrade/boss loop;
- `damage_entity` removes dead enemies without exposing a death event to script;
- script host can only spawn empty entities, so dynamic enemies/pickups cannot receive visible model components yet;
- runtime frame submission currently uses world render extract without project HUD extract;
- navigation content is a flat square, not a jungle terrain walkable mesh with obstacle-aware topology;
- glTF animation import still records animation placeholder data instead of clip channels, and scene animation player components are not yet sampled into the skinning path.

## Runtime Architecture

Gameplay mechanics remain owned by the project script in `examples/vampire/scripts/vampire_game/main.zr`. Shared runtime support belongs in `zircon_runtime`:

- `script::vm::gameplay_host` exposes reusable gameplay functions for dynamic visible entity spawning, death-aware damage, component JSON, and HUD state.
- `dynamic_api::session` converts project HUD state into `UiRenderExtract` and submits render frames with UI through the existing render framework surface.
- `scene` remains the runtime authority for entities, transforms, renderable components, and dynamic components.
- animation playback lands as runtime scene/asset behavior, not as vampire-specific script code.

This follows the fixed package roles: `zircon_app` hosts the process, `zircon_runtime` owns runtime state and rendering, and project scripts consume host capabilities.

## Gameplay Design

Player state:

- starts at 120 HP, level 1, 0 XP, 5.2 movement speed, 1.0 attack/attack-rate multipliers, no shield;
- gains one level when XP reaches `8 + level * 4 + floor(level^1.35 * 3)`;
- level-up pauses upgrade selection in the HUD until the player presses `1`, `2`, or `3`;
- upgrade choices are max HP, attack multiplier, or movement speed.

Enemy spawning:

- a spawn manager entity ticks every frame;
- next spawn interval is deterministic pseudo-random between 1.2 and 3.4 seconds at the start and ramps down toward 0.45 seconds;
- enemies spawn in an annulus around the player, outside close view but inside the jungle playfield;
- alive count is capped to prevent runaway entity growth;
- a boss spawns every 300 seconds with much higher HP, larger scale, higher XP, and a distinct dynamic component flag.

Drops:

- dead enemies grant XP directly and may also spawn pickups near their death position;
- XP shard is common, heal/attack buff/attack speed buff/shield are low probability;
- chest is rare and opens on pickup into a three-choice weapon selection;
- pickups despawn when collected.

Weapons:

- Blood Bolt remains the default targeted attack;
- Orbit Blade damages nearby enemies periodically while visually represented by orbiting markers;
- Lance fires straight-line directional attacks at intervals;
- Pulse Curse performs intermittent area damage around the player.

HUD:

- top-left text shows level, XP, HP, shield, active weapons, and buff timers;
- upgrade prompt shows three numbered choices;
- chest weapon prompt shows three numbered weapon options;
- boss timer/status is visible when a boss is active or about to spawn.

## Jungle Scene And Navigation Design

The scene moves from a graveyard arena to a jungle clearing:

- terrain becomes a multi-tile low-poly terrain grid with height variation, swamp/stone/grass material accents, and visible path corridors;
- walkable navigation polygons cover the terrain corridors and exclude decorative obstacle clusters;
- tree trunks, rocks, roots, ruins, vines, mushrooms, lanterns, and fog lights make the first viewport visibly dense;
- the navmesh remains authored as a `.navmesh.toml` asset until a full editor bake tool exists, but it must correspond to the terrain corridors and obstacle layout instead of a single flat square.

## Animation Design

Animation work follows the gameplay milestone because it changes shared runtime foundations:

- import glTF animation channels into `AnimationClipAsset` with translation/rotation/scale tracks;
- sample clips into joint/local transforms each scene tick;
- evaluate `AnimationStateMachinePlayerComponent` parameters such as `speed`, `attacking`, `dead`, and `hit`;
- write joint palettes into the existing skinned mesh render path;
- vampire example uses idle/run/attack states and enemy chase/attack/death states.

## Acceptance Evidence

Gameplay/content milestone is accepted when tests and a runtime screenshot prove:

- enemies spawn around the player over time;
- enemies can die and award XP;
- level HUD changes and upgrade choices appear;
- buff state appears and expires;
- pickups and chest weapon choices can be generated and collected;
- boss state is represented in data and can be forced in tests;
- jungle terrain/navmesh/decorations are visible and importable.

Animation milestone is accepted when tests and runtime evidence prove:

- a glTF animation fixture imports non-empty tracks;
- animation state machine changes active state from movement/action parameters;
- sampled joint transforms affect a skinned render packet or palette path;
- vampire actors no longer rely only on scale pulses for action feedback.
