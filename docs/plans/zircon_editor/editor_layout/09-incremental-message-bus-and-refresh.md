---
related_code:
  - zircon_editor/src/core/editor_message/mod.rs
  - zircon_editor/src/core/editor_message/bus.rs
  - zircon_editor/src/core/editor_message/message/mod.rs
  - zircon_editor/src/core/editor_message/refresh_report.rs
  - zircon_editor/src/core/editor_message/topic.rs
  - zircon_editor/src/core/editor_message/view_dirty_set.rs
  - zircon_editor/src/core/editor_message/subscriber.rs
  - zircon_editor/src/tests/editor_message/bus/mod.rs
  - zircon_editor/src/tests/editor_message/refresh.rs
  - zircon_editor/src/core/editor_event/service/state.rs
  - zircon_editor/src/ui/host/editor_event_dispatch.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/invalidation_bridge/dirty_flags.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/invalidation_bridge/dirty_marking.rs
  - zircon_editor/src/ui/retained_host/app/invalidation/mask/requirements.rs
  - zircon_editor/src/ui/retained_host/app/invalidation/mask/summary.rs
  - zircon_editor/src/ui/retained_host/app/assets/refresh/apply.rs
  - zircon_editor/src/core/editor_event/mod.rs
design_references:
  - docs/ui-and-layout/ai-workbench-style/prototype/README.md
plan_sources:
  - docs/plans/zircon_editor/editor_layout/08-plugin-page-interface-and-messaging.md
  - docs/plans/zircon_editor/editor_ui/index.md
status: in_progress-09s2-focused-passed-partial-snapshot-backend
---
# 09 增量消息总线与刷新协议(避免全局卡顿)

## 1. 目标

设计编辑器内的**多协议事件通信 + 增量刷新机制**:类似网页的消息设计(发布/订阅、主题路由),但**严格增量**——只刷新真正变脏的视图/区域,绝不因任一变更触发全局重算/重绘。目标是编辑器复杂(多窗口、多页签、多插件页面)时仍无全局卡顿。本计划是 07/08 的承载基座:页签合并、抽屉流转、插件页面消息全都走这套总线,且全部增量。

## 2. 现状(按代码核实)

### 2.1 已存在的设施(增量地基已部分成立,不重做)

| 能力 | 落点 | 证据 |
| --- | --- | --- |
| 位掩码失效模型 | `invalidation/mask/requirements.rs` | `HostInvalidationMask::{LAYOUT,TREE_STRUCTURE,WINDOW_METRICS,PRESENTATION_DATA,RENDER,HIT_TEST,PAINT_ONLY,POINTER_HOVER,VIEWPORT_IMAGE}` 分级 |
| 脏标记分流 | `dirty_marking.rs` | `mark_layout_dirty`/`mark_presentation_dirty`/`mark_render_and_presentation_dirty` 按需置位 |
| 失效→重算门控 | `requirements.rs` | `requires_layout/presentation/render/hit_test/host_recompute` 决定重算面 |
| paint-only 快路径 | `assets/refresh/apply.rs` | `record_paint_only_invalidation` 仅重绘不重布局 |
| 失效诊断 | `invalidation/mask/summary.rs` | 可观测脏区计数(UiPerfCounter::Dirty*) |
| 编辑器事件运行时 | `core/editor_event` | `EditorEventRuntime::refresh_reflection` |

### 2.2 真实缺口与隐患

- `refresh_reflection_locked` 当前是**全量重建** chrome/view_model/snapshot 并整体 publish——这是全局卡顿源头,需改为按主题/按视图增量。
- 缺**主题路由的发布/订阅总线**(网页式消息),目前是宿主直调反射刷新。
- 缺**视图级脏区粒度**:现有掩码是 host 级,缺"哪个 window/页签/抽屉脏了"的细粒度。
- 缺多协议(请求-响应 / 发布-订阅 / 广播)的统一抽象。

## 3. 设计

### 3.1 增量原则(硬约束)

1. **不存在全量刷新入口**:任何变更产生**最小脏集**(视图 id + 失效面 mask),只重算/重绘脏集。
2. **主题粒度订阅**:订阅者声明主题(如 `selection.changed`、`asset.<id>.changed`、`view.<id>.invalidated`),只在主题命中时被唤醒。
3. **批合并 + 帧末刷新**:同帧多次脏标记按视图合并掩码,帧末一次性按脏集刷新(不逐消息重绘)。
4. **复用既有位掩码语义**:视图级脏先在 core 层使用同构的 `EditorViewInvalidationMask`,09.S2 再桥接到 retained-host 的 `HostInvalidationMask`;paint-only/hover 语义保持快路径。

### 3.2 多协议消息

| 协议 | 语义 | 用途 |
| --- | --- | --- |
| 发布/订阅(pub-sub) | 主题广播给订阅者 | 选择变更、资产变更、视图失效——增量刷新主入口 |
| 请求/响应(req-rep) | 点对点带返回 | 页面向编辑器查状态、请求布局动作 |
| 广播(broadcast) | 全订阅者通知 | 全局态(主题切换、布局预设切换) |

三协议共享同一**主题命名空间**与同一**脏集合并器**。

### 3.3 增量刷新管线(替换全量 refresh_reflection)

```
变更源(用户/插件页面/资产 watcher)
   → 发布主题消息(带受影响视图 id)
   → 总线路由到订阅者 + 标记视图级脏(mask)
   → 帧末:收集脏集 → 按视图增量重建其反射片段(非整体 view_model)
   → 仅 publish 脏视图的 snapshot 增量(非整快照)
```

`refresh_reflection_locked` 拆成:`refresh_view(view_id, mask)` 增量函数 + 仅在结构性变更(新增/删除视图)时才走更宽重建。

### 3.4 视图级脏区粒度

引入 `ViewDirtySet: BTreeMap<ViewInstanceId, EditorViewInvalidationMask>`,把 host 级掩码语义下沉到视图级;window registry(07)的每个页签/抽屉/浮窗都是独立脏区单元,互不牵连——这是"编辑器复杂也不卡"的关键。

09.S1 不让 core 反向依赖 retained-host 私有 `HostInvalidationMask`;core 侧 `EditorViewInvalidationMask` 保持同一组位语义,为 09.S2 的 retained-host bridge 留出清晰转换点。

### 3.5 与现有掩码的衔接

视图脏 → 聚合为 host 掩码(仅当跨视图布局影响时);多数变更停在视图级,不触发 host 级 layout/render。paint-only/hover 继续走 `record_paint_only_invalidation` 快路径。

## 4. 接口与数据结构草案(Rust)

```rust
pub struct EditorMessageBus {
    subscribers: BTreeMap<EditorSubscriberId, BTreeSet<EditorTopic>>,
    subscriptions: BTreeMap<EditorTopic, BTreeSet<EditorSubscriberId>>,
    dirty: ViewDirtySet,
}
impl EditorMessageBus {
    pub fn register_subscriber(&mut self, topics: impl IntoIterator<Item = EditorTopic>) -> EditorSubscriberId;
    pub fn publish(&mut self, topic: EditorTopic, msg: EditorMessage) -> EditorMessageDispatchReport;
    pub fn request(&mut self, target: EditorSubscriberId, topic: EditorTopic, msg: EditorMessage, handler: &mut impl EditorRequestHandler) -> Result<EditorMessageResponse, EditorMessageBusError>;
    pub fn broadcast(&mut self, topic: EditorTopic, msg: EditorMessage) -> EditorMessageDispatchReport;
    pub fn mark_view_dirty(&mut self, view: ViewInstanceId, mask: EditorViewInvalidationMask);
    pub fn drain_dirty(&mut self) -> ViewDirtySet;
}
// editor_event 增量刷新
impl EditorEventRuntime {
    pub fn refresh_view(view: ViewInstanceId, mask: HostInvalidationMask);      // 替代全量 refresh_reflection
}
```

## 5. 模块与文件落点

| 动作 | 文件 | 说明 |
| --- | --- | --- |
| 新增 | `zircon_editor/src/core/editor_message/mod.rs` | 主题 + 总线 + 多协议 |
| 新增 | `zircon_editor/src/core/editor_message/bus.rs` | pub-sub / req-rep / broadcast 路由与 delivery inbox |
| 新增 | `zircon_editor/src/core/editor_message/topic.rs` | 结构化 topic 校验 |
| 新增 | `zircon_editor/src/core/editor_message/message/mod.rs` | message / request / response / delivery DTO |
| 新增 | `zircon_editor/src/core/editor_message/subscriber.rs` | subscriber id |
| 新增 | `zircon_editor/src/core/editor_message/view_dirty_set.rs` | 视图级脏集 |
| 修改 | `editor_event_runtime_reflection.rs` | 拆全量为 `refresh_view` 增量 |
| 修改 | `core/editor_event/mod.rs` | 接入总线 + 帧末 drain 脏集 |

## 6. 里程碑切片化

| # | 切片 | 涉及文件 | 验证命令 | 硬切换 |
| -- | --- | --- | --- | --- |
| S1 | 主题总线 + pub-sub/req-rep + 视图脏集 | editor_message/mod.rs / view_dirty_set.rs | `cargo test -p zircon_editor --lib --locked` | — |
| S2 | 增量 refresh_view 替换全量重建 | editor_event_runtime_reflection.rs / editor_event | `cargo test -p zircon_editor --lib --locked` | 删除全量 refresh 调用点(改增量) |

## 7. 测试矩阵

- 发布一主题只唤醒订阅该主题的视图,其它视图无脏。
- 同帧多次脏标记按视图合并,帧末只刷脏集。
- 单视图变更不触发 host 级 layout/render(除非跨视图)。
- paint-only/hover 走快路径不重布局。
- 多窗口多页签场景下,改一个页签不影响其它页签脏区(UiPerfCounter::Dirty* 计数可证)。

## 8. 风险与对策

- 风险:增量化引入脏区遗漏(该刷没刷)。对策:保留一个**显式**全量重建命令(仅调试/兜底用,不在常规路径),并用脏区诊断计数回归。
- 风险:主题命名失控。对策:主题用结构化命名空间(domain.event / domain.<id>.event),集中登记。

## 9. 完成定义

编辑器内多协议消息总线就绪;刷新全程增量(视图级脏集 + 帧末批刷);无全量 refresh 常规入口;多窗口多页签下改一处不引发全局重算。

## 10. 边界约束

不改运行时 UI 的 surface 失效模型(沿用既有掩码);不引入全量刷新常规路径;主题路由不点对点硬耦合。

## 11. 参考实现对照(dev/ 源码锚点)

- `dev/bevy/crates/bevy_ecs`(事件/变更检测):增量变更检测样板,取"只处理脏"理念。
- `dev/UnrealEngine/.../SlateCore/Public/FastUpdate`:Slate 增量更新(invalidation panel)参考。
- `dev/theatre/packages/core`:pointer/derivation 增量传播参考。

## 12. 状态与产出记录

| 日期 | 切片 | 状态 | 产出/证据 | 后续项 |
| --- | --- | --- | --- | --- |
| 2026-06-23 | 09.S1 主题总线 + pub-sub/req-rep + 视图脏集 | implemented-focused-passed | 新增 folder-backed `zircon_editor/src/core/editor_message/` owner,包含 `EditorTopic` 校验、`EditorMessageBus` pub-sub/request/broadcast 路由、subscriber inbox、`EditorViewInvalidationMask` 与 `ViewDirtySet` 帧末 drain;`core/mod.rs` 只挂载模块。focused tests 覆盖主题只唤醒匹配订阅者、视图脏集合并、request/response target 校验与 broadcast 全量通知。验证:`cargo test -p zircon_editor --lib editor_message --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-runtime-state-reducer-0623 --message-format short --color never -- --test-threads=1 --nocapture` 4/4 通过。 | 09.S2:把 `refresh_reflection_locked` 拆到 `refresh_view(view_id, mask)`,桥接 `EditorViewInvalidationMask` 到 retained-host `HostInvalidationMask`,删除常规全量 refresh 调用点;随后 07/08 可接入该总线。 |
| 2026-06-23 | 09.S2 refresh_view 增量入口与 dirty drain | implemented-focused-passed-partial-snapshot-backend | `EditorEventRuntimeState` 接入 `EditorMessageBus`;新增 `EditorEventRuntime::refresh_view(...)`、`drain_pending_view_refreshes(...)`、`refresh_workbench_for_effects_locked(...)` 与 `EditorViewRefreshReport`,把事件分发和状态访问路径从直接全量 `refresh_reflection_locked` 改为先标记 `ViewDirtySet` 再帧末 drain。focused test 验证指定 view/mask 被记录并 drain,且当前后端明确使用 full snapshot materialize fallback。验证:`cargo test -p zircon_editor --lib editor_message --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-runtime-state-reducer-0623 --message-format short --color never -- --test-threads=1 --nocapture` 5/5 通过。 | partial snapshot publish API 仍不存在,所以 09.S2 只关闭增量触发/脏集入口,未关闭真正局部 snapshot 发布。后续需在 `EditorUiControlService` / `UiEventManager` 增加 partial tree/diff 发布后,再删除 materialize fallback;07/08 可先依赖 view dirty API,但高频 UI 更新仍需等待 partial publish。 |
| 2026-08-10 | 09.M2 retained-host scoped presentation observability | implemented-static-reviewed-validation-pending | UI Asset Editor 的 `View + PRESENTATION_DATA` fast path 复用单次 pane projection，并对根和匹配 native presenter 应用局部 presentation patch。补充 scoped floating-row visit/clone、native presenter visit、damage、projection-missing 与 presenter-coverage 六项指标；访问量覆盖 predicate probe、native identity collection 和 patch traversal，所有回退保留已经发生的工作。精确 `BTreeSet<MainPageId>` 覆盖不符即完整回退，full/scoped floating 内容几何复用同一 helper。最终独立静态复审 P0/P1/P2=0。性能审计:`09/2026-08-09-retained-host-invalidation-architecture-performance-audit.md`。 | 受管 Windows 编译、运行时 profile 与 24 次 UI Asset 编辑截图仍待执行；profile 输出仅 `E:\zircon-profiles`，截图只能写入 `docs/tests/editor`，不得作为已验收或性能改善结论。 |
| 2026-08-10 | 09.M2 hierarchy WorldStructure fragment retained fast path | implemented-static-reviewed-validation-pending | pure `TREE_STRUCTURE` dirty publication 接通 generation-safe `WorldStructure` watch、SceneInspection Latest consumer、Patch/Reflow bridge 与 filtered authoritative recovery；bridge 成功后才提交 host presentation/pointer cache。selection delta 合并保留 revision 语义，非 SceneInspection Latest 继续共享原 `Arc`。runtime override 与 host-row overlay 均使用 persistent path-copy index，单行累计 patch 不复制历史映射。selection revision gap 现在先修复 selection overlay；仅当快照 revision 与 fragment 一致时重试同一 sparse fragment，并把两步 control id 合并为一次 host patch。较新快照或重试异常仍走显式 authoritative reflow。`EditorHostEventController` 显式拥有/初始化 SceneInspection subscriber 与 WorldSyncPump，`Drop` 注销 subscriber。新的 source guard 与 Rust formatting 已完成，修复后独立静态复审 P0/P1/P2=0；managed evidence 仍待完成，failure 记录保持 open。 | 等 coordinator 调度 source-bound Windows focused/upward Cargo 与性能证据；成功前不将 `09/failure-2026-08-05-retained-hierarchy-dirty-refresh-full-snapshot-fallback.md` 改名为 fixed。截图若需要只能写 `docs/tests/editor`。 |
| 2026-08-10 | 09.M2 Workbench retained projection sparse metadata index | implemented-static / managed-validation-pending | Workbench 长生命周期 source projection 在初始化时预建 control-id→作者 attributes/style-token 索引；单节点语义刷新由原来的完整作者树扫描收敛为按变化节点查询，几何 patch、binding 投影与拓扑异常的 full-rebuild fallback 保持既有语义。运行时 focus 属性在作者元数据合并后重新投影，确保程序聚焦和键盘焦点可见性优先。新增 K=1 hover/focus 与 >1000 节点 resize 对比 full projection 的回归，并记录 metadata lookup profile counter。静态审计:`09/2026-08-09-retained-host-invalidation-architecture-performance-audit.md`。 | 需受管 Windows focused/upward Cargo、真实 profile 与窗口截图后才可宣称性能改善；profile 仅写 `E:\zircon-profiles`，截图仅写 `docs/tests/editor`，不得写入 `target`。 |
| 2026-08-10 | 09.M0 profile capture output discipline | implemented-static-reviewed / source-bound capture pending | `tools/ui-profile-capture.ps1` 仅消费协调器预构建的 profiling 可执行文件，强制绝对且受管的 Windows `CARGO_TARGET_DIR`，不再包含直接 Cargo build 或仓库 `target` 的 profiling/project 回退。profile 输出根只允许 `E:\zircon-profiles` 及其子目录；可执行路径合同覆盖允许根、`target`、同名 sibling 与 `..` 逃逸。每个 session 在启动前写 `source_manifest.json`，绑定 revision、脏树摘要、retained-host/text/batching 源码 SHA-256、editor/runtime 二进制 SHA-256、场景和采样参数；Git revision/status、任一关键源码或二进制指纹缺失均在创建 session 前 fail-closed。trace 数据和临时 profile 工程保留在 `E:\zircon-profiles`；每次验证 PNG 导出至 `docs/tests/editor/profile-captures/<session>`。Pester 合同 10/10 通过并完成二次静态复审。 | 直接 `-SkipBuild` 探针被本机防病毒在脚本执行前阻止，未产生文件；待 coordinator-owned Windows source-bound profile 记录 manifest、计数、WPR/trace 与 docs 下验证图，不得将本静态结果作为性能或视觉验收。 |
