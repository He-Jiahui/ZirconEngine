---
status: in_progress
review_date: 2026-09-07
plan_sources:
  - docs/plans/astra/optimize/01-review-and-repair.md
  - docs/plans/optimize/zircon_editor/54-editor-workbench-shell-autolayout-constraint-language-responsive-region-binding-geometry-product-integration-review.md
  - docs/plans/optimize/zircon_hub/02-web-shell-catalog-settings-team-cloud-accessibility-performance-review.md
  - docs/plans/optimize/zircon_hub/06-current-source-product-control-plane-project-lifecycle-process-delivery-web-host-test-evidence-review.md
---

# Editor 与 Hub 布局及交互修复计划

## 范围

Editor 负责已有 workbench autolayout/geometry/constraint 路径，Hub 负责已有 Web shell 状态和布局。保持现有设计语言，优先修复内容溢出、不可达操作、状态误导和输入问题，不做无证据的视觉重设计。

当前 Editor side width/vertical band、drawer 和 Hub bridge/CSS 已完成关键调用链复核。ED-A1..A4 与 HUB-A1..A4 已有实现和测试源码，状态统一为 `implemented_pending_validation`；以下保留发现时的触发条件用于回归。原生 minimums、分屏树恢复及 Hub 状态版本已落源但仍待原生验收；逐叶独立渲染和完整视觉矩阵继续实施。

本轮接续 ED-A5/A6 的原生 minimum、树恢复和逐叶投影已有修改，剩余重点是持久的每叶 camera/focus/session、对应 render/input 消费者与无变化帧失效检查。Hub 事件版本的旧浏览器 9/9 证据只属于当时快照；整合本地服务后须重新覆盖全部路由、语言、弹窗及真实 Tauri 窗口，不能继承为当前产品验收。

## 验收矩阵

| 范围 | 场景 | 门槛 |
|---|---|---|
| Editor geometry | 正常、窄、短、零尺寸、NaN/Inf/超大有限值 | 所有输出 finite、非负；布局在有效窗口内；band/panel 不错误重叠；splitter 与实际 panel 同源 |
| Editor 交互 | 展开/折叠、drawer、resize、保存恢复 | 有效布局和现有最小内容规则保持一致；布局恢复不引入非法值 |
| Hub Web | 360/768/1280/1920 宽度、长路径/中英文、列表和 modal | 无整页水平溢出；主要操作可见可达；文字不相互遮挡 |
| Hub 状态 | 首次加载、失败、重试、并发响应、切换页面 | 状态和实际操作目标一致；失败状态不能展示虚假的可操作数据 |
| 性能 | 已有 geometry/projection 分配测试与重复 resize | 保留现有 allocation budget；新检查不得引入每帧全量序列化或额外列表重建 |
| 视觉证据 | 浏览器截图与必要的原生 Hub/Editor capture | 标明 fixture/浏览器与真实 Tauri/WGPU 的区别；不能将 mock 当原生产品验收 |

## 当前源码确认项：Editor

四项均经当前生产调用链复核，已有有限预算及 mount-change 修复，状态 `implemented_pending_validation`；归属 Editor54 布局 owner，优先级统一 P1。源码和 helper 回归不替代原生窗口视觉验收。

### ED-A1：持久化最大有限尺寸使面板几何溢出

- `layout/layout_persistence_document.rs` 仅 serde decode，`layout/manager/normalize.rs` 未检查 drawer extent 和 constraint override。`ActivityDrawerLayout::extent` 可从合法 JSON 获得 `f32::MAX`，snapshot 原样转发。
- `callback_dispatch/template_bridge/workbench/drawer_layout.rs:360-389` 与 `autolayout/region/tool_region/presence.rs:49-60` 消费这些值；`geometry/side_width_allocation.rs:16-41` 对两个极大有限值求和成为 Inf，随后 Inf-Inf 可生成 NaN。
- 修复：在最低共享几何分配边界使用可证明有限的预算算法，约束 override/持久化投影的有效范围；有效普通布局保持一致。避免仅修命令入口，因为文件加载绕过该入口。
- 验收：真实序列化/反序列化最大有限 drawer extent 后求布局；所有 frame 有限且落在窗口内。已有命令非有限值拒绝测试保留。

### ED-A2：窄窗口回收宽度后又将超额分配给 document

- `autolayout/geometry/region_frames.rs:166-210` 先缩两侧，再把全部 reduction 加回 document；Runtime 约束 solver 合同允许保留超过容器的硬 minima，所以此算法仍维持超预算总宽。
- 修复：Editor 在最终窗口几何边界按实际残余预算分配 document 与两侧，不改变 Runtime 通用 solver 的 min 语义；separator 也必须纳入实际可用宽度。
- 验收：0/1/7/32/120/319/320/479/480/640/900 宽度、无/单/双侧面板，满足 frame finite/nonnegative、x+width<=host、总宽+separator<=host+0.001。保持正常宽度原有偏好结果。

### ED-A3：极短窗口仍分配固定 chrome 与底部最小高度

- `geometry/vertical_bands.rs:38-40` 内容预算归零后 solver 仍保留固定 chrome；`:123-137` 的底部 compact floor 可在可用高度为 0 时产生正高度。`splitter_frames.rs:45-53` 继续发布窗口外的 splitter。
- 修复：定义不足空间时的有限降级分配，所有 chrome/content/bottom/status 消费同一个有限总预算；只有相邻可见正尺寸 band 才产生有效 splitter，hit geometry 与 paint geometry 一致。
- 验收：高度 0/1、separator 总和前后、chrome 总和前后、120/420；scale fallback/1/1.25/2，drawer hidden/collapsed/pinned。frame 不越界、不相互错误重叠，正常 420/620 高度保留原合同。

### ED-A4：无变化的 drawer projection 标脏全部模板根

- `callback_dispatch/template_bridge/workbench/drawer_layout.rs:181-220` 无条件调用 `mark_roots_layout_dirty`，`:266-273` 遍历所有根；实际 drawer setters 已只在值变化时标脏。
- 修复：相同 mount 尺寸/scale 和相同 drawer 输入时不全根失效；真实 mount-size/scale 变化仍需根布局失效。修改 mount 更新 owner 时与 drawer owner 显式协同，不遗漏 resize。
- 验收：稳定 recompute 的 root/drawer dirty 数为 0；单 drawer 变化只影响必要祖先；1/64/1024 无关根不增加无变化请求工作量。运行 release paired 样本并报告 p95，目标相对旧全根标脏路径 <=70%，保持现有 scratch allocation 上限；未测量前不宣称此比例实现。

Editor correctness 独占 `autolayout/geometry`、必要 constraint admission 和直接回归；drawer projection 与 mounted-layout 作为同一 owner 的后续切片。当前命令非有限值/非法 split ratio 拒绝、shell metrics 规范化、descriptor ID 去重和 template-authority 既有修复均保留。

### ED-A5：原生 minimums 未应用，极短窗口降级顺序不完整

- `native_window.rs` 创建窗口只请求初始尺寸，已计算的 `window_minimums` 未传给原生窗口线程。布局有限化只能避免非法几何，不能保证最低可用窗口。
- owner 在创建、DPI 和 layout tier 改变时应用同源 minimums；仍可收到不足尺寸时按 topbar、hostbar、document、bottom、status 顺序消费有限预算，仅相邻正尺寸 band 发布 separator。
- 实施细化于 `02-native-minimums-and-layout-restore.md`；覆盖短窗口、各 DPI、布局 tier 切换与真实 WGPU 宿主，不把纯几何测试当原生属性生效。

### ED-A6：分屏树被扁平化为单 pane，恢复比例不完整

- workbench `collect.rs` 及 shell content/pane/scene projection 使用全局 active selection；preset 重建为 0.5 split，不能表达持久化树的每个叶内容和比例。
- layout owner 精确保留树、节点 ID 与 ratio，并按叶发布 document content、焦点、toolbar 和 scene viewport；持久化归一化走共享 load owner，保持有效历史格式并明确迁移非法值。
- 首批 minimums/restore 修复不关闭 per-leaf 产品缺口。后续单独 owner 迁移 projection 消费者，验证双/多分屏独立文档、焦点/工具栏、scene render/input、保存重开比例及短窗口/DPI；稳定帧保留增量/no-op 合同。

## 当前源码确认项：Hub

以下四项在 2026-09-05 已复核生产调用链，已有修复，状态为 `implemented_pending_validation`。对应 Hub02/06 的控制面、交互与布局缺口，必须在当前 source snapshot 上重新核验完整矩阵。

### HUB-A1：后端失败后示例数据仍可发出真实操作（P1）

- 证据：`zircon_hub/web/src/tauri/hubApi.ts:14-24` 将真实 `hub_state` 或解析失败转成 `fallbackShellState`，而同文件 `dispatchHubAction` 仍调用后端；`App.tsx` 初始也直接显示 fallback。
- 影响：用户看到的项目/任务与真实操作对象可能不一致。`TopBar.tsx:127-139` 在窄窗口隐藏演示标记，进一步遮蔽状态。
- 修复：明确启动、就绪、后端不可用和协议错误状态；真实 Tauri 未取得有效状态时只显示失败/重试入口并拒绝业务 dispatch。浏览器预览保留明确可见的示例标识。
- 验收：模拟 invoke 拒绝、非法 payload、重试成功；失败期间业务 invoke 数为 0，初始加载无可操作示例闪现。浏览器 fixture 结果与真实 Tauri 验收分别记录。

### HUB-A2：设置的已发出草稿更新可晚于保存落地（P1）

- 证据：`pages/SettingsPage.tsx:49-52` 发出更新后不等待；`84-90` 仅取消未触发的 timer。`App.tsx:102-114` 的响应 generation 只防 UI 陈旧响应，无法撤回后端已经执行的旧更新。
- 修复：在设置 owner 中串行并合并 draft 更新；Save/Browse/Discard/Defaults 是显式屏障，等待先前更新，且新草稿不得越过相应操作。错误必须被现有反馈路径观察。
- 验收：deferred promise 阻塞一个 draft 后点击保存/丢弃，断言严格后端顺序、最终配置正确、无提交后的旧更新。100 次连续键入在 debounce 窗口内至多提交一次草稿；不为每次按键创建队列副本。

### HUB-A3：目录切换与空搜索保留错误选择（P1）

- 证据：`pages/CatalogPage.tsx:46-50` 保留 query/tab/selection；`HubWindow.tsx:25-39` 对 Assets/Plugins/Learn 复用同一个组件实例。`CatalogPage.tsx:66-68` 在 filtered rows 为空时选中未过滤的第一行。
- 修复：按目录路由隔离组件状态；选择只能来自当前 visible rows，空列表呈现真实空详情。
- 验收：Learn guide -> Assets 得到有效默认筛选；无结果搜索没有隐藏条目的详情/操作；原有效选择和路由行为保持。

### HUB-A4：紧凑导航占位宽度与实际宽度不一致（P1）

- 证据：`components/shell/NavigationDrawer.tsx:48,60-75` 的 Drawer root 使用展开宽度，<=980 的 media rule 只收窄 paper，可能留下约 144px 空占位。隐藏文字的按钮缺少独立 `aria-label`/`aria-current`，折叠命令也与可见状态不一致。
- 修复：用同一 effective compact 状态驱动 root、paper、label 与切换控制；为 icon-only 导航保留可访问名称、当前页状态和 tooltip。
- 验收：800px 下 root/paper 宽度一致、无空白侧沟；360/768/1280/1920 及长标题检查溢出，键盘可达所有导航，当前页面可被辅助技术识别。

Hub 独占源码范围为上述 App/bridge、Settings、Catalog、HubWindow、NavigationDrawer/TopBar 及其直接测试。依赖安装复用现有 lockfile，不更新包版本；typecheck/build 与浏览器回归按四项完成后统一执行。

### HUB-A5：旧事件可覆盖较新的操作响应

- 后端 under-lock snapshot 之后在锁外 emit，响应与事件可乱序；仅前端请求 generation 不能判断后端状态的新旧。
- 状态 owner 在同一锁中发布 backendEpoch/stateRevision，revision 使用可跨 JS 精度边界无损的 wire 表示。bootstrap 建立 epoch；response/event 共用排序准入，旧 epoch 事件不能重置当前状态。
- 实施归 `../features/hub/01-state-publication-order.md`；覆盖并发命令、晚事件、重复版本、重启 epoch、失败/重试与大整数版本。

## 完整视觉与交互覆盖

Hub 每个路由、语言和弹窗均在 360/768/1280/1920 验证长路径、空列表、失败、加载和可操作状态；检查可见文本父容器、水平 overflow、焦点/键盘可达及实际操作目标。浏览器 fixture 可验证响应式边界，真实 Tauri 另验后端事件、原生窗口尺寸和桌面生命周期。

账号团队管理的成员编辑、所有权移交、创建及撤销邀请弹窗由 [Hub 团队管理桌面工作流](../features/hub/03-team-administration.md) 记录本批交互与双语言截图证据；该记录只接受浏览器 fixture 范围，真实 Windows Tauri 与身份服务验收仍归对应服务计划。

Editor 需真实 WGPU 窗口检查短高度、DPI、侧栏/drawer、splitter、多个 document pane 与保存恢复；截图、像素和输入证据关联 source fingerprint、adapter 和 workload。只有可见正确性及现有分配/性能门槛实际通过，才把对应布局 finding 升为 accepted。

## 实施与批量测试

M0：两个 owner 分别提交当前源码 finding 和最小修改范围，写入本计划后才实施。

M1：Editor 几何与 Hub Web 可并行修复；按用户可见行为补回归。

M2：Editor 与其他 Rust 修复合并提交异步验证；Hub typecheck/build/browser 在其改动批次统一运行。编译期间进行源码复审和其他功能修复。

M3：复核截图与布局边界、交互流程、实际性能证据，失败仅回修对应 owner。

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
