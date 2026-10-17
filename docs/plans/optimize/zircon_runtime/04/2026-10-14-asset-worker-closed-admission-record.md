---
title: Runtime04 Closed IO Asset Worker Admission
report_id: Runtime04-closed-io-admission-2026-10-14
date: 2026-10-14
implementation_status: source_reviewed
validation_status: native_and_performance_pending
---

# Runtime04：IO 关闭后的资产任务准入回滚

`AssetWorkerPool::request` 在 Runtime IO owner 关闭准入后提交新的 unique 请求，原来会在插入 entry、增加 scheduled charge 后调用会 panic 的 `TaskPool::spawn`，留下没有实际 job 完成的占位。现在返回已有的 `CoreError::ChannelSend`，撤销占位与 charge，并在释放 registry 锁后终止 entry、取消 expiry、释放被拒绝闭包及移出的键。

本项对应 [Runtime04 计划](../04-core-resource-asset-serialization-review.md) 的异步请求所有权和 [Runtime02 计划](../02-core-runtime-events-tasks-review.md) 的停止准入/排空边界。公共 factory 可以接收外部 IO owner；本次没有证明默认 CLI 关闭流程必然触发该问题。完整资产系统、任务图及动态库卸载里程碑仍待各自验收。

## 实现契约

- crate-private `TaskPool::try_spawn<F> -> Result<(), F>` 在拒绝时归还原闭包；成功时复用既有私有 submission lease。公共 spawn 与 lease 的退出责任保持原契约。
- in-flight merge 和 completed cache 先于新 IO 准入，关闭准入后继续复用真实结果。
- provisional entry、scheduled charge、提交和 admitted 更新位于同一 registry 临界区，完成发布只能在 admitted 记账后取得该锁。
- 拒绝时以 `Arc::ptr_eq` 核对当前 entry、移出 entry 并回滚 charge，随后 `drop(state) → terminate(Rejected) → drop(job)/drop(removed)`。timer 取消及捕获对象最终释放在锁外发生。
- never-admitted 请求不增加 admitted、queue-rejected、failed、expired 或 cancelled。已准入任务自然排空；expiry generation 与 entry 身份保护保持。

独立静态复核确认 registry → diagnostics 锁序兼容；queue-age 返回后再进入 completion publish，未发现持有 diagnostics guard 再取 registry 的反向嵌套。

## 冻结源码

命名基线 `f28ba39a5095f4964c046ecc702778627f80cfd5`。初次修复前，三路径 raw SHA 与该基线一致，working/index scope 为空；未执行 Git mutation。当前交付为累计 R2：两个生产文件沿用 V1 冻结字节，仅修复两个既有测试及其 gate fixture 的失败清理生命周期；V1 patch/evidence/guard 保留为历史证据。

| 文件 | 冻结字节数 | 冻结 raw SHA256 |
| --- | ---: | --- |
| `zircon_runtime/src/core/runtime/tasks/pool.rs` | 15752 | `09bfb0b34d78893bab750486617a175c6e4e1d025b75b91a5289c6d71ecb1e45` |
| `zircon_runtime/src/asset/pipeline/worker_pool.rs` | 17390 | `6628cd3ed36637325d5f071e8720299a2bae8ef9537e844ebfd665e25bf5afdd` |
| `zircon_runtime/src/asset/pipeline/worker_pool/tests/cases.rs` | 30651 | `1e015a5f20dfbfcd1cc9ddb90515d10c297bf73216e284e50e4bf9297e3a1f03` |

[补丁](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-worker-admission-r2.patch)：13724 B，SHA256 `f53d82b842c9a648d72dd2a20c4214813eacfaa1c55b40a79939dd0084c75fe5`。

[源码/命令/选例证据](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-worker-admission-r2-evidence.json)：79170 B，SHA256 `265f7f9af2a017506b45f6e6f692a143451ee6a6f990628583da5f24e94e06ac`，owner 实际收取 packet writer/readback 最终 exit 0。

[精确三路径 guard](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-worker-admission-r2-guard.json)：952 B，SHA256 `473e6c9a1366e69af302dee18dd89145b5b84686cd7766458a42dba53bf1596d`；实际 subprocess/外层 writer 均 exit 0，0 errors / 0 warnings / 0 suppressed。

最终 scoped rustfmt、diff 和源码/wire/inventory 静态检查均 exit 0。格式化的 Windows 映射占用错误 1224、排版检查失败和 guard 包装器启动前的路径登记断言失败均保留在 evidence/artifacts。核心实现及取舍由 gpt-6.1-sol max 处理，独立 gpt-6-luna max 复核无静态阻断。

## 回归与合批

新增三个普通用例位于真实 `asset::pipeline::worker_pool::tests::`，没有额外 `::cases::` 段：

| 用例 | 可验证的契约 |
| --- | --- |
| `closed_runtime_io_rejects_new_asset_requests_without_retaining_admission` | IO owner 活着但准入关闭；同键/新键真实 request 均返回 IO 拒绝，registry、charge、诊断和活 timer 注册容量不残留。 |
| `admitted_asset_request_drains_after_runtime_io_admission_closes` | 已准入任务排空，merge/cache 观察同一实际 payload owner。 |
| `rejected_task_pool_submission_preserves_job_until_after_registry_unlock` | 通用 helper 归还未执行、未提前析构的原闭包，调用方解锁后可释放。 |

第三例直接验证 helper 所有权。真实 request 最终捕获对象的释放时点由当前代码与静态顺序检查支持，未作该时点的动态证明；请求接口继续不接收任意用户 job。真实拒绝用例使用独立运行的 TaskTimer。gated 用例保存观察后先释放 gate、排空/join IO、释放 pool/timer，再断言。

另外修复了两个既有 Barrier 用例：`cancelling_a_running_entry_wakes_its_ticket_and_releases_admission` 和 `optimization_batch_fl_runtime468_full_waiter_lookup_preserves_rejection_semantics`。它们在 gate 期间采集取消、票据、诊断或重复请求的观察，并以 `catch_unwind` 保存意外 panic；catch 内没有断言。随后释放 Barrier、调用 `shutdown_until` 尝试排空/join、释放 pool 及其 owned timer，再判定 join 结果、恢复观察或执行原断言。既有 `with_test_execution_gate` fixture 只被这两例调用，现在显式持有独立 timer。原来的重复请求拒绝/observer-budget 错误、首票据完成和诊断计数要求保留；该清理补修不增加名字或普通测试数量。

worker_pool namespace 共 13 普通（10 既有 + 3 新增）与 3 ignored；TaskPool namespace 在 Windows 为 3 个既有普通，其中 TLS destructor 用例有 cfg(windows)。旧 Runtime 普通选例对新增 16 普通名字的有效交集为 0。

第 13 批计划在现有 ordinal 2 arm 尾部追加两个完整 namespace，Windows floor 73 → 89，逐一要求全部 16 名字执行。3 ignored 不计普通 pass。保留既有 16 Cargo arms、2 Python arms、16 原始 producer 身份、独立 ordinal 11 门槛及原始矩阵/数值；不逐例编译。

## 性能与待验收

拒绝回滚为常数次表查询、移除与计数，平均 O(1)，没有新增全局注册表或 payload 克隆。接受分支临界区延伸至提交与 admitted 更新；当前没有该分支的时延或吞吐实测。

原 Runtime03 完整 60 样本、相对 helper 门槛及冻结绝对 P95 预算，Runtime44 的 54 行 RSS 与 peak/growth 双 128 MiB 上限，以及其他既有性能/资源/产品门槛全部保留。静态 pass、用例库存和文件交接均不代表原生、性能或里程碑达标。

本记录冻结时第 12 批已实际文件交接，第 13 批为计划合批；后续交接/终态回执单独写入任务 artifacts/log。Runtime 独立修复在第 11 批实际交接后继续，没有等待第 12 批清单或编译完成。

[对应计划完成列表](../../../astra/features/runtime/2026-10-14-runtime04-asset-worker-admission-completion-list.md) 保留未完成的原生项。本次保留必要 patch、静态证据和计划记录，实际释放 0 B；没有创建本地编译/缓存、源码副本或原生临时夹具。Jenkins 执行前登记归属与实际物理 E:/cargo-targets/zircon-local 编译/缓存/子进程 TEMP/TMP 路径，结束后 join 自有进程并登记归属清理及释放字节。
