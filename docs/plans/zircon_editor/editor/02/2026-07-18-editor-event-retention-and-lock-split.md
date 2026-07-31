# Editor02 editor-event retention 与 fanout 锁拆分

## 目标与边界

- 修复 `editor-event-journal-listener-unbounded-retention`：消除 journal/global deliveries 无界 `Vec`、per-listener 深复制和 sequence/revision 锁内 fanout。
- journal 与 listener inbox 共用同一 retention authority；不新建第二套 undo/operation history。
- 旧单文件 `core/editor_event/listener.rs` 直接删除并硬切为职责目录，不保留 alias、forwarding wrapper 或兼容 re-export。
- 本切片不吸收 Editor14 job pump、Editor10、Render 或 Coordinator01 生产文件。

## 实现清单

- [x] `retention.rs` 定义 durable replay、frame-local、latest-state 三类记录及独立条数/编码字节/年龄预算。
- [x] journal 与每个 listener inbox 共用 `EditorEventRetentionStore`；drop/coalesce、retained bytes、age 与 sequence gap 均显式诊断。
- [x] service 每条记录只构造一个 `Arc<SharedEditorEventRecord>`；fanout 只复制 Arc，查询边界才生成拥有型 delivery DTO。
- [x] listener registry 改为稳定注册顺序 + `HashMap` 索引 + per-listener inbox；status/query/ack 不再扫描全局 deliveries。
- [x] sequence/revision、journal、listener 拆成独立 mutex；journal 发布结束后释放锁，再进入 listener filter/fanout。
- [x] retention queue 以 O(1) 到达顺序 append/evict，查询边界按 sequence 排序；旧的晚到 latest-state 不能覆盖新状态。
- [x] 增加 1,000 latest-state coalescing、10,000 paused-listener、字节/年龄预算、逆序 fanout 与结构边界测试。
- [x] 同步当前模块与 UI binding/reflection 架构文档；历史 acceptance 命令保留原始证据，不伪改旧记录。

## 验收门禁

- [x] RED：新 retention policy 未导出且 sequence state 仍拥有 journal，静态契约按预期失败。
- [x] 定向 rustfmt 与 `git diff --check` 通过；无租约文件的递归格式噪声已精确清除，staged_total=0。
- [x] 静态结构 gate：共享 Arc payload、`VecDeque` retention、per-listener inbox、三锁 owner 与旧 global delivery Vec 缺失均通过。
- [ ] source-bound `cargo test -p zircon_editor --lib editor_event --locked`。
- [ ] 1k/10k stress 受管运行、完整 editor_event 回归与独立 review 0/0/0。
- [ ] failure -> fixed return 与 exact-manifest managed commit。

当前 Cargo 门禁不得从共享可变工作树盲跑。Frameworks01/Editor10 等 whole-lib 消费方已证明会读取运行中变化的完整编译输入；Coordinator01 的 `full-compile-input-snapshot-barrier-missing` fixed return 是本切片 source-bound Cargo 的前置条件。

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 证据/剩余项 |
| --- | --- | --- | --- |
| 2026-07-18 11:20-11:39 +08:00 | `source_landed_static_green_managed_validation_blocked` | 完成统一 retention owner、三类预算、共享 payload、per-listener inbox、sequence/journal/listener 分锁、旧 listener 单文件硬切、压力/顺序/诊断测试与模块文档 | RED 已观测；定向 rustfmt、静态 retention contract、`git diff --check` 通过；19 路 exact lease 由 `editor02-event-retention-lock-split-r3-20260718` 持有。待 Coordinator01 immutable full-input fixed return 后执行 managed Cargo、review、failure return 和 commit。 |
