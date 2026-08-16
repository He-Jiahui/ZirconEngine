---
record_kind: failure_closeout_delivery
status: accepted
lifecycle_keys_json: ["e:/git/zirconengine/docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md|e:/git/zirconengine/docs/plans/zircon_runtime/render/18-advanced-lighting-features.md|app01-editor-host-hgi-compile-drift", "e:/git/zirconengine/docs/plans/zircon_runtime/render/18-advanced-lighting-features.md|e:/git/zirconengine/docs/plans/zircon_runtime/render/18-advanced-lighting-features.md|hgi-m5-scene-prepare-neutral-sideband"]
delivery_paths_json: ["docs/tests/runtime/render/plan18_hybrid_gi_m5_global_sdf_build_wgpu_20260813.png", "docs/tests/runtime/render/plan18_hybrid_gi_m5_global_sdf_trace_wgpu_20260810.png", "docs/tests/runtime/render/plan18_hybrid_gi_m5_scene_prepare_renderdoc_20260816_capture_capture.rdc", "docs/tests/runtime/render/plan18_hybrid_gi_voxel_miss_fallback_wgpu_20260707.png"]
---

# Render18 HGI M5 DX12 动态证据交付

本记录只把三个 DX12 exporter 的 PNG 与同源 RenderDoc 捕获纳入两个重叠 failure 的原子
closeout。源码路径继续由两个 fixed handoff 的 `related_code` 清单约束。

- Global SDF build PNG：512x512，SHA-256
  `6F4BDFF143937A0109AD5F16F40753345A9E2DCEB6359C55FE455813F82676DE`。
- Global SDF trace PNG：128x128，SHA-256
  `2E44C6C48585F022C90A851C76E36C1B96B75126409D6E081DAD3D6B62E556D1`。
- RenderFramework product PNG：192x128，1,540 visible pixels，SHA-256
  `AE814A49F6870437C37FC02643B24726CC2834DD889D95FCE8483D410C73366A`。
- RenderDoc `.rdc`：13,015,972 bytes，SHA-256
  `3CBDA2F359F664F51BF4FDED4C2F72917ABD2B55716304DB0EDD731630AE29F5`，
  `renderdoccmd replay --loops 1` 退出码为 0。
- 性能协议为 300 帧稳定期加 31 个热样本：HGI Global SDF CPU prepare
  p50/p95/max `1,431 / 1,914 / 1,988 us`；cache hit、零 scene sync、零 transient
  allocation/upload 均为 `31/31`。
