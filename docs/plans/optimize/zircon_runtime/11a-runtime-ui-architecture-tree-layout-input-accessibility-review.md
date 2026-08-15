---
related_code:
  - zircon_runtime/src/ui
  - zircon_runtime_interface/src/ui
  - zircon_runtime/src/dynamic_api/session/runtime_ui.rs
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/dynamic_api/session/construction.rs
  - zircon_runtime/src/dynamic_api/session/project.rs
  - zircon_runtime/src/dynamic_api/session/extract.rs
  - zircon_app/src/entry/runtime_entry_app
  - zircon_editor/src/ui
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
  - docs/plans/zircon_runtime/runtime/09/2026-08-07-runtime-ui-incremental-refresh.md
  - docs/plans/zircon_runtime/runtime/09/2026-08-09-ui-architecture-performance-reassessment.md
  - docs/plans/zircon_runtime/runtime/09/failure-2026-07-17-woc-project-runtime-ui-bridge.md
  - docs/plans/zircon_runtime/runtime/09/failure-2026-07-19-dynamic-ui-extract-generation.md
  - docs/plans/performance/01/2026-07-23-runtime-interface-ui-ecs-contract-static-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore
  - dev/bevy/crates/bevy_ui
  - dev/bevy/crates/bevy_a11y
  - dev/Fyrox/fyrox-ui
  - dev/godot/scene/gui
  - dev/godot/servers/display
  - dev/godot/servers/display/accessibility_server.h
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 11A · Runtime UI 架构、Tree、Layout、Input 与 Accessibility 工程化差距

## 1. 结论

Zircon 的 retained UI 不是空实现。`UiSurface` 已拥有 tree、component state、typed dirty domain、layout、arranged tree、hit grid、render extract、focus、pointer/keyboard/gamepad/IME 路由、popup/tooltip、editable text 和 accessibility snapshot；v2 `.zui` 可编译为 retained tree。当前动态游戏会话的未跟踪 `RuntimeUiSurfaceSet` 也已取代早期 Vampire 专用 HUD/menu 的一部分旁路：项目 manifest 可以声明多个 UI root，产品 render/input/accessibility 会访问同一组 surface。这些基础应保留。

但产品闭环仍在最关键的边界断开。`RuntimeUiSurfaceSet` 把完整 `UiInputDispatchResult` 压缩为一个 `bool`，因此组件事件、binding report、clipboard、popup、tooltip、pointer lock、high-precision pointer、link activation 和 IME host request 都不会进入 app 或 gameplay。surface 内部会修改一部分控件状态，所以静态画面和局部单测可以呈现“可交互”；外部游戏逻辑实际上没有收到 action/value change 的权威通道。`UiEventManager` 虽由 `UiModule` 注册，却只被 Editor control service 使用，与动态游戏 surface 没有连接。

时间与窗口生命周期也没有进入产品 surface。每个 `UiInputManager` 从不 `tick`，所有事件 timestamp 恒为 0；double-click、tooltip、typeahead、submenu、toast、deferred focus/IME 生命周期不会按真实时间推进。动态会话只把 focus/window status 交给 core input，没有调用已有 `surface/input/window_pump.rs`，因此 DPI、raster scale、focus loss、occlusion、application active、close/destroy 和 transient teardown 都与 UI 脱节。焦点、pointer capture 或 IME owner 可在失焦后滞留。

模块架构没有拥有这些责任。`UiRuntimeDriver` 是 unit struct，`UiConfig.enabled` 无消费者；module descriptor 只创建空 driver 和孤立 event manager。真正 surface 生命周期藏在 dynamic session 的未跟踪文件中，既不参加模块 tick/teardown，也不能服务多 world、多 window、Editor/Play 或插件贡献。测试使用的 `RuntimeUiManager` 又明确位于 `#[cfg(test)]`。当前存在三种看似相近却未收敛的 owner。

Tree/layout 的增量基础是真实的，但工程声明超过实装。局部 dirty rebuild 会选择 subtree 并 patch arranged/hit/render；然而 Taffy bridge 每个 container 临时创建一棵只含 parent 和直接 child leaf 的 `TaffyTree`，计算后立即丢弃。它不是 Bevy 式 persistent Taffy graph，不能利用跨帧 dirty/measure cache，也不具有完整嵌套 CSS layout 语义。所谓 virtualized list 仍物化和测量全部 child，再隐藏不可见 subtree；通用 node pool 没有进入运行时数据源循环。

Tree 的公开可变图和 hit grid 还存在 P0 robustness 风险。`UiTree.roots/nodes/slots` 都可绕过 transaction 直接修改；大量 recursive traversal 没有统一 cycle/depth guard。Hit grid 根据所有 entry 的联合 bounds 直接分配 `columns * rows` 个 cell，没有 finite、checked arithmetic、最大 cell/bytes 或 authored-content budget。一个极远坐标或巨大 frame 可以导致溢出、巨量分配或 OOM。

Accessibility 已有较完整的中立 DTO、名称解析、诊断和动作回写，也有 optional AccessKit snapshot converter；但没有 app/editor 的 AccessKit window adapter、OS tree publication、incremental update、action callback 或 focus synchronization。产品只提供手工 JSON snapshot ABI。角色/关系/live-region/text semantics 也远窄于工程级桌面与游戏 UI。

本轮登记 7 项 P0、31 项 P1、8 项 P2。重构必须先建立唯一 product UI runtime owner，并让完整 dispatch result、时间、窗口和 host request 穿过 ABI；随后关闭 tree/hit-grid 安全缺口；再推进 persistent layout、真实 virtualization、navigation/popup、多窗口/world-space 和 OS accessibility。11B 单独审查 font/shaping/text layout/editing/IME 细节，11C 单独审查 GPU UI renderer、atlas、SDF、batch、clip 和提交性能。

## 2. 审查边界与证据

### 2.1 当前源码范围

| 集合 | 文件 / 物理行 | 本轮证据 |
|---|---:|---|
| `zircon_runtime/src/ui` architecture production | 370 / 70,644 | E3：module、tree、layout、surface、dispatch、binding、event、component、template/v2、accessibility、platform input |
| `zircon_runtime_interface/src/ui` architecture production | 196 / 17,732 | E3：tree/layout/focus/navigation/input/window/a11y/component/template/ECS DTO；text/render DTO 延后 |
| dedicated architecture tests | 372 / 89,208 | E2：1,482 个 `#[test]`、1 个 ignored；按发现读取关键行为，不把数量当产品闭环 |
| deferred UI text/render integration | 120 / 30,333 | queued：11B/11C，当前只追踪与 input/layout/a11y 相交调用点 |
| dynamic product bridge | 6 / 2,952 | E3：project load、construction、event dispatch、extract、host request drain、a11y capture |
| product bridge workspace state | 5 modified + 1 untracked | source recheck：`runtime_ui.rs` 未被 Git 跟踪，其他 5 个桥文件均为外部修改 |
| architecture overlap | 68 dirty production files | 不修改、不回退；实施前重新取指纹并复核 |

`zircon_runtime/src/ui` architecture production fingerprint 为 `414e14cd2dd03f73f10886b93d58d5d2aa26a4ebb2b2ef34e773f5316afe6e54`；`zircon_runtime_interface/src/ui` 对应 fingerprint 为 `6e3f27c235509e9699e9aa300b88a73ff484f6d55ada588ad9eea9af8b249359`。算法与 09H 系列一致：路径排序、逐文件 SHA-256，再对 `path<TAB>hash<LF>` 清单取 SHA-256。

本轮对 566 个 architecture production 文件做物理清单、结构与关键符号扫描，并沿高风险 owner 逐层深读。component catalog 的每个视觉 leaf、text shaping 和 GPU paint 实现没有在 11A 重复逐算法判断；它们分别进入 11B/11C。因大量当前源码由其他 Session 修改，本文只声称列出的 owner chain 和发现达到 E2/E3，不声称整个 UI/text/render 三域已完成。

### 2.2 产品链读取深度

本轮从 `ProjectManifest.ui_roots` 开始，追踪 asset registry、prototype store、v2 surface build、dynamic session construction、viewport extract、多-surface command 合并、pointer/keyboard/text/IME/gamepad/accessibility event、core/app host request drain、window/lifecycle event、accessibility capture ABI。内部再追踪 `UiInputManager` timer/IME、surface window pump、focus/navigation、popup stack、tree mutation、incremental layout、Taffy bridge、virtualization、pool、hit grid、event manager、reflection store、ECS projection和 AccessKit converter。

### 2.3 参考源码

- Bevy `UiPlugin` 注册 persistent `UiSurface` resource，并把 Focus、Prepare、Propagate、Content、Layout、PostLayout、Stack 放入明确 schedule；`layout/ui_surface.rs` 持久保存 entity-to-Taffy map 和 `TaffyTree`，以 upsert/update/remove 维护跨帧 graph。
- Fyrox `UserInterface` 同时拥有 keyboard focus、captured node、layout event、double-click timer 和 tooltip update；`update(screen_size, dt, switches)` 用真实 `dt` 推进时间行为。
- Unreal `FSlateApplication` 是 window/input/user focus/tick/modal/popup 的统一 application owner；Slate 使用 typed invalidation reason、invalidation root、widget path 和 platform accessibility bridge。
- Godot `Control` 通过 minimum-size/queue update 传播失效，`LineEdit` 将 IME active/candidate position直接同步到 `DisplayServer`，Control/AccessibilityServer 形成 OS-facing 更新与动作回调。
- Unity Graphics 仓内源码不包含 Unity 主 UI retained tree/input/accessibility 实现，11A 不用 graphics package 猜测闭源 UI 行为；11C 只在 GPU resource/batch/clip 范围引用可证明部分。

### 2.4 明确未做

本轮没有运行 Cargo、Editor、游戏产品、屏幕阅读器、WGPU、真实 IME、触控/手写笔或性能采样。原因不是以静态扫描代替验收，而是当前 UI/text/render 有 142 个工作区状态项，产品 `runtime_ui.rs` 又未跟踪；贸然编译不能代表稳定 owner。历史 `docs/tests` 中存在 UI 命名相关 log/PNG/RDC，但未建立与上述 fingerprint 的一一对应，不能证明 current source。

## 3. 可保留的真实基础

### 3.1 `UiSurface` 已形成 retained state 骨架

Tree、component state、focus、input transient state、arranged tree、hit index、render extract、window state和 dirty publication集中在一个 surface。后续应收敛 owner 和 publication，不应退回 immediate-mode command 拼装或 WOC/Vampire 专用 HUD 分支。

### 3.2 输入结果 DTO 的表达力足够作为迁移起点

`UiInputDispatchResult` 已能携带 reply、diagnostics、effects、host requests、component events 和 binding reports。缺口主要在产品桥将其丢弃，而不是再发明第四种 result。应以这个 typed result 为基础做 compact receipt/generation，而不是保留 bool-only 旁路。

### 3.3 Dirty domain 和局部 patch 不是空统计

`rebuild_dirty` 会把 layout dirtiness 向需要的 ancestor 扩大，只对受影响 subtree重算 layout，并尝试 patch arranged geometry、hit entry和 render command。该边界可以升级为 generation-owned immutable artifacts；问题是部分内部步骤仍全 subtree/full scan，以及 publication/report 语义不精确。

### 3.4 Accessibility 中立层可复用

Role/state/action、name/description/label relation、focus、bounds、诊断和 text-selection byte offset 已脱离平台库。AccessKit converter 也证明平台适配可以位于 host 层。应补齐 schema 和 adapter lifecycle，不应把 AccessKit 类型泄漏进 `zircon_runtime_interface`。

### 3.5 v2 prototype store 和 manifest root 是正确方向

项目可声明 UI roots，v2 compiler/build surface能解析跨文档 component/style。这个 authoring入口应保留；需要补 dependency-closed loading、compiled artifact generation、hot reload transaction和 runtime data binding。

## 4. P0：先修复产品正确性与安全边界

### P0-1：产品把完整 UI dispatch result 丢成 `bool`

`RuntimeUiSurfaceSet::dispatch_input`、`dispatch_pointer_to_surface` 和 `dispatch_accessibility_action` 只返回 `result.reply.stops_propagation()`。`dynamic_api/session/events.rs` 因而只能决定是否继续 gameplay input。`component_events`、`binding_reports` 与 `host_requests` 全部消失。

这会直接破坏 Button/action、TextInput value change、clipboard、context popup、tooltip、pointer lock、high-precision pointer、rich link 和 IME host sync。测试可以检查 result 内事件存在，但产品调用方永远看不到。必须让 session owner收集 typed UI result，按 surface/input sequence 发布 gameplay event、binding mutation receipt和 host request；handled 只是一项字段。

### P0-2：运行时 surface 没有真实时钟和 `UiInputManager::tick`

`next_input_metadata()` 的 timestamp 永远是 default，只有 sequence递增。动态会话只 tick core runtime/operations，不 tick每个 UI input manager。double-click expiry、tooltip delay、typeahead reset、submenu hover、toast expiry以及 deferred focus/IME lifecycle因此没有产品时间推进。

必须由唯一 UI driver在固定 frame phase接收 monotonic timestamp和clamped delta，先处理window/lifecycle，再处理input queue，再推进timer，最后提交mutation/layout/extract。事件 timestamp不得由每个 adapter临时默认。

### P0-3：窗口失焦、DPI、销毁和 application lifecycle 未进入 UI

内部 `window_pump.rs` 已能处理 metrics、scale factor、position、focus、application active、occlusion、close、closed、destroyed和 redraw reason；dynamic product path没有调用它。`events.rs` 的 FocusLost/WindowStatus只进入 core input。`render_extract.raster_scale` 因而保持 1.0，pointer capture/focus/IME/popup在失焦或销毁后可能滞留。

必须按 window/surface identity建立 geometry barrier：resize/scale后第一项位置事件必须消费新 layout/hit generation；focus/application deactivate必须取消capture/drag、提交或取消composition、dismiss transient UI并发出host release；destroy必须完成反向 teardown。

### P0-4：`UiModule` 没有拥有实际 runtime UI 生命周期

`UiRuntimeDriver` 无状态无方法，`UiConfig.enabled` 无消费者。module只注册空driver和孤立`UiEventManager`；真实surface set由dynamic session私有构造，test manager又位于`#[cfg(test)]`。模块 deactivation、world unload、session destroy、plugin unload和window destroy都没有共同 UI owner。

应建立 `UiRuntimeService`/driver，按 world + window/render target拥有generational surface registry、input queues、host bridge、publication和teardown。dynamic API、Editor/Play和embedded host只适配同一服务，不再各建 manager。

### P0-5：游戏逻辑没有权威 UI action/data-binding 接口

内部控件 reducer会修改component state并生成component events，但 dynamic runtime既不消费它们，也不连接`UiEventManager`。当前`UiEventManager`只在Editor control service有生产消费者；其reflection property又是独立snapshot，不会反向修改live surface。项目加载后的surface也没有公开mount/unmount、model update、command/action sink或script/plugin binding。

必须定义 game/UI 边界：compiled binding target指向typed gameplay command/model field；UI event提交事务生成compact receipt；game state变化以generation/delta回投surface。禁止用字符串反射mirror成为第二truth，也禁止业务代码直接拿`UiTree.nodes.get_mut()`。

### P0-6：Hit grid 可被 authored geometry 触发无界分配或溢出

`build_hit_grid` 对所有entry联合bounds按64像素切格，直接执行 `(columns * rows) as usize` 和 `vec![cell; count]`。没有 finite/negative/maximum extent、checked multiplication、cell/bytes budget或sparse fallback；大entry还会复制到每个相交cell。极远坐标、NaN/Inf或巨大UI资产可造成panic、overflow、OOM或长时间卡死。

导入/compile边界必须验证finite geometry和规模；runtime spatial index必须checked allocation并受surface budget约束。超预算应返回typed diagnostic并降级到bounded BVH/quadtree或拒绝surface，不能让内容数据决定任意内存分配。

### P0-7：公开 Tree 可绕过不变量，递归遍历缺少统一防环/深度门禁

`UiTree.tree_id/roots/nodes/slots` 公开，roots/slots可直接变更而不产生dirty transaction；`node_mut`只标记该node，无法维护parent/children/slot/root一致性。多条measure、focus、subtree和layout递归依赖图无环且深度合理。反序列化或host mutation可制造dangling edge、重复parent、cycle和深链，随后触发错误命中、漏失效或栈溢出。

Tree应变为封装的generational arena，只允许validate-then-commit transaction执行insert/remove/reparent/reorder/slot change。compile/import一次验证identity、single parent、root reachability、acyclic、depth和budget；runtime traversal仍使用iterative stack与visited/depth guard，不能把资产验证当唯一安全层。

## 5. P1：架构、规模和产品能力差距

### P1-1：当前产品 UI owner 存在于未跟踪源码

`zircon_runtime/src/dynamic_api/session/runtime_ui.rs` 是 `??`，而construction/events/extract/project/state均已修改引用它。任何计划、测试或artifact若不记录这一工作区状态，都可能在clean checkout根本没有当前桥。先由原owner完成提交/整合和current-source测试，再实施本文；本轮不接管该文件。

### P1-2：启动会 eager load 项目 registry 中全部 UI 资产

`project_ui_prototype_store` 遍历所有 `UiLayout/UiWidget/UiStyle` entry并逐个`load_artifact`，最后才`build_for_roots`。大型项目的无关Editor UI、DLC/package UI和未使用skin都会进入启动I/O、解压和内存。应从root dependency graph按content generation加载，支持async priority、last-good和missing dependency diagnostics。

### P1-3：Surface root 是启动时静态集合

没有运行时mount/unmount、layer activation、hot reload、asset watcher、theme/locale generation或model refresh入口。UI root改变只能重建session。应有transactional surface registry和dependency generation；reload失败保留last-good surface与focus/scroll/state migration receipt。

### P1-4：多 Surface 只按 manifest 顺序拼接，没有组合语义

render把所有commands flatten到一个Vec，input反序遍历；没有typed layer、modal barrier、per-surface visibility、global focus arbiter、popup portal、capture arbitration或独立scale。最终`raster_scale`取循环中最后一个surface。应发布ordered segment handles和composite generation，并显式定义modal/focus/input/render/a11y次序。

### P1-5：Global node ID 用48位mask静默截断

`global_node_id` 把local ID与`NODE_ID_LOCAL_MASK`相与，再把surface index左移48位；没有local ID上界、surface count或collision校验。超过48位的local identity会静默别名，超过16位surface namespace也会越界。应使用typed `{surface handle generation, node handle generation}`，只在ABI codec边界做checked packing或table handle。

### P1-6：Runtime UI 绑定camera viewport，不是window/render-target模型

初始layout硬编码1280x720，后续只读取camera controller viewport。没有WindowId、render target、split viewport、safe area、orientation、DPI、per-user或headless target。多窗口Editor/Play、local multiplayer、offscreen UI和XR都无法拥有独立surface metrics。

### P1-7：Taffy bridge 是逐container临时叶子树

`compute_taffy_child_frames` 每次创建新`TaffyTree`，只把direct child作为leaf，compute后丢弃。nested measure仍由Zircon递归提供。这满足“Taffy只有一个调用入口”的结构守卫，却没有persistent node identity、dirty cache、measure context update和跨层CSS graph。应由surface generation拥有一棵persistent graph，tree transaction直接upsert/remove/reparent。

### P1-8：Layout 后端语义是混合近似而非统一合同

Horizontal/Vertical/Wrap/Grid/Block部分走Taffy，Free/Canvas/Container/Overlay/Space/Size/Scrollable/Masonry走Zircon。Grid bridge又只生成均匀`fr(1)`track。跨后端嵌套的min/max/content sizing、baseline、percentage、overflow、rounding和grid track语义没有一份oracle。需要明确每种container的canonical semantics与unsupported error，不能靠fallback改变结果。

### P1-9：局部 rebuild 仍会递归收集完整受影响 subtree

incremental path是真实patch，但每个root仍遍历完整subtree、snapshot geometry并更新多个wide artifact；Taffy也无跨帧cache。单leaf变化若ancestor是auto layout可扩大为大subtree。应以persistent graph和changed frontier驱动measure/layout/paint damage，并记录内部lookup/ancestor probes，不只报告outer visited。

### P1-10：Rebuild report 会把局部 patch 记成整体 rebuilt

部分路径即使成功patch arranged/render，report仍设置`arranged_rebuilt`/`render_rebuilt`。诊断无法区分full rebuild、local patch、metadata-only publication。必须以typed outcome和准确counter记录visited/changed/reused/allocated bytes，否则性能门禁会false green。

### P1-11：`UiSurface` 可序列化且运行缓存被跳过

Surface同时包含authoring/runtime mutable state，并通过serde跳过部分index/cache。反序列化后可能得到tree/component/input状态与默认cache/generation组合，依赖调用方手工full rebuild。应只序列化versioned asset/state snapshot DTO；live surface、handler、capture、IME、timer、cache和publication handle禁止直接serde。

### P1-12：Tree 顺序插入是 O(N²)

每次`insert_root/insert_child`都扫描所有nodes求最大paint order；模板顺序构树因此是O(N²)。bulk compile应一次生成dense paint order，动态tree维护单一next-order cursor；deserialize只重建一次并校验overflow。

### P1-13：Tree mutation index 在反序列化后丢失

`UiTreeNodes`的derived mutation index不会随payload恢复；initial content与随后mutation可能要求额外full rebuild或产生不精确dirty。应让generation publication从canonical transaction log构建changed set，不把可丢失的side index当truth。

### P1-14：每次导航都全树收集并排序候选

Tab/方向/gamepad路径会递归构建`Vec<NavigationCandidate>`，多处sort；active modal又扫描全树。10k/100k节点界面中，按键成本随总节点增长。应在layout/focus generation发布focus graph、tab order和spatial index，局部visibility/focusability变化只patch相关scope。

### P1-15：多个公开 focus/navigation 契约未被生产消费

`UiFocusContract.restore_on_close` 只有声明；`UiNavigationBoundary`主要只在测试出现；group `parent`/`wrap`没有形成完整生产策略。interface的`focus_chain(tree)`与runtime navigation又是两套算法。必须收敛一份focus scope graph和restore stack，删除无消费者字段或完成产品语义。

### P1-16：Modal/focus/popup 依赖组件名和属性别名

Dialog/ConfirmDialog/Modal/Popover/Menu以及`open/popup_open`、camel/snake alias在focus与popup实现中硬编码。自定义组件、插件组件或重命名会绕过modal trap和dismissal。compiled component descriptor应发布typed behavior/focus/popup handles，hot path不解析String/TOML。

### P1-17：所谓 virtualized list 只是可见性裁剪

`compute_virtual_list_window`只按固定item extent算index。layout仍持有全部child，measure阶段先测量全部节点，arrange scrollable也为全部child算位置，再隐藏window外subtree。这不是data-source virtualization，无法承载10万/百万item、variable height、async data、anchor correction或focus/a11y虚拟集合。

### P1-18：Node pool 没有成为通用运行时回收器

当前pool主要由Editor virtual row bridge调用；key包含完整node path，使跨row复用非常窄；没有capacity/byte budget/eviction或resource generation。运行时surface set没有data provider/pool调用。应由virtual collection owner按template handle + item kind回收instance，状态重置和binding rebinding必须可验证。

### P1-19：Popup stack 同步仍需扫描Tree并识别字符串

popup open状态通过全树metadata扫描、组件名和boolean属性推断。没有跨surface popup portal、native window owner、placement/monitor/safe-area、nested menu aim或global modal。应由popup transaction显式open/close/owner/parent/restore focus，stack变化不从全树反推。

### P1-20：Pointer/navigation handler 注册没有生命周期token

dispatcher按node/kind保存`Arc`/closure，但没有unregister/owner generation。tree节点移除、hot reload或plugin unload后handler生命周期无法原子退休。route invocation还clone完整context/result。应返回generational subscription token，随surface/plugin generation teardown，并将normal-path diagnostics置于debug gate。

### P1-21：`UiEventManager` route 覆盖会留下stale entry

同一`native_binding`再次注册会覆盖`routes_by_binding`，旧`routes_by_id`仍保留；没有remove route。`register_route_stub`又能创建必然以“no execution handler”失败的产品route。需要immutable compiled route generation、duplicate policy、unload token和atomic publication。

### P1-22：UI notification subscriber 是无界队列和全量clone fanout

`subscribe()`使用`crossbeam_channel::unbounded`；broadcast对每个subscriber clone notification。慢/失联Editor或remote consumer可无界积压wide reflection/invocation payload。必须有per-subscriber entry/bytes/age预算、coalesce/drop/disconnect policy和diagnostics。

### P1-23：Reflection store 是可漂移的第二棵 UI 状态树

`replace_tree`保存owned `UiReflectionSnapshot`，`set_property`只修改snapshot中的JSON值并广播diff，不会修改`UiSurface`。`rebuild_node_index`每次全扫所有tree。应只发布live surface generation的read-only reflection artifact；写操作解析到surface transaction，不能在mirror里成功后让产品画面不变。

### P1-24：输入设备语义不足且adapter重复

Winit translator只识别左/右/中键，额外mouse buttons丢失；TabletTool被当mouse，DTO没有pressure、tilt、twist、contact geometry。dynamic ABI又手写一套conversion，而`runtime_event_adapter.rs`无产品消费者。需要唯一platform-normalization owner并保留window/device/pointer/user identity。

### P1-25：UI host request 与 core host request 是两条断开的合同

app可执行core cursor/IME/rumble request；UI manager只内部累计部分IME request，其余host request留在dispatch result。`RuntimeDynamicSession::drain_host_requests`只drain core input manager。应由UI service将request转换为统一host envelope，保留surface/owner/sequence/generation和结果回执；host failure要反馈UI state。

### P1-26：IME platform bridge 只实现了一小部分

dynamic event把preedit clauses固定为空，IME Enabled不进入surface；Editor host主要只调用`set_ime_allowed`，没有完整candidate cursor area和surrounding text lifecycle。11B会审查文本offset/selection算法；11A要求先保证Enable/Disable/Reset/cursor/surrounding/delete-surrounding按window owner可达且focus loss成对释放。

### P1-27：UI ECS 是诊断投影，不是调度执行层

`UiEcsProjectionSnapshot`从surface全量构建node DTO，再多轮派生totals、10-stage impacts和8-domain impacts；产品schedule不消费它来执行系统。它有价值作为diagnostics，但不能称作Bevy-style ECS UI。应让surface transaction/changed set直接驱动真正stage runner，diagnostic只借用published generation。

### P1-28：Accessibility 没有OS host adapter

`accessibility-accesskit`只启用optional `accesskit`，converter为`pub(crate)`；app/editor没有AccessKit Winit adapter或对应feature。没有window adapter activation、TreeUpdate push、action callback、focus event和shutdown。JSON capture不能让NVDA/VoiceOver/Orca访问产品。

### P1-29：Accessibility schema不足以描述工程级控件

当前role缺少heading、link、switch、progress、tree/treeitem、table/grid/cell/row、combobox、listbox/option、separator、status/live region等；relation只有labelled_by/label_for，缺少described_by、controls/owns、flow_to；也没有language、orientation、level、position/set size、range min/max/step、live politeness或rich text runs。不能仅把Generic映射成container掩盖语义缺失。

### P1-30：Accessibility capture/action 都会重建完整快照

extract每次多轮遍历surface tree、构造BTreeMap/BTreeSet/owned Vec并validate；`snapshot.node()`线性查找。AccessKit converter每次生成包含全部nodes的TreeUpdate。应按accessibility generation发布indexed immutable tree和delta，stable capture/OS update不重建，single node变化只发布受影响关系闭包。

### P1-31：`WorldSpaceSurface` 是目录合同，不是产品功能

component catalog声明world transform、pixels-per-meter、billboard、depth-test、camera target和`WorldSpaceUi` capability，测试也验证descriptor；但scene/dynamic runtime没有world-space surface component、camera projection、offscreen target、ray hit、occlusion/depth或render consumer。`runtime_world_space()`只是能力集合构造器。接线前产品必须报告unsupported，不能把catalog entry当功能交付。

## 6. P2：诊断、性能证据与维护差距

### P2-1：Layout profiling 依赖环境变量和 `eprintln!`

`ZR_UI_LAYOUT_PROFILE`输出没有进入统一profiler、frame generation或structured diagnostics。应把stage duration、visited/changed/reused、allocation bytes和fallback reason接入现有trace/counter；release hot path默认零格式化。

### P2-2：大量测试没有产品桥反例

1,482个test属性覆盖了许多DTO、reducer和source contract，却没有current-source产品测试证明component event到gameplay、clipboard round trip、timer tick、focus loss capture cleanup、DPI update和OS a11y。应优先补断链反例，而非继续增加只检查result内部Vec的单测。

### P2-3：现有artifact无法绑定当前源码

历史UI log/PNG/RDC和计划产出很多，但缺少fingerprint、binary hash、feature set、GPU/OS/scale/locale和exact command。当前桥未跟踪使旧artifact更不能代表clean checkout。产品验收bundle必须携带这些metadata。

### P2-4：缺少规模与内存安全基准

当前没有以1/1k/10k/100k nodes、1M events、100k virtual items、极端bounds、slow subscriber和多surface测量alloc/visits/RSS/p95的current-source结果。性能计划已有多个正确预算，仍需真正counter和benchmark落地。

### P2-5：AccessKit text position转换缺少跨平台oracle

converter把AccessKit character index按grapheme cluster换算为UTF-8 byte offset。组合字符、ZWJ emoji和平台text-provider对“character”的定义必须用AccessKit/OS行为测试证明，不能只靠serde/unit example。11B拥有最终text index contract。

### P2-6：多处hot path仍拥有wide clone/String/TOML解析

route context、component event、binding、reflection、node metadata和attribute alias在input/navigation/a11y路径重复clone或解析。既有PERF-MVP-254/265/274/278/283/572已经拥有性能重构编号；11A只要求它们绑定同一surface/component generation，不另建cache truth。

### P2-7：错误和fallback缺少产品可观测性

unsupported component、Taffy fallback、host request丢失、a11y unsupported action、surface input error和global ID拒绝没有统一运行时health event。应提供bounded structured diagnostic，Editor可定位asset/node/path/generation，shipping可聚合计数而不泄漏文本内容。

### P2-8：现有计划状态需要按能力而非结构守卫纠正

Runtime09保持`in_progress`是正确的，但部分子记录把“单入口”“测试名存在”“converter存在”写成完成。本文要求按第11节重开；不删除历史证据，只把结构完成与产品完成分开。

## 7. 目标架构与唯一所有权

### 7.1 Runtime owner

建立由`UiModule`注册的`UiRuntimeService`，其生命周期低于dynamic ABI、Editor和plugin host：

1. `UiSurfaceRegistry`以`WorldId + UiTargetId + UiSurfaceHandle(generation)`拥有surface；target可为window viewport、camera viewport、offscreen texture或world-space surface。
2. `UiFrameScheduler`定义 Window/Lifecycle -> Input Collect -> Focus/Widget/Timer -> Model Transaction -> Text Measure -> Layout -> Post Layout -> Picking/A11y/Render Publication -> Host Request Drain 的固定phase。
3. `UiHostBridge`统一clipboard、IME、cursor、pointer lock、popup、tooltip、link和a11y adapter；每个request有owner、sequence、deadline和result。
4. `UiPublication`原子发布tree/layout/hit/a11y/render/component子generation和changed receipts；consumer只能借用handle。
5. destroy/deactivate按反向顺序停止输入、撤销host owner、关闭popup/IME、退休handler/plugin generation、等待reader pin，再释放surface/assets。

### 7.2 Identity 和 transaction

- `UiSurfaceHandle`与`UiNodeHandle`必须generational；ABI可用opaque table handle，不再用48位mask静默压缩。
- Tree字段私有，所有结构与属性变化进入`UiMutationTransaction`；validate全部操作后一次commit，每domain generation每frame最多推进一次。
- compiled component/template generation提供dense descriptor/field/event/slot/behavior handle；surface node只保存handle和mutable state delta。
- gameplay model/action bridge以typed binding handle和command receipt通信；reflection/a11y/diagnostics只投影同一publication。

### 7.3 Layout、virtualization 和 spatial index

- 每个surface generation拥有persistent Taffy graph或明确的非Taffy owner；node upsert/remove/reparent与tree transaction同提交。
- unsupported layout语义在compile时报错或明确选择canonical Zircon backend，禁止运行时silent fallback改变布局。
- Virtual collection以data source、item key、template handle、estimate/measure cache、overscan、recycle pool和anchor correction为核心；只物化visible + overscan。
- Hit index使用checked extent/budget和sparse spatial structure；全屏backdrop不复制到无界cell，radius query有dedup scratch和cost budget。

### 7.4 Focus、popup 和 accessibility

- 每个user/device拥有focus path，global arbiter处理modal scope、restore stack、navigation boundary和cross-surface order。
- Popup由显式stack transaction拥有，不从组件字符串反推；native/embedded portal共享placement、dismiss和focus contract。
- Accessibility以surface publication生成indexed neutral tree/delta；host adapter按window激活并把OS action回投同一input queue。
- text、selection和range index contract由11B统一定义；平台adapter不得自创第二套offset转换。

## 8. 硬切范围

1. 删除bool-only UI dispatch产品接口；调用方一次迁移到typed `UiProductDispatchReceipt`。如动态ABI需兼容旧宿主，只能在ABI版本协商处明确拒绝/降级，不保留内部双路由。
2. `UiRuntimeDriver`由空类型硬切为真实service driver；dynamic session不再私有创建独立surface set。
3. `UiTree` graph字段转私有；删除直接`nodes.get_mut`的产品写路径，测试fixture通过builder/transaction构造。
4. 删除48位global node packing；a11y/input/render diagnostics使用surface/node opaque identity。
5. `UiEventManager` reflection mirror写接口要么路由到surface transaction，要么改为read-only；不得继续假成功。
6. 删除runtime字符串识别modal/popup/component behavior的权威地位；兼容资产在compile migration阶段转换为typed descriptor，runtime不保留alias shim。
7. 现有fixed-height culling API改名为visible-range culling，直到真实data-source virtualization接管；不能继续公开“only materializes visible window”的错误表述。
8. `WorldSpaceSurface`在产品接线前从runtime capability中报告unsupported；完成后一次接入scene/camera/render/input/a11y，不保留descriptor-only伪实现。

## 9. 测试先行重构里程碑

### M0：冻结当前source与失败反例

- 原owner先把`runtime_ui.rs`纳入受控change并解决6个dynamic bridge文件的提交边界。
- RED：Button event到gameplay、clipboard、IME host request、timer expiry、focus-loss capture cleanup、DPI raster scale、huge hit bounds拒绝、cycle tree拒绝。
- 记录566-file fingerprints、features、binary hash和dirty overlap；任何变更先recheck。

### M1：唯一 `UiRuntimeService` 与schedule

- 让module driver拥有surface registry、clock、window queues和teardown。
- dynamic session/Editor test manager迁移到同一service；`UiConfig.enabled`真正gate创建和输入。
- 验收session/world/window/plugin unload无stale thread/closure/capture/IME/popup。

### M2：完整dispatch receipt和host bridge

- Product ABI携带component/binding/host/effect结果和handled状态。
- app执行typed clipboard/IME/cursor/pointer/popup/link request并返回结果。
- gameplay command/model binding完成一个Button、Slider、TextInput、List selection端到端闭环。

### M3：Window/Input/Multi-user lifecycle

- 统一Winit/runtime ABI adapter；补extra buttons、touch/tablet数据和identity。
- resize/scale geometry barrier、focus/application lifecycle、real timestamp/delta、timer tick。
- per-user focus/capture/navigation和multi-window surface target。

### M4：Tree arena与原子mutation

- private generational graph、bulk builder、validate-then-commit、iterative traversal。
- O(N) bulk paint order、cycle/depth/node/bytes budget、deserialize migration。
- 移除产品直接mutate与surface serde。

### M5：Persistent layout与准确publication

- surface-owned persistent Taffy graph、canonical style mapping、measure context cache。
- changed frontier局部layout/arranged/hit/render，report区分full/patch/reuse。
- stable generation visits/build/clone/alloc为0。

### M6：真实virtual collection

- data source、item key、variable extent cache、overscan、pool budget/eviction、anchor correction。
- focus/a11y呈现logical collection，未物化item可按请求scroll/realize。
- 100k/1M item只保留visible + overscan instance。

### M7：Focus/navigation/popup收敛

- compiled focus graph、boundary/group/restore contract、modal stack和cross-surface arbiter。
- typed popup portal、placement/dismiss/native window bridge。
- 删除MUI组件名/属性alias热路径。

### M8：Accessibility OS桥

- 扩展neutral schema，发布indexed tree/delta。
- Windows/Linux/macOS目标按仓库支持矩阵接入AccessKit或平台adapter；window activation/action/focus/shutdown完整。
- NVDA/VoiceOver/Orca产品脚本和keyboard-only验收。

### M9：Multi-target与world-space UI

- window/camera/offscreen/world-space target统一surface identity但拥有独立metrics/render/input adapter。
- world transform、pixels-per-meter、billboard、depth/occlusion、ray hit和camera change接线。
- split-screen/XR后续只扩展target adapter，不复制UI tree系统。

### M10：Generation publication与性能收口

- 与PERF-MVP-254/261/262/263/265/274/278/573等既有owner合并实现。
- multi-surface发布segment handle，不flatten command或重写全部ID。
- bounded subscriber、diagnostic gate、allocation/visit/cost counters。

### M11：产品验收与旧路径删除

- 删除test-only manager依赖、bool dispatch、hardcoded HUD/menu、reflection mirror write、descriptor-only world-space capability和字符串fallback owner。
- clean checkout构建App/Editor/Play/package，current-source artifact绑定fingerprint。
- WOC级action bar、inventory、chat、popup、drag/drop、touch和accessibility product test通过。

## 10. 验证矩阵

| 维度 | 必须覆盖 | 通过条件 |
|---|---|---|
| Tree property/fuzz | cycle、duplicate parent/root、dangling slot、deep chain、NaN/Inf/huge frame、serde旧版本 | typed reject，无panic/OOM/stack overflow，事务失败零部分提交 |
| Input | mouse 5+buttons、touch multi-pointer、tablet、keyboard/IME/gamepad、multi-user | identity不丢；route/capture/focus顺序稳定；完整receipt可见 |
| Window | resize、DPI、move、occlude、deactivate、close/destroy、多窗口 | geometry barrier正确，host owner成对释放，无stale surface |
| Binding/host | button/slider/text/list、clipboard、link、popup、pointer lock、host拒绝 | gameplay和UI状态一致，失败可回执/诊断，无silent drop |
| Layout | nested flex/grid/wrap/scroll、min/max/percent/baseline、mixed content | 与canonical oracle逐像素/几何对拍；stable generation零重算 |
| Virtualization | 1/100k/1M fixed/variable items、insert/remove、jump、focus/a11y | materialized <= visible+overscan+bounded pool；anchor稳定 |
| Accessibility | roles/states/relations/live/text/range、dynamic updates、actions | OS工具可见；single change发delta；focus/action回投live surface |
| Multi-surface | overlay/modal/popup、不同scale、independent target、capture | render/input/a11y使用同一composite generation和明确order |
| Fault | asset missing/corrupt、hot reload failure、plugin unload、slow subscriber、host timeout | last-good/rollback或typed failure；队列/内存有界 |
| Performance | 1/1k/10k/100k nodes、1M events、600-event产品样本 | stable build/visit/clone/alloc=0；leaf change随affected frontier；无总节点full scan |
| Product | clean App、Editor、Play、packaged game、screen reader、IME/DPI | artifact带source/binary/feature/OS metadata；无专用fixture旁路 |

## 11. 既有计划需要纠正或重开

1. `09-ui-subsystem-architecture.md` 的M2.1“单Taffy入口”结构结论可保留，但不能作为persistent Taffy/layout cache完成；按本文M5重开行为与规模验收。
2. M2.2和测试名`virtualized_list_only_materializes_visible_window`需要纠正。当前只计算visible range并隐藏已物化child，不满足“只物化”；按本文M6重开并先改名避免false claim。
3. `2026-08-07-runtime-ui-incremental-refresh.md` frontmatter标记`completed`与正文M4-M7 pending/当前full-subtree事实不一致。历史里程碑记录保留，但总体状态应恢复`in_progress`，直到publication/scale/product gates完成。
4. `2026-07-17-woc-project-runtime-ui-bridge.md` 的“无project surface”已被当前未跟踪`RuntimeUiSurfaceSet`部分改变；该failure不能直接关闭，因为action/binding/host/tick/window/hot-reload/scale验收仍失败。
5. `2026-07-19-dynamic-ui-extract-generation.md` 仍open且正确。当前render每次flatten commands并重写node ID，没有generation-ownedmulti-surface segment。
6. accessibility文档中AccessKit converter的unit pass只能证明DTO映射；app host/screen-reader milestone未完成，不能写成产品accessibility bridge完成。
7. performance reassessment中的PERF-MVP编号继续拥有clone/index/publication性能实现；11A拥有UI product lifecycle、identity、dispatch、tree safety和OS accessibility合同，禁止重复authority。

## 12. Owner 边界

- 11A：module/runtime service、tree/layout/input/focus/popup/a11y neutral/host lifecycle。
- 11B：font database、fallback、shaping、BiDi、line break、caret/selection/index、rich text、IME editing semantics。
- 11C：UI render command到GPU、atlas/cache、SDF/vector/image、clip/mask、batch、damage、surface composition。
- `zircon_runtime_interface`后续报告：dynamic ABI version、buffer ownership、opaque handle和host request envelope；11A定义语义，不复制ABI细节。
- `zircon_app`后续报告：Winit/application host执行与window/screen-reader设备接入；runtime仍拥有UI状态机。
- `zircon_editor`后续报告：authoring transaction、Workbench多窗口、designer/preview；不能再创建第二套runtime UI truth。

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| 11A-R0 | Runtime UI architecture/tree/layout/input/accessibility current-source review | review_complete_implementation_pending_source_recheck | 2026-08-16 | 566 production architecture files / 88,376 lines inventoried and fingerprinted；6-file product bridge traced；Bevy/Fyrox/Unreal/Godot owners cross-read；7 P0、31 P1、8 P2；production untouched |
