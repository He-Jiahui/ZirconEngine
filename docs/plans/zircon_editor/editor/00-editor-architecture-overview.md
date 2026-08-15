---
related_code:
  - zircon_editor/src/lib.rs
  - zircon_editor/src/core/mod.rs
  - zircon_editor/src/ui/host/module.rs
  - zircon_editor/src/core/editor_event/mod.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime_interface/src/runtime_api.rs
reference_sources:
  - dev/Fyrox/editor/src/lib.rs
  - dev/godot/editor/editor_node.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd
plan_sources:
  - docs/plans/zircon_editor/editor/index.md
  - docs/plans/engine-code-structure-convention.md
status: planned
---
# 00 Zircon Editor 总体架构设计（各分计划的公共骨架）

本文是 01–17 分计划共享的**总体设计定形**：分层、聚合根、线程模型、帧数据流、状态事实源、目标目录布局。分计划只在本骨架的既定位置填充细节；分计划之间的接口以本文的层边界为准。分计划与本文冲突时，**修订本文并回改分计划**，不允许各计划私设平行骨架。

## 1. 设计原则（从取证事实推导，非空泛口号）

1. **编辑器是 runtime 的客户端，不是宿主**。runtime 经 `zircon_runtime_get_api_v2` → `ZrRuntimeApiV2`（19 字段、17 个 session 入口）动态加载，每 session 独立 `CoreRuntime` + `LevelSystem`。编辑器一切世界访问走门面（01 `EditorRuntimeGateway`），不直接持有 world 引用穿透边界；长任务统一走 submit/poll/harvest，不保留 V1 表兼容入口。
2. **单事实源，多订阅者**。当前 `EditorEventRuntimeState` 单 `Mutex` 聚合 14 字段是反例：状态归属不清导致所有面板抢一把锁。总体设计给每类状态指定唯一权威子系统（§6 事实源表），其余组件只经事件/查询消费。
3. **注册制扩展，宣告式接入**。既有 `EditorExtensionRegistry` 13 张描述符表证明「表驱动」路线已成立；缺的是统一生命周期（ticket/revoke/changed_since）。新功能域一律以「注册一条贡献」的形态接入（06 `ContributionStore`），使插件（12）与第一方功能走同一扩展面。
4. **headless 是一等形态**。产品无头入口只保留 `--run <commandlet>`；内核层（L1–L3）不得依赖窗口或工作台物化，commandlet（16）与 CI 直接驱动内核层。
5. **编辑器状态不进 runtime 序列化**。既有 authoring token 守卫必须持续通过；选中/展开/布局等编辑器态只存在于编辑器层与编辑器落盘域（`.zircon/`、`~/.zircon/editor/`）。
6. **硬切换**。每个分计划的「现物迁移映射表」是执行合同：新 owner 落地即迁调用方、删旧路径，无兼容层。

## 2. 目标分层架构

```
┌─ L5 呈现层（外观实现归 editor_layout/editor_ui 计划集）──────────────┐
│  workbench 停靠壳 / ViewHost 视图宿主 / inspector·field 渲染        │
│  viewport 场景视口 / 领域编辑器（动画·状态机·行为树）/ 通知·日志面板   │
├─ L4 扩展与贡献层 ────────────────────────────────────────────────┤
│  ContributionStore(06) / 命令注册表+Keymap(08) / DocumentToolkit(06) │
│  编辑器插件(12) / commandlet 投影(16) / 设置页贡献(17)               │
├─ L3 编辑模型层（可 headless 运转）─────────────────────────────────┤
│  事务与撤销 EditorTransactionEngine(03) / SelectionModel(05)        │
│  GraphModel·TimelineFoundation(07) / EditorAssetIndex·zmeta(09)     │
│  ZirconProjectManifest·AssetGuid·RegistryIndex(10) / 迁移链(11)     │
├─ L2 runtime 交互层 ──────────────────────────────────────────────┤
│  EditorRuntimeGateway 双实现(01) / WorldSyncProtocol(02)            │
│  PlaySessionController(04) / ScriptBuildOrchestrator(13)            │
│  ExportPipeline(15)                                                 │
├─ L1 编辑器内核（服务底座，全部 headless 可用）─────────────────────┤
│  EditorContext 聚合根(01) / EditorMessageBus 类型化载荷(01)          │
│  EditorJobSystem(14) / SettingsRegistry(17) / EditorLog(17)         │
│  autosave·恢复(17) / i18n(17) / EditorEventJournal（既有）           │
├─ L0 进程与引导（zircon_app / zircon_hub）─────────────────────────┤
│  EditorLaunchArgs 统一解析(16) / EditorModule 模块注册（既有）       │
│  hub 握手·单实例锁(16) / LoadedRuntime libloading（既有）            │
└──────────────────────────────────────────────────────────────────┘
```

层间依赖只允许**上层依赖下层**；同层横向依赖须经事件（bus）或注册表（贡献），禁止直接互持。判定示例：inspector（L5）改字段 → 发命令（L4）→ 事务引擎（L3）→ gateway（L2）→ runtime；runtime 变更 → invalidation（L2）→ `ViewDirtySet`（L1 bus 拓扑）→ 面板重取（L5）。**任何绕层直连（面板直接 `with_world_mut`）都是返工项。**

## 3. 内核聚合根 `EditorContext`（01 详设，按当前实现与规划边界定形）

当前已实现聚合面（`core/context/editor_context.rs`）：

```rust
pub struct EditorContext {
    bus: SharedEditorMessageBus,
    events: Arc<EditorEventService>,
    jobs: EditorJobSystem,
    notifications: EditorNotificationService,
    transactions: EditorTransactionEngine,
    commands: EditorCommandRegistryHandle,
    command_eval: CommandEvalSnapshotHandle,
    tools: ToolSchedulerService,
    gateway: EditorRuntimeGatewayHandle,
}
```

规划接入字段不是当前实现：`settings` / `log`（17）、`journal`、`selection`（05）、`assets`（09）、`project`（10）、`contributions`（06）。这些 owner 只有在对应计划交付真实服务和构造依赖后才接入 `EditorContext`；不得先加空壳字段或第二事实源来让代码表面符合计划。

定形规则：

- `EditorContext` 由 `EditorManager`（既有 Lazy manager，`ui/host/module.rs:54-63`）在首次物化时构造并持有；headless 路径由 commandlet runner（16）直接构造，**不经工作台**。
- 字段显式、访问受控：新 L1–L4 服务一律成为私有字段并由类型化访问器暴露，禁止 `HashMap<TypeId, Box<dyn Any>>` 式服务定位器（丢编译期检查）。`EditorContextBuilder` 的构造顺序必须遵循依赖拓扑，禁止各计划无约束地向尾部堆叠初始化。
- `EditorEventRuntimeState` 的 14 字段按 §6 事实源表拆散归位后删除该类型（01 迁移映射表为执行合同）。

## 4. 线程模型

| 线程/池                | 归属    | 职责                                      | 约束                                                                                                                                             |
| ---------------------- | ------- | ----------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| 主线程                 | L5      | Retained UI、输入派发、命令执行、事务提交 | 事务提交必须在主线程（撤销栈无锁化前提）；禁止阻塞式等待 job                                                                                     |
| runtime 帧线程         | runtime | session tick、`LevelSystem` 突变应用    | 编辑器经 gateway 排队访问，`with_world_mut` 在帧边界应用                                                                                       |
| `EditorJobSystem` 池 | L1(14)  | 导入/烘焙/导出/autosave/缩略图/脚本编译   | 包装 runtime`TaskPool`/`JobScheduler`，按 `JobCategory` 配额 + `MutexGroup` 互斥；**回流主线程走 bus 消息**，不回调闭包跨线程摸 UI |
| Play 子进程            | L2(04)  | P3 后端游戏进程                           | stdout/stderr 管道进`EditorLog`；参数经 16 `subprocess_args` 透传                                                                            |

裸 `thread::spawn` 全仓禁止（14 落地后加守卫测试；现存唯一例 `ExportWizardJobController` 收编为门面首个客户）。

## 5. 帧数据流（编辑一拍的完整路径）

```
输入(键鼠/菜单/命令面板)
  → EditorCommandRegistry.dispatch(cmd, context)          # L4，WhenClause 判定
  → 命令体开 TransactionScope，构造 EditCommand           # L3
  → EditorTransactionEngine.commit(cmd)                   # L3：执行 + 入 HistoryStore
  →   cmd.apply(gateway)                                  # L2：with_world_mut / 资产写
  → runtime 帧边界应用突变，产 InvalidationBatch           # L2(02)
  → WorldSyncProtocol 折算 WatchKey 命中                   # L2(02)
  → bus 投递（类型化载荷）→ ViewDirtySet 置脏              # L1
  → 各面板下一 UI 帧按脏键重取（query API，非推全量）       # L5
```

反向（runtime 自发变化，如 PIE 运行、热重载 `AssetReloadFrameApplyReport`）从第 5 步进入同一管线——**面板永远只有一条数据来路**。

## 6. 状态事实源表（唯一权威，越权即返工）

| 状态                       | 权威                                                   | 消费方式                                                              |
| -------------------------- | ------------------------------------------------------ | --------------------------------------------------------------------- |
| 世界数据（实体/组件/层级） | runtime`LevelSystem`（每 session 独立）              | 02 query/watch；编辑经 03 事务                                        |
| 选中集（场景域/资产域）    | `SelectionModel`（05；尚未接入 `EditorContext`）     | bus`Focus` 族事件；**`selected_node` 迁出 runtime session** |
| 文档脏态                   | `HistoryStore.saved_top`（03）                       | 标题星号/关闭拦截/autosave 触发均问它                                 |
| Edit/Play 状态             | `PlaySessionController`（04）                        | bus`Mode` 族事件；命令 `WhenClause` 引用                          |
| 资产元数据/引用图          | `EditorAssetIndex` + `AssetRegistryIndex`（09/10；尚未接入 `EditorContext`） | 查询 API；变更走导入管线                                              |
| 工程身份/路径/设置指针     | `ZirconProjectManifest`（10；`ProjectAuthority` 尚未接入 `EditorContext`） | 只读快照；写经`ProjectAuthority`                                    |
| 设置值                     | `SettingsRegistry` 三层 resolve（17；尚未接入 `EditorContext`） | `SettingChanged` 热应用                                             |
| 布局/视图态                | `ViewHost` + `LayoutPreset`（06）                  | 持久化走 17 路径规则                                                  |
| 扩展贡献                   | `ContributionStore`（06；尚未接入 `EditorContext`）  | ticket 注册 / revoke / changed_since 拉取                             |

## 7. 目标目录布局（`zircon_editor/src` 收敛终态）

```
zircon_editor/src/
  lib.rs                    # 薄化：只留 EditorModule/EditorPlugin/入口 run_* 等高层导出
                            # （现状 :18-58 的 export-wizard 巨型 re-export 面收回 owner 模块）
  core/                     # L1–L3：headless 可用，禁止依赖 ui/
    context/                # EditorContext 聚合根与构造 owner（01）
      mod.rs
      builder.rs
      editor_context.rs
      tool_scheduler.rs
    editor_message/         # 既有 bus + 类型化载荷（01）
    sync/                   # WorldSyncProtocol（02）
    transactions/           # EditCommand/HistoryStore/TransactionScope（03）
    play/                   # PlaySessionController + 三后端（04）
    selection/              # SelectionModel（05）
    commands/               # 注册表+keymap 自 ui/host/commands 迁入（08）
    asset_index/            # EditorAssetIndex/zmeta/DirtyRegistry（09）
    project/                # ProjectAuthority/manifest/guid/registry（10）
    migrate/                # VersionedPayload/MigrationChain（11）
    editor_plugin*/         # 既有插件三件套 + LoadingPhase（12）
    script_build/           # ScriptBuildOrchestrator（13）
    jobs/                   # EditorJobSystem（14）
    export/                 # ExportPipeline 编排（15；阶段契约在 interface）
    commandlet/ hub_link/   # 16
    settings/ recovery/ logging/ notifications/ i18n/   # 17
    editing/ editor_event/ editor_extension.rs …        # 既有，逐计划收编
  scene/                    # 视口编辑域：viewport 八子模块、gizmo、模式栈（05）
  ui/                       # L5：workbench/ViewHost/inspector/领域编辑器/呈现
```

`core/` 不依赖 `ui/` 是可机检不变量（深度测试基线：grep `use crate::ui` 于 core/ 下为零，08 registry 迁移完成后启用）。

## 8. 计划→架构映射与执行波次

| 波次        | 计划                                                     | 落点层         |
| ----------- | -------------------------------------------------------- | -------------- |
| W1 基座     | 01 内核/门面 · 11 序列化 · 14 job                      | L1/L2 骨架先立 |
| W2 核心编辑 | 02 同步 · 03 事务 · 05 场景 · 06 扩展框架             | L2/L3/L4 主干  |
| W3 功能域   | 04 PIE · 07 领域编辑器 · 08 命令 · 09 资产 · 10 工程 | 填充功能域     |
| W4 生态交付 | 12 插件 · 13 脚本 · 15 发行 · 16 CLI/hub · 17 服务   | 外沿与交付     |

（17 的 `SettingsRegistry` 虽列 W4，其**路径规则**被 06 布局落盘引用——若 06 M3 先行，先落 `settings/scopes.rs` 单文件，17 到位后收编。）

## 9. 与既有模块注册的关系

`EditorModule`（`ui/host/module.rs:37-101`）保持五模块依赖与「1 Driver + 4 Lazy Manager」注册形态不变；本计划集新增服务**不逐个注册进模块内核**，而是全部经 `EditorContext` 聚合（`EditorManager` 构造）。理由：模块内核的 manager 粒度面向 runtime 生命周期，编辑器内部服务粒度细得多，逐个注册会把 13+ 服务名常量灌进模块描述符（root wiring 变厚，违反结构规则）。`EDITOR_COMMAND_REGISTRY_NAME`/`EDITOR_KEYMAP_NAME` 两个既有 manager 注册位在 08 迁移后指向 core/ 新 owner，名称与解析行为保持。

## Code Review 收敛结果（2026-08-01）

### 已同步当前实现

- §3 已拆分当前聚合字段与规划接入字段；未交付服务不再伪装成当前 `EditorContext` 成员。
- §7 已同步为 `core/context/` 目录 owner，并列出 `mod.rs`、`builder.rs`、`editor_context.rs`、`tool_scheduler.rs`。
- §6 已标注尚未接入 `EditorContext` 的权威服务；这不关闭对应计划，也不关闭现有 core-root facade failure。

### 保留的架构约束

- 继续禁止服务定位器；新增服务必须显式建模，并按 `EditorContextBuilder` 依赖拓扑构造。
- 当前源码与计划的收敛证据见 [`00/2026-08-01-current-source-plan-convergence.md`](00/2026-08-01-current-source-plan-convergence.md)。
