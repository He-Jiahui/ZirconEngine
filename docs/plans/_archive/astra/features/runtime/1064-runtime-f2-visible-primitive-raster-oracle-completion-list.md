---
doc_type: feature-completion
status: source_candidate_validation_pending
implementation_status: applied_source_candidate
validation_status: managed_validation_pending
product_status: wgpu_capture_pending
plan_sources:
  - docs/plans/mvp/03-f2-scene-runtime.md
  - docs/plans/optimize/zircon_runtime/43-dynamic-runtime-session-registry-ffi-frame-event-extract-host-request-world-sync-ui-shader-prewarm-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/99zf-runtime-dynamic-session-registry-ffi-frame-event-extract-host-request-world-sync-ui-shader-prewarm-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/43/2026-09-29-f2-visible-primitive-pixel-oracle.md
implementation_files:
  - zircon_runtime/src/dynamic_api/session/tests/foundation_render.rs
  - zircon_runtime/src/dynamic_api/session/tests/foundation_render/f2_evidence.rs
tests:
  - zircon_runtime/src/dynamic_api/session/tests/foundation_render.rs
---

# Runtime1064 / F2 persisted primitive raster oracle

| Acceptance item | Candidate evidence | Required gate | Status |
| --- | --- | --- | --- |
| Visible persisted primitive | Render the persisted one-unit Cube and a cube-free scene generated from the same template pack, preserving Camera and Sun; count exact per-pixel RGBA differences and require at least 64 pixels. | Normal grouped managed Runtime/Editor library-test batch with --nocapture; this batch includes the varying-background CPU regression and the unignored WGPU product test. | applied; validation pending |
| Runtime behavior and steady state | Candidate retains template asset readiness, input press/release, draw/light/material diagnostics, RenderGraph cache reuse, zero unchanged-frame GPUScene dirties/uploads, restart comparison, PNG roundtrip, and fixture teardown checks. | Normal grouped managed Runtime/Editor library-test batch with --nocapture. | open |
| Product capture | The actual WGPU test emits first, unchanged-second, and post-teardown restart PNGs in the managed target f2-evidence directory by default; each PNG is reread and pixel-checked. | Normal grouped managed Runtime/Editor library-test batch with --nocapture on a product-approved Windows WGPU adapter; compute SHA-256 from the actual three PNGs and retain dimensions, observed pixels, backend/adapter/type/limits, and runtime counters in the receipt. | open |
| Performance claim | The existing cache and upload counters remain acceptance assertions. This test-only oracle candidate makes no speedup, latency, or throughput claim. | Record actual steady-state counters in the product receipt; keep any separate numeric performance budget open until measured. | open |

This source candidate has been applied to the shared checkout; managed validation remains pending. The reviewed V3 paired-frame assertions remain unchanged. The logging-only
follow-up prints successful per-label dimensions and primitive/non-transparent pixel
counts; its scratch formatting and patch-integrity checks passed. Managed Cargo validation and adapter-backed product evidence remain pending.

## Successful frame statistics

After the assertions pass, the test emits f2_basic_scene_frame with label,
width, height, primitive_pixels and non_transparent_pixels. Capture stdout
with --nocapture in the managed product run. This line is test instrumentation;
its presence in source is not adapter-backed raster or performance evidence.

## Managed product-evidence followup

The applied source followup wires foundation_render/f2_evidence.rs directly into the existing unignored
WGPU product test. The runner already supplies CARGO_TARGET_DIR and ZIRCON_MANAGED_BUILD_POLICY to
the test process. With that marker, the raw CARGO_TARGET_DIR must be absolute and lexically beneath one
exact approved root. That root must canonicalize to its literal D:/cargo-targets, E:/cargo-targets, or
F:/cargo-targets location. The resolved target must stay beneath that physical root and equal the raw
configured path after normalizing drive case, separators, and the Windows verbatim prefix. This rejects
root junctions, nested symlink/junction aliases, foreign lexical roots that resolve into an approved
subtree, missing targets, and canonicalization failures. A valid target receives all three labeled
captures under a unique f2-evidence directory. If the managed marker is present but the target is
missing or invalid, evidence setup fails closed even when an explicit capture override is set. Only an
absent managed marker with no explicit override skips automatic output and continues normally. A Unicode
ZR_F2_BASIC_SCENE_CAPTURE_PNG path remains the first-frame path; labeled sibling paths hold the
unchanged-second and restart frames. For explicit overrides, stdout uses stable
explicit-override/<capture-slot-filename> labels and does not echo the user directory or basename,
while files retain their configured paths. A present non-Unicode override preserves the old no-capture
behavior after managed target validation, so it emits no artifacts and cannot satisfy the product
evidence gate. The normal grouped product receipt still requires all three actual PNGs.

The --nocapture product receipt must contain successful per-frame paths, dimensions, primitive and
non-transparent pixel counts, backend, adapter name/type, currently published device limits, and
driver=unknown because the existing runtime diagnostic surface does not publish driver data. Each
actual PNG is reopened and checked byte-for-byte against its captured RGBA. The existing sha2 crate
then hashes the emitted file bytes, and the printed SHA-256 is retained in the grouped product receipt.

The threshold and assertions remain those of the already reviewed paired-frame oracle. This followup
is applied with managed validation pending; managed compilation and Windows adapter-backed WGPU evidence are
still open. The gate is the normal grouped managed Runtime/Editor library-test batch with --nocapture;
there is no separate per-fix lane. A library-test success without adapter-backed capture does not
satisfy Runtime1064's product-capture gate.
