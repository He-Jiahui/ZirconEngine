# Plugins 03 Physics Backend Trait And Builtin

## Scope

- Complete Plugins 03 M1-T1 by replacing the placeholder `backend.rs` with a folder-backed `PhysicsBackend` contract.
- Introduce opaque `u32 index + u32 generation` shape, body, and constraint handles with stale-handle rejection.
- Move the existing builtin step, query/contact, and trigger algorithms under `backend/builtin` without changing manager-visible behavior.
- Keep Jolt unavailable until M1-T3 links a real native backend; preserve the Frameworks `backend-jolt` hard-cut name and no-silent-fallback rule.

## Baseline

- The pre-change `backend.rs` was a 107-line string selector with no trait, backend instance, object ownership, generation handle, command buffer boundary, or typed backend error.
- Builtin rigid-body stepping lived under `manager/builtin_step.rs`; query/contact and trigger implementations lived as root sibling trees outside a backend owner.
- The test-first RED build reached `zircon_plugin_physics_runtime` and failed only because `PhysicsBackend`, `BuiltinPhysicsBackend`, `BodyDesc`, handles, commands, and typed errors did not exist.

## Implementation

- `backend/mod.rs` is a 22-line façade; contracts, typed errors, handle encoding/pool, descriptors/commands, selection, and builtin behavior have separate owner files.
- `handle_pool.rs` advances the generation whenever a slot is destroyed and reused. Commands targeting the stale generation return `PhysicsBackendError::InvalidHandle`.
- `BuiltinPhysicsBackend` implements shape/body ownership, typed command application, fixed-step body updates, active-state reads, ray/overlap/initial-shape-cast queries, and contact/trigger event draining.
- Constraint handles and descriptors are part of the stable trait surface, while builtin constraint creation explicitly returns typed `Unsupported` until Plugins 03 M3.
- The previous manager step/query/contact/trigger implementations moved to `backend/builtin/**`; all old root paths were removed in the same hard cut.

## Test Inventory

- `physics_backend_generation_rejects_destroyed_body_after_slot_reuse`
- `builtin_physics_backend_trait_steps_active_bodies_and_answers_queries`
- `builtin_constraint_gap_is_a_typed_unsupported_error`
- Existing `physics_manager_runtime_contract` integration suite: 32 contact/query/step/backend-selection/world-sync tests.

## Tooling Evidence

RED:

```bash
CARGO_TARGET_DIR=/tmp/zircon-plugins03-m1t1-backend-root \
cargo +nightly test --manifest-path zircon_plugins/Cargo.toml \
  -p zircon_plugin_physics_runtime --lib --locked --offline --jobs 1 \
  backend::tests -- --nocapture --test-threads=1
```

- Exit 101 after 12m27s.
- Two E0432 diagnostics listed only the eight intentionally missing backend contract symbols.

GREEN and regression:

```bash
CARGO_TARGET_DIR=/tmp/zircon-plugins03-m1t1-backend-root \
cargo +nightly test --manifest-path zircon_plugins/Cargo.toml \
  -p zircon_plugin_physics_runtime --lib --locked --offline --jobs 1 \
  backend::tests -- --nocapture --test-threads=1

CARGO_TARGET_DIR=/tmp/zircon-plugins03-m1t1-backend-root \
cargo +nightly test --manifest-path zircon_plugins/Cargo.toml \
  -p zircon_plugin_physics_runtime --test physics_manager_runtime_contract \
  --locked --offline --jobs 1 -- --nocapture --test-threads=1
```

## Results

- Backend contract tests passed 3/3 with exit 0.
- The post-review rerun after replacing the remaining production `expect` with typed error propagation also passed 3/3 in 12m40s with exit 0.
- Existing Physics integration tests passed 32/32 with exit 0.
- Scoped rustfmt and diff hygiene passed.
- Old `backend.rs`, `manager/builtin_step.rs`, root `query_contact{.rs,/}`, and root `trigger{.rs,/}` paths are absent.
- The new façade is 22 lines; the largest new production owner is 387 lines, below the 800-line review threshold.
- No new production `.expect`, `.unwrap`, or `panic!` remains in the backend tree.

## Acceptance Decision

- Accepted for Plugins 03 M1-T1: the backend trait, generation handles, typed failure surface, builtin ownership tree, and behavior-preserving migration are complete.
- This record claims only M1-T1. M1-T2 and M1-T3 are accepted by their own later records; M1-T4 change detection/command-buffer integration remains open.
