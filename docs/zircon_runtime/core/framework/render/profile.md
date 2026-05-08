---
related_code:
  - zircon_runtime/src/core/framework/render/profile.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/tests.rs
  - zircon_app/src/entry/entry_config.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - dev/bevy/Cargo.toml
  - dev/bevy/docs/cargo_features.md
implementation_files:
  - zircon_runtime/src/core/framework/render/profile.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/tests.rs
  - zircon_app/src/entry/entry_config.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
plan_sources:
  - user: 2026-05-08 continue ZirconEngine Bevy-Level Rendering Completion Plan M1
  - .codex/plans/ZirconEngine Bevy-Level Rendering Completion Plan.md
  - docs/assets-and-rendering/bevy-rendering-capability-matrix.md
  - dev/bevy/Cargo.toml
  - dev/bevy/docs/cargo_features.md
tests:
  - zircon_runtime/src/core/framework/tests.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - cargo test -p zircon_runtime render_profile --locked
  - cargo check -p zircon_app --locked --all-targets
doc_type: module-detail
---

# Runtime Render Profile Contracts

## Purpose

`zircon_runtime::core::framework::render::profile` owns the neutral product-profile vocabulary for Bevy-level rendering. It turns Bevy's Cargo feature profiles and collections into runtime-selectable Zircon product bundles instead of copying Bevy's compile-time feature model.

The contract is intentionally data-only: it defines product names, feature dependencies, backend capability gates, validation errors, and the config key used by `zircon_app` to store the active bundle. Concrete renderer execution stays in `zircon_runtime::graphics`, `zircon_runtime::rhi`, `zircon_runtime::rhi_wgpu`, and `zircon_runtime::render_graph`.

## Bevy Evidence

Bevy profiles and collections drive the shape:

- `dev/bevy/Cargo.toml:133-151` defines `default`, `2d`, `3d`, and `ui` profiles.
- `dev/bevy/Cargo.toml:198-261` defines `common_api`, `2d_api`, `2d_bevy_render`, `3d_api`, `3d_bevy_render`, `ui_api`, and `ui_bevy_render` collections.
- `dev/bevy/docs/cargo_features.md:10-52` explains that profiles are high-level product groups and collections compose those profiles.

Zircon deliberately diverges by making these runtime product choices rather than Cargo features, because the app host already selects `EntryProfile`, `RuntimeTargetMode`, plugin manifests, and export profiles at runtime.

## Data Model

`RenderProductProfile` currently names these product choices:

- `Headless`
- `CommonRenderApi`
- `Render2d`
- `Render3d`
- `Ui`
- `DefaultRender`
- `AdvancedRender`
- `SolariExperimental`

`RenderProductFeature` names the neutral capabilities that a profile bundle can require, such as camera, image, mesh, material, shader, sprite, light, PBR, core pipeline, UI render, render target, post-process, anti-aliasing, Virtual Geometry, Hybrid GI, and Solari.

`RenderProfileBundle` stores the selected profile, implied product profiles, and enabled product features. Its constructors are the canonical bundle definitions:

- `headless()` activates no render products.
- `render_2d()` includes camera/image/mesh/material/shader/sprite/core pipeline and post-process.
- `render_3d()` includes camera/image/mesh/material/shader/light/PBR/core pipeline/post-process/AA.
- `ui()` includes UI render/core pipeline/render target plus common API features.
- `default_render()` includes `CommonRenderApi`, `Render2d`, `Render3d`, and `Ui` without advanced features.
- `advanced_render()` adds Virtual Geometry and Hybrid GI on top of default rendering.
- `solari_experimental()` adds Solari capability requirements on top of advanced rendering.

## Validation

`RenderProfileBundle::validate()` checks product dependencies before a bundle is accepted. The M1 hard requirements are:

- `Render2d` requires camera, image, mesh, material, shader, sprite, and core pipeline.
- `Render3d` requires camera, image, mesh, material, shader, light, PBR, core pipeline, post-process, and AA.
- `Ui` requires UI render, core pipeline, and render target.

`validate_capabilities(...)` additionally maps product features to backend capability gates through existing neutral `RenderCapabilityKind` values. `VirtualGeometry`, `HybridGlobalIllumination`, and `Solari` remain explicit opt-in paths and are not part of `DefaultRender`.

## App Wiring

`EntryConfig` now carries a `RenderProfileBundle`. Runtime and editor profiles default to `RenderProfileBundle::default_render()`, while headless defaults to `RenderProfileBundle::headless()`. The app host stores the chosen bundle in `CoreRuntime` config under `RENDER_PROFILE_CONFIG_KEY` before module activation so runtime modules and later plugin groups can query the active render product surface without coupling to `zircon_app` internals.

## Test Coverage

`zircon_runtime/src/core/framework/tests.rs` covers bundle defaults, dependency rejection for 2D/3D/UI, and advanced capability rejection when backend capability summaries do not satisfy the bundle.

`zircon_app/src/entry/tests/profile_bootstrap.rs` covers app bootstrap storage for default runtime rendering and explicit headless bundle selection.

Fresh M1 validation evidence from 2026-05-08:

- `cargo test -p zircon_runtime render_profile --locked`: passed with `5 passed; 0 failed; 1047 filtered out` for render-profile tests; integration binaries with zero matching tests also completed without failures.
- `cargo check -p zircon_app --locked --all-targets`: passed with warning-only output after lower shared compile blockers in asset animation export/fallback, UI surface pooling serialization, ECS query marker lifetimes, and the active state foundation settled.

Milestone testing commands are the M1 gates from the Bevy-level rendering plan: `cargo test -p zircon_runtime render_profile --locked` and `cargo check -p zircon_app --locked --all-targets`.
