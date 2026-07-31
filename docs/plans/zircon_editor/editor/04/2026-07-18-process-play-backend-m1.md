# Editor04 M1.2 ProcessPlayBackend

## 目标与状态

- 状态：进程后端内核源码完成；runtime CLI consumer、默认装配、受管 Cargo、独立复审与产品进程门未完成。
- 目标：在 M1.1 三态权威下落地可注入的 `PlayBackend`、版本化场景快照、进程 spawn/poll/stop/reap、输出回流和 crash transition，且 P1 不冒充可 attach backend。
- 非声明：本切片没有实现 Editor16 的 `runtime_preview` 参数消费、P2 embedded session、report-pipe transport、PIE 视口或 live edit。

## 完成项目

- 新增 `PlayBackend` typed `start/stop/poll` 与 `NoopPlayBackend`/`ProcessPlayBackend`；controller 只持 `Arc<dyn PlayBackend>`，不把 `Child` 或平台调用塞进状态机。
- 启动顺序固定为 `PluginBridgeActivation.activate` → `backend.start` → Playing；backend start failure 反向 deactivate 并保持 Edit。停止固定为 backend stop/reap → plugin deactivate → Edit。
- `poll_backend` 在 Running 时只排空 diagnostics；terminal exit 清理快照并产生 `PlayTransitionCause::Crashed { exit_code }` 后回 Edit。
- `PlaySceneSource::from_world` 只用 Plan11 `DynamicScene` 当前版本 writer；`PlaySnapshotStore` 以 temporary + flush + rename 原子发布到 `.zircon/play/<instance>`，owned snapshot stop/crash/spawn failure/drop 全路径清理，persisted source 永不删除。
- stdout/stderr 使用独立 reader 与 1,024 行 bounded channel；满载只累计 dropped-lines diagnostic，不允许日志洪泛形成无界内存。
- `PlayChild` 在 pipe setup failure、显式 stop 与 backend drop 上 kill+wait；自然退出由 poll `try_wait` 后 finish/reap。
- Process backend 显式 `attachable=false`；menu 只给 attachable backend 启动 editor gateway consumer。tick 在无 consumer 时仍先 poll backend，terminal process 可同步 shell/controller 回 Edit。
- menu 在进入 Play 前从当前 edit World 生成 typed scene source，不覆盖工程 default scene 或用户源文件。

## TDD 与静态证据

- RED：`python -m unittest tools.tests.test_editor04_process_play_backend_contract -v` 初次 4/4 error，分别命中 backend contract、process flags、snapshot owner 与 controller ordering 缺失。
- GREEN：同命令 4/4 passed；原 `tools.tests.test_editor04_play_session_controller_contract` 仍 4/4 passed。
- Rust tests 已新增：activation/backend 顺序与逆序、start failure rollback、terminal exit crash transition、process exact args、versioned snapshot roundtrip/cleanup、persisted source 不删除。
- Rust 文件已 `rustfmt --edition 2021`；当前未运行 Cargo。Coordinator01 full compile-input immutable snapshot failure 仍 open，共享树盲跑不能形成 current-source 证据。

## 开放依赖与后续

- Editor16 failure：`../16/failure-2026-07-18-runtime-preview-play-scene-report-args.md`。当前 runtime parser 会拒绝 `--play-scene/--play-report-pipe`，所以 startup 不注入 `ProcessPlayBackend`；禁止把必失败进程伪装成 MVP 完成。
- Editor16 返回后：startup 注入 current-install runtime executable，运行假工程及真实工程三轮 start/stop/crash，验证无孤儿 PID、快照目录清零、输出丢弃计数和 terminal shell transition。
- report pipe 的 typed transport 与日志17接入未完成；当前 stdout/stderr bounded pump 是 P1 可用回流，不冒充 report pipe 已连接。
- P2/P3/P4 仍按 parent plan 执行。

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-07-18 | M1.2 backend/snapshot/process monitor core | 内核源码完成 / runtime consumer open | Typed backend、严格 lifecycle 顺序与 rollback、bounded output、kill+wait、current DynamicScene atomic snapshot/RAII cleanup、crash transition、P1 non-attachable host routing完成；Python TDD RED 4E→GREEN 4/4，M1.1 guard 4/4，Rust矩阵落盘并rustfmt。Editor16 flags consumer、Cargo/review/product gate未完成。 |

2026-07-22性能补充：terminal poll在reader join/snapshot cleanup前释放active mutex，live output每poll最多64行；但单行`read_until`仍无bytes上限、每session手建两reader thread，World→pretty JSON→sync file仍为主线程。分别交接PERF-MVP-550/552与Editor04/14 open failures，本记录状态不提升。
