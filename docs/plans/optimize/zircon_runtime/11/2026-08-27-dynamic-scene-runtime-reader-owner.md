# Runtime11 Dynamic Scene Runtime Reader Owner 架构与验证计划

> 日期：2026-08-27
> 所属 failure：`runtime/11/failure-2026-07-22-dynamic-scene-session-bounded-async-io.md`
> 状态：`source_implemented_static_passed_managed_validation_pending`

## 1. 重审结论

现有 `RuntimeSessionArchive::load_from_path` 在 caller 线程执行 `File::open`、metadata、最多
512 MiB 的流式 JSON 反序列化、normalize 和 validate。直接把该函数包装进 task 只能移动执行位置，
不能解决完成结果在 lane terminal 后继续驻留却不再计入预算的问题，也不能给同 path 请求建立统一
identity、取消和 shutdown 合同。

最低共享层缺口分为两层：Runtime11 没有“任务完成后仍由结果句柄持有”的通用 RAII 字节预算；
dynamic scene I/O 没有 Runtime-owned read request owner。此次先补这两层，不修改同步 path facade 的
行为，也不保留 process-global reader constructor。

## 2. 上游证据

### Unreal Engine

- `dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/Serialization/AsyncPackageLoader.h`
  的 `LoadPackage` 返回独立 request ID，同一 package 可以关联多个 ID；loader 集中拥有 queued/loading
  计数、cancel、flush 和 process 边界。
- `dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/Serialization/AsyncLoading2.cpp`
  的 `FCompletedPackageRequest` 把 package result、request IDs 和 completion callbacks 聚合到 loader owner，
  `AsyncPackageLookup` 与 pending request 集合不由调用点复制。

### Bevy

- `dev/bevy/crates/bevy_asset/src/server/mod.rs` 的 `AssetServer::load` 立即返回 strong handle；相同 path 已有
  handle 时复用，不重复 spawn load task；`load_internal` 在 I/O task 中取得 reader 并发布 typed failure。
- strong handle 生命周期决定结果是否继续保留，调用方 drop handle 可以撤销尚未完成且无人持有的 load。

### Fyrox

- `dev/Fyrox/fyrox-resource/src/manager.rs` 的 `find_or_load` 对已有 path 返回共享 resource handle，否则创建
  `Pending` resource 并由统一 task pool 加载；task 将共享 state 原子提交为 `Ok` 或 load error。
- 同文件 `resource_wait_context_is_all_loaded`、`resource_manager_state_request` 等 inline tests 覆盖 pending、
  shared resource 和 wait-context 语义。

三者共同语义是：提交句柄、物理加载请求和完成资源寿命分离；path/package identity 由统一 owner 管理；
完成状态显式发布。Zircon 不复制 UObject、ECS asset storage 或 Fyrox resource registry，而把该语义压缩到
session archive 的最小 Runtime11 owner。

## 3. 基础能力决策

新增 `RetainedByteBudget` / `RetainedByteLease` 到 `core::runtime::tasks`：

- `try_reserve`、lease clone 和最后 owner Drop 都是 O(1) mutex 临界区；
- lease 的最后一个 clone Drop 才归还字节，适用于 task terminal 后仍保留的 immutable result；
- byte capacity 与 active lease capacity 同时限制大结果和大量小结果；
- `close` 只拒绝新 reservation，不撤销仍被调用方合法持有的结果；
- diagnostics 区分 byte/lease capacity、retained reservation、active lease 和 closed。

该类型不认识 archive、asset 或 editor 类型。dynamic scene reader 复用它，不再复制 text raster、operation
或 render loader 中已有的私有计数器形态。现有私有预算的后续收敛需独立盘点，本切片不横向改写。

## 4. Reader 所有权与状态机

`RuntimeSessionArchiveReader` 的 production constructor 只接受 `CoreHandle`，并把工作投到该 Runtime 的
I/O pool。后续 prepared-path authority 切片已删除临时 `RuntimeSessionArchivePath`，reader 现在只接受
`asset::project::ResolvedProjectPath`。caller 在 project/path boundary 完成一次 physical resolution；reader
以 `ResolvedProjectPathIdentity` 同时作为 weak request map key 与 Runtime11 lane typed key，admission 本身
不执行 canonicalize、metadata 或 open。

每次新 path 请求执行：

1. 验证 Runtime owner 与 reader accepting 状态；
2. 在弱引用 request map 中只复用尚未 terminal 的同 path physical request；下一次同 path submit 会先惰性
   替换 terminal entry，失败恢复或文件更新必须创建新 ticket；
3. 预留该请求允许读取的最大 archive bytes；
4. 同一 `max_entries` 同时约束存活 result reservation 数与 lane entry 数，再进入
   `BoundedKeyedIoLane` 的 metadata/deadline admission；
5. activate 后仅在 I/O worker 内 open、metadata、bounded streaming decode、normalize、validate；
6. success 把 byte lease 移交给 cloneable immutable result，failure/cancel/panic 自动 Drop lease；
7. 最后一个 submission Drop 会尝试 before-start cancel，并仅在 map 仍指向自己时删除 weak entry，避免旧
   terminal submission 删除后来重试的 request；完成 result 的最后一个 clone Drop 后归还预算。

同 path 的并发 nonterminal submission 返回同一 physical ticket；terminal 后再次提交总是新 request。
`cancel_shared_before_start` 的命名显式表明取消作用于共享请求。当前 deadline 也是 physical request 属性，
首个 admission 决定该共享工作的 deadline；本层没有
伪造每 subscriber 独立 terminal ID。若产品需要 UE 式多 request ID/priority/deadline，必须在 Runtime11
增加通用 subscription terminal 层后再扩展，不能把第二套 ticket 状态机塞入 scene 模块。

## 5. 复杂度、内存与限制

| 操作 | 复杂度 | 常驻边界 |
|---|---:|---:|
| 新 path admission | `O(log R * K)` | 1 request entry + 1 result reservation |
| 同 path nonterminal reuse | `O(log R * K)` | 不增加 lane entry 或 result reservation |
| terminal replacement | `O(log R * K)` | 旧 weak entry 退休后创建新 request/reservation |
| streaming decode | `O(n)` | 输入上限为 `max_archive_bytes`，buffer 固定 64 KiB |
| result clone | `O(1)` | archive payload与byte lease均为共享引用 |
| request retirement | `O(log R * K)` | map 只保留 weak entry，最后 submission Drop 删除 |

`R` 为存活 physical request identity 数，`K` 为平台 path compare 成本。这里刻意使用现有
`ResolvedProjectPathIdentity::Ord`，不为 Windows path 再造 hash/casefold 规则。

默认 `max_archive_bytes = 512 MiB`、`max_retained_result_bytes = 512 MiB`，因此默认最多保留一个最大规格
不同 path 结果；较小产品规格可以显式降低单 archive 上限或提高总 result budget。diagnostics 的
`retained_result_bytes` 表示已保留的 quota，不是进程 RSS 采样。

JSON decode 后的结构体、索引和 allocator overhead 可能高于源文件字节数。固定 schema 和输入上限使
复杂度保持 `O(n)`，但本切片没有实际 512 MiB RSS amplification 数据，因此不宣称 RSS/功耗已经达到
其它引擎经验值。若 profile 证明 amplification 使产品预算失真，下一步应实现 payload retained-size
estimator、分区 arena 或 chunked archive schema；禁止用任意倍率 magic constant 冒充测量。

## 6. 测试与性能计划

已挂载但尚未取得受管 Cargo 回执的 13 项确定性回归：

- project path authority 将 lexical dot-segment alias 收敛到同一 physical identity；
- 同一 resolved physical identity 的两个 submission 共享一个 ticket 和一份 result reservation；
- 不同 path 在 held result 占满预算时 typed reject，最后 result Drop 后可重新 admission；
- queued cancel 不执行 filesystem closure并立即释放 reservation；
- missing file 保留 typed I/O error且释放 reservation；
- missing file terminal 在目标出现后产生新 ticket 并成功重试；
- successful terminal 不会缓存成永久结果，文件更新后产生新 ticket 并读取新 revision；
- expired Runtime owner 在 admission 和 filesystem work 前拒绝；
- generic byte lease 在最后 clone Drop 前不归还，closed budget 不撤销 live lease；
- generic lease capacity 在 byte capacity 尚有余量时仍拒绝过量小结果。

静态阶段已完成 scoped rustfmt、无 private thread/channel/unsafe 扫描、caller-side filesystem source
guard、module size、trailing whitespace 和 diff check。`job_system_boundary` 独立测试 3/3 通过，报告
owner 15/15、behavior anchors 62/62、mirror docs 5/5、oversized modules 0、direct-Rayon owner 2/2、
runtime-to-editor dependency 0、`risks = []`。受管阶段仍需执行 failure 中声明的
`dynamic_scene_session` Cargo filter，以及 1/64/512 MiB、1/1k request burst、0/10/1000ms slow I/O 的
queue wait、service latency、CPU、allocation、RSS、WPR/功耗矩阵。

同步 path facade 尚未迁移，active cooperative cancellation 也未实现；本 failure 继续保持 open，
不得据此提交 Runtime11 里程碑或发送“性能已达标”的企微结论。

Prepared-path authority 的 Unreal/Bevy/Fyrox 对照、typed lane key 设计、实现结果和后续动态矩阵见
`docs/plans/optimize/zircon_runtime/11/2026-08-27-dynamic-scene-prepared-path-authority.md`。
