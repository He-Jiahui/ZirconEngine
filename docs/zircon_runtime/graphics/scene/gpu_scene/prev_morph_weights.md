---
related_code:
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_morph_weights.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/mod.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache/morph.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache/morph/skinned_velocity.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/gpu_scene_sync.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/morph_payload_upload.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_geometry_morphed.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_geometry_skinned_morphed.wgsl
implementation_files:
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_morph_weights.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache/morph.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache/morph/skinned_velocity.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/gpu_scene_sync.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/morph_payload_upload.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_geometry_morphed.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_geometry_skinned_morphed.wgsl
plan_sources:
  - docs/plans/zircon_runtime/render/08-material-shader-permutation.md
  - docs/zircon_runtime/graphics/scene/scene_renderer/temporal/velocity.md
tests:
  - rustfmt --edition 2021 --check on Plan 08 morph previous-weight touched Rust files
  - cargo check -p zircon_runtime --lib --no-default-features --features target-server --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-morph-prev-0701 --message-format short --color never
  - source anchor scan for previous_morph_weights, stage_current_morph_weights, roll_prev_morph_weights_after_success, previous_weights, and payload.y + payload.w
  - git diff --check scoped to touched files, with LF/CRLF warnings only
  - cargo test -p zircon_runtime --lib morph_payload ... did not produce a counted result: one run was blocked by unrelated dirty-state status-doc test files, and one rerun timed out during Windows lib-test compile/link
  - rustfmt --edition 2021 --check on Plan 08 direct morph velocity product touched Rust files
  - cargo check -p zircon_runtime --lib --no-default-features --features target-server --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-morph-velocity-product-0701 --message-format short --color never
  - cargo test -p zircon_runtime render_product_direct_mesh_morph_weight_change_writes_scene_velocity_pixels --lib --no-default-features --features target-server --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-morph-velocity-product-0701 --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime render_product_skinned_mesh_morph_weight_change_writes_scene_velocity_pixels --lib --no-default-features --features target-server --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-morph-velocity-product-0701 --message-format short --color never -- --nocapture --test-threads=1
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_morph_weights.rs::tests::render_gpu_scene_rolls_explicit_zero_morph_weights_for_starting_velocity
doc_type: module-detail
---

# GPUScene Previous Morph Weights

This module is the renderer-owned previous-weight surface for Plan 08 GPU morph velocity. It stores source morph weights by stable mesh instance key and rolls them only after a successful frame submission, matching the previous-transform, previous skinned-palette, and previous skinned-source policy.

`stage_current_morph_weights(...)` is called while mesh draws are synchronized into GPUScene. Direct mesh draws with morph targets stage nonempty source morph weights even when every weight is zero; draws with no source weights remove the current entry for that stable key. `PendingMeshDraw.source_morph_weights` intentionally decouples history staging from active GPU morph payload creation, so a first frame with `[0.0]` can become the previous row for a later `[1.0]` velocity frame. `roll_prev_morph_weights_after_success(...)` copies the staged current map into the previous map for the next frame and drops stale previous entries when the current frame stopped staging them.

`morph_payload_upload.rs` consumes the rolled previous weights while building the next frame's morph payload. The `GpuMorphPayload` header ABI does not change: `weight_base` still points at the current-weight block, and the previous-weight block begins at `weight_base + target_count`. The upload keeps a target if either the current or previous weight is nonzero, so current-zero/previous-nonzero transitions still have a payload for velocity. When no previous row exists yet, the upload falls back to the current weight for first-frame stability.

The direct mesh path can therefore create a GPU Morphed source even when the current morph weights are zero but previous weights were nonzero. That keeps the Velocity pass on the morphed shader source instead of dropping to a static CPU-baked fallback that cannot reconstruct pure morph motion.

`zr_geometry_morphed.wgsl` reads previous weights through `zr_morph_previous_weight(...)` when implementing `fetch_prev_position(...)`. `zr_geometry_skinned_morphed.wgsl` first reconstructs the previous morphed position and then applies the previous skinning matrix, preserving the same morph-before-skin order used by the current position path.

Current validation covers the code-level data path, shader consumption, explicit zero-weight rolling, and focused direct plus skinned 0.0 -> 1.0 WGPU product readbacks that write nonzero `scene-velocity` pixels through the forward-plus/Core3d path. RenderDoc capture, broader product miss=0/second-launch acceptance, and full CI remain separate Plan 08 gates.
