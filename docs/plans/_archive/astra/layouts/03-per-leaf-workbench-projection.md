---
status: in_progress
review_date: 2026-09-26
parent_plan: astra-full-domain-20260905
plan_sources:
  - docs/plans/astra/layouts/01-responsive-layout.md
  - docs/plans/astra/layouts/02-native-minimums-and-layout-restore.md
  - docs/plans/optimize/zircon_editor/193-editor-scene-viewport-layout-split-view-orthographic-quadrant-maximize-link-sync-slot-focus-toolbar-persistence-performance-product-integration-current-source-review.md
---

# Editor 每 leaf 工作台投影与精确恢复

## 范围

本计划完成 Editor workbench split 的产品闭环：递归布局树中的每个 leaf 保留稳定 node identity，拥有独立的 tab/content selection、focus、toolbar/chrome、render viewport 和 pointer/keyboard routing；同一布局树在 paint、hit-test、input dispatch 和 retained no-op frame 中使用同一组 leaf frames；保存与重新打开保留完整 axis/ratio/leaf order/tab assignment/active tab/node IDs。已有 native minimum 与短高度 priority 逻辑继续作为输入，不在本计划中重写。Hub 不在本计划内。

禁止通过把递归树拍平成一个 tab 列表、复制固定 0.5 split、或把模型字段写成未消费的 DTO 来宣称完成。单 leaf 默认布局必须与当前行为保持兼容。

## 里程碑

### M1：稳定递归身份与会话状态

- 为 split node 与 leaf 引入持久化 node ID；旧快照从确定性遍历生成 ID，随后保存值优先。
- 让 `DocumentTabModel`/workspace snapshot 携带 leaf identity、path 和独立 active tab/content selection。
- 为每 leaf 保存 focus target、toolbar state、render viewport/camera slot 与 input capture owner；切换 leaf 不泄漏上一个 leaf 的状态。

### M2：递归 geometry 与 per-leaf projection

- 从同一递归 `DocumentNode` 计算所有 leaf、tab strip、toolbar、document viewport、splitter 和 drop zones。
- 通过 `HostWindowSurfaceData`/scene data 发布 leaf keyed frames；paint 与 retained scene 均逐 leaf 消费，空 leaf/无变化 leaf 保持稳定 no-op。
- separator 只由正尺寸相邻 leaf 生成；undersized window 使用 02 计划的已清洗 band metrics，不改变树拓扑。

### M3：focus、toolbar、render 与 input 路由

- shell selection、pointer hit-test、keyboard focus、toolbar actions 和 viewport input 以 leaf node ID 为路由 key。
- 同一 frame 的 hover/capture/command target 只能命中一个 leaf；拖拽 splitter、tab、dock 和 maximize 维护该 key。
- 每 leaf 的 render viewport 与 camera/editor mode 更新只触发自身 projection；未变化 leaf 不产生额外 paint/scene mutation。

### M4：精确 persistence 与恢复

- shared load boundary 校验 node IDs 唯一、leaf assignment 有效、ratio/axis 合法；只修复非法字段并保留合法 exact topology。
- 保存/加载 round-trip 覆盖嵌套 split、非 0.5 ratios、leaf order、tab order、active tabs 与每 leaf session state。
- 旧版本无 IDs 或重复 IDs 使用稳定迁移规则，不能静默合并 leaf 或丢弃内容。

### M5：验证与 Windows 验收

- 添加 focused production tests：递归 projection frame count/key uniqueness、per-leaf focus/input/toolbar isolation、unchanged no-op frame、nested save/reopen equality 与 legacy migration。
- 运行 source-format/diff checks；Cargo 与真实 Windows full-domain window/browser acceptance 由受管验证批次执行。
- 产品验收需在实际窗口中创建两个以上 leaf，分别切换 tabs/focus/viewport/input，拖动 splitter，保存后重开并逐项比较 node IDs、ratios、frames 与 active content。

## 依赖与边界

依赖 02 计划提供的 native minimum、undersized vertical priority 与 ratio normalization。依赖现有 snapshot/tree/preset contracts，不改变 Hub contracts、runtime renderer、资产后端或网络服务。当前 retained host 已按可见 Scene surface 分配 runtime viewport 并提交 per-view render extract；剩余边界在 Editor 的 slot/session currentness、命令/toolbar/input 路由与持久化恢复，不得用静态占位 viewport 冒充真实渲染。MVP `00` 未完成前，本计划只开展只读审计或不依赖编译的测试设计，不升级产品验收。

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M1 | stable node identity + per-leaf session | in_progress | 2026-09-05 | DocumentNodeId UUID owner + explicit legacy serde migration implemented; per-leaf camera/session pending |
| M2 | recursive geometry/projection | implemented_pending_validation | 2026-09-05 | recursive leaf pane/frame projection, retained scene conversion and per-leaf paint |
| M3 | focus/toolbar/render/input routing | in_progress | 2026-09-05 | leaf-keyed projection and per-surface render submission exist; exact toolbar command target, focus guard and per-leaf chrome remain open |
| M4 | exact persistence/reopen | in_progress | 2026-09-05 | deterministic legacy missing/duplicate ID repair is implemented in source, pending managed validation; preset restore must normalize IDs against the destination layout, and live per-leaf camera/session/focus save and reopen remain open |
| M5 | source checks + Windows acceptance | pending | | |

### 2026-09-05 source batch

- New identity owner: `layout/document_node_id.rs`; leaf wire migration in `layout/document_leaf_layout.rs`; cross-window identity uniqueness at `layout/workbench_layout/deserialize.rs`.
- `document_identity_tests.rs` covers exact nested IDs/ratios/content, legacy migration and duplicate-ID repair; `document_leaves.rs` covers different active content in two leaves; `document_tab_pointer/leaf_receipt_tests.rs` covers leaf-local tab receipts and stable no-op sync.
- No Cargo or Windows runtime execution in this source lane. New modules passed rustfmt checks. Existing-file formatting writes intermittently encounter Windows error 1224 (mapped file); compiler and full runtime acceptance remain mandatory.

### 2026-09-26 current-source audit

- `DocumentNodeId` is persisted on split nodes and leaves, recursive leaf projection exists, and `app/host_lifecycle/render_submission.rs` enumerates visible Scene surfaces, allocates a runtime viewport for each surface, and submits a per-`ViewInstanceId` extract. `SceneViewportSessionRegistry` holds per-view controllers. These source paths establish per-surface submission, not yet a two-Scene Windows product acceptance result.
- Toolbar hit routes carry `surface_key`, but `callback_dispatch/viewport/route_mapping.rs` discards it, reads active settings for cycle controls, and dispatches an unqualified binding. `editor_event_execution/viewport_event.rs` applies commands to the active controller, while `app/viewport/toolbar_pointer/chrome_projection.rs` projects one active chrome state. The unknown-control fallback in `callback_dispatch/shared_pointer/viewport_toolbar.rs` also dispatches without a surface target. ED72-P0-02 remains open until the bound slot/view/session/window/surface and layout/session generation are revalidated at command execution.
- `focus_scene_viewport_surface` returns `false` for both an invalid route and an already-focused Scene; the toolbar click currently ignores that result. Registry `focus` and `session` can fork an unknown view ID from the active controller. Route admission must distinguish validity from focus change and validate the current workspace binding before any fork or mutation. The Scene descriptor is still single-instance, so a second live Scene cannot be opened through the ordinary UI.
- Project workspace schema v1 saves layout, view instances and one focused view, but does not snapshot live per-leaf camera/settings/session state; Scene viewport focus does not update the saved workbench focus. Reopen therefore has no per-leaf camera/session restore path. The M4 source follow-up replaces fresh UUID repair of missing or duplicate node IDs with stable traversal IDs while reserving all saved IDs across activity and floating windows; repeat-load and collision regressions are present, with managed validation still pending.
- `LayoutPresetPersistenceStore::restore_into_layout` applies saved exact split IDs to an existing layout without a shared identity normalization pass. Restoring a preset captured from a different layout can duplicate IDs across activity and floating windows before the next normalization. This restore path needs its own owned repair and immediate-save regression before M4 closes.
- Remaining ED-A6 acceptance: exact per-leaf toolbar/input/chrome isolation, persistent camera and focus state, complete template hit-index and scoped content patch consumers, no-op generation checks, and managed real Windows split/save/destroy/reopen evidence. Source inspection and existing unit tests do not close these product gates.


### 2026-10-03 Editor r8 精确源码续接（执行门开放）

Root 于 `2026-10-03T04:44:05.562509+00:00` 将原 Editor 实施线的42个变更路径按当前原始字节落源，
53路径上下文严格重建为零差异；publication `3ccda793de54d5c203c5934eb16984adc62409473d3021a189ffd13804247fb8`，
独立复审 `81edf99ba10f10663639d19d505966d86e5b06ec534cac955c1cb32159eee1e7`。
这次源码覆盖正常导航命令目标、Scene routing、启动呈现和文档事务消费者：

- 已准入的 Scene 激活失败保留原 close owner，正常 begin/commit/finalize 清理项目后可重试；
  Welcome 在项目 open 成功后才 dismiss，后续失败恢复原 manager/retained projection。
- 序列化 reopen 测试从正常 ProjectManager 创建、保存、关闭、打开项目，比较两个 Scene
  identity、focus、viewport session 及首次 retained presentation；它不是已执行的产品证据。
- durable workspace/scene 事务使用正常项目目录与 journal owner 校验；Windows reparse
  fixture 的 typed Unsupported 仍代表测试环境门未通过，不能作为成功提前返回。
- Asset owner 的 ProjectManager mod/open 和 Editor document_roundtrip 三路径仅为只读上下文；
  Core DynamicSession 三路径仍由唯一组合线负责。Native DynamicSession undo/redo 的 ignored
  fixture 未执行，不能替代普通提交、tick、poll、harvest 和 Editor undo/redo 验收。

源码 apply-check、42个postimage哈希及 scoped diff-check 均通过，shared index 保留。
测试数为0，编译、M3/M4实测、Windows短窗口/DPI/逐叶 toolbar/input/chrome、保存重开、
性能、Jenkins和原 build-editor/现场 Penpot 门全部开放。历史foreign Editor36个诊断仍保留：
缺少该失败代与此候选的完整源码同一性，未据此宣称当前编译失败或通过。
