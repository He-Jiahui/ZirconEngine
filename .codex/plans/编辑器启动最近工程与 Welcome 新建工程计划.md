# 编辑器启动最近工程与 Welcome 新建工程计划

## Summary
- 固定采用 `Welcome 页内嵌现有 Slint workbench` 的方案，不做独立启动器，也不做“无工程时自动静默建默认工程”。
- 启动时默认读取并验证上次打开的工程；若工程有效则直接自动打开，若不存在、manifest 损坏或不再合法，则降级进入 Welcome 页并显示恢复提示，不阻塞启动。
- “无工程”不再靠 `sandbox-project + project_open bool` 伪装；改成显式 `EditorSessionMode::{Welcome, Project}`，Welcome 成为真正的一等启动状态。
- 新建工程固定为 `可渲染模板`：创建目录式项目、默认 scene/material/shader 和最小可打开内容，创建成功后立即打开。
- Welcome UI 参照 JetBrains 新建工程页，但保持 Zircon 架构约束：项目状态由 `EditorManager + ConfigManager + EditorProjectDocument` 驱动，UI 只消费 snapshot，不直接拼路径或做文件系统决策。

## Public Interfaces
- `zircon_editor` 新增启动会话模型：
  - `EditorSessionMode::{Welcome, Project}`
  - `EditorStartupSessionDocument`
  - `RecentProjectEntry`
  - `RecentProjectValidation::{Valid, Missing, InvalidManifest, InvalidProject}`
  - `WelcomeSnapshot`
  - `NewProjectDraft`
  - `NewProjectTemplate::RenderableEmpty`
- `EditorState` 去掉“未打开工程也总持有默认 world”的隐式约束，改为基于 `EditorSessionMode` 生成 snapshot；`DEFAULT_PROJECT_PATH` 不再作为真实启动输入。
- `EditorManager` 增加统一启动/工程入口：
  - `resolve_startup_session()`
  - `open_project_and_remember(path)`
  - `create_project_and_open(draft)`
  - `recent_projects_snapshot()`
  - `forget_recent_project(path)`
  - `update_recent_project(path, display_name)`
- `ConfigManager` 下固定使用单一结构化键：
  - `editor.startup.session`
  - 内容至少包含 `last_project_path` 和按最近打开时间排序的 `recent_projects`
- `EditorProjectDocument` 增加显式脚手架入口：
  - `create_renderable_template(root, project_name)`
  - 用它替代“靠 save/open 顺手补齐工程骨架”的隐式行为
- Workbench/UI 新增欢迎页内容模型：
  - `ViewContentKind::Welcome`
  - `WelcomePaneSnapshot`
  - `RecentProjectItemSnapshot`
  - `NewProjectFormSnapshot`
- `ProjectOverview` 保持“已打开工程后的项目概览”；Welcome 不复用 `ProjectOverviewPane`。

## Implementation Changes
- 启动与状态分层：
  - `SlintEditorHost::new` 先向 `EditorManager` 请求启动决策，再决定进入 `Welcome` 还是 `Project`，而不是先创建默认 level。
  - 有效 `last_project_path` 直接走统一 `open_project_and_remember` 链路，成功后同步 world、asset catalog、editor workspace。
  - 无有效最近工程时进入 `Welcome` 模式；失效最近工程保留在 recent list 中并带诊断，不自动静默清除。
- 最近工程持久化：
  - recent list 只存规范化绝对路径、显示名、`last_opened_unix_ms`；有效性在启动时重新验证，不把瞬时错误写回配置当权威状态。
  - 成功打开或创建工程时更新 `last_project_path`，并对 recent list 做去重、按时间排序、截断到固定上限。
  - `File > Open Project`、Welcome recent 点击、Welcome 创建成功三条路径都必须汇入同一 remember 逻辑。
- Welcome 页与 JetBrains 风格交互：
  - Welcome 作为 workbench 内的独占主页面，默认隐藏场景工具抽屉，保留主壳体菜单和状态栏。
  - 左侧为 `Recent Projects` 列表，显示名称、路径、最近打开时间、失效标记；支持 `Open`、`Remove`，失效项额外支持 `Locate/Rebind`。
  - 右侧为 `New Project` 表单，字段固定为 `Project Name`、`Location`、只读 `Project Path Preview`、模板说明；主操作为 `Create Project`，次操作为 `Open Existing Project`。
  - v1 路径选择固定为文本输入主导；host 只预留可选 `Browse` 能力接口，UI 仅在 host 声明支持时显示按钮，不引入临时平台专用对话框分支。
- 新建工程脚手架：
  - `RenderableEmpty` 模板固定创建 `zircon-project.toml`、`assets/`、`library/`、默认 scene、默认 material、默认 shader，以及最小可打开内容。
  - Welcome 表单创建成功后立即调用统一 open 流程，并把 session 切到 `Project` 模式。
  - 创建目标目录必须是不存在或空目录；路径冲突、非法字符、目录不可写都在表单验证层给出结构化错误。
- 编辑器 presenter 与消息：
  - 新增 welcome 相关消息：创建草稿字段编辑、创建工程、打开 existing、打开 recent、移除 recent、重新定位失效工程。
  - `WorkbenchViewModel` 和 Slint presenter 直接消费 `WelcomeSnapshot`；不再依赖 `project_open == false` 时的空状态卡片来模拟启动页。
  - `Scene/Hierarchy/Assets` 在 `Welcome` 模式下不再背后持有假 world；它们的空状态由 session mode 派生。

## Test Plan
- 单元测试：
  - `editor.startup.session` roundtrip、recent list 去重、排序、截断。
  - 最近工程验证结果覆盖 `Valid/Missing/InvalidManifest/InvalidProject`。
  - `NewProjectDraft` 到最终项目路径推导与表单校验。
  - `EditorSessionMode` 下 snapshot 构建：`Welcome` 不要求 world，`Project` 恢复现有行为。
  - `create_renderable_template` 生成 manifest、assets、default scene/material/shader 的完整性。
- `zircon_editor` 集成测试：
  - 有效 `last_project_path` 启动时自动打开工程，并同步 editor workspace。
  - 失效 `last_project_path` 启动时进入 Welcome，recent list 保留条目并附带诊断。
  - Welcome 创建工程后立即打开，recent 和记忆路径同步更新。
  - `File > Open Project` 与 Welcome recent/open/create 共用同一 remember 链路。
- UI/presenter 测试：
  - `ViewContentKind::Welcome` 的 exclusive page 构建正确。
  - Welcome recent 列表、创建表单、错误提示、按钮可用性都能由 snapshot 重建。
  - 没有 host `Browse` 能力时不渲染临时浏览按钮；有能力时能显示标准入口。
- 验收命令：
  - `cargo test -p zircon_editor -p zircon_asset -p zircon_manager`
  - `cargo test --workspace --locked`
  - 增加一条 editor 启动回归：`cold start -> auto open last project / fallback to welcome -> create project -> open project`。

## Assumptions
- v1 不实现原生目录选择器，只把 host 目录选择能力做成可选接口；默认交付为表单输入和路径校验。
- Welcome 页是新的独立内容类型，不复用现有 `ProjectOverviewPane`。
- recent list 默认保留有限条目并按最近打开时间排序；失效条目默认不自动删除。
- 新建工程模板固定为“可渲染空工程”，不是示例工程，也不是仅 manifest 的极简骨架。
- 这是一次有意的状态模型替换；`sandbox-project` 和“未打开工程但持有默认 world”的兼容路径不保留为长期主链。
