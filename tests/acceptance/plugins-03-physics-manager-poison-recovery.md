# Plugins 03 Physics Manager Poison Recovery

## Scope

- Harden `zircon_plugins::physics::runtime::manager` shared state for the engine structure convention E9 rule.
- Keep existing public physics manager APIs and simulation/query behavior unchanged.
- Centralize mutex poison recovery for settings, core attachment, fixed-step accumulators, synchronized worlds, contacts, trigger pairs, and trigger events.

## Baseline

- The indexed pre-change source contained 18 production `.lock().expect("... mutex poisoned")` sites under `zircon_plugins/physics/runtime/src/manager`.
- A worker panic while holding any of those locks made later infallible runtime ticks or queries panic instead of recovering the retained inner state.
- The worktree contains unrelated active Frameworks, rendering, editor, and animation changes; this slice does not edit their owners.

## Test Inventory

- `physics_manager_settings_and_clock_recover_poisoned_state_locks`
  - poisons the real settings mutex;
  - stores valid settings through the production API;
  - poisons the real accumulator mutex;
  - verifies fixed-step planning remains usable.
- `physics_manager_world_state_recovers_poisoned_state_locks`
  - poisons synchronized-world, contact, trigger-pair, and trigger maps;
  - synchronizes a real world snapshot through `PhysicsManager`;
  - verifies snapshot lookup and both event drains remain usable.
- Static failure-path scan requires zero production `mutex poisoned` panic sites below the manager owner.
- Existing Physics runtime contract tests remain the parent integration suite for unchanged simulation, contact, trigger, and query behavior.

## Tooling Evidence

- Windows formatter: `rustfmt --edition 2021 --check` over the eight changed manager source/test files.
- Static baseline/current scan: `git grep --cached` reported 18 indexed production poison-panic sites; current `git grep` reported 0.
- WSL toolchain: `rustc 1.94.1 (e408947bf 2026-03-25)` and `cargo 1.94.1 (29ea6fb6a 2026-03-24)`.
- Focused Linux type command:

```bash
PKG_CONFIG_PATH=/tmp/zircon-deps/libudev/usr/lib/x86_64-linux-gnu/pkgconfig \
LIBRARY_PATH=/tmp/zircon-deps/libudev/usr/lib/x86_64-linux-gnu \
CARGO_TARGET_DIR=/tmp/zircon-plugins03-physics-lock-recovery \
cargo +nightly check --manifest-path zircon_plugins/Cargo.toml \
  -p zircon_plugin_physics_runtime --lib \
  --features zircon_runtime/target-server --locked --jobs 1
```

- Focused Linux test command:

```bash
PKG_CONFIG_PATH=/tmp/zircon-deps/libudev/usr/lib/x86_64-linux-gnu/pkgconfig \
LIBRARY_PATH=/tmp/zircon-deps/libudev/usr/lib/x86_64-linux-gnu \
CARGO_TARGET_DIR=/tmp/zircon-plugins03-physics-lock-recovery \
cargo +nightly test --manifest-path zircon_plugins/Cargo.toml \
  -p zircon_plugin_physics_runtime --lib \
  --features zircon_runtime/target-server --locked --jobs 1 \
  physics_manager_ -- --nocapture --test-threads=1
```

## Results

- Scoped formatting passed.
- Scoped diff hygiene passed with only the repository's line-ending warnings.
- Production poison-panic scan passed at 0 current sites, down from the indexed baseline of 18.
- The no-feature plugin check reproduced an external lower-layer E0432 before Physics compiled: `asset/runtime_asset_path.rs` imports `crate::diagnostic_log`, while the Plugins workspace disables runtime default features and the module is gated by `diagnostic-log`.
- The focused `zircon_runtime/target-server` Linux check passed in 6m33s. It reached and checked `zircon_plugin_physics_runtime` successfully.
- The focused manager test command passed 2/2 in 9m30s. The worker-thread panic messages are intentional poison setup; the test process exited 0 with no failed tests.
- After removing two pre-existing unused imports from the touched `world_sync.rs`, scoped rustfmt/diff checks passed. A dependency-drift rerun was deliberately terminated and rejected when the active Frameworks session began writing to the same target directory; its result is not used as evidence.
- The full plan-output audit currently reports 24 violations outside Plugins 03, owned by active index, Editor UI, Frameworks, and Render sessions. This plan row is not among the findings.

## Acceptance Decision

- Accepted for this E9 hardening slice: the affected Physics package type-checks under the current server feature contract and both real poisoned-state boundaries pass.
- Plugins 03 M1 remains open; this hardening slice does not claim the backend trait, Jolt, system-anchor split, or change-detection tasks are complete.
