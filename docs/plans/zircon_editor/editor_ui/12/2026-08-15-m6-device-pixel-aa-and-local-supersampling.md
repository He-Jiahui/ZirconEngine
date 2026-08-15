# UI12 M6 Device-Pixel AA And Local Supersampling

## Problem

The retained editor already creates its native GPU and softbuffer surfaces from
`Window::surface_size()`, so the final backing extent is the physical client
extent. The visible quality defect is downstream of that boundary:

- software rounded rectangles classify one pixel-center as fully inside or out;
- software scaled images use nearest-neighbor source selection;
- SVG assets are rasterized only at their final display extent;
- the WGPU path tessellates each rounded corner into six line segments and uses
  a one-sample solid pipeline; its rounded-box shader helper is not consumed.

Increasing only the `.zui` radius cannot repair those sampling defects.

## Unreal Reference Contract

The local Unreal source establishes the target architecture:

- `WindowsPlatformApplicationMisc.cpp` resolves per-monitor effective DPI.
- `SWindow.cpp` composes application scale and window DPI into the local to
  screen/layout transform while keeping window bounds in desktop pixels.
- `SlateRHIRenderer.cpp` creates the viewport/backbuffer from the physical
  window viewport extent.
- `SlateShaderCommon.ush` evaluates rounded-box signed distance and derives the
  transition width with `fwidth`, then applies `smoothstep` coverage.
- `SlateElementPixelShader.usf` uses screen-space derivatives for SDF text and
  antialiased lines.

Unreal's scene TSR/screen percentage is not the Slate UI raster contract. Slate
is composed at output resolution; density-independent layout, output-pixel
rasterization, analytic coverage, and filtered assets keep the UI sharp.

## Zircon M6 Contract

1. The native presentation surface remains at least the physical client extent.
2. Layout and pointer input remain in the existing logical/device transform;
   quality scaling must not change control size or hit geometry.
3. Software rounded fills and borders use subpixel coverage and alpha blending,
   not binary center tests.
4. Software image scaling uses premultiplied-alpha-safe bilinear sampling.
5. SVG icons use a bounded 2x local raster target and are filtered into their
   physical destination. This is selective supersampling, not a permanent 4x
   whole-window memory and fill-rate tax.
6. WGPU rounded fills and borders use the original shape rectangle plus the
   clipped visible rectangle in a pixel-domain analytic SDF fragment path. Edge
   width comes from screen-space derivatives. Raising tessellation segment
   counts is not an accepted substitute.
7. The global quality floor is 1.0 physical pixel per output pixel. Optional
   quality factors may only increase local render resolution and must stay
   bounded by the vector-raster edge limit.
8. RHI image payloads remain straight-alpha RGBA8 at the producer boundary.
   WGPU premultiplies each admitted generation exactly once before upload;
   shared textures, linear filtering, and the image fragment output remain
   premultiplied. Renderer-owned external textures must explicitly declare
   `Opaque` or `Premultiplied`; a post-filter premultiply is forbidden because
   it squares edge coverage and creates dark fringes.

## Acceptance

- Rounded fill and border pixel tests contain fractional edge coverage and keep
  opaque interiors/background exteriors.
- A 2x2 to 3x3 software image scale produces a blended center sample.
- SVG raster tests prove a bounded 2x source while the destination frame stays
  unchanged.
- WGPU tests prove the analytic shader consumes rounded-box distance,
  `fwidth`, and coverage for both fill and border without clip-radius drift.
- A real offscreen WGPU image-pipeline readback proves that filtering from a
  transparent texel to half/fully opaque blue yields premultiplied midpoint
  pixels `[0, 0, 64, 64]` and `[0, 0, 128, 128]`, with no transparent red
  bleed or second alpha multiplication.
- Current-source editor screenshots at 640x520, 900x620, and 1672x941 show no
  staircase corners, nearest-neighbor icon blocks, clipped command labels, or
  placeholder icon boxes.
- `.zui` radius tiers are tuned only after the raster fixes are visible.

## Current Evidence

Status on 2026-08-15 is `implementation_ready_visual_acceptance_pending`.

- Managed `zr_rhi_wgpu` focused tests pass for the analytic vertex/shader ABI,
  fractional rounded geometry, clip preservation, retained-cache offscreen
  rendering, and disjoint rounded-quad batching. The batching regression was
  updated from the removed polygon-fan contract to assert two six-vertex
  analytic quads, no ordinary-instance fallback, and retained radius/border
  parameters.
- The exact `zircon_runtime_interface` design-token test passes with the
  6/8/10/12 pixel small/control/large/panel radius hierarchy.
- Managed current-source `zircon_runtime` build job
  `86fb00309a674a86830939caf8fbed6f` passes. This supersedes the earlier
  editor attempt whose dependency graph still reported 27 shared runtime
  errors.
- The next managed `zircon_editor` attempt reached the editor test crate and
  exposed an independently split workbench-menu test module with invalid
  ancestor imports and a private sibling alignment enum. That test-only module
  has been repaired and passes targeted `rustfmt` and `git diff --check`; the
  retained dynamic retry is pending the shared Cargo pool.
- The software rounded-coverage, bilinear-image, and SVG-resolve tests remain
  pending until that editor test binary compiles. The required 640x520,
  900x620, and 1672x941 current-source product screenshots have not been
  captured and no visual acceptance is claimed.

## Performance Guardrails

- Do not supersample the complete editor surface by default.
- Keep rounded coverage work inside rounded primitive bounds.
- Cache SVG pixels by the actual supersampled source extent and tint.
- Preserve retained damage rendering and command generation caches.
- Record physical presented pixels separately from local supersampled source
  pixels so profiling does not misreport the swapchain extent.
- Follow
  `docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md`:
  popup geometry and hit testing keep one publication authority, event-time
  input remains read-only/local, and steady-state SVG interaction performs no
  parse, raster, or upload work.
- Product profiling must complete 1000 source-bound clicks, 1000 pointer moves,
  and 200 resize steps, reporting CPU core/system utilization, RSS/private
  memory, input-to-damage, and damage-to-submit distributions. A 999/1000 run
  is incomplete evidence rather than a pass.
