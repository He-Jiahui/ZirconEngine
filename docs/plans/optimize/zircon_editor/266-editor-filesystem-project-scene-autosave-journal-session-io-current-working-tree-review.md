---
related_code:
  - zircon_editor/src/core/project/authority
  - zircon_editor/src/core/project/scene_document.rs
  - zircon_editor/src/core/project
  - zircon_editor/src/core/recovery/autosave
  - zircon_editor/src/core/recovery/session_guard
  - zircon_editor/src/core/recovery/activation_ledger
  - zircon_editor/src/core/editing/engine/journal/durable
  - zircon_editor/src/core/editing/engine
  - zircon_editor/src/ui/workbench/project/editor_workspace_persistence.rs
  - zircon_editor/src/ui/workbench/project/layout_preset_assets.rs
  - zircon_editor/src/core/extension/toolkit/save/source_write_authority.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/save.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/editing/node_ops.rs
  - zircon_runtime/crates/zr_resource/src/io
  - zircon_runtime/src/core/resource
  - zircon_runtime_interface/src/hub_protocol/recent_projects
tests:
  - zircon_editor/src/core/project
  - zircon_editor/src/core/recovery
  - zircon_editor/src/core/editing/engine/journal
  - zircon_editor/src/ui/host/asset_editor_sessions
plan_sources:
  - docs/plans/optimize/zircon_runtime/204-runtime-filesystem-resource-io-path-atomic-transaction-recovery-security-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/25-filesystem-path-uri-vfs-mount-watch-sandbox-atomic-io-review.md
  - docs/plans/optimize/zircon_editor/247-editor-scene-world-authoring-play-hierarchy-document-current-working-tree-review.md
  - docs/plans/optimize/zircon_editor/256-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-current-working-tree-review.md
  - docs/plans/optimize/zircon_editor/265-editor-settings-preferences-project-settings-scope-schema-overlay-persistence-migration-restart-plugin-window-current-working-tree-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/GenericPlatform/GenericPlatformFile.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/IO/IoDispatcher.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/Misc/PackageName.h
  - dev/bevy/crates/bevy_asset/src/io/mod.rs
  - dev/bevy/crates/bevy_asset/src/io/source.rs
  - dev/Fyrox/fyrox-resource/src/io.rs
  - dev/godot/core/io/file_access.h
  - dev/godot/core/io/dir_access.h
  - dev/godot/core/io/resource_loader.h
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Editor266 - Editor Filesystem / Project / Scene / Autosave / Journal / Session I/O 当前工作树复审

> 审查日期：2026-08-31
> 工作树基线：`630d66c362013e3b5b72f97362ad56fc54ff6d8c`；只读复核共享工作树，不回退其他变更。
> 审查性质：current-working-tree source review + refactor plan；不修改生产 Rust/Cargo/ABI/test/UI 代码。
> 排除范围：Tooling；未运行 Cargo、Editor 动态流程、真实 crash/fault/multi-process/scale/soak/benchmark，也未查询、轮询、等待或实时跟踪协调器。
> Runtime 边界：[Runtime204](../zircon_runtime/204-runtime-filesystem-resource-io-path-atomic-transaction-recovery-security-current-working-tree-review.md)；相关 Editor：[Editor247](247-editor-scene-world-authoring-play-hierarchy-document-current-working-tree-review.md)、[Editor256](256-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-current-working-tree-review.md)、[Editor265](265-editor-settings-preferences-project-settings-scope-schema-overlay-persistence-migration-restart-plugin-window-current-working-tree-review.md)

## 1. 结论

Editor 的本地恢复能力比一个简单的 temp+rename 原型复杂得多。当前已经存在：项目 authority 的 staging/backup/rollback；Scene document 的 Runtime staging、hard-link publication 和 catalog reconcile；Autosave 的 create-new sequence reservation、immutable snapshot、metadata digest、retention；Session Guard 的 Windows named mutex/Unix directory lock 与 platform-specific replacement；Activation Ledger 的 shared atomic writer；Durable Editing Journal 的 checksummed append、sync-data 和 compaction。它们是可以保留的工程底座。

问题不是 Editor 没有任何事务，而是这些事务没有共同的 Runtime I/O owner、root capability、mount/source generation 或 terminal receipt：

1. Project/Scene/Autosave/Session/Journal 各自实现 staging、rename/hard-link、directory sync、lock、cleanup 和错误映射；同一次编辑可能跨越多个 writer，却没有一个可恢复的 intent/commit point。
2. Editor `ProjectPaths` 的安全路径规则没有贯穿所有 authoring、extension、asset session 和 layout writer；多数 API 仍接受任意 `PathBuf`，check 后再 open/remove/rename。
3. Scene/catalog、autosave snapshot/metadata、workspace layout/config、extension asset source 的物理提交与 Editor generation/dirty state 分离；崩溃后可能看到文件已更新但 catalog、tab、dirty 或 recovery UI 尚未更新。
4. Session Guard 已经显式暴露 Windows `PublishedWithDurabilityUncertainty`，但其他 Editor writers 仍可能把 `Ok(())` 当作 durable；Editor 没有统一显示“已发布但需要恢复/清理”的状态。
5. Durable Journal 虽有强校验，但 owner lock、compaction、recovery ledger 与 Runtime durable transaction 不是同一个协议；它不能证明 Scene/Asset/Settings 的跨文档原子性。

本篇不新增独立底层 P0；继承 Runtime204 的 `FSIO-P0-001..003`，并把 Editor 侧接线列为 Cross-owned。新增 **39 项 P1（35 Open / 4 Partial）**、**10 项 P2（9 Open / 1 Partial）** 和 **20 个 Editor I/O gates（18 Fail / 2 Partial / 0 Pass）**。Runtime204 拥有 provider/path/receipt 内核；本篇拥有 Editor document/project/recovery/session 消费顺序、可见状态、迁移台账和 UI/operator contract。

## 2. 当前 Editor I/O 拓扑

```text
Editor Host / Project Session / Document Generation
              |
              +--> ProjectAuthority rename/backup/rollback
              +--> SceneDocument staging -> hard-link -> catalog reconcile
              +--> Autosave snapshot + metadata + retention
              +--> SessionGuard lease + private atomic mutation
              +--> DurableEditJournal append + compaction
              +--> Extension/AssetSession/Workspace/Layout private writers

              -X-> single Runtime FileSystemProvider / RootCapability
              -X-> single MountRegistry generation
              -X-> common IoOperationReceipt / RecoveryLedger
```

Runtime204 当前的 `zr_resource` public I/O 只有 atomic byte write，多文件 transaction 仍 assembly 内部；所以 Editor 暂时不能把 `zr_resource::io` 当作完整 provider。正确迁移顺序是先建立 Runtime provider/receipt，再逐个收口 Editor writer，而不是把更多 Editor helper re-export 成“统一 API”。

## 3. 已有基础与保留条件

| 基础 | 当前实现 | 保留条件 |
|---|---|---|
| Project authority staging | target→backup、staging→target、失败恢复、transaction path cleanup | 改用 Runtime durable directory transaction；返回 publication/recovery receipt |
| Scene staging | Runtime 写 staging，hard-link 到 source，失败时尝试移除 staging/source | 绑定 project root handle、catalog generation 和 parent sync |
| Autosave identity | create-new reservation、immutable snapshot、metadata digest、retention | snapshot+metadata 同一 bundle transaction；rotation 可恢复且可观测 |
| Session lease | named mutex/Unix flock-like directory lease、owner record | lease 保留为 Editor ownership；文件 publish hard-cut 到 Runtime provider |
| Activation ledger | shared atomic write/new、bounded read/schema | 成为 recovery projection；写入结果携带 provider receipt |
| Durable journal | framed checksum、sync-data append、restart parsing、compaction | Journal codec 与 storage backend 分离；避免再造第二 WAL owner |
| Editor dirty/generation | document/session dirty、savepoint、history generation、reconcile | publication 必须携带同一 document generation，禁止先改可见 state 再写盘 |

## 4. 继承 P0 与 Editor 责任裁决

| Runtime P0 | Editor 表现 | Editor 收口责任 |
|---|---|---|
| FSIO-P0-001 provider 缺失 | 每个 Editor domain 都能直接使用 `std::fs` 或 private temp/rename | Editor 只能提交 logical authoring intent 与 typed write set；所有物理 I/O 经过 Runtime provider |
| FSIO-P0-002 receipt 不统一 | Session Guard 已有 uncertainty，Project/Scene/Autosave/Journal 各自定义 success | 所有 Editor operation 显示并保存 `Published/Durable/RecoveryRequired/CleanupDeferred/Conflict` receipt |
| FSIO-P0-003 root handle 缺失 | ProjectPaths admission 与各个 check/open/remove 时序不一致 | Editor operation 必须持有 project/document/root capability；不能把 display path 重新当操作 path |
| FSIO-P0-004 publication 与 durable 终态分离 | Scene/catalog、asset session、settings/layout 可能在底层 deferred 后继续 publish | Editor 不得在 receipt 未达 contract 前提交 dirty-clean、catalog-visible 或 recovery-complete 状态 |

## 5. P1 详细差距

### 5.1 Project / Scene / Document publication

| ID | 状态 | 证据与重构方向 |
|---|---|---|
| ED-FSIO-P1-001 | Open | `core/project/authority/transaction.rs` 只用 rename 组织 directory publish，没有 WAL/commit point；改为 Runtime directory transaction |
| ED-FSIO-P1-002 | Open | authority cleanup 先 `is_dir/exists` 再 remove，存在 TOCTOU；只接受 capability-bound cleanup handle |
| ED-FSIO-P1-003 | Open | project backup/staging parent 没有统一 directory durability receipt；崩溃后无法判断目录 rename 是否持久 |
| ED-FSIO-P1-004 | Open | SceneDocument hard-link final 后才 reconcile catalog/source；crash window 可导致文件和 catalog generation 不一致 |
| ED-FSIO-P1-005 | Open | Scene rollback/remove 只返回 local error，未把 parent sync、catalog rollback、recovery action 编入 receipt |
| ED-FSIO-P1-006 | Open | Scene `Drop`/cleanup 静默吞掉 remove 失败；应进入 Editor recovery ledger 和 operator report |
| ED-FSIO-P1-007 | Partial | ProjectPaths/Scene source path 有 root admission 与 logical/display 分离，但不是每个 Editor extension/session writer 都使用 |
| ED-FSIO-P1-008 | Open | document save 没有 expected file identity/revision；外部 formatter/editor 可以在 publish 前修改源文件而被覆盖 |
| ED-FSIO-P1-009 | Open | project open/close 与 Runtime mount/source generation 未绑定；打开项目后 source 变化没有 pinned snapshot |
| ED-FSIO-P1-010 | Open | Scene authoring write set 与 history/dirty/savepoint 不是同一 transaction；可见状态可先于物理 publication 改变 |

### 5.2 Autosave / restore / session guard

| ID | 状态 | 证据与重构方向 |
|---|---|---|
| ED-FSIO-P1-011 | Open | Autosave snapshot 与 metadata 分开 publish，可能留下只存在其一的 orphan pair；改为 bundle manifest transaction |
| ED-FSIO-P1-012 | Open | Autosave retention 删除在独立 loop 中执行，删除失败/重启没有 durable cleanup receipt |
| ED-FSIO-P1-013 | Open | sequence reservation Drop 静默删除；崩溃残留会推进序号但没有统一 reservation recovery/explanation |
| ED-FSIO-P1-014 | Partial | snapshot 有 digest、immutable payload 和 bounded retention；缺 project/document/root/mount generation lineage |
| ED-FSIO-P1-015 | Open | project root resolve 失败可 fallback 到 caller raw path；安全上应 fail closed，不允许隐式外部路径 |
| ED-FSIO-P1-016 | Open | restore executor 与 autosave store 没有同一 conflict/CAS/expected identity；恢复可能覆盖用户刚修改的文件 |
| ED-FSIO-P1-017 | Partial | Session Guard 有 OS lease 和 Windows durability uncertainty，证明能力真实存在；仍重复 private atomic/platform implementation |
| ED-FSIO-P1-018 | Open | session mutation 先 `destination.exists()` 再 replace，存在 TOCTOU 和 external writer race |
| ED-FSIO-P1-019 | Open | session lock/owner record 没有统一 principal、document generation、expiry、recovery action schema |
| ED-FSIO-P1-020 | Open | Editor 没有把 recovery-required/cleanup-deferred/uncertain durability 映射为可操作的 resume/repair/forget 状态 |

### 5.3 Journal / history / extension / workspace

| ID | 状态 | 证据与重构方向 |
|---|---|---|
| ED-FSIO-P1-021 | Partial | Durable Editing Journal frame checksum、bounded records、sync-data append 是强基础；没有跨 document/project transaction owner |
| ED-FSIO-P1-022 | Open | journal store 没有 OS owner lock/multiprocess lease；`path.exists`+open race 允许两个 writer 共享 append path |
| ED-FSIO-P1-023 | Open | compaction 先写 temp、再读完整 temp 到内存、再调用 atomic write，造成重复 I/O/RSS 峰值 |
| ED-FSIO-P1-024 | Open | journal compaction 与 document savepoint/history generation 没有共同 commit receipt；重启选择依据不统一 |
| ED-FSIO-P1-025 | Open | extension toolkit `source_write_authority` 仍是 Editor-local physical writer，缺 plugin principal/capability/mount policy |
| ED-FSIO-P1-026 | Open | asset editor session save/node ops 可分别写 source/metadata/preview，缺 multi-file manifest 与 rollback |
| ED-FSIO-P1-027 | Open | workspace/layout persistence 继续消费 Foundation ConfigManager，与 Editor SettingsStore/Runtime Preference 形成多重 durable authority |
| ED-FSIO-P1-028 | Open | layout preset assets 没有 source generation/asset UUID/manifest，重命名和外部修改会形成 stale view |
| ED-FSIO-P1-029 | Open | activation ledger 采用 shared atomic primitive，但不记录 provider capability、parent durability、recovery correlation |
| ED-FSIO-P1-030 | Open | Editor background jobs/compile/preview 只提交局部 file effects，取消或关闭时没有统一 staged artifact cleanup |

### 5.4 Security / observability / scale / migration

| ID | 状态 | 证据与重构方向 |
|---|---|---|
| ED-FSIO-P1-031 | Open | Editor API 仍广泛接受 arbitrary `PathBuf`，没有 operation-specific root/document capability |
| ED-FSIO-P1-032 | Open | error 中可能显示绝对 workspace/project path；引入 logical URI、redacted display path 和 privileged forensic path |
| ED-FSIO-P1-033 | Open | 没有 Editor-level direct-fs allowlist/CI gate，新增 writer 容易重新复制 temp+rename |
| ED-FSIO-P1-034 | Open | 没有 per-document/project I/O budget、queue fairness、P95/P99/save latency、bytes copied 和 recovery-time metrics |
| ED-FSIO-P1-035 | Open | autosave/journal/preview 大 payload 仍以 Vec/全量 temp 处理，没有 streaming/spool lease/backpressure contract |
| ED-FSIO-P1-036 | Open | project/session close 只能等待局部 Editor maps，不能 join Runtime readers/watchers/provider handles；bounded close 未证明 |
| ED-FSIO-P1-037 | Open | mount/source generation 变化时没有 Editor reopen/rebind/reconcile protocol |
| ED-FSIO-P1-038 | Open | network/read-only/full-disk/locked-file/permission/reparse errors 没有一致的 Editor remediation and retry policy |
| ED-FSIO-P1-039 | Open | 旧 writer 迁移没有逐文件 ownership ledger；不能证明 Runtime/App/Interface 旁路已清零 |

## 6. P2 差距

| ID | 状态 | 方向 |
|---|---|---|
| ED-FSIO-P2-001 | Open | Editor operation/display/logical path 三元组与 URI/file URL 编码版本 |
| ED-FSIO-P2-002 | Open | autosave/journal/session reservation boot nonce、process identity、sequence exhaustion diagnostics |
| ED-FSIO-P2-003 | Open | metadata/ACL/xattr/ADS/permissions preservation policy for scene/assets/layout |
| ED-FSIO-P2-004 | Open | paged project/catalog/directory view，不在 UI thread 全量 `read_dir`/clone |
| ED-FSIO-P2-005 | Open | recovery dashboard：pending transaction、orphan pair、quarantine、last action、retry/forget |
| ED-FSIO-P2-006 | Open | cross-version journal/manifest schema migration and N-2 reader |
| ED-FSIO-P2-007 | Open | provider conformance fixtures for memory/fs/pack/remote/read-only in Editor preview and import |
| ED-FSIO-P2-008 | Open | I/O trace correlation with document generation, command group, frame/tick and preview job |
| ED-FSIO-P2-009 | Open | editor-side copy-on-write/reflink/preallocation policy for large scene/assets |
| ED-FSIO-P2-010 | Partial | existing local fault tests and bounded autosave tests；补 fresh-process/power-loss/ACL/reparse/soak |

## 7. Editor I/O 资格门

| Gate | 当前 | 退出条件 |
|---|---|---|
| ED-FSIO-G01 | Fail | Project/Scene/Asset save only through Runtime provider |
| ED-FSIO-G02 | Fail | document/project/root capability pinned for full operation |
| ED-FSIO-G03 | Fail | no check/open/remove/rename TOCTOU |
| ED-FSIO-G04 | Fail | scene source/catalog same generation receipt |
| ED-FSIO-G05 | Fail | autosave snapshot+metadata atomic bundle |
| ED-FSIO-G06 | Partial | session lease exists, writer not unified |
| ED-FSIO-G07 | Fail | Windows/Unix durability level visible to Editor |
| ED-FSIO-G08 | Fail | external writer CAS/file identity conflict |
| ED-FSIO-G09 | Fail | journal owner lock and bounded compaction |
| ED-FSIO-G10 | Fail | history/savepoint/document generation commit order |
| ED-FSIO-G11 | Fail | cleanup/recovery receipt never swallowed by Drop |
| ED-FSIO-G12 | Fail | workspace/config/layout single persistence authority |
| ED-FSIO-G13 | Fail | extension/plugin writer principal and capability |
| ED-FSIO-G14 | Fail | preview/import staged artifact cleanup on cancel/close |
| ED-FSIO-G15 | Fail | direct-fs CI allowlist |
| ED-FSIO-G16 | Fail | bounded per-document I/O budget and latency metrics |
| ED-FSIO-G17 | Fail | project/session close joins provider handles/tasks |
| ED-FSIO-G18 | Fail | crash/restart/full-disk/reparse product matrix |
| ED-FSIO-G19 | Fail | recovery dashboard/operator actions |
| ED-FSIO-G20 | Partial | activation ledger uses shared atomic primitive, lacks provider lineage |

## 8. 重构顺序与 Owner

1. Runtime204 先交付 `FileSystemProvider`、`RootCapability`、typed `IoOperationReceipt`、directory transaction、file identity/CAS 和 provider conformance；Editor 不在此之前增加新的物理 writer。
2. Editor 建立 `EditorStorageContext`：project identity、document identity、principal、mount/source generation、dirty/history generation、recovery policy 和 I/O budget 均在 session open 时冻结。
3. ProjectAuthority 与 SceneDocument 迁移为一个 Runtime durable directory transaction；catalog/index/dirty/savepoint 作为同一 logical write set 或显式 rebuildable projection。
4. Autosave 改为 snapshot bundle（payload + metadata + manifest + digest + retention intent），restore 使用 expected identity/CAS；orphan/retention 进入 recovery ledger。
5. Session Guard 保留 ownership lease，删除 private platform replace；所有 mutation 通过 Runtime receipt，并将 uncertainty 映射为 UI/operator action。
6. Durable Journal 保留 codec/semantic replay，下沉 append/compact/lock/recovery storage；compaction 使用 stream stage，不能全量读 temp 再写一次。
7. Extension/AssetSession/Workspace/Layout 迁移到 provider；Runtime203 的 Preference/Config authority 收口后，Editor 只持有 authoring projection，不再拥有第二 filesystem authority。
8. 迁移完成后启用 `zircon_editor` direct-fs CI gate，逐文件 ledger 必须有 owner、provider route、receipt consumer、recovery behavior 和删除日期。

## 9. 测试与验证要求

1. fresh process 在 project rename、scene hard-link、catalog commit、autosave pair、session mutation、journal compaction 每个阶段 kill/restart。
2. 外部 editor/formatter 与第二个 Zircon Editor 同时修改同一 scene/layout/asset，必须得到 typed conflict，不得静默覆盖。
3. Windows reparse/UNC/ACL/ADS、Unix symlink/hard-link/non-UTF8/network share 与 case-folding。
4. 取消、关闭、hung provider、locked file、full disk、permission denied、volume removal；证明 close deadline 和 no late publish。
5. large scene/autosave/journal/preview streaming 的 RSS、bytes copied、write amplification、queue wait、P95/P99 和 recovery time。
6. UI 必须消费 `PublishedWithDurabilityUncertainty`、`RecoveryRequired`、`CleanupDeferred`、`Conflict`，禁止把 static success/queued 文案当作完成。

## 10. Owner 与非重复边界

| Owner | 本篇拥有 | 其他报告 |
|---|---|---|
| Runtime204 | provider/path/root/receipt/durable primitive | 本篇不重新实现 Runtime filesystem |
| Editor266 | Editor session/document/project/recovery consumption order、visible state、migration ledger | Editor247/256 继续拥有 scene/world/command history semantics |
| Runtime203 | Preference/Config durable authority and multi-process config | 本篇只收口 Editor layout/config consumer |
| Hub/Interface | public DTO/lease intent | Interface 不再持有低层 temp/file/lock implementation |
| Tooling | cook/package/export/distribution | 本篇不进入 Tooling 目录 |

禁止在 Editor 新增第二个 `atomic_write`、private `FileAccess`、temp naming、parent sync、platform ReplaceFileW、owner lock 或 recovery journal。Editor 可以保留 domain journal codec、lease policy、recovery UI 和 authoring intent，但物理文件动作必须由 Runtime provider 执行并返回可持久化 receipt。

## 11. 本轮状态

本轮只完成 Editor 源码复审、Runtime 边界引用、参考引擎对照和重构顺序；没有生产实现或动态验证。Editor 目前具有局部可靠恢复组件，但不能宣称 project/scene/autosave/journal/session 已构成一个工程级 durable storage system。

## 状态与产出记录

| 里程碑 | 状态 | 日期 | 证据 |
|---|---|---|---|
