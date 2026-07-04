---
related_code:
  - zircon_editor/src/ui/preferences.rs
  - zircon_editor/src/ui/workbench/layout/manager/persistence.rs
  - zircon_editor/src/core/editor_event/replay.rs
  - zircon_editor/src/ui/activity
  - zircon_editor/src/ui/host/commands/keymap.rs
  - zircon_runtime/src/core/runtime/tasks/diagnostics.rs
reference_sources:
  - dev/godot/editor/settings/editor_settings.h
  - dev/godot/editor/export/editor_export_preset.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd
plan_sources:
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
  - docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
status: planned
---

# 17 编辑器必要服务：设置分层 / 自动保存 / 崩溃恢复 / 日志诊断 / 通知 / 本地化

## 参照证据（dev/）

**godot 设置双层**（`dev/godot/editor/settings/editor_settings.h:43-148`）：`EditorSettings` 单例（`get_singleton()/create()`，`_THREAD_SAFE_CLASS_` 线程安全宏，内部 `HashMap<String, VariantContainer> props`）持**用户级**设置；工程级归 `ProjectSettings`（core 层独立单例）；导出 preset 等工程数据另存工程内（`editor_export_preset.h`）——**「跟人走」与「跟工程走」物理分离**是 godot 设置体系的第一原则。

**UE 恢复与消息日志惯例**（`Editor/UnrealEd`）：AutoSave 定时快照到独立目录 + 启动时检测非正常退出弹恢复会话；Message Log 按来源分频道、条目可携带跳转 token——本计划日志/恢复的行为模板（机制按 zircon 现物重设计）。

## 现状与证据（zircon）

**设置只有一个孤岛**：`EditorAppearancePreferences`（`preferences.rs`，2026-07-05 复核）——`APPEARANCE_PREFERENCES_VERSION: u32 = 1`（:12）、`APPEARANCE_PREFERENCES_PATH_ENV = "ZIRCON_EDITOR_APPEARANCE_PREFERENCES"`（:13，**路径靠环境变量注入**，无默认用户目录约定）、单字段 `design_tokens: EditorDesignTokens`（:17，token 族=interface 侧 `EditorControlTokens/EditorDensityTokens/EditorPaletteTokens/EditorStateRoleTokens/EditorTypographyTokens` 五类，:7-10——迁 settings 后 schema 按此五类分组）、四方法（:104-141）；两常量均 `pub(crate)`（收编时无外部消费者顾虑）。无工程级/会话级概念，无变更通知，无设置页数据源。

**布局持久化是空壳**（06 已证）：`workbench/layout/manager/persistence.rs:6-27` 四函数（`load/save_global_default`、`load/save_project_workspace`）**返回克隆、零文件 IO**——但其函数签名已经隐含「全局默认/工程工作区」双层——本计划设置分层与 06 M3 落盘共用同一路径规则。

**恢复素材已有两件**：`EditorEventJournal` + `EditorEventReplay::replay`（`editor_event/replay.rs:3-6`，复核在案）；03 计划将产 `serialize_journal` 事务 journal——两条素材线均已规划，缺**编排者**（会话锁/autosave 触发/启动检测/恢复对话）。

**通知呈现件**：`ui/activity/` 实测为 `slot.rs/view.rs/window.rs` 三件（槽位/视图/窗口）——通知中心呈现层的迁移对象即此三件。

**日志分散**：runtime 侧 `tasks/diagnostics.rs`（任务观测）、`ZrHostApiV3.diagnostics{emit, metric}`（插件诊断域）、`--log-level/--log-filter`（CLI 既有）、04 Play 子进程 stdout/stderr、09 导入警告、13 编译诊断——**六个来源无汇聚面**。

**通知雏形**：`ui/activity/` 活动型 UI 存在；14 进度中心、09 导入结果、04 PlayCrashed、11 `migrated_from` 提示均已在各计划声明「投通知中心」——收口者缺位。

**本地化**：无框架，UI 文案硬编码中英混排。

## 目标

1. **`SettingsRegistry` 三层作用域**：

```rust
pub enum SettingsScope { User, Project, Session }   // godot 双层 + 会话易失层
pub struct SettingDefinition {
    pub key: SettingsKey,            // "editor.autosave.interval_secs" 点分命名空间
    pub scope: SettingsScope,
    pub schema: SettingSchema,       // Bool/Int{range}/Float/String/Enum{variants}/Color/Chord…
    pub default: SettingValue,
    pub requires_restart: bool,
    pub category_path: String,       // 设置窗口分组
}
// resolve(key)：Session 覆盖 Project 覆盖 User 覆盖 default
// 变更事件 SettingChanged{key, scope} 入 01 bus —— 订阅方热应用（requires_restart=false 者）
```

   存放：User 层 `~/.zircon/editor/settings.toml`（`APPEARANCE_PREFERENCES_PATH_ENV` 环境变量保留为 User 层根覆盖）、Project 层 `<root>/.zircon/settings.toml`（10 manifest `settings` 指针指之）、Session 层不落盘；全部 11 壳。首批迁入：preferences design_tokens（User）、08 keymap 用户覆盖（User）、05 吸附步进（Project）、14 类别配额（User）、13 合批窗口（User）。设置页=06 贡献一类（`SettingsPageContribution`，12 插件设置页同型）。
2. **自动保存与崩溃恢复**：
   - autosave：触发=定时（默认 300s，设置化）∧ 存在脏文档（03 `saved_top` 判定）；产物=脏文档快照写 `<root>/.zircon/autosave/<doc-id>/<seq>.*`（**绝不触源文件**，序号轮转保 3 份）；执行=14 `JobCategory::Misc` 低优先且与保存互斥组。
   - 会话锁：`<root>/.zircon/session.lock`（16 单实例锁复用，PID+心跳）；正常退出删锁。
   - 启动检测：残锁 ∧ autosave 新于源文件 → 恢复对话（逐文档：恢复自 autosave / 丢弃 / 并列打开对比）；第二素材线=03 事务 journal 重放（autosave 缺失时的兜底，M2 后接）。
3. **`EditorLog` 汇聚**：六来源统一条目 `LogEntry { source: LogSource, severity, message, timestamp_frame, jump: Option<JumpToken> }`（`LogSource::{Editor, Runtime, Play(instance), Plugin(id), Import, ScriptBuild}`）；环形内存缓冲（上限设置化）+ 滚动落盘（`--diagnostics` 路径或 `<root>/.zircon/logs/`，按日轮转）；面板数据源（频道过滤/severity 过滤/JumpToken 分派——13 诊断跳转、09 资产定位共用）；13 `ScriptDiagnostic` 与插件 `diagnostics{emit}` 域折算接入。
4. **通知中心**：三类契约——`Toast`（自动消退）/ `Progress`(绑 14 JobTicket) / `Decision`（需用户选择，如恢复对话、10 删除阻断）；生产者清单收口（14 job 事件、09 导入、04 Play 崩溃、11 migrated_from、12 插件 Faulted、本计划恢复对话）；`ui/activity/` 迁为呈现层。
5. **本地化框架**：`i18n` key 化——词条表为编辑器自身资产（09 A 义只读源，`assets/i18n/<lang>.toml`）；`tr!(key)` 宏 + 缺词回退链（当前语言→en→key 原文显示）；语言为 User 设置项热切换；首批接入：08 命令 label/菜单、设置项 `category_path` 与描述、通知模板；插件词条随插件包（12 清单声明词条文件）。

## 非目标

- 遥测/在线崩溃上报（隐私与后端另立案）；设置云同步；无障碍接入（runtime 无障碍树 `capture_accessibility_tree` 契约已有，编辑器消费另评估）；日志结构化查询（面板过滤够用）。

## 架构设计

### 模块布局

```
zircon_editor/src/core/settings/
  mod.rs / registry.rs / scopes.rs / io.rs      # 三层读写（11 壳 + 路径规则）
zircon_editor/src/core/recovery/
  mod.rs / autosave.rs / session_guard.rs / restore_flow.rs
zircon_editor/src/core/logging/
  mod.rs / sink.rs / entry.rs / rolling_file.rs
zircon_editor/src/core/notifications/
  mod.rs / center.rs / kinds.rs
zircon_editor/src/core/i18n/
  mod.rs / catalog.rs / macros.rs
```

五服务全挂 `EditorContext`（01）；**全部事件化**——面板只是订阅者，headless（16 commandlet）下 settings/logging 照常运转（commandlet 的 `--diagnostics` 输出即 logging 落盘面）。

### 现物迁移映射

| 现物 | 去向 |
| --- | --- |
| `EditorAppearancePreferences` + 版本常量 + env | `settings/` User 层首批条目（11 M2 收编版本常量；env 降级为根覆盖）；`preferences.rs` 删除 |
| `persistence.rs` 空实现四函数 | 06 M3 实 IO，路径规则由本计划 `scopes.rs` 提供（global→User 层目录、project→`.zircon/`） |
| `ui/activity/` | `notifications/` 呈现层（数据源迁 center.rs） |
| `EditorEventJournal`/replay | 保留（01 M1 已定位事件服务）；`restore_flow.rs` 第二素材线消费者 |

### 关停顺序（14 收尾协议并入）

`shutdown`: 停收新 job → 最后一次 autosave（若脏）→ job 门面 `shutdown(deadline)` → 日志冲刷落盘 → 删会话锁。顺序固定并入 14 M3 收尾协议测试。

### 深度测试

新增设置项/通知类/日志来源均为注册制：夹具各一（设置项出现在设置页数据源并热应用、通知走三类生命周期、日志来源可过滤可跳转），五服务目录零改动。

## 里程碑

### M1 设置框架

- 切片 1.1：`settings/` 四文件（registry/resolve 链/三层 IO/变更事件）；preferences 迁入删除；首批五组设置项登记。
- 切片 1.2：`SettingsPageContribution`（06 贡献族）+ 设置窗口数据源（外观 editor_layout preferences-window 设计稿）。
- 测试阶段：`cargo test -p zircon_editor --lib --locked`（preferences 既有测试迁移后须过 + resolve 覆盖链矩阵 + 热应用/requires_restart 分流 + 坏文件回退 default）。更新 `docs/zircon_editor/core/settings.md`。

### M2 自动保存与崩溃恢复

- 切片 2.1：`autosave.rs`（触发/轮转/不触源守卫）+ `session_guard.rs`（16 锁复用）。
- 切片 2.2：`restore_flow.rs` 启动检测 + 恢复对话（Decision 通知）+ 逐文档三选；03 journal 兜底线接口留位（03 M4 后接通）。
- 测试阶段：子进程 kill 夹具 → 重启检测 → 恢复断言（集成测试）；autosave 不污染源文件守卫（源 digest 前后一致）；轮转保 3 份边界。

### M3 日志、通知与本地化

- 切片 3.1：`logging/`：六来源接入（Play stdout 管道、13 诊断、插件 diagnostics 域折算）+ 环形缓冲 + 滚动落盘 + 面板数据源。
- 切片 3.2：`notifications/` 三类契约 + 生产者收口清单迁移 + activity 收编。
- 切片 3.3：`i18n/`：catalog + `tr!` + 回退链 + zh-CN/en 双包；首批词条接入（命令/设置/通知模板）；语言热切换。
- 测试阶段：日志来源路由/过滤/JumpToken 分派矩阵；通知三类生命周期（消退/进度绑定/决策回执）；缺词回退链与热切换单测；证据记状态节。

## 风险与开放问题

- autosave 与大场景序列化耗时：快照走 14 后台 job + 写临时文件后原子改名——若单文档序列化 >1s（基线实测），改为仅序列化脏子树的增量 autosave（依赖 02 世代号），证据裁决。
- `tr!` 宏引入的编译期词条校验（缺 key 编译警告）需 build script 扫描——先运行期回退 + `audit-i18n` commandlet 离线校验（16 注册），不做编译期魔法。
- Play 子进程 stdout 编码（Windows 控制台代码页）——统一要求子进程 UTF-8 输出（runtime_preview 侧 env 注入），乱码兜底按字节透传标注 source。
