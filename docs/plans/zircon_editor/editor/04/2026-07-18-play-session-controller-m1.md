# Editor04 M1.1 PlaySessionController

## 目标与状态

- 状态：M1.1 源码完成；受管 Cargo、独立复审、failure fixed-return 与 commit 未完成。
- 目标：以 `PlaySessionController` 统一 `Edit/Building/Playing` 权威状态，把现有 native plugin 活性切换按真实语义改名为 `PluginBridgeActivation`，并让菜单与 CommandEval 消费同一状态源。
- 非声明：本切片没有实现 P1 子进程、P2 副 session、PIE 视口或 live edit，不把插件活性切换伪称为游戏运行后端。

## 完成项目

- 新增 `PlayMode` / `PlayModeKind` / `PlayKind`、`PlayStartRequest`、`PlaySessionError` 与 `PlayTransitionReport`。
- `request_play(immediate)`、`request_play(after_build)`、`on_build_finished(ok)`、`request_stop` 覆盖计划迁移表；activation/deactivation 失败保持原状态以便重试。
- controller 以专用 transition gate 串行生命周期；state/activation lock 在调用插件代码前释放，不跨外部插件调用持有 owner lock。
- `NativePluginBridgeActivation` 保留 plugin load、active snapshot、双进拒绝、宽容空退出、diagnostics sort/dedup 与 bridge matrix；默认提供 noop activation。
- host 持单一 controller；startup 注入 native activation；菜单入口只调用 `request_play/request_stop`，runtime event consumer 清理失败保持 Playing。
- `CommandEvalCtx.play_state` 直接读取 controller mode并完整投影 Building；UI chrome session mode不再是求值权威。
- 旧 `core/play/bridge.rs`、`EditorRuntimePlayModeBackend`、`EditorPlayBridge`、`NativePluginEditorRuntimePlayModeBackend` 与 setter/accessor 命名硬删除，无兼容别名。

## TDD 与静态证据

- RED：`python -m unittest tools.tests.test_editor04_play_session_controller_contract -v` 初次为 3 failures + 1 error，命中旧 bridge仍存在、三态文件缺失、CommandEval仍猜chrome、menu仍直调backend。
- GREEN：同命令 4/4 passed。
- Rust测试已加入：immediate play、build→playing、build fail→edit、Playing二次play拒绝、Edit stop no-op、Building stop cancel、native activation roundtrip。
- `rustfmt --edition 2021` 已覆盖本切片 Rust 文件；Cargo未运行。Coordinator01 full compile-input immutable snapshot failure仍open，共享树盲跑不能形成current-source证据。

## 剩余工作

- M1.2：process backend、快照目录、spawn/monitor/stop/crash 内核源码已转入 successor [2026-07-18-process-play-backend-m1.md](2026-07-18-process-play-backend-m1.md)；Editor16 runtime flags consumer、默认装配与产品进程门仍 open。M1.3 edit policy/pending edits 未开始。
- M2：runtime world payload注入、embedded session、PIE viewport、Simulate与零污染。
- M3/M4：play domain live link、双源同步、volatile history、live edit与显式回写。
- 本切片仍需受管 focused/broad Cargo、独立review、Editor08 failure fixed-return与coordinator managed commit。

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-07-18 | M1.1 controller/activation/CommandEval hardcut | 源码完成 / 待验证 | 三态权威、build迁移、menu同源API、plugin activation真实命名与CommandEval Building投影完成；旧backend/bridge API为0；Python TDD RED 3F+1E→GREEN 4/4，Rust测试落盘并rustfmt。Cargo/review/fixed-return未完成。 |

2026-07-22性能补充：inactive `poll_backend`只读一次mode；native activation成功deactivate改为move snapshot、失败放回。`transition_gate`仍跨plugin/backend foreign调用，按PERF-MVP-553与`failure-2026-07-22-play-snapshot-transition-main-thread-stall.md`保持open；源码守卫不提升M1.1状态。
