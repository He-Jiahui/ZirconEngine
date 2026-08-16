---
handoff_kind: fixed
status: fixed
created_at: 2026-08-11
resolved_at: 2026-08-16
summary_slug: hgi-m5-scene-prepare-neutral-sideband
origin_plan: docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
fixing_plan: docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
origin_child_dir: docs/plans/zircon_runtime/render/18
fixing_child_dir: docs/plans/zircon_runtime/render/18
failure_scope: local
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/framework/render/prepared_runtime_sidebands.rs
  - zircon_plugins/hybrid_gi/runtime/src/provider.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/root_output_sources/runtime_prepare_collector.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources/execute_prepare/execute/collect_inputs.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources/global_sdf/tests.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/test_sources/hybrid_gi_render_framework_stats/product_wgpu.rs
  - zircon_runtime/src/graphics/runtime_prepare_mesh_geometry_seed.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources/execute_prepare/execute/dispatch_probe_trace_tiles/tests/global_sdf.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/test_sources/hybrid_gi_render_framework_stats.rs
tests:
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_plugin_hybrid_gi_runtime -LibTests
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_plugin_hybrid_gi_runtime -LibTests -TestFilter export_global_sdf_build_wgpu_png -IgnoredTests
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_plugin_hybrid_gi_runtime -LibTests -TestFilter export_hybrid_gi_m5_global_sdf_trace_wgpu_png -IgnoredTests
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_plugin_hybrid_gi_runtime -LibTests -TestFilter export_hybrid_gi_voxel_miss_fallback_wgpu_png -IgnoredTests
  - ZR_HGI_M5_RENDERDOC_CAPTURE=1 WGPU_BACKEND=dx12 RenderDoc capture of export_hybrid_gi_voxel_miss_fallback_wgpu_png
  - D:/Tools/renderdoc/renderdoccmd.exe replay --loops 1 docs/tests/runtime/render/plan18_hybrid_gi_m5_scene_prepare_renderdoc_20260816_capture_capture.rdc
---

# Render18 M5: scene-prepare neutral sideband dynamic acceptance fixed

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/render/18-advanced-lighting-features.md`
- 来源执行切片：M5 scene-prepare neutral sideband WGPU execution gate
- 修复责任计划：`docs/plans/zircon_runtime/render/18-advanced-lighting-features.md`
- 交接原因：failure 与修复均属于 Render18 的 prepared-runtime sideband、HGI provider、
  runtime-prepare collector、Global SDF 与 probe trace owner。

## 失败现象与复现证据

原 failure 在唯一生产 `HybridGiGpuResources::execute_prepare` 调用上收到空 scene-prepare，
因此 card capture、surface-cache page、voxel clipmap 与 voxel cell 无法从 neutral prepared
sideband 到达 WGPU prepare。源码前向修复完成后，仍缺少同源 DX12 Global SDF build/trace、
RenderFramework 最终帧、RenderDoc 捕获和冷/热性能数据，不能关闭动态验收。

## 最低共享层根因

最低共享层根因是 renderer-neutral `RenderHybridGiPreparedFrame` 未承载 scene-prepare DTO，且
collector 无法用 `stable_instance_key` 将权威 prepared geometry world bounds 投影回插件输入。
旧路径只能在 provider 侧产生 CPU 合成反馈，或以 transform-scale sphere 近似缺失几何；这两种
结果都不能证明唯一 WGPU prepare 路径实际消费了精确场景资源。

## 架构修复验收

- core prepared-runtime sideband 承载 renderer-neutral scene frame；provider 只做单向 DTO
  投影，collector 只在唯一 WGPU prepare 入口恢复插件内部类型。
- collector 按 stable instance key 使用 `RuntimePrepareMeshGeometrySeed` 的变换后 local bounds；
  缺失或无效几何保持 typed unavailable，并省略相应 card/page/voxel 描述符。
- runtime-prepare mesh projection 缓存保存同次遍历得到的 canonical mesh snapshot 与 world
  bounds；稳定 revision 的热帧不重建 scene mesh，也不创建 Global SDF 临时上传资源。
- WGPU 产品证据使用帧数上限和 15 秒 wall-clock deadline 等待真实 depth/probe readback；
  超时会输出 in-flight、completion、slot reuse 与 Global SDF page 诊断。
- Global SDF trace fixture 的 `[-2, 2]` bounds、`0.5` voxel spacing 与 `+/-2` distance range
  保持物理一致，既不突破 128-page typed fallback 上限，也保留方向性 hit/miss 变化。

## 验证与量化结果

- `zircon_plugin_hybrid_gi_runtime` 完整 lib-test 图注册 229 项，结果为
  `209 passed / 20 ignored / 0 failed`；原编译漂移同时保持 `42 -> 0`。
- 三个 ignored DX12 exporter 分别通过：Global SDF build、Global SDF directional trace、
  RenderFramework voxel-miss fallback 最终帧。三张 PNG 均经人工视觉检查，无空白、损坏或裁切：
  - build PNG：512x512，6,894 bytes，SHA-256
    `6F4BDFF143937A0109AD5F16F40753345A9E2DCEB6359C55FE455813F82676DE`；
  - trace PNG：128x128，860 bytes，SHA-256
    `2E44C6C48585F022C90A851C76E36C1B96B75126409D6E081DAD3D6B62E556D1`；
  - product PNG：192x128，2,659 bytes，1,540 visible pixels，max luma 134.92，
    SHA-256 `AE814A49F6870437C37FC02643B24726CC2834DD889D95FCE8483D410C73366A`。
- 同源 RenderDoc 捕获大小为 13,015,972 bytes，SHA-256
  `3CBDA2F359F664F51BF4FDED4C2F72917ABD2B55716304DB0EDD731630AE29F5`；
  `renderdoccmd replay --loops 1` 退出码为 0。注入捕获运行不参与性能统计，捕获后重新运行了
  无 RenderDoc 的产品 exporter 来恢复干净 PNG 和性能数据。
- 冷启动首次 HGI Global SDF CPU prepare 为 7,381 us；经过 300 帧稳定期后的 31 个热样本
  p50/p95/max 为 `1,431 / 1,914 / 1,988 us`。冷帧到热样本 p50 减少 5,950 us，即 80.6%；
  该比较仅描述同一测试中的单次初始化冷帧与稳定热帧，不外推为通用帧时或功耗结论。
- 31 个热样本的 mesh projection cache lookup p50/p95/max 为 `7 / 9 / 12 us`，cache hit
  `31/31`、mesh scene sync 零开销 `31/31`、Global SDF transient buffer/bind-group/upload 零创建
  `31/31`。热提交 wall time p50/p95/max 为 `118,308 / 140,353 / 148,166 us`。
- 首帧 submit wall time 13,125,643 us 包含 device/shader 初始化。首帧 transient upload 原始值为
  0，是因为 provider 初始帧尚未 dispatch Global SDF；因此不把该字段用于声称冷到热上传降幅。

## 禁止临时方案

- 不把插件私有 `HybridGiScenePrepareFrame` 暴露给 `zircon_runtime`。
- 不增加第二个 GPU scene-prepare 入口、重复 DTO、测试专用生产导出或静默 fallback。
- 不恢复 transform-scale sphere，也不把 provider CPU 合成记录伪装成 GPU readback。
- 不使用 RenderDoc 注入帧作为性能样本，不以单次冷启动数值替代稳定期分位数。

## 修复结果与回传

- 状态：`fixed`。
- 根因：neutral prepared sideband 缺少 scene-prepare payload 与 stable-instance-key geometry
  投影，导致唯一 WGPU prepare 入口无法消费精确场景资源。
- 架构修复：在 core neutral DTO、provider 单向映射和 collector 权威 world-bounds 投影上闭环，
  并以 revision-aware canonical mesh projection cache 消除稳定热帧重建与临时上传。
- 验证：完整测试 `209/20/0`，三个 DX12 exporter 通过，三张 PNG 与同源 RenderDoc replay
  通过，300+31 性能协议满足 cache/sync/transient 三项 `31/31` gate。
- 回传：`docs/plans/zircon_runtime/render/18/2026-08-16-hgi-m5-scene-prepare-neutral-sideband-return.md`
  记录本 failure 的动态验收闭环。
