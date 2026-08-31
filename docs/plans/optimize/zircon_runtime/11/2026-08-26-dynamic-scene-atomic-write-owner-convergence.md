# Runtime11 Dynamic Scene Atomic Write Owner 收敛计划

> 日期：2026-08-26
> 所属 failure：`runtime/11/failure-2026-07-22-dynamic-scene-session-bounded-async-io.md`
> 状态：`source_implemented_static_passed_managed_validation_pending`

## 1. 当前源码结论

`RuntimeSessionArchiveWriter` 已经把已封存 artifact 放入 Runtime11
`BoundedKeyedIoLane`，但 `session/io/atomic.rs` 又私有实现了一套
`File::create -> BufWriter::flush -> target 备份 -> fs::rename` 流程。该流程与 Runtime04
已经硬切到 `core::resource::io::atomic_file` 的共享 crash-safe owner 重复，并且只刷新了
用户态 writer：没有对 staging file 执行 `sync_all`，也没有在发布或清理 backup 后同步父目录。

这不是可接受的 last-good 持久化合同。在系统崩溃或断电边界，返回 `Ok` 只能说明字节进入了
操作系统缓存，不能证明 staging 内容与目录项已经达到共享 owner 所定义的 durability 水位。
同时 `canonical_archive_target` 会在 lane admission 前调用 `create_dir_all`，使一个最终被 entry/
bytes backpressure 拒绝的请求仍可在提交线程修改文件系统。

## 2. 全模块与参考实现复核

- Runtime04 的唯一共享 owner 已提供 `stage_atomic_write(path, bytes)` 和
  `PendingAtomicWrite::commit()`：stage 负责唯一 sibling staging、完整写入、flush、file sync 和
  新目录持久化；commit 负责平台特定 replace/backup、committed target sync、父目录 sync 与清理。
- Dynamic-scene 特有职责只有 artifact/path generation、lineage/revision CAS 和 stale publication
  拒绝；它不应复制平台持久化算法。
- 本地 Unreal `PackageAutoSaver.cpp` 将备份与真正 package save 分开，并只在 save 成功后推进
  autosave authority；`FileManagerGeneric.cpp::FArchiveFileWriterGeneric::Flush` 由文件 owner 承担
  flush。Zircon 不复制 UE 的 Editor singleton，而是保持“业务 generation policy 在上层、文件
  durability 在共享 I/O owner”的同一职责划分。

## 3. 硬切设计

写入状态机收敛为：

1. caller 只完成 Runtime owner、路径 identity 与 lane entry/bytes admission；不得创建目录或文件；
2. I/O worker 调用共享 `stage_atomic_write`，把已封存 `Arc<[u8]>` 写入并同步 sibling staging；
3. worker 在 session path authority 下复核 admitted generation、expected commit 与 lineage/revision；
4. 复核失败时直接 drop `PendingAtomicWrite`，由共享 owner 删除 staging，last-good target 不变；
5. 复核成功时调用唯一 `commit()` 发布；若共享 owner 在目标已替换后的 committed-file/父目录同步或
   backup 清理阶段返回错误，仅在该错误路径流式比较目标与 artifact。内容已经发布时仍推进 session
   commit/revision，同时把原 durability 错误返回调用方；正常成功路径不重复读取；
6. 删除 dynamic-scene 私有 temp-name、backup、restore、`File::create`、`BufWriter` 与 `fs::rename`
   算法，不保留 wrapper 或双轨 fallback。

对尚不存在的嵌套 parent，路径 identity 使用“最近存在祖先的真实 canonical path + 缺失的词法尾部”。
这样可以在 admission 前识别同一目标，又不提前创建目录；真正目录创建与 durability 仍只发生在
worker 的共享 stage owner。

## 4. 算法、资源与性能边界

| 项目 | 当前私有实现 | 硬切后 |
| --- | --- | --- |
| artifact 写入复杂度 | `O(bytes)` | `O(bytes)` |
| staging 常驻副本 | 1 个文件 | 1 个文件 |
| 用户态额外 payload clone | 0（Arc） | 0（Arc） |
| lane 拒绝前文件系统 mutation | 可能创建 parent | 0 |
| file/parent durability owner | session 私有且不完整 | Runtime04 共享 owner |
| stale CAS 后清理 | session 私有 temp 删除 | `PendingAtomicWrite` RAII |
| 平台 backup/replace 算法 | session 重复实现 | 唯一共享实现 |
| post-publication durability 错误 | 无法区分 | 错误路径流式核对并同步 authority |

共享 owner 增加的 `sync_all` 与目录 barrier 可能增加单次 durable-save wall time，这是正确性成本，
不能在无数据情况下伪装成吞吐优化。lane queue age、stage write/sync、CAS wait、commit/parent-sync、
错误路径 publication reconcile、总 service latency 必须分开测量，才能判断后续是否需要 per-path authority shard；本切片不改变
全局 path revision map，也不宣称锁瓶颈已经消失。

## 5. 验证与性能采样计划

源码阶段：

- source guard 要求 `stage_atomic_write`/`PendingAtomicWrite::commit` 存在；
- source guard 要求 post-publication commit error 经过 bounded-buffer 流式核对，不分配第二份 artifact；
- source guard 拒绝 `File::create`、`BufWriter`、私有 backup/restore、`temporary_archive_path` 和
  session-local `fs::rename`；
- 确定性回归在单 worker gate 下提交嵌套新路径，证明 admission/queue 阶段 parent 不存在，worker
  执行后才创建并发布 archive；
- 保留 capacity rejection、before-start cancel、expired Runtime owner 与 stale generation 回归；
- 执行 scoped `rustfmt --check`、source assertions、trailing whitespace 和 scoped diff check。

受管动态阶段仍由协调器在非 `C:` 目标执行：

- `1/64/512 MiB` artifact，`1/1k` write burst，`0/10/1000 ms` slow I/O；
- 记录 caller wall、queue age、stage write/sync、CAS wait/hold、commit/parent sync、temp/backup 残留、
  service p50/p95/p99、RSS 与 CPU；
- Windows host 允许时再采 WPR I/O/CPU/功耗；没有真实回执时不声明性能、功耗或跨引擎接近值。

## 6. 完成定义

- dynamic-scene 不再拥有第二套 atomic-file 平台算法；
- backpressure 拒绝和 queued 状态不创建 parent、staging、backup 或 target；
- stale/失败 publication 保持 last-good，并由共享 RAII owner 清理 staging；
- 成功返回遵循共享 file/parent durability 合同；
- failure 仅记录为 source implemented/static checked，直到受管行为与产品矩阵取得终态。

## 7. 2026-08-26 源码实现与静态结果

- `session/io/atomic.rs` 已硬切到 `stage_atomic_write -> session CAS -> commit`，私有
  `File::create`、`BufWriter`、temp/backup/restore 和 `fs::rename` 路径全部删除；
- 缺失 parent 只做最近存在祖先的 canonical identity 计算，目录创建由 I/O worker 中的共享 stage
  owner 执行；新增单 worker gate 回归锁定 queued/admitted 阶段零目录副作用；
- commit 错误路径使用固定 64 KiB 栈缓冲流式核对已发布目标，不分配第二份 artifact；已发布内容会
  推进 session path revision，但原 durability 错误仍返回调用方；
- `support.rs` 删除私有临时文件命名器；源码守卫要求唯一 shared atomic owner，并拒绝旧算法回流；
- scoped `rustfmt --check`：4/4 Rust owner 通过；静态 source contract：9/9；生产旧 owner 扫描：
  0 命中；owner 行数为 atomic 287、support 13、writer 211、writer tests 249，均低于结构预算；
  scoped `git diff --check` 通过，仅有工作区 LF/CRLF 提示。

以上是 current-source 静态证据。受管 Cargo、fault-injection 上行回归、slow-I/O/write-storm、
1/64/512 MiB、CPU/RSS/WPR/功耗与跨引擎耗时数据均未执行，本报告不作相应通过或性能结论。
