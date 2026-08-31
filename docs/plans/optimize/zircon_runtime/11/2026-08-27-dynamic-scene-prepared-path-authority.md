# Runtime11 Dynamic Scene Prepared Path Authority 架构与性能计划

> 日期：2026-08-27
> 所属 failure：`runtime/11/failure-2026-07-22-dynamic-scene-session-bounded-async-io.md`
> 状态：`source_implemented_static_passed_managed_validation_pending`

## 1. 当前源码重审

`RuntimeSessionArchiveWriter::try_submit` 虽已把 parent 创建、staging、fsync 与 replace 放入 Runtime
I/O worker，仍会在 admission 前调用 `canonical_archive_target`。该函数执行 current-dir 解析、逐级
physical ancestor canonicalization 和 alias 收敛，所以主调用线程仍可能阻塞在 filesystem metadata / reparse
point 路径上。把 canonicalize 机械移动进 writer closure 会使 lane 在解析前只能按 lexical key 排队；两个
alias 可并发进入不同 key，破坏 same-physical-path generation 和 stale-write authority。

前一 reader 切片新增的 `RuntimeSessionArchivePath` 只做 lexical absolute normalize。继续扩展该类型会复制
已经存在于 `asset::project::ProjectPaths` 的完整 physical path owner。当前仓库的 `ResolvedProjectPath` 同时
保留 filesystem operation path 与 diagnostic display path，`ResolvedProjectPathIdentity` 对 Windows junction、
SUBST、verbatim prefix、case alias 和未创建尾段提供统一有序 identity；对应 tests 已覆盖 existing alias、
uncreated tail、broken symlink、drive-relative rejection 与 operation/display 分离。

同步 facade 调用图也已量化：session owner 内有 95 个同步 load 调用；session 外 34 个
`RuntimeSessionArchive::load_from_path` 引用全部属于 crate 内测试，没有 production consumer。因此本切片
不机械地给 95 个离线 archive 运算函数注入 `CoreHandle`，先收敛已经存在的 async reader/writer 主链。

## 2. 上游证据

### Unreal Engine

- `Engine/Source/Runtime/CoreUObject/Public/Misc/PackagePath.h` 的 `FPackagePath` 明确区分
  `MountedPath`、`PackageOnlyPath` 与 `LocalOnlyPath`；低层 package resource manager 消费已经准备的
  `FPackagePath + EPackageSegment`，不在每次 loader admission 中重新解释裸字符串。
- `Engine/Source/Runtime/CoreUObject/Private/Misc/PackagePath.cpp` 的 `TryFromMountedName` 先通过 mount
  registry 取得 package root、file root 与 relative path，再构造 mounted identity；`FromMountedComponents`
  是已解析组件的低成本入口。
- `PackagePathTest.cpp` 与同目录 package-name tests 覆盖 extension、有效/无效 package path 和 path
  projection，而不是只覆盖一次成功读取。

### Bevy

- `dev/bevy/crates/bevy_asset/src/path.rs` 的 `AssetPath` 把 `AssetSourceId`、normalized relative path 与
  label 保存在 typed descriptor；`resolve` / `resolve_embed` 的大量 inline tests 覆盖 rooted、relative、
  `.`、`..`、source 和 label 组合。
- `dev/bevy/crates/bevy_asset/src/io/source.rs` 让 source registry 映射到 reader/writer owner，I/O 层消费
  已选择 source 的 path，而不是让每个提交点 canonicalize 本地文件系统。

### Fyrox

- `dev/Fyrox/fyrox-resource/src/manager.rs` 的 resource manager 先以 normalized resource path 查找共享
  handle，再进入 `Pending -> Ok/LoadError` worker 状态；path identity 与异步执行 owner 分离。

共同语义是：路径解析/mount identity 先于异步 I/O admission，并由独立 path authority 单点拥有；I/O
队列只消费稳定 typed identity。Zircon 采用现有 `ProjectPaths` 作为该 authority，不复制 UE mount registry。

## 3. 基础层修复

现有 `BoundedKeyedIoLane` 把 key 固定为 `Arc<str>`，迫使非 Unicode path 或平台化 equality 在调用点转成
lossy/lowercase 字符串。新增 domain-neutral `BoundedKeyedIoKey`：

- wrapper 以 `Arc<dyn erased key>` 保留任意 `Eq + Send + Sync + 'static` typed value；
- equality 只在相同 concrete key type 间委托领域 `Eq`，不同类型永不碰撞；
- string constructor 保留现有 preference/operation caller 的零迁移调用形态，但不是 path 的 canonical
  identity owner；
- lane queue、active entry 与 fence prerequisite 统一携带 wrapper，不增加第二套 coalescing 状态机。

该设计不要求 `Hash`，因为当前 lane 本来就在有序 queue 中线性比较同 key；因此
`ResolvedProjectPathIdentity` 可以原样复用其 Windows `CompareStringOrdinal` 语义，不生成第二份 hash/casefold
规则。

## 4. Dynamic Scene 硬切

- 删除重复的 `RuntimeSessionArchivePath` / `RuntimeSessionArchivePathError`。
- `RuntimeSessionArchiveReader::try_submit` 与 `RuntimeSessionArchiveWriter::try_submit` 只接受
  `ResolvedProjectPath`；调用方必须在 project/path 边界完成一次 physical resolution。
- reader weak single-flight map 使用 `BTreeMap<ResolvedProjectPathIdentity, Weak<_>>`；lane 使用同一 identity
  的 typed key，只复用尚未 terminal 的 request；成功或失败 terminal 在下一次同 path 提交时惰性替换，或
  在最后 submission Drop 时精确退休，因此文件更新和失败恢复都会产生新 ticket。
- atomic write authority 从永久 retained revision map 改为
  `BTreeMap<ResolvedProjectPathIdentity, Weak<ArchivePathWriteState>>`；同步 direct-save 和 async writer 共享
  同一 physical identity。每个 live path state 私有持有 revision mutex，最后 reservation/ticket Drop 时用指针
  身份退休对应 weak entry，容量拒绝不会留下唯一 path 状态。
- artifact lineage 共享唯一 revision allocator、publication revision 与 mutex gate。clone 后各自修改的分支
  也会取得不同且单调的 revision。staging 在 gate 外并行；最终 stale check、atomic replace 和 durability
  reconciliation 只对同 path 或同 lineage 串行，保证一个 lineage 即使写入不同目标也不能让旧 revision 在
  新 revision 之后发布。进程级 path registry mutex 不覆盖任何文件 I/O。
- writer admission 只做 O(path bytes) retention accounting、O(log P) revision lookup 和 lane reservation；
  不调用 `std::fs`、`canonicalize`、metadata、current-dir 或 parent creation。

不保留 raw-`Path` async overload、lexical alias 或 compatibility wrapper。同步 archive facade 仍是明确的
离线同步 API，后续只有发现 production consumer 时才通过统一 session I/O facade 硬切，不能逐函数注入
runtime handle。

## 5. 复杂度与测量

| 操作 | 目标复杂度 | I/O / allocation 边界 |
|---|---:|---|
| physical path preparation | `O(depth + path bytes)` | project/path boundary 一次 filesystem resolution |
| typed key clone | `O(1)` | 1 次 `Arc` clone |
| writer admission | `O(log P + Q * K)` | 无 filesystem；`P` 为 live physical path states，`Q` 为同 epoch queue，`K` 为 key compare |
| same-path reader lookup | `O(log R * K)` | 无 filesystem；弱 request entry |
| atomic staging | `O(n)` | Runtime I/O worker；不同 path/lineage 可并行写 staging file |
| final publication | `O(n)` | 仅同 path 或同 lineage 串行 commit/fsync/reconciliation；global registry lock 不参与 |

`BoundedKeyedIoLane` 的既有 coalescing 是线性 queue partition，本切片不冒充它为 O(1)。受管性能阶段需比较
string key 与 typed `ResolvedProjectPathIdentity` 在 key 长度 32/260/4096、queue 1/1k/100k 下的 admission
P50/P95、comparison count 和 allocation；同时用 junction/case alias 写入证明同一 physical path 最终发布
不超过一个 generation。产品矩阵继续记录 caller wall time、queue wait、service latency、CPU、RSS、WPR 与
功耗。没有这些数据前不声明性能或功耗接近其它引擎。

## 6. 验证计划与源码结果

源码已完成：

- generic lane 新增 3 项 typed-key equality/domain/string-conversion 测试与 1 项 lane coalescing 行为测试；
- reader 7 项测试保留，并把 path owner 与 single-flight 用例强化为两个 lexical input 收敛到同一 resolved
  physical identity、ticket 与 result reservation；新增失败 terminal 后文件出现可重试、成功 terminal 后文件
  更新可重读，旧 submission 保持存活也不能遮蔽新 request；
- writer 既有测试保留，并把同 path generation 用例强化为 physical alias coalescing；新增 prepared-target
  source guard、64 个 rejected unique path retirement 回归和 staging-before-path-lock 源码守卫，锁定 admission
  closure 前无 filesystem resolve/canonicalize/metadata、容量拒绝不扩张全局注册表；
- atomic authority 使用 weak physical-path state、per-path revision mutex 与 lineage publication gate；reader weak
  request map 使用相同 typed identity 且只复用 nonterminal request；`RuntimeSessionArchivePath`、caller-side
  canonical helper、lossy identity helper、raw async path overload 和 lane `Into<Arc<str>>` 在目标源码范围均为
  0 命中；
- 独立源码复审首轮给出 `Critical 0 / Important 3`：terminal request 永久复用、global revision map 无界、global
  mutex 覆盖 staging/commit/reconciliation。三项均按上述状态生命周期与锁域修复，并增加行为/源码回归；
- follow-up 确认前述 3 项关闭后发现 clone fork 可生成同 lineage、同 revision、不同 payload；lineage-owned
  revision allocator 已消除重复 revision，并新增 clone-fork 跨 path stale-publication 回归；
- 最终 follow-up 复审为 `Critical 0 / Important 0 / Minor 0`；该结论仅接受源码结构与静态回归设计，不替代
  受管 Cargo、Windows physical-alias fixture 或产品 profile 回执；
- scoped rustfmt、diff check 和 source guards 通过；目标实现文件最大 634 行，低于 1000 行结构预算。

受管阶段仍需执行 failure 声明的 Cargo filter、Windows junction/SUBST/case alias fixture，以及 slow-I/O、
1/1k/100k queue、CPU/allocation/RSS/WPR/功耗矩阵。当前没有这些动态数据，不声明性能瓶颈或功耗已消失。
