---
handoff_kind: failure
status: open
created_at: 2026-07-12
summary_slug: command-eval-play-state-projection
origin_plan: docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
fixing_plan: docs/plans/zircon_editor/editor/04-pie-and-simulation.md
origin_child_dir: docs/plans/zircon_editor/editor/08
fixing_child_dir: docs/plans/zircon_editor/editor/04
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/commands/when.rs
  - zircon_editor/src/ui/host/command_eval_projection.rs
  - zircon_editor/src/core/play/controller.rs
  - zircon_editor/src/core/play/mode.rs
tests:
  - cargo test -p zircon_editor --lib --locked command_eval
  - cargo test -p zircon_editor --lib --locked play_mode
  - cargo test -p zircon_editor --lib --locked tests::commands::when::play_mode_predicates_distinguish_edit_building_playing_and_cleanup -- --exact --test-threads=1
  - cargo test -p zircon_editor --lib --locked core::play::tests::edit_request_play_with_build_waits_for_build_result -- --exact --test-threads=1
  - cargo test -p zircon_editor --lib --locked core::play::tests::failed_build_returns_to_edit_without_activation -- --exact --test-threads=1
  - cargo test -p zircon_editor --lib --locked core::play::tests::build_completion_backend_start_failure_returns_to_edit_and_publishes_mode_boundary -- --exact --test-threads=1
  - cargo test -p zircon_editor --lib --locked --no-run --message-format short --color never
---

# Editor 04：CommandEvalCtx 缺少 Building/PlaySessionController 权威投影

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md`
- 来源执行切片：Plan08 M1.2 类型化 when 谓词与统一求值环境
- 修复责任计划：`docs/plans/zircon_editor/editor/04-pie-and-simulation.md`
- 交接原因：Plan08 负责谓词类型与消费一致性；`Edit/Building/Playing` 状态机和 `PlaySessionController` 的生命周期权威属于 Editor04，不能由命令层制造或猜测。

## 失败现象与复现证据

`WhenClause::PlayMode` 与 `CommandEvalCtx.play_state` 已存在，但 `ui/host/command_eval_projection.rs` 当前只把 `EditorSessionMode::Playing` 映射为 `Playing`，把 `Welcome/Project` 映射为 `Edit`。Chrome 快照没有 `Building` 变体，因此真实构建阶段永远无法使命令谓词命中 `PlayMode(Building)`；现有 Playing 值也来自 UI session mode，而非本计划尚未落地的权威 `PlaySessionController`。

静态复现：读取 `command_eval_ctx_from_chrome` 的 `match chrome.session_mode`，可见没有任何 `PlayStateKind::Building` 写入路径。当前共享里程碑禁止本切片运行 Cargo，因此不声明行为门通过。

## 最低共享层根因

Editor04 尚未落地三态 `PlaySessionController` 及其稳定快照/消息出口，Plan08 的中性 `CommandEvalCtx` 因而只能诚实保留不完整投影。最低修复层是 Play session 权威状态，不是菜单、命令面板或单条命令特判。

## 架构修复验收

- `PlaySessionController` 成为 `Edit/Building/Playing` 唯一权威，并提供宿主可订阅或读取的类型化快照。
- `CommandEvalCtx.play_state` 从该权威快照更新；进入构建、构建失败回 Edit、构建成功进 Playing、退出 Playing 回 Edit 均触发一致更新。
- 菜单、命令面板、UI binding 使用同一共享 snapshot；不得各自读取不同 UI flag。
- 增加三态转换与 `WhenClause::PlayMode` 投影回归，并回跑 Plan08 command eval 测试。

## 禁止临时方案

- 禁止把 Building 折叠为 Edit 或 Playing，禁止用任务文案、进度条、按钮可见性猜测状态。
- 禁止在菜单/命令面板单独特判 play 状态，禁止建立第二份命令求值 context。
- 禁止恢复旧 play-mode backend 兼容别名或把 UI session mode 宣称为最终权威。

## 修复结果与回传

Open state: `源码修复已落地，待受管Cargo与独立复审后回传fixed`。`PlaySessionController` 已成为 host-owned 三态权威，`command_eval_ctx_from_chrome` 现在显式接收 controller `PlayModeKind` 并覆盖 `PlayStateKind::Building`；生产投影不再匹配 `chrome.session_mode`。菜单、startup 与测试已硬切 `PluginBridgeActivation`，旧 backend/bridge 兼容名为0。Python契约 RED（3 fail+1 error）→GREEN 4/4；Rust迁移矩阵已落盘但当前未执行Cargo，故不改 `status: open`、不向 Editor08 宣称 fixed。

## 产出记录与时间

2026-09-11 滚动复核：补齐 `plan_link_mode: child_record_only` 后，协调器重新导入四条 `related_code`（request `0eeac538c3b74deea46150f78b23d688`）。handoff validator 818/818、scoped `git diff --check` 与本 failure 相关的六个 Python 断言通过。

完整 Python 文件另有一条旧位置断言仍要求已迁移到
`controller/runtime_ownership.rs` 的 `detach_terminal_play_gateway` 位于旧
`controller.rs`，不属于本 failure。静态 ticket
`81a4fec0b72a456abb71990c3e4024ec` 已排队但受下游 failure 图阻塞；Cargo
admission request `e6613c5fc9aa4f0c9900798b9b723eeb` 在生成 ticket 前因外部
`E:\\Git\\zr_vm` 脏工作树被拒绝（`validation_ticket_external_worktree_dirty`）。
因此保持 `status: open`，待依赖与外部 owner 清理后执行精确
`command_eval`/`play_mode` Cargo 门及独立复审。

| 日期 | 事项 | 状态 | 证据与后续 |
| --- | --- | --- | --- |
| 2026-07-18 | PlaySessionController → CommandEval 三态投影 | 源码完成 / open待验证 | controller `Edit/Building/Playing`、build结果迁移、menu同源API与CommandEval Building投影已完成；Python静态契约4/4，旧token 0。待 managed focused/broad Cargo、独立review及 lifecycle fixed-return。 |
| 2026-07-27 | CommandEval 测试输入硬切 | resolving_failure | 已复核 production projection、reflection 与 remote/CLI 路径均读取 `PlaySessionController::mode()`，菜单 effect 与 backend terminal 路径均会刷新 command-eval snapshot；旧 backend/bridge token 搜索为 0。测试 helper 已改为显式接收 `PlayModeKind`，不再由 `chrome.session_mode` 推断；`python tools/tests/test_editor04_play_session_controller_contract.py`（4/4）、`rustfmt --check` 与 `git diff --check` 通过。仍待受管 Cargo、独立复审与 lifecycle fixed-return。 |
| 2026-08-23 | Build 完成后的启动失败状态归一 | implementation_complete / static_validation_complete / managed_validation_blocked | 复读 `on_build_finished(true)` 发现插件激活或 backend 启动失败会清理启动序列却遗留 `Building`，使 CommandEval 与下一次 Play 请求读取错误权威态。现在失败路径统一提交 `Edit` 并在释放 transition gate 后发布唯一 `Building → Edit` typed mode message；新增 backend-start failure 回归覆盖 rollback 调用顺序、终态与总线边界。`rustfmt --check`、`git diff --check` 已通过；本会话受 coordinator `unmanaged_artifacts_detected` 拒绝启动受管 Cargo，故本交接仍保持 `open`。 |
| 2026-08-23 | PIE snapshot E1 typed error hard cut | implementation_complete / static_validation_complete / managed_validation_blocked | `PlaySceneSource::from_world` 不再把 `DynamicScene::from_world` 和版本化 JSON 写入错误压扁为 `String`；snapshot owner 新增 `PlaySceneSourceError::DynamicScene(#[from] DynamicSceneError)`，公开 facade 同步导出，签名回归锁定该 contract。唯一 production caller `execute_menu_action` 仅在最后 UI event string boundary 显式 `to_string()`；snapshot source/error 内 `String`/`format!` 扫描为 0。`rustfmt --check` 与 scoped `git diff --check` 通过；受管 Cargo 仍被 coordinator `unmanaged_artifacts_detected` 阻断，故本交接仍保持 `open`。 |
| 2026-08-23 | PIE process-backend install E1 typed error hard cut | implementation_complete / static_validation_complete / managed_validation_blocked | `ProcessPlayBackend::for_current_install` 与其 executable resolver 不再以 `String` 汇总安装定位失败；新增 owner leaf `ProcessPlayBackendInstallError`，明确区分 current executable、missing install parent、install root 与 sibling runtime resolution，并保留每个 `io::Error` source。公开 facade 同步导出，签名与 missing-parent 变体回归已加入；resolver/error 内裸 `String` result/格式化压扁扫描为 0。`rustfmt --check` 与 scoped `git diff --check` 通过；当前存在其他会话的 Cargo/Rustc 进程且 coordinator 仍报告 `unmanaged_artifacts_detected`，因此未运行受管 Cargo，本交接保持 `open`。 |

## 2026-09-25 current-source rolling reconciliation

- The current projection accepts an explicit `PlayModeKind` and maps all four
  controller states, including `PlayModeKind::Building` to
  `PlayStateKind::Building`; no `match chrome.session_mode` remains in the
  projection. The controller still owns `request_play`, `on_build_finished`,
  `request_stop`, and `play_after_build`, and the command predicate test covers
  Edit/Building/Playing/CleanupFailed.
- The repository contract suite
  `python -X utf8 -m unittest tools.tests.test_editor04_play_session_controller_contract`
  passed `7/7` at current source. This is static/Python evidence only; no Rust
  command was run directly and no managed Cargo pass is inferred.
- Current related-code SHA-256 manifest:
  `zircon_editor/src/core/commands/when.rs=cd4677a2b70cf7f1313e93b028d463a78a5f25f170e9e7881b23212853fa47ff`,
  `zircon_editor/src/ui/host/command_eval_projection.rs=6d2e76c665481c5a18e96fb16ffd2cf71785b1a907df054e66ab4db6ae943b45`,
  `zircon_editor/src/core/play/controller.rs=d89ce8bab196671f4fc8fb5ce32f326728eb49aa495d2d2dc425bb63fde48c63`,
  `zircon_editor/src/core/play/mode.rs=8f82bd416626a44411aa7d046d23666a0842577321a4269b3706be9058761c5c`.
- `when.rs` was dirty in the shared checkout at the manifest scan and remains
  foreign to this doc-only reconciliation. The independent reviewer also
  observed `core/play/mode.rs` dirty during review; a subsequent status check
  shows `mode.rs` clean again while `when.rs` remains modified. Both observations
  are retained as provenance, and no source line was edited or absorbed by this
  Session.
- This current-source pass adds exact command-eval and PlaySession transition
  filters plus an Editor no-run compile to the manifest. Managed Windows
  selection of the immutable target, upward Editor04 gates, independent review,
  canonical fixed return, and closeout remain pending; the failure stays
  `open`.

## 2026-09-25 independent static review receipt

- Reviewer `/root/review_editor03_gizmo_private` re-read snapshot 3794 at
  document SHA-256 `1b6c013407ebf683cfca5133cf7fbaae470e5f7f96a1e3f4e4a8be2702dc7b59`.
  All four related paths exist; the projection maps Edit/Building/Playing/
  CleanupFailed through `PlayModeKind`; `chrome.session_mode` is absent; the
  contract's seven methods and all four exact test names resolve.
- The initial review reported C/I/M `0/1/0` for the omitted transient
  `mode.rs` dirty state. After reconciling that observation above against the
  current working tree, the provenance issue is resolved: current status shows
  only foreign `when.rs` dirty, while the review-time `mode.rs` observation is
  preserved rather than erased. No source owner or line was claimed.
- No Cargo command was run or represented as passing. Managed Windows exact,
  group, and upward gates, canonical fixed return, and closeout remain pending;
  the failure stays `open`.

## 2026-09-26 successor intake (failure-roll-01a084c8-editor04-command-eval-r4)

- The stale r3 lifecycle was cancelled through the coordinator with no active
  lease. Successor `failure-roll-01a084c8-editor04-command-eval-r4` now owns
  only this failure document; its document lease was acquired against base
  SHA-256 `a1d058d7b69236adad605b5c7f63661d68b6e1b623aa4aadbb62e6cce841c679`.
  No source path is leased or edited by this successor.
- Current source-bound hashes were rechecked before this intake:
  `zircon_editor/src/core/commands/when.rs`
  `cd4677a2b70cf7f1313e93b028d463a78a5f25f170e9e7881b23212853fa47ff`,
  `zircon_editor/src/ui/host/command_eval_projection.rs`
  `6d2e76c665481c5a18e96fb16ffd2cf71785b1a907df054e66ab4db6ae943b45`,
  `zircon_editor/src/core/play/controller.rs`
  `d89ce8bab196671f4fc8fb5ce32f326728eb49aa495d2d2dc425bb63fde48c63`, and
  `zircon_editor/src/core/play/mode.rs`
  `8f82bd416626a44411aa7d046d23666a0842577321a4269b3706be9058761c5c`.
  `when.rs` is dirty under foreign ownership; that provenance remains
  unclaimed and no source line is absorbed here.
- The existing 7/7 Python contract and static review are historical evidence,
  not dynamic acceptance. Managed Windows command-eval/play-mode filters,
 Editor04 upward gates, independent successor review, canonical `fixed-*`
 return, coordinator closeout, and WeCom notification remain pending. The
 failure stays `open`.

## 2026-09-26 independent successor review (review-editor03-gizmo-private)

- Reviewer `/root/review_editor03_gizmo_private` rechecked intake snapshot
  `3929` at document SHA-256
  `99bbb7dd0801cd62d956c87692a6480f8f52d8b4e4145f30cabb2e5fdc81e92a`.
  Stale-r3 cancellation/no-lease evidence, the r4 document-only lease/base
  hash, all four source hashes, and the foreign `when.rs` provenance are
  consistent; no source line was absorbed.
- The exact 7/7 Python/static evidence remains historical and is not promoted
  to dynamic acceptance. Managed command-eval/play-mode and upward Editor04
  gates, canonical `fixed-*` return, coordinator closeout, and WeCom
  notification remain pending.
- Independent review result: `Critical=0, Important=0, Moderate=0`. The
  successor remains open pending managed validation and closeout.
