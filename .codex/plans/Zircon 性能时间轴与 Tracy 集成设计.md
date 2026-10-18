# Zircon 性能时间轴与 Tracy 集成设计

## Summary
- 在 `zircon_runtime::core::diagnostics` 下新增 `profiling` 子系统，作为 runtime/editor 共用的性能时间轴入口。
- 首版同时支持两种查看方式：外部 Tracy 实时查看，以及 Editor 内置 Performance Timeline 面板。
- 普通 `release` 构建不带 profiling 依赖和插桩；新增独立 `profiling` Cargo profile 用于优化构建下采样和分析。
- 每次 capture 输出文件到 `target/zircon-profiles/<session-id>/`：`timeline.zrtrace.json`、`timeline.perfetto.json`、`hotspots.json`、`summary.md`。

## Key Changes
- Cargo 编译开关：
  - `profiling`：启用 Zircon 本地时间轴 recorder、离线导出、Editor 面板数据。
  - `profiling-tracy`：启用 `tracing-tracy`/Tracy 实时 sink。
  - `profiling-chrome`：启用 Perfetto/Chrome trace JSON 导出。
  - `profiling-memory` 暂不进首版，保留 feature 名称但不实现内存采样。
  - `cargo build --release --features profiling*` 必须失败并提示使用 `cargo build --profile profiling ...`。
- 公共接口：
  - 新增 `zircon_runtime::core::diagnostics::profiling::{ProfileRecorder, ProfileScope, ProfileCaptureConfig, ProfileSnapshot, HotspotReport}`。
  - 新增宏：`profile_frame!`、`profile_scope!`、`profile_counter!`；feature 关闭时展开为空，不取时间、不分配。
  - `zircon_runtime_interface` 追加 profiling DTO 与一个可选 ABI 函数：`profile_control(session, request_json, out_json)`，用于 editor 查询动态 runtime 的 timeline snapshot/export report。
- 插桩点：
  - runtime：`RuntimeEntryApp` redraw/event、动态 ABI `handle_event/capture_frame/present_viewport`、`RuntimeRenderBridge`、render framework submit/present/capture、graph prepare/render/feedback/stats。
  - editor：`UiHostWindowEventLoop` redraw/input/about_to_wait、`RetainedEditorHost::tick/recompute_if_dirty`、viewport submit/poll、`SoftbufferHostPresenter::present/repaint/copy/present`。
  - core：module register/activate/deactivate、manager resolve 只加粗粒度 span，避免污染普通服务调用。
- 可视化与分析：
  - Tracy sink 使用 tracing spans 和 frame marks，外部 Tracy GUI 负责实时可视化。
  - Editor 新增 Performance Timeline 面板，合并 editor host timeline 与 runtime dynamic session timeline，显示 frame 条、span 树、top hotspots、frame budget 超限。
  - Hotspot analyzer 按 `stream/category/name/path` 聚合 `total/avg/p95/max/count/frame_count`，默认预算 runtime/editor 都按 16.67ms 标红；摘要只给基于已记录阶段名的优化提示，不编造原因。

## Milestones
- M1：基础 profiling spine
  - 加 feature/profile/build guard、no-op 宏、本地 recorder、Perfetto/native JSON 导出、hotspot analyzer。
  - 接 runtime preview 和 editor host 的帧边界与关键 CPU span。
- M2：Tracy 与动态 ABI
  - 接 `tracing-tracy`，补 Tracy frame mark。
  - 追加 runtime dynamic profiling control ABI，并让 editor 能拉取 runtime session snapshot。
- M3：Editor 可视化
  - 在现有 Runtime Diagnostics/Workbench 面板体系中加入 Performance Timeline。
  - 支持 live ring buffer、capture start/stop、export report 展示。
- M4：渲染深挖
  - 细化 render graph/pass span 与锁等待 span。
  - GPU timestamp/Tracy GPU 只作为后续扩展，不阻塞首版 CPU 时间轴交付。

## Test Plan
- Unit tests：recorder 嵌套 span、frame ring buffer、Perfetto JSON shape、hotspot p95/排序、ABI optional field 兼容、disabled feature no-op macro 编译。
- Editor tests：Performance 面板能显示 editor/runtime 两条 stream；feature 关闭时 UI 不轮询 profiling。
- Build gates：
  - `cargo build -p zircon_app --release --locked`
  - `cargo build -p zircon_app --release --features profiling --locked` 预期失败。
  - `cargo check -p zircon_app --profile profiling --features "target-editor-host profiling profiling-tracy profiling-chrome" --locked`
  - milestone 完成阶段运行 workspace validator：`.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1`
- Docs：更新 `docs/zircon_runtime/core/diagnostics/profiling.md`、`docs/zircon_editor/ui/performance-timeline.md`、`docs/zircon_runtime_interface/profiling.md`。

## Assumptions
- “release 不带上”解释为普通 `--release` 永远无 profiling；需要性能分析时使用独立 optimized `--profile profiling`。
- Tracy 负责实时交互查看；稳定落盘文件采用 Zircon JSON、Perfetto/Chrome trace JSON 和热点摘要。
- 参考依据：Rust `tracing-tracy`/`tracy-client` 文档、Perfetto Chrome trace 兼容格式，以及本仓库 `dev/bevy` 中 `trace_tracy/trace_chrome` 的 feature-gated span 方案。
