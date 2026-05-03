# Particles GPU Readback Mailbox

## Scope
- Changed the neutral particle GPU readback mailbox seam for M6 particles GPU follow-up.
- Affected layers: plugin-owned particle GPU readback conversion, runtime neutral render DTOs, scene-renderer advanced plugin output storage, and particles runtime/editor validation.

## Baseline
- The particles GPU backend already decoded counter words into `ParticleGpuCounterReadback` and projected them into `RenderParticleGpuReadbackOutputs`.
- The scene-renderer neutral plugin output mailbox stored `RenderPluginRendererOutputs`, but only VG/HGI had explicit take/accessor coverage.
- Current workspace baseline is dirty and actively changing. During validation, unrelated physics/animation runtime files were transiently deleted or rewired, which blocked a fresh `zircon_runtime --lib` filtered test rerun.

## Test Inventory
- Unit or focused subsystem cases: particle readback DTO empty detection, particle mailbox storage and take semantics, neutral readback collection preserving particle payloads, and particle GPU counter/CPU parity conversion.
- Integration or project cases: `zircon_plugin_particles_runtime` package tests and `zircon_plugin_particles_editor` package tests.
- Boundary cases: default empty readback payload, non-default alive count, non-default indirect args, per-emitter spawned counts, and taking particle outputs without clearing VG outputs.
- Negative cases: current runtime rerun blocked by unrelated physics/animation module churn rather than particle readback code.

## Tooling Evidence
- Tool name and version: Windows PowerShell, Cargo/Rust toolchain from the local workspace, Python `tomllib`, `rustfmt`, and `git diff --check`.
- Why these tools were used: scoped milestone validation needed formatting checks, TOML template parsing, whitespace checks, focused runtime readback tests, and plugin package regressions without claiming workspace-wide green.
- Exact commands:
  - `Get-PSDrive -Name E`
  - `cargo fmt -p zircon_runtime --check`
  - `cargo fmt --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_particles_runtime -p zircon_plugin_particles_editor --check`
  - `python -c "import pathlib, tomllib; paths=['zircon_plugins/particles/templates/cpu_sprite_system.toml','zircon_plugins/particles/editor/authoring.ui.toml','zircon_plugins/particles/editor/preview.ui.toml','zircon_plugins/particles/editor/particle_system.drawer.ui.toml']; [tomllib.loads(pathlib.Path(p).read_text(encoding='utf-8')) for p in paths]; print('parsed', len(paths), 'particle TOML templates')"`
  - `rustfmt --edition 2021 --check "zircon_runtime/src/core/framework/render/plugin_renderer_outputs.rs" "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/output_access.rs" "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/output_storage.rs" "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_readbacks/collect_into_outputs.rs" "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/mod.rs" "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/take_last_particle_gpu_readback_outputs.rs"`
  - `git diff --check -- "zircon_plugins/particles" "zircon_runtime/src/core/framework/render/plugin_renderer_outputs.rs" "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/output_access.rs" "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/output_storage.rs" "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_readbacks/collect_into_outputs.rs" "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/mod.rs" "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/take_last_particle_gpu_readback_outputs.rs" "docs/zircon_plugins/particles/runtime.md" ".codex/sessions/20260503-1529-particles-gpu-renderer-closeout.md"`
  - `$env:CARGO_INCREMENTAL="0"; cargo test -p zircon_runtime --lib plugin_renderer_outputs --locked --offline --jobs 1 --target-dir "E:\cargo-targets\zircon-particles-editor-test-nonincr" --message-format short --color never -- --nocapture`
  - `$env:CARGO_INCREMENTAL="0"; cargo test --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_particles_runtime --locked --offline --jobs 1 --target-dir "E:\cargo-targets\zircon-particles-editor-test-nonincr" --message-format short --color never`
  - `$env:CARGO_INCREMENTAL="0"; cargo test --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_particles_editor --locked --offline --jobs 1 --target-dir "E:\cargo-targets\zircon-particles-editor-test-nonincr" --message-format short --color never`

## Results
- Passed checks:
  - `Get-PSDrive -Name E` reported 170.43 GB free on the target drive before Cargo validation.
  - Plugin `cargo fmt --check` passed for `zircon_plugin_particles_runtime` and `zircon_plugin_particles_editor`.
  - Python parsed all 4 particle TOML templates.
  - `rustfmt --edition 2021 --check` passed for the touched runtime readback files.
  - Scoped `git diff --check` reported no whitespace errors; it only emitted existing LF-to-CRLF warnings.
  - One warmed `zircon_runtime --lib plugin_renderer_outputs` run reached the filtered tests and passed 5 tests, including the new particle readback DTO, storage, and collection cases.
  - `zircon_plugin_particles_runtime` passed 18 tests plus 0 doctests with `--locked --offline`.
  - `zircon_plugin_particles_editor` passed 1 test plus 0 doctests with `--locked --offline` after an initial compile-time timeout was rerun on the warmed target.
- Failed or blocked checks:
  - `cargo fmt -p zircon_runtime --check` failed on unrelated `zircon_runtime/src/plugin/scene_hook/mod.rs` formatting drift.
  - The first `zircon_runtime --lib plugin_renderer_outputs` run timed out during dependency compilation and is not acceptance evidence.
  - A later fresh `zircon_runtime --lib plugin_renderer_outputs` rerun failed before executing readback tests because active physics/animation work left `crate::animation` and `crate::physics` unresolved in `zircon_runtime` test compilation.
- Fixes made in response:
  - Local formatting issues in the touched runtime files were corrected manually and rechecked with scoped `rustfmt`.
  - New particle-only accessors were annotated where they are intentionally host-visible before a runtime feedback consumer exists, preventing this slice from adding dead-code warnings to plugin package validation.

## Acceptance Decision
- Accepted for particle plugin package behavior, editor authoring package behavior, TOML parsing, scoped formatting, scoped whitespace, and the neutral readback mailbox implementation files.
- Blocked for current `zircon_runtime --lib` acceptance because the latest fresh runtime filtered test fails in unrelated active physics/animation module churn before particle readback tests can execute.
- Remaining risks: the built-in renderer still does not automatically execute particle GPU transparent rendering from `alive_indices` and `indirect_draw_args`, `RenderPassExecutionContext` remains metadata-only, and no particles runtime feedback provider consumes `RenderParticleGpuReadbackOutputs` yet.
