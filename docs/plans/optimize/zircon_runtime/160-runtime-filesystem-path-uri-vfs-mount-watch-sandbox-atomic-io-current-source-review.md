---
title: Runtime Filesystem、Path、URI、VFS、Mount、Watch、Sandbox 与 Atomic I/O 当前源码复核
category: zircon_runtime
report_id: Runtime160
review_date: 2026-08-29
baseline_head: a6cc2b0ca091014944b1a0ce0b48e1a8c3101eb6
canonical_owner: Runtime25
refreshes:
  - docs/plans/optimize/zircon_runtime/25-filesystem-path-uri-vfs-mount-watch-sandbox-atomic-io-review.md
related_code:
  - zircon_runtime/crates/zr_resource/src/io
  - zircon_runtime/src/asset/project
  - zircon_runtime/src/asset/watch
  - zircon_runtime/src/asset/pipeline/manager
  - zircon_runtime/src/asset/runtime_asset_path.rs
  - zircon_runtime/src/asset/safe_project_path.rs
  - zircon_runtime/src/asset/module.rs
  - zircon_runtime_interface/src/resource
  - zircon_hub/src/projects
  - zircon_editor/src/core/project
  - zircon_editor/src/core/recovery
  - zircon_app/src/entry/runtime_library
tests:
  - zircon_runtime/crates/zr_resource/src/io
  - zircon_runtime/src/asset/project
  - zircon_runtime/src/asset/watch
  - zircon_runtime/src/asset/pipeline/manager
  - zircon_runtime_interface/src/resource
  - zircon_hub/src/projects
plan_sources:
  - docs/plans/optimize/zircon_runtime/25-filesystem-path-uri-vfs-mount-watch-sandbox-atomic-io-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime_interface/02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md
  - docs/plans/optimize/zircon_hub/01-project-engine-build-editor-launch-process-persistence-delivery-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/HAL/IPlatformFileModule.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/HAL/PlatformFileManager.h
  - dev/UnrealEngine/Engine/Source/Developer/DirectoryWatcher/Public/IDirectoryWatcher.h
  - dev/UnrealEngine/Engine/Source/Developer/DirectoryWatcher/Private/DirectoryWatcherTests.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/Misc/PackageName.h
  - dev/UnrealEngine/Engine/Source/Programs/LowLevelTests/FoundationTests/Tests/PackageNameTests.cpp
  - dev/bevy/crates/bevy_asset/src/io/mod.rs
  - dev/bevy/crates/bevy_asset/src/io/source.rs
  - dev/bevy/crates/bevy_asset/src/io/file/file_asset.rs
  - dev/bevy/crates/bevy_asset/src/io/file/file_watcher.rs
  - dev/Fyrox/fyrox-resource/src/io.rs
  - dev/godot/core/io/file_access.h
  - dev/godot/core/io/dir_access.h
  - dev/godot/core/io/file_access.cpp
  - dev/godot/core/io/dir_access.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/AssetDatabaseHelper.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: partial_foundation_provider_missing
source_recheck_required: true
---

# Runtime160 · Filesystem、Path、URI、VFS、Mount、Watch、Sandbox 与 Atomic I/O

## 1. 结论

当前 Runtime 已经有一些可以保留的工程基础，但它们仍是多个局部实现，尚未形成可替换、可审计、可恢复的文件系统架构。`ProjectPaths` 已区分 operation path 与 display path，项目根和 package 根会做 canonical containment、重复根和歧义 URI 检查；扫描器拒绝 symlink/reparse point 并过滤事务临时文件；`atomic_file` 与 project durable transaction 已具备 staging、sync、journal、owner lock、替换和恢复测试；Asset watcher 对 ingress 和 pending 队列同时限制 entries 与 bytes，溢出会要求 reconciliation；项目导入通过 preparation epoch、project generation 与事务提交避免旧扫描覆盖新状态。这些底座值得下沉，而不是推倒重写。

当前真正的工程缺口是统一 owner 不存在。`ResourceIo` 只有 `read/write/exists` 三个同步方法，trait 被 sealed，仓内没有实现或消费者；`AssetIoDriver` 虽然已经从可构造的空 unit struct 改为不可构造空 enum，但 asset module 仍声明 driver 名称，descriptor 却没有注册 driver。项目、artifact、plugin、font、Hub、Editor 和部分工具路径继续直接调用 `std::fs`，没有 provider allowlist、mount/source registry、root capability、异步 stream、range read、取消、优先级、关闭和统一 telemetry。

路径语义也没有闭合。`ResourceLocator` 用 host `std::path::Path::components` 解析 portable logical path，导致同一文本在不同 host 上可能得到不同结果；`source_uri_for_path`、watch mapping、Hub file URL 和多个持久化边界使用 `to_string_lossy` 或手写 URI，非 UTF-8、保留字符、UNC、drive、大小写和 Unicode 归一化没有统一 codec。`runtime_asset_path` 对 rooted、prefix、`..` 组件会静默丢弃，显式错误 root 又以 panic 结束启动。`is_safe_regular_file`、transaction owner lock 和多个写入路径都是“先检查，再按原 path 打开”，仍有 TOCTOU 与 hard-link 身份问题。

watcher 的线程生命周期本身已有 `JoinHandle` 和 Drop join，不应再把“无 join”作为当前缺陷；问题在事件语义：只有 `RenameMode::Both` 才能生成 rename，`From/To/Any` 会退化为 Modify；路径映射失败被 `.ok()` 静默过滤，rename 不完整时不会把不确定性送入已有 reconciliation。watch batch 也没有 source/mount generation、rename cookie、root identity 或可重放 receipt。

本报告是 Runtime25 的当前源码重分类，**没有新增唯一 finding**。旧报告的 40 项 P1 当前为 **33 Open、7 Partial、0 Closed**，12 项 P2 全部仍为 Open；没有任何 P1 可以因为局部基础而判定为完整关闭。20 个工程门为 **15 Fail、5 Partial、0 Pass**。这意味着当前版本不能声称已经具备 Unreal、Bevy、Fyrox 或 Godot 级别的可替换 VFS，也不能声称性能优于 Unreal；性能结论必须等 provider、调度、基准和故障资格线实现后再给出。

## 2. 审查边界与证据

### 2.1 当前源码选择

本轮聚焦所有直接拥有 Runtime25 文件系统合同的目录，并包含跨层消费者：

| 选择集 | files | lines | bytes | `#[test]` | `#[ignore]` | fingerprint |
|---|---:|---:|---:|---:|---:|---|
| Runtime resource I/O、asset project/watch/pipeline、Interface resource、Hub projects、Editor project/recovery、App runtime-library | **327** | **50,584** | **1,774,856** | **486** | **20** | `b7f5f64a0525a11346c2e1d80c4d4f6691a98a7aeb2e09d9424ec4b3606ecf29` |
| Unreal、Bevy、Fyrox、Godot、Unity Graphics reference selection | **16** | **6,984** | **279,714** | n/a | n/a | `b90d0104917155a9a36d6b36ed1a03137c0e93ec8b95bc33caf022b39425629` |

这是 owner-focused source review，不是把仓库全部 Rust 文件的关键词数量当成结论。未纳入 Tooling 的 cook、pack、DDC、发布和安装实现；这些由 Tooling 报告拥有。本轮也没有等待或查询协调器状态，动态阻断按 review evidence 记录。

### 2.2 当前实现的可保留证据

| 区域 | 当前事实 | 工程判断 |
|---|---|---|
| `ProjectPaths` | 解析现有路径与未创建尾段，保留 operation/display 双路径，检查项目根和 asset root | 可作为 `PhysicalPathResolver` 的输入，但不能继续作为全局 VFS |
| `ResolvedProjectPathIdentity` | 对 project path 提供稳定排序与 containment | 仍缺 filesystem file-id、hard-link policy 和 Hash/identity receipt |
| `safe_project_path` | 拒绝 symlink/reparse、做 physical containment | check 与后续 open 分离，不能提供安全 open 资格 |
| `collect_files` | 递归 `read_dir`，逐项 `symlink_metadata`，过滤 meta/transaction/auxiliary | 缺 scan snapshot、depth/bytes/file-count budget、cancel 和 provider 注入 |
| `atomic_file` / durable transaction | staging、`create_new`、sync、journal、owner lock、替换、recovery、故障测试 | 应成为统一 writer primitive，并接收 root capability/handle |
| Asset watcher | ingress/pending entries+bytes bounded、debounce、max latency、overflow reconciliation | mapping 不确定性必须显式进入 reconciliation，而不是静默过滤 |
| Project generation | preparation epoch、project generation、事务提交和重试 | 需要纳入 source/mount generation 与 read snapshot |
| `ResourceIo` | 公开但 sealed，只有同步 `read/write/exists` | 是接口占位，不是可用 provider；不能作为完成度证据 |
| `AssetIoDriver` | 不可构造空 enum，module 仍有名称常量但 descriptor 不注册 | 已避免虚假构造能力，仍缺实际 driver/provider 路由 |
| Hub metadata | 有 canonicalize/fallback path key 与 extended prefix 处理 | 与 Runtime identity/codec 分叉，不能继续各自演化 |

## 3. 逐条当前源码差异与重构归属

状态定义：`Open` 表示当前没有足够合同或实现；`Partial` 表示局部底座真实存在但无法承担完整工程语义；`Closed` 只允许在实现、消费者、测试和 owner receipt 全部存在时使用。本报告没有 P1 Closed。

### 3.1 P1：Path、URI、Encoding、Identity、Provider 与 Mount

| ID | 当前源码证据 | 当前状态 | 需要重构 |
|---|---|---|---|
| FILESYSTEM-P1-001 | 没有明确区分 logical、mounted、physical operation、display、persistent、filesystem identity path | Open | 在 Interface/Runtime 交界建立强类型 taxonomy，并让每个 API 声明持久化与 I/O 资格 |
| FILESYSTEM-P1-002 | `ResourceLocator` 归一化使用 host `Path::components`，portable 文本随 OS 变化；`zircon_runtime_interface` 拥有 DTO 缺陷 | Partial | Runtime 提供纯字符串 segment grammar 与 corpus，Interface02 完成 parser hard cutover |
| FILESYSTEM-P1-003 | `#` label、`%`、空格和保留字符没有统一 escaping，多个位置手写 `format!("...://")` | Open | 采用 versioned percent/length codec，parse/format 必须双射，禁止业务层拼 URI |
| FILESYSTEM-P1-004 | source URI 与 package URI 通过 `to_string_lossy`，不可逆 OS 文件名会折叠 | Open | 选择严格 portable UTF-8 source set 或可逆 bytes codec；invalid name 必须 typed reject/quarantine |
| FILESYSTEM-P1-005 | `PathBuf`、`RelPath`、URI、URL 和 process argument 之间没有 projection receipt | Open | 增加带 source、mount、generation、codec、lossless、identity 的 `PathProjectionReceipt` |
| FILESYSTEM-P1-006 | case、Unicode normalization、reserved name、trailing dot/space 依赖 OS | Open | source admission 声明规则并建立跨目标 collision index，cook 前拒绝不兼容名称 |
| FILESYSTEM-P1-007 | ProjectPaths、transaction PathIdentity、Hub metadata 各自计算 operation/display/key | Partial | 共享 `ResolvedPhysicalPath` 与 `FilesystemIdentity`，ProjectPaths 只保留 manifest 规则 |
| FILESYSTEM-P1-008 | Hub 与其他 consumer 手写 `file://` 或 lossy path，没有 UNC/drive/authority codec | Open | 唯一 file URL codec，所有产品 receipt 使用它并带 lossless/identity 结果 |
| FILESYSTEM-P1-009 | Hub recent/project metadata 以 canonicalize 或 raw path fallback 为 key，不带 product/profile/mount namespace | Open | 将 recent identity 绑定 product、engine profile、mount generation 和 relocation receipt |
| FILESYSTEM-P1-010 | manifest/URI schema 没有 codec version、source provenance、migration receipt | Open | 增加 versioned path schema、origin and migration evidence，拒绝隐式 reinterpretation |
| FILESYSTEM-P1-011 | `ResourceIo` trait sealed，仓内无 provider、无 consumer、无构造路径 | Open | 实现 `FileSystemProvider` 和可测试 `FsProvider`，由 runtime-owned registry 注入并可替换 |
| FILESYSTEM-P1-012 | `AssetIoDriver` 是不可构造空 enum，module descriptor 不注册它，异步 I/O 声明与实际不符 | Partial | 保留 fail-closed 形态；接入真实 driver descriptor、队列、cancel、shutdown 和 capability report |
| FILESYSTEM-P1-013 | 没有 source/mount registry、priority、overlay、unmount/quiescence 或 mount lifetime | Open | 建 `MountRegistry`，以 source instance、priority、generation、teardown receipt 管理解析 |
| FILESYSTEM-P1-014 | `ResourceScheme` 是固定 `Res/Library/Package/Builtin/Memory` enum，不能注册 source 实例 | Open | scheme 只做语法；source identity、backend、priority 和 capability 由 registry 提供 |
| FILESYSTEM-P1-015 | provider capability 没有 read-only、stream、range、watch、atomic、secure-open、remote/offline 描述 | Open | 以 typed capability matrix 做 admission，缺能力必须在计划阶段失败 |
| FILESYSTEM-P1-016 | `exists` 返回裸 bool，不能表达 permission、offline、stale、corrupt、cancel 或 retryability | Open | 用分类错误和 `ProbeResult`，区分 NotFound、Denied、Unavailable、Stale、Corrupt 与 Retryable |
| FILESYSTEM-P1-017 | `ResourceIo::read` 返回整块 `Vec<u8>`，asset scan/import 也直接 `fs::read` | Open | 引入 async stream、range/read-at、size limit、backpressure、cancel 和 allocator budget |
| FILESYSTEM-P1-018 | 没有统一 open/stat/list/create/remove/rename/copy/flush/sync/mapped-read contract | Open | 按 capability 拆分 provider traits，明确 handle ownership、deadline、atomicity 和 durability |
| FILESYSTEM-P1-019 | 没有 mount lifecycle、quiescence、in-flight drain 或 unmount result | Open | mount close 必须阻止新请求、等待 in-flight、发布 generation、返回可审计 receipt |
| FILESYSTEM-P1-020 | manifest 能校验 roots 重复/overlap，也能返回全部 roots，但多个消费者使用 primary/first root | Partial | 每个 URI 必须绑定唯一 root；新建目标显式选择 root，扫描/读取不得隐式 first-root |

### 3.2 P1：Asset Root、Sandbox、Watch、Atomicity 与资格线

| ID | 当前源码证据 | 当前状态 | 需要重构 |
|---|---|---|---|
| FILESYSTEM-P1-021 | `normalize_runtime_asset_relative_path` 静默忽略 prefix、root、`.`、`..`，rooted input 可变成合法相对路径 | Open | 对输入做严格 reject；声明 portable relative grammar，所有调用者使用 typed `AssetRelativePath` |
| FILESYSTEM-P1-022 | `ZIRCON_ASSET_ROOT` 无效时通过 `panic!` 结束，测试也固定 panic 语义 | Open | 启动阶段返回 typed configuration error，记录 source/value/reason，不在 library path panic |
| FILESYSTEM-P1-023 | `asset_uri_for_path` 与 project URI 映射使用 `to_string_lossy` | Open | provider 返回可逆 logical segment；不可表示名称变成显式 mapping error 和 reconciliation |
| FILESYSTEM-P1-024 | `AssetManager::open_project` 和 capability API 接收 `&str`，迫使 PathBuf caller lossy conversion | Open | 以 `ProjectRootInput`/validated `Path` 接口替代字符串，process/CLI 边界另行 codec |
| FILESYSTEM-P1-025 | project identity 与 transaction `PathIdentity` 都存在，但比较、canonicalize 和 owner 不统一 | Partial | 合并 identity owner；每个 operation 保留 physical file-id、path key、root generation 和 comparison policy |
| FILESYSTEM-P1-026 | containment 主要依赖 `starts_with`、canonical path 或 string comparison，没有 OS file-id | Open | 引入 file-id/equivalence provider；path prefix 只作为预检，最终以 identity/root handle 判定 |
| FILESYSTEM-P1-027 | `is_safe_regular_file` 检查后，调用者再按原 path `fs::read`/open | Open | secure provider 使用 root-relative no-follow handle，check/open 必须是一个 operation |
| FILESYSTEM-P1-028 | symlink/reparse 会拒绝，但 hard link 可以让文件 identity 脱离 project root | Open | 明确 hard-link 策略；必要时比较 file-id、link count 或拒绝不受控 identity |
| FILESYSTEM-P1-029 | 未创建尾段通过 deepest existing ancestor + canonicalize 后再 join，创建阶段仍可能被并发替换 | Open | relative handle traversal、no-follow create、parent identity recheck 和 race test |
| FILESYSTEM-P1-030 | project root 是 PathBuf，不是带权限、mount、principal、lifetime 的 capability | Open | `RootCapability` 绑定 provider、root handle、principal、allowed operations、generation 与 expiry |
| FILESYSTEM-P1-031 | `map_notify_event` 只对 `RenameMode::Both` 生成 rename，From/To/Any 会退化为 Modify | Open | 建 rename cookie/temporal pairing；无法配对必须输出 `ReconcileRequired`，禁止伪造 Modify |
| FILESYSTEM-P1-032 | Create/Modify/Remove mapping 使用 `.ok()`；不支持路径或 incomplete rename 会静默生成空 batch | Open | mapping 返回 `Mapped | IgnoredByPolicy | ReconcileRequired | Error`，把不确定性纳入 batch diagnostics |
| FILESYSTEM-P1-033 | watcher batch 没有 source/mount generation、root identity、rename cookie 或 replay receipt | Open | raw event 带 provider/source/root/generation；batch 可持久化、重放、去重并校验 epoch |
| FILESYSTEM-P1-034 | project、artifact、plugin、font 和 Hub 仍直接调用 `std::fs`，没有机器可读 bypass allowlist | Open | 规定 direct-fs 只允许 provider/backend；CI 建调用点清单、owner、理由、测试和 exception expiry |
| FILESYSTEM-P1-035 | 没有 runtime-owned I/O scheduler；direct reads 与 importer/task pool 争抢线程和内存预算 | Open | driver 统一 queue、priority、deadline、cancel、bytes budget、fairness、shutdown drain 和 metrics |
| FILESYSTEM-P1-036 | project durable transaction 具备强保障，但 deletion/relocation/asset readers/Hub/Editor writers 未统一接入 | Partial | 以 `DurableWriter` 覆盖所有 persistent writes，并声明 atomicity/durability level 与 recovery receipt |
| FILESYSTEM-P1-037 | `ResourceIoError` 只有 `NotFound(String) / Io(String) / ReadOnly(String)` | Open | 错误必须含 source、operation、path projection、OS code、retryability、offline/corrupt/stale/permission 分类 |
| FILESYSTEM-P1-038 | watcher 与 transaction 各有计数和 diagnostics，没有统一 I/O trace、queue wait、bytes、retry、generation 维度 | Partial | 统一 `IoOperationId`、provider/source/mount labels、bounded telemetry 和 privacy policy |
| FILESYSTEM-P1-039 | 现有测试偏向 temp-dir happy path，缺 non-UTF8、case/Unicode collision、hard-link、network/offline、mount swap、rename loss | Open | 建跨平台 corpus、fault provider、property/fuzz、replay、power-loss、soak 和规模资格套件 |
| FILESYSTEM-P1-040 | 没有 provider/mount/secure-open 的 product qualification、性能基线或可签名 receipt | Open | 设 runtime qualification gate：功能、故障、安全、规模、性能、观测和 shutdown 全部产出 receipt |

### 3.3 P2：当前仍需后续处理的完整性问题

| ID | 当前状态 | 后续动作 |
|---|---|---|
| FILESYSTEM-P2-001 | Open | 统一 path error 的显示路径与安全脱敏策略 |
| FILESYSTEM-P2-002 | Open | 记录目录枚举排序、大小写与 locale policy，避免跨平台 import 漂移 |
| FILESYSTEM-P2-003 | Open | 明确 watcher debounce、coalescing 与 import generation 的时间合同 |
| FILESYSTEM-P2-004 | Open | 为 source/mount 建可观测 health、last-success、last-error 和 stale 状态 |
| FILESYSTEM-P2-005 | Open | 将 cache、library、artifact、user data 与 project source 的权限和清理策略分开 |
| FILESYSTEM-P2-006 | Open | 设计长路径、UNC、verbatim path、权限拒绝与设备移除的用户诊断 |
| FILESYSTEM-P2-007 | Open | 给批量扫描增加 progress、partial result、resume cursor 与取消 receipt |
| FILESYSTEM-P2-008 | Open | 为读写 provider 建内存、压缩、加密和远程 adapter 的一致性测试 |
| FILESYSTEM-P2-009 | Open | 约束临时文件命名、journal retention、crash cleanup 与磁盘配额行为 |
| FILESYSTEM-P2-010 | Open | 让 Editor/Hub 错误模型保留 Runtime source/mount/generation provenance |
| FILESYSTEM-P2-011 | Open | 对 direct-fs exception 建 owner、期限、替换计划和 CI 审计输出 |
| FILESYSTEM-P2-012 | Open | 形成 filesystem threat model、权限最小化和供应链/插件隔离文档 |

## 4. 参考引擎对照

### 4.1 Unreal

`IPlatformFileModule` 与 `PlatformFileManager` 把 physical backend、wrapper chain、sandbox、pak/cache 和 lower-level file 访问放在可组合层；`DirectoryWatcher` 有独立注册、回调和测试；`PackageName` 把 logical package name 与 mount point、register/unregister 和 override chain 分开。Zircon 应借鉴 provider/wrapper/mount/lifetime 分层，不能把 source 解析塞进固定 scheme enum。

### 4.2 Bevy

Bevy `AssetSource` 可注册命名 source，并组合 unprocessed/processed reader、writer 和 watcher；file reader/writer/watch 以异步和 stream 语义协作。Zircon 当前只有 sealed `ResourceIo` 占位和 direct `fs::read`，必须补 source instance、异步请求、取消、预算和 watcher generation。

### 4.3 Fyrox

Fyrox 的 `ResourceIo` 是实际可注入的 provider，覆盖 async load/write/move/delete/copy、canonicalize、目录遍历和 reader，并有 filesystem implementation。它证明同名 trait 只有在有 constructible provider、consumer、错误合同和测试时才算完成；Zircon 不应继承弱 `exists -> bool` 或默认空 iterator 语义。

### 4.4 Godot

Godot `FileAccess`/`DirAccess` 将 Resources、UserData、Filesystem 访问域与 compressed、encrypted、memory、pack backend 分开，并提供 seek、length、flush、directory、link 和 case/equivalence 行为。Zircon 应吸收 access-domain 与 backend selection，但用 typed capability/error 替代全局 factory 和隐式 last-error。

### 4.5 Unity Graphics 参考边界

本次 `dev/Graphics` 只提供 Unity Graphics package 的路径消费端 `AssetDatabaseHelper.cs`，不包含 Unity 底层 filesystem/VFS 实现。因此它只能证明 graphics consumer 不应自建 path codec，不能作为底层 provider 完成度证据；pack/range/artifact 由 Runtime04 与 Tooling 相关报告拥有。

## 5. 目标架构与迁移顺序

### M0 · 先锁定合同

由 Runtime25 与 Interface02 联合冻结 `LogicalResourcePath`、`AssetRelativePath`、`MountedPath`、`PhysicalOperationPath`、`DisplayPath`、`PersistentPath`、`FilesystemIdentity` 和 `PathProjectionReceipt`。建立跨 OS golden corpus：drive/UNC/verbatim、separator、reserved、Unicode、非 UTF-8、保留字符、`#/%`、`.`/`..`、大小写和 hard-link。

### M1 · Provider 与 root capability

在 `zircon_runtime` 建 runtime-owned `FileSystemProvider`、`RootCapability`、`FileHandle`、`DirectoryCursor` 和 typed `ResourceIoError`。先实现本地 FsProvider，所有 operation 接受 validated relative path 或 handle；保留 direct OS calls 仅在 provider/backend 文件，并建立 allowlist CI。

### M2 · Source/Mount Registry

实现 source instance registry、priority/overlay、capability admission、mount generation、quiescent unmount、in-flight drain 和 source health。`ResourceScheme` 退化为 grammar token，`res://` 等 URI 通过 registry resolution 得到 source/mount receipt。

### M3 · Asset driver 与导入链

让 `AssetIoDriver` 变成可注册、可关闭的真实服务，提供 async read/stream/range/stat/list/write、priority、cancel、deadline、memory/bytes budget。project scan/import、artifact store、plugin/font/resource residency 改为 driver consumer；`AssetManager::open_project` 采用 validated path input。

### M4 · Watch reconciliation

将 notify adapter 变为 provider watcher。事件必须带 source/root/mount generation 和 file identity；rename 配对失败、mapping 不确定、overflow、generation mismatch 都输出 `ReconcileRequired`。保留现有 bounded queues 和 project generation publication。

### M5 · Secure and durable writes

以 root-relative no-follow handle traversal 替换 `symlink_metadata + canonicalize + starts_with` 组合；统一 atomic file、project transaction、relocation、deletion、Hub/Editor save 的 `DurableWriter`，产出 atomicity/durability/recovery receipt。

### M6 · Qualification and performance

只有 M0-M5 完成后才比较性能：冷/热读、并发 stream、目录扫描、watch burst、rename storm、cache hit、remote/offline、import throughput、shutdown drain 和 peak memory。每项都要有 provider/source/mount/generation 标签，避免只测 temp-dir happy path 后宣称优于 Unreal。

## 6. 测试与验收门

| Gate | 当前状态 | 必须新增的证据 |
|---|---|---|
| FS-G01 path/URI grammar cross-OS | Partial | pure string parser、golden corpus、roundtrip 与 invalid corpus |
| FS-G02 escaping/codec losslessness | Fail | `%/#`、UNC、非 UTF-8、URL 和 receipt 双向测试 |
| FS-G03 case/Unicode/collision | Fail | per-source policy、NFC/NFD、case-fold、reserved-name matrix |
| FS-G04 physical identity/equivalence | Fail | file-id、hard-link、junction、network share provider tests |
| FS-G05 root capability admission | Fail | principal、operation、generation、expiry、revoke tests |
| FS-G06 no-follow secure open | Fail | check/open race、symlink swap、reparse/hard-link attack tests |
| FS-G07 provider capability matrix | Fail | local/memory/fault provider and unsupported-capability admission |
| FS-G08 async stream/range/cancel | Fail | backpressure、deadline、cancel、partial-read 和 memory budget |
| FS-G09 source/mount registry | Fail | priority、overlay、collision、unmount/quiescence、generation |
| FS-G10 direct-fs governance | Fail | allowlist、owner、exception expiry、CI diff gate |
| FS-G11 watcher rename/reconciliation | Partial | From/To/Any pairing、mapping error、overflow、replay 和 reconciliation |
| FS-G12 generation-bound watch/import | Partial | source/mount/project generation mismatch and retry tests |
| FS-G13 bounded scheduler | Fail | queue fairness、bytes budget、priority inversion、shutdown drain |
| FS-G14 atomic/durable/recovery | Partial | all writers、power-loss、disk-full、cross-device、journal replay |
| FS-G15 typed error/telemetry | Fail | retryability、offline/stale/corrupt/permission taxonomy and trace |
| FS-G16 project/package/plugin sandbox | Fail | package admission、plugin root、cross-root and principal isolation |
| FS-G17 scan scale/budget/resume | Fail | millions of entries、depth/bytes/file count、cancel/resume |
| FS-G18 network/offline/device loss | Fail | share disconnect、remount、device removal、stale handle |
| FS-G19 fuzz/property/security | Fail | URI/path fuzz, race harness, malformed event and journal fuzz |
| FS-G20 product qualification receipt | Partial | signed matrix for functionality, security, performance and shutdown |

当前 gate 汇总：**0 Pass / 5 Partial / 15 Fail**。`Partial` 只表示已有局部基础，不能作为发布资格。

## 7. Owner 边界与后续计划

Runtime25 拥有跨 Runtime 的 path taxonomy、provider、mount、secure open、watch contract、direct-fs governance 与 qualification。Interface02 拥有公共 `ResourceLocator`/`RelPath` DTO 的 parser/schema hard cutover；Runtime04 拥有 artifact、library、pack/range、residency；Plugins01 拥有 plugin package trust/native admission；Hub01 拥有 Hub 产品项目流程与本地操作；Editor02 拥有 document save/autosave；Tooling03/08/09 拥有 cook、DDC、artifact repository、install/update transaction。

实现顺序必须遵循 M0 -> M1 -> M2 -> M3 -> M4 -> M5 -> M6，不能先在单个 importer 或 Editor 面板中增加另一套 path/VFS。每个阶段都要更新 direct-fs allowlist、跨层调用图、故障矩阵和本报告的 source recheck；若公共 DTO 或 ABI 发生变更，必须由 Interface/ABI owner 另行审查。

## 8. Review-only 结论

本轮只新增 review 文档、Runtime index 和 coverage 记录，没有修改 production Rust、tests、Cargo、ABI、运行时行为或文件系统数据，也没有运行 Cargo、Editor、DLL、GPU、跨进程、fault、fuzz、scale、soak 或动态 benchmark。当前最优先的不是再增加一个临时 `fs::read` helper，而是完成 M0-M2 的 owner、类型和 provider 边界；在此之前，任何“VFS 已完成”“sandbox 已安全”或“性能优于 Unreal”的结论都不成立。
