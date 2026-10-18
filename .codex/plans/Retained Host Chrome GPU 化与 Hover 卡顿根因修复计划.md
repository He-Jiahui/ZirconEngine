# Retained Host Chrome GPU 化与 Hover 卡顿根因修复计划

## Summary
方向确认：不继续把 `write_pixel`/`draw_rect_clipped` 做大，而是把 editor chrome 从“每帧软件重绘整块位图”改成“retained 状态 + 增量命令流 + GPU presenter”。第一阶段先切断 hover/move 期间的 `chrome_snapshot()` 和 `WorkbenchViewModel::build()`，第二阶段把 host chrome 输出成可缓存的 render commands，第三阶段接 GPU 后端。

参考路线：Unreal Slate 的 retained draw elements/invalidation、Fyrox 的 retained widget/message、Bevy 的 extract/layout separation、Slint 的状态驱动 scene/damage。

## Key Changes
- 建立 `HostPresentationCache`：
  - 常驻缓存 `EditorChromeSnapshot` 派生出的 workbench/model/layout/pointer-layout。
  - 按 domain 维护 generation：`structure/layout/pointer/visual/text/viewport/asset`。
  - `EditorChromeSnapshot` 只允许在结构或数据域变化时刷新，不允许 pointer hover/move 触发。

- 修复当前明确热点：
  - `welcome_recent_pointer.rs` 不再在 click/move/scroll 调 `runtime.chrome_snapshot()`。
  - Welcome recent、Hierarchy、Console、Inspector 等 pointer layout 都改成使用已提交缓存。
  - hover/press/focus 只更新小型 interaction state，并请求 paint-only/region damage。

- 引入 chrome 命令流：
  - 新增 backend-neutral `ChromeCommandStream`，表达 quads、borders、text glyph runs、images、clips、z-order、damage rect。
  - 静态层缓存：菜单栏、dock panel、tab、列表背景、文本。
  - 动态层 patch：hover、pressed、selected、focus ring、drag indicator、viewport image。
  - 旧 softbuffer painter 只保留为兼容 fallback，不再扩大职责。

- Presenter 分层：
  - 新增 `HostChromePresenter` trait。
  - `SoftbufferChromePresenter` 继续承接现有路径，方便测试和回退。
  - `GpuChromePresenter` 通过 runtime RHI/wgpu 抽象接入，避免 editor 到处直接写 raw wgpu。
  - GPU 路径按 command stream 合批绘制，使用 scissor/damage，避免全帧 CPU 像素循环。

- Dirty-domain 规则硬约束：
  - hover/move：禁止 `chrome_snapshot`、禁止 model rebuild、禁止 layout、禁止 full-frame。
  - click：只有结构变化才触发布局；selection/focus 走 visual dirty。
  - drawer resize：coalesce 到每帧一次 layout update，拖动中不重复全量 model rebuild。
  - viewport image：只更新 texture/damage，不触发 layout/presentation。

## Profiling Guards
- 在现有 UI profiling 基础上增加 counters：
  - `chrome_snapshot_count`
  - `workbench_model_build_count`
  - `chrome_command_full_rebuild_count`
  - `chrome_command_patch_count`
  - `software_fallback_present_count`
  - `gpu_upload_bytes`
  - `gpu_draw_calls`

- 热点判定：
  - hover 场景出现 `chrome_snapshot_count > 0` 标红。
  - hover 场景出现 `presentation_rebuild_count > 0` 标红。
  - region repaint 退化 full-frame 标红。
  - viewport image 触发 layout/presentation 标红。
  - drawer resize 每个 pointer move 都 slow path 标红。

## Test Plan
- 单测：
  - welcome recent hover/move 不调用 `runtime.chrome_snapshot()`。
  - hierarchy/menu/tab hover 只产生 visual dirty。
  - viewport image update 不触发布局或 presentation rebuild。
  - command stream patch 不重建静态层。

- 集成 profiling：
  - startup、idle hover、click、drag resize、viewport refresh 全部导出 `ui_hotspots.json`。
  - idle hover 验收：`chrome_snapshot_count=0`、`workbench_model_build_count=0`、`presentation_rebuild_count=0`。
  - resize 验收：每帧最多一次 layout-domain 更新。
  - viewport refresh 验收：只出现 render/texture update。

- 视觉验证：
  - softbuffer fallback 与 GPU presenter 截图对比。
  - Unreal 风格 chrome 重绘前后保持 dock/tab/panel/toolbar 几何一致。
  - hover/pressed/focused Material/Unreal 风格状态能动态变化。

## Assumptions
- 长期主线采用 GPU command stream，不再继续扩展软件 painter。
- 短期仍保留 softbuffer fallback，直到 GPU presenter 功能和截图测试稳定。
- 优先修复交互卡顿根因：先禁止 pointer hover/move 拉完整 snapshot，再做 GPU 化。
- editor 不新增独立渲染体系，优先复用 runtime RHI/wgpu 抽象。
