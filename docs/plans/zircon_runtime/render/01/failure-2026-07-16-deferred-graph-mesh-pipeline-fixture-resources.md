---
handoff_kind: failure
status: open
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

Open state: `待修复`; Render05 cannot treat its broad shadow gate as green while these two fixtures fail.
