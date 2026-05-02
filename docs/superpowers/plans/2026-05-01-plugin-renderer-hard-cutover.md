# Plugin Renderer Hard-Cutover Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Cut over moved Virtual Geometry and Hybrid GI renderer/pass sources to plugin-local ownership without restoring old `zircon_runtime::graphics` owner paths or compatibility shims.

**Architecture:** Work lower-to-upper: refresh coordination and stale-path inventory, stabilize neutral runtime contracts, cut over plugin-local DTO imports and visibility, wire VG/HGI renderer module trees, then validate plugin crates and update docs. `zircon_runtime` remains the neutral host for frame/graph/descriptor contracts; plugin crates own concrete VG/HGI renderer resources, prepare data, readbacks, and pass helpers.

**Tech Stack:** Rust 2021, Cargo workspaces, `wgpu`, `zircon_runtime`, `zircon_plugins`, render graph descriptors, plugin runtime crates, Markdown docs with machine-readable frontmatter.

---

## Repository Baseline

- Work from the existing `main` checkout at `E:\Git\ZirconEngine`; do not create a worktree or feature branch.
- Do not commit automatically. The repository policy develops on `main`, and the current user preference is no automatic commits.
- Root workspace members are `zircon_app`, `zircon_runtime`, and `zircon_editor`; plugin runtime crates live in the separate `zircon_plugins/Cargo.toml` workspace.
- CI runs `cargo build --workspace --locked --verbose` and `cargo test --workspace --locked --verbose` on Ubuntu, but this plan uses scoped plugin/runtime validation because active UI sessions have unrelated dirty changes and known graphics/plugin drift.
- Active coordination note: `.codex/sessions/20260501-1850-plugin-renderer-hard-cutover.md`.
- Approved design spec: `docs/superpowers/specs/2026-05-01-plugin-renderer-hard-cutover-design.md`.
- Fresh stale-path inventory before this plan found 293 VG matches and 120 HGI matches for old-owner patterns under plugin runtime source trees.

## File Structure Map

### Coordination And Planning

- Modify: `.codex/sessions/20260501-1850-plugin-renderer-hard-cutover.md` to record scope changes, validation evidence, blockers, and completion state.
- Create/modify: `docs/superpowers/plans/2026-05-01-plugin-renderer-hard-cutover.md` for this implementation plan.
- Reference: `docs/superpowers/specs/2026-05-01-plugin-renderer-hard-cutover-design.md` for accepted architecture and acceptance criteria.

### Neutral Runtime Contracts

- Modify: `zircon_runtime/src/graphics/types/mod.rs` only to expose neutral frame DTOs that plugin crates genuinely need.
- Modify: `zircon_runtime/src/graphics/mod.rs` only to re-export neutral public graphics contracts.
- Inspect before editing: `zircon_runtime/src/graphics/types/viewport_render_frame.rs` to confirm fields remain neutral and no VG/HGI concrete DTO fields are exposed.
- Inspect before editing: `zircon_runtime/src/graphics/types/viewport_render_frame_with_virtual_geometry_debug_snapshot.rs` to keep debug-snapshot mutation crate-private.
- Inspect before editing: `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs` and `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs` for neutral executor hook requirements.
- Possible neutral draw-view extraction: if plugin VG pass code still requires private `MeshDraw`, add a public neutral view under `zircon_runtime/src/core/framework/render/` instead of exposing `zircon_runtime::graphics::scene::scene_renderer::mesh::MeshDraw` directly.

### Virtual Geometry Plugin Runtime

- Modify: `zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/mod.rs` to wire `mod renderer;` after old paths are cut over.
- Modify: `zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/mod.rs` to keep structural wiring and plugin-local exports.
- Modify: `zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/types/mod.rs` only if additional plugin-local type exports are required by moved renderer/test code.
- Modify renderer subtrees under `zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/gpu_readback/`, `gpu_resources/`, `root_mesh_sources/`, `root_output_sources/`, `root_render_passes/`, and `root_state_readbacks/` to replace stale imports and visibility scopes.
- Modify test sources under `zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/test_sources/` only for moved renderer-related tests that are wired in this slice.
- Do not modify or restore deleted runtime owner paths under `zircon_runtime/src/graphics/runtime/virtual_geometry` or `zircon_runtime/src/graphics/scene/scene_renderer/virtual_geometry`.

### Hybrid GI Plugin Runtime

- Modify: `zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/mod.rs` to wire `mod renderer;` after old paths are cut over.
- Modify: `zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/mod.rs` to keep structural wiring and plugin-local exports.
- Modify: `zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/types/mod.rs` only if additional plugin-local type exports are required by moved renderer/test code.
- Modify renderer subtrees under `zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_readback/`, `gpu_resources/`, `post_process_sources/`, and `root_output_sources/` to replace stale imports and visibility scopes.
- Modify test sources under `zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/test_sources/` only for moved renderer-related tests that are wired in this slice.
- Do not modify or restore deleted runtime owner paths under `zircon_runtime/src/graphics/runtime/hybrid_gi` or `zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi`.

### Documentation

- Modify: `docs/assets-and-rendering/virtual-geometry-nanite-foundation.md` with final VG renderer module paths, plan sources, implementation files, and validation evidence.
- Modify: `docs/assets-and-rendering/hybrid-gi-lumen-scene-representation.md` with final HGI renderer module paths, plan sources, implementation files, and validation evidence.
- Keep `related_code`, `implementation_files`, `plan_sources`, and `tests` frontmatter current for both docs.

## Milestone 0: Coordination And Stale-Path Inventory

- Goal: establish the live baseline and exact old-owner search set before production edits.
- In-scope behaviors: coordination note freshness, branch policy, current stale import/visibility counts, active-session overlap check.
- Dependencies: approved design spec and current `main` checkout.
- Lightweight checks: read/search only; no Cargo commands in this milestone.

### Implementation Slices

- [ ] Confirm branch policy from the repository root.

Run:

```powershell
git branch --show-current
```

Expected: `main`.

- [ ] Refresh coordination context and update `.codex/sessions/20260501-1850-plugin-renderer-hard-cutover.md` if a new active session overlaps graphics/plugin files.

Run:

```powershell
.\.opencode\skills\zircon-project-skills\cross-session-coordination\scripts\Get-RecentCoordinationContext.ps1 -RepoRoot "E:\Git\ZirconEngine" -LookbackHours 4
```

Expected: no active graphics/plugin session other than `.codex/sessions/20260501-1850-plugin-renderer-hard-cutover.md`.

- [ ] Record the current old-owner inventory for VG.

Run:

```powershell
rg --line-number "crate::graphics::types|crate::graphics::scene::scene_renderer|pub\(in crate::graphics|crate::graphics::runtime|crate::graphics::tests" "zircon_plugins/virtual_geometry/runtime/src/virtual_geometry"
```

Expected: every hit is an in-scope stale moved-source path to eliminate or a newly discovered blocker to record.

- [ ] Record the current old-owner inventory for HGI.

Run:

```powershell
rg --line-number "crate::graphics::types|crate::graphics::scene::scene_renderer|pub\(in crate::graphics|crate::graphics::runtime|crate::graphics::tests" "zircon_plugins/hybrid_gi/runtime/src/hybrid_gi"
```

Expected: every hit is an in-scope stale moved-source path to eliminate or a newly discovered blocker to record.

### Testing Stage

- [ ] No Cargo validation for Milestone 0. Accept the milestone only after the session note records branch, coordination, and stale-path inventory evidence.

### Exit Evidence

- Branch is `main`.
- Coordination note is current.
- VG and HGI stale-path search outputs are summarized in the session note.

## Milestone 1: Neutral Runtime Contract Boundary

- Goal: expose only the neutral runtime contracts plugin renderer code needs, without leaking VG/HGI concrete DTOs back into runtime graphics types.
- In-scope behaviors: public neutral `ViewportRenderFrame` export, continued crate-private debug snapshot mutation, no runtime dependency on plugin crates.
- Dependencies: Milestone 0 inventory.
- Lightweight checks: scoped `cargo check` for `zircon_runtime` at the testing stage.

### Implementation Slices

- [ ] Inspect `zircon_runtime/src/graphics/types/viewport_render_frame.rs` and confirm public fields remain neutral: `scene`, `extract`, `viewport_size`, and `ui` are runtime frame/extract data; `virtual_geometry_debug_snapshot` remains crate-private.

- [ ] Modify `zircon_runtime/src/graphics/types/mod.rs` to make `ViewportRenderFrame` publicly re-exported when plugin renderer code imports it.

Target line shape:

```rust
pub use viewport_render_frame::ViewportRenderFrame;
```

- [ ] Modify `zircon_runtime/src/graphics/mod.rs` to make `ViewportRenderFrame` part of the public `zircon_runtime::graphics` surface.

Target line shape:

```rust
pub use types::{
    GpuResourceHandle, GraphicsError, ViewportFrame, ViewportFrameTextureHandle,
    ViewportRenderFrame,
};
```

- [ ] Do not change `zircon_runtime/src/graphics/types/viewport_render_frame_with_virtual_geometry_debug_snapshot.rs`; `with_virtual_geometry_debug_snapshot` stays `pub(crate)`.

- [ ] If plugin code requires private `MeshDraw` access, do not publicize `MeshDraw`. Add a neutral render draw-view type or trait under `zircon_runtime/src/core/framework/render/` and expose only the fields/methods used by plugin pass helpers: indirect-args buffer reference, indirect offset, indirect draw flag, virtual-geometry execution selection key, execution segment, submission order record, draw submission record, and draw submission token record.

Neutral extraction target if required:

```rust
pub struct RenderVirtualGeometryExecutionDraw<'a> {
    pub indirect_args_buffer: Option<&'a wgpu::Buffer>,
    pub indirect_args_offset: u64,
    pub uses_indirect_draw: bool,
    pub execution_selection_key: Option<(u64, u32)>,
    pub execution_segment: RenderVirtualGeometryExecutionSegment,
    pub submission_order_record: Option<(Option<u32>, u64, u32)>,
    pub draw_submission_record: Option<(u64, u32, u32, usize)>,
    pub draw_submission_token_record: Option<(u64, u32, u32, u32, usize)>,
}
```

Add this type only when a compile error proves it is needed by plugin-owned code.

### Testing Stage

- [ ] Run the scoped runtime check only after the neutral export edit is complete.

Run:

```powershell
cargo check -p zircon_runtime --lib --locked --offline
```

Expected: command completes, or failures are unrelated active-session drift and recorded with exact diagnostics in the session note.

### Exit Evidence

- `ViewportRenderFrame` is public through `zircon_runtime::graphics` if plugin code needs it.
- `virtual_geometry_debug_snapshot` mutation remains crate-private.
- No `zircon_runtime` file imports `zircon_plugins`.

## Milestone 2: Virtual Geometry Renderer Local Cutover

- Goal: make moved VG renderer/pass/test-source code import plugin-owned DTOs and plugin-local renderer owners directly.
- In-scope behaviors: VG `crate::graphics::*` removal, plugin-local visibility scopes, renderer module wiring, renderer-related test-source wiring.
- Dependencies: Milestone 1 neutral runtime boundary.
- Lightweight checks: stale-path `rg` after each slice; Cargo check in the testing stage.

### Implementation Slices

- [ ] Replace neutral runtime imports in VG plugin files.

Use these replacement rules:

```rust
// Old moved-runtime assumptions
use crate::core::framework::render::{/* ... */};
use crate::graphics::types::GraphicsError;
use crate::graphics::types::ViewportRenderFrame;

// New plugin crate imports
use zircon_runtime::core::framework::render::{/* ... */};
use zircon_runtime::graphics::{GraphicsError, ViewportRenderFrame};
```

Apply to files under `zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/` and renderer-related files under `zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/test_sources/`.

- [ ] Replace VG concrete type imports with plugin-local `crate::virtual_geometry::types` imports.

Use this import shape where concrete VG prepare/pass types are referenced:

```rust
use crate::virtual_geometry::types::{
    VirtualGeometryClusterRasterDraw, VirtualGeometryClusterSelection,
    VirtualGeometryNodeAndClusterCullChildWorkItem,
    VirtualGeometryNodeAndClusterCullClusterWorkItem,
    VirtualGeometryNodeAndClusterCullTraversalRecord, VirtualGeometryPrepareCluster,
    VirtualGeometryPrepareClusterState, VirtualGeometryPrepareDrawSegment,
    VirtualGeometryPrepareFrame, VirtualGeometryPrepareIndirectDraw,
    VirtualGeometryPreparePage, VirtualGeometryPrepareRequest,
};
```

Remove unused names after each file is converted.

- [ ] Replace fully qualified VG concrete paths.

Examples:

```rust
crate::graphics::types::VirtualGeometryPrepareRequest
```

becomes:

```rust
VirtualGeometryPrepareRequest
```

after importing from `crate::virtual_geometry::types`.

- [ ] Replace VG renderer visibility scopes.

Use these target scopes:

```rust
pub(in crate::virtual_geometry::renderer)
pub(in crate::virtual_geometry::renderer::gpu_readback)
pub(in crate::virtual_geometry::renderer::gpu_resources)
pub(in crate::virtual_geometry::renderer::root_render_passes)
pub(crate)
```

Use `pub(crate)` only for values consumed by sibling plugin modules or plugin tests. Use narrower `pub(in ...)` scopes for renderer internals.

- [ ] Update `zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/mod.rs` to structural plugin-local exports.

Target shape:

```rust
mod gpu_readback;
mod gpu_resources;
mod root_mesh_sources;
mod root_output_sources;
mod root_render_passes;
mod root_state_readbacks;

pub(crate) use gpu_readback::{
    VirtualGeometryGpuPendingReadback, VirtualGeometryGpuReadback,
    VirtualGeometryGpuReadbackCompletionParts,
};
pub(crate) use gpu_resources::VirtualGeometryGpuResources;
```

Keep exports narrower if compile evidence shows a type is used only inside `renderer`.

- [ ] Update `zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/mod.rs` to declare `mod renderer;` only after the renderer root is structurally valid.

Target line placement:

```rust
mod renderer;
```

Place it near `mod residency_management;`, `mod snapshot;`, and `mod types;`.

- [ ] If VG renderer code still references runtime-private `SceneRendererAdvancedPluginResources`, convert that dependency to a plugin-local parameter or a neutral runtime view rather than importing runtime private scene-renderer internals.

Accepted target shapes:

```rust
use crate::virtual_geometry::renderer::gpu_resources::VirtualGeometryGpuResources;
```

or, for neutral inputs:

```rust
use zircon_runtime::core::framework::render::RenderVirtualGeometryCullInputSnapshot;
```

Do not import `zircon_runtime::graphics::scene::scene_renderer::core::scene_renderer_core::SceneRendererAdvancedPluginResources` from the plugin crate.

- [ ] Wire VG moved tests only when their imports point at plugin-local types and public neutral runtime APIs.

Candidate test source declarations in `zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/mod.rs`:

```rust
#[cfg(test)]
#[path = "test_sources/virtual_geometry_gpu.rs"]
mod virtual_geometry_gpu_tests;
#[cfg(test)]
#[path = "test_sources/virtual_geometry_prepare_render.rs"]
mod virtual_geometry_prepare_render_tests;
```

Add each test source only after its old-owner paths are removed.

### Testing Stage

- [ ] Confirm VG stale paths are gone.

Run:

```powershell
rg --line-number "crate::graphics::types|crate::graphics::scene::scene_renderer|pub\(in crate::graphics|crate::graphics::runtime|crate::graphics::tests" "zircon_plugins/virtual_geometry/runtime/src/virtual_geometry"
```

Expected: no live Rust code hits. Comments are acceptable only when they explicitly describe historical old paths.

- [ ] Run VG package check.

Run:

```powershell
cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_virtual_geometry_runtime --lib --locked --offline
```

Expected: command completes. If it fails, fix the lowest plugin-local stale import, missing neutral export, or visibility error first, then rerun this exact command.

### Exit Evidence

- VG renderer tree is declared in `virtual_geometry/mod.rs`.
- VG renderer root remains structural.
- VG package check passes or an in-scope blocker is documented with exact diagnostics.

## Milestone 3: Hybrid GI Renderer Local Cutover

- Goal: make moved HGI renderer/pass/test-source code import plugin-owned DTOs and plugin-local renderer owners directly.
- In-scope behaviors: HGI `crate::graphics::*` removal, plugin-local visibility scopes, renderer module wiring, renderer-related test-source wiring.
- Dependencies: Milestone 1 neutral runtime boundary and any reusable pattern proven in Milestone 2.
- Lightweight checks: stale-path `rg` after each slice; Cargo check in the testing stage.

### Implementation Slices

- [ ] Replace neutral runtime imports in HGI plugin files.

Use these replacement rules:

```rust
// Old moved-runtime assumptions
use crate::core::framework::render::{/* ... */};
use crate::graphics::types::GraphicsError;
use crate::graphics::types::ViewportRenderFrame;

// New plugin crate imports
use zircon_runtime::core::framework::render::{/* ... */};
use zircon_runtime::graphics::{GraphicsError, ViewportRenderFrame};
```

Apply to files under `zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/` and renderer-related files under `zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/test_sources/`.

- [ ] Replace HGI concrete type imports with plugin-local `crate::hybrid_gi::types` imports.

Use this import shape where concrete HGI prepare/resolve types are referenced:

```rust
use crate::hybrid_gi::types::{
    HybridGiPrepareCardCaptureRequest, HybridGiPrepareFrame, HybridGiPrepareProbe,
    HybridGiPrepareSurfaceCachePageContent, HybridGiPrepareUpdateRequest,
    HybridGiPrepareVoxelCell, HybridGiPrepareVoxelClipmap, HybridGiResolveProbeSceneData,
    HybridGiResolveRuntime, HybridGiResolveTraceRegionSceneData, HybridGiScenePrepareFrame,
    HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT, HYBRID_GI_VOXEL_CLIPMAP_CELL_RESOLUTION,
};
```

Remove unused names after each file is converted.

- [ ] Replace fully qualified HGI concrete paths.

Examples:

```rust
crate::graphics::types::HybridGiPrepareSurfaceCachePageContent
crate::graphics::types::HybridGiResolveRuntime
```

become:

```rust
HybridGiPrepareSurfaceCachePageContent
HybridGiResolveRuntime
```

after importing from `crate::hybrid_gi::types`.

- [ ] Replace `HybridGiScenePrepareResourcesSnapshot` imports with plugin-local renderer readback imports.

Target shape:

```rust
use crate::hybrid_gi::renderer::HybridGiScenePrepareResourcesSnapshot;
```

If a file only needs runtime scene-representation data, use:

```rust
use crate::hybrid_gi::HybridGiRuntimeScenePrepareResources;
```

Do not import `crate::graphics::scene::scene_renderer::HybridGiScenePrepareResourcesSnapshot`.

- [ ] Replace HGI renderer visibility scopes.

Use these target scopes:

```rust
pub(in crate::hybrid_gi::renderer)
pub(in crate::hybrid_gi::renderer::gpu_readback)
pub(in crate::hybrid_gi::renderer::gpu_resources)
pub(in crate::hybrid_gi::renderer::post_process_sources)
pub(crate)
```

Use `pub(crate)` only for values consumed by sibling plugin modules or plugin tests. Use narrower `pub(in ...)` scopes for renderer internals.

- [ ] Update `zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/mod.rs` to structural plugin-local exports.

Target shape:

```rust
mod gpu_readback;
mod gpu_resources;
mod post_process_sources;
mod root_output_sources;

pub(crate) use gpu_readback::{
    HybridGiGpuPendingReadback, HybridGiGpuReadback,
    HybridGiGpuReadbackCompletionParts, HybridGiScenePrepareResourcesSnapshot,
};
pub(crate) use gpu_resources::HybridGiGpuResources;
```

Keep exports narrower if compile evidence shows a type is used only inside `renderer`.

- [ ] Update `zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/mod.rs` to declare `mod renderer;` only after the renderer root is structurally valid.

Target line placement:

```rust
mod renderer;
```

Place it near `mod runtime_feedback;`, `mod scene_inputs;`, and `mod scene_representation;`.

- [ ] Wire HGI moved tests only when their imports point at plugin-local types and public neutral runtime APIs.

Candidate test source declarations in `zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/mod.rs`:

```rust
#[cfg(test)]
#[path = "test_sources/hybrid_gi_gpu.rs"]
mod hybrid_gi_gpu_tests;
#[cfg(test)]
#[path = "test_sources/hybrid_gi_gpu_scene_light_seed.rs"]
mod hybrid_gi_gpu_scene_light_seed_tests;
```

Add each test source only after its old-owner paths are removed.

### Testing Stage

- [ ] Confirm HGI stale paths are gone.

Run:

```powershell
rg --line-number "crate::graphics::types|crate::graphics::scene::scene_renderer|pub\(in crate::graphics|crate::graphics::runtime|crate::graphics::tests" "zircon_plugins/hybrid_gi/runtime/src/hybrid_gi"
```

Expected: no live Rust code hits. Comments are acceptable only when they explicitly describe historical old paths.

- [ ] Run HGI package check.

Run:

```powershell
cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_hybrid_gi_runtime --lib --locked --offline
```

Expected: command completes. If it fails, fix the lowest plugin-local stale import, missing neutral export, or visibility error first, then rerun this exact command.

### Exit Evidence

- HGI renderer tree is declared in `hybrid_gi/mod.rs`.
- HGI renderer root remains structural.
- HGI package check passes or an in-scope blocker is documented with exact diagnostics.

## Milestone 4: Runtime Execution Boundary Decision

- Goal: decide whether descriptor/no-op executor registration is sufficient for this cutover or whether a minimal neutral runtime executor hook must be added.
- In-scope behaviors: render feature descriptor preservation, runtime/plugin dependency direction, neutral executor context only.
- Dependencies: Milestones 2 and 3 package checks or exact blocker diagnostics.
- Lightweight checks: focused runtime render graph/plugin registration tests only if runtime hook files change.

### Implementation Slices

- [ ] Inspect compile evidence from Milestones 2 and 3.

Decision A: if plugin crates compile with renderer modules wired and current descriptor/no-op execution remains coherent, do not add a runtime hook in this milestone.

Decision B: if compile or integration evidence proves plugin-owned pass execution needs a runtime-owned hook, edit only neutral runtime graph/extension files.

- [ ] For Decision B, modify `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs` only with neutral fields. Do not add VG/HGI concrete types.

Allowed context data:

```rust
pub struct RenderPassExecutionContext {
    pub pass_name: String,
    pub executor_id: RenderPassExecutorId,
    pub declared_queue: QueueLane,
    pub queue: QueueLane,
    pub flags: PassFlags,
    pub dependencies: Vec<RenderPassId>,
    pub resources: Vec<RenderGraphPassResourceAccess>,
}
```

Add fields only when they are neutral render graph or frame-host data.

- [ ] For Decision B, modify `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs` so plugins can register neutral executor functions without runtime importing plugin crates.

Allowed dependency direction:

```rust
// Runtime defines the function type and registry.
pub type RenderPassExecutorFn = fn(&RenderPassExecutionContext) -> Result<(), String>;
```

Plugin crates may provide functions matching the runtime-defined type. Runtime must not name plugin crates.

- [ ] For Decision B, modify `zircon_runtime/src/plugin/extension_registry/register.rs`, `zircon_runtime/src/plugin/extension_registry/access.rs`, and `zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs` only if executor functions must be registered through `RuntimeExtensionRegistry` rather than through `RenderFeatureDescriptor` metadata.

- [ ] For Decision B, update `zircon_plugins/virtual_geometry/runtime/src/lib.rs` and `zircon_plugins/hybrid_gi/runtime/src/lib.rs` to register executor functions alongside their existing render feature descriptors.

Accepted shape:

```rust
registry.register_render_feature(render_feature_descriptor())?;
registry.register_render_pass_executor("virtual-geometry.node-cluster-cull", virtual_geometry_node_cluster_cull_executor)
```

Use exact executor ids already declared by `render_feature_descriptor()`.

### Testing Stage

- [ ] If Decision A is chosen, record in the session note that no runtime hook files changed and skip runtime hook tests.

- [ ] If Decision B is chosen, run focused runtime graph/plugin registration checks.

Run:

```powershell
cargo test -p zircon_runtime --lib render_pass_executor --locked --offline -- --nocapture
```

Expected: graph executor registry tests pass. If the filter matches zero tests, run the narrower known graph/plugin extension filters discovered in `render_pass_executor_registry.rs` and `zircon_runtime/src/tests/plugin_extensions/extension_registry.rs`, then record the exact commands.

- [ ] If Decision B is chosen, rerun both plugin package checks.

Run:

```powershell
cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_virtual_geometry_runtime --lib --locked --offline
cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_hybrid_gi_runtime --lib --locked --offline
```

Expected: both checks complete.

### Exit Evidence

- Session note states Decision A or Decision B.
- Runtime still has no dependency on `zircon_plugins`.
- If hook files changed, runtime graph/plugin tests and plugin package checks pass or exact blockers are recorded.

## Milestone 5: Plugin Test Wiring And Package Validation

- Goal: promote the cutover from compile coherence to plugin package behavior coverage.
- In-scope behaviors: moved renderer test modules that now compile in plugin crates, plugin package tests, search gates, debug/correction loop.
- Dependencies: Milestones 2 and 3 checks; Milestone 4 decision complete.
- Lightweight checks: none; this milestone is the testing stage for the plugin cutover.

### Implementation Slices

- [ ] Add renderer-related test module declarations only for test sources whose old-owner imports are gone.

VG candidates in `zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/test_sources/`:

```text
virtual_geometry_gpu.rs
virtual_geometry_prepare_render.rs
virtual_geometry_args_source_authority.rs
virtual_geometry_execution_args_authority.rs
virtual_geometry_execution_stats.rs
virtual_geometry_node_and_cluster_cull_execution.rs
virtual_geometry_submission_authority.rs
virtual_geometry_submission_execution_order.rs
virtual_geometry_unified_indirect.rs
```

HGI candidates in `zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/test_sources/`:

```text
hybrid_gi_gpu.rs
hybrid_gi_gpu_scene_light_seed.rs
hybrid_gi_gpu_runtime_source.rs
hybrid_gi_resolve_render.rs
hybrid_gi_resolve_history.rs
hybrid_gi_resolve_dynamic_lights.rs
hybrid_gi_resolve_surface_cache.rs
hybrid_gi_scene_prepare_material_fixtures.rs
```

Wire a test source only when its imports use plugin-local owners and public neutral runtime contracts.

- [ ] Run the VG package test suite.

Run:

```powershell
cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_virtual_geometry_runtime --lib --locked --offline -- --nocapture
```

Expected: tests compile and run with zero failures. If failures occur, fix the lowest shared plugin type/import/runtime neutral-contract issue first, then rerun this command.

- [ ] Run the HGI package test suite.

Run:

```powershell
cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_hybrid_gi_runtime --lib --locked --offline -- --nocapture
```

Expected: tests compile and run with zero failures. If failures occur, fix the lowest shared plugin type/import/runtime neutral-contract issue first, then rerun this command.

- [ ] Re-run old-owner searches for both plugins.

Run:

```powershell
rg --line-number "crate::graphics::types|crate::graphics::scene::scene_renderer|pub\(in crate::graphics|crate::graphics::runtime|crate::graphics::tests" "zircon_plugins/virtual_geometry/runtime/src/virtual_geometry"
rg --line-number "crate::graphics::types|crate::graphics::scene::scene_renderer|pub\(in crate::graphics|crate::graphics::runtime|crate::graphics::tests" "zircon_plugins/hybrid_gi/runtime/src/hybrid_gi"
```

Expected: no live Rust code hits.

### Testing Stage

- [ ] Run `git diff --check` after code and docs are formatted.

Run:

```powershell
git diff --check
```

Expected: no whitespace errors. LF-to-CRLF warnings may appear in this repository and must be reported separately from whitespace errors.

### Exit Evidence

- VG plugin package tests pass or exact in-scope blockers are recorded.
- HGI plugin package tests pass or exact in-scope blockers are recorded.
- Old-owner searches have no live code hits.
- `git diff --check` has no whitespace errors.

## Milestone 6: Documentation And Session Closeout

- Goal: make docs and coordination state match the code migration and validation evidence.
- In-scope behaviors: VG/HGI docs, implementation file lists, plan/test evidence, session archive.
- Dependencies: Milestone 5 validation or exact unresolved blocker diagnostics.
- Lightweight checks: doc header inspection and `git diff --check`.

### Implementation Slices

- [ ] Update `docs/assets-and-rendering/virtual-geometry-nanite-foundation.md` frontmatter.

Ensure these plan sources are present:

```yaml
plan_sources:
  - docs/superpowers/specs/2026-05-01-plugin-renderer-hard-cutover-design.md
  - docs/superpowers/plans/2026-05-01-plugin-renderer-hard-cutover.md
  - .codex/plans/GI_VG 插件化激进迁移计划.md
  - .codex/plans/zircon_plugins 全量插件化收敛规划.md
  - .codex/plans/M5 Nanite-Like Virtual Geometry 全链收束计划.md
```

Ensure tests include the final VG commands from Milestone 5.

- [ ] Update `docs/assets-and-rendering/hybrid-gi-lumen-scene-representation.md` frontmatter.

Ensure these plan sources are present:

```yaml
plan_sources:
  - docs/superpowers/specs/2026-05-01-plugin-renderer-hard-cutover-design.md
  - docs/superpowers/plans/2026-05-01-plugin-renderer-hard-cutover.md
  - .codex/plans/GI_VG 插件化激进迁移计划.md
  - .codex/plans/zircon_plugins 全量插件化收敛规划.md
```

Ensure tests include the final HGI commands from Milestone 5.

- [ ] Add a documentation section to both docs describing the final ownership boundary.

Required wording content:

```text
The plugin runtime crate owns concrete renderer resources, readbacks, pass helpers, and feature-specific prepare/resolve DTOs. `zircon_runtime` only exposes neutral graphics/frame/render-graph contracts used by plugin registration and execution boundaries. Old `zircon_runtime::graphics::runtime::*` and `zircon_runtime::graphics::scene::scene_renderer::{hybrid_gi,virtual_geometry}` owner paths are not compatibility surfaces.
```

- [ ] Update `.codex/sessions/20260501-1850-plugin-renderer-hard-cutover.md` with final commands, pass/fail status, in-scope blockers, and coordination warnings.

- [ ] If the task completes, move the session note to `.codex/sessions/archive/20260501-1850-plugin-renderer-hard-cutover.md` and set `status: completed` in frontmatter.

### Testing Stage

- [ ] Run final whitespace check.

Run:

```powershell
git diff --check
```

Expected: no whitespace errors. Report LF-to-CRLF warnings separately if they appear.

### Exit Evidence

- Both rendering docs list touched code, implementation files, plan sources, and validation commands.
- Active session note is archived or contains exact blockers if implementation cannot finish.
- Final response states which validation passed, which validation was not run, and why.

## Final Acceptance Checklist

- [ ] No live plugin code references `crate::graphics::types`, `crate::graphics::scene::scene_renderer`, `pub(in crate::graphics...)`, `crate::graphics::runtime`, or `crate::graphics::tests`.
- [ ] VG plugin renderer code imports concrete VG DTOs from `crate::virtual_geometry::types` and neutral runtime contracts from `zircon_runtime::*`.
- [ ] HGI plugin renderer code imports concrete HGI DTOs from `crate::hybrid_gi::types` and neutral runtime contracts from `zircon_runtime::*`.
- [ ] `zircon_runtime` does not depend on `zircon_plugins`.
- [ ] No compatibility modules, shim traits, facade wrappers, bridge folders, or migration-only `pub use` exports are added.
- [ ] Plugin renderer root modules are structural and do not become implementation buckets.
- [ ] Scoped plugin checks and package tests pass, or exact in-scope blockers are recorded for the next correction loop.
- [ ] VG/HGI docs and archived session note contain implementation files, plan sources, tests, validation evidence, and remaining risks.
