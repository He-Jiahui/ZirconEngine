# Editor17 M2.1 自动保存恢复基础层

本记录覆盖 `17-editor-services-and-recovery.md` 的 M2.1 中已可独立交付的
autosave core。它不宣称 Editor16 项目会话锁、Editor14 job adapter 或 M2.2
恢复对话已经完成。

## 实现边界

- 新增 `zircon_editor::core::recovery` 与 `autosave.rs`：默认 300 秒单飞调度、
 3 份数值序号轮换、同目录原子快照写、进程内序号互斥和严格路径标识校验。
- 脏态输入硬切为 Editor03 `HistoryDirtyState` 的不可变投影；恢复层没有生产
  bool dirty constructor、save token 或源文件保存入口。
- `AutosaveJobPolicy` 固定 `JobCategory::Misc`、`JobPriority::Background`，并仅
  接受调用方给出的保存 mutex group。
- 更新 `docs/zircon_editor/core/recovery.md`，记录已实现契约与跨进程边界。

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 证据 |
|---|---|---|---|
| 2026-07-23 18:07 +08:00 | `实现完成-静态门与独立复审通过-受管Cargo排队` | M2.1 autosave core 已完成：History dirty 投影、300 秒 single-flight、submit/terminal 两条释放路径、项目 `.zircon/autosave` 原子快照、每文档保留 3 个序号、同一 store 并发序号互斥、`Misc/Background/save mutex` job policy 与基础单测。 | `rustfmt --edition 2024 --check` 通过；生产源码扫描无 `dirty/clean` constructor、旧 claim/permit、源文件复制或 save token 调用；最终增量独立复审 `Critical 0 / Important 0 / Minor 0`。snapshot `1066`，4-path manifest；CPU reservation `7182eea32f424a60b1ddaec3c2f7053b`，fingerprint `9c0752fc7ff1e13504324ae2fc092f34569bf55dac2faae9ece513754a79055f`，target `E:\cargo-targets\zircon-engine\editor17-recovery-autosave-focused-r1`，当前 FIFO pending，尚未产生 Cargo job。 |
| 2026-07-23 18:10 +08:00 | `依赖已路由-不阻塞基础层静态交付` | 项目会话锁与 autosave 跨进程排他交接 Editor16；实际 job admission、保存 mutex 串行和 terminal callback 交接 Editor14。 | [Editor16 session lock handoff](../16/failure-2026-07-23-project-session-lock-reuse-for-recovery.md)；[Editor14 autosave job handoff](../14/failure-2026-07-23-autosave-job-admission-and-save-mutex-adapter.md)。 |

## 未闭环项

- 仅在 reservation 变为 FIFO 队首后运行 `cargo test -p zircon_editor --lib --locked core::recovery::tests -- --test-threads=1`；在成功的受管 run、最终复审和 coordinator commit 前，本记录不标记 M2.1 accepted。
- Editor16 的单实例 session lock 接通后，才能把 autosave 接入真实项目启动和 M2.2 异常退出恢复检测。
- Editor14 adapter 接通后，才能把计划文档的快照写实际提交到唯一 `EditorJobSystem`，并接入 shutdown 最后一次 autosave 顺序。

## 2026-07-30 Performance01 性能补充

- current-source 3/3复读确认本基础层仍无生产adapter，因此以下是接线前门禁而非现行UI热点；性能任务统一记为`PERF-MVP-592`。
- `plan`会clone/sort全部dirty document ID；Editor14接线必须以bounded admission window逐文档惰性取得generation snapshot，禁止在排队前同时序列化全部dirty文档。
- 每次`write_snapshot`当前在写前后各枚举一次document autosave目录，并为filename/path/map建立owned临时集合；steady write应改用持久小manifest或固定ring slot，目录reconcile只在startup/recovery执行一次。
- autosave私有atomic writer应汇入Runtime11/Runtime Foundation共享bounded streaming stage、fault injection与durability counter；不得靠删除fsync/atomic replace提速。
- 验收矩阵为dirty docs `1/100/10k`、document `1KiB/1GiB`、directory entries/orphans `3/1k/100k`、filesystem `0/10ms/2s`。要求queued payload=0、额外payload内存按bounded buffer、steady directory scan=0、UI filesystem wall=0，并保持source digest、sequence race、rotation/recovery/crash/shutdown合同。
