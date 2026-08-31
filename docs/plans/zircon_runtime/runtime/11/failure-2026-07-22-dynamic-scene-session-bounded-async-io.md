---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: dynamic-scene-session-bounded-async-io
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/11
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/scene/dynamic_scene/session/io
  - zircon_runtime/src/scene/dynamic_scene/session/path_mutation
  - zircon_runtime/src/scene/dynamic_scene/session/path_api
tests:
  - cargo test -p zircon_runtime --lib dynamic_scene_session --locked --jobs 1 -- --nocapture --test-threads=1
  - slow disk, write storm, cancellation and shutdown fixtures
---

# Runtime11：dynamic scene session有界异步I/O交接

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：dynamic scene session核心195/563逐Rust文件审查，PERF-MVP-475
- 修复责任计划：`docs/plans/zircon_runtime/runtime/11-job-system-task-model.md`
- 交接原因：Runtime11拥有统一job dependency、取消、队列预算与shutdown语义；Runtime04提供不可变archive artifact。
- 生命周期键：`dynamic-scene-session-bounded-async-io`

## 失败现象与复现证据

load/save/path mutation在调用线程执行完整`read_to_string`/parse/pretty String/`fs::write`；atomic save也先驻留完整payload再temp write/rename。每个小mutation重新加载并重写整个archive，没有in-flight bytes/count/time上限、同path合并、取消或shutdown结果合同。

## 最低共享层根因

session path facade直接拥有同步文件系统流程，没有经过Runtime11统一I/O lane，也没有path+generation ticket和bounded publication。

## 架构修复验收

- caller只提交Runtime04 immutable artifact ticket；I/O lane按path+generation single-flight，newer写合并/取消older未发布工作。
- streaming reader/writer避免完整pretty String常驻；temp write后flush/fsync/atomic rename，失败保留last-good且清理temp。
- read/write分别具count/bytes/time预算、backpressure和公平性；发布queue depth/bytes/age/drop/cancel/wait/service latency及RSS诊断。
- shutdown明确选择flush或cancel并可测试；任务failure/panic/cancel经唯一terminal observer回传，不靠poll storm。
- 1/64/512MiB、1/1k write burst和0/10/1000ms slow I/O下caller blocking I/O=0、pending bytes/RSS有界、每path同时发布≤1、stale write publish=0。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止每次save新建线程或绕过Runtime11线程预算。
- 禁止无界channel、DetachOnDrop fire-and-forget或仅在完成后丢弃stale结果。
- 禁止把serialize或fsync搬到主线程回调阶段。

## 修复结果与回传

Open state: `writer admission-intent correctness、submission-owned before-start cancel capability、
Runtime-owned production construction 与 shared atomic-file owner 收敛源码已修复且静态通过；同步
path facade、完整 bounded async I/O、active cooperative cancellation、托管 Cargo 与产品性能/功耗
仍待完成`。

2026-08-27 read-owner 重审确认，现有同步 load 即使整体包进 task，完成后的 archive 仍会在 lane 释放
retained bytes 后无预算驻留；同 path 调用也没有统一 physical request owner。本次先在 Runtime11 增加
cloneable RAII `RetainedByteBudget`/lease，再新增 production 只接受 `CoreHandle` 的
`RuntimeSessionArchiveReader`。后续 prepared-path authority 切片删除临时 lexical path owner，reader 与
writer 只接受 `asset::project::ResolvedProjectPath`；caller 在 project/path boundary 完成一次 physical
resolution，两个 alias 以 `ResolvedProjectPathIdentity` 共享 ticket/generation 和一份最大 archive
reservation；open、metadata、固定
64 KiB buffer 的 bounded streaming decode、normalize 与 validate 全部在 Runtime I/O worker 执行。
failure、queued cancel、panic 和最后 result Drop 均归还 reservation，request map 只保留 weak entry。

Unreal/Bevy/Fyrox 证据、`O(log R * K)` admission/reuse/retirement 与 O(n) decode 分析、边界测试和 profile
矩阵见 `docs/plans/optimize/zircon_runtime/11/2026-08-27-dynamic-scene-runtime-reader-owner.md`。新增 13 项 reader/
budget 源码回归；scoped rustfmt、无 private thread/channel/unsafe、caller-side filesystem source guard、
module size与trailing whitespace静态检查通过；JobSystem 审计 3/3，owner 15/15、behavior anchors 62/62、
mirror docs 5/5、`risks = []`。受管 Cargo、同步 facade 迁移、active cancellation、
512 MiB decoded heap amplification、RSS/WPR/功耗仍未执行，failure 保持 open。

2026-08-26 current-source 重审发现，已有 `RuntimeSessionArchiveWriter` 虽然使用 Runtime11
`BoundedKeyedIoLane`，却在 lane admission 之前推进 process-global path generation。容量或 bytes
拒绝的请求会使前一个已接受写入在 final commit 时错误变 stale，从而丢失最后一个合法 submission。
本次将 path intent 硬切为 `reserve -> lane admission -> admit -> activate -> final commit` 两阶段：
reservation 只分配唯一序号，只有 lane 已接受的 generation 才能成为 final publication authority；
direct atomic save 也走同一 reserve/admit authority。新增单 worker、单 entry 的确定性回归，证明第二个
同 path 请求被 `EntryCapacityExceeded` 拒绝后，第一个合法写入仍可成功发布。

架构、复杂度、UE `PackageAutoSaver` 对照与待测矩阵见
`docs/plans/optimize/zircon_runtime/11/2026-08-26-dynamic-scene-write-admission-intent.md`。源码
`rustfmt` 3/3、状态机断言 9/9、owned trailing whitespace 0；Rust 回归尚未取得受管 Cargo 回执，
因此本 failure 保持 open，也不声称 slow-I/O、burst、CPU、RSS、功耗或同步 facade 已达标。

2026-08-26 cancellation authority 重审确认，writer 在 lane activation 前丢弃了 admission 发放的
`BoundedKeyedIoCancelAuthority`，使 mutation owner 无法按既有 Runtime11 合同撤销 queued write。本次
让 `RuntimeSessionArchiveWriteSubmission` 私有持有 capability，并提供只委托同一 ticket/capability 对的
typed `cancel_before_start`；terminal 继续由 lane ticket 唯一拥有，未复制 domain 状态机。新增单 worker
gate 回归，覆盖幂等取消、`CancelledBeforeStart`、filesystem closure 不执行与目标文件不存在。

取消所有权、UE `FAsyncTaskBase::Cancel` 对照、复杂度与 profile 指标见
`docs/plans/optimize/zircon_runtime/11/2026-08-26-dynamic-scene-write-cancellation-authority.md`。
源码 `rustfmt` 2/2、capability/state-machine 断言 9/9、owned trailing whitespace 0；受管 Cargo 与
产品性能/功耗仍未执行，因此本 failure 继续保持 open。

2026-08-26 runtime-owner 重审确认，公开 `RuntimeSessionArchiveWriter::new` 在没有 Runtime owner 时
静默取得 process-global I/O pool，且全仓没有产品调用者。本次删除该 constructor，production 唯一入口
硬切为 `with_runtime(&CoreHandle)`；writer 保存 `CoreWeak`，每次 submit 在 path canonicalization、
generation reservation 与 lane admission 前验证 owner，过期时返回 typed `RuntimeUnavailable`。isolated
`with_scheduler` 只在 `cfg(test)` 编译，不构成 production fallback。新增 expired-owner 无文件落盘回归
和 no-process-global source guard。

所有权、复杂度、UE explicit queued-pool 对照与 profile 矩阵见
`docs/plans/optimize/zircon_runtime/11/2026-08-26-dynamic-scene-writer-runtime-owner-hard-cut.md`。
源码 `rustfmt` 2/2、owner/admission/path-intent 断言 14/14；受管 Cargo、shutdown、性能与功耗仍未执行，
因此本 failure 继续保持 open。

2026-08-26 atomic owner 重审确认，session writer 私有复制了
`File::create -> BufWriter::flush -> backup -> fs::rename`，既绕过 Runtime04 已收敛的共享
`core::resource::io::atomic_file` owner，也没有完成 staging file 与父目录 durability；缺失 parent
还会在 lane admission 前由 caller 创建。本次硬切为共享 `stage_atomic_write` 在 Runtime I/O worker
完成 write/file sync，随后复核 session path generation/commit/lineage，再由唯一 `commit` owner 执行
平台 replace、committed-file/parent sync 与 backup cleanup。缺失 parent identity 只规范化最近存在祖先，
不在 admission 前修改文件系统。

共享 commit 若在目标已替换后的 durability/cleanup 阶段返回错误，writer 只在该错误路径用固定
64 KiB 缓冲流式核对目标内容；内容已发布时同步推进 session path revision，但原 durability error
仍回传，不把错误伪装为成功。私有 temp/backup/restore/rename 算法和临时名 helper 已物理删除，
新增单 worker 回归锁定 queued 阶段零 parent 副作用及 shared owner 源码守卫。架构、复杂度与测量矩阵见
`docs/plans/optimize/zircon_runtime/11/2026-08-26-dynamic-scene-atomic-write-owner-convergence.md`。
scoped `rustfmt` 4/4、static contracts 9/9、production old-owner scan 0、owner 行数
287/13/211/249，scoped diff check 通过；受管 Cargo、fault/slow-I/O、CPU/RSS/WPR/功耗均未执行，
failure 保持 open。

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 证据 |
|---|---|---|---|
| 2026-07-22 | `open / cross-plan-handoff-recorded` | 从 performance/01 路由 dynamic scene session 同步无界 I/O、全量重写与缺失 shutdown/diagnostics 合同。 | 本 failure 的失败现象、根因与验收矩阵。 |
| 2026-08-26 | `open / admission-intent-source-implemented-static-passed` | writer path generation 拆分为 reservation/admitted authority；lane 拒绝不再破坏 last-good submission；direct save 共用同一 authority；新增 1 项确定性 Rust 回归。 | `session/io/atomic.rs`、`session/io/writer.rs`、`session/io/writer/tests.rs`；架构报告；`rustfmt` 3/3、状态机断言 9/9、trailing whitespace 0；托管 Cargo 与产品负载数据未声明通过。 |
| 2026-08-26 | `open / cancellation-authority-source-implemented-static-passed` | archive submission 保存 lane 发放的唯一 cancel capability；owner 可幂等取消 before-start write，terminal 仍由 Runtime11 ticket 单点拥有；新增 1 项确定性 Rust 回归。 | `session/io/writer.rs`、`session/io/writer/tests.rs`；cancellation authority 架构报告；`rustfmt` 2/2、capability/state-machine 断言 9/9、trailing whitespace 0；受管 Cargo 与性能/功耗数据未声明通过。 |
| 2026-08-26 | `open / writer-runtime-owner-source-implemented-static-passed` | 删除 process-global writer constructor；production 只接受 live `CoreHandle`，expired owner 在 path intent 前 typed 拒绝；test scheduler constructor 限制到 `cfg(test)`；新增 2 项回归。 | `session/io/writer.rs`、`session/io/writer/tests.rs`；runtime-owner 架构报告；`rustfmt` 2/2、owner/admission/path-intent 断言 14/14；受管 Cargo、shutdown 与性能/功耗数据未声明通过。 |
| 2026-08-26 | `open / shared-atomic-owner-source-implemented-static-passed` | 删除 session 私有 atomic-file 平台算法，改为共享 stage/CAS/commit；parent 创建移入 Runtime I/O worker；post-publication durability error 流式 reconcile authority；新增 2 项回归。 | `session/io/atomic.rs`、`session/io/support.rs`、`session/io/writer/tests.rs`；atomic owner 收敛报告；`rustfmt` 4/4、static contracts 9/9、production old-owner 0；受管 Cargo、fault/slow-I/O 与性能/功耗数据未声明通过。 |
| 2026-08-27 | `open / runtime-reader-owner-source-implemented-static-passed` | 新增通用完成结果 byte/lease-count budget 与 Runtime-owned archive reader；同 physical path nonterminal single-flight、terminal retry/refresh、结果生命周期预算、worker-only filesystem、typed failure/cancel/shutdown 源码合同已落地；新增 13 项回归。 | `core/runtime/tasks/retained_byte_budget*`、`session/io/reader/`、bounded load limit；reader owner与prepared-path authority架构报告；scoped rustfmt与源码守卫通过；JobSystem 3/3、owner 15/15、behavior 62/62、mirror docs 5/5、risks 0；同步 facade、受管 Cargo、RSS/WPR/功耗未声明通过。 |
| 2026-08-27 | `open / prepared-path-authority-source-implemented-static-passed` | 删除重复 lexical archive path owner；reader/writer 硬切为 `ResolvedProjectPath`，single-flight、lane coalescing 与 atomic revision authority 共用 `ResolvedProjectPathIdentity`；通用 lane 支持保留领域 `Eq` 的 typed key，alias 不再因 lossy string key 分裂。 | prepared-path authority 架构报告；新增 typed-key 4 项与 writer source guard 1 项，强化 reader/writer physical-alias 用例；目标旧 path/canonical/lossy/raw overload 0 命中，最大实现文件 634 行，scoped rustfmt/diff/source guards 通过；受管 Cargo、Windows alias、slow-I/O、RSS/WPR/功耗未声明通过。 |
| 2026-08-27 | `open / prepared-path-lifecycle-review-fixed-static-passed` | 独立复审的前 3 项 Important 已按 owner 生命周期修复：reader 仅复用 nonterminal request；process path authority 只保留 live state 的 weak entry；staging 不持锁，最终 publication 只按同 path/lineage 串行，跨 lineage/path 保持并行。follow-up 发现 clone fork 重复 lineage revision 后，revision 分配也收归 lineage shared allocator。 | 新增 terminal retry/refresh、64 unique rejected-path retirement、cross-path stale-lineage、clone-fork unique revision 与锁域源码回归；最终 follow-up `Critical 0 / Important 0 / Minor 0`；受管 Cargo/产品 profile 仍待回执。 |
