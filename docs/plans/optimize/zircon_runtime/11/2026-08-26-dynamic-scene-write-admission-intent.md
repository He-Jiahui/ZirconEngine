# Runtime11 Dynamic Scene Write Admission Intent 架构与验证计划

> 日期：2026-08-26
> 所属 failure：`runtime/11/failure-2026-07-22-dynamic-scene-session-bounded-async-io.md`
> 状态：`source_implemented_static_passed_managed_validation_pending`

## 1. 当前源码结论

`RuntimeSessionArchiveWriter::try_submit` 已使用 Runtime11 `BoundedKeyedIoLane`，并对
artifact retained bytes、entry count、deadline、same-path generation、shutdown 和 terminal
ticket 建立了边界。但是 path generation 在 lane admission 之前由
`prepare_archive_path_write` 推进，导致“请求被观察到”和“请求被资源 owner 接受”被错误地
合并为一个状态。

确定性失败序列：

1. 单 worker 被测试 gate 占用，writer 容量为 1；
2. path `P` 的 generation 1 成功进入 lane queue；
3. generation 2 先推进 process path record，随后被 lane 以 entry capacity 拒绝；
4. gate 释放后 generation 1 写完 temp file；
5. final commit 看到 process record 已是 generation 2，将 generation 1 判为 stale；
6. generation 2 从未被接受或执行，path `P` 没有任何合法写入发布。

因此一次正常背压拒绝可以破坏最后一个已经接受的写入。这是 correctness P0，不是微观性能问题。

## 2. 参考引擎与本仓语义

主要参考为 Unreal Engine
`dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/PackageAutoSaver.cpp`：

- autosave 先计算 `NewAutoSaveIndex` 和 `PackagesToSave`；
- dirty package 集合只在对应 save result 成功后清空；
- `AutoSaveIndex` 只在 `bSuccess` 后推进。

这里采用的是“成功接受/完成后推进 authority”的语义，不复制 UE 的全局 Editor singleton、
package 类型或同步保存流程。

Zircon 自身的 `BoundedKeyedIoAdmission` 已有相同两阶段合同：`try_admit` 返回 armed
admission，调用 `activate` 后才进入执行队列，未 activate 的 Drop 会释放 reservation。path
publication intent 必须跟随这个资源 admission，而不能领先于它。

## 3. 目标状态机

每个 canonical path 的 process record 分离两个单调序号：

- `next_write_generation`：只分配唯一 reservation identity，允许有间隙；
- `admitted_write_generation`：只记录 lane 已接受、允许竞争 final publication 的最新 generation。

调用顺序硬切为：

1. `reserve_archive_path_write`：校验当前 committed lineage/revision，分配唯一 generation；
2. `BoundedKeyedIoLane::try_admit`：执行 entry/bytes/closed/deadline admission；
3. `admit_archive_path_write`：再次校验 committed lineage/revision，刷新 expected commit，并推进
   `admitted_write_generation`；
4. 把 prepared ticket 发布到 worker closure；
5. `BoundedKeyedIoAdmission::activate`；
6. worker 流式写 temp，final commit 同时校验 admitted generation、commit 和 lineage/revision，
   然后 atomic rename。

lane 拒绝发生在步骤 2，因此不会执行步骤 3。reservation generation 的间隙不参与 stale 判定。
不同 writer 或 direct save 仍共享 process path record；后接受的 generation 可以阻止旧 temp 发布，
但未接受的请求没有该权限。

## 4. 所有权、并发与异常边界

- `ArchivePathWriteReservation` 只能消费一次，不能执行 I/O；
- `ArchivePathWriteTicket` 只能由 admit 产生，worker 只能拿到 admitted ticket；
- writer 用一次性 cell 在 lane admission 成功后、activate 前发布 ticket，避免 closure 提前执行；
- admit 二次校验可发现 reservation 与 admission 之间已经提交的 newer artifact；
- admit 返回错误时 armed lane admission 由 Drop 回滚，不留下 entry/bytes reservation；
- final filesystem write 和 callback 不在 path record mutex 外泄 user callback；现有 final rename 期间的
  process-global mutex 仍是待 profile 的结构性风险，本项不在无数据情况下改成 per-path shard。

## 5. 复杂度与资源变化

| 路径 | 修复前 | 修复后 |
|---|---:|---:|
| generation reserve | process map `O(1)` average | `O(1)` average |
| lane admission | `O(1)` owner path | 不变 |
| admission commit | 无 | process map `O(1)` average |
| final stale check | process map `O(1)` average | `O(1)` average |
| 被拒绝请求对合法写入的影响 | 可使其 stale | 0 |
| 每 submission 常驻额外状态 | 无 | 一个小型 one-shot ticket cell |

新增一次短 process-map mutex acquisition 和一个 one-shot cell；相对 JSON artifact 写入和 fs rename
不是吞吐优化，本项不宣称延迟、CPU、RSS 或功耗改善。它消除的是背压下的数据丢失语义。

未来若优化全局 path map，必须先测：并发 path 数、same-path 冲突率、admission rejection rate、map
mutex wait P50/P95/P99、temp-write/rename latency、CPU、allocation、RSS 与功耗。只有证明全局锁是
瓶颈后，才评估 per-path state、sharded registry 和有界 tombstone retention；不能先用更复杂结构
替换可证明的线性化语义。

## 6. 验证计划

确定性回归不使用 sleep：

- 单 I/O worker 先运行 blocking gate；
- writer `max_entries = 1`；
- 第一个 artifact 成功 admission；
- 第二个同 path artifact 得到 `EntryCapacityExceeded`；
- 释放 gate，第一张 ticket 必须 `Succeeded`，outcome 必须 `Ok`，落盘 archive 必须是第一个 artifact；
- shutdown guard 完成后删除仓库当前盘 `.codex/tmp` 下的 fixture。

静态阶段执行 scoped `rustfmt --check`、源码状态机断言、owned diff check 和 trailing whitespace。
受管 Cargo 的 `dynamic_scene_session` 行为测试、slow-disk/write-storm/cancel/shutdown，以及
1/64/512 MiB、1/1k burst、0/10/1000ms 产品性能/功耗矩阵仍保持 pending；没有回执前 failure
不得改名为 fixed，不提交 milestone commit，不发送手工企微。

## 7. 本切片完成定义

- lane admission 被拒绝不会推进 `admitted_write_generation`；
- 只有 admitted ticket 可以进入 filesystem worker；
- direct atomic save 同样经过 reserve/admit，不产生第二套 authority；
- 新回归测试挂在 writer 私有叶模块，不修改其他会话已改动的综合测试文件；
- 源码状态与验证状态分别记录，不把静态证据冒充 Cargo/性能证据。

## 8. 2026-08-26 源码验证结果

- scoped `rustfmt --check`：3/3 Rust 文件通过；
- reserve/admit/publication 状态机源码断言：9/9 通过；
- 4 个 owned 路径 trailing whitespace：0；
- tracked owned diff `git diff --check`：通过，仅有 Git 的 LF/CRLF 工作区提示；
- 确定性 capacity-rejection Rust 回归：1 项已挂载，尚未取得受管 Cargo 执行回执；
- 动态 slow-I/O、burst、CPU、RSS 与功耗样本：0，本项没有性能改善声明。

静态结构已经将“被拒绝请求可使已接受写入 stale”的影响上限从 1 个 last-good submission
收敛为 0；该结论仍需新回归与既有 cross-writer/direct-save 测试的受管执行确认。
