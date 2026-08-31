---
title: Editor Project Operations、Source Control、Changelist、Diff、Automation、Validation、Submission 与 Health 当前源码复核
category: zircon_editor
report_id: Editor101
review_date: 2026-08-26
baseline_head: 3282dfad2a3a0dce246dfa8f300d7d30d70ed9a9
baseline_epoch: 524
canonical_owner: Editor27
refreshes:
  - docs/plans/optimize/zircon_editor/27-project-operations-source-control-changelist-diff-automation-report-submit-gates-health-dashboard-review.md
  - docs/plans/optimize/zircon_editor/85-editor-project-operations-source-control-provider-workspace-changelist-diff-automation-validation-submission-gate-health-dashboard-product-integration-current-source-review.md
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/production
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/index/workbench_extension_module_workspaces.zui
  - zircon_editor/src/ui/retained_host/workbench_preview_actions
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_field_edit.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings
  - zircon_editor/assets/ui/editor/project_overview.zui
  - zircon_editor/src/ui/layouts/views/project_overview.rs
  - zircon_editor/src/ui/workbench/project/asset_workspace_state.rs
  - zircon_editor/src/ui/workbench/snapshot/data/project_overview_snapshot.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions
  - zircon_editor/src/ui/retained_host/app/build_export_projection.rs
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection
  - zircon_editor/src/ui/host/editor_manager_plugins_export
  - zircon_editor/src/core/commandlet
  - zircon_editor/src/ui/retained_host/app/automation.rs
  - zircon_app/src/entry/entry_runner/editor/project_automation.rs
  - zircon_runtime/src/asset/project/manifest/project_manifest.rs
tests:
  - zircon_editor/src/core/commandlet/tests.rs
  - zircon_editor/src/ui/retained_host/app/tests/retained_host_automation.rs
  - zircon_app/src/entry/entry_runner/editor/project_automation.rs
  - zircon_editor/src/tests/ui/project_overview
plan_sources:
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/10-notification-center-toast-decision-history-actions-retention-accessibility-diagnostic-integration-review.md
  - docs/plans/optimize/zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md
doc_type: current_source_refresh
review_status: complete
implementation_status: pending
source_recheck_required: true
finding_status:
  p0: 5 open
  p1: 60 open
  p2: 12 open
gate_status:
  fail: 30
  partial: 1
  pass: 1
---

# Editor27/101 · Project Operations、Source Control、Changelist/Diff、Automation、Validation、Submission 与 Health 当前源码复核

## 1. 结论

当前 Editor 同时维护两条成熟度不同的产品链。第一条是真实但范围很窄的底座：内置 Project Overview 从 `AssetWorkspaceState` 投影项目名、project/assets/cache root、default scene、catalog revision、folder count 与 asset count；Build Export 已有 target projection、job queue、progress、cancel 和 output；Plugin Manager 已有 project/native status report、selection policy 与 live-host action；`authoring-automation` commandlet 通过正常 `EditorApplicationComposition` 和 retained-host callback 执行 selection、Inspector、Save、Undo/Redo，并输出项目/manifest/scene identity、inspection generation、event record 与 scene snapshot。这些路径应保留并成为未来证据提供者。

第二条是仍被注册为 `production` 的五张 Workbench Extension Workspace。Source Control 固定显示 `CL_2048`、18 files、2 conflicts、6 checks；Automation Report 固定显示 642 tests、7 failed、3 flakes、Worker_03 和 Screenshot diff；Project Overview 固定显示 `NebulaGame`、`Healthy`、72% coverage、7 tasks、3 build jobs，并允许直接编辑 Health；Build Export 与 Plugin Manager 也各自显示固定 cook/CDN 或 installed/update/warning。它们被 extension index 和 Assets Workspace 直接收录，不是未使用的设计稿。

五张页面各自约 230 行 ZUI、19 条 route、22 个 event 声明，`data_production.rs` 返回固定 feedback，`module_field_edit.rs` 只修改控件的 `value`/`value_text`，`render_asset_vfx.rs` 负责安装 binding。没有 repository、provider、revision、test attempt、artifact 或 admission authority。Source Control 即使展示 2 conflicts 仍会返回 `Submit queued`；Automation 也没有读取真实 commandlet 或 Tooling10 result；Project Overview 将派生健康状态变成可编辑字符串。这是 truthfulness、数据丢失和错误提交门的 P0，而不是“后端稍后补齐”的 P1。

当前重判保持 Editor27/85 的 finding 编号：5 个 P0 全部 Open，60 个 P1 全部 Open，12 个 P2 全部 Open；32 个验收门为 30 Fail、1 Partial、1 Pass。唯一 Pass 是 authoring automation 继续经过 retained-host callback/journal，没有 direct dispatch 旁路；Partial 来自已有真实 Project/Asset/Plugin/Build/Job 投影，不能证明 VCS、Validation Control Plane、Submission Gate 或 Health Dashboard 已交付。

本报告只做 Editor 当前源码复核和重构计划。Tooling10/03/09 只作为外部 owner 边界被引用，不在本报告修改或复制 runner、build、release 实现；本轮不运行 Cargo、Editor、repository checkout/submit、worker、artifact publication 或 release admission。

## 2. 逐层源码证据

### 2.1 冻结范围与可复算统计

统计口径：递归展开 frontmatter 的 `related_code` 与 `tests`，路径转小写正斜杠后排序；逐文件 SHA-256，再按 `path + NUL + lowercase hash + LF` 计算集合 fingerprint。测试数是 Rust/常见 automation attribute 的静态声明计数，不是通过数。

| 范围 | 文件 | 行 | 非空行 | bytes | 静态 test 声明 | fingerprint |
|---|---:|---:|---:|---:|---:|---|
| Zircon Editor/App/Runtime selected union | 195 | 30,302 | 28,220 | 1,169,737 | 216 | `3c1edd6e7d483cbd191b40d44161cf5ad31248d044ee4c6f8146f875e61d1de6` |
| Unreal/Godot/Automation/SourceControl reference set | 28 | 13,483 | 11,652 | 473,967 | 0 | `238301807e7f0f17b2aecf1b4965b16e6788aec8f6364f6408a11a7738c7ed2b` |

关键物理事实：五张 production ZUI 各为 230 或 231 行；扩展总索引为 506 行；production navigation data 为 615 行；template binding 为 789 行。真实 `ProjectOverviewSnapshot` 只有 8 个字段，`ProjectManifest` 只有 project/asset/plugin/script/export profile 字段；二者都没有 repository/provider/workspace revision、validation set、build artifact 或 release policy。

### 2.2 五张 production workspace 的数据流

| Surface | 当前固定业务事实 | 当前动作路径 | 结论 |
|---|---|---|---|
| Source Control | `CL_2048`、4 个文件行、2 conflicts、18 files、6 checks | navigation spec -> preview binding -> `data_production::feedback` | 无 provider、workspace revision、diff/changelist state 或 submit receipt；conflict 仍可 queued |
| Automation Report | Rendering/Smoke/Gameplay、642 tests、7 failures、3 flakes、Worker_03、Screenshot diff | 同上；Validate/Publish 只改 output row 文本 | 无 TestPlan/Attempt/CaseResult/Artifact/flake policy；不消费 authoring commandlet 或 Tooling10 |
| Project Overview | `NebulaGame`、M3、Healthy、Win64 Development、72%、7 tasks、3 jobs | Health/Owner/Channel 字段可编辑，Refresh/Publish 固定反馈 | 派生健康被当作手填事实；与真实 Project Overview 重复且断开 |
| Build Export | Cook Content 62%、Publish CDN Pending | 与真实 export job queue 并行的 preview route | 重复 Tooling03/真实 Build Export，不共享 job/artifact provenance |
| Plugin Manager | Audio Runtime、18 installed、3 updates、1 warning | 与真实 Plugin status/live-host action 并行的 preview route | 重复 Editor06，不共享 plugin set generation/diagnostic report |

`EXTENSION_MODULE_WORKSPACE_CONTROLS` 和 `EXTENSION_MODULE_NAVIGATION_SPECS` 把五个 workspace 注册成正式入口；Assets Workspace 另有同名 open route。`module_field_edit.rs` 的唯一领域写入是把用户输入写回控件属性并刷新 template surface，未调用 project authority、document transaction、job admission、command registry 或 journal。当前 exact contract 搜索中 `SourceControlProvider`、`RepositoryIdentity`、`WorkspaceRevision`、`SubmissionCandidate`、`SubmissionAdmissionReceipt`、`ProjectOperationsSnapshot`、`ValidationSet` 和 `TestAttempt` 均为零命中。

### 2.3 真实基础的边界

1. `ProjectOverviewSnapshot` 是真实的 project/catalog 投影，但只提供项目路径和 asset 计数，不能冒充 Operations Snapshot；它没有 owner、generation、produced_at、expires_at 或 stale reason。
2. `AssetWorkspaceState` 有 catalog/resource generation、selection reconcile、增量 patch 与缓存，适合作为 Editor04 的 source，但当前 Project Overview 没有引用失败 import、reference error、document dirty 或 artifact readiness。
3. Build Export 的 job queue 已有 phase、progress、cancel requested、poll、output capture 和 focused tests；它应作为 Tooling03/Editor Build provider 的 receipt source，不应被五张静态页面再造一套 62% 状态。
4. Plugin Manager 已有 `EditorPluginStatusReport`、project/native status、selection policy、live-host dispatch 和 diagnostics；它应作为 Editor06 的 generation-qualified source，不应从 fixture 派生 installed/update 数。
5. `AuthoringAutomationCommandletRequest` 只携带 project root 与 automation path；request 的 binding sequence 会经过正常 retained-host dispatch，report 携带 project/manifest/scene identity、inspection generation、records 和最终 scene snapshot。这是窄域 authoring evidence，不是全局测试控制面。
6. commandlet 没有 schema version、attempt ID、source revision、build/toolchain/environment identity、worker lease、deadline/cancel token、deterministic seed/clock、per-binding duration 或 artifact manifest，不能直接满足 Tooling10 的 `TestAttemptReceipt`。
7. `ProjectManifest` 不应塞入 credential；repository/workspace identity 应属于 Editor project session，credential 应由 `zircon_app` secure broker 提供，manifest 只引用不敏感的 provider configuration。

## 3. 参考引擎对照

- Unreal `ISourceControlProvider`、`ISourceControlState`、`ISourceControlOperation`、`ISourceControlChangelistState` 和 `SourceControlOperations` 将 provider lifecycle/capability、file/changelist state、异步 operation、cancel、history、shelve/unshelve、workspace 与 state-change event 分开；Zircon 需要 provider-neutral SPI，不应复制 Git-only API。
- Unreal `IAutomationControllerManager` 与 `IAutomationReport` 提供 worker discovery、cluster/device、run/stop/tick、pass/filter、duration、skip/exclusion、report/result/artifact 和 export；Automation UI 应消费这类不可变结果，不应把固定 Worker_03 变成 authority。
- Godot `EditorVCSInterface` 和 `VersionControlEditorPlugin` 是可用 Editor VCS 的下限：typed status、staged/unstaged diff、hunk/line、stage/unstage/discard、commit/history/branch/remote/pull/push/fetch、amend 与危险操作确认。Zircon 还需要大型团队的 capability、lock、changelist、lease、binary asset diff 和 generation contract。
- Bevy/Fyrox/Unity Graphics 参考没有同级的一体化 Editor VCS；它们的资产 source、build report、async validation、engine stats 与 CI validation 可用于 artifact/provenance 设计，不能被误读成已存在的 Zircon Editor 产品。

## 4. 必须保留的 owner 边界

| Owner | 真实职责 | Editor27 允许消费的证据 |
|---|---|---|
| Editor02 | saved document generation、dirty/history/save/recovery/conflict | `SavedWorkspaceSnapshot` 与 save barrier |
| Editor04 | asset catalog/import/reference/thumbnail | catalog generation、failed/stale/importing/reference snapshot |
| Editor06 | plugin discovery/enable/reload/diagnostics | resolved plugin set、manifest generation、reload/restart status |
| Editor08/09/10/11 | command、job、notification、journal、operation receipt | typed dispatch、progress/cancel、terminal receipt、audit correlation |
| Editor27 | VCS SPI、workspace/diff/changelist UI、Operations snapshot、candidate/admission projection | provider registry、source snapshot、candidate、admission receipt；不拥有下游测试/build/release算法 |
| Tooling03 | build/cook/package/sign/artifact | immutable `BuildArtifactSet` |
| Tooling09 | channel/promotion/rollback policy | release admission/promotion receipt |
| Tooling10 | test plan/attempt/result/artifact/validation | immutable `ValidationSet` |
| `zircon_app` | process/credential broker/provider executable/native loading | secure credential lease、provider process lifecycle |

## 5. P0：先关闭假产品与危险命令

| ID | 当前源码证据 | 必须重构 |
|---|---|---|
| P0-1 | Source Control 的 CL、文件和 checks 全为 ZUI/fixed feedback；2 conflicts 仍返回 queued submit | 立即禁用 Validate/Submit 或标记 Unavailable；先完成 provider state、expected revision、conflict gate、operation receipt |
| P0-2 | Automation 的 642/7/3、worker、screenshot diff 无 result owner | 删除固定事实和 Publish 成功语义；UI 只能投影 Tooling10 `ValidationSet` 或带 provenance 的 authoring attempt |
| P0-3 | Project Overview 的 Healthy/72%/7 tasks/3 jobs 可编辑 | 健康由 versioned policy 对证据派生；删除可编辑健康字段和静态 build/test/coverage |
| P0-4 | 五张页面与真实 Project/Build/Plugin 路径并存，100 个 binding 落入 generic preview mutation | 建立唯一 owner map；重复入口改为只读 projection 或硬切删除，禁止继续增加 fixture 分支 |
| P0-5 | 没有 saved document、source change-set、validation、artifact、policy 的不可分割提交候选 | 实现 `SubmissionCandidate -> AdmissionReceipt -> ProviderReceipt`，任何输入 generation 变化都 fail closed |

## 6. P1：Source Control SPI、Workspace、Diff、Changelist

| ID | 差异与重构要求 | ID | 差异与重构要求 |
|---|---|---|---|
| P1-1 | provider registry、availability、auth、settings、capability、owner lease | P1-2 | repository/workspace/root/remote/branch/stream identity |
| P1-3 | tracked/untracked/ignored/rename/conflict/staged/lock 状态带 observed generation | P1-4 | Connect/Refresh/Stage/Revert/Resolve/Submit 走 Editor09 job、cancel/deadline/receipt |
| P1-5 | revision/history/base/local/remote content 读取与 large/binary streaming budget | P1-6 | typed file/hunk/line diff、range、encoding、binary/truncated 状态 |
| P1-7 | rename/copy/submodule/case-only path 语义与历史保留 | P1-8 | Git index、Perforce changelist、uncontrolled staging 的 capability abstraction |
| P1-9 | checkout/lock/ownership 区分 self/other/soft/exclusive | P1-10 | content/rename/delete/binary conflict state machine 与 resolve evidence |
| P1-11 | Editor04/authoring owner 提供 semantic asset diff，binary fallback 显式 | P1-12 | filesystem watcher、provider refresh、catalog/document invalidation 与 overflow rescan |
| P1-13 | path/status/diff/history/watch/UI rows 的 CPU/I/O/bytes/page/cancel/truncation budget | P1-14 | credential 只能经 secure lease，不进 manifest/settings/receipt/journal |
| P1-15 | 每个 adapter 的 fake repo、offline/auth-expiry/head-race/partial/cancel/large-diff conformance suite | P1-16 | Automation UI 从 versioned Tooling10 TestPlan 解析 suite/platform/retry/selection |
| P1-17 | immutable attempt ID、parent plan、ordinal、worker、terminal state、superseded | P1-18 | worker discovery/capability/lease/health/capacity；禁止字符串 Worker_03 |
| P1-19 | run/stop/cancel/pause 状态机由 Tooling10/Editor09 驱动 | P1-20 | deadline、heartbeat、lost worker、late result、retry identity |
| P1-21 | Pass/Fail/Skip/Disabled/Timeout/Crash/Infrastructure Error 分离 | P1-22 | artifact manifest/hash/media type/size/producer/retention/redaction/auth |
| P1-23 | visual regression 绑定 baseline/candidate/diff、viewport/DPI/color/GPU tolerance | P1-24 | flake 由 attempt history/policy 判定，保留 first failure、quarantine owner/expiry |
| P1-25 | coverage 绑定 instrumented build/source revision/path mapping/merge completeness | P1-26 | source/document/toolchain/environment 变化后结果 Stale/Superseded |
| P1-27 | authoring request 加 schema version、binding/bytes/deadline/cancel/seed/clock budget | P1-28 | authoring report 加 attempt/source/build/environment/duration/artifact/terminal reason |
| P1-29 | Automation Report 支持 plan/suite/case/state/tag/worker/revision 查询、分页、cursor | P1-30 | Publish/Export 具有 target/format/retention/redaction/hash/terminal receipt |

## 7. P1：Operations Snapshot、Health、Submission 与产品集成

| ID | 差异与重构要求 | ID | 差异与重构要求 |
|---|---|---|---|
| P1-31 | 建立只读 `ProjectOperationsSnapshot`，聚合 owner snapshot，不复制可变模型 | P1-32 | 绑定 project manifest identity、open generation、repository/workspace/revision |
| P1-33 | 每节显示 owner/generation/produced/expiry/stale reason | P1-34 | versioned health policy、digest、required signals、unknown/stale 规则 |
| P1-35 | 投影 dirty/autosave/recovery/conflict/last saved generation/save barrier | P1-36 | 投影 catalog/import/reference failure 与 artifact readiness |
| P1-37 | 投影 plugin set/manifest/reload/restart/diagnostics，不显示静态 18 installed | P1-38 | 投影 Tooling03 job/artifact，区分 queued/running/failed/cancelled/succeeded/stale |
| P1-39 | 投影 Tooling10 required/optional/missing/stale/failing validation lanes | P1-40 | 投影 Editor09 queued/blocked/cancel/resource admission 与真实 job navigation |
| P1-41 | 投影 Tooling09 channel/candidate/promotion/rollback；Channel 是权限操作 | P1-42 | 聚合 barrier 或显式 mixed-generation，patch 带 base/target generation |
| P1-43 | candidate 必须引用 Editor02 persisted document generation | P1-44 | 冻结 paths/content hashes/base/head/local revision/provider generation |
| P1-45 | 按 project/channel/change impact 解析 required ValidationSet 和 policy digest | P1-46 | result/build/plugin/toolchain/source/document identity 必须一致 |
| P1-47 | provider mutation 前纯函数评估 dirty/source/conflict/asset/plugin/test/build/permission | P1-48 | Submit 携 expected revision，head/local/document 变化 typed stale/conflict 失败 |
| P1-49 | Save/refresh/validate/build/stage/submit/publish 持久 step journal 与 compensation | P1-50 | override/approval 有 actor/scope/reason/expiry/approver/evidence audit |
| P1-51 | immutable submit/publish receipt 带 candidate/policy/provider ID、paths/hashes、evidence | P1-52 | crash/restart/idempotency key 恢复 in-flight operation，禁止重复提交 |
| P1-53 | 同名 fake/real navigation 归一到唯一 authority | P1-54 | 通过 Editor08 typed command descriptor/capability，不走 100 个 preview binding |
| P1-55 | notification/diagnostic/journal 关联 operation、attempt、provider、candidate ID | P1-56 | Offline/AuthExpired/Unavailable/Unknown/Stale/Degraded 必须分开呈现 |
| P1-57 | keyboard/accessibility、diff focus、dangerous scope confirmation 以真实 diff tree 为源 | P1-58 | credential/log/artifact redaction 和最小暴露策略 |
| P1-59 | repository/provider/worker/artifact fault、scale、soak、latency、memory qualification | P1-60 | fixture/schema migration 后 zero-reference hard-cutover 与禁止回退测试 |

## 8. P2：规模、协作与迁移能力

| ID | 当前差异 | 重构方向 |
|---|---|---|
| P2-1 | 无多 provider/mixed workspace | capability adapter composition 与冲突优先级 |
| P2-2 | 无 branch/stream/worktree transaction | workspace switch、lease、open document barrier |
| P2-3 | 无 sparse/partial clone、LFS、大 binary budget | storage/bandwidth provider capability 与 stream API |
| P2-4 | 无 semantic merge/binary asset conflict tool | Editor owner-specific merge participant |
| P2-5 | 无 review/change-request identity | provider-neutral external review link/receipt |
| P2-6 | 无 distributed worker/capacity scheduling | 由 Tooling10/06 提供 worker authority，Editor 只投影 |
| P2-7 | 无 validation/health trend history | immutable metric schema、retention 与 comparison cursor |
| P2-8 | 无 query/layout versioned operations view | saved query/layout schema，不再用 fixture field |
| P2-9 | 无 notification subscription/external delivery receipt | policy、redaction、retry、delivery evidence |
| P2-10 | 无 automation recording/locator migration/schema tooling | binding schema version、record/replay 与 migration |
| P2-11 | Alice/Bob/Chen/Owner 都是 fixture | authenticated actor/team/policy metadata |
| P2-12 | 无 provider/test/result import compatibility | versioned import/export contract 与 migration gate |

## 9. 32 个验收门重判

- **Source/identity gates（G1-G8）**：provider registry、repository/workspace identity、generation-qualified status、watcher refresh、typed diff、changelist/lock、credential boundary、adapter conformance，全部 Fail。
- **Automation/result gates（G9-G16）**：TestPlan/Attempt/worker lease、run/cancel/deadline、typed case result、artifact/visual diff、flake/coverage/currentness、authoring adapter，全部 Fail。
- **Snapshot/health gates（G17-G22）**：canonical aggregate、identity join、per-section provenance、policy-derived health、dirty/save projection、asset/plugin/build/test/job/release projection，全部 Fail。
- **Submission gates（G23-G29）**：saved precondition、source freeze、ValidationSet、artifact binding、preflight、expected revision、step journal/override/receipt/restart，全部 Fail。
- **Product governance gates（G30-G32）**：unique authority/typed command/diagnostic correlation、offline/redaction/accessibility/scale、fixture hard cutover，G30-G31 Fail，G32 Partial；authoring retained-host callback/journal gate Pass。汇总为 **30 Fail / 1 Partial / 1 Pass**。

## 10. 分层重构顺序

1. **Truthfulness cutover**：删除/隔离五张 production workspace 的固定业务 rows、fixed feedback、editable health 与 Submit/Publish queued；保留布局壳，但状态全部来自 `Unavailable` 投影。
2. **Editor27 SPI**：定义 provider registration、capability、repository/workspace identity、file state、typed diff、changelist/lock/conflict、operation request/receipt；由 `zircon_app` 提供 secure credential/process bridge。
3. **Generation spine**：为 Editor02/04/06/Tooling03/09/10/Editor09/11 建立统一 `OwnerSnapshotHeader { owner, generation, source_revision, produced_at, expires_at }`，实现 `ProjectOperationsSnapshot` 的 mixed-generation 明示与增量 resync。
4. **Automation adapter**：把 authoring commandlet report 转换为 Tooling10 `TestAttempt` participant，补 attempt/source/build/environment/deadline/artifact 字段；Automation UI 只读 ValidationSet。
5. **Submission transaction**：实现 saved barrier -> source change-set freeze -> required validation -> artifact/policy evaluation -> provider mutation -> immutable receipt；expected revision 和 idempotency key 缺一不可。
6. **Projection hard cutover**：Project Overview 成为聚合只读视图；Build Export、Plugin Manager、Automation、Source Control 只挂唯一 owner provider。完成 zero-reference 测试后删除 preview action、route、binding 和 fixed feedback。
7. **Qualification**：fake provider、offline/auth-expired、head race、binary/large diff、worker lost、artifact mismatch、crash/restart、permission override、redaction、scale/soak/latency 全部纳入 Editor/Tooling acceptance。

## 11. 禁止的临时修补

- 不得把 Git 命令行、`git status` 文本或 provider credential 直接塞进 UI callback、ProjectManifest、ZUI 或 journal。
- 不得给五张页面继续添加固定 row、条件字符串、fake progress、fake worker、fake health 或“queued/succeeded” feedback。
- 不得让 Project Overview 复制 Build/Plugin/Test/Release 的可变模型；它只能引用 immutable snapshot 并显示 provenance/freshness。
- 不得将 authoring commandlet 改成绕过正常 UI/transaction 的第二套 automation API，也不得用同步全量 clone 取代分页/cursor/result artifact。
- 不得在存在冲突、dirty document、stale validation、artifact mismatch、权限/credential unknown 或 expected revision 变化时自动降级为 Submit/Publish。

## 12. 本轮验证边界与交付

已完成：递归源码枚举、固定业务事实搜索、exact contract/provider token 搜索、参考路径存在性检查、195 文件/28 参考文件 fingerprint 冻结，以及与 Editor27/85/02/04/06/08/09/10 owner 边界交叉核对。未运行 Cargo 或动态 Editor lane；因此本报告不宣称任何实现已通过编译、集成、性能或故障注入。`source_recheck_required: true` 是因为共享工作树仍 dirty，后续实现前必须对所有 selected path 重新计算 fingerprint。

后续实施按本报告 owner 分层推进；Editor27 只承担 VCS/Operations projection/Submission admission 的合同和 UI，Tooling10/03/09 继续作为外部唯一执行 authority，禁止通过协调器状态或旁路任务掩盖上述缺口。
