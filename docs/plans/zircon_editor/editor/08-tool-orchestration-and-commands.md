---
related_code:
  - zircon_editor/src/core/commands/registry.rs
  - zircon_editor/src/core/commands/registry_handle.rs
  - zircon_editor/src/core/commands/descriptor.rs
  - zircon_editor/src/core/commands/when.rs
  - zircon_editor/src/core/commands/keymap.rs
  - zircon_editor/src/core/commands/menu.rs
  - zircon_editor/src/core/tools/scheduler.rs
  - zircon_editor/src/core/context/tool_scheduler.rs
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/ui/host/module.rs
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/ui/workbench/model/menu/extension_menu.rs
reference_sources:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/EditorModeManager.h
  - dev/Fyrox/editor/src/message.rs
  - dev/godot/editor/editor_node.h
plan_sources:
  - docs/plans/zircon_editor/editor/00-editor-architecture-overview.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/zircon_editor/editor/06-ui-extension-framework.md
status: in_progress
---

# 08 工具管理调度 / 命令系统 / 命令面板

- fixed 已修复：[command-registry-hard-cut-cli](08/fixed-2026-07-12-command-registry-hard-cut-cli.md)
- fixed 已修复：[editor-operation-path-deserialize-validation-bypass](09/fixed-2026-07-15-editor-operation-path-deserialize-validation-bypass.md)
- 跨计划失败交接（`open / Editor04 接管 Building/Play 权威状态投影`）：[`04/failure-2026-07-12-command-eval-play-state-projection.md`](04/failure-2026-07-12-command-eval-play-state-projection.md)
- fixed 已修复：[command-eval-scene-mode-selection-projection](08/fixed-2026-07-26-command-eval-scene-mode-selection-projection.md)
- fixed 已修复：[command-eval-focused-document-projection](08/fixed-2026-07-15-command-eval-focused-document-projection.md)
- M1 全量行为门失败交接（`open / Editor10 接管项目与资产引用回归`）：[`10/failure-2026-07-12-project-asset-reference-full-gate-regressions.md`](10/failure-2026-07-12-project-asset-reference-full-gate-regressions.md)
- fixed 已修复：[plugin-extension-validation-regressions](08/fixed-2026-07-15-plugin-extension-validation-regressions.md)
- fixed 已修复：[editor-full-gate-thread-exhaustion](08/fixed-2026-07-14-editor-full-gate-thread-exhaustion.md)
- fixed 已修复：[rigid-body-sleep-policy-consumer-cutover](08/fixed-2026-07-12-rigid-body-sleep-policy-consumer-cutover.md)
- fixed 已修复：[realtime-ibl-option-then-type-errors](08/fixed-2026-07-12-realtime-ibl-option-then-type-errors.md)

## 参照证据（dev/）

**godot 反例**（`editor_node.h:120-150`）：`MenuOptions` 巨型枚举 + 中心分派——每加动作改中心类。架构红旗：命令执行体不得聚合在中心 match。

**Fyrox 正例**（`message.rs:47-124`）：菜单/快捷键/按钮折算 `Message` 变体投递，处理者分散各 owner `on_message`——命令→消息→分散处理者，即 01 总线消费侧。

**UE 模式调度**（`EditorModeManager.h`）：`FEditorModeTools` 是独占资源仲裁者——视口输入先问活跃模式栈；互斥组内先退后进。`ToolScheduler` 直译并泛化到「模态向导/独占面板」。

## 现状与证据（zircon，2026-08-31 current-source 复核）

### 命令与操作已收敛到 core 单一注册表

`EditorCommandRegistry` 现位于 `core/commands/registry.rs`，以 `BTreeMap<EditorOperationPath, EditorCommandDescriptor>` 持有唯一命令元数据，并在同一 owner 内保存 operation factory registration、generation 与 `OnceLock<Arc<EditorCommandPaletteCatalog>>`。`EditorCommandDescriptor` 已包含 menu/when/payload/headless/remote/capability/asset-write 元数据；`EditorCommandAction` 的当前形状是 `Emit(EditorEvent) | Operation | HeadlessAssetMigration | HeadlessPluginList`，operation factory 由 registry 内部 map 绑定，不内联到 enum，也没有旧 `Menu` 变体。

### when enablement 分派已落地，binding context 仍待硬切

`WhenClause` 已覆盖 `ProjectOpen/UndoAvailable/RedoAvailable/FocusedDocumentKind/SceneModeActive/SelectionNonEmpty/AssetWritable/PlayMode/Capability` 与 `All/Any/Not`。`CommandEvalCtx` 以 `interactive` 区分 UI/headless；headless 对不可求值的 UI 谓词返回 inapplicable，组合子按 `Option<bool>` 传播，最终 `eval` 才把 inapplicable 收敛为 false。

`EditorKeymap` 已支持内建 preset + typed settings override、borrowed keyboard signature index、按 command id 二分回查，以及按当前 `CommandEvalCtx` 过滤同 chord 候选的输入分派。current source 还新增了基于 `WhenClause` 可满足性的 conflict API，但生产设置/注册链没有 consumer，且 enablement predicate 不能替代稳定 binding context。默认 chord 又同时存在于 descriptor 与 `default.keymap.toml`，manager 输入与 menu/palette 仍有双权威；共享 command contribution v3 同时缺少 chord/context/when，插件 materialization 只能得到无默认 chord 的 `Always` descriptor。2026-08-31 复审已用真实四态枚举修复 PlayMode exact-one containment；最终合同改为 registry-derived descriptor defaults + preset delta + settings override、显式 context registry、父子域与 context-local signature index，并要求 runtime interface/SDK/materializer/registry transaction 退役 v3 后原子硬切，详见子计划 `08/2026-08-31-keymap-binding-context-current-review.md`。

### 菜单与 ToolScheduler 的真实边界

菜单保持两阶段投影：`core/commands/menu.rs` 物化 command-registry base；`ui/workbench/model/menu/extension_menu.rs` 再合并 extension `menu_items`、focused `DocumentToolkitDescriptor::menu_items` 与 extension views。extension/toolkit 项按 priority/path/operation 稳定排序，以 operation path 跨源去重，要求 canonical command 存在，并复用同一 when/keymap/i18n 投影；失焦后 toolkit 第三源立即退出。`EditorCommandMenuProjection::{CommandRegistry, ExtensionRegistry}` 仍只描述 base/append 两阶段 ownership，不承担 toolkit 生命周期。

`ToolScheduler` 已实现单资源和 canonical `ToolResourceSet` 原子租约、FIFO set queue、有界拒绝、withdraw/release-all，以及 `#[must_use] ToolScheduleReport` 生命周期事件发布合同；`ToolSchedulerService` 已挂入 `EditorContext`。当前源码只有 service/core tests 使用 acquire API，05 scene-mode 与 15 export-wizard 的真实生产接入仍是开放项。

### 剩余缺口

1. keymap 显式 binding context、生产 conflict generation 与设置/extension 消费；2. scene mode/export wizard 对 set-lease scheduler 的生产接入；3. palette/menu 的 1k/10k 规模、锁等待和 clone-byte 产品证据。上述缺口保持 open，不恢复已删除的 `ui/host/commands`、全量 `command_palette_value` 或第二 operation registry。

## 目标

1. **单一命令权威**：保持 `EditorCommandDescriptor` + registry-owned operation factory 的单一 id/metadata owner；CLI、面板和菜单都从该 registry 解析，不再引入 enum-inline factory 或第二 operation registry。
2. **when 谓词**：保留当前结构化 `WhenClause` 与 interactive/headless applicability 语义，所有入口复用同一 `CommandEvalCtx`。
3. **菜单合流**：保持 command-base + extension/toolkit/view append 两阶段投影、跨源稳定去重/排序和统一 enablement；toolkit 菜单只随 focused toolkit 生命周期出现，不把 projection ownership enum 扩成第二套菜单权威。
4. **keymap 分层**：command descriptor/registry 作为 default binding 唯一 owner，preset 只存 delta，再叠加 settings override；保留 signature index 与 current enablement dispatch，新增 registry-owned typed binding context，把父子域冲突、兄弟域共存和 active-context 查找从 `WhenClause` SAT 中硬切出来。
5. **`ToolScheduler`**：保留单资源/set-lease、有界队列和 lifecycle report；完成 05 模式栈与 15 导出向导生产接入。
6. **命令面板成型**：保留 generation-owned catalog、typed query window、when 过滤与 MRU，并补规模/锁/分配验收（外观归 editor_layout）。

## 非目标

- 宏录制/脚本化命令（依赖 13，远期）；面板视觉；输入底层（editor_ui/01）；能力体系本体（消费 01 `RuntimeCapabilities` 能力名）。

## 架构设计

### 模块布局

```
zircon_editor/src/core/commands/
  mod.rs
  descriptor.rs        # canonical descriptor + action route
  registry.rs          # command metadata + operation factories + palette generation
  when.rs              # WhenClause + CommandEvalCtx
  keymap.rs            # preset + settings override + context-local signature index
  menu.rs              # command-registry base menu projection
  palette.rs           # generation-owned catalog/query window/MRU
zircon_editor/src/core/tools/
  mod.rs / scheduler.rs
zircon_editor/src/ui/workbench/model/menu/
  extension_menu.rs    # extension/toolkit/view append + cross-source dedup
```

### 关键类型

```rust
// when.rs
pub enum WhenClause {
    Always, ProjectOpen, UndoAvailable, RedoAvailable,
    FocusedDocumentKind(DocumentKind), SceneModeActive(SceneModeId),
    SelectionNonEmpty, AssetWritable, PlayMode(PlayModePredicate), Capability(String),
    All(Vec<WhenClause>), Any(Vec<WhenClause>), Not(Box<WhenClause>),
}
impl WhenClause { pub fn eval(&self, ctx: &CommandEvalCtx) -> bool; }
// UI-only predicates in a headless context are inapplicable, not silently true.

// descriptor.rs
pub enum EditorCommandAction {
    Emit(EditorEvent),
    Operation,
    HeadlessAssetMigration,
    HeadlessPluginList,
}
// OperationCommandFactoryRegistration is stored by EditorCommandRegistry under the same id.

// tools/scheduler.rs
pub enum ExclusiveResource { ViewportInput, ModalSurface, SceneModeSlot }
impl ToolScheduler {
    pub fn acquire(...) -> ToolScheduleReport<AcquireOutcome>;
    pub fn acquire_set(...) -> ToolScheduleReport<AcquireSetOutcome>;
    pub fn release_set(...) -> ToolScheduleReport<ReleaseSetOutcome>;
    pub fn withdraw_set(...) -> ToolScheduleReport<WithdrawSetOutcome>;
    pub fn release_all(...) -> ToolScheduleReport<ReleaseAllOutcome>;
}
// ToolSchedulerService publishes every report event to the editor bus before exposing new state.
```

### 三入口同源

| 入口 | 路径 | 过滤 |
| --- | --- | --- |
| 菜单/工具栏 | `commands/menu.rs` base + workbench extension/toolkit/view append | when 置灰 |
| 命令面板 | `palette.rs` 模糊+MRU | when 隐藏 |
| CLI `--run/--operation` | 16 commandlet → `command(id)` | `callable_from_remote` 且 headless-when 通过 |

### 迁移映射（执行合同）

| 现物 | 去向 |
| --- | --- |
| 旧 `ui/host/commands` owner | 已硬切到 `core/commands/registry.rs`；全量 palette value 入口已删除，统一为 generation-owned query window |
| 旧 command→operation 字符串链接/第二 registry | 已收敛为 descriptor `Operation` route + 同 registry id 下的 factory registration；不内联 factory，不恢复第二 registry |
| 旧无 when descriptor | 已并入 `WhenClause`、`CommandEvalCtx`、remote/headless/capability/asset-write metadata |
| 旧单层 `EditorKeymap` | current 已有 preset + settings override + signature index + contextual enablement dispatch；后续硬切为 registry descriptor defaults + preset delta + settings override，并删除默认 chord 双权威。PlayMode exact-one containment 已落地，显式 binding context 与生产 conflict generation 仍 pending |
| 旧无调度 owner | `ToolSchedulerService` 已挂 `EditorContext`；05/15 真实 consumer 接入仍 pending |

### 深度测试

夹具功能域注册 5 命令（when/chord/menu_path/remote 各形态）：三入口全通、菜单含新项、面板过滤正确、CLI 可调——`core/commands/` 零改动。

## 里程碑

### M1 合一注册表与 when（排 01 M1 之后）

- 源码状态：`core/commands`、单一 descriptor/registry、registry-owned operation factories、`WhenClause/CommandEvalCtx` 与 headless applicability 已落地；旧 `ui/host/commands` 和第二 operation registry 不再是当前 owner。
- 剩余门：current-source `cargo test -p zircon_editor --lib --locked`、CLI/list-operations 一致性和当前模块文档验收；未取得这些证据前 M1 不标 completed。

### M2 菜单合流与 keymap 双层

- 切片 2.1：command base 与 extension/toolkit/view append 已落地；focused toolkit 第三源、capability filter、跨源 operation 去重、稳定排序、canonical command gate 与失焦退出已有源码和聚焦测试。`CommandAction::Menu` 已不存在，不再保留其迁移任务；current-source Cargo 与产品链路仍待验收。
- 切片 2.2：settings override、signature index 与上下文 enablement dispatch 已落地；PlayMode exact-one correctness containment 已静态完成。当前通用 SAT conflict API 无生产 consumer，不作为完成态；显式 binding context、父子冲突、兄弟共存与 conflict generation 仍 pending。共享 command contribution v3 无法表达该结构，必须与 SDK/materializer/registry transaction 一次性硬切为新 schema，不保留旧 schema 或静默默认域。
- 测试阶段：运行现有 toolkit 三源快照，并补 same-context/parent-child/sibling、missing/cycle/depth-limit，以及输入事件→active-context→命令端到端。

### M3 面板与 ToolScheduler

- 切片 3.1：共享 immutable catalog generation、typed query window 与 MRU 已落地；禁止恢复 `command_palette_value` 全量投影。1k/10k query、锁等待和 clone-byte 产品门仍 pending。
- 切片 3.2：scheduler core/service 已支持 single/set lease、withdraw/release-all、有界 queue 与 typed lifecycle report；05 模式栈/15 导出向导生产接入仍 pending。
- 测试阶段：复用 `core/tools/tests.rs` 的现有调度矩阵，新增真实 mode/wizard 竞争集成与面板产品链路；证据记状态节。

## 风险与开放问题

- registry 迁 core 与 01 M1 context 服务化排程耦合——M1 硬排在 01 M1 后。
- `CommandAction::Emit` 依赖 01 类型化载荷已落地；01 未完时以 `Custom` 过渡记债。
- when 谓词不给插件自定义（序列化限制）：插件组合内建谓词，不足场景走 `Capability` 自定义能力名兜底——契约注释声明。

## 产出记录与时间

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

| 日期 | 里程碑/切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-07-12 | M1 / Editor16 CLI 命令注册表硬切回传 | 已修复-目标行为门通过-全量门仍由外部失败阻断 | `zircon_app` 已删除 `EditorEventRuntime`、`QueryOperationStack` 与 `--operation-stack`，改由 `EditorHostEventController` 复用 `EditorManager.context().commands()` 唯一注册表；CLI 硬切为 `--operation-history` / `QueryOperationHistory`，factory 未就绪时保留 `OperationHistoryPendingFactory` 类型化失败。App 旧符号扫描为 0，`cargo +nightly fmt --all -- --check` 通过，`target-editor-host` 的 `editor_cli_operation_` 目标测试 14/14 通过；回传记录见 [fixed artifact](08/fixed-2026-07-12-command-registry-hard-cut-cli.md)。六 profile 聚合复跑仍仅被并发 Runtime Text 的 `RichTable*` 导出缺失阻断，不归属本修复。 |
| 2026-07-15 | M1.2 / focused document 权威投影回传 | 已修复（fixed） | Editor07 将 typed `ViewDescriptor.document_kind` 与唯一 `focused_view` 接入共享 `CommandEvalCtx`；默认项目无显式焦点保持 `None`，失效的显式焦点才回退 active document。current-source 16/16 通过，见 [fixed artifact](08/fixed-2026-07-15-command-eval-focused-document-projection.md)。 |
| 2026-07-14 | M1 / full-lib harness 线程耗尽回传 | 已修复（fixed） | Runtime service CoreWeak 硬切与两个确定性测试夹具修复后，3157-test full-lib 自然结束并产生 summary；功能断言与 Runtime11 瞬时峰值预算继续独立处理，见 [fixed artifact](08/fixed-2026-07-14-editor-full-gate-thread-exhaustion.md)。 |
| 2026-07-12 | M1 / rigid-body sleep policy consumer hard-cut | 已修复（fixed） | Physics consumer 已统一到 `PhysicsSleepPolicy::{Allow,Never}`，未恢复 `can_sleep` 字段或兼容 getter；Physics 27/27、reflection 1/1、property/project round-trip 2/2 与 Editor compile gate 通过，见 [fixed artifact](08/fixed-2026-07-12-rigid-body-sleep-policy-consumer-cutover.md)。 |
| 2026-07-18 | M3.1 / command palette catalog generation 与 typed query window | 源码完成，受管编译/性能/独立复核待协调器屏障 | `EditorCommandRegistry` 新增 generation-owned `Arc<EditorCommandPaletteCatalog>`，成功变更才推进代际并失效缓存；旧 `command_palette_entries/command_palette_value` 已硬删除。查询以 256 固定评分桶两遍流式扫描，只保留 `offset/limit` 页句柄且完整 match count 可分页；Workbench open/query edit 均收敛为 8 visible + 4 overscan，`.zui` Change route、binding authority、host intercept 与 bridge generation/match/window metadata 已接通且 registry lock 在 UI refresh 前释放。TDD RED 3+2、叶文件 rustfmt、ZUI 2/2、旧 API 扫描 0、静态合同 12/12、changed 15/scope 16、scoped diff check、staged 0 通过；Coordinator01 full-input snapshot failure 未关闭，故不声明 Cargo、1,000 输入 p95、独立 review、failure fixed 或 commit。深页键盘缺口已按组件所有权交接 [EditorUI06 failure](../editor_ui/06/failure-2026-07-18-command-palette-paged-keyboard-navigation.md)。子记录见 [2026-07-18-command-palette-catalog-query-generation.md](08/2026-07-18-command-palette-catalog-query-generation.md)。 |
| 2026-07-18 | M3.1 / CommandPalette 深页键盘窗口适配 | 源码完成，受管行为门待协调器屏障 | Editor08 新增权威 `CommandPalette/WindowRequested` Change route；host 校验请求 current offset，读取当前 query/catalog generation，复用 `command_palette_query_window` 查询目标页，并在 generation 漂移时无副作用拒绝。bridge 只投影 12 行容量、实际 visible count、total count 与 offset，不恢复完整 catalog。EditorUI06 typed request 与 native 有界导航见 [failure record](../editor_ui/06/failure-2026-07-18-command-palette-paged-keyboard-navigation.md)；Python 3/3、ZUI TOML、rustfmt、scoped diff check 通过，Cargo/产品交互/独立 review 未执行。 |
| 2026-07-22 | commands/keymap current-source性能复核 | 局部止损完成 / 索引与规模门待执行 | key normalize/Display删除临时lowercase与parts Vec，command→chord改有序binary search；每keyboard event按chord线性扫bindings仍归PERF-MVP-074，需compact signature index。Palette generation/window已成立，但非空query仍两遍全document fuzzy scan并按String id二次BTree查descriptor；PERF-MVP-211更新为slot/enablement index与增量候选目标。源码守卫/rustfmt/diff通过，Cargo/1M keyboard/1k query p95未执行。 |
| 2026-07-30 | commands current-source全量复核 | 17/17静态完成 / current-source动态门待执行 | 当前`key_chord/keymap`已发布borrowed keyboard signature与`HashMap<signature,candidate indices>`，故PERF-MVP-074旧的owned chord+全bindings扫描结论失效，只待collision/probe/alloc、Cargo与F4门。Palette已按catalog slot直接enablement且不再String id回查registry，但每query仍clone含capability Strings的context、持command mutex全scan，并最多substring+subsequence两遍；继续PERF-MVP-211的incremental index/immutable context generation。`menu_bar_model`每model build在mutex内按7个label重复registry scan并物化rows，纳入PERF-MVP-076/099；extension batch/route-name index归079/538。17/17 rustfmt通过，无Rust edit/Cargo/RenderDoc；证据见performance `01/2026-07-30-editor-core-commands-current-review.md`。 |
| 2026-08-13 | M3.1 / palette immutable query index、共享 context 与短锁消费 | 实现与二审完成 / 受管验收待回执 | Catalog generation 新增 256-byte postings 和 descriptor-aligned enablement slots，non-empty query 从最稀有 posting 无损收窄并以单次 document byte pass 同时计算 exact/subsequence；`CommandEvalSnapshotHandle` 按语义 generation 发布共享 context `Arc`，palette 不再逐键 clone capability strings。Registry query facade 硬删除，retained host 在锁内仅取得 catalog `Arc`，之后锁外完成 when/MRU/query/window 投影。Rust 回归新增 1k selective visit、exact/subsequence、重复 query byte、窗口/总命中和 shared context Arc 代际；Editor08+EditorUI06 Python contract 7/7、生产 Rust 旧 facade 扫描 0、rustfmt/diff-check 通过。独立二审先发现 1 个 stale consumer guard Important，前向修复后最终 `C/I/M=0/0/0`。current-source Cargo、1k input p95、pixel/disabled/commit 产品门未取得前保持 resolving failure。 |
| 2026-07-22 | command retired-symbol test infrastructure | 源码止损 / Cargo待执行 | PERF-MVP-561把`command_owner_hard_cut...`从“读取全部editor Rust并拼成巨型String”改为逐文件stream检查5个退役符号，peak source owner收为单文件；Python源码合同2/2中的对应门通过。后续统一结构inventory/changed-file audit与current-source Cargo仍待。 |
| 2026-07-22 | workbench layout/view-model tests静态复核 | 40/40静态完成 / 产品规模门待执行 | layout/focus/resize/registry测试仅有1–3个view，plugin menu仅2项，未覆盖PERF-MVP-077/097/099/538的增量复杂度。Editor08验收补1k/10k views、rapid focus/resize/page switch与plugin menu generation，记录placement/menu/reflection build、clone bytes、changed=false、持久化I/O和主线程p95；证据见performance `01/2026-07-22-editor-workbench-tests-static-review.md`。 |
