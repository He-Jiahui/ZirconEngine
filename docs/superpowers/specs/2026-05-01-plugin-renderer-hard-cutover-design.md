# Plugin Renderer Hard-Cutover Design

## Summary

This spec defines the next staged hard-cutover for moved Virtual Geometry and Hybrid GI renderer sources. The goal is to remove stale old-owner assumptions from plugin-owned renderer/pass/test-source files before broad module wiring, then connect the plugin renderer trees through direct plugin paths and neutral runtime contracts only.

The target boundary is strict: `zircon_plugins/virtual_geometry/runtime` owns concrete Virtual Geometry prepare, GPU, readback, pass, and renderer helper types; `zircon_plugins/hybrid_gi/runtime` owns concrete Hybrid GI prepare, resolve, GPU, readback, post-process, and renderer helper types; `zircon_runtime` keeps neutral graphics contracts such as render feature descriptors, render graph pass metadata, base frame submission, and renderer execution interfaces. `zircon_runtime` must not depend on `zircon_plugins`.

## Goals

- Replace stale moved-source imports such as `crate::graphics::types::*`, `crate::graphics::scene::scene_renderer::*`, `crate::graphics::runtime::*`, and `crate::graphics::tests::*` inside VG/HGI plugin runtime crates.
- Replace old `pub(in crate::graphics...)` visibility scopes with plugin-local scopes that match the new owners.
- Wire broad plugin renderer module trees only after the stale path cutover makes ownership explicit.
- Keep `zircon_runtime` as the neutral host for frame/graph/descriptor contracts, exposing only curated public runtime DTOs when plugin crates genuinely need them.
- Update module documentation and coordination notes with code paths, plan sources, tests, and validation evidence.

## Non-Goals

- Do not recreate `zircon_runtime::graphics::runtime::{hybrid_gi,virtual_geometry}`.
- Do not recreate `zircon_runtime::graphics::scene::scene_renderer::{hybrid_gi,virtual_geometry}` as compatibility modules, re-exports, facades, shims, or bridges.
- Do not make `zircon_runtime` import or depend on any `zircon_plugins` crate.
- Do not touch Runtime UI, editor UI, or active UI session paths.
- Do not delete moved renderer/pass code simply to avoid wiring it, unless a file is proven obsolete and has no intended plugin-owned responsibility.

## Approved Approach

Use the staged direct hard-cutover approach.

The rejected full-integration approach would cut over imports, wire all renderer/pass modules, and add runtime execution hooks in one pass. That is too risky because `ViewportRenderFrame`, advanced plugin resources/readbacks, and concrete VG/HGI outputs still straddle runtime/plugin ownership.

The rejected quarantine/delete approach would remove unwired moved renderer files or leave them disconnected. That reduces short-term compile drift, but it violates the migration goal because the migrated renderer/pass implementations are intended to converge into plugin ownership.

## Architecture

`zircon_runtime` remains the host and neutral contract layer. It owns base graphics types such as `GraphicsError`, `ViewportFrame`, render feature descriptors, render graph scheduling metadata, frame history bindings, and the compiled graph execution record. It may expose `ViewportRenderFrame` publicly if plugin renderer code needs the frame extract, scene snapshot, viewport size, and UI extract as a neutral input packet. Any such public exposure must not carry VG/HGI concrete prepare DTOs back into `zircon_runtime::graphics::types`.

`zircon_plugins/virtual_geometry/runtime` owns the Virtual Geometry renderer implementation. Plugin-owned types under `crate::virtual_geometry::types` replace removed runtime type paths for prepare frames, clusters, pages, draw segments, cluster selections, node/cluster cull records, raster draws, and indirect stats data. Renderer code under `crate::virtual_geometry::renderer` owns GPU resources, readbacks, root render passes, mesh sources, output sources, and state readback helpers.

`zircon_plugins/hybrid_gi/runtime` owns the Hybrid GI renderer implementation. Plugin-owned types under `crate::hybrid_gi::types` replace removed runtime type paths for prepare frames, probes, voxel clipmaps/cells, surface-cache page content, resolve runtime, and resolve scene data. Renderer code under `crate::hybrid_gi::renderer` owns GPU resources, readbacks, scene prepare outputs, post-process sources, hierarchy helpers, and trace/probe source helpers.

The runtime graph and plugin feature descriptors remain the schedule authority. Plugin render feature descriptors already provide pass names, executor ids, queues, dependencies, and resource access declarations. The hard-cutover should preserve this model and add only the minimal neutral hook needed at the runtime execution-boundary milestone.

## Module Boundaries

Plugin root files stay structural.

- `zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/mod.rs` may declare `mod renderer;` and re-export only the plugin-local items needed by sibling plugin modules or tests.
- `zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/mod.rs` may declare `mod renderer;` and re-export only the plugin-local items needed by sibling plugin modules or tests.
- `renderer/mod.rs` files may wire child modules and expose a minimal plugin-local surface. They must not become implementation files.
- Root output/source folders that were moved from runtime keep their current behavior files, but their imports and visibility must point at plugin-local modules or public neutral runtime contracts.

Visibility replacements should be as narrow as practical:

- `pub(in crate::graphics::scene::scene_renderer...)` becomes `pub(in crate::virtual_geometry::renderer...)` or `pub(in crate::hybrid_gi::renderer...)` for renderer internals.
- `pub(in crate::graphics)` becomes `pub(crate)` only when sibling plugin modules or tests require access.
- Public exports are allowed only for stable plugin crate APIs, registration functions, or neutral runtime-facing hook types.

## Data Flow

The implementation should cut over data flow in this order:

1. Plugin-owned DTO imports: replace stale `crate::graphics::types::{VirtualGeometry..., HybridGi...}` references with `crate::virtual_geometry::types::*` or `crate::hybrid_gi::types::*`.
2. Neutral runtime imports: replace runtime-owned neutral imports with direct `zircon_runtime::*` paths, such as `zircon_runtime::graphics::GraphicsError` and, if approved by compile evidence, `zircon_runtime::graphics::ViewportRenderFrame`.
3. Renderer ownership imports: replace stale `crate::graphics::scene::scene_renderer::*` references with plugin-local `crate::virtual_geometry::renderer::*` or `crate::hybrid_gi::renderer::*` paths when the type was moved into the plugin.
4. Module wiring: declare and wire renderer subtrees after stale imports and visibility scopes no longer point at old owners.
5. Runtime execution boundary: only after plugin modules compile locally, decide whether runtime needs a neutral registration/executor hook beyond the existing descriptor/no-op executor path.

This staged order intentionally prefers visible compile failures over compatibility wrappers. A stale import after each step is migration evidence, not a reason to restore an old runtime path.

## Runtime Hook Boundary

The first milestone should not invent a broad plugin renderer ABI. It should use current render feature descriptors and `RenderPassExecutorRegistry` behavior as the existing neutral boundary, then extend it only if compile wiring proves the no-op executor registry cannot represent plugin-owned renderer execution.

If an extension is required, it should be neutral and host-owned. Acceptable shapes include a runtime-defined executor registration that passes a neutral execution context containing pass metadata, device/queue/encoder/frame references, resource handles, and erased plugin output slots. Unacceptable shapes include runtime imports of `HybridGiGpuResources`, `VirtualGeometryGpuResources`, plugin readback completion parts, or plugin-specific prepare structs.

Any hook added in this slice must satisfy these constraints:

- The trait or function type is declared in `zircon_runtime`.
- Plugin crates register implementations through existing runtime extension registration paths or a direct successor to those paths.
- `zircon_runtime` stores only neutral descriptors, ids, trait objects, or erased packets.
- Plugin-specific GPU resources, readbacks, completion parts, and prepare outputs stay inside the plugin crate.

## Error Handling

Runtime-facing failures should use public neutral runtime errors such as `zircon_runtime::graphics::GraphicsError` where the code crosses graphics host boundaries. Plugin-local preparation, readback, and validation helpers may use `Result<_, String>` or plugin-private error helpers if the error never crosses a public runtime boundary.

Do not introduce broad error enums only to paper over import migration. Add a new error type only if multiple plugin-owned modules need a shared semantic error contract.

## Tests And Validation

Implementation milestones should use scoped checks during development and full milestone testing at the boundary.

Expected implementation-stage gates:

- Search `zircon_plugins/virtual_geometry/runtime/src/virtual_geometry` for `crate::graphics::types`, `crate::graphics::scene::scene_renderer`, `pub(in crate::graphics`, `crate::graphics::runtime`, and `crate::graphics::tests` after each cutover slice.
- Search `zircon_plugins/hybrid_gi/runtime/src/hybrid_gi` for the same old-owner patterns after each cutover slice.
- Run `cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_virtual_geometry_runtime --lib --locked --offline` after VG renderer wiring reaches a coherent compile boundary.
- Run `cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_hybrid_gi_runtime --lib --locked --offline` after HGI renderer wiring reaches a coherent compile boundary.

Expected milestone testing-stage gates:

- Run plugin package tests for `zircon_plugin_virtual_geometry_runtime` when renderer/test-source wiring is coherent enough to exercise the moved tests.
- Run plugin package tests for `zircon_plugin_hybrid_gi_runtime` when renderer/test-source wiring is coherent enough to exercise the moved tests.
- If runtime hook code changes, run the focused runtime render graph/plugin registration tests that cover descriptor validation and executor registration.
- Document any failures as either in-scope blockers or unrelated active-session drift with exact command output summaries.

## Documentation

Code changes in this migration must update the existing functional docs rather than creating a new random bucket.

- Update `docs/assets-and-rendering/virtual-geometry-nanite-foundation.md` with the final VG renderer module paths, moved type owners, validation commands, and plan sources.
- Update `docs/assets-and-rendering/hybrid-gi-lumen-scene-representation.md` with the final HGI renderer module paths, moved type owners, validation commands, and plan sources.
- Keep machine-readable related-code headers current in those docs.
- Update `.codex/sessions/20260501-1850-plugin-renderer-hard-cutover.md` as the active coordination note during implementation, then archive it with validation evidence when complete.

## Acceptance Criteria

- Plugin renderer/test-source trees no longer contain live stale references to `crate::graphics::types`, `crate::graphics::scene::scene_renderer`, `pub(in crate::graphics...)`, `crate::graphics::runtime`, or `crate::graphics::tests` except historical text in comments that is explicitly marked non-code history.
- VG/HGI concrete prepare, resolve, GPU resource, readback, and renderer helper types are imported from plugin-local owners.
- Neutral runtime types are imported through public `zircon_runtime` surfaces, not old crate-local runtime paths.
- Plugin renderer module trees are wired structurally without root files becoming implementation buckets.
- No compatibility re-exports, shims, facades, bridge modules, or legacy aliases are added to preserve old migration paths.
- Scoped plugin checks pass, or remaining failures are documented as concrete in-scope blockers for the next correction loop.
- VG/HGI docs and the active session note record the implementation files, plan sources, tests, and validation evidence.
