---
related_code:
  - zircon_plugins/rendering/features/reflection_probes/runtime/src/capture/mod.rs
  - zircon_plugins/rendering/features/reflection_probes/runtime/src/capture/request.rs
  - zircon_plugins/rendering/features/reflection_probes/runtime/src/capture/face_view.rs
  - zircon_plugins/rendering/features/reflection_probes/runtime/src/capture/execute.rs
  - zircon_plugins/rendering/features/reflection_probes/runtime/src/capture/consume.rs
  - zircon_plugins/rendering/features/reflection_probes/runtime/src/lib.rs
  - zircon_plugins/rendering/features/reflection_probes/runtime/src/plugin.rs
  - zircon_plugins/rendering/features/reflection_probes/editor/src/capture/trigger.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_hdr_capture.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap/pmrem.rs
  - zircon_runtime/src/asset/artifact/ibl_source_cubemap_staging.rs
implementation_files:
  - zircon_plugins/rendering/features/reflection_probes/runtime/src/capture/request.rs
  - zircon_plugins/rendering/features/reflection_probes/runtime/src/capture/face_view.rs
  - zircon_plugins/rendering/features/reflection_probes/runtime/src/capture/execute.rs
  - zircon_plugins/rendering/features/reflection_probes/editor/src/capture/trigger.rs
plan_sources:
  - user: 2026-07-11 continue reflection-probe capture, cmft filtering and visual verification
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
tests:
  - zircon_plugins/rendering/features/reflection_probes/runtime/src/capture/request.rs
  - zircon_plugins/rendering/features/reflection_probes/runtime/src/capture/face_view.rs
  - zircon_plugins/rendering/features/reflection_probes/runtime/src/capture/execute.rs
  - zircon_plugins/rendering/features/reflection_probes/editor/src/capture/trigger.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap/tests.rs
doc_type: module-detail
---

# Reflection Probe Capture

## Purpose

The reflection-probes rendering feature owns the authoring boundary that converts one scene position into a persistent HDR cubemap and its derived IBL data. The editor sends a serialized command; the runtime renders the six faces, converts them to the canonical cmft face layout, reuses the shared source-mip/PMREM/SH9 implementation, and writes the existing `.zcube` plus `.zribl` staged bundle.

## Request Contract

`ReflectionProbeCaptureRequest` is a versioned JSON DTO. It carries probe identity, output asset URI, world position, near/far planes, power-of-two face size, prefilter quality and source revision. Validation rejects unknown schema versions, empty identities, non-finite positions, invalid clip ranges, and face sizes outside the runtime cubemap limits.

Quality maps to the planned cmft-style sample budgets:

| Quality | Middle mips | Last two mips |
|---|---:|---:|
| Fast | 32 | 64 |
| Normal | 64 | 128 |
| High | 128 | 256 |

Mip zero remains the unfiltered HDR capture. Every derived mip integrates the source cubemap using cosine-power weighting and the existing cross-face sampling path; no ordinary enlarged low-resolution mip is substituted for PMREM.

## Face Orientation

Faces use the canonical order `+X, -X, +Y, -Y, +Z, -Z`. The renderer uses the same 90-degree camera axes as point-light cube capture, then converts the rendered image to the cmft storage basis:

- `+X`, `-X`, `+Z`, `-Z`: horizontal flip.
- `+Y`, `-Y`: vertical flip.

The face-view test derives the renderer's right/down axes from the actual camera transform, applies the declared storage transform, and compares both axes with `cubemap_direction_from_scaled_uv`. This guards the prior up/down and front/back reflection inversion.

## Execution Flow

1. The editor command stores only validated request JSON.
2. Runtime clones the scene for each face, replaces the camera, removes overlays/debug payloads, and calls `SceneRenderer::render_scene_color_hdr`.
3. The face image is transformed into cmft storage orientation and appended face-major.
4. A BLAKE3-derived 128-bit source hash is computed from face size and exact HDR texel bits.
5. The shared source-cubemap builder creates angular source mips, PMREM and SH9.
6. `IblSourceCubemapStagingStore` writes the `.zcube` source and matching `.zribl` derived artifact under one `IblBakeArtifactRequest` identity.

## Captured Asset Consumption

`register_captured_reflection_probe` closes the persistence-to-runtime boundary. It validates the capture request and serialized placement, decodes the current `.zribl` with the exact capture bake request, creates a linear RGBA16F PMREM `TextureAsset`, registers that texture in `ProjectAssetManager`, and returns `ReflectionProbeData` whose `baked_cubemap` points to the registered resource. The placement DTO carries influence shape, projection bounds, rotation, intensity, priority, layer mask and bake timing; editor commands store both capture and placement as validated JSON.

The old no-op `reflection-probe-composite` graph pass and executor were removed. Probe shading is already integrated through the shared environment bindings; capture work is invoked only when the editor submits a command, so a feature-disabled graph has no probe capture pass.

## Current Validation State

Source formatting and whitespace checks are clean, and the root runtime library check passed after the HDR split. The runtime plugin suite passes 5/5 and the editor plugin suite passes 1/1 with `zircon_runtime/default`; the first attempts exposed the active Physics hard-cut dependency window, then the same locked commands passed after that owner moved the remaining import.

The runtime plugin suite passes 6/6 plus one ignored WGPU product acceptance. The product test passes 1/1, executes six real WGPU HDR renders, registers the resulting PMREM texture, and verifies the returned `ReflectionProbeData` references the captured resource at the requested world position. It reports source hash `[7cf9b4b6, 1310f030, 78c9e17a, c8161e72]` and writes non-empty `.zcube`/`.zribl` below `docs/tests/runtime/shader/reflection_probe_capture_product_20260711`. The decoded source/PMREM contact sheet and numeric report prove seven valid mips; PMREM luma standard deviation decreases from `0.123635` at mip 0 to `0.000000` at mip 6 while mean luma stays near `0.267`.

Capture render targets now come from the renderer's shared `TransientResourcePool`. The six-face acceptance asserts the sixth face creates zero textures and reuses all three compatible HDR/final/depth backings, with three entries retained after release. The editor command/placement tests pass 2/2. A full editor-host button/panel surface remains outside this rendering plugin crate; the plugin now provides the complete serialized operation payload and execute-and-register trigger used by that host.
