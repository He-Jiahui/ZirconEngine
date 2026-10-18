# Runtime 渲染风险清单与 RenderDoc 调试支持计划

## Summary
- 当前重影最高风险仍是默认 3D 管线启用 `HistoryResolve`，但历史帧复用缺少 motion vectors、重投影、camera cut、disocclusion 校验。
- 当前渲染架构方向合理，但实现仍处在过渡态：render graph 已存在，真实 pass 执行仍大量落在 `SceneRenderer` 固定流程里。
- RenderDoc 支持应优先走 wgpu 29 的公开 API：`Device::start_graphics_debugger_capture()` / `stop_graphics_debugger_capture()`，不直接绑定 `renderdoc-sys`，避免额外 ABI 风险。

## Risk Register
- **P0 Temporal Ghosting**：`HistoryResolve` 默认启用且固定混合上一帧颜色，历史有效性只看 viewport/pipeline/binding；相机移动、物体移动、resize 后下一帧都可能残留旧画面。
- **P0 History Slot Semantics**：history 名义是 scene color，但复制源和 postprocess 阶段存在语义混杂风险；一旦 bloom/GI/color grading 写入 history，下一帧会把后处理结果再混入当前 scene。
- **P0 Clear/Load Boundary**：部分 pass 使用 `LoadOp::Load` 依赖前序 pass 已清屏；只要某个 profile/feature 跳过清屏 pass，offscreen texture 复用就可能产生残影。
- **P1 Render Graph Ownership**：graph 当前更像计划/统计层，真实资源生命周期、copy、postprocess、overlay 仍由固定路径管理；这会削弱 Unity SRP/RenderGraph 式的 pass 依赖验证。
- **P1 ECS Extract Staleness**：如果 transform propagation、visibility、UI resolve、render extract 顺序不稳定，渲染会消费上一状态或半更新状态；应对齐 Bevy 的 PostUpdate -> Extract 边界。
- **P1 Feature Gating**：高级效果默认过多，尤其 temporal/history 类效果没有 capability gate；默认质量应优先无重影，temporal 效果应显式 opt-in。
- **P1 wgpu Backend Config**：当前 offscreen backend 使用 `Instance::default()`，设备 `trace` 固定为 `Off`，缺少统一的 backend/env/debug 配置入口。
- **P1 Debuggability**：已有一些资源和 pass label，但缺少帧级 debug group、stage marker、RenderDoc capture request、当前 pipeline/profile/history 状态记录。
- **P2 Performance Ceiling**：普通运行路径若每帧 GPU->CPU readback，会限制 wgpu 性能上限；截图/测试 readback 应与真实呈现路径分离。
- **P2 RHI Confusion**：`rhi_wgpu::WgpuRenderDevice` 当前偏内存模拟设备，真实 wgpu backend 在 graphics backend；命名和职责容易让后续 SRP/RHI 收束跑偏。

## RenderDoc Support
- 新增渲染调试接口：在 `RenderFramework` 暴露 `request_graphics_debugger_capture(viewport)` 和 `query_graphics_debugger_status()`。
- 新增状态类型：`GraphicsDebuggerStatus { available, backend_name, capture_pending, active_capture, last_capture_frame, last_error }`，只报告状态，不强依赖 RenderDoc DLL。
- 在下一次目标 viewport 提交时执行 capture：录制命令前调用 `device.start_graphics_debugger_capture()`，提交队列后等待/同步调试帧，再调用 `stop_graphics_debugger_capture()`。
- 增加环境开关：`ZR_RENDERDOC_CAPTURE_NEXT=1` 触发启动后一帧捕获；`WGPU_BACKEND=dx12|vulkan` 选择 RenderDoc 后端；`WGPU_DEBUG=1`、`WGPU_VALIDATION=1` 打开 wgpu 调试信息。
- 增加 debug markers：按 `FrameExtract`、`Clear`、`Prepass`、`MainScene`、`DeferredLighting`、`PostProcess`、`HistoryCopy`、`Overlay`、`UI`、`Readback` 分组，保证 RenderDoc 捕获可读。
- 增加文档：记录 Windows 下从 RenderDoc 启动 Zircon、通过环境变量选择 DX12/Vulkan、触发单帧捕获、定位 history texture 和 postprocess pass 的步骤。

## Test Plan
- 静态测试：确认 RenderDoc/debugger API 不改变默认渲染行为，未请求 capture 时不调用 unsafe debugger capture。
- 行为测试：请求 capture 后只捕获下一帧，成功/失败都清空 pending 状态，并记录 `last_error`。
- 渲染风险测试：移动相机、移动物体、resize、切换 pipeline/profile 时验证 history 不复用非法上一帧。
- 架构测试：统计 pass labels/debug groups 覆盖关键阶段，确保 history copy、postprocess、overlay/UI 顺序可从日志或 RenderDoc 中对应上。
- 验证命令：优先跑 `cargo test -p zircon_runtime --locked render_framework_bridge project_render`，RenderDoc 手工捕获作为桌面验证项。

## Assumptions
- RenderDoc 支持先按 Windows 桌面开发场景设计，DX12 和 Vulkan 都允许，默认由 `WGPU_BACKEND` 选择。
- RenderDoc 是开发调试能力，不进入 release 默认路径，不作为运行时硬依赖。
- 本轮仍是计划与风险整理，不直接修改代码。
