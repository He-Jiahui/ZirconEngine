# Plugin Renderer Neutral Readback Execution Surface Design

## Summary

This spec defines the follow-up milestone after the VG/HGI plugin renderer hard-cutover. The goal is to turn the current descriptor-only plugin renderer integration into a neutral execution chain: plugin crates register render pass executors through runtime-owned contracts, plugin-owned readback/output sources publish neutral payloads back to the host, and deferred renderer tests become eligible for promotion without restoring old runtime graphics owner paths.

The accepted direction is a layered neutral execution surface. `zircon_runtime` owns render graph execution, plugin extension registration, neutral resource/readback/output slots, and frame metadata. `zircon_plugins/virtual_geometry/runtime` and `zircon_plugins/hybrid_gi/runtime` own concrete VG/HGI GPU resources, completion parts, prepare/resolve DTOs, renderer helpers, root output helpers, and readback interpretation. Runtime never imports plugin crates.

## Goals

- Add a runtime-owned executor registration surface that lets plugin crates register pass executors beside their existing render feature descriptors.
- Add neutral runtime readback/output slots that can carry renderer outputs across the host/plugin boundary without exposing plugin-private resource structs.
- Wire VG `root_state_readbacks` against neutral outputs instead of `SceneRenderer` internals.
- Wire HGI `post_process_sources` and `root_output_sources` against neutral HGI scene-prepare/output DTOs instead of runtime-private or stale scene-renderer assumptions.
- Promote deferred renderer tests only after their imports and fixtures use plugin-local owners plus public neutral runtime contracts.
- Update docs and coordination evidence with implementation files, validation commands, and remaining risks.

## Non-Goals

- Do not make `zircon_runtime` depend on `zircon_plugins`.
- Do not restore `zircon_runtime::graphics::runtime::{virtual_geometry,hybrid_gi}` or `zircon_runtime::graphics::scene::scene_renderer::{virtual_geometry,hybrid_gi}` owner paths.
- Do not add compatibility modules, bridge folders, shim traits, alias re-exports, or migration-only facades.
- Do not publicize runtime-private `MeshDraw`, `ResourceStreamer`, `MaterialCaptureSeed`, or plugin concrete GPU resource/completion types.
- Do not turn `ViewportRenderFrame` into an HGI or VG state carrier.
- Do not edit active Runtime UI, editor UI, `zircon_runtime_interface`, or editor cutover files unless a later approved plan explicitly scopes that work.
- Do not attempt full `wgpu` command encoding through plugin executors in the first slice unless neutral frame/resource handles already exist and compile evidence proves they are required.

## Approved Approach

Use a staged runtime-owned neutral execution surface.

The first layer adds executor registration to `RuntimeExtensionRegistry`. Plugin crates register executor ids and functions using runtime-defined types. Runtime stores ids and callbacks as neutral data and registers them into `RenderPassExecutorRegistry` during graph execution setup.

The second layer adds neutral readback/output DTOs. These DTOs carry stable payloads such as counts, ids, packed records, atlas samples, probe ids, trace-region ids, and neutral resource/view handles. They do not contain plugin-owned `VirtualGeometryGpuResources`, `HybridGiGpuResources`, readback completion parts, or prepare/resolve frames.

The third layer rewires the deferred plugin renderer folders to consume the neutral mailbox. VG root-state readbacks stop acting like inherent `SceneRenderer` methods. HGI post-process and root-output sources stop assuming HGI-specific fields on runtime frame state. Only after those boundaries compile should deferred tests move into the plugin crates.

## Rejected Alternatives

### Full GPU Executor Context Immediately

This would extend `RenderPassExecutionContext` with device, queue, encoder, frame target, bind groups, and plugin output access in one step. It is too risky for the current codebase because the existing context only carries graph metadata, and jumping straight to GPU command encoding would likely force runtime to know feature-specific state.

### Readback-Only With No Executor Registration

This would keep current descriptor/no-op executor behavior and only add neutral output DTOs. It is smaller, but it does not satisfy the selected full execution-chain scope because plugin pass ids would remain validation metadata rather than callable execution hooks.

## Architecture

`zircon_runtime::graphics::scene::scene_renderer::graph_execution` remains the host execution authority. Current evidence shows `RenderPassExecutorFn` is `fn(&RenderPassExecutionContext) -> Result<(), String>`, `RenderPassExecutorRegistry` stores functions by `RenderPassExecutorId`, and `RenderPassExecutionContext` contains pass metadata, queue, flags, dependencies, and resource access declarations. The design preserves that owner and expands it only with neutral host data when required.

`zircon_runtime::plugin::extension_registry` becomes the plugin registration bridge for executors. It already stores render feature descriptors and VG runtime providers. The new surface should add a small executor registration type beside those existing vectors, plus register/access APIs and catalog merge behavior. The shape should be direct and neutral:

```rust
pub struct RenderPassExecutorRegistration {
    pub executor_id: RenderPassExecutorId,
    pub executor: RenderPassExecutorFn,
}
```

The exact type location may be under graph execution or plugin extension registration, but the dependency direction must stay runtime-owned. Plugin crates consume this type; runtime never names plugin crates.

`zircon_runtime::graphics` owns neutral renderer output/readback mailbox types. The mailbox may be part of the existing scene-renderer advanced outputs/readbacks area or a new focused subtree if that avoids making a root file an implementation bucket. It should be split by responsibility: declaration files for DTOs, accessors for read-only views, mutation/update files for host writes, and tests under the appropriate runtime test tree.

VG/HGI plugin runtime crates own concrete conversion and interpretation. A plugin executor may fill or consume neutral host slots, but concrete conversion from plugin GPU readback structures into neutral payloads stays in plugin folders. If a payload becomes reusable across runtime and plugins, place only the neutral declaration in runtime and keep feature-specific construction helpers plugin-local.

## Data Flow

Plugin registration flows through existing plugin extension paths:

1. VG/HGI plugin `register()` creates a `RuntimeExtensionRegistry`.
2. The plugin registers its `RenderFeatureDescriptor` as it does today.
3. The plugin registers one executor function per executable pass id that should no longer be a no-op.
4. Runtime plugin catalog merge preserves both descriptors and executor registrations.
5. Runtime graph setup first registers built-in/no-op executors for declared descriptors, then overwrites or augments with explicit plugin executor registrations for matching ids.
6. Pipeline validation fails if an executable pass references an executor id that has no registered function.

Readback/output flow is deliberately mailbox-oriented:

1. Runtime graph execution provides pass metadata and neutral resource declarations.
2. Plugin executor or plugin-owned helper interprets its own resources and writes neutral output data to host-visible slots.
3. Runtime post-process/readback consumers request neutral snapshots from the mailbox.
4. Plugin test fixtures assert concrete behavior through plugin-local conversion helpers plus neutral host DTOs.

The first implementation slice may keep executor functions metadata-only if no neutral GPU command context exists yet. That is acceptable as long as the registration path is real, tested, and ready to carry richer neutral context later.

## Neutral DTO Requirements

VG neutral readback/output DTOs should expose these categories, using stable primitive/neutral records rather than plugin structs:

- Page table entries, completed page assignments, and page replacements.
- Selected cluster source, count, and entries.
- Visbuffer64 clear value, source, count, and entries.
- Hardware rasterization source, count, and packed records.
- Cull launch/work item/traversal/page-request payloads needed by promoted root-state readback tests.

HGI neutral readback/output DTOs should expose these categories:

- Cache entries, completed probe ids, and completed trace-region ids.
- Probe irradiance RGB and probe RT lighting RGB samples.
- Scene-prepare snapshot data: occupied atlas/capture slots, atlas/capture samples, voxel clipmap ids, voxel samples, occupancy, cell data, texture extents, and layer counts.

The DTOs should prefer small record structs over `Vec<Vec<_>>` or unstructured string maps. Use explicit field names and deterministic ordering so tests can compare payloads without relying on plugin-internal memory layout.

## Runtime Execution Context Evolution

The first accepted executor registration may keep the current context shape:

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

If later compile evidence proves executors need more than metadata, add only neutral host data. Acceptable additions are resource handles, frame target handles, pass resource views, frame id, viewport dimensions, and read/write access to neutral output mailbox slots. Unacceptable additions are `HybridGiGpuResources`, `VirtualGeometryGpuResources`, concrete readback completion parts, plugin prepare/resolve frames, or editor-owned state.

## Module Boundaries

Runtime root and `mod.rs` files stay structural. New runtime implementation should use focused folders rather than adding behavior to root wiring files. Likely runtime destinations are:

- `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/` for executor registry/context types and registration bridging into pass execution.
- `zircon_runtime/src/plugin/extension_registry/` for extension storage, registration, access, and catalog merge changes.
- `zircon_runtime/src/graphics/scene/scene_renderer/core/.../advanced_plugin_readbacks/` or a new focused child folder for neutral readback mailbox state.

Plugin renderer roots also stay structural:

- `zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/mod.rs` should declare child modules and expose only necessary plugin-local items.
- `zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/mod.rs` should declare child modules and expose only necessary plugin-local items.
- `root_state_readbacks`, `post_process_sources`, and `root_output_sources` should contain behavior files that consume plugin-local types plus neutral runtime DTOs directly.

## Error Handling

Executor registration should reject duplicate executor ids only when the duplicate would be ambiguous inside one registry merge. If overriding no-op registrations is intentional, that behavior must be explicit and tested.

Executor execution errors may remain `Result<(), String>` in the first slice to match current `RenderPassExecutorFn`. New structured error types should be added only if multiple runtime/plugin call sites need stable semantic matching. Error messages should include executor id and pass name for graph diagnosis.

Readback/output DTO conversion failures should be plugin-local unless they cross the runtime host boundary. Runtime mailbox access should return explicit missing-output errors rather than panicking when a pass did not produce optional output.

## Tests And Validation

Use milestone-first validation. Implementation slices may add tests, but Cargo build/test commands belong in milestone testing stages unless an early syntax check is required.

Runtime executor registration tests should cover:

- Registering executor ids through `RuntimeExtensionRegistry`.
- Catalog merge preserving executor registrations.
- Explicit plugin executor registration replacing or augmenting descriptor-created no-op executors.
- Pipeline validation failing for missing executor ids.
- Duplicate executor id behavior.

Runtime neutral mailbox tests should cover:

- Empty mailbox access returns explicit absence, not panic.
- VG neutral payload roundtrip preserves deterministic ordering.
- HGI scene-prepare payload roundtrip preserves extents, layers, slots, and sample data.
- Optional outputs remain optional and do not force VG/HGI coupling into base frame state.

Plugin tests should be promoted only after their source files no longer depend on old runtime owner paths. Candidate deferred areas are:

- VG `root_state_readbacks` and related test sources for traversal, page requests, selected clusters, visbuffer64, and hardware rasterization records.
- HGI `post_process_sources` and `root_output_sources` test sources for scene-prepare resources, resolve output, probe/trace-region completions, and surface-cache payloads.

Expected scoped validation commands after implementation are:

```powershell
cargo test -p zircon_runtime --lib render_pass_executor --locked --offline -- --nocapture
cargo test -p zircon_runtime --lib advanced_plugin_readbacks --locked --offline -- --nocapture
cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_virtual_geometry_runtime --lib --locked --offline -- --nocapture
cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_hybrid_gi_runtime --lib --locked --offline -- --nocapture
```

If filters match zero tests, the implementation plan must name the exact discovered filters after inspecting the test modules.

Search gates remain mandatory:

```powershell
rg --line-number "crate::graphics::types|crate::graphics::scene::scene_renderer|pub\(in crate::graphics|crate::graphics::runtime|crate::graphics::tests" "zircon_plugins/virtual_geometry/runtime/src/virtual_geometry"
rg --line-number "crate::graphics::types|crate::graphics::scene::scene_renderer|pub\(in crate::graphics|crate::graphics::runtime|crate::graphics::tests" "zircon_plugins/hybrid_gi/runtime/src/hybrid_gi"
```

Expected result: no live Rust code hits.

## Reference Evidence

Current repository precedent:

- `zircon_runtime/src/render_graph/builder.rs` owns neutral pass/resource graph construction with executor ids and neutral resource handles.
- `zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs` compiles feature-provided pass descriptors into neutral graph passes and resource access declarations.
- `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs` already defines host-owned executor lookup and validation.

External reference precedent:

- `dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraph.cs` and `RenderGraphBuilders.cs` show a host-owned render graph where features add passes, declare neutral texture/resource handles, and provide pass callbacks through render functions.

The design intentionally follows the common pattern: the host graph owns scheduling and neutral handles, while feature code provides pass behavior and keeps feature-private state outside the host graph contract.

## Documentation

Implementation must update existing functional docs rather than creating a random changelog bucket:

- `docs/assets-and-rendering/virtual-geometry-nanite-foundation.md`
- `docs/assets-and-rendering/hybrid-gi-lumen-scene-representation.md`

If new runtime module-level docs are required, create them under a source-path mirror only when no existing functional doc owns the behavior. All affected docs must keep machine-readable `related_code`, `implementation_files`, `plan_sources`, and `tests` headers current.

## Acceptance Criteria

- Plugin crates can register pass executor functions through runtime-owned APIs.
- Runtime graph execution can validate and dispatch registered executor ids without importing plugin crates.
- Neutral readback/output mailbox DTOs exist for the VG/HGI data needed by deferred root-state, post-process, and root-output sources.
- VG `root_state_readbacks` no longer assumes inherent methods on runtime `SceneRenderer` or access to runtime-private advanced output fields.
- HGI `post_process_sources` and `root_output_sources` no longer assume HGI-specific fields on `ViewportRenderFrame` or runtime-private scene-renderer internals.
- `ViewportRenderFrame` remains neutral and does not gain concrete VG/HGI state fields.
- Deferred renderer tests are promoted only after old-owner path gates are clean.
- No compatibility shims, bridge modules, alias re-exports, or old owner paths are introduced.
- Scoped runtime/plugin validation passes, or exact in-scope blockers are documented with command output summaries.
