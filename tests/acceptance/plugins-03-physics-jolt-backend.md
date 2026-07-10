# Plugins 03 Physics Jolt Native Backend

## Scope

- Complete Plugins 03 M1-T3 by linking `joltc-sys 0.3.1+Jolt-5.0.0` behind the Physics plugin's `backend-jolt` feature.
- Implement JoltC allocator/factory/type registration, collision layers, native world lifetime, Box/Sphere/Capsule shapes, body lifecycle, body commands, fixed stepping, and active-state readback.
- Give each Zircon scene world a persistent native Jolt world and connect it to `physics.step` / `physics.sync_to_scene`.
- Keep feature-off Jolt explicitly unavailable and prohibit silent builtin fallback.
- Do not claim M1-T4 change detection/manager command buffering or later native query, event, and constraint parity.

## Reference And Dependency Evidence

- The selected crate is the cached official `joltc-sys 0.3.1+Jolt-5.0.0` package from `SecondHalfGames/jolt-rust`; it vendors JoltC and Jolt sources and builds them with CMake plus bindgen.
- The bundled JoltC HelloWorld and C headers define allocator/factory/type registration and physics-system lifetime ordering used by `native_world.rs`.
- The repository's `dev/godot/modules/jolt_physics` tree informed the separation of native world, layer filters, conversion, and backend integration owners.
- WSL provided CMake 3.22.1, clang 14, and libclang 14. Windows used the Visual Studio MSVC environment and `E:\Visual Studio\VC\Tools\Llvm\x64\bin\libclang.dll`.

## Test-First Evidence

Backend RED:

```bash
CARGO_TARGET_DIR=/tmp/zircon-plugins03-m1t3-jolt-root \
cargo +nightly test --manifest-path zircon_plugins/Cargo.toml \
  -p zircon_plugin_physics_runtime --features backend-jolt --lib --locked \
  --offline --jobs 1 backend::tests::jolt_contract -- --nocapture --test-threads=1
```

- Exit 101 after the native dependency build.
- The Physics compile failure was limited to the intentionally missing `JoltPhysicsBackend` owner.

Manager RED:

- Two feature-gated manager tests ran and both failed because Jolt still reported no active backend and performed zero steps.
- The run took 20m22s and exited 101; this was the expected missing-integration failure.

## Implementation

- `backend/jolt/layers.rs` owns object and broad-phase layer filters.
- `backend/jolt/conversion.rs` validates and converts neutral transforms, vectors, body types, and the three M1 collider shapes.
- `backend/jolt/native_world.rs` owns native allocator, job system, filters, physics system, and process-global Jolt registration.
- `backend/jolt/runtime.rs` implements `PhysicsBackend` with generation-checked Zircon handles mapped to native Jolt bodies and shapes.
- `manager/jolt_world.rs` owns one persistent backend per scene world, reconciles entity/body records, applies commands, steps, reads active bodies, and publishes the synchronized writeback snapshot.
- `backend/selection.rs` reports Ready only when `backend-jolt` is linked. The manager has separate Builtin/Jolt/Disabled/Unavailable branches; Jolt errors do not invoke builtin stepping.
- `build.rs` links `stdc++` on non-Apple Unix targets because `joltc-sys` does not emit that link directive. MSVC uses the toolchain C++ runtime.

## Verification

WSL feature-on full suite:

```bash
CARGO_TARGET_DIR=/tmp/zircon-plugins03-m1t3-jolt-root \
cargo +nightly test --manifest-path zircon_plugins/Cargo.toml \
  -p zircon_plugin_physics_runtime --features backend-jolt --locked --offline \
  --jobs 1 -- --nocapture --test-threads=1
```

- Library tests: 11/11.
- `physics_manager_runtime_contract`: 32/32.
- Total meaningful tests: 43/43, exit 0, 17m15s.

WSL default feature-off full suite:

```bash
CARGO_TARGET_DIR=/tmp/zircon-plugins03-m1t3-default-root \
cargo +nightly test --manifest-path zircon_plugins/Cargo.toml \
  -p zircon_plugin_physics_runtime --locked --offline --jobs 1 \
  -- --nocapture --test-threads=1
```

- Library tests: 9/9.
- `physics_manager_runtime_contract`: 32/32.
- Total meaningful tests: 41/41, exit 0, 13m08s.
- The default build did not compile JoltC.

Windows MSVC focused native suite:

- The first native build compiled vendored Jolt/JoltC successfully, then bindgen failed because it could not discover `libclang.dll`. This tool-discovery run was rejected as passing evidence.
- The rerun imported the Visual Studio environment, set `LIBCLANG_PATH=E:\Visual Studio\VC\Tools\Llvm\x64\bin`, and passed both native backend tests 2/2 with exit 0 in 176 seconds.

## Static And Structure Results

- Scoped rustfmt and `git diff --check` passed.
- Production `backend/jolt/**` contains zero `unwrap`, `expect`, `panic!`, `unreachable!`, or lint-suppression `allow` sites.
- `tools/audit_plugin_structure.py --json` reported zero manifest, capability, registration, and distribution violations.
- The plan-output audit reported 22 pre-existing issues owned by other plan families and zero Plugins 03 issues; they were not modified by this slice.

## Acceptance Decision

- Accepted for Plugins 03 M1-T3 with status `plugins_03_m1_t3_joltc_native_backend_wsl_feature_43_of_43_default_41_of_41_windows_2_of_2_passed`.
- Plugins 03 M1 remains open only for M1-T4 change detection and command-buffer integration.
- Native query/event/constraint parity remains assigned to M2 through M4 and is not implied by this acceptance record.
