---
related_code:
  - zircon_runtime/src/ui/surface
  - zircon_runtime/src/ui/surface/surface/rebuild.rs
  - zircon_runtime/src/ui/surface/surface/rebuild/incremental.rs
  - zircon_runtime/src/ui/surface/surface/property_transaction.rs
  - zircon_runtime/src/ui/surface/surface/pointer_component_events.rs
  - zircon_runtime/src/ui/surface/surface/pointer_component_events/state_invalidation.rs
  - zircon_runtime/src/ui/surface/surface/pointer_component_events/template_action.rs
  - zircon_runtime/src/ui/layout/pass/incremental.rs
  - zircon_runtime/src/ui/text/measure_cache.rs
  - zircon_runtime_interface/src/ui/pipeline
  - zircon_runtime_interface/src/ui/surface
  - zircon_editor/src/ui/retained_host
  - zircon_editor/src/ui/layouts/views/view_projection.rs
related_failures:
  - failure-2026-07-17-ui-render-command-transient-extraction.md
status: in_progress
---

# Runtime UI 增量刷新与不可变帧计划

Plan: docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md

Milestone: M3

Status: in_progress

Files: ["docs/plans/zircon_runtime/runtime/09/2026-08-07-runtime-ui-incremental-refresh.md", "zircon_editor/src/ui/retained_host/app/asset_reference_pointer/events/motion.rs", "zircon_editor/src/ui/retained_host/app/asset_reference_pointer/target/dispatch.rs", "zircon_editor/src/ui/retained_host/app/asset_reference_pointer/target/state.rs", "zircon_editor/src/ui/retained_host/app/tests/root_pointer_fallbacks.rs", "zircon_editor/src/ui/retained_host/asset_pointer/content/bridge.rs", "zircon_editor/src/ui/retained_host/asset_pointer/reference/bridge.rs", "zircon_editor/src/ui/retained_host/asset_pointer/tree/bridge.rs", "zircon_editor/src/ui/retained_host/host_contract/native_pointer/move_dispatch/clear.rs", "zircon_editor/src/ui/retained_host/host_contract/native_pointer/move_dispatch/entry.rs", "zircon_editor/src/ui/retained_host/host_contract/native_pointer/move_dispatch/pane/entry.rs", "zircon_editor/src/ui/retained_host/host_contract/native_pointer/move_dispatch/pane/entry/target.rs"]

> 协调器提交节点从 M1 编号；coordinator `M1` 已完成下文逻辑 M0，当前 coordinator `M2` 对应逻辑 M1，不改变后续 M1-M7 的产品执行顺序。

## 目标

将编辑器输入到上屏的热路径收敛为：

```text
输入事件
  -> typed editor delta
  -> 帧内合并
  -> dirty node set
  -> 局部 layout / text / hit / render patch
  -> 原子发布 immutable frame generation
```

稳定 generation 不得重建 `UiSurface`、复制整棵 presentation、扫描全部 arranged nodes、重新测量未变化文本、重建完整 hit grid 或重新提取完整 render command stream。结构变化、尺寸变化和资源 generation 变化可以扩大失效范围，但必须由 typed dirty reason 显式决定，不能由 consumer 私建第二权威 cache。

## 参考引擎结论

- Slint：property tracker 记录依赖，dirty region 与 cached rendering 只处理失效元素。
- Unreal Slate：typed invalidation reason、invalidation root 与 shaped text cache 分离布局、绘制和文本成本。
- Bevy UI：Changed/Added/Removed 驱动持久 Taffy node 与文本更新，不按帧重建 UI 树。
- Godot：`queue_redraw` 合并延迟刷新，minimum-size 向上失效，文本内部使用分层 dirty flags。
- Fyrox：measure/arrange cache 与 formatted text buffer 持久存在，布局和视觉失效分域。

共同约束是 retained state + typed invalidation + generation publication；本计划不引入永久全树 clone cache、事件降采样或丢输入来掩盖同步成本。

## 统一观测与预算

M0 起持续记录 layout visited/changed/skipped、arranged/hit-grid/render outer visited、commands rebuilt/reused、text measure/layout hit/miss/shape、frame clone bytes 和各阶段耗时。outer visited 只表示阶段外层遍历项，不把 `UiArrangedTree::get` 线性查找和祖先链探测伪装成精确访问量；这些内部成本先由阶段耗时与规模矩阵暴露，M4/M5 再用索引和局部 patch 消除。稳定帧的 rebuild、outer visit、shape、owned clone 均为 0；最终单节点变化的工作量应与受影响节点/祖先/相交 damage 线性，不与总节点数线性。

产品采样沿用固定 `1672x941` Workbench：预热后两个真实 document controls 间交替 600 次 pointer move，间隔 30 ms。debug wall time `<= 20.5 s`、CPU `<= 0.35 core`、无 `NotResponding`；input-to-damage p95 `<= 8 ms`、damage-to-submit p95 `<= 16.7 ms`。规模合同以 1/100/10,000 nodes 覆盖稳定 no-dirty 与单节点 layout dirty；文本 cache 由专用 fixture 覆盖，interaction 由产品 pointer storm 覆盖。

## 里程碑

### M0 基线、访问量计数与性能合同

- 实现切片：补齐 arranged/hit/render 的 outer traversal 与 text cache hit/miss 计数；让 pipeline/debug report 区分“产物数量”和“外层遍历项数量”，内部 lookup/ancestor probe 不从集合长度推导；固化 1/100/10,000 nodes 的稳定与单节点 layout 变化矩阵，并复用文本 fixture 和产品 pointer storm 入口。
- 测试阶段 `M0-Baseline-Gate`：运行 Runtime UI report/dirty-domain/focused tests、`zircon_runtime` 与 interface 受管 build；记录当前全树 post-layout/hit/render 放大以及文本 cache 行为；复跑产品 600-event 样本三次。
- 验收：三次产品样本 wall/core/p95 波动 `<= 15%`；单节点 layout 变化能够直接显示 layout visited 为局部而 arranged/hit/render outer visited 为全量，阶段耗时暴露内部超线性查找成本；计数器本身不改变产品路径。

### M1 失效事务与 generation

- 实现切片：引入单帧 invalidation transaction，统一结构、layout、text、hit、render、interaction、resource dirty reasons；同值 mutation 不推进 generation；一次提交发布 domain generations 与 changed node set。
- 测试阶段 `M1-Invalidation-Gate`：覆盖 dirty reason 映射、同值 no-op、重入 mutation、跨域升级、旧 generation 拒绝 apply 和 1/100/10,000 node dirty-set 成本。
- 验收：一个 frame 内每个真实 domain generation 最多提交一次；稳定 generation transaction/changed-set build 为 0。

### M1.1 鼠标命中热路径缓存

- 实现切片：在 hit-grid rebuild 时一次性缓存 effective input policy 与 bubble route；正常 `UiSurfaceFrame` 查询直接读取 entry frame、policy 和 route，不再对候选节点及祖先链反复执行 `UiArrangedTree::get` 线性扫描。`UiHitTestResult::top_entry` 让 Editor 直接读取命中 entry 的 control metadata，删除普通 pane/viewport-toolbar 路由的最后一次 arranged lookup；缺少新增字段的历史序列化帧继续回退到 arranged tree。
- 测试阶段 `M1.1-Cached-Hit-Gate`：覆盖新帧在移除 arranged node lookup 数据后仍能命中并产生一致传播路径、旧帧字段缺失时的 Serde 回退、exact/radius 既有行为和 Runtime UI focused tests。
- 验收：新生成 hit grid 的每次 pointer query 及产品 control-id 解析不依赖 arranged node vector 或 ancestor lookup；旧快照行为兼容，命中顺序与传播路径不变。

### M2 Editor typed delta 与帧内合并

- 实现切片：Editor event reflection 输出 typed delta，连续 pointer move 只在不跨 press/release/scroll/focus/geometry barrier 时合并；control request 边界排空 delta；删除空 effects 到全 presentation refresh 的升级。
- 测试阶段 `M2-Editor-Delta-Gate`：覆盖事件顺序、barrier、capture/drag、tree/property delta 合并和 retained/control-request reflection 边界；运行 editor integration contracts 与真实输入样本。
- 验收：same-target move 的 reflection/full projection 为 0；每帧每节点每属性最多保留最终 delta，同时不丢离散输入。

### M3 持久 UiSurface 与增量绑定

- 实现切片：Editor view/workbench 持有长期 `UiSurface` 与 compiled document generation；binding delta 直接修改 node/property，不再为每次 projection 构建 surface、layout 和全量 `ViewTemplateNodeData`；移除临时 base-node clone cache。
- 测试阶段 `M3-Persistent-Surface-Gate`：覆盖模板 reload、size/resource generation、文本 override、节点增删和稳定投影；测量 surface builds、node clones 与 owned bytes。
- 验收：稳定 view update 的 surface build/layout/full node clone 为 0；单文本变化只触及目标 node 与必要祖先。

### M4 增量 arranged tree 与 hit grid patch

- 实现切片：用 layout geometry changed set patch arranged nodes、draw order 与 spatial hit cells；只有 topology/order/clip 大范围变化才升级 scoped/full rebuild；输入 consumer 只读取同代 hit generation。
- 测试阶段 `M4-Layout-Hit-Gate`：覆盖 parent-directed/auto layout、clip、z-order、popup、scroll、detach/attach 与 1/100/10,000 nodes 单节点变化。
- 验收：非 auto parent 下单节点尺寸变化的 arranged/hit visited 与 changed nodes 线性；未变化 hit cells 和 draw-order entries 不重建。

### M5 增量文本与 render extract patch

- 实现切片：按 text/layout/resource generation 保留 shaped/layout buffer；从 dirty/geometry-changed nodes 更新 generation-owned command buckets 与 ordered ranges，不再先全量 extract 再比较 cache；完成 `ui-render-command-transient-extraction` failure 的 typed element range、规模计数与 Runtime/Editor parity。
- 测试阶段 `M5-Text-Render-Gate`：覆盖 text/style/frame/resource 失效、multi-command node、删除/插入、damage union、pixel parity 和 1/100/10,000 commands。
- 验收：单文本变化只 shape/rebuild 目标文本与受影响命令；稳定 extract 的 node visit、element Vec build、payload clone 为 0；failure 完成 upward validation 与 return。

### M6 Arc frame、Workbench 局部同步与虚拟化

- 实现切片：`UiSurfaceFrame` 改为 generation-owned `Arc` artifacts，稳定 read 不复制 arranged/render/hit/focus/report/ECS projection；Workbench/native windows 持 applied generation cursors；长列表只 materialize visible range。
- 测试阶段 `M6-Frame-Sync-Gate`：覆盖旧 frame 生命周期、跨窗口共享、局部 pane sync、virtualized scroll、ECS delta 和 10 分钟 retained-memory 压力。
- 验收：稳定 frame read 的 clone bytes/reprojection 为 0；pane/native OS 更新只随对应 generation；RSS/Private Bytes 无持续增长。

### M7 真实运行时验收与旧路径删除

- 实现切片：删除 event-time full surface/projection/render fallback 与过渡 cache；接入 diagnostics、GPU/Softbuffer submit、bundle 构建和真实窗口采样。
- 测试阶段 `M7-Product-Gate`：执行 Runtime/Editor 受管 build、focused contracts、debug/profiling bundle smoke、三向像素对拍、600-event 三次样本与 10 分钟 hover/scroll/typing/docking 压力。
- 验收：统一预算全部达标；无输入丢失、焦点/拖拽/popup/clip/z-order 回归；独立 EXE 携带完整资产/DLL 并可启动。

## 调试与纠偏

失败时从最低共享层向上检查：dirty transaction -> changed set -> layout/text cache -> arranged/hit patch -> render ranges -> immutable frame -> Editor consumer。不得在 event handler、pane bridge 或 painter 增加无 generation 约束的旁路 cache。每个里程碑只在完整 testing stage、独立 review、协调器提交与量化通知完成后进入下一里程碑。

## Scope Delivered

M0 交付 Runtime UI rebuild/pipeline/debug 的 arranged、hit-grid、render outer traversal 计数和 text measure/layout/shape cache hit/miss 计数；计数命名明确不包含内部线性 lookup 与祖先链 probe。规模合同同时覆盖 no-dirty 零工作和单节点 layout dirty 的全树 outer traversal。父计划补齐协调器可提交节点 M1 -> M2 -> M3 的机器可读依赖边；逻辑 M0 仍按本文件说明映射到 coordinator M1，使未由本基线里程碑解决的 failure 可以保留到严格后继节点。

## Fresh Testing Evidence

受管 `zircon_runtime` integration contract 2/2 通过，ignored 1/100/10,000-child stable + single-node dirty matrix 通过，`zircon_runtime` 与 `zircon_runtime_interface` build 通过；`rustfmt +1.94.1` 与 `git diff --check` 通过。协调器 validation-copy 两次均为 36/36，耗时 69.706 s 与 71.019 s。三轮 600-event 产品样本均产生 601 组有效 damage，wall/input/submit 和所有重复性波动满足预算；CPU 0.392-0.444 core 仍高于 0.35 core，是 M1-M7 的已记录优化缺口。

## Review

r4 最终代码独立复核为 accepted，review `066744fae9c64ce4ac35df432b53e181`，Critical 0、Important 0。审查确认 outer traversal 语义准确、no-dirty 规模矩阵完整、强制 rebuild cache reuse 与稳定帧已区分、text cache stats 生命周期闭合且新增序列化字段均有 `serde(default)`。本 r5 仅补计划记录规范段落和父计划依赖元数据，Rust 交付字节未变化；提交前仍对 r5 manifest 执行独立复核。

## M0 基线证据

- 规模矩阵：稳定 no-dirty 在 2/101/10,001 nodes 下的 rebuild、outer visit 与 text cache/shape 计数均为 0。单节点 layout dirty 均只访问 1 个 layout node，但 arranged/hit/render 外层遍历分别进入 2/101/10,001 个节点；该计数不包含线性的 `UiArrangedTree::get` 与祖先链探测。三轮 10,001-node post-layout 合计耗时约 662.8/672.7/697.0 ms，波动约 5%，耗时增长明确暴露了 outer traversal 内部仍存在的超线性成本。
- 文本观测：pipeline/debug report 已区分 measure/layout/shape cache hit/miss；测试明确区分无脏 `rebuild_dirty()` 的零工作稳定帧与强制 `rebuild()` 的 cache 复用。editable text fixture 覆盖强制重建时的 layout-cache hit 合同。
- 产品基线：当前源码 profiling bundle 在 `1672x941` Workbench 中完成三轮 600-event pointer storm，每轮得到 601 组 input-to-damage/damage-to-submit 样本、601 次 region paint、0 次 `NotResponding`。wall 为 18.005/18.014/18.006 s，CPU 为 0.443460/0.444111/0.392234 core，input-to-damage p95 为 264.0/271.7/252.9 us，damage-to-submit p95 为 11,784.5/11,729.8/11,644.3 us；对应波动约 0.05%/12.2%/7.2%/1.2%。wall 与两段 p95 达预算，CPU 仍高于 0.35 core，是后续里程碑必须消除的基线缺口。
- 刺激修正：文档页签和 viewport toolbar controls 会消费 pointer move，但当前不持续产生 hover damage，不能形成有效 p95 样本；正式基线改用真实 `template.left.SelectRoot` hierarchy control 与同 pane 空白区域间的 enter/leave。无效样本不参与验收，document-control hover invalidation 缺口保留为后续 typed delta/interaction dirty 的产品诊断输入。

## 状态与产出记录

| 里程碑 | 状态 | 完成日期 | 验证与量化证据 |
|---|---|---|---|
| M0 | completed | 2026-08-07 | Runtime integration contract 2/2 通过、1/100/10,000-child stable + single-node dirty ignored 规模矩阵通过，`zircon_runtime`/`zircon_runtime_interface` 受管 build 通过，协调器 validation-copy 36/36 通过；三轮产品样本各 601 个有效 damage，波动均小于 15%。Runtime 全 lib test 仍被 134 个共享工作区错误阻断，interface focused lib test 被 9 个未租赁 IME/source-map 测试编译错误阻断；两组诊断均未命中本 manifest 文件。CPU 基线 0.392-0.444 core 高于 0.35 预算，转入 M1-M7 优化。 |
| M1 | completed | 2026-08-07 | 新增单帧 typed invalidation transaction、7 类 reason、domain generations、changed-node 合并、旧 generation 拒绝和 `UiSurface` mutation/rebuild 提交边界；公开 `UiTree::insert_root/insert_child` 也会产生可归并的 Structure dirty。受管 integration contract 14/14 通过，覆盖同值 no-op、重入 dirty、跨域升级、稳定帧、Serde roundtrip、外部 transaction 原子性、直接 public-tree topology mutation 和真实 1/100/10,000-node 规模。Runtime 全 lib test 被当前共享工作树 135 个无关编译错误阻断；interface 全 test 被 9 个未命中本 manifest 的 IME/source-map 测试错误阻断。最终独立复审 Critical 0 / Important 0 / Minor 0；M1 不声称降低全树 rebuild 成本，changed set 将由后续里程碑消费。 |

| M1.1 | completed | 2026-08-07 | hit-grid entry 在 rebuild 时缓存 effective input policy 与 bubble route，`top_entry` 直接提供 control metadata；fresh `UiSurfaceFrame` 查询在移除 arranged nodes 后仍能命中并生成一致路径，旧 Serde 帧缺字段时回退原路径。受管 `runtime_ui_incremental_refresh_contract` 4 passed / 1 ignored，受管 `zircon_editor` build 通过，`rustfmt` 与 `git diff --check` 通过；独立复审 Critical 0 / Important 0 / Minor 0。普通 pane/viewport-toolbar 查询不再执行候选、祖先或 control-id 的线性 arranged lookup，也未新增第二个 per-query 命中栈分配；完整 route 的每-entry 构建/克隆内存成本保留为后续共享 arena 优化项。 |
| M2 | completed | 2026-08-07 | Editor retained event path 输出 typed node/property delta，并以 press/release/scroll/focus/geometry/commit barrier 切分帧内合并；Runtime 原子校验并按 tree 广播 reflection patch，空 effects 不再升级为 presentation refresh。受管 editor/runtime 公开 integration contracts 通过，其中 `editor_world_sync_watch_map` 默认并发 7/7；最终独立复审 Critical 0 / Important 0 / Minor 0。当前源码 profiling-feature bundle 构建与 smoke 通过。`1672x941` Workbench 的 600-event 样本得到 599 组有效 damage、599 次 region paint、0 次 full paint/presentation rebuild/chrome snapshot/model build，wall 19.593 s、input-to-damage p95 367.6 us、窗口始终 responding；CPU 0.412 core 与 damage-to-submit p95 18.029 ms 仍高于最终预算，转入 M3-M6。 |
| M3 | completed | 2026-08-08 | Editor asset content/reference/tree 的 pointer bridge 改为持久 layout + O(1) 行命中；稳定 hover 不再走通用 `UiSurface` dispatch、focus、Host source-window 解析或状态 clone。引用列表跨 sibling 空白移动时，Host 与 bridge retained hover state 原子清理，并覆盖 `references row0 -> used_by blank -> references row0` 的真实 Host 回归路径。受管 `zircon_editor` production build 通过；独立复审 accepted，Critical 0 / Important 0。lib test 仍在测试 harness 编译前被共享工作树 153 个既有 test-only 错误阻断。profiling EXE `zircon_editor.exe` 与匹配 `zircon_runtime.dll` 通过构建/发布，真实窗口 600 次 `WM_MOUSEMOVE` smoke exit code 0；602 个 `idle_hover` frames 的 frame p95 1.584 ms、max 4.583 ms，presentation/layout/model/chrome full rebuild 均为 0，仅 13 次局部 hover patch。 |
| M4 | completed | 2026-08-09 | layout/style/text changed set 保守 patch arranged frame/clip，混合 input/hit、topology、paint/z-order、visibility/policy、slot identity、clip ancestor 或 tree/index 不一致自动回退全量构建。`UiTreeNodes`、arranged/slot/hit cell 与 layout-engine selection 均有反向索引；responsive MUI 校验 cached slot identity，零面积 pointer 恢复正面积会回退 full hit build。当前源码 integration contract 14 passed / 0 failed / 1 ignored；显式运行 ignored 规模门 1 passed（13.83 s）。2/101/1,001/10,001 nodes 的单节点 layout dirty 均为 layout/arranged/hit/render outer visit `1/1/1/1`，10,001 nodes 样本 layout/arranged/hit/render 为 157/32/76/338 us。 |
| M5 | completed | 2026-08-09 | render cache 持久化 `node -> command range` 与严格 geometry-eligible set。仅 owner-frame/owner-clip、单命令且无 `text_layout` 的节点可原位平移；文本、相对 command geometry、尺寸变化、命令数变化或任何预检失败都事务性转入 changed-node extract/full fallback。当前 14/15 contract 与 10,001-node 规模门均动态通过；真实 render-only style 样本仅访问 1 个 render node，10,001 nodes 为 305 us。 |
| M6 | completed | 2026-08-09 | `UiSurfaceFrame` 发布稳定 `Arc` generation，clean frame 复用同一 allocation，dirty frame 单调推进 generation 且旧 frame 生命周期独立；Workbench pane/viewport/presenter 使用 generation cursor 跳过稳定同步与重复提交。受管 Runtime/Editor production build 通过，frame generation 与 presenter cursor focused contracts 已纳入当前源码。 |
| M7 | in_progress | 2026-08-09 | 产品 mouse/hover gate 已关闭：默认 GPU backend 的真实 Workbench 1000/1000 pointer move，压力阶段 100/100 `Responding`、0 unresponsive，工作集 +4.35 MiB、private +2.59 MiB，进程存活且像素截图有效；10 分钟 hover soak 完成 5,318 moves/531 samples，首分钟 2 次短暂无响应、后 9 分钟 0 次，private 预热 +107.24 MiB 后稳定。双 native surface 升级导致的 DX12 `0x87d`/Vulkan `Invalid surface` 已改为先释放 startup surface；最终 bundle `editor-debug-20260809-010434-43411acd` 可直接创建并打开修复后的 `renderable-empty`（assets=7 ready=7 failed=0），300/300 move、30/30 responding。完整 profiling 三场景、CPU 统一预算与 typing/docking soak 尚未重采，因此不关闭全部 M7。 |

## M1 失效事务交付

- `UiInvalidationTransaction` 以 `UiNodeId` 为键帧内合并重复 mutation；空事务不分配 changed-node 列表、不推进 generation，同节点跨域 mutation 只发布一个 change。
- `Structure`、`Layout`、`Text`、`HitTest`、`Render`、`Interaction`、`Resource` 保留原始 typed reason，同时按实际下游 dirty 域推进 generation；每个域在一次 commit 内最多推进一次。
- `UiSurface::mark_node_dirty`、成功的 `mutate_property` 与 `invalidate_node(reason)` 进入同一 pending transaction；`rebuild_dirty`、`compute_layout` 在产物完成后提交，`rebuild` 遇到 layout 域仍 dirty 时不会提前提交或清除，显式 clear 会取消未消费事务。
- 外部 transaction 携带 base generation，先完整校验 generation 和 node set，再原子合并到 retained tree 与 pending ledger；过期或缺失节点不会留下部分 dirty。
- `UiTree::insert_root/insert_child` 在拓扑 API 内给相关节点合并 Structure dirty，rebuild 前的一次现有 dirty scan 会将 public tree、pseudo-state 后代、slot 和 node-pool 结构变化统一归并进私有 ledger；pending ledger 参与 Serde roundtrip。

## M1 验证证据

- 受管 `runtime_ui_invalidation_transaction_contract` 最终稳定 14/14 通过；测试可执行文件直接复跑 14/14、耗时 1.41 s。
- 合同覆盖 7 类 reason 及 visible-range/input 下游矩阵、同节点跨域合并、空事务、旧 generation、缺失节点原子拒绝、force-rebuild 延迟提交、pending Serde roundtrip、直接 descendant/state dirty、slot/node-pool/public-tree 结构变更，以及真实 1/100/10,000-node surface 下单 changed-node 恒定。
- `zircon_runtime` 常规 library/build 路径在 focused integration target 中编译通过。全 `--lib` 测试树因共享工作区 135 个无关错误无法生成测试二进制；`zircon_runtime_interface` 全 test 另被 9 个既有 IME/source-map 测试编译错误阻断，均未命中本 manifest 文件，也未修改或回退其他会话文件。

## M1 独立复审

- 最终复审为 accepted，Critical 0、Important 0、Minor 0。审查确认 7 类 reason 下游 generations、事务预校验与原子合并、pseudo/state descendant dirty 归并、pending Serde roundtrip、layout generation 延迟提交、slot/node-pool typed reason 和 public `UiTree::insert_root/insert_child` 结构 dirty 均已闭合。
- reviewer 在 Windows 受管 target 复跑聚焦 validator 与测试二进制，结果 14/14 通过，并确认 `git diff --check` 通过；底层 `UiTree` 容器仍公开是剩余封装风险，当前生产代码检索未发现无配套失效标记的直接拓扑修改。

## M2 Editor typed delta 交付

- `UiReflectionNodePatch` 与 `EditorUiDeltaBatch` 将 retained event reflection 变化收束为 node/property/state patch；同一 barrier segment 内按 Runtime 全局 `UiNodePath` 与 property/state latest-wins，`view` 仅保留最后来源元数据，跨 press/release/scroll/focus/geometry/commit barrier 保留顺序。
- `SharedEditorMessageBus::drain_view_updates` 在一个锁边界内排空 dirty set 与 delta batch；retained-host tick 在 recompute 前消费一次，control service 通过 Runtime `apply_reflection_patches` 更新既有 reflection node，不构建整棵 workbench snapshot。
- 同帧存在 full fallback 时先发布 authority snapshot，再重放已排空 delta；纯 patch 遇到 stale snapshot 时重建并重试一次，因此一个 view 的 dirty 不会静默吞掉另一个 view 的合法 patch。
- Runtime patch 先校验全部 tree、node 与 property，再执行 mutation；无效 patch 不留下部分写入，每棵变化 tree 只发布一个 typed `ReflectionDiff`。同值 patch 不广播 diff。
- retained PointerMoved 的空 effects 不再升级为 `PRESENTATION_DATA`；精确 HoverNode、PressNode 与 drawer-resize transient 事件直接产生 patch。无法表达旧节点身份的 focus/drag 以及 layout 变化仍显式保留一次 deferred full fallback，等待后续 typed identity 扩展。

## M2 验证证据

- 受管 `zircon_editor` production build 通过；公开 editor integration target 中 delta latest-wins/dirty atomic drain 2/2、真实 `EditorHostEventController` 1000 次 PointerMoved 合同 1/1 通过。后者确认 dirty set、delta 与 full fallback 均为 0，事件 retention 合并 999 次、丢弃 0 次并保留最终坐标。独立复审追加的跨 view 同路径排序与 dirty+delta 同帧回放合同均已先红后绿；环境型控制器测试共享进程级 config 锁，禁止默认并发执行时互相覆盖 `ZIRCON_CONFIG_PATH`。
- 受管 Runtime 公开 integration contract 1/1 通过，覆盖多节点 patch 每 tree 单 diff，以及第二个无效 patch 使整批原子拒绝。Runtime/Editor 全 lib test tree 分别仍被共享工作树 134/152 个既有 test-only 编译错误阻断，错误未命中本 M2 生产文件；常规 production build 与独立 integration targets 可编译运行。
- 当前源码 `target-editor-host + profiling + profiling-chrome` EXE/DLL 通过协调器受管构建和 `--help` smoke。固定 `1672x941` Workbench 在真实 `template.left.SelectRoot` 与同 left-surface 空白区之间交替 600 次 PointerMove，30 ms 间隔；有效结果为 600 frames、599 input-to-damage、599 region paint、0 full paint、0 presentation rebuild、0 chrome snapshot、0 workbench model build，wall 19.593 s、CPU 0.411508 core、input-to-damage p95 367.6 us、damage-to-submit p95 18,028.8 us，窗口全程 responding 且 exit code 0。profile artifact 位于 `C:/Users/HeJiahui/ZirconBuilds/runtime-ui-m2-profile-20260807-211900/m2-profile-output/runtime-ui-m2-damage-run4-20260807`。
- wall、响应状态与 input-to-damage 达预算；CPU 与 damage-to-submit 尚未达到统一最终预算。M2 只关闭 event-time reflection/full projection 放大，不声称关闭后续 paint/submit 成本；持久 surface、局部 arranged/hit/render patch 与 Arc frame 继续由 M3-M6 负责。
- 打开现有项目的额外采样被共享源码中的 shader IDE stub 缺陷阻断：独立 `zr_bindless_material.wgsl` stub 缺少 `ZR_BINDLESS_MATERIAL_SLOT_CAPACITY` 定义。无项目 Workbench 可正常启动并提供与 M0 相同的真实 retained control/几何，故 M2 interaction 样本有效；shader IDE 问题不在本 M2 manifest，未在本里程碑旁路修改。

## M2 独立复审

- 最终复审为 accepted，Critical 0、Important 0、Minor 0。审查确认 delta 按 Runtime 全局 `UiNodePath` 合并且 property/pressed latest-wins，`view` 只保留最新来源；full fallback 会先重建 authority snapshot 再重放其他 view 的 delta，纯 patch 原子失败只在重建后重试一次，不改变 barrier、dirty report 或单 tree 单 diff 语义。
- reviewer 在 Windows 受管 target 以默认测试并发复跑 `editor_world_sync_watch_map`，结果 7/7 通过，并确认进程级 `ZIRCON_CONFIG_PATH` fixture 由共享 `OnceLock<Mutex<()>>` 串行保护、析构顺序先恢复环境再释放锁；`git diff --check` 通过，未修改文件。

## M3 持久指针投影与局部 hover 交付

- Asset content、reference、used-by、folder-tree pointer bridge 均复用已提交 layout/state；内容列表与缩略图网格使用直接坐标计算，引用与树列表使用 viewport/scroll/row stride 算术命中，间隙、裁剪、禁用条目和边界语义保持与通用 `UiSurface` hit-test 一致。
- passive `PointerMoved` 只在 hovered row 改变时写入状态；相同行、空白区和未命中区域不会 clone `AssetListPointerState`、重建 bridge surface、解析 callback source window 或触发 focus。click/press/scroll 仍显式聚焦并保留原事件语义。
- 引用列表 sibling hover 清理同时作用于 Host state 与对应 bridge retained state；全量 pointer-left 也清理两层状态，避免“离开后重新进入同一行”被错误判定为 no-op。非引用区域不再无条件触发 references/used-by leave callback。

## M3 验证证据

- `validate-matrix.ps1 -Package zircon_editor -SkipTest` 通过；`zircon_app` profiling bundle 使用 `target-editor-host,profiling,profiling-chrome` 构建并发布到 `C:/Users/HeJiahui/ZirconBuilds/runtime-ui-m3-profile-20260808-r3`，EXE SHA256 `3A9A46881DA8082940B912DA7279A94925A6C634964BBF8D220B94EB928900B7`。
- 发布 EXE 搭配匹配 `zircon_runtime.dll` 启动真实窗口，注入 600 次 `WM_MOUSEMOVE`、30 ms 间隔后正常关闭，PID 34804 exit code 0，进程无残留。profile 输出：`C:/Users/HeJiahui/ZirconBuilds/runtime-ui-m3-profile-output/mouse-hover-optimized-20260808-r3`。
- `ui_hotspots.json` 显示 `idle_hover` 602 frames，frame p95 `1.584 ms`、max `4.583 ms`、slow path 0、presentation rebuild 0、dirty layout/render/model/chrome full rebuild 0、region paint 13、chrome patch 13；无 profile alerts。startup 的两次 presentation/full paint 仅属于首次启动，不计入 hover 热路径。
- 新增真实 Host 回归覆盖 sibling 空白跨列表和同一行重新进入；由于 `zircon_editor` lib-test harness 仍在执行测试前被共享工作树 153 个既有 test-only 编译错误阻断，未声称该测试已执行。独立复审确认状态同步和 O(1) 稳定 no-op 路径，无 Critical/Important。

## 2026-08-27 增量重建所有者结构收敛

状态：`runtime_09_15_ui_surface_incremental_rebuild_owner_split_static_passed_cargo_profile_deferred`。

M0-M3 的历史交付与证据保持不变；总体计划恢复为 `in_progress`，直到 M4-M7、publication、
scale 与产品门禁真正关闭。本切片在完整复审当前 `UiSurface` rebuild 路径和 Unreal Slate
invalidation root/widget-list/heap/index 分工后，只把 `rebuild_dirty`、1/4/256 增量降级预算及
layout-engine report patch/merge helper 从 1194 行父文件迁入 711 行 child，父文件降到 500 行。
四个移动项规范化哈希 4/4 与拆分前一致，静态 production-owner guard 1/1 通过。

本次没有改变 dirty frontier、Taffy 生命周期、patch outcome 或性能阈值，故 P1-9/P1-10 与
persistent graph 仍是待 profile 后实施的算法任务。Cargo、真实 App/Editor/Play、CPU/allocation/
RSS/power 画像均未执行，不据此声明增量算法、能耗或整体 UI MVP 已验收。

## 2026-08-27 Surface Property Transaction 所有者收敛

状态：`runtime_09_15_ui_surface_property_transaction_owner_split_static_passed_cargo_profile_deferred`。

按 Unreal Slate attribute descriptor/value-change 与 invalidation reason 分工复审后，surface
property transaction 已从 959 行 `surface.rs` 移入 485 行 `surface/property_transaction.rs`，
父 owner 降到 483 行。transaction 仍以同一 `UiSurface` 原子同步 tree property、component
state、runtime style、focus/popup、editable text、clipboard revision 和 typed invalidation；12 个
移动项规范化哈希 12/12 与拆分前一致，静态 production-owner guard 1/1 通过。

该结构切片没有改变 mutation/dirty/focus/popup/text 算法或 transaction 成本。M4-M7、
property update 的 allocation/latency、真实 UI 产品路径、Cargo 与 power profile 继续开放，不能
据此关闭增量刷新计划或宣称算法最优。

## 2026-08-27 Pointer Component State 所有者收敛

状态：`runtime_09_15_ui_pointer_component_state_owner_split_static_passed_cargo_profile_deferred`。

对照 Unreal `SlateApplication` 输入路由与 `SWidget` hover/invalidation state 分工，pointer
component event root 中的 hover/pressed/focus state、pseudo-style propagation 与 render dirty 已
移入 226 行 `pointer_component_events/state_invalidation.rs`，父 event/binding/action owner 降到
674 行。7 个移动项规范化哈希 7/7 与拆分前一致，静态 production-owner guard 1/1 通过。

该结构切片不改变 ancestor probes、subtree style update、dirty domain、event ordering 或 binding
payload 成本。M4-M7、真实 pointer 产品路径、Cargo 与 CPU/allocation/RSS/power profile 继续开放，
不能据此关闭增量刷新或宣称 pointer 热路径已优化。

## 2026-08-28 Pointer Template Action 所有者收敛

状态：`runtime_09_15_ui_pointer_template_action_owner_split_static_passed_cargo_profile_deferred`。

pointer event root 的 binding/action payload 责任继续按 Unreal pointer routing 与 `FUIAction`
contract 分开：事件 envelope、focus/damage 留在 426 行父 owner，handle lookup、action/route
projection、missing-value policy 与 payload expression/property resolution 移入 262 行
`pointer_component_events/template_action.rs`。9 个移动方法规范化哈希 9/9 一致，未增加第二
dispatch、action registry、tree 或 binding store。

该结构切片没有修改 event ordering、compiled handle 映射、payload 解析或 allocation 算法。
M4-M7、真实 pointer/action 产品路径、Cargo 与 CPU/allocation/RSS/power profile 继续开放，不能
据此关闭增量刷新、宣称 pointer/action 热路径已优化或触发 milestone commit/企微同步。
