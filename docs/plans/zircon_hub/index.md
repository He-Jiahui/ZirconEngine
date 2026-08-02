---
related_code:
  - zircon_hub/src/tauri_app/commands.rs
  - zircon_hub/src/tauri_app/action_request.rs
  - zircon_hub/src/tauri_app/runtime_state.rs
  - zircon_hub/src/tauri_app/view_model.rs
  - zircon_hub/src/state/hub_snapshot.rs
  - zircon_hub/src/projects
  - zircon_hub/src/settings
  - zircon_hub/src/engines
  - zircon_hub/web/src
  - zircon_hub/tests
plan_sources:
  - .codex/plans/Zircon Hub 本地闭环 v1 功能实现设计.md
  - .codex/plans/Zircon Hub Tauri + ReactMUI 硬切换计划.md
  - docs/zircon_hub/index.md
  - docs/zircon_hub/ui/tauri-react-shell.md
  - docs/zircon_hub/ui/responsive-component-system.md
  - docs/zircon_hub/pages/actionable-pages.md
  - docs/zircon_hub/pages/settings-status.md
---

# Zircon Hub 本地闭环 v1 架构稳定与功能完善总体计划

本目录承接 `.codex/plans/Zircon Hub 本地闭环 v1 功能实现设计.md`：该设计的功能骨架（Tauri command → action 路由 → 后台任务 → `hub-state-changed` 全量 ViewModel 推送，Rust 侧 `HubTextBundle` 双语文案，43 个契约测试）已经落仓，但按 2026-06-12 实仓审计，仍存在 stringly-typed 分发、持久化无并发保护、生命周期无事务回滚、视觉 fixture 混入生产判定、前端组件化不足与布局缺陷等问题。本计划把"稳定架构、完善逻辑、减少耦合与临时代码、布局对齐参考图"落成可执行子计划。

## 1. 现状审计结论（2026-06-12 实仓核对）

### 1.1 已收敛项（不再重复做）

- 单向数据流已成形：`hub_action` command → `HubRuntimeSession::apply_action()` → `HubActionRequest::action()` / `parse_as(action_id)` → `snapshot()` → `HubViewModel::from_snapshot()` → `app.emit("hub-state-changed")`。
- 后台任务队列已存在：`runtime_state/action_tasks.rs` 的 `VecDeque` FIFO + `background_worker_active` 单工标志，build/package/install/open-editor 不阻塞 command 线程。
- 本地化骨架完整：`view_model/localized.rs`（`HubTextBundle`，中/英）+ `view_model/ui_text.rs`（947 行结构化 UI 文案），前端只消费 DTO 字符串，无 i18n 库依赖。
- 前端统一分发器：`web/src/tauri/hubApi.ts` 的 `dispatchHubAction(actionId, targetId, payload)` 为唯一 IPC 出口，`App.tsx` 以 `actionSequenceRef`/`stateGenerationRef` 防竞态。
- 43 个契约测试覆盖 shell/页面/项目生命周期/action 路由/本地化/所有权边界，且全部面向 Tauri/React/MUI，零 Slint 残留。

### 1.2 问题清单（本计划的工作对象）

| # | 问题 | 证据 | 子计划 |
|---|------|------|--------|
| P1 | action id 双处字符串匹配：`parse()` 30+ 个 `match self.action_id.trim()` 分支，后台分发又在 `commands.rs:73-88` 重复 `action_id.trim() == "build-project"` 等四处比对；拼写错误无编译期检查 | `action_request.rs:236-344`、`commands.rs:73-88` | 01 |
| P2 | payload 为 `Option<serde_json::Value>`，按 action 临时反序列化，无统一校验层；前端 `HUB_ACTION` 常量与 Rust 字符串靠人工同步 | `action_request.rs:10-17`、`web/src/types/hub.ts` | 01 |
| P3 | 四个 `run_background_*_action` 函数共享同一 prepare→run→complete→emit_and_continue 骨架，约 210 行重复 | `commands.rs:106-320` | 02 |
| P4 | 持久化无并发保护与原子性：`persist_hub_config` 类调用散落在各 `*_actions.rs`，后台线程与主线程可并发写 `hub.toml`；写入非 tmp+rename 原子方式 | `runtime_state/*.rs` 各持久化点 | 02 |
| P5 | 交付链路无清理/回滚：`package.rs` 拷贝中断残留半成品目录；`device_install.rs` 的 `exists()`→`create_dir_all()` 存在 TOCTOU；create-project 在目录已建成后若记录失败则项目对 Hub 不可见 | `projects/package.rs`、`projects/device_install.rs`、`runtime_state/project_actions.rs` | 02 / 03 |
| P6 | 视觉 fixture 项目名与 `C:/ZirconProjects/` 路径前缀硬编码进生产过滤逻辑 | `state/hub_snapshot.rs:80-96` | 03 |
| P7 | 历史问题（实现已解决）：本地化 detail 曾依赖 23 处英文 `strip_prefix`；当前已由结构化 `HubMessage` / `HubMessageId` 双语模板取代 | `state/hub_message/`；受管 Hub07 failure return 仍待完成 | 07 |
| P8 | 巨型文件：`view_model.rs` 1357 行、`runtime_state.rs` 1076、`project_actions.rs` 966、`ui_text.rs` 947、`project_delivery_actions.rs` 832、`settings_dto.rs` 806 | `wc -l` 实测 | 01 / 02 / 07 |
| P9 | 前端 fallback mock `hubData.ts` 995 行，内嵌完整假项目、硬编码相对时间（"修改于 2 小时前"）与全量中文文案，易被当作真实后端数据 | `web/src/data/hubData.ts` | 05 |
| P10 | 前端无运行时 DTO 校验（`invoke<HubShellState>` 直接断言）、无 ErrorBoundary、action 失败仅写 taskSummary 无重试 | `web/src/tauri/hubApi.ts:19,34`、`web/src/App.tsx:103-124` | 05 |
| P11 | 页面路由为 7 分支 if-else 链；`ProjectsDashboard`（341 行）内联搜索栏与新建项目对话框；`SettingsPage`（304 行）单文件无分节组件 | `web/src/components/shell/HubWindow.tsx:49-65`、各 pages | 05 |
| P12 | 布局缺陷：卡片网格断点 1360px 处列数跳变、TopBar `minmax(250px, 1fr)` 窄屏溢出、`ProjectTable` 无横向滚动容器、个别硬编码色值/圆角绕过 tokens | `web/src/pages/ProjectsDashboard.tsx`、`web/src/components/shell/TopBar.tsx`、`web/src/components/data/ProjectTable.tsx` | 06 |
| P13 | settings 草稿与已存设置的生命周期边界（draft→validate→save→cancel→restore-defaults）未完全成形；folder picker 取消与错误未区分 | `runtime_state/settings_actions.rs` | 04 |
| P14 | 实现已修复、Rust gate 待补：Source Engine 已深验 workspace/runtime/build tool，open-editor 已增加 active engine 预检 | `engines/validation.rs`、`runtime_state/editor_launch_actions.rs`；聚焦测试受 Runtime11 CPU reservation 阻塞 | 04 |

## 2. 子计划地图与执行顺序

> 2026-08-01 实仓复核：7 个子计划均已有当前源码实现或修复切片，状态统一按各子计划 front matter 更新为 `in_progress`。这只表示正在收敛和验收，不代表里程碑已经完成；各计划的剩余 gate 与 open failure 仍以子计划正文为准。

| 计划 | 文档 | 依赖 | 状态 |
|------|------|------|------|
| 01 action 分发与 payload 类型化 | `01-action-dispatch-and-typed-payload.md` | 无 | in_progress |
| 02 后台任务框架与持久化一致性 | `02-background-task-framework-and-persistence.md` | 01（typed action 判定） | in_progress |
| 03 项目生命周期健壮性 | `03-project-lifecycle-robustness.md` | 02（事务/清理基建） | in_progress |
| 04 settings 草稿生命周期与 Source Engine 校验 | `04-settings-draft-and-source-engine.md` | 可与 03 并行 | in_progress |
| 05 前端组件化与类型安全 | `05-frontend-componentization-and-type-safety.md` | 01（action 契约定稿） | in_progress |
| 06 布局与视觉标准对齐 | `06-layout-and-visual-standard.md` | 05（组件拆分先行） | in_progress |
| 07 本地化 schema 与"敬请期待"能力目录 | `07-localization-schema-and-coming-soon.md` | 02 / 03（消息产生点收敛后） | in_progress |

阶段划分：

- 阶段 A（后端地基）：01 → 02。先把 action id / payload 收敛为单一类型来源，再以此重建后台任务执行框架与持久化纪律。
- 阶段 B（正确性）：03 与 04 并行。生命周期事务化、fixture 剥离、settings/engine 校验。
- 阶段 C（前端）：05 → 06。组件拆分与类型安全先行，布局与参考图对齐殿后（避免在待拆组件上做样式微调被重做）。
- 阶段 D（口径收口）：07。消息 schema 化必须等 02/03 把消息产生点收敛后做，coming-soon 目录可提前并行。

## 代码结构规范

Hub 同样遵守引擎级 [`engine-code-structure-convention.md`](../engine-code-structure-convention.md)：

- 巨型文件（P8：`tauri_app/view_model.rs`(1357)、`runtime_state.rs`(1076)、`project_actions.rs`(966)、`ui_text.rs`(947)、`project_delivery_actions.rs`(832)、`settings_dto.rs`(806)）按规范 §1（owner 叶子拆分、行数预算软 800 / 硬 1000）治理，与 `large-file-ownership-m1.md` 的 `support-hub`(3) owner 类联动；拆分进度计入相关子计划（01/02/07）。
- 前端组件化（P11：`HubWindow.tsx` 7 分支 if-else、`ProjectsDashboard.tsx`(341)、`SettingsPage.tsx`(304) 内联）按规范 §1/§4 的 owner 边界 + 单一测试 owner 治理（计划 05/06）。
- façade/命名/测试组织遵循规范 §1–§4；`tauri_app` 模块去 `_inner`/`util` 类无主名（规范 §2）。

## 3. 全局边界约束（各子计划必须遵守）

1. 所有 Hub 功能留在 `zircon_hub`：不把 Hub 注册进 `zircon_runtime` 生命周期，不新增非网络语义的 `server` 命名层，不向 `zircon_editor` 回引 Slint 路径。
2. 硬切换纪律：新 owner 路径落地的同一变更内迁移调用方并删除旧路径；不留兼容 `pub use`、shim、迁移期双轨。
3. UI 业务文案全部由 Rust DTO 边界所有（`localized.rs` / `ui_text.rs` / `settings_dto.rs`）；React 侧不得新增硬编码业务文案，默认语言中文。
4. project target 解析顺序保持 `projectPath` > 稳定 `projectId` > legacy `targetId`；仅 dashboard 型快捷操作允许 latest-recent 回退，selected-project 动作不得改目标。
5. `update-settings-draft` 只改 draft 并重算 health，不 persist；只有 `save-settings` 才持久化 `hub.toml`、注册 Source Engine、刷新 catalogs。
6. 只读控件语义不破坏：`HubCheckbox`/`HubSwitch` 缺 `onChange`、`SourceEngineList`/`HubList` 缺 `onSelect` 时必须呈只读态。
7. 43 个契约测试是结构守卫：任何重构在同一变更内刷新对应契约断言，不得绕过、不得整体削弱断言面；`hub_docs_contract` 要求的 `docs/zircon_hub/*.md` machine-readable header 在文档刷新时保持。
8. 前端依赖克制：v1 不引入路由库、状态库、i18n 库、表单库；如确需运行时校验等第三方依赖，先在子计划中记录决策再引入。
9. Learn `open-resource` 只允许打开当前 catalog 内真实存在的文件；远程账号/云同步/插件市场/团队权限一律 disabled +"敬请期待"。

## 4. 全局验收与测试基线

按 [`milestone-validation-policy.md`](../milestone-validation-policy.md) 执行：切片期只做格式、类型/源码静态检查与已知失败的聚焦回归；里程碑末将 Rust、前端和必要集成检查合并为一次测试阶段。

- Rust 里程碑：通过 `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_hub` 合并执行受管 build/test；切片期对触及的 Rust 文件运行 `rustfmt --edition 2021 --check`。
- 前端：在同一里程碑阶段运行一次 `npm run typecheck` 与 `npm run build`（`zircon_hub/` 下）。
- 集成：先通过上述受管验证器完成 Rust 构建，再运行协调器产出的 Hub 二进制或 staged payload；不得用 `npm run tauri:dev` / `tools/zircon_build.py` 绕过共享 Cargo reservation。
- 视觉验收（06 计划详述）：仅在改变页面布局、文案或组件行为的里程碑运行 Tauri Hub 截图矩阵——Projects / New Project / Project Detail / Editor / Builds / Cloud / Settings，覆盖中文默认、错误态、运行中、空态；确认无溢出、无遮挡、无英文硬编码残留。

## 5. 协调与避让

- 仓库存在大规模未提交工作（git status 数十个 M 文件）：实施时视为当前基线，不回滚无关改动，只做聚焦编辑。
- `.codex/plans/` 下另有《Zircon Hub 响应式组件化重构计划》《Zircon Hub Flex 组件化重构设计方案》：05/06 执行前复读，避免与其已完成切片双写。
- Build/Package/Install 操作耗时长且写共享 `CARGO_TARGET_DIR`：测试阶段避免与其他重型 Cargo 构建并行。

## Code Review 处理结果 (2026-08-01)

### 已处理

- 子计划 01-07 的索引状态与 front matter 已统一为 `in_progress`。虽然主要 owner 和多项行为已经落地，但受管 Rust gate、截图矩阵与 open failure 尚未全部关闭，因此不采用旧建议中的 `completed`，避免把源码存在误报成产品验收完成。
- P7 已标为实现解决；P14 已补齐 deep validation 与 open-editor preflight，并明确记录受管 Rust 聚焦测试尚未执行。

### 仍开放

- `07/failure-2026-07-11-hub-message-legacy-test-drift.md` 的源码根因已消失，`project_management_contract.rs` 已使用 `HubMessage::raw_text("opened")`；但受管聚焦测试被 Runtime11 reservation 拒绝。该 failure 在测试通过并执行 canonical return 前继续保持 open，不以静态源码命中代替修复闭环。
