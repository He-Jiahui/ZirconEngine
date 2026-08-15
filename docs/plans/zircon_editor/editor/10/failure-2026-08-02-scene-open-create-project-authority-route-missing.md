---
handoff_kind: failure
status: open
created_at: 2026-08-02
summary_slug: scene-open-create-project-authority-route-missing
origin_plan: docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
fixing_plan: docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
origin_child_dir: docs/plans/zircon_editor/editor/08
fixing_child_dir: docs/plans/zircon_editor/editor/10
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/project
  - zircon_editor/src/core/document
  - zircon_editor/src/core/commands
  - zircon_editor/src/ui/host/editor_event_execution/menu_action.rs
tests:
  - project_authority_scene_open_create_success_cancel_failure_matrix
  - editor_command_scene_open_create_routes_through_project_authority
  - cargo test -p zircon_editor --lib --locked
---

# Editor10: 场景打开与创建尚未路由到 ProjectAuthority

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md`
- 来源执行切片：MVP 文件/场景命令到 host effect 的执行链
- 修复责任计划：`docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md`
- 交接原因：场景目标的选择、工程根校验、资产标识、模板创建与已提交文档的打开必须由
  Editor10 `ProjectAuthority` 和 document authority 共同拥有；Editor08 只负责将命令和 UI 选择
  结果路由为 typed request，不能在菜单层猜测路径或维护第二份场景状态。

## 失败现象与复现证据

`zircon_editor/src/ui/host/editor_event_execution/menu_action.rs` 的
`MenuAction::OpenScene | MenuAction::CreateScene` 分支只设置
`"Scene open/create workflow is not wired yet"`，随后返回 `changed: false` 和纯
presentation/reflection effect。该路径既没有提交 `ProjectAuthority` 请求，也没有获得一个稳定的
scene asset target、创建目标或 document id，因此用户无法从菜单打开已有场景或创建可编辑场景。

这是一条可达的 MVP 断链：工程已打开、资产工作区可见且场景编辑器可用时，菜单仍以状态文本代替
实际行为。当前菜单 action 是无参数枚举，不能安全地隐式选择“最近场景”或任意文件，也不能把
`EditorState` 内存 world 当成持久化场景身份。

## 最低共享层根因

ProjectAuthority 已拥有项目打开、创建、模板和受限路径的权威边界，但没有公开一个由 command
routing 消费的 typed scene open/create request。Editor08 因此保留了无输入的菜单分支，无法把
picker/选中资产/创建目标转换为 ProjectAuthority 能验证并原子提交的请求。

## 架构修复验收

- Editor10 提供 typed scene-open 与 scene-create request/result，目标以受工程根约束的稳定资产
  identity 表示；打开、创建、模板写入、冲突检查和失败分类均在 ProjectAuthority/document
  authority 内完成。
- Editor08 菜单/命令层只发起选择或创建意图；取得用户选择后提交 typed request。取消、工程未打开、
  越界路径、缺失资产和名称冲突均不得改变当前 document、scene world 或最近工程记录。
- 成功打开或创建只经已提交的 document lifecycle 安装 authoring 场景，并由既有 typed document
  message producer 发布事实；不得在 UI menu handler 直接替换 `EditorState` world、selection 或
  persistence 路径。
- 覆盖 success/cancel/failure/no-op 精确状态矩阵，以及打开既有场景、创建后立即打开、重复创建冲突、
  根路径逃逸和 command-to-authority 路由。上游 Editor08 命令可用性与 Editor05 场景编辑入口必须重跑。

## 禁止临时方案

- 不得保留状态行 no-op、自动选择最近/任意场景、UI 私有 `PathBuf` 协议或按菜单分支直接写盘。
- 不得以 clone 当前 authoring world、重置 selection 或创建第二个 scene/document registry 伪造打开。
- 不得通过兼容 action、silent fallback、测试专用直接调用或跳过 ProjectAuthority 的路径/模板校验
  宣称完成。

## 修复结果与回传

Open state: `source integrated / retained-host E2E and managed validation pending`; no pass is
claimed. ProjectAuthority 的 typed scene open/create 合同、document authority 接线和 Editor08
菜单 picker intent 已落地；尚须冻结 retained-host 精确输入，复放真实菜单到 picker 到已提交 document
的 success/cancel/failure 矩阵，并执行受管 Cargo。

## 产出记录与时间

| 时间 | 状态 | 完成项目与证据 |
|---|---|---|
| 2026-08-02 CST | `OPEN / 已路由` | 已实读可达 `MenuAction::OpenScene | CreateScene` 分支：只写未接线状态文本并返回 `changed: false`，无 project/document request。按最低共享 owner 路由 Editor10 ProjectAuthority；未修改菜单、场景或文档生产代码，未运行 Cargo。 |
| 2026-08-05 CST | `source_integrated_static_audited / retained_host_e2e_and_managed_validation_pending` | 旧 no-op 文本已从 editor source 删除。Editor08 menu action 只发出 Open/Create Scene Picker effect；retained picker 保存 project-session `ScenePickerTicket`、拒绝隐藏/过期的选择并将 typed `SceneOpenRequest`/`SceneCreateRequest` 提交至 host。`EditorManager` 通过 `SceneDocumentRoute` 调用 ProjectAuthority，唯一 `EditorStateSceneInstaller` 先创建 runtime level 再替换 authoring world，成功后才发布 scene inspection resync 与 refresh。`scene_route_tests` 覆盖 ticket 失效、取消、冲突、安装失败和 catalog 回滚，callback-runtime 回归覆盖菜单 effect。retained-host `app/tests` 当前属于并行 dirty 输入，尚未补写或宣称 E2E 通过；待冻结后的真实 success/cancel/failure 矩阵与受管 Cargo，failure 保持 open。 |
