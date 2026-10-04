---
title: Runtime43 F2 persisted primitive pixel oracle
category: zircon_runtime
status: source_candidate_validation_pending
implementation_status: applied_source_candidate
validation_status: managed_validation_pending
product_status: wgpu_capture_pending
related_code:
  - zircon_runtime/src/dynamic_api/session/tests/foundation_render.rs
  - zircon_runtime/src/dynamic_api/session/tests/foundation_render/f2_evidence.rs
  - templates/projects/renderable-empty/assets/scenes/main.scene.toml
  - templates/projects/renderable-empty/assets/models/cube.obj
  - templates/projects/renderable-empty/assets/materials/default.zmaterial
related_tests:
  - zircon_runtime/src/dynamic_api/session/tests/foundation_render.rs
plan_sources:
  - docs/plans/mvp/03-f2-scene-runtime.md
  - docs/plans/optimize/zircon_runtime/43-dynamic-runtime-session-registry-ffi-frame-event-extract-host-request-world-sync-ui-shader-prewarm-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/99zf-runtime-dynamic-session-registry-ffi-frame-event-extract-host-request-world-sync-ui-shader-prewarm-product-integration-current-source-review.md
---

# Runtime43 / F2: persisted primitive pixel oracle

## Candidate change

`assert_basic_scene_frame` previously compared every output pixel with the top-left pixel. A
spatially varying background can satisfy that check while the persisted Cube contributes no visible
raster. The applied source candidate creates the normal project and a reference project from one rendered
`RenderableEmpty` template pack. The reference keeps the same manifest identity, all non-scene
assets, Camera, and Sun; its only scene change is removal of the `Cube` entity. The local
`RenderedProjectTemplate` export is available through `zircon_runtime_interface::project`.

The persisted `cube.obj` vertices span `[-0.5, +0.5]` on each axis, so the mesh has one-unit sides
with the scene's unit scale. At the persisted 100-degree vertical field of view and a 640 x 360
capture, the vertical focal length is about 151.0 pixels. The front face lies at z = +0.5, about
14.0 units from the camera at z = 14.5; with the identity camera rotation it is parallel to the
image plane. Its projected side is about 10.8 pixels and its footprint is about 116 pixels. The
candidate requires at least 64 exact RGBA pixels to differ from the paired cube-free frame, slightly
more than half of that projected face footprint. This is a projection-based floor, not a claim about
measured adapter coverage.

The product test captures paired frames at the same frame positions and counts exact per-pixel RGBA
differences against the cube-free reference. A focused CPU regression supplies a spatially varying
image as both captures and expects zero primitive pixels, then changes one pixel and expects one.

The frame still must submit a mesh and directional light, execute graph passes, avoid material
fallbacks and validation errors, and return non-transparent RGBA. The cube-free reference must
execute the graph, retain the light, and submit no mesh.

## Preserved coverage

Input event acceptance and release, imported template asset readiness, second-frame RenderGraph
cache reuse, stable cache size, zero static GPUScene dirty entries and upload bytes, restart draw and
light counts, physical test-binary fixture roots, PNG RGBA roundtrip, and teardown deletion remain
in the candidate.

## Managed product-evidence followup

The applied source followup extracts PNG artifact handling and product-frame evidence into the test-only
foundation_render/f2_evidence.rs module. The WGPU product test calls it after each existing frame
oracle: first launch, unchanged second frame, and second launch after teardown. The existing oracle
thresholds and cache, input, asset, restart, and teardown assertions are unchanged.

The managed Cargo runner passes CARGO_TARGET_DIR and ZIRCON_MANAGED_BUILD_POLICY to the Rust test
process. With the marker present, the helper first requires an absolute raw CARGO_TARGET_DIR under
one exact approved root. It then verifies that the root itself canonicalizes to its literal
D:/cargo-targets, E:/cargo-targets, or F:/cargo-targets location, and that the resolved target
stays beneath that physical root and equals the configured path after normalizing drive case,
separators, and the Windows verbatim prefix. Root junctions, nested symlink/junction aliases,
foreign lexical roots that resolve into an approved tree, missing targets, and canonicalization
failures are rejected.
A valid managed target receives a unique CARGO_TARGET_DIR/f2-evidence/foundation-render-<pid>-<timestamp>-<attempt>
directory with first-launch.png, unchanged-second-frame.png, and second-launch-after-teardown.png.
If the managed marker is absent and there is no explicit capture override, the test writes no files
and continues normally. If the marker is present but its target is invalid, evidence setup returns
an actionable error and the product test fails, including when an explicit override is supplied.
An explicit ZR_F2_BASIC_SCENE_CAPTURE_PNG override still writes the first frame at that exact path
and places the other two labeled PNGs beside it. Its stdout uses stable
explicit-override/<capture-slot-filename> labels and does not echo the override directory or
basename; managed default paths remain fully printed. A present non-Unicode override preserves the
old no-capture behavior after required managed target validation, so it cannot satisfy the artifact
gate. The normal grouped product receipt still requires all three actual PNG files.

Each successful f2_basic_scene_evidence line includes its output path, 640 x 360 dimensions, the
observed per-pixel primitive count, non-transparent pixel count, backend, adapter name and type, and
the published device-limit fields. The runtime diagnostics currently publish no driver version, so
driver is printed as unknown; absent adapter or limits are also printed as unknown. Each written PNG
is reopened and compared with the captured frame's dimensions and complete RGBA bytes. The existing
zircon_runtime sha2 dependency hashes each actual PNG file after its write/readback check; the
resulting SHA-256 is printed with that frame's evidence line. No synthetic image hash is used.

## Validation and grouped product gate

The independently reviewed followup has been applied after exact preimage checks, live claims, and
source attribution. Scratch formatting and patch applicability checks passed. No Cargo command,
WGPU capture, or product test has run; managed and product validation remain pending.

Run the normal managed Runtime/Editor library-test batch as one grouped validation
with --nocapture on the batch's normal test arguments. Do not create a separate per-fix lane. The
existing render_product_f2_persisted_basic_scene_renders_accepts_input_and_shuts_down test is
unignored and must execute in that batch on a product-approved Windows WGPU adapter. The runner
provides the managed target environment; no per-task capture environment injection is required.
The receipt must include all three PNG paths and actual SHA-256 values, image dimensions and
primitive/non-transparent pixel counts, backend/adapter/type/limits, input and persisted-asset
checks, graph-cache reuse, zero unchanged-frame GPUScene dirty entries/uploads, restart comparisons,
and teardown results. A source check or library pass without adapter-backed capture does not close
the product gate.

This oracle repair claims no runtime speedup. It retains the existing cache/upload acceptance counters
but includes no latency or throughput measurement; any broader numeric performance budget remains open.

## Successful frame evidence output

With --nocapture, each actual product frame emits one f2_basic_scene_evidence line after the existing
pixel assertions pass. png_path=not-written occurs for ordinary unmarked runs without an explicit
capture override and for a present non-Unicode override after managed-target validation; the latter
preserves the legacy no-capture behavior and cannot satisfy the three-PNG product evidence gate. A
marked managed run with missing or invalid CARGO_TARGET_DIR fails evidence setup.
