---
handoff_kind: failure
status: open
created_at: 2026-08-19
summary_slug: cube-mip-range-test-visibility
origin_plan: docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md
fixing_plan: docs/plans/zircon_runtime/render/11-environment-lighting.md
origin_child_dir: docs/plans/zircon_runtime/text/03
fixing_child_dir: docs/plans/zircon_runtime/render/11
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_graph_plan/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_time_slice.rs
tests:
  - .\\.codex\\skills\\zircon-dev\\scripts\\validate-matrix.ps1 -Package zircon_runtime -LibTests -TestFilter text_oversized_run_keeps_one_logical_shaped_line -VerboseOutput
---

# Render11: CubeMipRange test visibility

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md`
- 来源执行切片：Text03 长文本逻辑行回归门。
- 修复责任计划：`docs/plans/zircon_runtime/render/11-environment-lighting.md`
- 交接原因：失败发生在实时 IBL 图计划的测试编译边界；Text03 仅触发完整 runtime 的 `cfg(test)` 门，未定义或使用该符号。

## 失败现象与复现证据

2026-08-19 的受管 Windows test lane 完成了 `cargo build -p zircon_runtime --locked`，随后在库测试编译前失败：

```text
error[E0422]: cannot find struct, variant or union type `CubeMipRange` in this scope
  --> zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_graph_plan/tests.rs:135:23
```

受影响的 Text03 命令为：

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -LibTests -TestFilter text_oversized_run_keeps_one_logical_shaped_line -VerboseOutput
```

构建阶段通过；`cfg(test)` 在运行任何 Text03 测试前以 exit 101 终止。

## 最低共享层根因

`realtime_ibl_graph_plan/tests.rs` 构造 `CubeMipRange`，但当前测试作用域不再能解析该类型；同一环境模块可见的近似类型为 `CubeFaceRange`。渲染时间切片的类型可见性或测试导入在硬切换后未与图计划测试同步。

## 架构修复验收

- Render11 使用时间切片的规范 `CubeMipRange` 路径或正确的模块级可见性，保持 mip 范围与 face 范围为不同概念。
- Render11 的相关图计划测试可编译并通过。
- 原始 Text03 命令完成并运行 `text_oversized_run_keeps_one_logical_shaped_line`。
- Text03 随后重跑长文本 profiling 和真实 WGPU 产品帧缓冲导出。

## 禁止临时方案

- 不要以 `CubeFaceRange` 替换 mip 范围来消除编译错误。
- 不要加入别名、兼容重导出、测试专用旁路或放宽 Text03 验收。
- 不要跳过完整库测试或将构建通过视为 Text03 测试通过。

## 修复结果与回传

Open state: `待修复`; no pass is claimed.
