---
related_code:
  - zircon_editor/src/core/settings
  - zircon_editor/src/core/editor_event/replay.rs
  - zircon_editor/src/ui/activity
  - zircon_editor/src/core/commands/keymap.rs
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
status: in_progress
---

# 17 编辑器必要服务：设置分层 / 自动保存 / 崩溃恢复 / 日志诊断 / 通知 / 本地化

> 2026-08-02 实仓复核：M1 的三层 `SettingsRegistry` 与 legacy preferences 硬切、M2 的 autosave 调度骨架，以及 M3 的 Decision 通知中心已经落入当前源码；设置页贡献/热应用、布局真实持久化、session guard/restore flow、Toast/Progress、统一日志和 i18n 仍未完成。计划状态保持 `in_progress`，未落地模块不得据此视为已验收。

## 参照证据（dev/）

**godot 设置双层**（`dev/godot/editor/settings/editor_settings.h:43-148`）：`EditorSettings` 单例（`get_singleton()/create()`，`_THREAD_SAFE_CLASS_` 线程安全宏，内部 `HashMap<String, VariantContainer> props`）持**用户级**设置；工程级归 `ProjectSettings`（core 层独立单例）；导出 preset 等工程数据另存工程内（`editor_export_preset.h`）——**「跟人走」与「跟工程走」物理分离**是 godot 设置体系的第一原则。

**UE 恢复与消息日志惯例**（`Editor/UnrealEd`）：AutoSave 定时快照到独立目录 + 启动时检测非正常退出弹恢复会话；Message Log 按来源分频道、条目可携带跳转 token——本计划日志/恢复的行为模板（机制按 zircon 现物重设计）。

## 现状与证据（zircon）

**设置框架与 legacy preferences 硬切已落地**：`core/settings/` 已包含 `SettingsRegistry`、`SettingsScope::{User, Project, Session}`、definition/defaults、三层 IO、keymap override、设置页 descriptor 与 change drain；`EDITOR_DESIGN_TOKENS_KEY` 以 User scope 注册，`editor_design_tokens_at_startup()` 只经 registry 解析。旧 `EditorAppearancePreferences` 与 `ui/preferences/` owner 已删除，不得恢复第二套 authority；剩余缺口是设置页贡献注册、热应用链和产品级验证。

**布局持久化空壳已移除**（06 已证）：旧 `workbench/layout/manager/persistence.rs` 四函数只做克隆/透传且零文件 IO，现已从 owner tree 删除。真实 User/Project 布局持久化仍由 EditorLayout06 M3 基于 `SettingsScope` 实现；不得为满足旧路径或旧测试恢复无效壳层。

**恢复已具 autosave 调度骨架**：`core/recovery/autosave.rs` 已有 `AutosavePolicy`、`AutosaveScheduler`、`AutosaveJobPolicy`，并通过 mutex group / job spec 接入后台任务；`EditorEventJournal` + `EditorEventReplay::replay` 仍是第二素材线。当前真正缺口是 session guard、启动残锁检测、restore flow 与逐文档恢复决策，而不是重新实现 autosave 调度器。

**通知呈现件**：`ui/activity/` 实测为 `slot.rs/view.rs/window.rs` 三件（槽位/视图/窗口）——通知中心呈现层的迁移对象即此三件。

**日志分散**：runtime 侧 `tasks/diagnostics.rs`（任务观测）、`ZrHostApiV3.diagnostics{emit, metric}`（插件诊断域）、`--log-level/--log-filter`（CLI 既有）、04 Play 子进程 stdout/stderr、09 导入警告、13 编译诊断——**六个来源无汇聚面**。

**通知中心已落 Decision 子域**：`core/notifications/service.rs` 与 `notifications/decision/` 已提供 pending/receipt 有界决策通知；`ui/activity/` 呈现层、Toast/Progress 两类生命周期，以及 14/09/04/11/12 的生产者收口仍未完成。

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
  mod.rs / registry.rs / scope.rs / definition.rs / defaults.rs / io.rs
  keymap_overrides.rs / page.rs                 # 已落地：三层读写、定义与页面描述符
zircon_editor/src/core/recovery/
  mod.rs / autosave.rs                          # 已落地
  session_guard.rs / restore_flow.rs             # 待落地
zircon_editor/src/core/logging/
  mod.rs / sink.rs / entry.rs / rolling_file.rs  # 待创建
zircon_editor/src/core/notifications/
  mod.rs / service.rs / decision/                # 已落 Decision；Toast/Progress 待补
zircon_editor/src/core/i18n/
  mod.rs / catalog.rs / macros.rs                # 待创建
```

五服务全挂 `EditorContext`（01）；**全部事件化**——面板只是订阅者，headless（16 commandlet）下 settings/logging 照常运转（commandlet 的 `--diagnostics` 输出即 logging 落盘面）。

### 现物迁移映射

| 现物 | 去向 |
| --- | --- |
| 已删除的 `EditorAppearancePreferences` + 旧版本常量 + env | 已硬切至 `settings/` User 层与 registry-only 启动解析；不得恢复 `preferences.rs` |
| 已删除的 `persistence.rs` 空实现四函数 | 06 M3 实 IO，路径规则由本计划 `scope.rs` 提供（global→User 层目录、project→`.zircon/`） |
| `ui/activity/` | `notifications/` 呈现层（数据源接现有 `service.rs`；不再假定不存在的 `center.rs`） |
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

## 2026-07-22 decision notification性能补充

`DecisionNotificationCenter`已有pending=128、receipt=256硬容量且通知主体使用`Arc`，不属于MVP高频主线程瓶颈。本轮把publish的全entries pending扫描改为维护O(1)计数，并用“resolve后立即释放pending容量”回归锁语义；后续只在产品trace证明snapshot/receipt轮询频率过高时再做cursor/index优化，不建立第二套缓存。

## 2026-07-30 settings current-source性能交接

- PERF-MVP-590：当前viewport单个snap-step命令会clone完整`SettingsRegistry`，随后同步clone整层、编码整份document并在UI caller执行write/fsync/rename。Editor17建立唯一内存authority和typed key/value+generation提交，按scope/key latest coalesce后交Runtime11共享bounded atomic-persistence ticket；UI立即read-your-write但filesystem wall=0，flush/shutdown显式等待durability。不得建立viewport/settings私有worker，也不得在排队前clone完整registry/document。
- PERF-MVP-591：EditorManager、retained-host design-token启动和viewport当前各自构造/加载registry；`changes: Vec`无生产drain且MRU变化可长期追加，stable `chrome_settings()`还为三个静态key重复分配/查三层BTree。Editor17发布唯一immutable settings generation，注册期把built-in keys编译为typed slots，change delta按entry+bytes+cursor/age有界，no-op set不递增revision/event；Editor05只消费resolved slots。startup每generation每文件read/decode≤1，generic strict-envelope双解析继续归Editor11 PERF-MVP-570。
- 验收使用definitions/keys `1/1K/100K`、value `0/1KiB/1MiB`、same-key/MRU changes `1/1K/1M`、stable snapshot `60/120Hz`、filesystem `0/10ms/2s`、consumers/writers `1/16`；记录authority、reads/decode passes、full clone bytes、key/String alloc、BTree probes、journal/queue entries+bytes+age、writes/fsync、RSS与UI p95。要求authority=1、stable key parse/lookup=0、journal/queue硬有界、UI filesystem wall=0、单key full-registry clone=0，并保持precedence/restart/keymap/tokens/MRU/snap/crash/flush语义。证据见`../../performance/01/2026-07-30-editor-core-settings-static-review.md`；managed Cargo与F0/F4仍open，不进入`review.md`。

## Code Review 处理结果 (2026-08-01)

### 已处理

- front matter 已提升为 `in_progress`；现状节与模块图已按 `core/settings/`、`core/recovery/autosave.rs`、`core/notifications/service.rs` 和 `notifications/decision/` 的真实落点更新。
- legacy preferences 硬切已完成；设置页贡献/热应用、真实布局持久化、session guard/restore flow、Toast/Progress 被明确保留为待接线项，没有因 status 提升而误标完成。

### 实现风险 / 技术债

- 目标 3 `EditorLog`（`core/logging/`）与目标 5 i18n（`core/i18n/`）两个模块目录当前仍不存在；M3 切片 3.1 和 3.3 必须从 owner 创建与消费接线开始，不能复用 settings/notification 的进度宣称。
- Decision 已落地不代表通知中心三类契约完成；Toast、Progress、`ui/activity/` 呈现迁移和生产者收口仍需单独验收。
