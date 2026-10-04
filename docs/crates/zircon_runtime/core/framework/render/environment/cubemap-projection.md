---
related_code:
  - zircon_runtime/src/core/framework/render/environment/cubemap_projection.rs
  - zircon_runtime/src/core/framework/render/environment/mod.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/tests/runtime_environment_cubemap_projection_contract.rs
  - dev/cmft/src/cmft/cubemaputils.h
  - dev/cmft/src/cmft/cubemapfilter.cpp
  - dev/UnrealEngine/Engine/Shaders/Private/ReflectionEnvironmentShaders.usf
implementation_files:
  - zircon_runtime/src/core/framework/render/environment/cubemap_projection.rs
  - zircon_runtime/src/core/framework/render/environment/mod.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/tests/runtime_environment_cubemap_projection_contract.rs
plan_sources:
  - user: 2026-07-05 cubemap skybox/reflection mosaic correction and cmft/Unreal mip design request
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
tests:
  - zircon_runtime/src/core/framework/render/environment/cubemap_projection.rs
  - zircon_runtime/tests/runtime_environment_cubemap_projection_contract.rs
doc_type: module-detail
---

# Cubemap Projection

## Purpose

`cubemap_projection.rs` owns the CPU-side golden math for the environment IBL cutover from sampled equirectangular buffers to real cubemap textures.

The module fixes three pieces of behavior that must be identical between Rust tests and the planned WGSL passes:

- cubemap face order and face-local UV axes,
- cubemap texel direction to equirectangular UV mapping,
- exact cubemap texel solid angle for filtering and SH projection weights.

This module does not allocate GPU resources by itself. It is the reference layer used by the EC-M1 source-cubemap bridge and by future `equirect_to_cube.wgsl`, GGX prefilter tests, SH9 tests, and seam checks.

## Related Files

The source module is [cubemap_projection.rs](/E:/Git/ZirconEngine/zircon_runtime/src/core/framework/render/environment/cubemap_projection.rs). The environment module re-exports the helpers through [mod.rs](/E:/Git/ZirconEngine/zircon_runtime/src/core/framework/render/environment/mod.rs), and the public render facade forwards them through [mod.rs](/E:/Git/ZirconEngine/zircon_runtime/src/core/framework/render/mod.rs).

The formulas are intentionally aligned with `dev/cmft/src/cmft/cubemaputils.h` and Unreal's `ReflectionEnvironmentShaders.usf`. cmft supplies the face axis table, lat-long projection, and Rory Driscoll solid-angle formula. Unreal confirms the same face order and uses the cubemap direction math when copying and filtering reflection captures.

## Behavior Model

`CubemapFace` uses the six-face order `+X, -X, +Y, -Y, +Z, -Z`. This matches cmft's face enum, Unreal's reflection environment shader, and wgpu's normal cube-array layer order for cubemap uploads. `CubemapFace::index()` and `CubemapFace::from_index()` keep tests and upload code from duplicating raw face numbers.

`cubemap_scaled_uv_for_texel(x, y, face_size)` maps texel centers to `[-1, 1]`:

```text
scaled = ((texel + 0.5) / face_size) * 2 - 1
```

`cubemap_direction_from_scaled_uv(face, scaled_uv)` applies the cmft face axis table and normalizes the result. `cubemap_texel_direction(...)` combines those two steps for per-texel tests and future CPU reference generation.

`equirect_uv_from_direction(direction)` normalizes the incoming direction, then uses the cmft lat-long convention:

```text
u = (pi + atan2(x, z)) / (2*pi)
v = acos(y) / pi
```

This makes `+Z` land at `u=0.5`, `+X` at `u=0.75`, `-X` at `u=0.25`, and the poles at `v=0/1`. A zero-length input falls back to `+Z` to avoid NaNs in defensive tests.

`cubemap_face_scaled_uv_from_direction(direction)` performs the reverse lookup from a direction to the major cube face plus scaled face UV. Source-cubemap mip filtering uses this path to bilinearly sample mip 0 by direction.

`cubemap_texel_solid_angle(...)` uses cmft's exact area-element formula:

```text
area(x, y) = atan2(x*y, sqrt(x*x + y*y + 1))
solid = area(x1,y1) - area(x0,y1) - area(x1,y0) + area(x0,y0)
```

Summing all six faces over a complete cubemap should return `4*pi` within floating-point tolerance. This is the weight model required by both cmft-style CPU reference filters and SH9 projection.

## Design And Rationale

The previous sampled equirectangular bridge carries its own direction mapping for equirect texels. That bridge is temporary and not a suitable source of truth for cubemap generation. The final plan 06 EC-M1 path needs a stable cubemap projection contract before WGSL and GPU textures are wired in.

Keeping the projection math in `core/framework/render/environment` avoids coupling tests to wgpu or scene-renderer implementation details. It also keeps the module usable by asset/importer tests when plan 13 `.zcube` support starts consuming the same face order.

The module adopts cmft's face axes and solid-angle formula rather than inventing a local convention. Unreal uses the same face ordering and rough direction mapping in its reflection capture shaders, so this choice keeps the CPU reference compatible with both local references the plan depends on.

## Edge Cases And Constraints

`face_size` is clamped to at least 1. Out-of-range texel coordinates are clamped to the last valid texel so defensive callers do not accidentally create invalid directions. Production equirect-to-cube and PMREM code should still dispatch only valid texel coordinates.

No edge-warp fixup is implemented here. cmft's warp mode targets old non-seamless cubemap filtering paths. The Zircon runtime target is wgpu-backed modern APIs, so seam quality should be validated by shared-edge luminance tests instead of by baking warped coordinates into the projection contract.

The helper uses `Real` precision, currently matching the runtime math layer. GPU passes may use `f32`; tests should use tolerances rather than byte-exact equality for direction and solid-angle sums.

## Test Coverage

The local tests in [cubemap_projection.rs](/E:/Git/ZirconEngine/zircon_runtime/src/core/framework/render/environment/cubemap_projection.rs) and the public API contract in [runtime_environment_cubemap_projection_contract.rs](/E:/Git/ZirconEngine/zircon_runtime/tests/runtime_environment_cubemap_projection_contract.rs) cover:

- cardinal face directions for all six cubemap faces,
- the cmft lat-long UV convention for major axes,
- solid-angle sum across a 16x16x6 cubemap, expected to equal `4*pi`,
- equirect height to cubemap face-size conversion.

The focused validation for this slice is:

```powershell
cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-cubemap-projection-check-0705 --message-format short --color never
cargo test -p zircon_runtime --test runtime_environment_cubemap_projection_contract --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-cubemap-projection-check-0705 --message-format short --color never -- --nocapture --test-threads=1
```

The first command passed after the public facade export was added. The second passed 4/4 public contract tests. Broader shader/IBL tests remain part of the EC-M1/EC-M2 milestone testing stages.

## Open Issues

This module is still only the projection foundation. EC-M1d now uses it for the CPU source-cubemap bridge and `texture_cube` HDRI screenshot path. Remaining follow-up is the hard deletion of the sampled equirectangular bridge and the EC-M2 production IBL chain: RGBA16F GGX PMREM, SH9, BRDF LUT, derived artifacts, and seam/roughness quantitative tests.
