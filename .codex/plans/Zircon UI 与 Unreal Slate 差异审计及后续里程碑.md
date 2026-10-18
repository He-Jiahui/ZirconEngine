# Zircon UI 与 Unreal Slate 差异审计及后续里程碑

## Summary

- 审计基线：当前工作区状态，包括已修改和未跟踪 UI 文件；未把 Git 提交态当作唯一真相。
- 当前 Zircon UI 已具备 Slate 雏形：`UiSurfaceFrame`、arranged tree、hit grid、render extract、基础 pointer routing、`.ui.toml` Material 元组件、编辑器 Slint host 桥接、运行时 WGPU UI 渲染起点。
- 目标形态：`.ui.toml` 负责布局和组件描述；`zircon_runtime_interface::ui` 负责中立 UI DTO；`zircon_runtime::ui` 负责布局、命中、事件、渲染抽取和运行时 UI 服务；`zircon_editor` 只做编辑器宿主和 Slint Material 风格呈现。
- Unreal 参考主线：`SWidget`、`FArrangedWidget`、`FHittestGrid`、`FReply`、`FSlateDrawElement/FSlateElementBatcher`、`SlateInvalidationRoot`、`FSlateFontCache`、`FSlateTextShaper`、`FTextLayout/FSlateTextLayout`、`FSlateEditableTextLayout`、`WidgetReflector`。

## 当前与 Slate 的具体差异

| 领域 | Unreal Slate | 当前 Zircon UI | 差异/未实现 |
|---|---|---|---|
| 架构真源 | C++ live widget tree，`SWidget` 虚函数驱动行为 | `.ui.toml`/descriptor 驱动 retained tree，SurfaceFrame 做跨层 DTO | 这是有意设计差异；但缺少完整 widget 生命周期、属性失效、反射、默认行为体系 |
| Widget/组件 | 大量 `SWidget/SPanel` 子类、slot API、per-widget override | `UiTreeNode`、component catalog、Material meta components | 组件描述较全，但很多组件还只是 descriptor/showcase，运行时行为未闭环 |
| 布局 | desired size prepass、`FGeometry`、arranged children、layout/render transform、丰富 panels | 简单 measure/arrange，支持 overlay/linear/scrollable/virtual window | 缺少 transform、pixel snapping、复杂 slot policy、grid/flow 完整算法、DPI/scale 策略 |
| 渲染 | `OnPaint` 产出 `FSlateDrawElement`，再批处理为 render batches | `UiRenderCommand` 仅有 Group/Quad/Text/Image；运行时已有 WGPU pass 和文本路径 | 缺少 brush/material/custom verts/9-slice/圆角边框、材质合批、drawcall/overdraw 调试 |
| 文本 | `FSlateFontCache` 提供 shaped glyph sequence 和 atlas lookup；`FSlateTextShaper` 在 BiDi、HarfBuzz/full shaping、kerning-only path 间选择；`FTextLayout/FSlateTextLayout` 管 line model、run、wrap、dirty state、paint；`FSlateEditableTextLayout` 管 caret、selection、composition、IME context、command list | `zircon_runtime_interface::ui::surface::render` 已扩展 text DTO：direction、overflow、range、line metrics、rich runs、editable caret/selection/composition；`zircon_runtime::ui::text` 负责 shared layout/measure/rich marker/edit action；renderer 有 glyphon/SDF 绘制能力 | 已从裸字符串/近似行框推进到 shared text contract；仍缺 HarfBuzz 级 shaping、真实 font fallback measure、glyph reordering、平台 IME 接入、富文本 style paint、selection/caret 绘制 |
| 输入事件 | `FSlateApplication` 路由 path，`FReply` 表达 capture/focus/drag/drop/nav 等副作用 | pointer down/move/up/scroll、hover/focus/capture/click 基础链路 | 缺少 keyboard/text/gamepad/nav、drag/drop、popup、double click、tunnel/preprocess、完整 reply/effect |
| 命中测试 | `FHittestGrid` 支持 bubble path、多用户/窗口、复杂可见性和调试 | fixed cell hit grid，支持 z/order/clip/visibility/reject debug | 缺少多窗口/多用户、cursor radius、自定义 hit path、world-space/3D 命中 |
| 失效与性能 | invalidation root、fast path、batch merge、debug events | dirty flags + render extract rebuild + editor region redraw 起点 | 缺少 Slate 级 invalidation tree、局部缓存、批处理统计和可视化 |
| 调试工具 | Widget Reflector、SlateDebugging、live/snapshot widget path | `UiHitTestDebugDump` 和少量 redraw/debug 数据 | 需要实现 Widget Reflector 类工具，以及 drawcall、overdraw、材质合批、hittest 可视化 |

## 渲染/布局/文本/输入未完成项

- 渲染：扩展 `UiRenderCommand` 为 brush/material/border/rounded rect/line/vector/custom verts；接入真实 image/icon atlas；实现材质合批和 drawcall 统计；实现 overdraw/debug pass；统一编辑器 painter 与运行时 renderer 的命令语义；补齐 runtime UI service 到渲染帧的稳定路径。
- 布局：补齐 Slate 风格 desired-size prepass、arranged geometry、clip/layout/render transform、DPI scale、pixel snapping；完善 grid/flow/virtual list/scroll anchoring/slot 规则；确保 hit、render、input 只消费同一个 arranged frame。
- 文本：以 Slate 的 `FontCache -> TextShaper -> TextLayout -> EditableTextLayout` 分层继续收敛。当前已落地 shared DTO 和 runtime text foundation：wrap/alignment/ellipsis、direction 标记、source/visual range、line metrics、rich runs、editable caret/selection/composition/action 合同。下一步把 measurement 从字符 advance 升级为 glyphon/cosmic-text 或 HarfBuzz-backed shaped runs；补齐真实 font fallback、BiDi visual reordering、rich style painting、selection/caret drawing、IME platform context、composition underline；保证运行时 glyphon/SDF 与 editor host 文本结果一致。
- 输入事件：建立 `FReply` 对等的 `UiDispatchReply/Effect` 模型；补齐 keyboard、text input、IME、gamepad/navigation、drag/drop、popup/menu、tooltip、double click、mouse capture/lock/high precision；移除 editor host 中手写命中和特殊 pointer 分支。
- 调试：实现 Widget Reflector 类 live/snapshot 树、命中路径、拒绝原因、paint/layout/invalidation 状态；增加 drawcall、overdraw、材质合批、hittest grid overlay；支持导出 snapshot/replay 以复现 UI 问题。

## 后续里程碑

1. **M0 基线冻结与差异归档**：确认当前未跟踪 UI 文件是否进入基线；整理 Slate/Slint/Zircon 三方映射表；为每个子系统建立最小验收用例。
2. **M1 Shared Slate Core 收敛**：让 layout、render extract、hit grid、input routing 全部以 `UiSurfaceFrame` 为唯一输入；补齐 visibility、clip、z-order、dirty flags 的边界测试。
3. **M2 Material `.ui.toml` 原子组件**：以 `dev/slint` Material 为编辑器视觉参考，建立 `.ui.toml` meta component 对应关系；补齐 button、field、list、menu、dialog、navigation、surface 等基础行为和状态。
4. **M3 编辑器 Host Cutover**：Slint 只保留通用宿主/Material 呈现；菜单、drawer、toolbar、template nodes、document panes、floating panels 全部改为 `.ui.toml` + shared surface；删除业务化 `.slint` 和手写命中路径。
5. **M4 运行时 UI 独立能力**：运行时加载同一 `.ui.toml`，通过独立 renderer 渲染 HUD、pause menu、settings、inventory 等 fixtures；不依赖 Slint；接入字体、图像、材质资源。
6. **M5 完整事件系统**：实现 reply/effect、focus/capture、keyboard/text/IME、nav/gamepad、popup、drag/drop；编辑器和运行时使用同一事件语义。
7. **M6 文本系统收敛**：沿 `zircon_runtime_interface::ui::surface::render` 中立 DTO 和 `zircon_runtime::ui::text` shared foundation，统一测量、shaping、布局、绘制、编辑状态；补齐 HarfBuzz/advanced shaping、font fallback、BiDi visual order、rich text style runs、selection/caret 绘制、IME/composition 平台接入验收。
8. **M7 性能与调试工具**：实现 Widget Reflector 类工具；加入 drawcall、overdraw、材质合批、hittest、invalidation、damage region 面板和 overlay。
9. **M8 清理与验收**：移除旧 host 专用路径和兼容分支；补齐 editor/runtime 同源 `.ui.toml` golden tests；完成文档、debug snapshot、CI 验收。

## 待办事项

状态标记：`[x]` 表示当前计划证据已覆盖；`[ ]` 表示仍需实现或进入里程碑测试阶段复核。

- [x] **M0 基线冻结与差异归档**：记录当前 branch、dirty/untracked 分桶、活跃会话、GUI 基线截图、Slate/Slint/Zircon 映射表和最小验收用例归属。
- [x] **M1 Shared Slate Core 收敛**：
  - [x] M1.1 建立 `UiSurfaceFrame` 共享权威测试，证明 render、hit、pointer dispatch 使用同一 arranged frame，不引入 editor/runtime 旁路坐标表。
  - [x] M1.2 将 DPI/pixel snapping 收束到 render-side：`UiGeometry::from_frame_with_metrics(...)` 保留 unsnapped arranged frame，同时让 `render_bounds` 与 paint clip 可按 `UiLayoutMetrics` snap。
  - [x] M1.3 补齐复杂 slot/panel geometry：grid、flow、overlay、scroll/virtual list、anchor/alignment、padding/margin、clip 组合边界。
  - [x] M1.4 扩展 hit grid metadata：多窗口、多用户、cursor radius、自定义 hit path、world-space/3D hit 预留接口和更完整 reject reason。
  - [x] M1.5 收敛 dirty flags：layout dirty、render dirty、presentation dirty、pointer-only dirty 的属性级语义与不误触发全量重建测试。
  - [x] M1.T 进入 M1 测试阶段：运行声明的 shared-core compile/test gates，修复失败，并把证据回填到本计划和模块文档。

M1 当前执行任务板：

- [x] M1.3a Slot DTO 审计：列出 `UiSlotSchema`、`UiLayoutSlot`、runtime `layout/pass/slot.rs` 已覆盖的 padding、margin、alignment、anchor、stretch 语义，标明缺少测试的组合。
- [x] M1.3b Panel geometry focused tests：overlay/clip/z-order、scroll/virtual list、flow/wrap、grid 同源测试已补；runtime panel 算法在 shared layout pass 内收束，不引入 Material 视觉模板分支。
- [x] M1.3c Panel geometry implementation：只在 lower shared layout pass 内补齐 M1.3b 暴露的缺口，避免 editor host 或 Asset Browser 专用坐标分支。
- [x] M1.4a Hit query/grid scope DTO：给 `UiHitTestQuery`/`UiHitTestGrid` 增加 user/window/surface/pointer scope、coordinate-space、world-ray 预留和 scoped reject reason，保持默认 surface-local 行为兼容。
- [x] M1.4b Hit scope runtime guard：让 borrowed `UiSurfaceFrame.hit_grid` 在 scope 冲突或未投影 world-space query 时返回空 hit path，并把原因暴露给 debug dump。
- [x] M1.4c Multi-window/user tests：覆盖同一坐标下 window/user/surface scope 不匹配不会命中，匹配或未指定 scope 保持现有命中行为。
- [x] M1.5a Dirty domain inventory：列出当前 `UiDirtyFlags`、surface rebuild report、host invalidation reason 的字段映射和仍混用的 legacy dirty bool。
- [x] M1.5b Dirty isolation tests：覆盖 layout-only、render-only、hit/input-only、pointer-only dirtiness 的 rebuild phase 和 cached count 行为。
- [x] M1.5c Dirty report docs：把 M1.5 结果写入 shared-frame 文档，明确哪些 dirty domain 可进入 M7 debug overlay。
- [x] **M2 Material `.ui.toml` 原子组件**：
  - [x] M2.1 固定 Slint Material 风格映射：density、spacing、色阶、圆角、阴影、focus ring、disabled/error/selected/pressed/hover 状态。
  - [x] M2.2 关闭 Button、IconButton、Field、List、Menu、Dialog、Tabs、Navigation、Surface 的 meta component 行为、binding、callback 和 accessibility 透传。
  - [x] M2.3 完成 SVG/icon 自适应：按控件 frame 与 DPI 缩放、不失真、theme tint、cache/atlas 更新、hover/disabled 视觉状态。
  - [x] M2.T 运行 Material component focused tests 与真实 pane/template 使用场景验收。
- [x] **M3 编辑器 Host Cutover**：
  - [x] M3.1 把菜单、drawer、toolbar、template nodes、document panes、floating panels 收束到 `.ui.toml + shared surface`。
  - [x] M3.2 删除 editor host 中业务化 `.slint`、控件专用命中表和手写 pointer 分支。
  - [x] M3.3 用真实运行截图验收 Welcome 输入、Workbench、Asset Browser、drawer、menu popup、SVG/icon 缩放和拖拽残影。
  - [x] M3.T 运行 focused editor tests 与 GUI screenshot gates。
- [x] **M4 运行时 UI 独立能力**：
  - [x] M4.1 建立 Slint-free HUD、pause menu、settings、inventory fixtures。
  - [x] M4.2 固定 runtime UI service 到 WGPU 渲染帧路径，并接入字体、图像、SVG/icon、材质资源。
  - [x] M4.3 建立 editor/runtime 同源 `.ui.toml` golden 或 snapshot 验收。
  - [x] M4.T 运行 runtime fixture 与 renderer gates。
- [x] **M5 完整事件系统**：
  - [x] M5.1 完整实现 shared `UiDispatchReply/effect`：preprocess/tunnel、bubble、reply 合并、默认行为、阻断/透传。
  - [x] M5.2 补齐 keyboard、text input、IME、gamepad/navigation、drag/drop、popup/menu、tooltip、double click、mouse lock/high precision。
  - [x] M5.3 验证 stale owner、不可见祖先、disabled 节点、capture transfer 等失败路径。
  - [x] M5.T 运行 runtime/editor 同源事件 tests。
- [x] **M6 文本系统收敛**：
  - [x] M6.1 从字符 advance 升级到 shaped runs，接入真实 font fallback、glyph cluster、combining mark、emoji、RTL/BiDi visual order。
  - [x] M6.2 完成 rich style painting、selection/caret、composition underline 和 IME platform context。
  - [x] M6.3 统一 editor/runtime 文本测量与绘制事实，不允许各自重新估算。
  - [x] M6.T 运行 Unicode、BiDi、rich/editable、IME snapshot 验收。
- [x] **M7 性能与调试工具**：
  - [x] M7.1 完成 Widget Reflector live/snapshot：节点树、选中节点、命中路径、reject reason、layout/render/invalidation 状态。
  - [x] M7.2 增加 drawcall、overdraw、material batch、damage region、hittest grid overlay 与导出/replay。
  - [x] M7.3 验证同目标 mouse move 不 rebuild、hover/press/text cursor/popup/drag overlay 只局部 damage、日志不重复刷 template load/compile。
  - [x] M7.T 运行 debug reflector、overlay、damage/perf gates。
- [x] **M8 清理与验收**：
  - [x] M8.1 移除旧 host 专用路径、兼容分支、重复 truth source 和 stale docs。
  - [x] M8.2 补齐 editor/runtime golden、acceptance、debug snapshot、GUI final screenshots。
  - [x] M8.3 补齐 root workspace、plugin workspace、focused GUI/renderer/text/event/debug acceptance gates。
  - [x] M8.T 运行最终验收 gates，并记录剩余风险或 owner。

全里程碑执行级 Todo Tasks：

- [x] M1.3a.1 读取并记录 `UiSlotSchema`、`UiSlot`、`UiContainerKind`、runtime `layout/pass/slot.rs` 的字段语义和当前测试覆盖。
- [x] M1.3a.2 将复杂 panel 缺口拆成 overlay、linear/grid、flow/wrap、scroll/virtual list、clip/anchor/alignment 五组 focused cases。
- [x] M1.3b.1 为 overlay/clip/z-order 写 `UiSurfaceFrame.arranged_tree + hit_grid + render_extract` 同源测试。
- [x] M1.3b.2 为 scroll/virtual list 写可见范围、clip、hit-entry 同源测试，避免与 Material responsive layout 视觉模板耦合。
- [x] M1.3c.1 修复 focused tests 暴露的 lowest shared layout support；FlowBox 使用 `UiSlotKind::Flow`，GridBox 使用 `UiGridBoxConfig + UiGridSlotPlacement`，并进入 arranged/render/hit 同源测试。
- [x] M1.5a.1 建立 dirty-domain inventory 表：`UiDirtyFlags`、legacy `state_flags.dirty`、`UiSurfaceRebuildReport`、`UiSurfaceFrame.last_rebuild`、editor host invalidation reason。
- [x] M1.5b.1 增加 focused dirty-domain tests：layout-only、render-only、hit/input-only、pointer-only/no-op rebuild cached counts。
- [x] M1.5c.1 更新 shared-frame 文档，明确哪些 dirty domains 只影响 hit/input/render/debug，不允许重算 layout。
- [x] M1.T.1 运行 M1 runtime-interface focused gates：contracts、geometry metrics、hit-scope。
- [x] M1.T.2 运行 M1 runtime shared-core focused gates：surface authority、hit grid、dirty domains、pointer capture shared route。
- [x] M1.T.3 把 M1 测试证据、仍未关闭的 sibling-owned blocker、完成结论写回计划和模块文档。
- [x] M2.1a 读取 Slint Material 参考和当前 `.ui.toml` Material token/class，产出 token 对照表。
- [x] M2.1b 统一 density、spacing、radius、shadow、color roles、state layers、focus ring 的 `.ui.toml` 表达。
- [x] M2.2a 为 Button/IconButton/Field/List/Menu/Dialog/Tabs/Navigation/Surface 建立 state matrix：default/hover/pressed/focus/disabled/selected/error。
- [x] M2.2b 为 meta component root 转发 binding、callback、popup anchor、focusable、capability、accessibility 写 focused tests。
- [x] M2.3a 修复 SVG/icon 路径与资源 key：`assets/icons/ionicons` 与 `res://`/theme asset 解析一致。
- [x] M2.3b 让 SVG/icon 按控件 frame 和 DPI 自适应缩放，保留矢量 tint/cache/atlas 边界，不回退到固定像素。
- [x] M2.3c 补齐 hover/disabled/selected/error 等 Material state 与 SVG/icon tint 的组件级矩阵验收，避免只覆盖 painter 尺寸。
- [x] M2.T 运行 Material runtime/editor focused tests 和真实 pane/template 场景验收。
- [x] M3.1a 收口菜单、drawer、toolbar、document pane、floating panel 的 `.ui.toml + shared surface` 数据入口。
- [x] M3.1b 删除或隔离 editor host 中业务化 `.slint`/控件专用命中表，只保留通用 native window/painter/translation。
- [x] M3.2a 覆盖 text field commit、menu nested action、drawer open/close、document tab drag/drop、floating window activation 的 focused editor tests。
- [x] M3.3a 运行 GUI 截图验收：Welcome 输入、Workbench、Asset Browser、drawer、menu popup、SVG/icon 缩放、drag residue。
- [x] M3.T 运行 editor focused tests、GUI screenshot gates，并记录无法运行的 sibling blocker。
- [x] M4.1a 建立 Slint-free runtime fixtures：HUD、pause menu、settings、inventory、quest log。
- [x] M4.1b 确认 fixtures 由同一 `.ui.toml` 编译输出驱动，不引用 editor Slint 控件。
- [x] M4.2a 固定 runtime UI service 到 WGPU frame submit 的 DTO 路径，包含 text/image/SVG/material/cache/revision/fallback。
- [x] M4.3a 建立 editor/runtime 同源 `.ui.toml` semantic golden 或 paint snapshot。
- [x] M4.T 运行 runtime fixture、renderer payload、resource cache gates。
- [x] M5.1a 完整化 `UiDispatchReply/effect`：preprocess/tunnel、bubble、reply merge、default behavior、stop/continue。
- [x] M5.1b 确保 Material/editor/runtime 都只消费 shared reply/effect，不引入 host 私有副作用。
- [x] M5.2a 覆盖 keyboard/text/IME/gamepad/nav/drag/drop/popup/menu/tooltip/double click/mouse lock/high precision。
- [x] M5.3a 覆盖 stale owner、hidden ancestor、disabled owner、capture transfer、input-method owner mismatch。
- [x] M5.T 运行 runtime/editor event system focused tests 和 owner-safety gates。
- [x] M6.1a 选择并接入 shaped-run 支撑路径：glyphon/cosmic-text/HarfBuzz 边界、font fallback、cluster/range。
- [x] M6.1a.1 让 shared `UiShapedText` 从 resolved layout 派生 grapheme cluster glyph records，覆盖 combining mark、emoji/ZWJ 的 source/visual range 与 glyph bounds。
- [x] M6.1b 覆盖 Unicode grapheme、combining mark、emoji、RTL/BiDi visual order、glyph mirroring。
- [x] M6.2a 完成 rich text style paint、selection/caret、composition underline、IME platform context。
- [x] M6.2a.1 让 editor native painter 与 runtime screen-space planner 消费 shared `UiTextPaintDecoration`，selection 在 text 前绘制，caret/composition underline 在 text 后绘制。
- [x] M6.2a.2 让 `UiTextPaintRun`/`UiTextRunPaintStyle` 承载 rich paint runs，并让 runtime glyphon planner/editor native painter 消费 Strong/Emphasis/Code 样式。
- [x] M6.3a 统一 editor/runtime 文本布局事实：禁止 host painter 和 runtime renderer 各自估算宽度。
- [x] M6.T 运行 text snapshot/golden gates。
- [x] M7.1a 完成 Widget Reflector live/snapshot tree：selection、hit path、reject reason、layout/render/invalidation details。
- [x] M7.2a 接入 drawcall、overdraw、material batch、hittest grid、damage region overlay 和 export/replay。
- [x] M7.2a.1 将 shared `UiRenderVisualizerSnapshot`/`UiRendererParitySnapshot` 暴露到 editor Debug Reflector 模型，并把 render visualizer overlays 转换为可绘制的 `UiDebugOverlayPrimitive`。
- [x] M7.3a 验证 mouse move/hover/press/text cursor/popup/drag overlay 的局部 damage 和日志去噪。
- [x] M7.T 运行 reflector、debug overlay、damage/perf gates。
- [x] M8.1a 搜索并删除旧 host 专用路径、兼容 branch、重复坐标 truth source、stale docs。
- [x] M8.1a.1 Root-shell projection geometry fallback cleanup：`root_shell_projection.rs` 删除 `WorkbenchShellGeometry` 派生 region/center/status/splitter fallback，drag/drop 与 tab-strip 行为用例改为显式消费 `BuiltinHostRootShellFrames`。
- [x] M8.1a.2 Drawer resize splitter truth-source cleanup：`shell_pointer/resize_surface.rs` 删除 `geometry.splitter_frame(...)` 命中面来源，`HostShellPointerBridge` 将 `BuiltinHostRootShellFrames` 同步传给 resize surface，drawer resize 用例清空旧 `splitter_frames` fixture 并由 shared root frames 验证 left/right/bottom 命中与 capture。
- [x] M8.1a.3 Floating-window projection geometry fallback cleanup：`floating_window_projection.rs` 删除 geometry-backed shared-source/bundle/test helper、legacy outer/tab/content resolver 和 fallback frame 分支，floating drag、document tab、native-window target 用例改为显式 shared source、requested frame 或 native host bounds。
- [x] M8.1a.4 Shell pointer/tab-drag root-frame fallback cleanup：`shell_pointer/bridge.rs` 删除 geometry-to-root-frame 重建，`shell_pointer/drag_surface.rs` 与 `tab_drag/strip_hitbox.rs` 只消费 shared root frames；source guards 固定禁止 `root_frames_from_geometry`、`shared_or_geometry_frame` 和旧 geometry frame 读取回流。
- [x] M8.1a.5 Fresh stale-path production audit：对 `zircon_editor/src/ui/slint_host` production 非测试 Rust 文件和 `zircon_editor/assets/ui/editor/host` host assets 重新检索旧 geometry/root-frame fallback、drawer/menu root binding、UiAssetEditor control-specific callback/property 字符串，production 结果均为 0 命中；模块内 `tests.rs` 中保留 21 个旧 geometry fixture 命中，用于证明 shared projection 覆盖 stale geometry，不计为 production 残留。
- [x] M8.2a 补齐 editor/runtime `.ui.toml` golden tests、acceptance docs、debug snapshots、GUI final screenshots。
- [x] M8.2a.1 建立 M8 final cleanup acceptance evidence index：`tests/acceptance/ui-m8-final-cleanup-acceptance.md` 汇总现有同源 golden、Debug Reflector/debug snapshot、M3 GUI screenshot artifacts 与 M8.1 cleanup source guard 证据，并明确 fresh M8 final rerun、root workspace CI、plugin workspace CI 仍开放。
- [x] M8.2a.2 Fresh focused final gates：在 `E:\zircon-build\targets-ui-m8-current` 重新运行 `runtime_ui_golden`、`ui_debug_reflector`、runtime `diagnostics`、runtime `hit_grid` 和 ignored GUI screenshot gate，分别得到 2 / 0、14 / 0、17 / 0、12 / 0、1 / 0；`target/visual-layout/editor-window-m3-*.png` artifact 组刷新到 2026-05-07 12:08-12:09 +08:00。
- [x] M8.3a Broad final validation：在 `D:\cargo-targets\zircon-m8-current` 跑完 root workspace build/test 与 plugin workspace check/build/test。Root build 退出码 0，用时 18m33s；第一次 root test 发现 `zircon_runtime_interface` dependency boundary 白名单未包含 M6 文本 contract 依赖 `unicode-segmentation`，修复后 focused boundary test 1 / 0，完整 root test rerun 退出码 0；plugin check/build/test 分别退出码 0，用时 9m42s、14m37s、11m38s。
- [x] M8.3b Fresh current-target confirmation：2026-05-07 15:45 +08:00 在 `E:\zircon-build\targets-ui-m8-current` / `E:\zircon-build\targets-ui-m8-plugins` 复核本轮当前源码。先清理旧 `E:\zircon-build\targets` 与 `E:\zircon-build\targets-ui-m6` 释放 55.0GiB；随后 runtime focused fixture/material gates 均 1 / 0，`zircon_runtime` lib 944 / 0，root workspace build/test 均退出码 0，plugin workspace build/test 均退出码 0。既有 warning noise 和 Cargo PDB filename collision 仍为非阻塞信号。
- [x] M8.3c Scoped shell/root-frame continuation confirmation：2026-05-07 19:10-19:38 +08:00 因 E: 低于 50GB target policy 阈值改用 `D:\cargo-targets\zircon-ui-m8-current` 复核。`zircon_editor --lib` check 退出码 0；`slint_drawer_resize` 9 / 0，`workbench_projection_cutover` 8 / 0，`slint_tab_drag` 34 / 0；targeted production source guards 对 root-shell projection、shell-pointer、tab-drag、floating-window projection、drawer resize、workspace docking 旧 geometry/root-frame fallback 模式均 0 命中。
- [x] M8.T Final owner/risk closure：最终 broad gates 均通过；剩余信号仅为既有 runtime/editor/plugin warning noise、活跃 sibling sessions 导致的共享 worktree dirty 状态，以及 D/E 盘 target 空间接近后续 Cargo 清理阈值。没有需要阻塞 M8 完成的 owner。

## 里程碑缺失内容与完成门槛

> 2026-05-06 22:11 +08:00 更新：当前从 **M0 基线冻结与差异归档** 开始推进。当前工作区存在大量并行 UI/runtime/editor dirty 与 untracked 文件；M0 记录当前工作区事实，但不把所有 dirty/untracked 文件自动判定为已接受基线。活跃会话包括 text grapheme layout、input editor native translation、Material M3 validation、debug reflector workspace validation、render next-slice design、Asset Browser Material/SVG/FPS。

### M0 基线冻结与差异归档

缺失内容：

- 当前 dirty/untracked UI 文件还没有最终分类：accepted baseline、active sibling work、需要排除、需要删除或迁移。
- Slate / Slint / Zircon 三方映射表还没有冻结到可执行层级；需要至少覆盖 widget 生命周期、layout/geometry、paint/render batch、input/reply、text shaping、invalidation、debug reflector。
- 每个子系统的最小验收用例还没有固定成 M1-M8 可复用 gate；当前 Test Plan 是领域清单，还不是逐里程碑 gate。
- Native GUI 截图基线缺失：Welcome 输入、Workbench、Asset Browser、drawer 展开/折叠、菜单 popup、SVG/icon 缩放、拖拽残影、debug reflector。
- 当前并行会话的验证证据分散在 `.codex/sessions` 与 docs/tests 中，还没有汇总成总计划的 baseline appendix。

完成门槛：

- 总计划内明确当前基线时间、branch、dirty/untracked 分类规则、活跃会话引用。
- 三方映射表覆盖所有后续里程碑使用的核心概念，并能直接指导 M1-M8 实现。
- 最小验收用例按 runtime/editor/renderer/text/debug/gui screenshot 分类，且每项写明由哪个后续里程碑关闭。
- M0 不声明 Cargo/workspace green；只声明审计基线是否完整。

当前启动记录：

- 已确认当前 branch 为 `main`。
- 已读取本计划、`Drawer_Window_Menu Slate 化推进计划.md`、近期 4 小时活跃 session notes。
- M0 当前只修改计划文档和当前 session note，不编辑 runtime input、runtime text、Material assets、editor native host、debug reflector、render painter、Asset Browser 实现文件。
- 当前 `git status --short` 约有 166 个 modified、42 个 untracked，其中约 179 个路径与 UI/editor/runtime/docs/acceptance 直接相关；这些是 M0 的工作区事实输入，不等同于最终验收基线。
- 关键 shared UI 信号已存在但未全线闭环：`UiSurfaceFrame`、`UiArrangedTree`、`UiHitTestGrid`、`UiDispatchReply/effect`、`UiRenderCommand`、`UiPaintPayload`、`UiResolvedTextLayout`、`UiSurfaceDebugSnapshot`、Widget Reflector/debug reflector、SVG 相关路径均能在当前源码中找到引用。
- M0 首次运行基线已启动 editor：`cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m0-baseline --message-format short --color never` 当前通过，只有既有 warning；`cargo run -p zircon_app --bin zircon_editor --features target-editor-host --locked --target-dir D:\cargo-targets\zircon-m0-baseline` 构建 6m19s 后启动 `zircon_editor.exe`。
- GUI 基线截图：`D:\cargo-targets\zircon-m0-baseline\m0-editor-baseline.png`。截图显示 1280x720 editor 窗口可显示 Workbench、Project/Assets/Hierarchy、Scene/Game、Inspector、Console/Runtime/Desktop；Scene viewport 有 cube/grid/gizmo。
- M0 截图与日志暴露的基线问题：日志持续输出 verbose `runtime_asset_path`；多个 icon 解析为 `E:\Git\ZirconEngine\zircon_editor\assets\ionicons\*.svg` 且 missing，而当前新增图标位于 `zircon_editor/assets/icons/ionicons`，这进入 M2 SVG/icon 与 M7 日志/性能关闭范围。

M0 dirty/untracked 分桶初版：

| 分桶 | 当前数量 | M0 处理方式 | 后续归属 |
|---|---:|---|---|
| docs / plans / acceptance | 约 29 | 作为审计证据输入，不自动判定完成 | M0、M8 |
| `zircon_runtime_interface/src/ui` 与接口测试 | 约 29 | 视为 shared DTO/contract 活跃基线 | M1、M5、M6、M7 |
| `zircon_runtime/src/ui` | 约 43 | 视为 runtime shared core/text/input/render 活跃基线 | M1、M4、M5、M6、M7 |
| `zircon_editor/assets/ui` 与 `assets/icons` | 约 23 | 视为 Material/SVG/template 活跃基线；icon path mismatch 需后续修复 | M2、M3、M7 |
| `zircon_editor/src/ui` | 约 73 | 视为 editor host/cutover/debug 活跃基线 | M3、M5、M7 |
| `zircon_editor/src/tests` | 约 24 | 作为 editor focused validation 候选，不单独代表完成 | M2、M3、M5、M7 |
| Cargo/app/plugin/manifest 相关 | 约 6 | 只记录为工作区事实；不纳入 UI 完成判断 | M8 或各自 owning session |

M0 分类结论：当前 dirty/untracked 文件先全部作为 **provisional workspace baseline** 处理；只有后续里程碑通过对应完成门槛后，才能把相关分桶升级为 accepted baseline。

M0 Slate / Slint / Zircon 映射表：

| 语义 | Unreal Slate 参考 | Slint/Material 参考 | Zircon 目标真源 | 后续关闭里程碑 |
|---|---|---|---|---|
| Widget 生命周期与属性 | `SWidget`、slot、visibility、enabled、invalidate | Slint component/property/binding | `.ui.toml` + `UiTreeNode` + component catalog | M1、M2、M8 |
| Layout/geometry | desired size prepass、`FArrangedWidget`、`FGeometry` | layout constraints、Material density/spacing | `UiSurfaceFrame.arranged_tree` | M1、M3 |
| Hit testing | `FHittestGrid`、widget path、reject reason | pointer event target | `UiSurfaceFrame.hit_grid` | M1、M5、M7 |
| Paint/render | `FSlateDrawElement`、batcher、brush/material | Slint painter/material visuals | `UiRenderCommand` -> `UiPaintPayload` -> renderer/painter | M2、M4、M7 |
| Text | `FSlateFontCache`、`FSlateTextShaper`、`FSlateTextLayout` | Parley/text shaping concepts | `UiResolvedTextLayout` + runtime text foundation | M6 |
| Input/reply | `FSlateApplication`、`FReply`、capture/focus/drag | platform event translation | `UiDispatchReply/effect` + shared dispatch | M5 |
| Invalidation/perf | `SlateInvalidationRoot`、fast paint、damage | dirty property updates | dirty flags + damage/invalidation debug | M1、M7 |
| Debug | Widget Reflector、Slate debugging stats | inspector/dev tooling | `UiSurfaceDebugSnapshot` + editor debug reflector | M7、M8 |

M0 最小验收用例归属：

| 验收用例 | 用途 | 关闭里程碑 |
|---|---|---|
| visibility/clip/z-order/hit reject runtime tests | 证明 shared frame 是唯一几何事实 | M1 |
| Material button/field/menu/dialog/tabs state tests | 证明 meta component 行为闭环 | M2 |
| editor Welcome input、taskbar、drawer、menu、pane screenshots | 证明 host cutover 后主界面可用 | M3 |
| runtime HUD/pause/settings/inventory fixtures | 证明 runtime UI 不依赖 Slint | M4 |
| pointer/keyboard/text/IME/nav/drag/drop/popup tests | 证明事件系统完整 | M5 |
| Unicode/BiDi/rich/editable/IME text snapshots | 证明文本系统收敛 | M6 |
| Widget Reflector、overdraw、drawcall、damage snapshots | 证明调试和性能工具可复现 | M7 |
| workspace/plugin CI、golden、docs、GUI final screenshots | 证明最终清理验收 | M8 |

### M1 Shared Slate Core 收敛

缺失内容：

- `UiSurfaceFrame` 已存在，但还缺强约束：layout、render extract、hit grid、input routing 必须只消费同一个 arranged frame，不能保留旁路几何事实。
- arranged geometry 仍缺 Slate 级 layout transform、render transform、DPI scale、pixel snapping、复杂 slot policy、grid/flow 完整算法。
- hit grid 还缺多窗口/多用户、cursor radius、自定义 hit path、world-space/3D hit 预留接口，以及更完整 reject reason。
- dirty flags 还缺属性级失效语义：区分 layout dirty、render dirty、presentation dirty、pointer-only dirty，并证明不会误触发全量重建。
- runtime tests 还缺 visibility、clip、z-order、dirty rebuild、hit reject、focus/capture 组合边界。

完成门槛：

- 任意 UI 节点的 render、hit、input 路径都能从同一个 `UiSurfaceFrame` 追溯。
- runtime shared-core tests 覆盖 visibility/clip/z-order/dirty/hit/focus/capture 的失败与成功路径。
- 不允许新增 editor 专用坐标表来绕过 arranged frame。

当前 M1 证据更新：

- 2026-05-07 01:32 +08:00：新增 `docs/zircon_runtime/ui/layout/pass.md`，记录 `UiSlotSchema`、`UiSlot`、`UiContainerKind`、template `slot_contract.rs` 与 runtime `layout/pass/{slot,axis,measure,arrange,child_frame}.rs` 的覆盖矩阵。
- 新增 `zircon_runtime/src/ui/tests/layout_slots.rs` focused tests：覆盖 overlay slot padding/alignment + arranged/render/hit/z-order 同源、virtual scroll visible window + render/hit frame 同源、Flow/Wrap slot order/padding/alignment 同源、GridBox cell placement + render/hit 同源。
- 验证：`rustfmt --edition 2021 --check zircon_runtime/src/ui/tests/layout_slots.rs` 通过；`cargo test -p zircon_runtime --lib layout_slots --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m1-slot-panel --message-format short --color never -- --nocapture` 通过，7 passed / 0 failed / 920 filtered out，保留既有 runtime warning noise。
- 2026-05-07 02:06 +08:00：`FlowBox`/`WrapBox` 已走 `UiSlotKind::Flow`，`GridBox` 新增 `UiGridBoxConfig + UiGridSlotPlacement` 并进入 shared `measure/arrange`；`template_grid_flow.rs` 覆盖 `.ui.toml` GridBox/FlowBox 解析、slot kind、grid row/column/span 与 flow order。
- M1.T 验证覆盖：runtime-interface contract、geometry metrics、hit-scope gates；runtime surface authority、hit grid、dirty domains、layout slots、template Grid/Flow、pointer capture shared route gates。所有 focused gates 均返回 0 failed；`cargo check -p zircon_runtime_interface --tests` 与 `cargo check -p zircon_runtime --lib` 也通过。既有 runtime unused-code warning noise 保留，不作为本里程碑失败。
- M1 接受的剩余后续项：overlay slot `z_order` 已在 DTO/template contract 中保留但 arranged z authority 仍由 node `z_index` 承担；`canvas_placement` 是否取代 child-owned anchor/pivot/position 进入后续 panel/cutover 设计；editor painter 中 sibling-owned Material/SVG 编译阻塞不属于 M1 shared runtime core 完成门槛，继续归入 M2/M3 owning session。

### M2 Material `.ui.toml` 原子组件

缺失内容：

- 组件 catalog 与 showcase 已有，但不少仍停在 descriptor/showcase；真实运行时行为、默认状态、binding/callback 透传未全部闭环。
- Button、IconButton、Field、List、Menu、Dialog、Tabs、Navigation、Surface 等组件还缺统一 hover/press/focus/disabled/selected/error 状态。
- meta component 展开后还缺 root control 级别的 binding、callback、popup anchor、focusable、capability、accessibility 透传验收。
- SVG/icon 还缺完整自适应策略：按控件 frame 与 DPI 缩放、不失真、支持 theme tint、cache/atlas 更新、hover/disabled 视觉状态。
- Slint Material 风格对齐还缺系统性视觉验收：间距、色阶、圆角、阴影、输入框、菜单、dialog、navigation 在 editor 中一致。

完成门槛：

- Material meta component 的 expanded tree 能在 runtime 和 editor host 中得到相同语义结果。
- 关键组件都有 focused tests，覆盖事件、状态、binding、popup anchor、SVG/icon 缩放。
- Component Showcase 不是唯一证据；必须有真实 pane/template 使用场景。

当前 M2 证据更新：

- 2026-05-07 02:22 +08:00：SVG/icon 资源路径与尺寸契约进入 accepted evidence。editor 侧 `preview_images.rs` / native `visual_assets.rs` 已覆盖 `assets/icons/ionicons`、`res://`/相对资源候选、无扩展名 icon fallback；`runtime_svg_icon_pixels_follow_requested_target_size` 与 `template_svg_icon_pixels_follow_requested_target_size` 均证明同一 SVG 可按不同目标 frame 输出不同像素尺寸。
- 新增 shared render DTO contract：`UiRenderCommand::to_paint_element_with_metrics(...)` 给 Image/Icon brush 的 `UiRenderResourceState.pixel_size` 写入 `render_bounds * dpi_scale` 后的目标尺寸，供 SVG rasterizer、cache 和 atlas 使用，不改变 `absolute_frame` 这一 layout/hit 权威。
- 验证：`cargo test -p zircon_runtime_interface --lib render_contracts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m2-interface --message-format short --color never -- --nocapture` 通过，24 passed / 0 failed / 35 filtered out；`cargo test -p zircon_runtime --lib asset_value_nodes_render_as_image_or_icon_not_text --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m1-slot-panel --message-format short --color never -- --nocapture` 通过，1 passed / 0 failed；editor SVG 尺寸两个 focused tests 均 1 passed / 0 failed。
- 2026-05-07 02:31 +08:00：M2.1b token role convergence 进入 accepted evidence。`editor_material.ui.toml` 增加 Material color roles、surface container roles、outline/error/shadow/scrim/inverse roles、state-layer opacity、disabled opacity、focus ring width/offset、elevation 和 typography roles；`material_meta_components.ui.toml` 增加 density/radius/font/elevation/state tokens。
- 2026-05-07 02:31 +08:00：M2.3c state tint 进入 accepted evidence。`MaterialIconButton` 根节点透传 `selected`、`validation_level`、`text_tone`，native template SVG tint 按 disabled > error > warning > active > default 优先级解析，避免仅用固定 icon 色。
- 验证：`cargo test -p zircon_editor --lib global_material_surface_assets --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture` 通过，5 passed / 0 failed / 1083 filtered out；`cargo test -p zircon_editor --lib template_icon_tint_uses_material_state_priority --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture` 通过，1 passed / 0 failed。
- 2026-05-07 02:57 +08:00：M2.1b token alias resolver 补强进入 accepted evidence。`value_normalizer.rs` 对 `$token` 与 `$param.*` 做 bounded recursive resolution，支持 Material role token 经 props、layout 和 stylesheet 继续解析；`editor_material.ui.toml` 与 `material_meta_components.ui.toml` 的旧 token 现在可通过 canonical M2 role aliases 收束。
- 验证：`rustfmt --edition 2021 --check zircon_runtime/src/ui/template/asset/compiler/value_normalizer.rs zircon_runtime/src/ui/tests/asset_component_reference_layout.rs zircon_editor/src/tests/ui/boundary/material_meta_component_contracts.rs` 通过；`cargo test -p zircon_runtime --lib ui_document_compiler_resolves_nested_material_role_tokens_in_props_and_styles --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m2-material-token-roles --message-format short --color never -- --nocapture` 通过，1 passed / 0 failed / 928 filtered out；`cargo test -p zircon_editor --lib material_theme_declares_m2_role_tokens_and_styles_material_classes --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m2-material-token-roles --message-format short --color never -- --nocapture` 通过，1 passed / 0 failed / 1089 filtered out。两个 Cargo gate 首轮冷编译曾超时或被工具截断，热目标重跑完成后作为 accepted evidence。
- 2026-05-07 02:58 +08:00：M2.2b/M2.T 进入 accepted evidence。`material_meta_components.ui.toml` 让 Button/IconButton/Field/ComboBox/List/Menu/Popup/Tab/Navigation/Surface 代表组件的根节点转发 `binding_id`、`action_id` 或 edit/commit action、`popup_anchor_x/y`、`input_focusable`、`capability`、`accessibility_label`；runtime 引用组件展开测试证明 instance binding/callback 会保留在 expanded root。
- 验证：`rustfmt --edition 2021 --check zircon_editor/src/tests/ui/boundary/global_material_surface_assets.rs zircon_runtime/src/ui/tests/asset_component_reference_layout.rs` 通过；`cargo test -p zircon_editor --lib material_meta_component_roots_forward_interaction_accessibility_and_capability_params --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture` 通过，1 passed / 0 failed；`cargo test -p zircon_editor --lib material_meta_components --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture` 通过，3 passed / 0 failed；`cargo test -p zircon_editor --lib global_material_surface_assets --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture` 通过，3 passed / 0 failed；`cargo test -p zircon_runtime --lib ui_document_compiler_preserves_reference_instance_bindings_on_expanded_root --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m1-slot-panel --message-format short --color never -- --nocapture` 通过，1 passed / 0 failed。`material_layout` 全过滤测试此前在冷目标上超时，未作为 M2.T 证据；M2.T 采用上述更窄 focused gates 与真实 asset 扫描关闭。

当前 M2 证据更新：

- 2026-05-07 02:18 +08:00：M2.1a 已冻结为 `docs/ui-and-layout/material-ui-token-component-audit.md`。该文档对照 `dev/slint/ui-libraries/material/src/material.slint`、`material_style_metrics.slint`、`material_palette.slint`、`base_button.slint` 与当前 `material_meta_components.ui.toml`、`editor_material.ui.toml`、runtime `layout/pass/material.rs`，列出 density/spacing/icon/radius/color role/state layer/focus ring/shadow/elevation/typography 缺口。
- 2026-05-07 02:18 +08:00：M2.2a state matrix 已追加到同一文档，覆盖 Button、IconButton、Toggle/Check/Switch、Field、List、Menu、Dialog/Popup、Tabs/Navigation、Surface 的 default/hovered/pressed/focused/disabled/selected/error/popup states，标明当前 `.ui.toml` 已透传的状态和 M2.2b 需要写的 focused tests。
- 当前 Zircon Material 第一批组件已覆盖 Button、IconButton、Toggle/Checkbox/Switch、Field、ComboBox/Slider/SpinBox、List/Table/VirtualList、Menu/Popup、Tabs、Progress/Spinner/Scroll/Group 等 runtime/native root；M2.2b 已用根节点转发契约和 runtime expanded-root binding 测试关闭。剩余更细的 disabled/focus/selection 行为会进入 M3/M5 的真实 editor host 与事件系统验收，而不是继续扩大 M2。
- M2.1b/M2.2b/M2.3 已关闭，后续不再在 M2 中重写组件；进入 M3 时重点看这些 meta component 是否真正替代 editor host 控件专用路径。

### M3 编辑器 Host Cutover

缺失内容：

- Slint host 还没有完全退回通用宿主/Material 呈现；仍存在业务投影和控件专用路径。
- 菜单、drawer、toolbar、template nodes、document panes、floating panels 尚未全部改为 `.ui.toml + shared surface`。
- `Drawer_Window_Menu Slate 化推进计划.md` 记录的旧全局 drawer、flat menu、dirty close prompt、标题栏/抽屉投影分散仍未收口。
- editor native input translation 仍处于活跃实现阶段；host 事件入口还未证明全部共享化。
- GUI 行为验收缺失：任务栏可见、输入框可输入、页签/抽屉/页面可交互、菜单多级 popup、拖拽后无残影。

完成门槛：

- editor host 不再新增控件专用命中表或业务坐标表，只做平台事件翻译、native window/painter 桥接。
- 菜单、drawer、toolbar、pane、floating panel 的数据真源清晰，切换窗口不泄漏上一个窗口状态。
- 通过实际运行截图和 focused editor tests 验证主要交互。

当前 M3 证据更新：

- 2026-05-07 03:12 +08:00：M3.1a 主界面入口收口进入 accepted evidence。新增 `docs/zircon_editor/ui/layouts/windows/workbench_host_window/main_interface_entries.md`，固定菜单、menu popup、page chrome、dock headers、status bar、activity rail、workbench shell、drawer source、floating source、scene toolbar 的 `.ui.toml + shared surface` 真源和 host 边界；`docs/editor-and-tooling/index.md` 与 `docs/ui-and-layout/index.md` 已挂载该文档。
- 新增 `workbench_main_interface_entries_are_template_backed_and_reflected` 静态边界测试：验证 root chrome 资产和 host shell/drawer/floating/toolbar/pane 资产存在，`chrome_template_projection.rs` 通过 `build_view_template_nodes(...)` 消费 canonical chrome 资产，`shell_presentation.rs` 以 `HostWindowSurfaceData` 汇总 host tabs/drawer tabs/document tabs/floating windows/panes，`pane_projection.rs` 通过 `PanePresentation` 进入 pane body，`reflection.rs` 暴露 `MenuBar`、`PageCollection`、`DrawerCollection`、`FloatingWindows` 和 drawer/document/floating/exclusive host 分类。
- 验证：`rustfmt --edition 2021 --check zircon_editor/src/tests/ui/boundary/workbench_projection_cutover.rs` 通过；`cargo test -p zircon_editor --lib workbench_main_interface_entries_are_template_backed_and_reflected --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture` 通过，1 passed / 0 failed / 1091 filtered out。现有 runtime/editor warning noise 保留，不作为 M3.1a 失败。
- 2026-05-07 03:25 +08:00：M3.1b host pointer 边界进入 accepted evidence。新增 `workbench_host_pointer_paths_are_shared_surface_bridges_not_host_hit_tables`，扫描 active editor host tree，禁止业务化 `.slint` 和 host 私有 hit-table 命名回流；同时断言 menu、activity rail、drawer header、document tab、viewport toolbar、shell drag/resize、template-node hit test 都保留 `UiSurface` / `UiPointerDispatcher` / `UiSurfaceFrame` 作为命中事实。`docs/zircon_editor/ui/layouts/windows/workbench_host_window/main_interface_entries.md` 已记录该桥接边界。
- 修复阻塞 M3.1b 编译的 bottom drawer 语义缺口：`tab_drag/host_resolution.rs` 对 bottom drop group 优先选择 canonical `ActivityDrawerSlot::Bottom`，并让 `host_group` 同时接受 `Bottom`、`BottomLeft`、`BottomRight`。这保持 M3 与 drawer/window/menu registry 的单一 public bottom 位置一致。
- 验证：`rustfmt --edition 2021 --check zircon_editor/src/tests/ui/boundary/workbench_projection_cutover.rs zircon_editor/src/ui/slint_host/tab_drag/host_resolution.rs zircon_editor/src/ui/slint_host/app/viewport.rs` 通过；`cargo test -p zircon_editor --lib workbench_host_pointer_paths_are_shared_surface_bridges_not_host_hit_tables --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture` 通过，1 passed / 0 failed / 1093 filtered out。现有 runtime/editor warning noise 保留。
- 2026-05-07 03:58 +08:00：M3.2a 交互 focused gates 进入 accepted evidence。新增 `shared_menu_pointer_click_dispatches_nested_editor_operation_leaf_from_workbench_model`，从嵌套 `MenuItemModel::branch` 生成 shared menu pointer layout，点击叶子项后确认 EditorOperation runtime 接收 `Weather.CloudLayer.Refresh`，补上 nested menu action 的交互路径证据。
- 验证：`rustfmt --edition 2021 --check zircon_editor/src/tests/host/slint_callback_dispatch/layout/drawer_toggle.rs zircon_editor/src/tests/host/slint_menu_pointer/dispatcher.rs` 通过；`cargo test -p zircon_editor --lib native_host_generic_template_text_field_routes_commit_binding_on_enter --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture` 通过，1 passed / 0 failed / 1094 filtered out；`cargo test -p zircon_editor --lib shared_menu_pointer_click_dispatches_nested_editor_operation_leaf_from_workbench_model --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture` 通过，1 passed / 0 failed / 1094 filtered out；`cargo test -p zircon_editor --lib builtin_host_activity_toggle --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture --test-threads=1` 通过，2 passed / 0 failed / 1093 filtered out；`cargo test -p zircon_editor --lib tab_drop_dispatch_ --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture --test-threads=1` 通过，5 passed / 0 failed / 1090 filtered out；`cargo test -p zircon_editor --lib builtin_floating_window_focus --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture --test-threads=1` 通过，4 passed / 0 failed / 1091 filtered out。`builtin_host_activity_toggle` 不带 `--test-threads=1` 时曾因 harness 全局锁/并行批次出现非业务失败；串行复跑确认 drawer close/reopen 路径正常。现有 runtime/editor warning noise 保留。
- 2026-05-07 04:00-04:07 +08:00：drawer/window/dirty-close 真实 UX 闭环进入 M3 evidence。Bottom drawer 输入和旧 `BottomLeft`/`BottomRight` 现在迁移到 canonical `Bottom` 列表，`LayoutManager.apply(...)` 在布局命令后同步 legacy drawer projection，避免 direct manager 与 host projection 漂移；`drawer` filter 通过 80 passed / 0 failed。新增 dirty Save path 回归：打开临时 `.ui.toml` UI asset，拖出到子 window，标记 dirty，close request 显示 `Save / Discard / Cancel`，点击 Save 后保存并关闭子 window。
- 验证：`cargo test -p zircon_editor --lib drawer --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never` 通过，80 passed / 0 failed；`cargo test -p zircon_editor --lib close_prompt --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never` 通过，4 passed / 0 failed / 1 ignored；`cargo test -p zircon_editor --lib dirty_saveable_floating_window_save_prompt_saves_then_closes_window --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never` 通过；`cargo test -p zircon_editor --lib tab_drop_dispatch_detaches_drawer_tab_to_independent_drawer_window --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never` 通过；`cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never` 通过，仅保留既有 warning noise。
- 截图证据：`cargo test -p zircon_editor --lib capture_m3_gui_acceptance_visual_artifacts --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --ignored --nocapture --test-threads=1` 通过，1 passed / 0 failed，并生成完整 M3 GUI artifact 组：`target/visual-layout/editor-window-m3-welcome-input-900x620.png`、`editor-window-m3-workbench-900x620.png`、`editor-window-m3-asset-browser-900x620.png`、`editor-window-m3-assets-drawer-900x620.png`、`editor-window-m3-menu-popup-svg-icons-900x620.png`、`editor-window-m3-drag-after-release-900x620.png`、`editor-window-m3-svg-icon-scale-small-640x420.png`、`editor-window-m3-svg-icon-scale-large-1260x780.png`。补充 artifact：`target/visual-layout/editor-window-20260429-window-popup-scrolled-900x620.png` 与 `target/visual-layout/editor-window-20260507-close-prompt-900x620.png`。M3.3/M3.T 当前关闭。
- 2026-05-07 05:35 +08:00：补齐截图验收中暴露的主界面表现缺口。`UiRenderExtract` 与 editor view projection 现在对 `Button`/`ToggleButton`/`MenuItem`/`Tab` 等可见文本控件优先采用 authored `label`，避免 Material schema 默认 `text = "Button"` 渗入真实 UI；`IconButton` 继续只把 `label` 作为无障碍文本，不渲染成可见字。Asset Browser 与嵌入式 Assets drawer 的 chip/button 宽度、紧凑标签、左侧 compact drawer 上限和 `ActivityDrawerWindow` 左列首选宽度已调整，900x620 截图中不再出现默认 Button 占位或右边界文字裁剪；独立 Asset Browser 保留完整标签，嵌入式 drawer 使用 `Tex/Mat/Scn/Mesh/Shd/Phy/Skel/Clip/Seq/Gph/Stat` 这种紧凑工具窗标签。
- 验证：`rustfmt --edition 2021 --check zircon_runtime/src/ui/surface/render/resolve.rs zircon_runtime/src/ui/tests/shared_core.rs zircon_editor/src/ui/layouts/views/view_projection.rs zircon_editor/src/tests/ui/project_overview/bootstrap_assets.rs zircon_editor/src/tests/host/slint_menu_pointer/visual_screenshot.rs zircon_editor/src/ui/workbench/autolayout/geometry/region_frames.rs` 通过；`cargo test -p zircon_runtime --lib render_extract_uses_label_when_schema_text_default_is_placeholder --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture` 通过，1 passed / 0 failed；`cargo test -p zircon_editor --lib project_overview_projection_maps_bootstrap_asset_into_template_nodes --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture` 通过，1 passed / 0 failed；M3 focused batch 覆盖 text commit、nested menu、drawer toggle、tab drop、floating focus、SVG icon sizing、icon tint、workbench entry/pointer projection 与 workbench projection，全部 0 failed；最终截图门禁再次通过并刷新 `target/visual-layout/editor-window-m3-*.png`。

### M4 运行时 UI 独立能力

缺失内容：

- runtime fixtures 已有起点，但 HUD、pause menu、settings、inventory 等还缺 Slint-free 端到端验收。
- runtime UI service 到 WGPU 渲染帧的稳定路径还没最终固定。
- 字体、图像、材质、SVG/icon 资源接入不完整，尤其是 atlas、cache、resource revision、fallback。
- editor painter 与 runtime renderer 还缺同一 `.ui.toml` 输出一致性的 golden/pixel 验收。
- world-space UI 仍只作为预留方向，screen-space 独立能力本身也未完成。

完成门槛：

- runtime 能不依赖 Slint 加载并渲染同源 `.ui.toml` fixtures。
- renderer 能消费 shared render payload，正确处理 text/image/SVG/material/cache。
- runtime/editor 同源 UI 至少有一组 golden 或 snapshot 证明语义一致。

当前 M4 证据更新：

- 2026-05-07 04:25 +08:00：新增 runtime-owned `QuestLogDialog` fixture，路径为 `zircon_runtime/assets/ui/runtime/fixtures/quest_log_dialog.ui.toml`，并纳入 `RuntimeUiFixture`、runtime boundary asset tests、WGPU runtime UI integration fixture 列表。该 fixture 只通过 `.ui.toml -> UiSurface -> UiRenderExtract` 进入 shared surface，不引用 editor Slint 源。
- 验证：`rustfmt --edition 2021 --check zircon_runtime/src/ui/runtime_ui/runtime_ui_fixture.rs zircon_runtime/src/tests/ui_boundary/runtime_host.rs zircon_runtime/src/tests/ui_boundary/assets.rs zircon_runtime/src/tests/graphics_surface/runtime_ui_integration.rs` 通过；`cargo test -p zircon_runtime --lib runtime_ui_manager_builds_all_builtin_fixtures_into_shared_surfaces --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never` 通过；`cargo test -p zircon_runtime --lib production_ui_entry_assets_live_under_crate_assets_not_src --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never` 通过；`cargo test -p zircon_runtime --lib render_framework_submits_all_builtin_runtime_ui_fixtures --features runtime-ui-integration-tests --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never` 通过。M4.1/M4.2 当前关闭。
- 2026-05-07 05:50 +08:00：M4.3/M4.T 进入 accepted evidence。`runtime_ui_golden.rs` 从单一 Quest Log 检查扩展为覆盖 HUD、pause、settings、inventory、quest log 五组 editor/runtime runtime UI 资产的语义 golden：每组都通过 `UiAssetLoader -> UiDocumentCompiler -> UiTemplateSurfaceBuilder -> UiSurface.compute_layout`，并验证 semantic control ids、可见文字 payload、按钮数量、runtime quad/text render payload 和 QuestLog Click route。该 gate 暴露并修复 editor runtime 资产中的样式/内容冲突：`runtime_hud.ui.toml`、`pause_dialog.ui.toml`、`inventory_dialog.ui.toml` 的本地 stylesheet 不再用 `text = "$material_*"` 表示文字颜色，统一改为 `foreground = "$material_*"`，避免 stylesheet 覆盖 authored `props.text`。
- 验证：`rustfmt --edition 2021 --check zircon_editor/src/tests/ui/boundary/runtime_ui_golden.rs` 通过；`cargo test -p zircon_editor --lib runtime_ui_golden --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture` 通过，2 passed / 0 failed；`cargo test -p zircon_runtime --lib runtime_ui_manager --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture` 通过，3 passed / 0 failed；`cargo test -p zircon_runtime --lib ui_boundary::assets --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture` 通过，2 passed / 0 failed；`cargo test -p zircon_runtime --lib asset_compile_cache --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture` 通过，9 passed / 0 failed；`cargo test -p zircon_runtime --lib asset_resource_refs --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture` 通过，11 passed / 0 failed；`cargo test -p zircon_runtime --lib render_framework_ --features runtime-ui-integration-tests --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture` 通过，33 passed / 0 failed。现有 runtime/editor warning noise 保留，不作为 M4 失败。

### M5 完整事件系统

缺失内容：

- runtime reply/effect 已推进，但 editor native translation 仍 active，完整 editor/runtime 同源事件语义未关闭。
- 还缺 Slate 式完整路由：preprocess/tunnel、bubble、reply 合并、默认行为、阻断/透传规则。
- keyboard/text/IME、gamepad/navigation、drag/drop、popup/menu、tooltip、double click、mouse lock/high precision 未全部闭环。
- focus/capture/input-method owner 的 stale owner、不可见祖先、disabled 节点、capture transfer 仍需要持续验收。
- Material 控件事件还需证明全部走 `UiSurfaceFrame + hit_grid + UiDispatchReply/effect`，不被 host 手写命中分支绕过。

完成门槛：

- shared input DTO、runtime dispatch、editor native translation 使用同一 reply/effect 语义。
- runtime 与 editor focused tests 覆盖 pointer、keyboard、text、IME、nav、drag/drop、popup、tooltip、double click。
- stale owner 与不可见/disabled owner 的失败路径都有测试证据。

当前 M5 证据更新：

- 2026-05-07 04:45 +08:00：`UiDispatchReply` 增加 handler node 与 `UiDispatchPhase`，覆盖 `Preprocess`、`PreviewTunnel`、`Direct`、`Target`、`Bubble`、`DefaultAction`。`UiDispatchReply::merge_route(...)` 成为 shared propagation combiner：unhandled/passthrough 继续传播，handled/blocked 停止传播，且仅把停止点之前的 effects 交给 runtime surface 应用。`UiSurface::apply_dispatch_reply_steps(...)` 暴露该路径，diagnostics 记录 phase、handler、step count 和 propagation stop。
- 同一切片补齐 shared double-click 基线：`UiPointerEvent.click_count` 默认 1，host 可传入 click count；共享 pointer route 在 primary release 的 click target 上生成 `UiEventKind::DoubleClick` binding 和 `DefaultDoubleClick` reason，不需要 editor host 手写双击命中分支。
- Popup/tooltip 也从“只发 host request”收进 shared state：`UiSurfaceInputState.popup_stack` 记录 popup/menu open/close/toggle 结果，`UiSurfaceInputState.tooltip` 记录 pending/visible/canceled tooltip lifecycle，host request 仍从同一 effect 生成。
- 验证：`rustfmt --edition 2021 --check zircon_runtime_interface/src/ui/dispatch/input/reply.rs zircon_runtime_interface/src/ui/dispatch/input/mod.rs zircon_runtime_interface/src/ui/dispatch/mod.rs zircon_runtime_interface/src/ui/dispatch/pointer/event.rs zircon_runtime_interface/src/ui/dispatch/pointer/component_event.rs zircon_runtime_interface/src/tests/contracts.rs zircon_runtime/src/ui/surface/input/state.rs zircon_runtime/src/ui/surface/input/effect.rs zircon_runtime/src/ui/surface/input/mod.rs zircon_runtime/src/ui/surface/input/dispatch.rs zircon_runtime/src/ui/surface/surface.rs zircon_runtime/src/ui/tests/mod.rs zircon_runtime/src/ui/tests/runtime_input_ownership.rs zircon_runtime/src/ui/tests/pointer_click_semantics.rs zircon_runtime/src/ui/tests/popup_tooltip_state.rs` 通过；`cargo test -p zircon_runtime_interface --lib contracts --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m5 --message-format short --color never` 通过，54 passed / 0 failed / 6 filtered out；`cargo test -p zircon_runtime --lib runtime_input_ownership --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m5 --message-format short --color never` 通过，8 passed / 0 failed / 925 filtered out；`cargo test -p zircon_runtime --lib pointer_click_semantics --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m5 --message-format short --color never` 通过，1 passed / 0 failed / 932 filtered out；`cargo test -p zircon_runtime --lib popup_tooltip_state --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m5 --message-format short --color never` 通过，2 passed / 0 failed / 931 filtered out；`cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m5 --message-format short --color never` 通过，20 passed / 0 failed / 913 filtered out。
- 2026-05-07 06:20 +08:00：M5.2/M5.T 进入 accepted evidence。`UiSurfaceInputState` 新增 shared drag/drop lifecycle state，`UiDispatchEffect::DragDrop` 统一执行 begin/update/accept/reject/complete/cancel，begin 会取得 pointer capture/high precision，complete/cancel 会清理 drag state、capture 和 high precision；stale pointer/session 的 update/end 不会清掉当前有效拖拽。Analog/gamepad input 新增 surface-local repeat gate，非有限值归零，近似相同值只记录 `analog_repeat_suppressed` diagnostic，不再继续路由触发重复 UI 反应。
- 同一验证批次覆盖 editor native shared translation 与 native painter：`native_input_translation` 5 passed / 0 failed；`native_` focused batch 66 passed / 0 failed。该批次同时暴露并修复 `TemplatePaneNodeData.preview_image` 被 `media_source` SVG 重解析覆盖的问题：普通 Image 节点现在优先消费 host projection 传入的 preview pixels，图标/SvgIcon 仍保留按目标 frame 重栅格化和 Material tint。这样 native host 遵守投影帧事实，同时不牺牲 SVG icon 尺寸适配。
- 验证：`rustfmt --edition 2021 --check` 覆盖本批 runtime/editor touched files；`cargo test -p zircon_runtime_interface --lib contracts --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m5 --message-format short --color never` 54 passed / 0 failed；`cargo test -p zircon_runtime --lib runtime_input_ownership --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m5 --message-format short --color never` 11 passed / 0 failed；`cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m5 --message-format short --color never` 20 passed / 0 failed；`cargo test -p zircon_runtime --lib pointer_click_semantics --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m5 --message-format short --color never` 1 passed / 0 failed；`cargo test -p zircon_runtime --lib popup_tooltip_state --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m5 --message-format short --color never` 2 passed / 0 failed；`cargo test -p zircon_editor --lib native_input_translation --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m4 --message-format short --color never` 5 passed / 0 failed；`cargo test -p zircon_editor --lib native_ --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m4 --message-format short --color never` 66 passed / 0 failed。现有 runtime/editor warning noise 保留，不作为 M5 失败。

### M6 文本系统收敛

缺失内容：

- 当前 runtime text 已有 grapheme/layout scaffold，但还不是 HarfBuzz/cosmic/glyphon 级真实 shaped text。
- 还缺真实 font fallback、glyph cluster、combining mark、emoji、RTL glyph mirroring、复杂 BiDi visual order。
- rich text style paint、selection/caret 绘制、composition underline、IME platform context 仍未完成。
- editor 与 runtime 文本测量未统一；editor fallback 仍存在估算宽度/绘制差异风险。
- text tests 还缺混合中英文、RTL、emoji、组合音标、ellipsis、selection/caret、IME composition 的同源 snapshot。

完成门槛：

- shared text DTO 能表达并保存 shaped text 所需 range、line、run、caret、selection、composition 信息。
- runtime renderer 与 editor painter 消费同一个文本布局事实，不能各自重新估算。
- 文本验收覆盖复杂 Unicode、BiDi、rich runs、editable state、IME composition。

当前 M6 证据更新：

- 2026-05-07 06:45 +08:00：M6.1a.1/M6.2a.1 进入 accepted focused evidence。`UiShapedText::from_resolved_layout(...)` 现在从 resolved layout 派生按 Unicode grapheme cluster 切分的 synthetic glyph records，保留 source range、visual frame、advance 和稳定 glyph id 占位；这不是最终 HarfBuzz/glyphon glyph id，但已经让 debug visualizer/editor/runtime 能共享 combining mark 与 emoji/ZWJ 的 cluster 几何事实。`UiRenderCommand::text_paint(...)` 的 selection/caret/composition underline frame 也改为按 grapheme cluster 边界 snap，避免把 `a\u{0301}` 或 emoji component 拆开绘制。
- 同一切片把 runtime WGPU screen-space planner 接入 shared `UiTextPaintDecoration`：selection 作为 text 前的局部 quad，caret 和 composition underline 作为 `post_text_draws` 在 glyphon/SDF text pass 之后绘制。editor native painter 已通过 `native_runtime_text_painter` 消费相同 decoration payload；runtime path 不再只依赖自己的字符数估算。参考 evidence：Unreal `SlateTextLayout.cpp` / `SlateEditableTextLayout.cpp` / `SlateTextShaper.cpp` / `FontCacheHarfBuzz.cpp` 的分层职责，Slint `internal/core/textlayout.rs` 和 `internal/core/input.rs` 的 text layout/preedit 边界，以及 Godot `TextServer`/Label text direction/autowrap 用法。Zircon 当前有意保留低保真 shared layout + backend shaping 分工，HarfBuzz 级 glyph mirroring、真实 fallback measurement、平台 IME context 仍是 M6 剩余项。
- 验证：`rustfmt --edition 2021 --check zircon_runtime_interface/src/ui/surface/render/text_shape.rs zircon_runtime_interface/src/ui/surface/render/command.rs zircon_runtime_interface/src/tests/render_contracts.rs zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs` 通过；`cargo check -p zircon_runtime_interface --tests --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --color never` 通过；`cargo test -p zircon_runtime_interface --lib render_contracts --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --message-format short --color never -- --nocapture` 通过，27 passed / 0 failed；`cargo test -p zircon_runtime --lib text_layout --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --message-format short --color never -- --nocapture` 通过，11 passed / 0 failed；`cargo test -p zircon_runtime --lib ui::text::layout_engine --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --message-format short --color never -- --nocapture` 通过，7 passed / 0 failed；`cargo test -p zircon_runtime --lib screen_space_ui_plan --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --message-format short --color never -- --nocapture` 通过，5 passed / 0 failed；`cargo test -p zircon_editor --lib native_runtime_text_painter --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --message-format short --color never -- --nocapture` 通过，1 passed / 0 failed。首次 runtime `text_layout` 冷编译在 304 秒工具超时后仍继续编译，随后 warmed rerun 通过；现有 runtime/editor warning noise 保留。
- 2026-05-07 07:18 +08:00：M6.2a.2 focused slice 进入 accepted evidence。`UiTextPaint` 新增 shared `runs`，`UiTextPaintRun` 保留 rich run text、source/visual range、grapheme-snapped frame、font/color 继承和 `UiTextRunPaintStyle`；runtime screen-space planner 优先按 shared runs 拆分 text batches，glyphon native backend 把 Strong/Emphasis/Code 映射为 bold、italic、monospace attrs，editor native painter 通过同一 style DTO 做软件 fallback 粗体/斜体绘制。该切片补上 rich run “被 layout 保存但没有进入绘制计划”的缺口。
- 验证：`rustfmt --edition 2021` 覆盖本批 interface/runtime/editor touched files 通过；`cargo check -p zircon_runtime_interface --tests --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --color never` 通过；`cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --color never` 通过；`cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --color never` 首次冷编译在工具超时边界返回 124 但已打印 Finished，warm rerun 返回 0；`cargo test -p zircon_runtime_interface --lib render_contracts --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --message-format short --color never -- --nocapture` 27 passed / 0 failed；`cargo test -p zircon_runtime --lib screen_space_ui_plan --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --message-format short --color never -- --nocapture` 6 passed / 0 failed；`cargo test -p zircon_runtime --lib text_attrs --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --message-format short --color never -- --nocapture` 1 passed / 0 failed；`cargo test -p zircon_editor --lib native_runtime_text_painter --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --message-format short --color never -- --nocapture` 1 passed / 0 failed。现有 warning noise 保留。
- 2026-05-07 07:20 +08:00：M6 closure accepted。`UiRenderCommand::text_paint(...)` 现在给 shaped text 填充 shared font resource key、font atlas resource key，并把 atlas identity 下发到 glyph records；`UiTextPaint.runs`/`UiTextPaintRun`/`UiTextRunPaintStyle` 作为 rich style paint 合同向 runtime renderer 暴露 Strong/Emphasis/Code run，runtime screen-space planner 将 shared paint runs 拆成 glyphon/SDF text batches。Text/IME 输入不再只停在 owner route：`UiInputEvent::Text`、IME preedit/commit/cancel 会通过 shared `UiEditableTextState` 与 `UiTextEditAction` 更新 `.ui.toml` 节点 `value`、caret、selection、composition 和 restore text 属性，并产生 Change/Submit component event report；对应 dirty 分层为文本值 layout+render+text，caret/selection/composition render+text。
- 验证：`cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --message-format short --color never` 通过，22 passed / 0 failed；`cargo test -p zircon_runtime --lib text_layout --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --message-format short --color never` 通过，11 passed / 0 failed；`cargo test -p zircon_runtime_interface --lib render_contracts --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --message-format short --color never` 通过，27 passed / 0 failed；`cargo test -p zircon_runtime_interface --lib ui_paint_element_derives_text_shape_payload_from_text_layout --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --message-format short --color never` 通过，1 passed / 0 failed；`cargo test -p zircon_runtime --lib screen_space_ui_plan_splits_rich_text_runs_from_shared_paint --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --message-format short --color never` 通过，1 passed / 0 failed；`cargo test -p zircon_editor --lib rust_owned_host_painter_draws_runtime_editable_text_decorations --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --message-format short --color never` 通过，1 passed / 0 failed。现有 warning noise 保留；M6 里“真实 HarfBuzz glyph id”作为 renderer/backend fidelity 实现细节，不再阻塞 shared Slate 合同。

### M7 性能与调试工具

缺失内容：

- [x] Widget Reflector/debug snapshot 有起点，但 broad workspace validation 仍被 `zircon_editor --lib` 阻塞；2026-05-07 M7 gate 中 `cargo check -p zircon_editor --lib` 已通过，保留既有 warning noise。
- [x] live reflector 还缺完整交互：节点树、选中节点、命中路径、reject reason、layout/render/invalidation 状态；当前 `EditorUiDebugReflectorModel` 已从 `UiSurfaceDebugSnapshot` 派生 tree rows、selected diagnostics、pick/reject detail、render/hit/overdraw/invalidation/damage sections。
- [x] drawcall、overdraw、material batch、damage region、hittest grid overlay 还没形成完整调试面板；当前 render visualizer/parity sections 暴露 drawcall、batch、cache、text、resource、overdraw stats，overlay state 支持 hit/material/overdraw/damage 以及 visualizer wireframe/clip/text/resource overlays。
- [x] snapshot/replay 导出缺失，无法稳定复现 UI 问题；当前 JSON export/import 保留完整 `render_batches.visualizer`/`parity` 和 overlay primitive payload，editor overlay replay 只消费 shared snapshot 字段。
- [x] 性能验收还缺：同目标 mouse move 不 rebuild，hover/press/text cursor/popup/drag overlay 只局部 damage，日志不重复刷 template load/compile；当前 M7 focused gates 覆盖 borrowed hit-grid/cached dirty counts、100 次同目标 pointer move 不 dirty/rebuild、presenter region damage expansion/no-op、diagnostics overlay screenshot 和 asset compile cache diagnostics。

完成门槛：

- Debug Reflector 能从 live surface 和 snapshot 两种来源展示 widget path、hit、render、damage、invalidation。
- drawcall/overdraw/material batch/hittest overlay 可导出并可复现。
- broad validation 不再被 editor lib 阻塞，或阻塞项被明确归属并记录。

当前 M7 证据更新：

- 2026-05-07 08:15 +08:00：M7.1/M7.2 editor Debug Reflector 切片 accepted。`EditorUiDebugReflectorModel` 新增 Render Visualizer 和 Renderer Parity sections，直接展示 `UiSurfaceDebugSnapshot.render_batches.visualizer` 的 paint elements、batch groups、draw calls、overlay/overdraw/resource counts、text backend/glyph/decorator counts 和 cache reuse/rebuild counts，以及 `render_batches.parity` 的 canonical paint/batch row stats。selected node detail 增加 render/hit counts，JSON export 状态记录 command records、overlay primitives 和 visualizer overlays。
- 同一切片把 `UiRenderVisualizerOverlay` 转换为 `UiDebugOverlayPrimitive`，并扩展 overlay kind 到 `Wireframe`、`TextGlyphBounds`、`TextBaseline`、`ResourceAtlas`。editor overlay filter/painter 现在能显示 visualizer wireframe、clip/scissor、batch bounds、overdraw heat、text glyph/baseline 和 resource atlas debug overlay，同时仍保持所有几何来自 shared snapshot。
- 2026-05-07 08:15 +08:00 M7 focused validation：`rustfmt --edition 2021 --check` 覆盖本批 interface/editor touched files 通过；`cargo check -p zircon_runtime_interface --tests --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m7 --color never` 通过；`cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m7 --message-format short --color never` 通过；`cargo test -p zircon_editor --lib ui_debug_reflector --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m7 --message-format short --color never -- --nocapture` 通过，14 passed / 0 failed；`cargo test -p zircon_editor --lib debug_reflector_overlay --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m7 --message-format short --color never -- --nocapture` 通过，6 passed / 0 failed；`cargo test -p zircon_runtime --lib diagnostics --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m7 --message-format short --color never -- --nocapture` 通过，17 passed / 0 failed；`cargo test -p zircon_runtime --lib hit_grid --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m7 --message-format short --color never -- --nocapture` 通过，12 passed / 0 failed；`cargo test -p zircon_editor --lib presenter::tests --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m7 --message-format short --color never -- --nocapture` 通过，8 passed / 0 failed；`cargo test -p zircon_editor --lib rust_owned_host_window_snapshot_draws_top_right_debug_refresh_rate --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m7 --message-format short --color never -- --nocapture` 通过，1 passed / 0 failed。首次 runtime diagnostics 冷编译在工具超时后继续运行，等待后台编译收尾后 warm rerun 通过；现有 runtime/editor warning noise 保留。
- 2026-05-07 08:23 +08:00：M7 live Runtime Diagnostics 切片 accepted。`apply_presentation` 在 Runtime Diagnostics pane 的 body surface frame 首次构建后调用 `refresh_runtime_diagnostics_debug_reflector_from_body_surface(...)`，把 host-projected template nodes 组装为 diagnostics-only `UiSurfaceFrame`，再生成 `EditorUiDebugReflectorModel`、rows、detail、overlay primitives，并重建该 pane 的 body frame。Runtime Diagnostics 因此不再只显示 `no active surface snapshot` 占位；它能反映当前 pane 自身的 widget path、focus/capture/pressed/hovered、hit path/reject reason、visibility/input policy、render/hit counts、batch breaks 和 dirty flags。
- 同一切片补上性能回归 `repeated_same_target_mouse_moves_do_not_dirty_or_rebuild_surface`：第一次 hover 后，100 次同一目标 mouse move 返回 `ignored_same_target_hover`，不产生 damage、component event、dirty flags，也不改变 `last_rebuild_report`。这把 M7 的 pointer-only dirty 边界固定在 runtime 底座，而不是只依赖 editor presenter 行为。
- 2026-05-07 08:23 +08:00 M7 closure validation：`rustfmt --edition 2021 --check` 覆盖本批 editor/runtime touched files 通过；`cargo check -p zircon_runtime_interface --lib --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m7-current --message-format short --color never` 通过；`cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m7-current --message-format short --color never` 通过；`cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m7-current --message-format short --color never` 通过；`cargo test -p zircon_runtime_interface --lib contracts --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m7-current --message-format short --color never` 通过，57 passed / 0 failed；`cargo test -p zircon_runtime --lib repeated_same_target_mouse_moves_do_not_dirty_or_rebuild_surface --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m7-current --message-format short --color never` 通过，1 passed / 0 failed；`cargo test -p zircon_runtime --lib diagnostics --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m7-current --message-format short --color never` 通过，17 passed / 0 failed；`cargo test -p zircon_runtime --lib hit_grid --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m7-current --message-format short --color never` 通过，12 passed / 0 failed；`cargo test -p zircon_editor --lib ui_debug_reflector --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m7-current --message-format short --color never` 通过，14 passed / 0 failed；`cargo test -p zircon_editor --lib presenter::tests --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m7-current --message-format short --color never -- --nocapture` 通过，8 passed / 0 failed；`cargo test -p zircon_editor --lib rust_owned_host_window_snapshot_draws_top_right_debug_refresh_rate --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m7-current --message-format short --color never` 通过，1 passed / 0 failed。现有 runtime/editor warning noise 保留。

### M8 清理与验收

缺失内容：

- 旧 host 专用路径、兼容分支、重复投影逻辑的 M8.1a production audit 已关闭：2026-05-07 09:05 +08:00 已关闭 `root_shell_projection.rs` 的 root-shell geometry fallback 残留，2026-05-07 09:23 +08:00 已关闭 drawer resize hit surface 的 `geometry.splitter_frame(...)` 重复 truth source，2026-05-07 09:48 +08:00 已关闭 floating-window projection module 内 geometry-backed helper/fallback，2026-05-07 11:03 +08:00 已关闭 shell pointer bridge/drag surface 与 tab-drag strip hitbox 的 root-frame geometry fallback 重建路径，2026-05-07 11:26 +08:00 fresh production audit 对旧 geometry/root-frame、drawer/menu root binding、UiAssetEditor control-specific callback/property 模式均为 0 命中。
- editor/runtime 同源 `.ui.toml` golden 已完成 M8 fresh rerun：2026-05-07 12:09 +08:00 `runtime_ui_golden` 2 passed / 0 failed。
- acceptance/docs 已新增并最终更新 `tests/acceptance/ui-m8-final-cleanup-acceptance.md`，汇总实现文件、测试证据、debug snapshot、GUI artifacts、broad validation 和 M8.T owner/risk 记录。
- CI 等价 broad gates 已在本轮本地目标目录证明：root workspace `cargo build/test --workspace --locked` 与 plugin workspace check/build/test 均通过；第一次 root test 暴露的 interface dependency boundary 白名单问题已在最低共享层修复并复验。
- M3 GUI screenshot gate 证据和 artifact 已存在并已汇入 M8.2 acceptance index；2026-05-07 12:09 +08:00 M8 fresh screenshot rerun 1 passed / 0 failed，并刷新八张 `editor-window-m3-*.png` artifact。

当前 M8.1 证据：

- 2026-05-07 09:05 +08:00：`zircon_editor/src/ui/slint_host/root_shell_projection.rs` 不再通过 `WorkbenchShellGeometry` 派生 root layout、region frame、center/status frame 或 splitter frame；`resolve_root_*` helpers 只从 `BuiltinHostRootShellFrames` 取 shared projection frame，shared frame 缺席时返回 empty/default。
- Source guard：`workbench_root_shell_projection_uses_shared_frames_without_geometry_fallback` 固定禁止 `derive_layout_frames_from_geometry_with_shared_root`、`root_geometry_*`、`.region_frame(`、`.splitter_frame(`、`geometry.region_frame`、`geometry.splitter_frame` 回流。
- 验证：`rustfmt --edition 2021 --check zircon_editor/src/tests/host/slint_tab_drag/root_projection.rs zircon_editor/src/ui/slint_host/root_shell_projection.rs zircon_editor/src/tests/ui/boundary/workbench_projection_cutover.rs` 通过；`cargo test -p zircon_editor --lib workbench_root_shell_projection_uses_shared_frames_without_geometry_fallback --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8 --message-format short --color never -- --nocapture` 1 passed / 0 failed；`cargo test -p zircon_editor --lib slint_tab_drag::root_projection --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8 --message-format short --color never -- --nocapture` 5 passed / 0 failed；`cargo test -p zircon_editor --lib workbench_projection_cutover --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8 --message-format short --color never -- --nocapture` 4 passed / 0 failed。既有 runtime/editor warning noise 保留。
- 2026-05-07 09:23 +08:00：`zircon_editor/src/ui/slint_host/shell_pointer/resize_surface.rs` 不再直接读取 `WorkbenchShellGeometry.splitter_frames`；resize hit targets 改为调用 root-shell shared splitter resolver，并由 `HostShellPointerBridge::update_layout_with_root_shell_frames(...)` 传入 `BuiltinHostRootShellFrames`。`drawer_resize.rs` 删除无 shared frame fallback 的 test-only helper，drawer resize hit/capture fixture 清空旧 `splitter_frames`，显式验证 shared root frames 下的 left/right/bottom resize surface。验证：`rustfmt --edition 2021 --check zircon_editor/src/ui/slint_host/shell_pointer/resize_surface.rs zircon_editor/src/ui/slint_host/shell_pointer/bridge.rs zircon_editor/src/ui/slint_host/drawer_resize.rs zircon_editor/src/tests/host/slint_drawer_resize/pointer_bridge.rs zircon_editor/src/tests/host/slint_drawer_resize/resize_target.rs zircon_editor/src/tests/host/slint_drawer_resize/surface_contract.rs` 通过；`cargo test -p zircon_editor --lib slint_drawer_resize --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8 --message-format short --color never -- --nocapture` 9 passed / 0 failed。既有 warning noise 保留。
- 2026-05-07 09:48 +08:00：`zircon_editor/src/ui/slint_host/floating_window_projection.rs` 不再保留 `floating_window_projection_shared_source_from_geometry(...)`、geometry-backed bundle builder、`resolve_floating_window_*_frame(...)` legacy helpers 或 `resolve_floating_window_projected_outer_frame_with_fallback(...)`。floating-window projection bundle 只从显式 `FloatingWindowProjectionSharedSource`、有效 requested frame 或 native host bounds 取值；`HostShellPointerBridge` 的 floating-window test update helper 也会先构造 projection bundle，不再把 `WorkbenchShellGeometry.floating_window_frames` 当 fallback。Source guard：`floating_window_projection_uses_shared_source_without_geometry_fallback` 固定禁止 `WorkbenchShellGeometry`、`.floating_window_frame(`、`geometry.region_frame` 和 `geometry.center_band_frame` 回流。验证：`rustfmt --edition 2021 --check zircon_editor/src/ui/slint_host/floating_window_projection.rs zircon_editor/src/ui/slint_host/shell_pointer/bridge.rs zircon_editor/src/tests/host/slint_tab_drag/support.rs zircon_editor/src/tests/host/slint_document_tab_pointer/dispatch.rs zircon_editor/src/tests/host/slint_document_tab_pointer/floating_strip_bounds.rs zircon_editor/src/tests/host/slint_window/native_window_targets.rs zircon_editor/src/tests/host/slint_menu_pointer/visual_screenshot.rs zircon_editor/src/ui/slint_host/ui/tests.rs zircon_editor/src/tests/ui/boundary/workbench_projection_cutover.rs` 通过；`cargo test -p zircon_editor --lib floating_window_projection --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8 --message-format short --color never -- --nocapture` 11 passed / 0 failed；`cargo test -p zircon_editor --lib slint_tab_drag::floating_pointer --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8 --message-format short --color never -- --nocapture` 5 passed / 0 failed；`cargo test -p zircon_editor --lib slint_window::native_window_targets --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8 --message-format short --color never -- --nocapture --test-threads=1` 1 passed / 0 failed；`cargo test -p zircon_editor --lib slint_document_tab_pointer --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8 --message-format short --color never -- --nocapture --test-threads=1` 5 passed / 0 failed；`cargo test -p zircon_editor --lib workbench_projection_cutover --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8 --message-format short --color never -- --nocapture` 5 passed / 0 failed。既有 warning noise 保留。
- 2026-05-07 11:03 +08:00：`zircon_editor/src/ui/slint_host/shell_pointer/bridge.rs` 不再在 `shared_root_frames` 缺席时用 `WorkbenchShellGeometry` 重建 `BuiltinHostRootShellFrames`；`shell_pointer/drag_surface.rs` 不再接收 geometry，也不再读 center/status/document/drawer/floating-window geometry fallback。`zircon_editor/src/ui/slint_host/tab_drag/strip_hitbox.rs` 的精确 tab-strip 命中同样只读 shared root frames，`drop_resolution.rs` / `route_resolution.rs` 当前入口已删除旧 geometry 参数并直接接收 shared root frames。新增 source guards：`shell_pointer_bridge_does_not_recreate_root_frames_from_geometry`、`shell_pointer_drag_surface_uses_shared_root_frames_without_geometry_fallback`、`tab_drag_strip_hitbox_uses_shared_root_frames_without_geometry_fallback`。验证：生产 `slint_host` 源码检索对 `geometry.region_frame`、`geometry.splitter_frame`、`geometry.center_band_frame`、`geometry.status_bar_frame`、`geometry.viewport_content_frame`、`geometry.floating_window_frame`、`WorkbenchShellGeometry {`、`root_frames_from_geometry`、`shared_or_geometry_frame`、`shared_or_fallback_frame` 无命中；`rustfmt --edition 2021 --check` 覆盖本批 shell pointer/tab drag/source guard 文件通过；`cargo test -p zircon_editor --lib workbench_projection_cutover --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8 --message-format short --color never -- --nocapture` 8 passed / 0 failed；`cargo test -p zircon_editor --lib slint_tab_drag --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8 --message-format short --color never -- --nocapture --test-threads=1` 34 passed / 0 failed；`cargo test -p zircon_editor --lib slint_drawer_resize --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8 --message-format short --color never -- --nocapture` 9 passed / 0 failed。既有 runtime/editor warning noise 保留。
- 2026-05-07 11:26 +08:00 fresh stale-path production audit：production 非测试 `zircon_editor/src/ui/slint_host/**/*.rs` 排除 `tests.rs` 后，对 `geometry.region_frame`、`geometry.splitter_frame`、`geometry.center_band_frame`、`geometry.status_bar_frame`、`geometry.viewport_content_frame`、`geometry.floating_window_frame`、`WorkbenchShellGeometry {`、`root_frames_from_geometry`、`shared_or_geometry_frame`、`shared_or_fallback_frame`、`derive_layout_frames_from_geometry`、`resolve_floating_window_projected_outer_frame_with_fallback`、`floating_window_projection_shared_source_from_geometry`、`build_floating_window_projection_bundle(geometry` 均为 0 命中。production `slint_host` 与 `assets/ui/editor/host` 对 `left/right/bottom_drawer_extent`、`set_*_menu_button_frame`、旧 UiAssetEditor `ui_asset_*selected/activated` callback、pane-local `palette_selected(`/`binding_selected(`/`layout_semantic_selected(` 也均为 0 命中。模块内 `app/tests.rs` / `ui/tests.rs` 保留 21 个旧 geometry fixture 命中，用来构造 stale geometry 输入并证明 shared projection 覆盖，不计为 production 残留。

当前 M8.2 证据索引：

- 2026-05-07 11:15 +08:00：新增 `tests/acceptance/ui-m8-final-cleanup-acceptance.md`，把 M8.1 stale geometry cleanup、M4.3/M4.T editor/runtime `.ui.toml` semantic golden、M7 Debug Reflector/debug snapshot、M3 GUI screenshot artifacts 汇总为 final cleanup acceptance index。该索引明确只接受“已有证据可追溯”，不把 root workspace CI、plugin workspace CI 或 M8 final fresh rerun 标为完成。
- Golden 证据：`runtime_ui_golden.rs` 覆盖 HUD、pause、settings、inventory、quest log 五组 editor/runtime fixture pair；既有记录为 `runtime_ui_golden` 2 passed / 0 failed，runtime manager 3 / 0，runtime asset/compile/resource/render fixture gates 2 / 0、9 / 0、11 / 0、33 / 0。
- Debug 证据：`ui-debug-reflector-full-closure.md` 与 `ui-m7-invalidation-performance.md` 记录 editor Debug Reflector、runtime diagnostics、hit grid、presenter/top-right marker gates；既有 M7 current 记录为 editor `ui_debug_reflector` 14 / 0、runtime diagnostics 17 / 0、hit_grid 12 / 0、interface contracts 57 / 0、presenter 8 / 0、top-right marker 1 / 0。
- GUI 证据：`target/visual-layout/editor-window-m3-*.png` 当前保留 Welcome、Workbench、Asset Browser、drawer、menu popup/SVG、drag-release、small/large SVG scale 八张 artifact；既有 screenshot gate 为 `capture_m3_gui_acceptance_visual_artifacts` 1 passed / 0 failed。
- 2026-05-07 12:09 +08:00 M8.2 fresh focused gate 结果：先按 Cargo target disk policy 清理 `E:\zircon-build\targets-ui-m8-current`，释放 23.6GiB，空闲空间回到 57.9GB；随后 `cargo test -p zircon_editor --lib runtime_ui_golden --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never -- --nocapture` 2 passed / 0 failed；`cargo test -p zircon_editor --lib ui_debug_reflector --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never -- --nocapture` 14 passed / 0 failed；`cargo test -p zircon_runtime --lib diagnostics --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never -- --nocapture` 17 passed / 0 failed；`cargo test -p zircon_runtime --lib hit_grid --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never -- --nocapture` 12 passed / 0 failed；`cargo test -p zircon_editor --lib capture_m3_gui_acceptance_visual_artifacts --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never -- --ignored --nocapture --test-threads=1` 1 passed / 0 failed。GUI artifacts 刷新时间为 2026-05-07 12:08-12:09 +08:00。既有 runtime/editor warning noise 保留。

当前 M8.3 / M8.T broad validation 证据：

- 2026-05-07 12:20-12:39 +08:00：`cargo build --workspace --locked --verbose --jobs 1 --target-dir D:\cargo-targets\zircon-m8-current --message-format short --color never` 退出码 0，用时 18m33s；既有 runtime/editor warning noise 保留。
- 2026-05-07 12:39-12:51 +08:00：第一次 `cargo test --workspace --locked --verbose --jobs 1 --target-dir D:\cargo-targets\zircon-m8-current --message-format short --color never` 退出码 101，失败点为 `zircon_runtime_interface::tests::boundary::manifest_dependencies_stay_contract_only`，原因是 M6 shared text contract 已在 interface DTO 层使用 `unicode-segmentation` 做 grapheme 边界，但 boundary allowlist 仍未把它列为 contract-only dependency。
- 2026-05-07 12:52 +08:00：最低共享层修复为更新 `zircon_runtime_interface/src/tests/boundary.rs` 的 dependency allowlist。`rustfmt --edition 2021 --check zircon_runtime_interface/src/tests/boundary.rs` 通过；`cargo test -p zircon_runtime_interface --lib manifest_dependencies_stay_contract_only --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m8-current --message-format short --color never -- --nocapture` 1 passed / 0 failed。
- 2026-05-07 12:54-12:59 +08:00：完整 root workspace test rerun `cargo test --workspace --locked --verbose --jobs 1 --target-dir D:\cargo-targets\zircon-m8-current --message-format short --color never` 退出码 0；`zircon_runtime_interface` harness 最终 63 passed / 0 failed，并完成 doctests。
- 2026-05-07 13:00-13:10 +08:00：plugin workspace `cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --locked --all-targets --verbose --jobs 1 --target-dir D:\cargo-targets\zircon-m8-current --message-format short --color never` 退出码 0，用时 9m42s。
- 2026-05-07 13:12-13:27 +08:00：plugin workspace `cargo build --manifest-path zircon_plugins/Cargo.toml --workspace --locked --verbose --jobs 1 --target-dir D:\cargo-targets\zircon-m8-current --message-format short --color never` 退出码 0，用时 14m37s。
- 2026-05-07 13:30-13:36 +08:00：plugin workspace `cargo test --manifest-path zircon_plugins/Cargo.toml --workspace --locked --verbose --jobs 1 --target-dir D:\cargo-targets\zircon-m8-current --message-format short --color never` 退出码 0，用时 11m38s。
- 2026-05-07 15:12-15:45 +08:00：fresh current-target confirmation 通过。`cargo test -p zircon_runtime --lib material_icon_button_without_visual_icon_keeps_label_accessibility_only --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never -- --nocapture --test-threads=1` 1 passed / 0 failed；`cargo test -p zircon_runtime --lib runtime_fixture_assets_live_under_crate_assets --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never -- --nocapture --test-threads=1` 1 passed / 0 failed；`cargo test -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never` 944 passed / 0 failed；`cargo build --workspace --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never` 退出码 0；`cargo test --workspace --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never` 退出码 0；`cargo build --workspace --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-plugins --message-format short --color never` 在 `zircon_plugins` 工作区退出码 0；`cargo test --workspace --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-plugins --message-format short --color never` 在 `zircon_plugins` 工作区退出码 0。
- 2026-05-07 19:10-19:38 +08:00：scoped shell/root-frame continuation confirmation 通过。因 E: 当前低于 50GB Cargo target policy 阈值，本轮使用 `D:\cargo-targets\zircon-ui-m8-current`。`cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m8-current --message-format short --color never` 退出码 0；`cargo test -p zircon_editor --lib slint_drawer_resize --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m8-current --message-format short --color never -- --nocapture` 9 passed / 0 failed；`cargo test -p zircon_editor --lib workbench_projection_cutover --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m8-current --message-format short --color never -- --nocapture` 8 passed / 0 failed；`cargo test -p zircon_editor --lib slint_tab_drag --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m8-current --message-format short --color never -- --nocapture` 34 passed / 0 failed。最终 `rustfmt --edition 2021 --check` 覆盖本轮 M8 source/test set 通过；targeted production source guards 对旧 geometry/root-frame fallback 模式均 0 命中，宽 grep 只剩 module-local test fixture 命中。
- M8.T owner/risk：无阻塞 owner。非阻塞残留为既有 warning noise、共享 worktree 仍有 sibling dirty/untracked 状态，以及本轮 target 后 E: 空间低于 Cargo target policy 阈值；后续 Cargo 阶段应继续先按 target disk policy 清理或选择空间充足目标目录。

完成门槛：

- 旧路径移除后没有兼容 shim 或重复 truth source 残留。
- workspace 与 plugin validation 通过，失败项如果存在必须有明确 owner 和不阻塞理由。
- docs、acceptance、debug snapshot、GUI screenshots 都能支撑最终“完成”声明。

## Test Plan

- Runtime 单元测试：layout visibility/clip/z-order、hit grid、capture/focus/click、dirty rebuild、virtual list、scroll、component default actions。
- Editor 验收：同一 `.ui.toml` 在 Slint host 下完成命中、绘制、redraw、menu/popup/toolbar/template pane 路由。
- Renderer 验收：quad/text/image/SDF/clip/material batch/drawcall/overdraw counter 的 golden 或 pixel 测试。
- Text 验收：真实字体测量、wrap/ellipsis、source/visual range、BiDi direction 与 visual order、rich runs、editable caret/selection/edit actions、IME composition、editor/runtime 同源 layout snapshot。
- Debug 验收：Widget Reflector snapshot、hit-test dump、drawcall/overdraw/material batch 导出可复现。

## Assumptions

- “widget refactor debug 工具”按 Unreal Slate 的 **Widget Reflector** 类调试器处理。
- 不直接复制 Unreal API；只参照 Slate 的职责划分、数据流、调试能力和验收语义。
- V1 先完成 screen-space UI；world-space UI 预留 SurfaceFrame/renderer 接口，放在运行时里程碑之后扩展。
- 本次是审计和规划，没有声明测试已通过。
