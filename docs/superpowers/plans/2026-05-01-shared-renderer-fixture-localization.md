# Shared Renderer Fixture Localization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task from the existing `main` checkout. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the moved GI/VG renderer test-source dependency on runtime-local `plugin_render_feature_fixtures` with plugin-owned fixture modules that source descriptors from each plugin crate.

**Architecture:** Each plugin runtime crate gets a `#[cfg(test)]` `test_support` module at crate root. The support module exposes only plugin-local fixture helpers and never recreates old `zircon_runtime::graphics::tests` paths. Existing unwired test sources are updated to import the plugin fixture path directly, while renderer/pass module wiring remains deferred until stale private renderer scopes are cut over.

**Tech Stack:** Rust, Cargo, `zircon_plugins` workspace, `zircon_runtime` neutral graphics/runtime DTOs, plugin render feature descriptors.

---

## Scope

- Work in the current `main` checkout; do not create a branch or worktree.
- Do not commit automatically.
- Do not restore `zircon_runtime::graphics::runtime::{hybrid_gi,virtual_geometry}`.
- Do not add compatibility modules, old-path re-exports, facades, or shims.
- Keep runtime as neutral DTO/base-render owner; plugin fixtures call plugin crate APIs directly.
- Do not wire broad `renderer` modules in this slice.

## File Structure

- Create `zircon_plugins/virtual_geometry/runtime/src/test_support/mod.rs`: structural test-support root for the VG plugin crate.
- Create `zircon_plugins/virtual_geometry/runtime/src/test_support/render_feature_fixtures.rs`: VG descriptor/framework helpers that call `crate::render_feature_descriptor()` and `crate::virtual_geometry_runtime_provider_registration()`.
- Modify `zircon_plugins/virtual_geometry/runtime/src/lib.rs`: add `#[cfg(test)] pub(crate) mod test_support;` near existing module declarations.
- Create `zircon_plugins/hybrid_gi/runtime/src/test_support/mod.rs`: structural test-support root for the HGI plugin crate.
- Create `zircon_plugins/hybrid_gi/runtime/src/test_support/render_feature_fixtures.rs`: HGI descriptor/framework helpers that call `crate::render_feature_descriptor()`.
- Modify `zircon_plugins/hybrid_gi/runtime/src/lib.rs`: add `#[cfg(test)] pub(crate) mod test_support;` near existing module declarations.
- Modify unwired files under `zircon_plugins/*/runtime/src/*/test_sources/*.rs`: replace only stale `crate::graphics::tests::plugin_render_feature_fixtures::*` imports with direct `crate::test_support::render_feature_fixtures::*` imports.
- Update `docs/assets-and-rendering/virtual-geometry-nanite-foundation.md` and `docs/assets-and-rendering/hybrid-gi-lumen-scene-representation.md` with the new fixture module paths and validation evidence.

## Milestone 1: Plugin-Owned Fixture Roots

Implementation slices:

- [x] Add `#[cfg(test)] pub(crate) mod test_support;` to both plugin runtime crate roots.
- [x] Add VG `test_support/mod.rs` with `pub(crate) mod render_feature_fixtures;`.
- [x] Add HGI `test_support/mod.rs` with `pub(crate) mod render_feature_fixtures;`.
- [x] Add VG fixture helpers:

```rust
use std::sync::Arc;

use zircon_runtime::asset::pipeline::manager::ProjectAssetManager;
use zircon_runtime::graphics::{RenderFeatureDescriptor, WgpuRenderFramework};

pub(crate) fn virtual_geometry_render_feature_descriptor() -> RenderFeatureDescriptor {
    crate::render_feature_descriptor()
}

pub(crate) fn pluginized_wgpu_render_framework_with_asset_manager(
    asset_manager: Arc<ProjectAssetManager>,
) -> WgpuRenderFramework {
    WgpuRenderFramework::new_with_plugin_render_features(
        asset_manager,
        [virtual_geometry_render_feature_descriptor()],
        [crate::virtual_geometry_runtime_provider_registration()],
    )
    .unwrap()
}
```

- [x] Add HGI fixture helpers:

```rust
use std::sync::Arc;

use zircon_runtime::asset::pipeline::manager::ProjectAssetManager;
use zircon_runtime::graphics::{RenderFeatureDescriptor, WgpuRenderFramework};

pub(crate) fn hybrid_gi_render_feature_descriptor() -> RenderFeatureDescriptor {
    crate::render_feature_descriptor()
}

pub(crate) fn pluginized_wgpu_render_framework_with_asset_manager(
    asset_manager: Arc<ProjectAssetManager>,
) -> WgpuRenderFramework {
    WgpuRenderFramework::new_with_plugin_render_features(
        asset_manager,
        [hybrid_gi_render_feature_descriptor()],
        Vec::new(),
    )
    .unwrap()
}
```

- [x] Add one small unit assertion in each fixture file proving the helper returns the plugin crate descriptor name and capability requirement.

Testing stage:

- [x] Run `cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_virtual_geometry_runtime --lib --locked --offline render_feature_fixture -- --nocapture`.
- [x] Run `cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_hybrid_gi_runtime --lib --locked --offline render_feature_fixture -- --nocapture`.
- [x] If either fails, fix only the fixture or root module wiring before continuing.

## Milestone 2: Rewrite Stale Fixture Imports

Implementation slices:

- [x] In VG `test_sources`, replace `crate::graphics::tests::plugin_render_feature_fixtures::{ pluginized_wgpu_render_framework_with_asset_manager, virtual_geometry_render_feature_descriptor }` with `crate::test_support::render_feature_fixtures::{ pluginized_wgpu_render_framework_with_asset_manager, virtual_geometry_render_feature_descriptor }`.
- [x] In VG `test_sources`, replace single-function imports of `virtual_geometry_render_feature_descriptor` with `crate::test_support::render_feature_fixtures::virtual_geometry_render_feature_descriptor`.
- [x] In HGI `test_sources`, replace single-function imports of `hybrid_gi_render_feature_descriptor` with `crate::test_support::render_feature_fixtures::hybrid_gi_render_feature_descriptor`.
- [x] If a HGI file needs framework helper later, use `crate::test_support::render_feature_fixtures::pluginized_wgpu_render_framework_with_asset_manager` rather than importing the runtime fixture path.
- [x] Re-run searches for `crate::graphics::tests::plugin_render_feature_fixtures` under both plugin runtime source trees; expected result is zero hits.

Testing stage:

- [x] Run `cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_virtual_geometry_runtime --lib --locked --offline`.
- [x] Run `cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_hybrid_gi_runtime --lib --locked --offline`.
- [x] Do not wire additional renderer tests if these checks expose unrelated stale `crate::asset`, `crate::graphics::types`, or private `scene_renderer` assumptions; record them as next-slice debt.

## Milestone 3: Documentation And Acceptance

Implementation slices:

- [x] Update VG docs header to include `zircon_plugins/virtual_geometry/runtime/src/test_support/mod.rs` and `zircon_plugins/virtual_geometry/runtime/src/test_support/render_feature_fixtures.rs`.
- [x] Update HGI docs header to include `zircon_plugins/hybrid_gi/runtime/src/test_support/mod.rs` and `zircon_plugins/hybrid_gi/runtime/src/test_support/render_feature_fixtures.rs`.
- [x] Add a short paragraph to both docs explaining that plugin render tests now source descriptors from plugin-local fixtures, not `zircon_runtime::graphics::tests`.
- [x] Update the active session note with exact commands, pass/fail status, and remaining stale path inventory.

Testing stage:

- [x] Run `cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_virtual_geometry_runtime --lib --locked --offline -- --nocapture`.
- [x] Run `cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_hybrid_gi_runtime --lib --locked --offline -- --nocapture`.
- [x] Report warnings separately from failures; do not claim workspace-wide validation.

## Self-Review

- Spec coverage: the plan covers plugin-owned fixtures, direct import rewrites, scoped validation, and documentation.
- Placeholder scan: no `TBD`, `TODO`, or unspecified test commands remain.
- Type consistency: fixture helpers use existing plugin crate functions and existing `WgpuRenderFramework::new_with_plugin_render_features(...)` signature.
