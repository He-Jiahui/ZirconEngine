---
handoff_kind: fixed
status: fixed
created_at: 2026-07-16
summary_slug: deferred-graph-mesh-pipeline-fixture-resources
origin_plan: docs/plans/zircon_runtime/render/05-lighting-shadows.md
fixing_plan: docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
origin_child_dir: docs/plans/zircon_runtime/render/05
fixing_child_dir: docs/plans/zircon_runtime/render/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/deferred.rs
  - zircon_runtime/src/graphics/tests/render_product_shadows/many_point_lights.rs
  - zircon_runtime/src/graphics/tests/render_product_shadow_captures/directional.rs
tests:
  - cargo test -p zircon_runtime shadow --locked
resolved_at: 2026-07-17
---


# Render01: deferred graph mesh-pipeline fixture resources

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/render/05-lighting-shadows.md`
- 来源执行切片：forward-depth shadow-atlas comparison-sampler correction的 broad shadow gate。
- 修复责任计划：`docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md`
- 交接原因：两个产品 parity fixture 在进入阴影测量前，均由 Render01 的 deferred graph executor 拒绝；最低共享层是 deferred pass 所需的 mesh-pipeline resource 装配，而不是 Render05 的采样比较、receiver 或阈值逻辑。

## 失败现象与复现证据

Managed broad job `cea7be4a2ab0484eb57cca59c11d8c66` / run
`3ac5c593d3d349feaf0445b93264201c` executed:

```text
cargo test -p zircon_runtime shadow --locked
```

It completed with exit `101`: `111 passed; 5 failed; 1 ignored; 8061 filtered`.
The following two failures occur before any shadow image metric is evaluated:

- `graphics::tests::render_product_shadows::many_point_lights::render_product_many_point_lights_forward_deferred_capture_parity`
  calls `submit_frame_extract` at `many_point_lights.rs:507` and receives
  `deferred graph executor for pass deferred-lighting requires mesh pipeline resources`.
- `graphics::tests::render_product_shadow_captures::directional::render_product_directional_shadow_atlas_forward_deferred_darkening_parity`
  calls `submit_frame_extract` at `render_product_shadow_captures/directional.rs:390` and receives the same error.

The shared error is emitted by
`zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/deferred.rs`.

## 最低共享层根因

The fixture path that submits a `deferred-lighting` graph does not provide the mesh-pipeline resources required by the Render01 deferred executor. This fails before Render05 shadow-atlas sampling or receiver darkening can be observed, so it cannot be repaired by changing a shadow sampler, bias, receiver material, or capture assertion.

## 架构修复验收

- Supply or correctly construct the shared mesh-pipeline resources required by the deferred graph executor through its normal fixture/runtime setup path.
- Re-run the two named focused product parity tests; both must reach their forward/deferred comparison rather than fail in `submit_frame_extract`.
- Re-run `cargo test -p zircon_runtime shadow --locked` through the managed Cargo coordinator and return the resulting evidence to Render05.

## 禁止临时方案

- Do not bypass deferred execution, skip the deferred half of either parity test, or add a test-only fallback resource.
- Do not weaken product thresholds or convert the error into an ignored/expected failure.
- Do not change Render05's forward-depth compare-function contract as a substitute for resource construction.

## 修复结果与回传

- 根因：Graph-pass assembly installed mesh pipeline resources only when the optional ResourceStreamer was also present. Deferred supplied a streamer, but the Lighting stage intentionally supplied None, so its valid MeshPipelineCache and draw lists were discarded and deferred-lighting failed before Render05 image metrics.
- 架构修复：Make execute_deferred_graph_stage require &mut MeshPipelineCache and pass the owned cache from both Deferred and Lighting callers. Install mesh pipelines plus draw lists independently in with_mesh_renderer, while keeping the ResourceStreamer as a separate optional stage resource. This preserves normal runtime ownership and adds no compatibility or test-only fallback.
- 验证：Focused lower job c753b96194c54b32b4bd8d3ae15b0f6b / run b4f9d6cdc8ff41f9854f839d1f2ec7ef executed the target exactly once and passed (1 passed, 8178 filtered). Many-point parity job 28cb7f882ce941fabde99689ca44cad7 / run d02771a7a3464eb1872dfdd1e1f1c145 executed its target exactly once and passed. Canonical lib-only directional job d57cb5f09ee24b9685c36e63fd445457 / run 02a1348e8b124e83aad8605ca43848e3 used cargo test -p zircon_runtime --lib render_product_directional_shadow_atlas_forward_deferred_darkening_parity --locked --jobs 1 -- --test-threads=1; raw stdout contains the target exactly once and reports 1 passed, 0 failed, 8178 filtered. Exact broad job 1eca3d1ab532400e95428876edaa8328 / run 136024091c254c5b85c3fb6ba9afff37 used cargo test -p zircon_runtime shadow --locked; both affected parity tests executed exactly once and passed. Broad terminal was exit 101 with 113 passed, 3 failed, 1 ignored, 8062 filtered; the remaining failures are post-submit Render05 visual/PCF assertions (multi-spot darkening, mixed-atlas frame delta, spot-PCF edge profile), not mesh-pipeline resource errors. rustfmt --check and git diff --check passed; independent review reported critical 0, important 0.
- 回传：Render01 lower mesh-pipeline propagation is fixed: both originally blocked forward/deferred parity fixtures now reach their comparisons and pass exactly once. Render05 broad shadow remains red only on three downstream visual/PCF assertions and must continue there; no Render05 sampler, threshold, receiver, or shader source was changed by this repair.
