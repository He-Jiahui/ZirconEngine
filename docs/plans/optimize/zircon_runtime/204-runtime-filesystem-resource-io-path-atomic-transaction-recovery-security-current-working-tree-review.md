---
related_code:
  - zircon_runtime/crates/zr_resource/src/io
  - zircon_runtime/src/core/resource
  - zircon_runtime/src/asset/project
  - zircon_runtime/src/asset/artifact
  - zircon_runtime/src/asset/migration
  - zircon_runtime/src/asset/watch
  - zircon_runtime/src/platform/preferences
  - zircon_runtime/src/graphics/shader/variant_cache
  - zircon_runtime/src/ui/template/asset/compiler/cache
  - zircon_runtime/src/plugin/export_build_plan/materialize
  - zircon_runtime/src/bin/zircon_font_sdf_bake
  - zircon_editor/src/core/project
  - zircon_editor/src/core/editing/engine/journal/durable
  - zircon_editor/src/core/recovery/autosave
  - zircon_editor/src/core/recovery/session_guard
  - zircon_runtime_interface/src/hub_protocol/recent_projects
  - zircon_runtime_interface/src/serialization/text/canonical_spool
tests:
  - zircon_runtime/crates/zr_resource/src/io/atomic_file/tests
  - zircon_runtime/crates/zr_resource/src/io/transaction/engine/tests
  - zircon_runtime/crates/zr_resource/src/io/transaction/recovery/tests
  - zircon_runtime/src/asset/tests
  - zircon_editor/src/core/project
  - zircon_editor/src/core/recovery
plan_sources:
  - docs/plans/optimize/zircon_runtime/25-filesystem-path-uri-vfs-mount-watch-sandbox-atomic-io-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/99w-runtime-asset-registry-index-persistence-rebuild-incremental-query-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/203-runtime-preference-config-storage-authority-durability-migration-multiprocess-product-integration-current-working-tree-review.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/GenericPlatform/GenericPlatformFile.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/HAL/PlatformFileManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/IO/IoDispatcher.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/AsyncFileHandle.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/Misc/PackageName.h
  - dev/UnrealEngine/Engine/Source/Runtime/PakFile/Public/IPlatformFilePak.h
  - dev/bevy/crates/bevy_asset/src/io/mod.rs
  - dev/bevy/crates/bevy_asset/src/io/source.rs
  - dev/Fyrox/fyrox-resource/src/io.rs
  - dev/godot/core/io/file_access.h
  - dev/godot/core/io/dir_access.h
  - dev/godot/core/io/resource_loader.h
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderPipeline/RenderPipelineGlobalSettingsUtils.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Material/AssetReimportUtils.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Runtime204 - Filesystem / Resource I/O / Path / Atomic Transaction / Recovery / Security 当前工作树复审

> 审查日期：2026-08-31
> 工作树基线：`630d66c362013e3b5b72f97362ad56fc54ff6d8c`；工作树仍有其他在途改动，本文不回退、不覆盖。
> 审查性质：current-working-tree source review + refactor plan；只写文档，不修改生产 Rust/Cargo/ABI/test/UI 代码。
> 排除范围：Tooling；未运行 Cargo、未做故障/性能/跨文件系统实测，也未查询、轮询、等待或实时跟踪协调器。
> 前置文档：[Runtime25](25-filesystem-path-uri-vfs-mount-watch-sandbox-atomic-io-review.md)、[Runtime99w](99w-runtime-asset-registry-index-persistence-rebuild-incremental-query-product-integration-current-source-review.md)、[Runtime203](203-runtime-preference-config-storage-authority-durability-migration-multiprocess-product-integration-current-working-tree-review.md)

## 1. 结论

当前工作树已经完成一次真实的资源基础迁移：旧 `zircon_runtime/src/core/resource/io` 的大块实现正在被删除，新的 `zircon_runtime/crates/zr_resource` 成为资源身份、registry、generation 与 I/O 事务的 canonical crate。旧报告中“`ResourceIo` 无实现、`AssetIoDriver` 为空”的判断只对历史状态成立；当前 `ResourceIo` 已不再存在，`zr_resource` 的 atomic file 和 durable multi-file transaction 也已经具备工程价值，不能再把它们当作临时占位。

但是，当前完成的是**可靠发布内核**，不是工程级资源文件系统。`zr_resource::io` 对外只公开 `atomic_write` 和 `atomic_write_new`；多文件 transaction、recovery、owner lock、path identity 和 observation 仍属于内部 assembly。没有统一的 `FileSystemProvider`、`AssetSource`、`MountRegistry`、opened handle、异步/批量 I/O、watch provider、包容器、远端源或 capability negotiation。上层仍在 Runtime、Editor、App 和 Runtime Interface 中各自直接使用 `std::fs`，形成互相矛盾的 durability、错误、清理、路径安全和恢复语义。

本轮的核心裁决是：

1. **P0-001：资源 I/O 没有唯一生产权威。** `zr_resource` 只拥有写入内核，读取/列举/监听/挂载/调度不存在；业务代码可以绕开所有 root、capability、generation、metrics 和恢复规则。
2. **P0-002：durability 结果不一致且不可组合。** durable transaction 有 `DurableCommitDisposition`，普通 `atomic_write` 仍返回裸 `io::Result<()>`；Editor session guard 还明确返回 Windows durability uncertainty。调用者无法用一个 receipt 判断“已发布、已落盘、待恢复、待清理”是否成立。
3. **P0-003：安全边界仍是 path check，而不是 opened capability。** `symlink_metadata`/`canonicalize` 之后再次按原 path 打开、替换或删除，无法抵抗 check/open 之间的目录项替换、reparse/junction、hard-link 和协作进程竞争。
4. **P0-004：资源发布与 I/O 终态仍可能分离。** 资产扫描/导入路径可能先安装 candidate/live generation，再在 `ensure_durable()` 处报告 recovery deferred；该跨报告根因由 Runtime99w 继续拥有，本篇只拥有底层 receipt 和 provider 约束，不重复计数。

因此，本轮新增 **3 项唯一 P0、61 项 P1、14 项 P2**。P0 为 `FSIO-P0-001..003` Open；`FSIO-P0-004` 继承 Runtime99w，标记 Cross-owned。P1 为 **56 Open / 5 Partial / 0 Closed**；P2 为 **13 Open / 1 Partial / 0 Closed**。本篇不把旧 Runtime25 的 40 P1/12 P2 机械相加，而是按当前 `zr_resource` hard-cut 后重新裁决；Runtime25 保留为架构基线，Runtime204 是当前状态和迁移约束的 canonical review。

## 2. 审查边界与证据冻结

统计口径：UTF-8 物理行、非空行、bytes；`tests` 为 `#[test]` 标记，`ignored` 为 `#[ignore` 标记。fingerprint 为 normalized lowercase relative path 排序后，对 `path + NUL + lowercase(file SHA-256) + LF` 集合再做 SHA-256。

| 范围 | files | lines | non-empty | bytes | tests | ignored | fingerprint |
|---|---:|---:|---:|---:|---:|---:|---|
| `zr_resource/src/io` production-like implementation | **27** | **5,534** | **5,121** | **185,642** | **19** | **0** | `2ad1b69353422ea6b205a956ff959082da100c455880b431ff15bbe956608967` |
| `zr_resource/src/io` test/support files | **12** | not folded into production total | not folded | not folded | counted separately | counted separately | source paths recorded above |
| Runtime production-prefix direct OS-I/O candidates | **105** candidate files; **55** write files / **339** writes; **81** read files / **190** reads | candidate scan | candidate scan | candidate scan | first `#[cfg(test)]` removed; filenames containing `test` excluded | n/a | source scan on current tree |
| Editor production-prefix direct OS-I/O candidates | **78** candidate files; **51** write files / **292** writes; **54** read files / **107** reads | candidate scan | candidate scan | candidate scan | same exclusion | n/a | source scan on current tree |
| App production-prefix direct OS-I/O candidates | **17** candidate files; **14** write files / **97** writes; **12** read files / **26** reads | candidate scan | candidate scan | candidate scan | same exclusion | n/a | source scan on current tree |
| Runtime Interface production-prefix direct OS-I/O candidates | **2** candidate files; **2** write files / **26** writes; **2** read files / **4** reads | candidate scan | candidate scan | candidate scan | same exclusion | n/a | source scan on current tree |

候选扫描不是“所有命中都违规”：provider/backend、journal、测试夹具本来应调用 OS；它证明当前没有机器可读的 direct-fs allowlist、owner 和 root-capability 分层，无法自动区分合法底层实现与业务旁路。生产 `zr_resource` 统计不把测试目录和文件名含 `test` 的文件纳入 27 文件总集，但测试证据仍在报告的 related paths 中。

对当前 `zr_resource/src/io` 的负向符号扫描也没有发现 `AssetSource`、`Provider`、`Mount`、`Reader`、`Watch`、`async`、`Range`、`mmap`、`Cancellation`、`Deadline` 或 `Priority` 合同；`FileSystem` 仅为文件系统错误/平台局部命名，不能视为 provider 已存在。该负向证据支持“可靠发布内核已迁移、完整资源 I/O provider 尚未出现”的裁决。

参考 revision：Bevy `fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`、Godot `8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、Fyrox `8d815db36494f1badb347547dfc7094bf4fbbdf8`、Unity Graphics `a7e4c051d256a781ab362c64316b125a1e104694`。`dev/UnrealEngine` 无独立 Git 元数据，以所列文件和当前物理内容为证据。

## 3. 当前拓扑与迁移事实

```text
ResourceLocator / AssetReference / ResourceRegistry
                    |
                    +--> zr_resource::io
                           +--> atomic_write / atomic_write_new (public)
                           +--> durable transaction / recovery / owner lock (internal)
                           +--> no reader / provider / source / mount / scheduler

Runtime Asset / Config / Preference / Shader Cache / UI Cache / Plugin Export
       |             |            |            |           |          |
       +-------------+------------+------------+-----------+----------+
                             direct std::fs and local writers

Editor Scene / Project / Autosave / Journal / Session Guard
       |                 |             |          |
       +-----------------+-------------+----------+
                   independent temp/rename/fsync implementations

Runtime Interface Recent Projects / Canonical Spool
       |
       +--> owns low-level filesystem, lease, quarantine and cleanup in ABI-adjacent crates
```

旧 Runtime25 的 `ResourceIo`/`AssetIoDriver` 空表面已被 hard-cut，这是进步；但删除旧 trait 不等于新的 provider architecture 已落地。当前 `zr_resource/src/io/mod.rs` 仅 re-export `atomic_write`、`atomic_write_new` 和 crate-private assembly helpers；`transaction/mod.rs` 公开的 transaction API 仍通过 `zr_resource::assembly` 进入 Runtime。资源身份/registry 有 generation 与 lease，物理文件却没有同等的 opened-handle/generation binding。

## 4. 可以保留的基础

| 基础 | 当前证据 | 保留条件 |
|---|---|---|
| `create_new` staging | sibling staging、flush、`sync_all`、create-new publish，避免半写目标 | 由 provider 接收 root capability 和 source lease，返回 typed receipt |
| Unix parent sync / Windows native replace | Unix 目录链 sync；Windows UTF-16、长路径、`MoveFileExW`/`ReplaceFileW`、backup restore | 对外暴露平台 durability level，不能用统一 `Ok(())` 掩盖差异 |
| Durable multi-file transaction | immutable intent、checksummed WAL、bounded journal、owner lock、commit point、rollback、restart recovery | 将 path-only operations 换成 handle-relative provider operations，加入 generation/CAS |
| Domain recovery policies | Project/Migration/IBL policy 对 registry、artifact、asset root、bundle layout 做 allowed-target 校验 | policy 输入必须来自 mount/root capability，不能信任绝对 path 字符串 |
| PathIdentity | 规范化 existing/missing tail、alias/overlap 检查、journal-owner separation | 增加 volume/file ID 与 opened handle identity，明确 hard-link/reparse 规则 |
| Bounded journal | 64 MiB frame、128 MiB journal、torn-tail truncation、unsupported evidence fail-closed | foreign/stale entry 需 bounded quarantine/operator receipt，不能永久阻塞整个目录 |
| Fault/recovery tests | replace ambiguity、namespace overlap、restart、rollback、journal evidence、profile fixtures | 补 fresh-process、power-loss、different volume/network/reparse/ACL 矩阵 |
| Asset generation publication | Runtime99w 的 generation/preparation/retry/reconciliation 真实存在 | publication 必须等待 terminal durable receipt，或明确 publish-as-rebuildable 语义 |

## 5. P0 当前裁决

| ID | 状态 | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| FSIO-P0-001 | Open | `zr_resource::io` public surface 只有两个 byte-write 函数；没有 `Reader/Writer/Stat/Directory/Watch/Source/Mount/Provider`，业务仍直接调用 `std::fs` | Runtime-owned `FileSystemProvider`、`AssetSource`、`MountRegistry`、opened handle、async scheduler 和 direct-fs lint/allowlist；任何资源 I/O 只能从 provider 进入 |
| FSIO-P0-002 | Open | durable transaction 返回 `Durable/CommitRecoveryDeferred/CleanupDeferred`，普通 atomic write 返回 `io::Result<()>`；Editor session guard 在 Windows 明确有 `PublishedWithDurabilityUncertainty`，各 writer 仍各自解释 success | 统一 `IoOperationReceipt`：publication point、durability level、cleanup/recovery action、generation、retryability、provider instance；禁止裸 `Ok(())` 代表产品 durable |
| FSIO-P0-003 | Open | `symlink_metadata`/`canonicalize` 后按原 path 再 open/rename/remove；owner lock 也是 metadata 后 `OpenOptions::open`；无 no-follow、root-relative opened handle、file ID 绑定 | `RootCapability` + handle-relative no-follow provider，打开时一次性验证 volume/file ID/reparse/link policy，所有后续 mutation 只接受 capability-bound handle |
| FSIO-P0-004 | Cross-owned | Runtime99w 的 scan/import 可先 install/publish candidate，再在 `ensure_durable()` 报 deferred；底层 transaction 已能表达 deferred，但产品 publication 未统一等待 | 由 Runtime99w 收口：candidate generation、catalog、artifact manifest 和 durable receipt 必须在同一 publication protocol 内提交或可证明为 rebuildable |

## 6. P1 详细差距

### 6.1 Provider、source、mount 与读取合同

| ID | 状态 | 证据与重构方向 |
|---|---|---|
| FSIO-P1-001 | Open | public crate 没有 FileSystem provider trait；建立 `FileSystemProvider` 和 object-safe erased form |
| FSIO-P1-002 | Open | 没有 source instance registry；把 `file/memory/embedded/pack/http/remote` 作为注册实例，不再由 scheme enum 固定 match |
| FSIO-P1-003 | Open | 没有 mount priority/override/unmount lifetime；建立 immutable mount snapshot、priority、collision policy、quiescence 和 generation |
| FSIO-P1-004 | Open | `ResourceLocator` 仅表达 logical locator，不能解析到 source/mount/capability；resolution 返回 pinned `ResourceAddress` |
| FSIO-P1-005 | Open | 没有 stat/list/open handle；读路径应返回 metadata、content identity、source generation 和 stable error |
| FSIO-P1-006 | Open | 没有 streaming/seek/range/mapped read；大纹理、shader package、font 和 cooked chunk 仍会走全量 Vec |
| FSIO-P1-007 | Open | 没有 async batch I/O、cancel、deadline、priority、request status；同步 API 无法承载加载风暴和 GPU upload pipeline |
| FSIO-P1-008 | Open | 没有 watcher provider/source event；Runtime25 的 bounded watcher 仍与路径映射和具体 OS watcher 分裂 |
| FSIO-P1-009 | Open | 没有 source capability negotiation；至少需要 read/write/list/watch/seek/range/map/atomic/transaction/durability/remote/offline/read-only |
| FSIO-P1-010 | Open | 没有 package/pak/cooked/remote backend；Unreal IoDispatcher/Pak、Godot pack/zip 与 Bevy source layering 的产品入口均不存在 |
| FSIO-P1-011 | Open | 没有 provider lifecycle、tick、health、drain、unmount receipt；热重载和设备/网络变更不能安全拆除 source |
| FSIO-P1-012 | Open | 没有统一 I/O scheduler budget、tenant fairness、throughput/latency/error/cancel metrics；无法证明“性能优于 Unreal” |

### 6.2 Path、encoding 与安全边界

| ID | 状态 | 证据与重构方向 |
|---|---|---|
| FSIO-P1-013 | Partial | `ProjectPaths`/`RelPath` 已有 root admission、portable segments 和 display/operation 分离；这些规则未进入所有 provider、Hub、Editor 和 transaction |
| FSIO-P1-014 | Open | atomic/transaction API 接受任意 `&Path`，没有 `RootCapability`、principal、mount identity 或 operation scope |
| FSIO-P1-015 | Open | check/open/remove/rename 是 path-based TOCTOU；把所有 final operation 改为 root-relative opened directory handle |
| FSIO-P1-016 | Open | `symlink_metadata` 拒绝 final symlink，但没有统一 Windows reparse/junction、Unix hard-link、ADS/xattr/ACL policy |
| FSIO-P1-017 | Open | `PathIdentity` 以规范化文本/native path 比较，没有 volume/file ID；无法绑定到已打开对象 |
| FSIO-P1-018 | Open | external writer 可在 digest check 后、replace 前修改目标；transaction 没有 universal expected file identity/CAS |
| FSIO-P1-019 | Open | journal 使用 absolute UTF-8 paths；non-UTF8/native path 被显式拒绝，跨平台搬迁/恢复没有 wire codec 或 rebinding receipt |
| FSIO-P1-020 | Open | diagnostic/error 可能携带绝对物理路径；增加 redacted display path、stable logical URI 和 privileged forensic path |
| FSIO-P1-021 | Open | 没有 root ACL/secret/PII classification；plugin/package/importer 写权限无法按 principal 和 operation 授权 |
| FSIO-P1-022 | Partial | watcher/scan 已过滤 symlink/reparse 与事务兄弟文件，但 mapping failure 仍可能 `.ok()` 静默丢事件；必须升级为 `ReconcileRequired` |
| FSIO-P1-023 | Open | 没有 direct `std::fs` CI allowlist；建立 source-level gate，只有 provider/backend、journal、测试适配层可直接调用 OS |

### 6.3 Atomic file 与普通写入

| ID | 状态 | 证据与重构方向 |
|---|---|---|
| FSIO-P1-024 | Open | `atomic_write` 的公开返回类型是 `io::Result<()>`；发布后 parent sync/backup cleanup 失败不能结构化区分 pre-publish/post-publish |
| FSIO-P1-025 | Open | Windows parent directory sync 是 no-op；`MoveFileExW`/`ReplaceFileW` 有平台措施，但没有可查询的 durability capability |
| FSIO-P1-026 | Open | atomic writer 没有 expected digest/revision/CAS；并发更新是 last-writer-wins |
| FSIO-P1-027 | Open | staging/recovery 临时目录在后续失败时可能残留；缺统一 startup sweep、bounded quarantine 和 operator receipt |
| FSIO-P1-028 | Open | 替换 target 会丢失权限、ACL、xattr、owner、ADS 等 metadata，缺 retention/copy policy |
| FSIO-P1-029 | Open | write/new directory side effects 没有独立 directory transaction receipt；目录创建成功但文件发布失败无法由调用者统一清理 |
| FSIO-P1-030 | Open | 临时名含 PID/process counter，create-new 避免大多数碰撞，但没有 boot/process identity 和 sequence exhaustion 的跨重启语义 |
| FSIO-P1-031 | Partial | `AtomicWriteFault` 已覆盖 write/sync/replace/backup/dir sync；生产错误仍无稳定 code、retryability、operation correlation |
| FSIO-P1-032 | Open | cleanup 多处在 `Drop` 中吞掉错误；必须把 cleanup outcome 写入 receipt/recovery ledger，不允许静默丢失 |
| FSIO-P1-033 | Open | font baker private temp+rename 在 Windows failure 时先删除 target；这是破坏性 fallback，必须删除并接入统一 writer |
| FSIO-P1-034 | Open | shader variant cache 私有 writer 分别发布 `.wgsl`/`.meta`，remove 直接吞错；两文件需要 generation manifest 或 transaction |
| FSIO-P1-035 | Open | UI compiler cache 分别 `fs::write` artifact/payload；可留下半对，须定义 rebuildable cache、manifest 和 corruption sweep |
| FSIO-P1-036 | Open | plugin export `fs::copy` 直接覆盖 destination，读者可观察到部分文件；改成 staged copy + manifest + final publish |

### 6.4 Durable transaction、journal 与 recovery

| ID | 状态 | 证据与重构方向 |
|---|---|---|
| FSIO-P1-037 | Partial | durable transaction 已有 intent/WAL/commit point/rollback/restart evidence；仍以 path operation 为中心，没有 opened handle/CAS |
| FSIO-P1-038 | Open | `PreparedFileWrite` 为每个 target 持有 `Vec<u8>`；大批量写入在 admission 前无界保留 payload |
| FSIO-P1-039 | Open | 多文件 backup 是完整复制，产生高写放大；引入 stream source、reflink/block clone、backup budget 和 lease |
| FSIO-P1-040 | Open | 单 owner lock 串行化整个 journal；无 partition、fairness、queue wait、lock contention 和 stalled owner diagnosis |
| FSIO-P1-041 | Open | journal read/parse 最多 128 MiB，仍是整体载入；提供 framed streaming replay 和 memory budget |
| FSIO-P1-042 | Open | 任一 foreign/stale/unsupported entry 使 discovery fail-closed 并阻塞整个 journal 目录；增加 namespace-owned quarantine 和 bounded operator action |
| FSIO-P1-043 | Open | `DurableCommitReport` 多数 getter 受 `cfg(test)`/profiling 限制；生产只得到 disposition，缺 file-level receipt、bytes、phase wall、recovery action |
| FSIO-P1-044 | Open | `DurableTransactionError` 有 phase/path/source，但没有稳定 error code、retry class、committed generation、redaction 和 source chain |
| FSIO-P1-045 | Open | owner lock 先 metadata 后 open，仍有 TOCTOU；`.zrlock` 永久 sibling 没有 principal/generation/lease expiry 记录 |
| FSIO-P1-046 | Open | rollback 依据 path/digest，无法证明外部 writer 未改动；引入 expected object identity 与 conflict receipt |
| FSIO-P1-047 | Open | recovery policy 只约束 target set，不约束 source/provider generation；mount/source 变化后 replay 语义未定义 |

### 6.5 Runtime、Editor、App 与 Interface 产品接线

| ID | 状态 | 证据与重构方向 |
|---|---|---|
| FSIO-P1-048 | Open | Runtime asset/artifact/project/config/preference/scene/shader/UI/plugin 等多 owner 已真实采用部分 atomic primitive，但没有 provider boundary、统一 recovery sweep 和 receipt harvest |
| FSIO-P1-049 | Open | `asset/pack/install/file_io.rs` 直接 read/create/write/rename/copy/remove，绕过 shared durability/security；改为 package provider consumer |
| FSIO-P1-050 | Open | Config/Preference 接入的是 atomic byte helper，不是 source/provider transaction；Runtime203 的 multi-process/CAS/identity 问题仍存在 |
| FSIO-P1-051 | Open | Editor project authority 通过 rename/rollback 组织目录，没有 WAL、parent sync、file identity 或 terminal receipt |
| FSIO-P1-052 | Open | Editor scene creation 用 Runtime 写 staging 后 hard-link final，rollback/Drop 删除不 durable；catalog/source 与文件发布存在 crash window |
| FSIO-P1-053 | Open | Editor autosave snapshot 与 metadata 分两次 publish，pair 可 orphan；rotation/reservation cleanup 不进入统一 recovery ledger |
| FSIO-P1-054 | Open | Editor durable journal 自己实现 append/fsync/compaction，compaction 读完整 temp 后再调用 atomic write，存在重复 I/O 和内存峰值 |
| FSIO-P1-055 | Partial | session guard 有 named mutex/flock 和 Windows durability uncertainty，这是有价值的 owner lease；仍复制 atomic/platform code、`exists` 后 replace，未接 Runtime provider |
| FSIO-P1-056 | Open | Runtime Interface recent-projects store 拥有 CAS-like revision、bounded lease、quarantine、atomic replace，但低层 filesystem/FFI 逻辑放错 crate且与 Runtime 重复 |
| FSIO-P1-057 | Open | canonical spool 有 memory-to-temp spill、512 MiB/file-count budget 和 attempt journal，但 journal 只 flush 不 sync，Drop 递归删除吞错，无统一 startup recovery |
| FSIO-P1-058 | Open | `runtime_interface` ABI-adjacent layer 直接持有 OS path/temp/thread/platform operations；改成 Interface 只表达 DTO/receipt，Runtime 提供实现 |
| FSIO-P1-059 | Open | App 没有把 product/project/profile/principal root、mount registry、provider stack、offline/read-only policy 组成一个启动快照 |
| FSIO-P1-060 | Open | 无 remote/pak/cooked/cache source 的统一 product lane；graphics、font、shader、asset compiler 各自制造 cache/artifact 文件 |
| FSIO-P1-061 | Open | direct-fs 统计显示 Runtime/Editor/App/Interface 都有旁路；没有分阶段迁移清单、owner 和禁止新增门禁 |

## 7. P2 差距

| ID | 状态 | 方向 |
|---|---|---|
| FSIO-P2-001 | Open | 统一 path separator、Unicode normalization、case sensitivity、confusable 与 URI percent-encoding 版本 |
| FSIO-P2-002 | Open | 明确 filesystem type、network share、case-folding、timestamp granularity、atomic rename 和 directory fsync 能力矩阵 |
| FSIO-P2-003 | Open | 提供 copy-on-write/reflink、sparse file、preallocation、compression/encryption filter capability |
| FSIO-P2-004 | Open | 设计 paged directory/stat API，避免 `read_dir`/metadata 全量收集和 bool-only exists |
| FSIO-P2-005 | Open | 统一 file URL/logical URI/display path，禁止 `to_string_lossy` 折叠非 UTF-8 名称 |
| FSIO-P2-006 | Open | 统一 temp/spool naming、boot nonce、process identity 和 exhaustion diagnostics |
| FSIO-P2-007 | Open | recovery dashboard/operator API：pending count、quarantine evidence、last action、retry/abort/repair policy |
| FSIO-P2-008 | Open | transaction journal schema migration、N-2 reader、forward unknown-field policy 与 cross-version downgrade refusal |
| FSIO-P2-009 | Open | provider conformance suite 覆盖 memory/fs/pack/http/read-only/host unavailable，测试同一 error/receipt 语义 |
| FSIO-P2-010 | Open | I/O tracing 关联 resource UUID、mount generation、request ID、frame/tick、tenant 与 GPU upload fence |
| FSIO-P2-011 | Open | read cache、negative cache、stale-while-revalidate 和 invalidation 与 source generation 绑定 |
| FSIO-P2-012 | Open | full disk/quota/permission/locked file/volume removal/network timeout 的 product-facing remediation policy |
| FSIO-P2-013 | Open | security audit log 对 plugin/import/package writes 记录 principal、capability、target class 和 decision |
| FSIO-P2-014 | Partial | 已有 fault/profile tests；补 cross-process, kill-at-phase, power-loss simulation, network/reparse/ACL/Unicode and soak evidence |

## 8. 直接 I/O 旁路台账

以下是已逐文件阅读并确认需要迁移或明确为合法 backend 的高信号 owner；统计扫描的所有候选留在第 2 节，不能把它们缩减成只有这张表。

| 当前 owner | 当前行为 | 风险 | 迁移裁决 |
|---|---|---|---|
| `zircon_runtime/src/asset/pack/install/file_io.rs` | raw read/create/write/rename/copy/remove | 包安装部分可见、没有 transaction/security/receipt | PackageSource provider；Tooling03继续拥有 cook/install policy |
| `zircon_runtime/src/bin/zircon_font_sdf_bake/write.rs` | private PID temp + write + rename；Windows 先删 target | 失败时破坏旧文件，metadata/durability丢失 | 统一 writer；bake tool消费 Runtime provider |
| `zircon_runtime/src/graphics/shader/variant_cache/disk.rs` | `.wgsl` 与 `.meta` 分别 private atomic write，删除吞错 | pair mismatch、cache corruption、无 generation | cache manifest/transaction；Runtime09C拥有 shader identity |
| `zircon_runtime/src/plugin/export_build_plan/materialize/copy.rs` | `fs::copy` 直接写 destination，内容相等才跳过 | partial destination、无 package receipt | staged materializer + provider；Plugins01/Tooling03拥有 trust/package |
| `zircon_runtime/src/ui/template/asset/compiler/cache/persistent.rs` | artifact/payload 分开 `fs::write`，raw read/remove | 半对、全量重建、静默 cleanup | rebuildable cache source + manifest + sweep |
| `zircon_editor/src/core/project/authority/transaction.rs` | rename target→backup、staging→target、Drop cleanup | 目录级 crash/TOCTOU/无 WAL | Runtime durable directory transaction |
| `zircon_editor/src/core/project/scene_document.rs` | Runtime staging 后 hard-link source，rollback/Drop remove | parent sync/catatalog publication 不一致 | scene asset transaction；Editor 只发 authoring intent |
| `zircon_editor/src/core/recovery/autosave/store.rs` | snapshot/metadata 分开 publish，保留与轮转局部健壮 | orphan pair、cleanup/recovery分裂 | snapshot bundle transaction + product recovery receipt |
| `zircon_editor/src/core/recovery/session_guard/mutation.rs` | 自有 hard-link/ReplaceFileW/parent sync；Windows uncertainty | 重复 platform authority、exists/open race | Session lease 保留；writer hard-cut 到 Runtime |
| `zircon_editor/src/core/editing/engine/journal/durable/store.rs` | append journal + `sync_data`，compaction 全量读 temp 后 atomic write | 与 Runtime journal 重复、内存峰值、无 owner lock | Editor journal codec 保留，storage provider/receipt 下沉 Runtime |
| `zircon_runtime_interface/src/hub_protocol/recent_projects/store.rs` | bounded registry、revision、lease、quarantine、atomic replace | ABI crate 持有 OS IO；quarantine→new write 非单事务 | Interface 保留 DTO/lease intent，Runtime 实现 provider |
| `zircon_runtime_interface/src/serialization/text/canonical_spool.rs` | memory→temp spill、attempt journal、Drop recursive cleanup | journal 未 sync、Drop吞错、无startup sweep | serializer 只请求 spool capability，Runtime 持有 temp/recovery |

## 9. 参考引擎对照与可吸收边界

### 9.1 Unreal

`IPlatformFile` 支持 lower-level wrapper chain、stat/list/symlink、读写 handle、async/mapped I/O、sandbox 与 file journal；`PlatformFileManager` 持有 topmost/physical owner 并负责 lifecycle；`IoDispatcher` 按 chunk ID、mount priority、batch request、cancel/update priority/status、range/target VA 管理容器；`FPackageName` 另行负责 logical package name、mount point、override chain 和 mount lifetime；Pak provider 负责 index/signature/encryption/integrity/cache。Zircon 应吸收“physical backend、logical package、wrapper、dispatcher、mount lifecycle”分层，不能只复制 UE 的布尔错误或全局 manager。

### 9.2 Bevy

Bevy `AssetSourceId`/`AssetSourceBuilder` 把 unprocessed/processed reader、writer、watcher 组合成可注册 source；`AssetReader` 支持异步 Reader、seekable fallback、directory/meta；`AssetWriter` 支持 async write/remove/rename/directory。它证明 source instance 和 capability 应注册在资源系统，而不是由固定 scheme 分支散落；Zircon 还需补更强的 root capability、durable receipt 和 generation。

### 9.3 Fyrox

Fyrox `ResourceIo` 明确 `load/write/move/delete/copy/canonicalize/directory/walk/exists` 与 `FileReader`/async future，并给出 `FsResourceIo`。这是最接近 Rust runtime provider 的参考，但其 `exists -> bool`、默认 reader 和 filesystem path contract 不能直接照搬；Zircon 需要 typed NotFound/Permission/Integrity/Cancelled/Deadline/Unsupported，而不是把错误吞成 false。

### 9.4 Godot

Godot `FileAccess`/`DirAccess` 用 access type 划分 Resources、UserData、Filesystem，并覆盖 seek/length/buffer/flush/temp、directory/link、read-only attribute；compressed/encrypted/memory/pack/zip 是不同 backend。`ResourceLoader` 另有 format-loader、cache mode、threaded load status、dependency progress 和 load token。Zircon 应吸收 access domain、format provider 和 threaded lifecycle，避免 Godot 的 global factory/thread-local last-error 形态。

### 9.5 Unity Graphics

仓内 Unity Graphics 代码主要是 shader/render-pipeline consumer：`AssetDatabase`/`StartAssetEditing`/`StopAssetEditing` 体现 authoring asset authority 与批量编辑边界，某些 bake/export/cache 路径仍使用 raw file。该参考不能替代底层 Unity filesystem 证据；本篇只吸收“AssetDatabase/authoring 与 runtime artifact 分域、批处理有明确边界”，graphics streaming 继续由 Runtime04/09C 和 Tooling08拥有。

## 10. 目标工程架构

```text
ProductHostConfig + Principal + ProjectIdentity
             |
             v
      MountRegistry snapshot (generation, priority, lifetime)
             |
             v
 ResourceLocator -> ResourceAddress -> SourceCapability
             |
             v
 Provider resolver (filesystem / memory / pak / cooked / http / remote)
             |
             v
 OpenedRootCapability + handle-relative no-follow open
             |
             +--> IoScheduler (range, batch, priority, cancel, deadline, budget)
             |       +--> Reader / Writer / Watch / Stat / Directory
             |
             +--> StagedWriter / stream source / metadata policy
                         |
                         v
                 Durable transaction + recovery ledger
                         |
                         v
                 IoOperationReceipt + generation publication
```

### 10.1 规范合同

1. `LogicalPath` 只允许 canonical URI segments；`PhysicalPath` 不离开 provider；`DisplayPath` 永不作为操作输入。
2. `RootCapability` 携带 product/project/principal/mount/source identity、权限、reparse/hard-link policy、generation 和 disposal token。
3. `ResourceAddress` 解析一次得到 pinned source/mount snapshot；unmount 只能在 outstanding handle drain 后完成。
4. `OpenedFile` 绑定 object identity；rename/remove/replace 使用 parent handle-relative primitives，禁止 check-then-open。
5. `IoRequest` 有 size/range/priority/deadline/cancel/tenant/request ID；不可取消 provider 必须在 capability 中声明并隔离队列或进程。
6. `WriteReceipt` 明确 staged、published、file-synced、parent-synced、remote-acked、cleanup-deferred、recovery-required；普通 success 不隐式等于 durable。
7. `TransactionIntent` 使用 logical address + provider generation + expected object identity；journal wire 采用 versioned path codec，不把不可移植 native bytes 静默转成 lossy UTF-8。
8. cache/artifact 可以声明 rebuildable，但仍必须有 manifest、digest、generation、bounded sweep 和 corruption receipt。

## 11. 重构里程碑与资格门

| Milestone | 内容 | 退出证据 |
|---|---|---|
| M0 | 冻结 provider owner、direct-fs allowlist、logical/physical/display path taxonomy、receipt vocabulary | 架构 ADR、机器可读 allowlist、无新增 raw bypass |
| M1 | `FileSystemProvider`、`OpenedRootCapability`、stat/list/open/read/write/remove/rename、typed error | fs/memory/read-only provider conformance；cross-platform path corpus |
| M2 | source/mount registry、priority、generation、unmount drain、watch event | mount snapshot/override/unmount tests；watch reconciliation evidence |
| M3 | async I/O scheduler、range/seek/map、cancel/deadline/priority、tenant budget | batch/latency/throughput/queue fairness/abort metrics；no full-file Vec requirement |
| M4 | atomic writer receipt、metadata policy、directory sync capability、startup sweep/quarantine | pre/post publish fault matrix；Windows/Unix/network/ACL/reparse result classification |
| M5 | durable transaction handle-relative rewrite、stream payload、CAS/file identity、journal vN | fresh-process kill-at-phase、external writer conflict、torn journal and rollback evidence |
| M6 | Runtime asset/config/preference/artifact/cache/plugin consumers迁移 | direct-fs Runtime/App scan只剩 allowlisted provider/backend；all receipts harvested |
| M7 | Editor project/scene/autosave/journal/session consumers迁移 | editor crash/restart/autosave pair/catalog consistency and bounded close |
| M8 | pak/cooked/http/remote providers、GPU upload bridge、product diagnostics | source capability matrix、offline/read-only/remote degradation and perf qualification |

资格门：

| Gate | 当前 | 判定 |
|---|---|---|
| FSIO-G01 | Fail | 所有业务 read/write 经过唯一 provider |
| FSIO-G02 | Fail | provider source/mount identity 与 generation pinned |
| FSIO-G03 | Fail | root-relative no-follow handle operation |
| FSIO-G04 | Fail | reparse/junction/hard-link/file ID policy |
| FSIO-G05 | Fail | async range/batch/cancel/deadline/priority |
| FSIO-G06 | Fail | typed error/retry/security redaction |
| FSIO-G07 | Partial | Runtime atomic write staging 已有，统一 receipt 未有 |
| FSIO-G08 | Partial | Runtime durable WAL/recovery 已有，handle/CAS 未有 |
| FSIO-G09 | Fail | platform durability capability |
| FSIO-G10 | Fail | metadata/ACL/xattr/ADS retention policy |
| FSIO-G11 | Fail | external writer conflict detection |
| FSIO-G12 | Fail | bounded startup sweep/quarantine/operator receipt |
| FSIO-G13 | Fail | streaming payload/backups/reflink budget |
| FSIO-G14 | Fail | transaction owner partition/fairness/metrics |
| FSIO-G15 | Fail | remote/pak/cooked/read-only providers |
| FSIO-G16 | Partial | watcher bounded queue已有，provider/mapping/reconcile未统一 |
| FSIO-G17 | Fail | Runtime/App direct-fs allowlist gate |
| FSIO-G18 | Fail | Editor direct-fs hard cut to Runtime |
| FSIO-G19 | Fail | Interface low-level OS I/O removed from ABI crate |
| FSIO-G20 | Fail | asset publication waits/records durable terminal |
| FSIO-G21 | Fail | product startup composes root/provider/mount/policy snapshot |
| FSIO-G22 | Fail | cache/artifact pair manifest/generation |
| FSIO-G23 | Fail | fault matrix across processes/filesystems |
| FSIO-G24 | Fail | I/O performance and fairness qualification |

## 12. 测试与验证缺口

本轮是 review-only，没有运行 Cargo 或修改代码。现有测试足以证明不少 local atomic/WAL/recovery invariant，但不能替代下列资格证据：

1. 两个独立进程同时写同一 target、不同 alias、同一 journal；验证 lock/CAS/file ID 和 conflict receipt。
2. 在 staging flush、file sync、replace、backup sync、parent sync、commit point、cleanup 各阶段 kill/restart；验证 live generation、catalog、manifest、receipt 一致。
3. Windows reparse/junction/ADS/ACL/long UNC、Unix symlink/hard-link/non-UTF8、case-folding 和 network share。
4. provider read range/seek/map、batch priority、cancel/deadline、queue fairness、hung backend 隔离和 bounded shutdown。
5. 目录损坏、foreign journal、unknown version、oversize frame、full disk、permission denied、locked target、volume removal、remote timeout。
6. Runtime asset scan/import/migration、Editor scene/autosave/journal、shader/UI cache、plugin export 的 crash/restart product lane，而不是只测 helper。
7. 端到端 throughput、P50/P95/P99 latency、bytes copied、write amplification、RSS、queue wait、recovery time 和 GPU upload overlap；没有这些不能支持性能超过 Unreal 的目标。

## 13. Owner、依赖与禁止重复实现

| Owner | 本篇拥有 | 邻接报告继续拥有 |
|---|---|---|
| Path/URI | logical/physical/display taxonomy、encoding、file identity、secure operation preconditions | Interface02 的 public `ResourceLocator/RelPath` DTO |
| FileSystem provider | read/write/stat/list/open/watch、capability、errors、scheduler、durability receipt | Runtime04 的 artifact/chunk/pack semantics |
| Source/mount | source registry、priority、generation、lifetime、quiescence | Plugins01 的 trust/native admission；Tooling03 的 cook/install policy |
| Durable transaction | staging、WAL、commit/rollback/recovery、CAS/object identity | Runtime99w 的 asset catalog/publication order |
| Runtime consumers | migration ledger、allowlist、receipt harvesting和产品 fallback | 每个 domain report 的 resource/artifact schema |
| Editor boundary | authoring intent、session/recovery UI、project/scene journal codec | Editor scene/settings/asset authoring domain contracts |
| Runtime Interface | stable DTO/command/receipt shape | 不再拥有低层 OS temp/file/lock implementation |

禁止新建第二个 `atomic_write`、`ResourceIo`、`FileAccess`、`TempSpool`、`PathIdentity`、`OwnerLock` 或 platform replace helper。新 consumer 先登记 provider capability 和 receipt，再迁移实现；若必须保留 rebuildable cache，必须显式标记其可重建性、manifest、sweep 和 corruption handling。

## 14. 本轮状态

本轮只完成源码复审、参考引擎对照、差异登记和重构计划；没有生产实现、Cargo 验证或 benchmark。当前 `zr_resource` durable writer 基础应保留并下沉为未来 provider 的 primitive，但在 M1-M5 完成前，不应宣称 Zircon 已有 Unreal 级 filesystem/resource I/O，更不应把各处局部 temp+rename 或“写入成功”当成统一工程能力。

## 状态与产出记录

| 里程碑 | 状态 | 日期 | 证据 |
|---|---|---|---|
