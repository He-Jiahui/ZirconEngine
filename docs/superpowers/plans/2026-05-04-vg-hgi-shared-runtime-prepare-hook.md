# VG/HGI Shared Runtime Prepare Hook Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a shared runtime-side prepare hook so scene rendering can collect real VG/HGI plugin outputs centrally without moving VG/HGI ownership into `zircon_runtime`.

**Architecture:** `zircon_runtime` will host a neutral prepare hook interface and a central collection point in scene rendering. `zircon_plugins/virtual_geometry/runtime` and `zircon_plugins/hybrid_gi/runtime` will remain the concrete owners of readback-producing state; the runtime hook will only call plugin-provided collectors and merge their neutral `RenderPluginRendererOutputs` into the existing scene-renderer output path. This is a hard boundary change: no compatibility shims, no plugin concrete types in runtime, and no backdoor population from fake CPU-only feedback.

**Tech Stack:** Rust, Cargo, `zircon_runtime`, `zircon_plugins`, neutral render DTOs, scene renderer prepare/feedback flow, docs under `docs/`.

---

## Architecture Notes

- Ownership boundary: `zircon_runtime` owns the hook protocol, dispatch site, and neutral aggregation path; VG/HGI plugins own the real producer state and the conversions into `RenderPluginRendererOutputs`.
- Data flow: scene renderer runtime prepare executes the shared hook, plugins package their own VG/HGI readbacks into neutral output, runtime stores the combined output sideband, and submit/feedback continues projecting from that neutral packet.
- Hard-cutover rule: do not introduce a shared runtime wrapper around plugin internals or a second source of truth for VG/HGI readbacks.

## File Map

- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_resources/runtime_prepare.rs` to execute the shared prepare hook and return collected neutral plugin outputs.
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_resources/scene_renderer_advanced_plugin_resources.rs` to hold any small runtime-owned hook state needed for central collection.
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_readbacks/scene_renderer_advanced_plugin_readbacks.rs` if the neutral packet shape needs a helper for merging hook outputs.
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_readbacks/collect_into_outputs.rs` if collection needs to consume the new shared hook result path.
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_compiled_scene/render/render.rs` only if the call site needs to pass the hook result through the render path.
- Modify: `zircon_plugins/virtual_geometry/runtime/src/provider.rs` so the VG provider exposes a runtime-prepare collector that returns `RenderPluginRendererOutputs` from real VG state.
- Modify: `zircon_plugins/hybrid_gi/runtime/src/provider.rs` so the HGI provider exposes a runtime-prepare collector that returns `RenderPluginRendererOutputs` from real HGI GPU readback state.
- Modify: `zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_output_sources/virtual_geometry_plugin_renderer_outputs.rs` if a public collector entry point is needed for provider use.
- Modify: `zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/root_output_sources/hybrid_gi_plugin_renderer_outputs.rs` if a public collector entry point is needed for provider use.
- Update: `docs/assets-and-rendering/virtual-geometry-nanite-foundation.md`.
- Update: `docs/assets-and-rendering/hybrid-gi-lumen-scene-representation.md`.
- Update: `.codex/sessions/20260502-2119-vg-hgi-unreal-followup.md`.

## Milestone 1: Shared Hook Boundary

### Implementation Slices

- [ ] Add a small neutral runtime prepare hook contract in `zircon_runtime` that can ask each advanced plugin for prepared neutral outputs without referencing plugin crate types.
- [ ] Thread that hook through `SceneRendererAdvancedPluginResources::execute_runtime_prepare_passes(...)` so the scene renderer prepares an aggregated `RenderPluginRendererOutputs` packet centrally.
- [ ] Keep the existing neutral output merge semantics intact: the runtime still stores one combined packet and later projections keep using the same `collect_into_outputs` path.
- [ ] Add focused unit coverage for the central merge path returning empty output when no plugin produces anything and returning the expected neutral packet when one plugin does.

### Testing Stage

- [ ] Run `rustfmt --edition 2021 --check` on the touched runtime files.
- [ ] Run `git diff --check` on the touched files.
- [ ] If a new runtime test is added, run the narrow `cargo test` target for that test only.
- [ ] Fix any compile or test failure in the lowest shared runtime layer first, then rerun the same check until clean.

## Milestone 2: Plugin-Side Collector Wiring

### Implementation Slices

- [ ] Expose a provider-side collector in `zircon_plugins/virtual_geometry/runtime/src/provider.rs` that packages the real VG readback source into `RenderPluginRendererOutputs` through the existing plugin-local helper.
- [ ] Expose a provider-side collector in `zircon_plugins/hybrid_gi/runtime/src/provider.rs` that packages the real HGI `HybridGiGpuReadback` into `RenderPluginRendererOutputs` through the existing plugin-local helper.
- [ ] Keep VG collection tied to the real `VirtualGeometryIndirectStats::node_and_cluster_cull_readback_outputs()` path, not to CPU prepare heuristics.
- [ ] Keep HGI collection tied to the real `HybridGiGpuReadback` path, including the scene-prepare readback snapshot already stored in plugin state.
- [ ] Add focused unit tests that prove each provider collector returns the expected neutral `RenderPluginRendererOutputs` from concrete plugin-owned readback state.

### Testing Stage

- [ ] Run `rustfmt --edition 2021 --check` on the touched plugin files.
- [ ] Run `git diff --check` on the touched files.
- [ ] Run the narrow plugin tests that cover the new collector behavior.
- [ ] Fix failures bottom-up, starting from the collector helper if the provider test fails.

## Milestone 3: Scene-Render Integration

### Implementation Slices

- [ ] Connect the shared runtime hook result into the render submission flow so `prepare_runtime_submission(...)` receives the centrally collected VG/HGI packet.
- [ ] Verify `collect_runtime_feedback(...)` still merges the prepared packet with the renderer's last output without double-counting or losing plugin-side state.
- [ ] Preserve the current particle filtering boundary: the VG/HGI path should not start admitting particle readbacks through this hook.
- [ ] Add an integration-style unit test around the prepare/submit boundary that confirms the plugin-side packet survives through submission into feedback.

### Testing Stage

- [ ] Run `rustfmt --edition 2021 --check` on the touched submit/prepare files.
- [ ] Run `git diff --check` on the touched files.
- [ ] Run the narrow `cargo test` target for the submit/feedback path that exercises the new hook.
- [ ] Fix the lowest failing shared layer first and rerun until the integration test passes.

## Milestone 4: Docs And Acceptance

### Implementation Slices

- [ ] Update the VG and HGI docs to explain that scene rendering now collects neutral plugin outputs centrally while VG/HGI readback ownership remains in the plugins.
- [ ] Update the coordination session note with the hook boundary, the collector ownership split, and the validation result.
- [ ] Re-run the hard-cutover search for any new runtime/plugin coupling or stale ownership assumptions introduced by the hook.

### Testing Stage

- [ ] Run `rustfmt --edition 2021 --check` on the touched docs-adjacent Rust files.
- [ ] Run `git diff --check`.
- [ ] Run the scoped plugin and runtime tests that cover the new prepare hook.
- [ ] Record the exact commands, failures, fixes, and accepted residual risks in the session note.

## Acceptance Checklist

- [ ] Scene rendering has a shared runtime-side prepare hook for advanced plugin outputs.
- [ ] `zircon_runtime` still only owns the neutral hook/aggregation path, not VG/HGI concrete readback state.
- [ ] VG and HGI providers still package real plugin-owned readback sources into `RenderPluginRendererOutputs`.
- [ ] The submit/feedback path still merges the neutral sideband without regressing existing runtime feedback behavior.
- [ ] Docs and session notes explain the new central collection boundary and the validation evidence.
