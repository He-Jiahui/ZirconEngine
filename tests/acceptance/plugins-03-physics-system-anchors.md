# Plugins 03 Physics System Anchors

## Scope

- Complete Plugins 03 M1-T2 by replacing the single registration entry with `register_runtime_systems`.
- Register `physics.step` in FixedUpdate and `physics.sync_to_scene` in FixedPostUpdate, both in `physics.main`.
- Make the FixedPostUpdate system perform real scene writeback from the manager's synchronized active-body snapshot.
- Keep Jolt linkage and change-detection/command-buffer work outside this M1-T2 record; their status is tracked by later milestone records.

## Test-First Evidence

The first command used an over-specific `--exact` filter and selected zero tests. It was rejected and is not acceptance evidence.

The corrected RED command selected `physics_plugin_registers_runtime_system_through_sdk_builder`; it ran one test and failed on the missing `physics.sync_to_scene` system assertion.

## Implementation

- `runtime_system.rs` owns the two IDs, the shared `physics.main` set, plural registration, fixed-step execution, and fixed-post-step writeback.
- `plugin.rs`, `lib.rs`, the package descriptor, and `plugin.toml` expose the same two anchors with no legacy singular registration alias.
- `manager/world_sync.rs` applies synchronized transforms and rigid-body type, mass, velocities, damping, gravity, sleeping state, and axis locks to existing scene bodies.
- `manager/tests.rs` exercises writeback against a real `World`; `src/tests.rs` locks both stages, the shared set, and the descriptor anchor list.

## Verification

```bash
CARGO_TARGET_DIR=/tmp/zircon-plugins03-m1t1-backend-root \
cargo +nightly test --manifest-path zircon_plugins/Cargo.toml \
  -p zircon_plugin_physics_runtime --lib --locked --offline --jobs 1 \
  physics_ -- --nocapture --test-threads=1

CARGO_TARGET_DIR=/tmp/zircon-plugins03-m1t1-backend-root \
cargo +nightly test --manifest-path zircon_plugins/Cargo.toml \
  -p zircon_plugin_physics_runtime --test physics_manager_runtime_contract \
  --locked --offline --jobs 1 -- --nocapture --test-threads=1
```

## Results

- Physics-filtered library tests passed 8/8 with exit 0 in 11m59s.
- `physics_manager_runtime_contract` passed 32/32 with exit 0 in 12m32s.
- The registration contract proves `physics.step` is FixedUpdate, `physics.sync_to_scene` is FixedPostUpdate, and both use `physics.main`.
- The manager contract proves synchronized body data mutates real scene transform and rigid-body state.
- Scoped formatting, diff hygiene, and legacy-name scans passed.

## Acceptance Decision

- Accepted for Plugins 03 M1-T2 with status `plugins_03_m1_t2_dual_physics_system_anchors_wsl_40_of_40_passed`.
- This record claims only M1-T2. M1-T3 is accepted by its later native-backend record; M1-T4 remains open.
