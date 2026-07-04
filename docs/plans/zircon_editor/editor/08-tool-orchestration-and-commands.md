---
related_code:
  - zircon_editor/src/ui/host/commands/registry.rs
  - zircon_editor/src/ui/host/commands/keymap.rs
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/ui/host/module.rs
reference_sources:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/EditorModeManager.h
  - dev/Fyrox/editor/src/message.rs
  - dev/godot/editor/editor_node.h
plan_sources:
  - docs/plans/zircon_editor/editor/00-editor-architecture-overview.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/zircon_editor/editor/06-ui-extension-framework.md
status: planned
---

# 08 工具管理调度 / 命令系统 / 命令面板

## 参照证据（dev/）

**godot 反例**（`editor_node.h:120-150`）：`MenuOptions` 巨型枚举 + 中心分派——每加动作改中心类。架构红旗：命令执行体不得聚合在中心 match。

**Fyrox 正例**（`message.rs:47-124`）：菜单/快捷键/按钮折算 `Message` 变体投递，处理者分散各 owner `on_message`——命令→消息→分散处理者，即 01 总线消费侧。

**UE 模式调度**（`EditorModeManager.h`）：`FEditorModeTools` 是独占资源仲裁者——视口输入先问活跃模式栈；互斥组内先退后进。`ToolScheduler` 直译并泛化到「模态向导/独占面板」。

## 现状与证据（zircon，2026-07-05 实读）

### 命令注册表比 v2 记载成熟（两处修正）

`EditorCommandRegistry`（`ui/host/commands/registry.rs:20-186`）完整 API：

```rust
pub struct EditorCommandRegistry { commands: Vec<EditorCommandDescriptor>, by_id: BTreeMap<String, usize> }
// new(Vec) -> Result（DuplicateCommand 守卫已有，:26-36）
// default_workbench() / commands() / command(id)
// event_for_command(id) -> Result<EditorEvent, _>            // 分派：Menu→WorkbenchMenu 事件；Operation→操作事件
// command_palette_entries(context) / command_palette_value(context) -> UiValue
// menu_bar_model(context) -> MenuBarModel                    // ← 菜单物化已存在（v2「有表无消费」失实）
// menu_model(label, context) -> Option<MenuModel>
// missing_default_keymap_bindings(...)                       // ← 注册表↔keymap 一致性检查已有
```

**修正一：菜单物化已有**——`menu_bar_model/menu_model(context)` 从命令生成菜单模型；缺的是与 06 `menu_items` 贡献表的合流（现只出注册命令，插件贡献菜单项不进模型）。
**修正二：命令→操作已单向链接**——`EditorCommandAction::{Menu(action), Operation(operation_id)}`（:52-60 分派处实读）：命令可指向操作 id，但两套描述符、两个 id 空间、操作字段（`menu_path/callable_from_remote/required_capabilities/payload_schema_id`）在命令侧不可见。

### keymap（`keymap.rs:12-84`）

`EditorKeymap { bindings: Vec<EditorKeyBinding{command_id, chord}> }`：`default_workbench/from_toml/bindings/resolve(chord)->Option<&str>/resolve_keyboard_input(UiKeyboardInputEvent)/chord_for_command`。输入事件直连解析已有；无用户覆盖层、无冲突检测、无上下文域。

### 操作层与 CLI

`EditorOperationDescriptor` 字段（03 已核）：`menu_path/payload_schema_id/callable_from_remote/required_capabilities` 声明先进于消费。CLI `--operation/--args/--operation-group/--list-operations/--operation-stack/--headless`（`entry_runner/editor.rs:313-361`）直连 `EditorOperationRegistry`——**无头调用面已存在**。

### 缺口（重新定界）

双描述符仅单向链接，字段不同源；`EditorCommandContext` 无谓词体系（when 缺）；`menu_bar_model` 不消费 `menu_items` 贡献表；keymap 无双层/冲突/域；无 `ToolScheduler`（05 模式栈只管视口，模态向导/独占面板无仲裁）；registry 住 `ui/host/` 使 headless 路径被迫拉起 ui host。

## 目标

1. **命令-操作合一**：`EditorCommandDescriptor` 吸收操作描述符全字段，单一 id 空间（`EditorOperationPath` 沿用为 id 类型）；`CommandAction::Operation` 从「链接外部 id」改为「内联 03 命令工厂」；`EditorOperationRegistry` 删除；CLI `--operation`/面板/菜单三入口同源。
2. **when 谓词**：结构化 `WhenClause` 枚举树；`command_palette_entries/menu_bar_model` 的 context 参数升级为求值环境（既有两方法签名保留语义升级）。
3. **菜单合流**：`menu_bar_model` 扩为消费「命令 menu_path + 06 `menu_items` 贡献 + `DocumentToolkit::contribute_menus`」三源合成（既有 MenuBarModel/MenuModel 产物类型保留）。
4. **keymap 双层**：内建预设（`from_toml` 沿用）+ 用户覆盖（17 settings）；`missing_default_keymap_bindings` 既有检查扩为冲突检测数据源（同 chord 同 when 域告警）；按 when 域分派。
5. **`ToolScheduler`**：独占资源仲裁（`ViewportInput/ModalSurface/SceneModeSlot`）；05 模式栈与 15 导出向导注册为受调度工具；生命周期事件入 bus。
6. **命令面板成型**：模糊匹配 + when 过滤 + MRU（外观归 editor_layout）。

## 非目标

- 宏录制/脚本化命令（依赖 13，远期）；面板视觉；输入底层（editor_ui/01）；能力体系本体（消费 01 `RuntimeCapabilities` 能力名）。

## 架构设计

### 模块布局

```
zircon_editor/src/core/commands/
  mod.rs
  descriptor.rs        # 合一 descriptor + CommandAction
  registry.rs          # 现 registry.rs 迁入（owner ui→core：headless 直用）
  when.rs              # WhenClause + CommandEvalCtx
  keymap.rs            # 现 keymap.rs 迁入 + 用户层
  menu.rs              # menu_bar_model 三源合成（MenuBarModel/MenuModel 类型迁入）
  palette.rs           # 面板数据源（模糊/MRU）
zircon_editor/src/core/tools/
  mod.rs / scheduler.rs
```

### 关键类型

```rust
// when.rs
pub enum WhenClause {
    Always,
    FocusedDocumentKind(DocumentKind),
    SceneModeActive(String),             // 05 模式 id
    SelectionNonEmpty,
    PlayMode(PlayModePredicate),         // 04 状态
    Capability(String),
    All(Vec<WhenClause>), Any(Vec<WhenClause>), Not(Box<WhenClause>),
}
impl WhenClause { pub fn eval(&self, ctx: &CommandEvalCtx) -> bool; }
// CommandEvalCtx 由 EditorContext 投影：焦点文档/模式栈顶/选中数/Play 态/能力集
// headless 求值环境：无文档无模式，仅能力集 + Always —— commandlet 门禁一致

// descriptor.rs
pub struct EditorCommandDescriptor {
    pub id: EditorOperationPath,
    pub display_name: String, pub category: String,
    pub menu_path: Option<String>,
    pub when: WhenClause,
    pub payload_schema_id: Option<String>,
    pub callable_from_remote: bool,
    pub required_capabilities: Vec<String>,   // 注册时折算为 when 的 Capability 合取
    pub default_chord: Option<EditorKeyChord>,
    pub action: CommandAction,
}
pub enum CommandAction {
    Emit(EditorMessagePayload),               // 纯消息（首选；01 类型化载荷）
    Operation(OperationCommandFactory),       // 内联 03 工厂（现 Operation(id) 链接升级）
    Menu(WorkbenchMenuAction),                // 既有 Menu 变体过渡保留，M2 菜单合流后清点收敛
}

// tools/scheduler.rs
pub enum ExclusiveResource { ViewportInput, ModalSurface, SceneModeSlot }
impl ToolScheduler {
    pub fn acquire(&mut self, tool: ToolId, res: ExclusiveResource) -> AcquireOutcome; // Acquired|Queued|Denied(holder)
    pub fn release(&mut self, tool: ToolId, res: ExclusiveResource);   // 队首唤醒
}
// 事件 ToolActivated/ToolDeactivated/ToolDenied 入 bus
```

### 三入口同源

| 入口 | 路径 | 过滤 |
| --- | --- | --- |
| 菜单/工具栏 | `menu.rs` 三源合成 → 点击发 action | when 置灰 |
| 命令面板 | `palette.rs` 模糊+MRU | when 隐藏 |
| CLI `--run/--operation` | 16 commandlet → `command(id)` | `callable_from_remote` 且 headless-when 通过 |

### 迁移映射（执行合同）

| 现物 | 去向 |
| --- | --- |
| `EditorCommandRegistry`（ui/host/commands） | `core/commands/registry.rs`（API 面保留：`command/commands/event_for_command/command_palette_entries/menu_bar_model/menu_model/missing_default_keymap_bindings` 签名不破坏，context 类型升级） |
| `EditorCommandAction::Operation(operation_id)` 字符串链接 | `Operation(OperationCommandFactory)` 内联；分派处 `event_for_operation_command` 删除 |
| `EditorOperationDescriptor/Registry` | 字段并入 descriptor；registry 删除；06 store 的 operations 位改存命令 id 集 |
| `EditorKeymap` | `core/commands/keymap.rs`；`missing_default_keymap_bindings` 扩冲突检测 |
| `EditorModule` 的两 manager 注册（`EDITOR_COMMAND_REGISTRY_NAME/EDITOR_KEYMAP_NAME`） | 指向 core 新位（名称常量与解析行为保持，00 §9 约定） |

### 深度测试

夹具功能域注册 5 命令（when/chord/menu_path/remote 各形态）：三入口全通、菜单含新项、面板过滤正确、CLI 可调——`core/commands/` 零改动。

## 里程碑

### M1 合一注册表与 when（排 01 M1 之后）

- 切片 1.1：`core/commands/` 迁入（ui/host/commands 删除，module.rs re-wire）；descriptor 合一；`EditorOperationRegistry` 消费方迁移删除（含 CLI `--list-operations` 改读命令注册表）。
- 切片 1.2：`WhenClause/CommandEvalCtx`；`required_capabilities` 折算；求值矩阵（All/Any/Not、Play 态变迁、能力缺失、headless 环境）。
- 测试阶段：`cargo test -p zircon_editor --lib --locked`（commands 既有测试迁移后须过 + id 冲突守卫沿用 + `--list-operations` 输出一致性断言）。更新 `docs/zircon_editor/core/commands.md`。

### M2 菜单合流与 keymap 双层

- 切片 2.1：`menu.rs` 三源合成（命令 menu_path + `menu_items` 贡献 + toolkit 菜单）；`CommandAction::Menu` 变体消费点清点收敛；主菜单硬编码项迁移清单（Grep 定稿）删除。
- 切片 2.2：keymap 用户层（17 settings）+ 冲突检测（`missing_default_keymap_bindings` 扩展）+ when 域分派（`resolve_keyboard_input` 升级为域感知）。
- 测试阶段：菜单三源合成快照测试（含 when 置灰、插件贡献项出现）；冲突矩阵（同 chord 同域告警/异域放行）；chord 端到端（输入事件夹具→命令消息）。

### M3 面板与 ToolScheduler

- 切片 3.1：`palette.rs`（模糊 + MRU 入 17 会话层）；`command_palette_value` UiValue 投影保留对接 editor_layout 呈现。
- 切片 3.2：`ToolScheduler` + 05 模式栈/15 导出向导注册；`ViewportInput` 争抢矩阵（模式激活期间向导启动→Queued）。
- 测试阶段：调度矩阵（acquire/release/queue/deny+事件序）；手验面板全链路（打开→搜索→执行→撤销）；证据记状态节。

## 风险与开放问题

- registry 迁 core 与 01 M1 context 服务化排程耦合——M1 硬排在 01 M1 后。
- `CommandAction::Emit` 依赖 01 类型化载荷已落地；01 未完时以 `Custom` 过渡记债。
- `CommandAction::Menu` 变体的存废：菜单合流后若全部菜单动作可折算 Emit/Operation 则删除该变体（M2 清点裁决记状态节）；不预判。
- when 谓词不给插件自定义（序列化限制）：插件组合内建谓词，不足场景走 `Capability` 自定义能力名兜底——契约注释声明。
