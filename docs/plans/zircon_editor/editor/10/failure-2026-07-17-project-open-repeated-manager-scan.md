---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: project-open-repeated-manager-scan
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/10
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/host/project_access.rs
  - zircon_editor/src/ui/workbench/project/editor_project_document_load.rs
  - zircon_editor/src/ui/workbench/project/editor_project_document_save.rs
  - zircon_editor/src/ui/workbench/startup/editor_startup_session_document_welcome_pane_snapshot.rs
  - zircon_editor/src/ui/workbench/project/layout_preset_assets.rs
  - zircon_editor/src/core/project/authority.rs
  - zircon_editor/src/core/project/opened_project.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/open_project.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs
tests:
  - project open manager/manifest/scan build-count regression
  - 1/100/1000 locator resolution project-manager open-count regression
  - open/save/import/workspace/watcher parity matrix
---

# Editor10：project open 与 locator resolution 重复 manager/scan

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：Editor core project + `ui/host/startup` 7/7 + `ui/host/project_access.rs` 静态审查
- 修复责任计划：`docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md`
- 交接原因：project generation、manifest、asset inventory、workspace 与 locator mapping 必须由 Editor10/Runtime asset project 的共同稳定 owner 提供。

## 失败现象与复现证据

`EditorUiHost::open_project` 先由 `ProjectAuthority::open_project` canonicalize/parse manifest，再调用 runtime `AssetManager::open_project` 与 editor asset refresh，最后 `EditorProjectDocument::load_from_path` 又新建 `ProjectManager`、`scan_and_import`、clone manifest 并加载 scene。一次 MVP open 因此存在多个 project owner/manifest parse/inventory scan 候选。`resolve_ui_asset_path` 与 `resolve_asset_locator_path` 还会为每次 locator 新建 `ProjectManager::open`，批量 UI asset/session restore 会放大为 N 次 project open。

Workbench 逐文件审查又确认：welcome snapshot 每次调用 `ProjectAuthority::probe_draft`，把 canonicalize/stat/manifest 验证放入普通 UI projection；project save 可能在序列化前重新 `scan_and_import`；layout preset load/save/list 反复打开 project manager，list 逐文件读取解析全部 preset。共享 project generation 还必须覆盖这些 consumer，而不只优化首次 open。

## 最低共享层根因

open flow 只在层间传 path/String，不传递一个 generation-bound `OpenedProject`/`ProjectManager` snapshot；authority、runtime asset manager、editor document 与 path helper 各自重新建立 project truth。

## 架构修复验收

- 一次 open 构造一个 canonical project generation，manifest parse 与 source inventory/scan 各至多一次；runtime asset manager、editor document、workspace/watcher 共用结果。
- locator batch resolution 复用当前 project generation 的 path index，不为每个 locator reopen project。
- save/reload/import/hot-change 精确生成新 generation；旧 snapshot lifetime 安全且不会读到半提交状态。
- welcome/preset 的稳定 snapshot 不执行文件 I/O；draft/path/preset-dir generation 变化才触发 worker probe/list，退出与显式 save 有 generation-safe flush。
- 1/100/1000 assets 记录 project-manager open/manifest parse/tree enumeration 次数、bytes-read 与 wall；scene/workspace/import diagnostics 和错误顺序等价。

## 禁止临时方案

- 不得只缓存 root `PathBuf` 而继续重建 manifest/inventory。
- 不得绕过 `ProjectAuthority` 的 canonical/symlink/reparse validation。
- 不得让 runtime/editor 各持一份可独立变更的 manifest truth。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 完成项目与证据 |
|---|---|---|---|---|
| Editor10 M1 / Performance01 | generation-bound open/document/watcher/locator | `open-实现与静态门完成-独立复审/受管行为门待执行` | 2026-07-18 | `OpenedProject` 持有已解析 manifest/registry index 的 prepared `ProjectManager`，Runtime 在同一实例执行唯一 source scan；Editor asset、document、watcher、save 与 UI asset external effects 复用活动 generation。`open_project_manager_for_paths`、document path load/save API 与旧 `_from_path.rs` owner 全仓清零。 |
| Editor10 M1 / Performance01 | welcome/locator/preset 稳定投影 | `open-独立复审整改已落地-复审/受管行为门待执行` | 2026-07-18 | welcome 创建/打开校验迁入 Background/Index job 并缓存结果，受控回归覆盖 A→B 迟到、退出取消及 job/submit failure；Runtime manager-owned path/URI 查询使 locator 与 preset 正常投影不再深 clone 完整 `ProjectManager`，1,000 locator/100 preset 回归已加入。list 不 reopen/read_dir/read_to_string；preset save/load 保留 typed `SceneProjectError`。详见 `2026-07-17-project-authority-reference-regression-closeout.md`。 |
| Editor10 M1 / Performance01 | active-scene identity / async preparation / lifecycle commit / dirty decision | `source_implemented_static_verified_review_ready / managed_behavior_and_product_trace_pending` | 2026-08-27 | 重复 manager open/scan 已清零；刷新目标硬切为 lifecycle-owned `ActiveSceneDocumentIdentity`，secondary scene 不再被 manifest default 覆盖；identity 新增 reservation 分配的 monotonic activation revision，A→B→A 时旧 A ticket 必定 superseded。`ProjectSceneLoadJob` 通过 `ProjectAuthority` 在 EditorJobSystem 后台执行 scene I/O/deserialization，retained host 单票据合并重叠请求；admission pressure 独立按 identity+Runtime generation 执行 64/128/256ms 三次退避，第四次进入同代 terminal diagnostic，新事件不重启循环；asset accumulator 与 admission retry 统一合并最早 maintenance deadline，poll 后同 tick 空 refresh 不再清除退避唤醒。project close 仅在 manager close 成功后取消票据。终端 `SceneDocumentReloadCoordinator` 与 open/create/close 串行化。dirty conflict 现绑定 identity+Runtime generation，并通过统一 Decision Center 提供 Save/Discard/Keep Editing。复审发现初版错误地把 lifecycle scene document id 送入只管理 toolkit/DirtyRegistry 的 batch save；该 provisional adapter/state 已硬切删除。冲突 Save 现在复用唯一 `SaveProject` command，由 `EditorManager::save_active_scene` 持久化活动场景并通过 Global transaction save token 标记 clean，随后复核 exact identity + clean history 才进入 reload；Keep Editing 抑制同代 watcher；Discard 仅在 lifecycle identity 与 Runtime generation fenced terminal install 内清空历史，load/install/supersession 失败不会提前丢失本地编辑。独立的 toolkit dirty-save coordinator 在任务 admission 前取得 SaveAll/ClosePrompt typed owner，非 owner 不得消费 completion；Save All 在 close batch 占用期间保留一次 queued request，Project/native close 在 owner 终态前禁止 teardown。Runtime generation fence 关闭检查后发布竞态，`PreparedLevel` 在安装失败时回滚未提交 registry publication。完整 Editor10 当前静态集 15/15、聚焦 generation/competition 静态集 13/13，scoped `rustfmt --check` 与 `git diff --check` 通过；最终独立源码复审为 `READY`，无新增 P1/P2。真实 completion wrong-owner、Save All 排队续接、Project Close 等待终态三类 Rust 行为回归已写入但未执行。本轮 E 盘 `cargo check -p zircon_editor --lib --offline` 一次在 161.3 秒后先失败于共享 `zircon_runtime` 的 61 errors/123 warnings；后续隔离 target 尝试在 364.2 秒仍停留 Runtime 且无诊断而超时，精确子进程树已清理，两次均未编译到 editor owner，不能计为本切片通过或失败。当前完整 Editor09 pattern 另有 3 个共享基线失败，分别为退休 `core/editor_plugin.rs` 路径、dirty facade 导出集合变化与 crate-root watcher poll re-export 缺失，不归本切片。Rust 行为执行、细粒度 job cancellation、UI-thread project-scene save/runtime-extension/level preparation 迁移、受管 Cargo 与 F0/F4 wall/UI-blocked/RSS/power trace 未完成，failure 保持 open。 |

## 修复结果与回传

Open state: `generation-bound 与 manager-owned source-index hot query 已实现；active-scene asset event 已进一步硬切为 lifecycle identity + active generation + ProjectAuthority typed scene-load job + dirty-aware terminal coordinator，UI tick 不再执行 scene artifact I/O/deserialization；Save/Discard/Keep Editing 冲突命令面已完成源码、静态门与独立源码复审，但 runtime-extension/level preparation、细粒度取消、current-source exact Cargo、Rust 行为执行及 open/scan/locator/scene-load I/O trace 未完成。save 后 transactional targeted import 尚由 Runtime04 failure-2026-07-18-project-source-index-targeted-import.md 处理，因此本 performance failure 不回传 fixed`。

2026-07-22 current-source补充：ProjectAuthority canonical root后的第二次ancestor link/reparse metadata walk已删除；该局部安全等价止损不关闭failure。welcome/recent probe稳定filesystem I/O和跨Authority/Runtime/Editor manager generation复用仍按PERF-MVP-075/100验收。

2026-07-30 retained-host current consumer补充：`reload_default_scene`在asset/resource event已经由活动manager处理后仍重新`ProjectManager::open + scan_and_import`；`import_model_into_project`也为活动root再open manager，并在UI callback执行stage/import链。Editor10不得新增缓存，而应让host从Runtime04活动generation取得prepared scene/model transaction ticket；default-scene event的manager open/scan=0，模型按钮manager open≤1且UI I/O=0。证据：`docs/plans/performance/01/2026-07-30-editor-retained-host-assets-current-review.md`；current-source managed Cargo与F0/F4 trace前failure保持open。
