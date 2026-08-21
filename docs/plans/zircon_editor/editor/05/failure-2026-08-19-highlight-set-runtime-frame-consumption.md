---
handoff_kind: failure
status: open
created_at: 2026-08-19
summary_slug: highlight-set-runtime-frame-consumption
origin_plan: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
fixing_plan: docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
origin_child_dir: docs/plans/zircon_editor/editor/05
fixing_child_dir: docs/plans/zircon_runtime/runtime/10
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/workbench/state/editor_state_render.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/render_submission.rs
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/core/framework/render/viewport_highlight_store.rs
  - zircon_runtime/src/dynamic_api/session/extract.rs
  - zircon_runtime/src/dynamic_api/session/extract_cache.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/dynamic_api/runtime_loop.rs
tests:
  - runtime dynamic-session highlight submission is present in the submitted RenderFrameExtract for the addressed viewport
  - equal selection generation with changed render attributes invalidates the runtime overlay projection
  - retained editor host and session renderer use an explicit runtime-to-renderer viewport binding
---

# Editor05 -> Runtime10: HighlightSet gateway 输入未进入实际帧生产

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md`
- 来源执行切片：M4.2 `HighlightSet` 正式化
- 修复责任计划：`docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md`
- 交接原因：Editor01 已提供 gateway/ABI/latest-value store；但动态 session 与 retained renderer 的实际 frame-production 路径仍未消费该 store。Editor05 不能以 UI 直接构造第二份 overlay 状态来伪造该连接。

## 失败现象与复现证据

当前 gateway 的 in-process 与 session 实现把 `EditorRuntimeHighlightSet` 写入
`LevelSystem::ViewportHighlightStore`。静态调用图只有 submit 和测试读取者：
`RuntimeDynamicSession::current_extract()` 仍只调用
`RuntimeFrameExtractCache::current_extract(&level, viewport_size)`，后者由 `World`
生成 extract 并直接返回；没有读取 `viewport_highlight_set` 或向
`RenderFrameExtract.debug.overlays` 投影。

随后 runtime session 的 `capture_frame` 和 `present_viewport` 将该 extract 直接提交给
`RuntimeRenderBridge`。retained editor host 也独立调用 `RenderFramework::submit_frame_extract_with_ui`，
其 renderer viewport handle 没有与 `ZrRuntimeViewportHandle` 建立映射。因此仅提交 gateway
不会让任何实际渲染帧消费 HighlightSet；而 editor 同时提交 gateway 和直接填充 frame extract
会形成两个输入通道，不符合 M4.2 的单一 runtime-owner 要求。

## 最低共享层根因

`ViewportHighlightStore` 的 owner 是 `LevelSystem`，实际 renderer 的 owner 是
`RenderFramework`；二者之间没有 frame-production bridge。当前的 runtime viewport ABI handle
也没有和 retained renderer 的 `RenderViewportHandle` 显式绑定。并且高亮 DTO 的 generation
当前仅来自 selection revision，不能表达 display mode/tint 等 render-attribute 变化，不能安全作为
extract cache 的唯一失效键。

## 架构修复验收

- Runtime10 在 runtime-owned frame production 中，以 runtime viewport 读取 latest accepted
  `HighlightSet`，并在提交 renderer 前一次性投影到中立 `RenderFrameExtract`；Editor05 不再直接
  填充该 overlay。
- `RuntimeFrameExtractCacheKey` 包含 viewport identity 和独立的 overlay revision，属性变化与实体
  集合变化均会失效；旧 generation 不得覆盖新值，且稳定帧不得扫描 world 或重复构造 overlay。
- 明确定义 runtime viewport ABI handle 与 `RenderViewportHandle` 的创建/绑定/销毁生命周期；retained
  host 与 session path 使用同一映射语义，不得用默认 handle、global cache 或 call-site fallback。
- 受管测试覆盖 session 和 retained/in-process 路径的多实体、viewport 隔离、stale generation、属性
  变化以及 renderer-visible frame extract；再执行 Editor05 和 Runtime10 上行门。
- 完成后 Editor05 删除任何直接从 `EditorRuntimeHighlightSet` 填充 `RenderOverlayExtract` 的代码，仅
  保留一次 gateway 提交和 anchor/handle/gizmo 的 frame-local projection。

## 禁止临时方案

- 不得保留 gateway store 与 editor-built `RenderOverlayExtract.highlights` 双写，或通过 copy/alias
  隐藏两条通道。
- 不得让 renderer 在热路径直接持有 `LevelSystem`、锁 world，或每帧 clone 完整 scene。
- 不得将 selection revision 假定为 render-attribute revision，或用 hash 替代单调 generation。
- 不得通过全局单 viewport cache、默认 ABI handle或仅测试 callback 掩盖 viewport 生命周期。

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 证据与后续 |
|---|---|---|---|
| 2026-08-19 | `open / Runtime10 frame-production bridge required` | 完成 Editor05 consumer、gateway store、dynamic-session extract cache、runtime bridge 与 retained-host render submit 的静态调用图复核；确认旧 `SelectionHighlightExtract`/`overlays.selection` 可硬切，但完整 M4.2 不能以双通道实现验收。 | 本次为结构缺口复核，未执行性能优化或声称性能数据。Runtime10 先完成单通道与 viewport 映射，再由 Editor05 删除临时 frame-local HighlightSet 投影并进行受管验证。 |
| 2026-08-19 | `in_progress / Editor05 direct projection hard-cut` | 已删除 `EditorRuntimeHighlightSet -> RenderOverlayExtract.highlights` 的临时直写；`render_frame_submission` 现在只向 runtime gateway 提交一次，editor-built snapshot 明确保留 `highlights: None`。同步调整 editor 和 runtime 边界断言，防止该旁路回归。 | `rustfmt` 与受影响路径静态 token 搜索通过。受管离线 `cargo test -p zircon_editor --lib viewport --locked --offline --jobs 1 -- --test-threads=1` 在编译前因本机缓存缺少 `image` 依赖退出（101），不构成代码通过证据；M4.2 仍须等待 Runtime10 的单通道 frame-production bridge 后复验。 |
