# Retained Host Chrome 性能根因修复计划

## Summary
源码是可定位的，不需要额外提供文件：热点在 `zircon_editor/src/ui/retained_host/host_contract/painter/primitives.rs` 的 `write_pixel`，拷贝热点在 `presenter.rs` 的 `copy_rgba_to_softbuffer`，更上游的重建热点在 `app/host_lifecycle.rs` 的 `recompute_if_dirty`。结论是两层问题叠加：软件像素 painter 很贵，但更关键的是交互路径仍可能把 hover/move 升级成 chrome snapshot/model/presentation 重建。

本轮目标不做视觉重构，先把性能闭环建立成可验证的修复链：阻断 pointer 期间全量 snapshot/rebuild，再引入 retained chrome 命令流，最后把 softbuffer painter 降级为 fallback，并准备 GPU presenter。

## Key Changes
- 增加 profiler guard：
  - 利用现有 profiling counters 和 `E:\Git\ZirconEngine\dev\tracy`。
  - 新增或补齐 `chrome_snapshot_count`、`workbench_model_build_count`、`chrome_command_full_rebuild_count`、`chrome_command_patch_count`、`software_fallback_present_count`。
  - hover/move 场景只要出现 snapshot/model build/presentation rebuild 就判定失败。

- 先修上游重建：
  - `welcome_recent_pointer.rs` 不再在 click/move/scroll 中调用 `runtime.chrome_snapshot()`。
  - pointer layout 使用已提交缓存；欢迎页 recent list、hierarchy、console、inspector 都按 generation 更新。
  - `recompute_if_dirty` 的 paint-only 分支必须成为 hover/press/focus 的默认路径。

- 建立 retained chrome 缓存：
  - 新增 `HostPresentationCache`，缓存 chrome snapshot 派生出的 model、layout、pointer layout、render command。
  - dirty domain 固定为 `structure/layout/pointer/visual/text/viewport/asset`。
  - pointer hover 只更新 visual state 和 damage region，不触发布局或 presentation。

- 引入 GPU-ready 命令流：
  - 新增 `ChromeCommandStream`：quad、border、text glyph run、image、clip、z-order、damage rect。
  - 先用 softbuffer backend 执行命令流，保证行为一致。
  - 后续新增 `HostChromePresenter` trait，落两个实现：`SoftbufferChromePresenter` fallback 和 `GpuChromePresenter`。
  - GPU 后端优先复用 runtime RHI/wgpu 抽象，避免 editor 内部散落 raw wgpu 逻辑。

- 软件 painter 只做战术收尾：
  - `write_pixel`/`copy_rgba_to_softbuffer` 可以加块拷贝、行级 fast path、减少 slice bounds check。
  - 这些优化只作为 fallback 减压，不作为主线架构目标。

## Test Plan
- Profiling 验收：
  - startup、idle hover、click、drag resize、viewport refresh 都生成 Tracy/Chrome trace 和 `ui_hotspots.json`。
  - idle hover 要求：`chrome_snapshot_count=0`、`workbench_model_build_count=0`、`presentation_rebuild_count=0`。
  - viewport image 要求：只触发 render/texture dirty，不触发 layout/presentation。
  - resize 要求：拖动中每帧最多一次 layout-domain 更新。

- 单元/集成测试：
  - welcome recent hover 不调用 `chrome_snapshot()`。
  - paint-only dirty 不触发布局和 hit-test grid rebuild。
  - command stream patch 不重建静态层。
  - softbuffer fallback 与 command stream 输出截图一致。

- 基准：
  - 新增小型 painter benchmark 只用于量化 fallback。
  - 主要性能指标以场景 profiler 为准，而不是单独 `write_pixel` 微基准。

## Assumptions
- 长期方向是 GPU command stream，不继续扩大软件 painter。
- 当前优先级是先切断交互期全量重建，再替换渲染管线。
- softbuffer 保留为兼容 fallback，直到 GPU presenter 和截图测试稳定。
- 本轮不改 Unreal 风格视觉，只做性能检测和架构性能修复入口。
