---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11b-runtime-text-font-shaping-layout-editing-ime-review.md
  - docs/plans/optimize/zircon_runtime/11c-gpu-ui-renderer-atlas-sdf-batch-clip-submit-review.md
  - docs/plans/optimize/zircon_runtime/81-runtime-text-shaping-unicode-bidi-script-run-cluster-line-break-wrap-layout-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/11c/2026-09-09-image-bind-group-idle-retention.md
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/image.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/image/geometry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/ui/text/layout_engine/paragraph_layout.rs
  - zircon_runtime/src/text/layout/measure.rs
  - zircon_runtime/src/graphics/scene/resources/ui_texture.rs
tests:
  - tools/tests/test_runtime10_plan_current_contract.py
  - tools/tests/runtime_text_infrastructure_compile_contract/rich_parser_admission.py
  - tools/tests/test_runtime_text_inline_resource_prepare_receipt_contract.py
  - tools/tests/test_runtime_text_render_batch_owner_contract.py
  - tools/tests/test_runtime_text_rich_source_contract.py
  - tools/tests/test_runtime11_preference_persistence_lane_contract.py
---

# Runtime Text and Image Owner Repair

The Runtime text and UI-image slices now match the current source contracts while preserving
their hard-cut behavior. Image geometry, scissor, upload, and cache-policy helpers moved under
the folder-backed `image/geometry.rs` owner, leaving the parent responsible for lifecycle,
dependency publication, and draw orchestration. The renderer text-batch import contract and the
text paragraph/measurement checks now describe the current checked-range, shaped-span, and
visibility owners. The inline texture dependency contract records its Arc-owned product instead
of requiring a value return. Stable image frames compare the published frame and texture
generation through a read-only path before advancing or trimming the bind-group cache, so the
steady-state path does not perform cache cleanup or segment traversal.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime11B/11C text and image owners | Keep bounded image cache products segment-owned, isolate geometry/upload helpers, and align stale static assertions with the V8/Arc/current shaped-span contracts | implemented_pending_validation | Focused Runtime text/image/ABI batch `58/58`; Runtime performance contracts `1143/1143`; Runtime pressure `142/142`; scoped Rustfmt and diff checks pass. |

## Validation boundary

The broad Runtime contract batch ran `1425` tests with one failure in the WOC preference lane:
`examples/woc/native/Cargo.lock` does not contain the `zircon_runtime` package/dependency required
by the already-updated `woc_client/Cargo.toml`. Regenerating that nested lock graph must happen in
the managed Cargo environment; this record intentionally does not hand-edit a generated lockfile.
The managed Runtime/Editor Cargo and release p50/p95/p99 gates therefore remain pending, and no
product CPU, allocation, RSS, or percentile claim is made here.

Editor static performance and pressure batches remain green at `581/581` and `122/122`; the
existing full Editor static run remains `1240/1241` in-scope, with only the excluded tooling
preview-token check outside this source scope.

## Coordinator dispatch log

- 2026-09-11: the combined `zircon_app`/`zircon_runtime`/`zircon_editor` Cargo batch was submitted
  with request id `astra-runtime-editor-compile-batch-20260911-r1`, but immutable admission
  rejected it before a ticket was created because external worktree `E:\Git\zr_vm` is dirty. No
  Cargo command ran and this record does not infer a compile result from the rejection.
