# Skybox IBL PBR Matrix Design

## Goal

Implement the Plan 11 EL-M1 path needed for a real PBR environment-reflection proof:

- replace the preview-only sky path with a formal skybox/environment extract;
- provide an engine-owned environment map path that PBR shading can sample for image-based reflections;
- generate an 8x8 PBR matrix screenshot under `docs/tests/runtime/shader`, with metallic increasing from left to right and smoothness increasing from top to bottom.

The matrix must prove engine rendering behavior, not an offline illustration. The visible result should show sky/environment color in metallic cells and progressively sharper reflection as smoothness approaches 1.

## Scope

This design covers Plan 11 EL-M1 only:

- `SkyboxSettings` / `EnvironmentExtract` contracts in `core/framework/render`;
- renderer-side skybox/environment support under `graphics/scene/scene_renderer/environment`;
- a standard PBR environment sampling include consumed by the existing material shader path;
- a product screenshot test owner for the 8x8 matrix.

Reflection probes, probe capture, lightmaps, light probe grids, fog, and full cubemap asset import remain future Plan 11 / Plan 13 work. The EL-M1 data shapes should leave those later paths with explicit extension points, but this slice must not implement temporary probe or lightmap behavior.

## Architecture

The contract layer gains a small environment module:

- `SkyboxSettings` describes `None`, procedural gradient, and engine-generated environment map modes.
- `ProceduralSkyParams` owns horizon, zenith, ground, intensity, and rotation inputs.
- `EnvironmentExtract` becomes the frame-level source of truth for skybox and IBL inputs.
- A bake key records content-affecting parameters only; intensity and rotation remain sampling-time values so changing them does not force a rebake.

The renderer layer owns all WGPU details:

- a folder-backed `environment` module holds skybox drawing, generated cubemap data, roughness-to-mip mapping, and the product test helpers;
- preview-sky behavior is absorbed by the skybox path instead of remaining a parallel executor;
- the first implementation may use a deterministic procedural cubemap generated from `ProceduralSkyParams`, because Plan 13 cubemap assets are not yet complete;
- the PBR shader samples an environment function with roughness derived from material smoothness.

The shader-facing contract is a single include-style interface:

- `zr_environment_specular(reflect_dir, roughness, metallic, base_color)` returns the environment contribution used by standard/fallback PBR;
- `roughness = 1.0 - smoothness`;
- metallic controls how strongly base color tints the reflected environment.

## Product Test And Artifact

Add a new product test owner instead of expanding `project_render/project_scenes.rs`:

- `zircon_runtime/src/graphics/tests/render_product_environment.rs`, or a folder-backed equivalent if it approaches budget;
- one focused ignored export test writes `docs/tests/runtime/shader/runtime_pbr_metallic_smoothness_matrix_20260704.png`;
- one non-ignored test verifies the frame is non-background, contains 64 visible cells, and has monotonic image signals across the two axes.

The screenshot layout is:

- columns: metallic `0.0, 1/7, 2/7, ... 1.0`;
- rows: smoothness `0.0, 1/7, 2/7, ... 1.0`;
- each cell renders a sphere or bevelled tile with identical lighting, material base color, and camera framing;
- the skybox remains visible behind the matrix so environment context is inspectable.

## Validation

Implementation evidence should include:

- scoped rustfmt for touched Rust files;
- focused environment unit tests for bake key stability and roughness/smoothness mapping;
- focused product test for the PBR matrix frame;
- ignored export test run that writes the PNG under `docs/tests/runtime/shader`;
- hash/dimensions check for the PNG;
- same-name scan proving the PNG was not written under repo `target` or external cargo target roots;
- status updates in Plan 11 and shader/render docs.

The work is not complete until current-state evidence proves that the PBR shader is sampling the environment path, not merely drawing a sky-colored background.

## Risks

Plan 13 cubemap import is not ready, so this slice must not pretend to finish external cubemap assets. The procedural environment map is acceptable only if it goes through the same renderer-owned environment sampling path that future cubemap assets will use.

The existing preview-sky path is broad. The implementation should hard-cut obvious preview-sky parallelism only where the new skybox path owns the same behavior. If a full preview-sky deletion would destabilize unrelated editor overlay code, the first implementation should route existing callers through `EnvironmentExtract` and record remaining deletion work explicitly in Plan 11 status.
