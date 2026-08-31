---
handoff_kind: failure
status: open
created_at: 2026-08-25
summary_slug: ui-srgb-coverage-and-native-drop-order
origin_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
fixing_plan: docs/plans/optimize/zircon_runtime/79-runtime-ui-renderer-display-list-paint-order-clip-transform-opacity-atlas-text-glyph-batch-wgpu-submit-product-integration-current-source-review.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/01
fixing_child_dir: docs/plans/optimize/zircon_runtime/79
related_code:
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/tests/native_submission.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/shaders/ui_material.wgsl
tests:
  - >-
    $env:RUST_TEST_THREADS='1'; $env:RUST_TEST_NOCAPTURE='1';
    $env:TEMP='E:\Git\ZirconEngine\.codex\state\session-coordinator';
    & .\.codex\skills\zircon-dev\scripts\validate-matrix.ps1
    -Package zr_rhi_wgpu -SkipBuild -LibTests
---

# Runtime79: UI sRGB fixture coverage coupling and native drop order

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
- 来源执行切片：Frameworks01 RHI/WGPU production integration failure convergence
- 修复责任计划：`docs/plans/optimize/zircon_runtime/79-runtime-ui-renderer-display-list-paint-order-clip-transform-opacity-atlas-text-glyph-batch-wgpu-submit-product-integration-current-source-review.md`
- 交接原因：失败位于 Runtime79 M7 的 SDR/sRGB/linear/alpha 与 native surface 生命周期边界；Frameworks01 只负责证明通用 RHI device/submission owner 的最低共享层已恢复，不能改写 UI renderer 的颜色夹具或 surface owner。

## 失败现象与复现证据

Windows 受管串行验证 job `4b44603041db430697199260a9c6a4b1` 在 D 盘 target 上完成编译并以标准 Cargo test exit `101` 结束。修正通用 RHI native 依赖对象的字段析构顺序后，原先聚合测试进程的 `STATUS_ACCESS_VIOLATION` 不再出现；当前唯一终态失败是：

```text
ui_surface::tests::native_submission::wgpu_ui_srgb_target_encodes_linear_light_alpha_coverage
expected RGB 187..=189 and alpha 127..=128
actual [174, 174, 174, 108]
```

夹具在 `render_flat_solid_sample` 中创建 `1 x 1` target，把全屏顶点的 `local_position` 直接设为 clip-space `position`，并走 `solid_fragment_color`。该 shader 必须对 rounded-box signed distance 执行 `fwidth` 与 `smoothstep` 解析覆盖，因此唯一采样同时测量 sRGB transfer 和边缘 coverage，不能单独证明 Runtime79 `RUIR-GATE-034` 的 linear-to-sRGB reference pixel。

同一静态生命周期审查还发现 `WgpuUiSurfaceContext` 与 `WgpuUiSurfaceRenderer` 都先声明 `Instance/Adapter/Device/Queue`，再声明 shared image、surface、pipeline、cache、text、readback 等 native dependents。Rust 按字段声明顺序析构，这违反了依赖对象先于 queue/device/adapter/instance 释放的 owner invariant。通用 `WgpuRenderDevice`、`WgpuRenderDeviceContext` 与 `WgpuSubmissionService` 已按该 invariant 修正并通过上述聚合进程验证；UI owner 仍待 Runtime79 收口。

## 最低共享层根因

1. 颜色资格夹具把 output transfer 与解析几何 coverage 合并到同一个 `1 x 1` 边缘样本，导致 `[174,174,174,108]` 不能区分 transfer 错误和约 `0.42` 的 coverage 衰减。
2. UI surface owner 没有把 Rust 字段声明顺序视为 native resource lifetime contract，导致 queue/device/adapter/instance 可能先于引用它们的 surface、pipeline、cache、text、readback 与 shared image state 析构。

## 架构修复验收

- 用 coverage-free solid-instance 路径，或使用有严格内部采样点的更大 target，独立验证 linear `50%` white 输出到 `Rgba8UnormSrgb` 时 RGB 为 `187..=189`、alpha 为 `127..=128`；不得降低 reference range。
- rounded-edge coverage 继续由现有 `wgpu_ui_rounded_solid_readback_contains_fractional_edge_coverage` 独立验证，颜色 transfer 与几何 AA 两类合同不得重新耦合。
- `WgpuUiSurfaceContext` 与 `WgpuUiSurfaceRenderer` 的所有 surface/pipeline/cache/text/readback/shared-image dependents 必须在 `Queue/Device/Adapter/Instance` 之前析构，并由 source contract 或等价生命周期测试冻结顺序。
- 重新运行本文件 frontmatter 中的完整 Windows 受管串行 `zr_rhi_wgpu --lib`；必须 exit `0`，不得出现 access violation、device-loss 假阳性或跳过 reference-pixel 断言。
- Runtime79 `RUIR-GATE-034`、`RUIR-GATE-035` 与 M7 color/surface matrix 继续保持开放，直到真实产品路径与 reference artifact 验证完成。

## 禁止临时方案

- 不得把 RGB 期望值改为 `174` 或把 alpha 期望值改为 `108` 来掩盖 coverage 耦合。
- 不得增加 alias、compatibility shim、silent fallback、duplicated truth、test-only bypass 或 call-site exception。
- 不得只让测试对象提前 `drop` 来绕开结构体错误字段顺序；production owner 本身必须表达正确生命周期。
- 不得以单个 offscreen 像素测试替代 Runtime79 的完整 color、surface、capture 与 backend 一致性门。

## 修复结果与回传

Open state: `待修复`; no pass is claimed.
