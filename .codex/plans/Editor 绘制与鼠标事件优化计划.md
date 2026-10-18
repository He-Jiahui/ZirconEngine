# Editor 绘制与鼠标事件优化计划

## Summary

当前卡顿的主要原因不是单点命中函数，而是 editor native host 仍按“全量刷新”运行：`tick()` 频繁进入粗粒度重算，`apply_presentation()` 重建完整展示模型，softbuffer presenter 即使收到局部 damage 也整帧绘制、整帧拷贝。参考 Unreal Slate 的 `FSlateInvalidationRoot`、`FHittestGrid`、`FWidgetPath`，本次优化目标是建立 editor/runtime 共用的“缓存布局 + 局部失效 + 命中路径路由”机制。

## Key Changes

- 引入 editor host invalidation root：把当前 `layout_dirty`、`render_dirty`、`presentation_dirty` 拆成明确原因，例如 `Layout`、`TreeStructure`、`PresentationData`、`PaintOnly`、`PointerHover`、`ViewportImage`、`HitTest`、`WindowMetrics`。只有结构、布局或数据变化走 slow path；鼠标 hover、scroll、局部状态变化只走 paint fast path。

- 改造 presenter 为真实局部绘制：保留上一帧 `HostRgbaFrame` 作为 backbuffer；首帧、resize、布局变化整帧绘制；hover、菜单高亮、pane scroll、viewport image 更新只重绘 damage 矩形，并且只把受影响行段写入 softbuffer，再调用 `present_with_damage`。现有 `Region` 不再只是提交阶段的提示。

- 缓存模板与投影结果：`EditorUiHostRuntime` / builtin template documents 不再在辅助函数或 floating window sync 中重复创建、重复加载；改为 host 级共享缓存。`UiSurfaceFrame` 按 document、size、model revision 缓存，尺寸或模型没变时复用 arranged tree、render extract、hit grid。

- 降低 `apply_presentation()` 成本：pane 投影只构建当前 pane kind 需要的数据，不再每次为每个 pane 计算所有 variant。pointer-only 状态从完整 presentation 重建路径中剥离，作为 paint-only interaction state 更新。

- 统一 native host 命中：建立 host-level surface registry，把 root shell、menu、pane、floating window、viewport toolbar、template surface 都注册为可命中的 arranged surface。鼠标事件只查询 hit grid 得到 `HostHitPath`，再按 bubble/tunnel/direct 策略分发；删除 toolbar 高度、按钮坐标、chrome loop 等手写命中路径。

- 鼠标事件 fast path：保存 hover path 和 capture path。鼠标移动时，如果 leaf 未变且没有 scroll/capture 状态变化，直接返回 idle，不请求绘制；hover enter/leave 只 damage 旧目标和新目标区域。toolbar/action 激活保持 `PrimaryPress`，release、secondary、middle 不触发 command。

- 调整刷新调度：16ms tick 只轮询外部变化并合并 invalidation，不再无条件推进 presentation/layout。`request_frame_update` 只处理当前 pending invalidation stage，避免从 redraw 回调重新进入完整 tick。

- 增加诊断计数：记录每秒 template load、presentation rebuild、layout rebuild、hit-grid rebuild、full paint、region paint、painted pixels、pointer routed/ignored。日志用于确认“刷新不再全量加载”。

## Test Plan

- presenter 单元测试：局部 damage 只重绘和拷贝对应区域；resize/layout dirty 强制整帧。
- invalidation 测试：pointer hover/scroll 不调用 `apply_presentation()`；layout/data 变化才触发 slow path。
- template cache 测试：启动后连续刷新和鼠标移动不会再次调用 builtin template load。
- hit routing 测试：toolbar、pane、floating window、menu 都通过 shared hit grid 命中；新增 `.ui.toml` button 不改 Rust 坐标即可命中。
- mouse activation 测试：primary press 只触发一次；release、secondary、middle 不触发 toolbar command。
- 性能验收：100 次同目标 mouse move 应为 0 次 presentation rebuild；hover 切换只产生局部 paint；editor log 中启动后不再反复出现 template compile/load。
- 构建/测试命令使用 `E:\zircon-build\targets` 作为 target dir，先跑 focused editor/native host tests，再跑 runtime/editor 相关包测试。

## Assumptions

- 继续采用完整 Slate 化路线，不保留 toolbar 专用坐标 shim。
- 当前 softbuffer 后端先保留，本轮优化不切换 GPU UI renderer。
- world-space UI 的射线拾取仍由 scene/editor viewport 层负责；进入具体 2D surface 后统一走 `UiHitTestGrid`。
- 这是一项跨 runtime/editor 的结构性优化，先保证 editor 卡顿路径收敛，再把可复用契约下沉到 runtime shared UI。

---

## Deep Optimization Stage: External Damage And Idle Invalidation

### Goal

继续收敛 editor native host 的“刷新即全量”路径。上一阶段已经让 pointer hover 和 softbuffer presenter 支持区域 fast path；这一阶段把非鼠标来源的局部变化也纳入同一套 damage 调度，避免 viewport image、重复状态行、重复 redraw 请求把 host 拉回完整 `tick()` / presentation rebuild。

### Reference Shape

- Unreal `FSlateInvalidationRoot`：失效原因分层，paint-only dirty 不触发布局/结构重建。
- Unreal `FSlateApplication` / `FWidgetPath`：事件路由先更新输入状态，再只对受影响 widget path 请求 repaint。
- Unreal `FHittestGrid`：render/hit 使用同一份 arranged geometry；本阶段继续沿用已有 `UiSurfaceFrame.hit_grid`，不回退到手写 toolbar 坐标。

### Milestone D1: Host-level external redraw queue

**Implementation files**
- `zircon_editor/src/ui/slint_host/host_contract/globals.rs`
- `zircon_editor/src/ui/slint_host/host_contract/window.rs`
- `zircon_editor/src/ui/slint_host/host_contract/redraw.rs`

**Work**
- 在 host contract state 中加入外部 pending redraw 槽，供 app 层提交 paint-only `Region` 或 `Full { frame_update }`。
- event loop 在 `about_to_wait` drain 外部 redraw，并与本地 pointer/window pending redraw 合并。
- 多个 region 合并成 union damage；full redraw 覆盖 region；`frame_update=false` 的 full redraw 不调用 `request_frame_update()`。
- 增加诊断计数：external redraw queued/drained/coalesced。

**Acceptance**
- app 层可以请求 region repaint，而不需要制造 pointer event 或调用 full frame update。
- repeated region requests 在一次 event-loop wait 前合并成一个 damage。

### Milestone D2: Viewport image paint-only fast path

**Implementation files**
- `zircon_editor/src/ui/slint_host/app.rs`
- `zircon_editor/src/ui/slint_host/app/host_lifecycle.rs`
- `zircon_editor/src/ui/slint_host/app/viewport_image_redraw.rs`
- `zircon_editor/src/ui/slint_host/host_contract/globals.rs`

**Work**
- 把 `tick()` 中的 `viewport.poll_image()` 处理拆到 `viewport_image_redraw.rs`，避免继续扩张 oversized `host_lifecycle.rs`。
- `poll_image()` 得到新 image 时只更新 `HostViewportImageData`，再对当前 viewport content frame 请求 `Region` redraw。
- 该路径不设置 `presentation_dirty`、`layout_dirty` 或 `render_dirty`，也不触发 `request_frame_update()`。
- 如果 viewport frame 为空或 image 无效，则不提交 redraw。

**Acceptance**
- viewport 连续出图时，native host 只 repaint viewport damage 区域。
- viewport image 更新不增加 presentation rebuild 计数。

### Milestone D3: Duplicate status/invalidation guard

**Implementation files**
- `zircon_editor/src/ui/host/editor_event_runtime_access.rs`
- `zircon_editor/src/ui/slint_host/app/host_lifecycle.rs`

**Work**
- 在 `EditorEventRuntime` 暴露轻量 `status_line()` 读取，不构建完整 chrome snapshot。
- `SlintEditorHost::set_status_line()` 在消息未变化时直接返回，不设置 `presentation_dirty`。
- 保留真实状态变化的 presentation invalidation，避免静默丢 UI 更新。

**Acceptance**
- 重复错误或重复状态消息不会每 16ms 触发 presentation rebuild。
- 状态行实际变化仍刷新 host presentation。

### Milestone D4: Documentation And Validation

**Documentation files**
- `docs/ui-and-layout/slate-style-ui-surface-frame.md`
- `docs/editor-and-tooling/editor-workbench-shell.md`
- `tests/acceptance/shared-slate-ui-surface-frame.md`

**Validation commands**
- `cargo fmt --all --check`
- `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never`
- `cargo test -p zircon_editor --lib tests::host::slint_window::native_host_contract --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture`
- `cargo test -p zircon_editor --lib presenter::tests --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture`
- `python tools/zircon_build.py --targets editor,runtime --out E:\zircon-build --mode debug`

**Acceptance**
- 计划、实现文件、测试命令和剩余风险记录到 docs/acceptance。
- 不声明 workspace-wide green；只声明实际跑过且通过的 scoped commands。
