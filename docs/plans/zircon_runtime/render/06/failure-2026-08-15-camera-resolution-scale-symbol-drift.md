---
handoff_kind: failure
status: open
created_at: 2026-08-15
summary_slug: camera-resolution-scale-symbol-drift
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/render/06-temporal-pipeline.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/render/06
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/framework/render/camera.rs
tests:
  - .\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipTest
---

# Render06: Camera resolution-scale symbol drift blocks the MVP product build

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：M0 current product baseline recovery before WPR, RenderDoc, and energy capture
- 修复责任计划：`docs/plans/zircon_runtime/render/06-temporal-pipeline.md`
- 交接原因：失败位于 Render06 正在迁移的 camera-to-view-family resolution policy boundary；该文件仍由 `render06-view-family-pipeline-20260815` 持有。

## 失败现象与复现证据

2026-08-15 15:28 CST 启动的受管 `zircon_runtime` build-only job `ebc7ae5809ac4713a0ece1b1503283e8` 在约 4 分 50 秒后以 exit 1 结束。保留在 D 盘共享目标池的 Cargo 指纹诊断显示：

- `E0425`：`zircon_runtime/src/core/framework/render/camera.rs:84:17` 找不到 `DEFAULT_RENDER_RESOLUTION_SCALE`。
- 同次构建的另一处 Render01 `BufferViewMut::fill` 错误已由其所有者在构建结束后修正，不属于本交接。
- 该构建产生 209 条 warning；本记录不把 warning 数量解释为性能结论。

## 最低共享层根因

`ViewportCameraSnapshot::render_view_family_pipeline` 已切换到 `RenderResolutionPolicy::with_scales`，但第二个 scale 仍引用迁移前符号 `DEFAULT_RENDER_RESOLUTION_SCALE`；当前模块只声明 `DEFAULT_DYNAMIC_RESOLUTION_SCALE`。在 Render06 确认 primary/display/history resolution 语义前，来源计划不得用本地别名或任意常量替换来掩盖契约漂移。

## 架构修复验收

- Render06 明确 `with_scales(primary_scale, secondary_scale)` 两个参数各自的像素空间语义，并让 camera 默认值引用唯一、现存的 owner constant。
- `camera_dynamic_resolution_adapts_into_the_view_family_resolution_policy` 及 Render06 的 view-family/temporal-history focused tests 通过。
- 原始受管 `zircon_runtime` build-only 通过，再运行 `tools/build-editor.ps1` 产出当前源码的 D/E/F 盘编辑器 bundle。
- 来源性能计划只在可运行 bundle 上继续 WPR、RenderDoc 和功耗采集，不使用 2026-08-10 的旧二进制。

## 禁止临时方案

- 不新增 `DEFAULT_RENDER_RESOLUTION_SCALE` 别名、兼容 shim、静默 fallback 或 call-site 特例。
- 不把第二个 scale 猜成 primary scale 的副本；必须由 Render06 的 resolution-space 契约和测试确定。
- 不降低产品构建或动态性能采集验收门。

## 修复结果与回传

Open state: `待修复`; no product-build or dynamic-performance pass is claimed.
