---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-preview-play-scene-report-args
origin_plan: docs/plans/zircon_editor/editor/04-pie-and-simulation.md
fixing_plan: docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md
origin_child_dir: docs/plans/zircon_editor/editor/04
fixing_child_dir: docs/plans/zircon_editor/editor/16
related_code:
  - zircon_app/src/entry/entry_runner/runtime_session_args.rs
  - zircon_app/src/entry/entry_runner/runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_runtime/src/dynamic_api/session/construction.rs
  - zircon_editor/src/core/play/process_backend/command.rs
  - zircon_editor/src/core/play/snapshot/source.rs
tests:
  - cargo test -p zircon_app --lib --locked runtime_session_args
  - cargo test -p zircon_app --lib --locked runtime_entry
  - cargo test -p zircon_runtime --lib --locked dynamic_api
---

# Editor16：runtime_preview 未消费 Play 场景与报告管道参数

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/04-pie-and-simulation.md`
- 来源执行切片：M1.2 `ProcessPlayBackend`
- 修复责任计划：`docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md`
- 交接原因：Editor04 负责快照、进程生命周期和 Play 状态；runtime executable 的类型化 CLI、动态 runtime 启动参数与帮助文本属于 Editor16。

## 失败现象与复现证据

历史上，Editor04 已按联合定稿构造：

`zircon_runtime --project <root> --runtime-session-profile runtime --play-scene <path> --play-report-pipe <name>`

2026-08-15 源码复审确认，历史的 `unknown runtime argument` 缺口已在共享源码中消除：`RuntimeSessionStartupArgs` 以 project-relative `RelPath` 类型化消费两条 flag，覆盖 equals/space、空值与重复值；`entry_runner/runtime.rs` 在创建 session 前验证 project 和参数，并将 `play_scene`、`play_report_pipe` 传给 `RuntimeSession::create_with_profile_and_project`。该入口构造单一 `ZrRuntimeSessionConfigV3`，dynamic runtime 在 event loop 前选择 Play override；版本化 `.zrscene.json` 走当前 `DynamicScene` reader，`.scene.toml` 走现有 scene asset reader，不会改写 manifest/default scene。报告出口由 app host 产生 starting/ready/start-failed/terminal 记录，Editor04 仍持有有界 stdout/stderr 泵。

Editor04 的 Play 快照仍写入 `.zircon/play/<instance>/play-scene.zrscene.json`，进程结束、stop 或 spawn failure 的清理由 snapshot owner 负责。默认 `ProcessPlayBackend` 产品装配和真实进程夹具仍须在本 failure 受管验证成功后启用，不能仅凭源码复审宣布闭环。

## 最低共享层根因

CLI parser、runtime session creation DTO 与动态 runtime 的首次世界选择没有同一类型化 Play startup contract。该缺口不能在 editor 侧通过删参数、改写工程默认场景或临时覆盖用户源文件绕过。

## 架构修复验收

- `RuntimeSessionStartupArgs` 类型化持有 `play_scene: Option<RelPath>` 与 `play_report_pipe: Option<String>`；help、空值、重复值、equals/space 两种形式均有回归。
- `--play-scene` 在首帧前覆盖默认世界：版本化 `.zrscene.json` 严格经 Plan11 `DynamicScene` 当前 reader 解码后注入；持久化 scene asset 路径走当前 SceneAsset reader。两者按明确格式合同选择，禁止旧格式 fallback/猜测。
- dynamic runtime/session API 采用当前架构允许的单一硬切入口传递场景 override；不得先创建并运行默认世界一帧再替换，也不得修改工程 manifest/default scene。
- `--play-report-pipe` 形成类型化启动报告出口，至少覆盖 starting/ready/start-failed/terminal；不可用时返回明确诊断，不能静默忽略。stdout/stderr 仍由 Editor04 的有界异步泵接收。
- project root、scene path 与 pipe name 只解析一次并在创建 session 前验证；场景载入失败时进程非零退出且不进入 event loop。
- 增加真实 `runtime_preview` 假工程/首帧退出夹具，证明 override 场景而非 manifest 默认场景被载入，报告序列有序，进程终止后无资源句柄残留。

## 禁止临时方案

- 禁止把两 flag 留在 `remaining_args` 后忽略，禁止 editor 删除 flag 退回“只运行默认场景”。
- 禁止覆盖用户默认场景文件、修改 `zircon-project.toml` 或复制完整 assets 目录制造临时工程。
- 禁止新增旧 CLI 别名、兼容双轨 reader 或未经 Plan11 reader 的裸 `serde_json::from_str<World>`。

## 修复结果与回传

Open / 源码已接线，受管验证待执行。Editor04 侧的 typed request、版本化快照、参数组装和进程清理，以及本计划的 runtime consumer、V3 startup/session contract、首帧前 override 与启动报告均已在当前源码复审中确认。尚无本记录列出的受管 Cargo 终态证据，也未运行真实 `runtime_preview` 假工程/首帧退出夹具；因此不得回传为 fixed，默认 Process backend 也不得据此提前启用。

## 产出记录与时间

| 日期 | 事项 | 状态 | 证据与后续 |
| --- | --- | --- | --- |
| 2026-07-18 | runtime_preview Play startup flags failure handoff | open / 已接收 Editor04 联合缺口 | 静态确认两 flag 进入 `remaining_args` 后被 `runtime.rs` 以 unknown argument 拒绝，且 session construction 无 override 注入；验收要求类型化解析、首帧前当前格式场景注入、报告出口与真实进程夹具，禁止 editor 覆盖用户工程。 |
| 2026-07-19 | Editor16 consumer 边界与 Runtime10 V3 原子依赖复核 | open / parser-only 中间态已拒绝 | `ZrRuntimeApiV3` 明确冻结，当前 `ZrRuntimeSessionConfigV2` 只有 profile/project/wake；其 V3 reactive-wake hard-cut 仍处于外部会话 `runtime03-10-11-reactive-wake-v3-design-r2-20260718` 的未提交原子范围。单独让 parser 消费 Play flags 会绕过 `remaining_args` 门禁并静默启动 manifest 默认场景，因此本轮未留下该错误中间态。后续必须在 Runtime10 V3 原子提交后，以新的单一版本化 startup/session 入口一次贯通 parser、首帧前 scene reader、report outlet 与真实进程夹具；不得把现有 V3 脏改动吸收到 Editor16 提交。Coordinator01 validation-copy Cargo terminal-evidence failure 节点 `516615` 仍在独立 owner 下 `resolving_failure`，故本记录不声称 Cargo GREEN。 |
| 2026-08-15 | runtime preview 参数链路源码重审 | open / 源码已接线 / 受管验证待执行 | `runtime_session_args.rs` 已以 `RelPath`/`String` 消费两 flag；`runtime.rs` 在 session 前校验 project 并建立 startup reporter；`runtime_session.rs` 使用 `ZrRuntimeSessionConfigV3`；dynamic session 在 event loop 前加载 override 并拒绝无 project、无效路径或无效报告出口。未运行 Cargo 或真实进程夹具，故不返回 Editor04、不启用默认 Process backend。 |
