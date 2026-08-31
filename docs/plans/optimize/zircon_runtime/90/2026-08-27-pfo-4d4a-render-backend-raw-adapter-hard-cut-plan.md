# PFO-4d4a RenderBackend Raw Adapter Hard Cut

Status: `source_implemented_static_checks_passed_dynamic_vulkan_cache_validation_pending`

Date: 2026-08-27

## Scope

Remove the outer `RenderBackend` raw `wgpu::Adapter` clone while preserving exact pipeline-cache
identity and the production `WgpuRenderDevice` native generation owner. Raw `Device/Queue` fields
remain until their independent resource and upload consumers are migrated.

## Review Findings

- production `WgpuRenderDevice` already owns the adapter for the complete device generation and
  validates it against `RenderDeviceProfile::adapter()` during ownership handoff;
- the outer raw adapter has only one production consumer after bootstrap: mesh pipeline-cache
  construction calls `get_info()`;
- WGPU 29.0.3's `util::pipeline_cache_key` supports Vulkan only and formats exactly
  `wgpu_pipeline_cache_vulkan_<vendor>_<device>`;
- neutral `RenderAdapterFacts` already carries backend, vendor id, device id, driver, name, and
  adapter class, so retaining another native adapter for this key is unnecessary.

## Hard-Cut Contract

1. `RuntimePipelineCache` accepts `RenderAdapterFacts` and emits the existing WGPU-compatible Vulkan
   key from neutral backend/vendor/device facts;
2. scene/core/mesh construction passes the immutable facts from `RenderDeviceProfile`;
3. `RenderBackend` no longer stores or exposes a raw adapter; bootstrap moves the native adapter only
   into `WgpuRenderDeviceContext`;
4. no compatibility field, native adapter accessor, or second identity representation is added;
5. source guards cover exact key compatibility and zero outer raw-adapter fields.

## Deferred Acceptance

Focused formatting and source contracts belong to this slice. Cargo, real Vulkan cache reuse,
Windows WGPU, screenshots, RenderDoc, profile, VRAM, and power remain part of the deferred product
acceptance lane; this owner cut does not claim a measured performance improvement.

## Source Implementation Evidence

- `RuntimePipelineCache`, `MeshPipelineCache`, and scene-core construction now consume
  `RenderAdapterFacts` borrowed from the immutable device profile;
- the neutral cache key preserves WGPU 29.0.3's Vulkan filename exactly and keeps all non-Vulkan
  backends disabled; a focused contract checks both branches;
- bootstrap moves the native adapter into `WgpuRenderDeviceContext`; outer `RenderBackend` raw
  adapter fields, `backend.adapter` consumers, `AdapterInfo` construction parameters, and adapter
  clones in the graphics backend/scene/pipeline scope are all zero;
- raw outer `Device/Queue` clones remain intentionally because their independent consumers are not
  all migrated; this slice does not misreport PFO-4d4 complete;
- focused `rustfmt --check` and scoped `git diff --check` pass. Cargo, Vulkan cache reuse, WGPU,
  screenshots, RenderDoc, profiling, VRAM, and power remain pending.
