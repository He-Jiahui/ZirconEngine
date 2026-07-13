# Plugins 04 M1 Dense Evaluation Acceptance

Date: 2026-07-11
Plan: `docs/plans/zircon_plugins/04-animation.md`
Status: M1-T3 partial acceptance; final pose ownership remains coordinated with Runtime.

## Accepted implementation

- `animation.evaluate` is registered at `PostUpdate` and routes through the folder-backed five-phase evaluation pipeline.
- The obsolete `scene_hook/**` tree and path-attribute wiring are absent.
- Clip targets/channels, graph topology/parameters/masks, and state/transition conditions compile into dense runtime representations.
- Clip, graph, state-machine, and diagnostic caches are bounded and revision-aware.
- Production evaluation has no `panic!`, `expect`, `unwrap`, `unreachable!`, or unused/dead-code allowance paths.

## Fresh evidence

- `animation_compiled_graph_contract`: 2 passed, 0 failed.
- `animation_compiled_state_machine_contract`: 2 passed, 0 failed.
- `runtime_physics_animation_tick_contract`: 20 passed, 0 failed.
- Locked/offline WSL Animation package `cargo check --tests`: exit 0.
- Scoped `cargo fmt --check` and `git diff --check`: exit 0.

## Open boundary

`LevelSystem::record_animation_poses` takes ownership of the final `BTreeMap<EntityId, AnimationPoseOutput>` and only exposes cloning reads. The active Runtime architecture session owns that API. M1-T3 must not be marked complete until a coordinated reusable/take-replace pose handoff exists and the full M1 testing stage is rerun.
