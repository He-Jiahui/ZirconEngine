---
related_code:
  - zircon_runtime/src/core/framework/render/environment/source_irradiance_cubemap.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap.rs
  - zircon_runtime/src/core/framework/render/environment/cubemap_projection.rs
  - zircon_runtime/src/core/framework/render/environment/mod.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/tests/runtime_environment_source_irradiance_cubemap_contract.rs
  - zircon_runtime/tests/runtime_environment_source_cubemap_contract.rs
  - dev/cmft/src/cmft/cubemapfilter.cpp
  - dev/cmftStudio/src/shaders/fs_mesh.shdr
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ReflectionEnvironmentDiffuseIrradiance.cpp
  - dev/UnrealEngine/Engine/Shaders/Private/ReflectionEnvironmentShaders.usf
implementation_files:
  - zircon_runtime/src/core/framework/render/environment/source_irradiance_cubemap.rs
  - zircon_runtime/src/core/framework/render/environment/mod.rs
  - zircon_runtime/src/core/framework/render/mod.rs
plan_sources:
  - user: 2026-07-06 real HDRI cubemap/PMREM/IEM continuation
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
tests:
  - zircon_runtime/tests/runtime_environment_source_irradiance_cubemap_contract.rs
doc_type: module-detail
---

# Source Irradiance Cubemap

## Purpose

`source_irradiance_cubemap.rs` is the staged CPU bridge for the optional IEM path described by Plan 06. It keeps diffuse irradiance as a real 32x32x6 cubemap instead of only SH9 coefficients, so future GPU/offline bake work can validate the cmft/cmftStudio PMREM/IEM consumer split without adding more responsibility to the already-large `source_cubemap.rs` owner.

The current runtime shader still consumes SH9 for source-cubemap diffuse lighting. This module is a CPU reference and contract surface for the next bake/upload slice: source cube mip0 remains the skybox, specular PMREM remains the rough reflection source, and this IEM cube is the low-frequency diffuse cubemap candidate.

## Algorithm

`build_source_cubemap_irradiance_cube(...)` builds a fixed 32x32 face-major cube. For each output texel, it computes the cubemap direction and performs direct cosine convolution over the source mip selected by `source_cubemap_irradiance_mip_level(...)`. The source samples use the same exact cubemap texel solid-angle weighting as the SH9 bridge.

The output is normalized by the accumulated cosine-weighted solid angle, so a constant environment remains constant in the IEM. That makes the CPU bridge comparable to the current SH9 contract while avoiding SH band truncation for future diffuse cubemap validation.

Sampling uses `source_cubemap_sample_irradiance_cube(...)`, which performs bilinear cube sampling. Taps that cross a face edge are projected back through the shared cubemap direction helpers and resolved on the neighboring face, matching the source/PMREM cross-face discipline.

## Public Contract

- `SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE` is fixed at 32.
- `SourceCubemapIrradianceCube::texels()` stores six faces in `CubemapFace::ALL` order.
- `SourceCubemapIrradianceCube::texel(...)` clamps public coordinates to the face extent.
- `build_source_cubemap_irradiance_cube(...)` reads the source mip pyramid, not the specular PMREM chain.
- `source_cubemap_sample_irradiance_cube(...)` accepts arbitrary normals and falls back to +Z for zero-length input.

## Test Coverage

`runtime_environment_source_irradiance_cubemap_contract.rs` covers two acceptance points:

- constant HDR diffuse environments are preserved across every IEM texel;
- a low-frequency vertical-gradient environment samples close to the SH9 diffuse bridge at 32 probe directions.

Focused evidence from this slice:

```powershell
rustfmt --edition 2021 zircon_runtime\src\core\framework\render\environment\source_irradiance_cubemap.rs zircon_runtime\src\core\framework\render\environment\mod.rs zircon_runtime\src\core\framework\render\mod.rs zircon_runtime\tests\runtime_environment_source_irradiance_cubemap_contract.rs
$env:CARGO_INCREMENTAL='0'; cargo test -p zircon_runtime --test runtime_environment_source_irradiance_cubemap_contract --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-hdri-iem-contract-0706 --message-format short --color never -- --nocapture --test-threads=1
```

The first focused Cargo run correctly failed because the root render facade did not yet re-export the new IEM symbols. After adding the facade exports, the same command passed 2/2. A later Cargo wrapper recheck timed out at the tool boundary after 904s and is not counted as pass evidence; the already-built test executable under `E:\cargo-targets\zircon-hdri-iem-contract-0706\debug\deps` was then run directly and passed 2/2 in 15.85s. This slice generated no screenshot.

## Pending Work

The production IEM chain is still open: GPU/offline bake artifact generation, derived-cache selection, runtime cube upload/binding, shader-side diffuse IEM sampling behind an engine option, and screenshot-level diffuse comparison against SH9 and direct source-cubemap references remain later Plan 06 / Plan 11 work.
