---
related_code:
  - zircon_runtime/src/ui/surface
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

Milestone: M1

Status: completed

Files: ["docs/plans/zircon_runtime/runtime/09/2026-08-07-runtime-ui-incremental-refresh.md", "zircon_runtime/src/ui/surface/surface/rebuild.rs", "zircon_runtime/src/ui/tests/pipeline_report.rs", "zircon_runtime/src/ui/tests/surface_dirty_domains/incremental_layout.rs", "zircon_runtime/tests/runtime_ui_incremental_refresh_contract.rs", "zircon_runtime_interface/src/ui/pipeline/stage_counters.rs", "zircon_runtime_interface/src/ui/surface/diagnostics.rs"]

> 协调器提交节点从 M1 编号；本文件的 coordinator `M1` manifest 仅对应下文逻辑里程碑 M0，不改变后续 M1-M7 的产品执行顺序。

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
