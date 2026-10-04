# Order-Independent Transparency (OIT)

## Ownership

- Framework contract: `core/framework/render/advanced_lighting/oit.rs`
- Render-graph feature: `zircon_plugins/rendering/features/oit/runtime`
- WGPU buffers and resolve: `graphics/scene/scene_renderer/advanced_lighting/oit_buffers`
- Graph resource sizing: `graphics/pipeline/render_pipeline_asset/resource_descriptors.rs`

The feature is optional and disabled by default. A camera activates it by supplying
`AdvancedLightingExtract::oit`. Installing the plugin without camera settings leaves the sorted
`transparent-mesh` pass unchanged.

## Frame Flow

1. The runtime profile checks fragment-writable storage and the storage-buffer binding limit.
2. Unsupported adapters disable the `oit` plugin feature and retain sorted transparency.
3. `oit.fragment_store` owns `oit.layers` and `oit.counts` and clears both before transparent draws.
4. Transparent fragments reserve a per-pixel slot with an atomic counter and store packed RGBA8
   plus WGPU normalized-depth `f32` bits.
5. `oit.resolve` keeps the nearest bounded layer set in ascending near-to-far depth order, merges overflow, and
   premultiplied-alpha composites into `scene-color`.

The two graph buffers are sized from the effective camera render size. Layer capacity is
`pixel_count * ceil(fragments_per_pixel_average)`. Each layer is 8 bytes and each per-pixel count is
4 bytes. Arithmetic saturates at `u64` limits.

## Viewport Contract

The WGPU uniform is exactly eight 32-bit scalar fields (32 bytes). It carries physical viewport
origin and local dimensions. Fragment-store and resolve shaders subtract the physical origin before
indexing local OIT buffers, while the resolve render pass applies the physical viewport and scissor.
This keeps split-screen and camera-stack viewports isolated.

## Capability Fallback

`RenderBackendCaps::supports_fragment_writable_storage` comes from
`wgpu::DownlevelFlags::FRAGMENT_WRITABLE_STORAGE`. The framework summary combines it with storage
buffer support and `max_storage_buffers_per_shader_stage`. `oit_support` returns the stable fallback
diagnostic used by tests and tooling; runtime profile compilation disables the plugin before graph
replacement when the capability gate fails.

## Current Completion State

Implemented: framework settings and CPU reference, capability gate, camera activation gate, graph
pass replacement contract, dual-buffer sizing, WGPU clear/resolve, bounded nearest-layer sorting,
overflow merge, viewport-safe uniform layout, transparent mesh source specialization, and
Transparent3d sprite fragment storage. Ordinary Forward shader sources do not include OIT bindings;
only the replacement pass requests the dedicated `fs_oit` variant.

The plan-owned product test renders three crossing transparent planes through sorted and OIT paths,
verifies feature-off byte parity, and exports a side-by-side PNG plus pass timing and buffer evidence.
Current-source Windows WGPU acceptance passed on 2026-07-12: both product gates and the ignored PNG
exporter passed, the image was inspected, and the report recorded 12,954 changed pixels with mean
absolute RGB error 2.2195. This closes AF-M4 Slice 1 only; planar reflection and SSS remain open in
AF-M4.
