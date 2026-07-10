# Plugins 03 Physics Change Detection And Command Buffer

## Scope

- Complete Plugins 03 M1-T4 without widening the neutral runtime trait owned by the active Frameworks session.
- Add plugin-local typed body commands for velocity, force, impulse, teleport, and body-type changes.
- Buffer writes made outside FixedUpdate and apply them at the next real physics step.
- Reconcile Jolt bodies from submitted snapshots, skip unchanged bodies, and recreate only when creation-only structure changes.
- Preserve the centralized poison-recovery and module-boundary rules required by the engine structure convention and June review findings.

## Test-First Evidence

Change-detection RED selected the exact `unchanged_bodies_skip_sync` test and exited 101 because `manager::change_detection` did not exist. Command-buffer RED selected the exact integration contract `force_applied_outside_fixed_update_lands_next_step` and exited 101 because `PhysicsBodyCommand` and `DefaultPhysicsManager::queue_body_command` did not exist.

After implementation:

- `unchanged_bodies_skip_sync`: 1/1 passed.
- `force_applied_outside_fixed_update_lands_next_step`: 1/1 passed; queueing did not mutate the scene immediately, and the next FixedUpdate changed velocity and position.
- `jolt_queued_force_lands_on_next_fixed_step`: 1/1 passed with the native backend.
- The poison-recovery test now also poisons the real command-buffer lock, then successfully queues and drains one command.

## Implementation

- `manager/change_detection.rs` compares the previously submitted body/collider/material snapshot with the next scene snapshot. Transform, velocity, and dynamic/kinematic changes become commands; static-boundary, shape, material, filter, sensor, and creation-only property changes recreate the body.
- `manager/command_buffer.rs` owns public `PhysicsBodyCommand` and typed `PhysicsCommandError`. Commands are keyed by `WorldHandle` and `EntityId`, retain submission order, reject non-finite vectors/transforms, and are bounded to 4,096 pending commands per world.
- The capacity is a private module constant because it is one implementation policy with one production owner, rather than a public protocol or cross-module configuration value.
- Builtin drains commands immediately before integration. Force contributes `force * dt / mass`; impulse contributes `impulse / mass`.
- Jolt drains commands only when a real step will run, reconciles entities first, translates entity commands to the current generation-checked backend handle, applies the commands in order, steps, and publishes active-state writeback.
- Backend changes, disabled/unavailable worlds, and explicit world clearing remove stale queued commands.
- The concrete manager API owns this queue for M1-T4. The shared `PhysicsManager` trait was intentionally not edited while its Frameworks owner remains active.

## Full Regression

WSL feature-on:

```bash
CARGO_TARGET_DIR=/tmp/zircon-plugins03-m1t4-full-root \
cargo +nightly test --manifest-path zircon_plugins/Cargo.toml \
  -p zircon_plugin_physics_runtime --features backend-jolt --locked --offline \
  --jobs 1 -- --nocapture --test-threads=1
```

- Library tests: 12/12.
- `physics_manager_runtime_contract`: 34/34.
- Total meaningful tests: 46/46, exit 0; build and execution completed in 17m47s plus test time.

WSL default feature-off:

```bash
CARGO_TARGET_DIR=/tmp/zircon-plugins03-m1t4-full-root \
cargo +nightly test --manifest-path zircon_plugins/Cargo.toml \
  -p zircon_plugin_physics_runtime --locked --offline --jobs 1 \
  -- --nocapture --test-threads=1
```

- Library tests: 10/10.
- `physics_manager_runtime_contract`: 33/33.
- Total meaningful tests: 43/43, exit 0; build and execution completed in 10m14s plus test time.

The first default invocation was rejected as test evidence because a parallel Reflection Probe session changed a plugin manifest after the shared lockfile snapshot; Cargo correctly stopped at `--locked` before compilation. Regenerating the shared plugin lockfile offline synchronized that independent dependency change, a subsequent locked metadata check passed, and the default Physics suite then passed in full.

## Static And Structure Results

- Scoped rustfmt and diff hygiene passed.
- Production manager files contain zero `unwrap`, `expect`, `panic!`, `unreachable!`, or lint-suppression `allow` sites.
- `manager.rs` remains a 49-line façade. The new owners are 103 lines (`change_detection.rs`) and 263 lines (`command_buffer.rs`); `jolt_world.rs` remains 305 lines.
- The queue capacity raw literal appears only in its private owner declaration.
- `tools/audit_plugin_structure.py --json` reports zero manifest, capability, registration, editor/runtime mirror, compatibility-shim, and distribution violations.

## Acceptance Decision

- Accepted for Plugins 03 M1-T4 with status `plugins_03_m1_t4_change_detection_command_buffer_wsl_feature_46_of_46_default_43_of_43_passed`.
- Plugins 03 M1 is complete. M2 shape/body/query work remains open and this record does not claim native query, event, constraint, or ragdoll parity.
