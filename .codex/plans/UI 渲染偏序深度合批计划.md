# UI 渲染偏序深度合批计划

## Summary
- 在现有 “GPU Command Stream 接管 Editor UI 渲染” 路线上继续推进，范围限定为 `zircon_editor` retained host command stream 到 `zircon_runtime` WGPU UI surface；不改 `zircon_hub`/Slint。
- 目标是把当前稳定全序绘制改成偏序约束下的深度层级合批：重叠元素保持现有像素顺序，不重叠元素允许重排以减少真实 GPU draw calls。
- 验收标准：视觉与 softbuffer/旧全序路径一致；profile 同时报告可见命令数和实际 GPU batch draw calls；startup/hover/viewport patch 的实际 draw calls 明显低于可见命令数，且不出现 software fallback。

## Key Changes
- 新增 runtime 内部批处理规划模块，建议落在 `zircon_runtime/src/rhi_wgpu/ui_surface/batching.rs`，由 `geometry.rs` 只负责生成裁剪后的可绘制 item。
- 每个 item 记录：原始稳定顺序 `(z_index, command_index, sub_index)`、有效矩形、裁剪后顶点/UV、材质 key、命令类型。`Border` 展开为 solid items；`Clip` 仍只参与统计，不生成 draw item。
- 偏序关系定义为：若 A 在旧全序中先于 B，且 A/B 的有效矩形相交，则添加 `A -> B` 依赖；不相交则视为不可比，可进入同一深度层并重排合批。
- 使用拓扑分层生成 `BatchDrawPlan`：逐层渲染保持重叠顺序；层内按材质合批。Solid 同层合为一个 draw；Text 同层合为一个 glyphon batch；Image 首版按相同 `resource_key` 合批，不做跨纹理 atlas。
- Solid/Image 不再依赖逐 item scissor 来保证裁剪；改为在 CPU 侧把 quad 顶点和 image UV 裁到有效矩形，这样同层同材质可以一次 draw。Text 继续使用 glyphon 每个 `TextArea` 的 bounds。
- `UiSurfacePresentStats` 增加真实统计：保留/新增可见命令计数，`draw_calls` 改为实际提交的 GPU batch draw calls，并增加 `batch_layer_count`、`batch_dependency_count`。`GpuChromePresenter` 的 `gpu_draw_calls` 改用真实 batch 数。
- 更新 hotspot/profile 输出：避免继续把 command count 误标成 draw calls；新增合批退化检测，例如 batch 数接近可见 item 数时给出警告。
- 更新 RHI UI surface 与 retained-host performance 文档，并把该阶段追加到现有 GPU command stream 计划的进度/验收记录。

## Test Plan
- Runtime planner 单测：不相交 quads 合到同层同 draw；重叠 quads/text/image 保持旧 z/index 顺序；同 z 重叠保持原始 index 顺序；damage/clip 后相交关系按有效矩形计算。
- Geometry 单测：solid 裁剪顶点正确；image 裁剪后 UV 正确；不同 `resource_key` image 分批，相同 key 合批；disjoint damage 不生成 item。
- Text 单测：同层多个 text command 进入同一 glyphon batch；与重叠 geometry 有依赖时拆层；clip/damage bounds 不泄漏。
- Presenter/profile 单测：headless `WgpuUiSurfacePresenter` 返回真实 batch stats；`GpuChromePresenter` 记录实际 `gpu_draw_calls`；`ui_hotspot` 同时识别 no-draw 和 batch 退化。
- Parity 验证：保留现有 command stream 与 softbuffer 像素对比，并新增混合 quad/image/text、重叠与非重叠并存的回归 fixture。
- Focused validation：`cargo test -p zircon_runtime --lib ui_surface --locked --jobs 1`、`cargo test -p zircon_editor --lib command_stream --locked --jobs 1`、`cargo test -p zircon_editor --lib gpu_presenter --locked --jobs 1`、`cargo test -p zircon_runtime --lib ui_hotspot --locked --jobs 1`、`cargo check -p zircon_app --features target-editor-host --locked`，再跑 startup/idle_hover/viewport_image profile capture。

## Assumptions
- 首版优先“像素正确 + 可观测 draw-call 下降”，不引入 GPU depth/stencil；alpha UI 仍由 CPU 偏序层保证确定性。
- Image atlas/texture array 是后续优化，不进入本轮；本轮只合批同 `resource_key` image。
- `ChromeCommandStream` 公共命令格式不变；合批完全由 runtime WGPU presenter 消费端完成。
- 若 Windows debug 全 workspace 测试继续超时，接受以 runtime/editor/app focused gates 加 profile evidence 作为本阶段闭环，并记录 workspace blocker。
