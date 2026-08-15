---
date: 2026-08-15
related_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
doc_type: structural-performance-research
status: r8_production_green_physical_path_identity_fixed_focused_test_foreign_compile_gate_pending
coordination_owner: docs/plans/zircon_runtime/frameworks/01
related_code:
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import/projected_inventory.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import/full_generation.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import/sources.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import/targeted.rs
  - zircon_runtime/src/asset/project/manager/durable_transaction.rs
  - zircon_runtime/src/asset/registry/inspection.rs
  - zircon_runtime/src/asset/registry/rebuild.rs
  - zircon_runtime/src/asset/registry/persistence.rs
  - zircon_runtime/src/asset/artifact/store.rs
  - zircon_runtime/src/asset/project/generation_observation.rs
  - zircon_runtime/src/asset/project/paths.rs
  - zircon_runtime/src/asset/migration/transaction.rs
  - zircon_runtime/src/asset/migration/transaction/recovery.rs
  - zircon_runtime/src/core/resource/io/atomic_file
  - zircon_runtime/src/core/resource/io/transaction
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/resource_publication.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/open_project.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs
references:
  - docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Private/AssetRegistryState.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Public/AssetRegistry/AssetRegistryState.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/Commandlets/AssetRegistryGenerator.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Private/AssetDataGatherer.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/Paths.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/PathViews.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Experimental/IoStore/OnDemand/Private/JournaledCache.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/StorageServerClient/Private/Cache/CacheJournalSectioned.cpp
  - dev/bevy/crates/bevy_diagnostic/src/diagnostic.rs
  - dev/bevy/crates/bevy_diagnostic/src/frame_time_diagnostics_plugin.rs
  - dev/bevy/crates/bevy_render/src/diagnostic/render_asset_diagnostic_plugin.rs
  - dev/bevy/crates/bevy_asset/src/processor/log.rs
  - dev/bevy/crates/bevy_asset/src/processor/mod.rs
  - dev/bevy/crates/bevy_asset/src/server/info.rs
  - dev/Fyrox/fyrox-resource/src/registry.rs
  - dev/godot/core/io/resource_uid.cpp
tests:
  - projected metadata inventory and registry parity
  - full-generation prepare/commit fault matrix
  - durable journal crash/restart recovery matrix
  - Resource reservation, project generation, and event-last ordering
  - coordinator-managed Windows WPR/ETW scale and power evidence
---

# Frameworks01 M1 Project Generation Durable Transaction Review

## Status

本记录是结构性复核、R5/R6 实现状态、R7 typed observation 实现状态和后续测量计划，不是性能优化完成记录，
也不是 M1 验收证据。
当前已完成代码流、故障窗口、现有 AssetMigration journal 以及 Unreal/Bevy/Fyrox/Godot 本地源码复核，
并完成 R5 projected inventory/full-generation 与 R6 durable transaction 非验收实现硬切；尚未取得
WPR/ETW、耗时、RSS、I/O 或功耗样本。R6 已写入 Core frame WAL、restart recovery、owner lock、
Project/AssetMigration policy adapter 以及 crash/fault/restart tests，但共享 `zircon_runtime` lib 仍被 Scene、
Graphics、UI 的 6 条外部错误阻断，测试 target 没有生成，不能记为 GREEN 或完成整体 R6。
最新 source fingerprint 上的 Windows managed production build 运行 279.6 s，输出 6 errors / 207 warnings：
Scene 重复 `first_stable_camera_entity` 3 条、Graphics `BufferViewMut::as_mut` 1 条、UI route move 后使用
2 条，R6 owned production errors 为 0。前一轮 354.5 s build 暴露的 2 条 `TryLockError` 映射错误已按
当前标准库 `WouldBlock | Error(io::Error)` 枚举修复；更早 fresh-directory `std::fs` import 错误也已修复。
这些 repair build 只证明 production 类型边界收敛，不替代未运行的 crash/restart tests。
全库 docs convention gate 已完成并返回 RED：652 条既有 path violation、影响 235 份文档；本记录与父计划
命中 0 条。全库 structure convention 组合门在 184.1 s 超时，无完整结果，因此不记为 GREEN；R6 自有生产
owner 的行数预算由精确文件计数单独核验。

R7 已完成算法优化前的第二轮 current-source 与参考引擎复核，并只落地测量基础设施，尚未改变 discovery、
import、索引或并行算法。新增 `ProjectGenerationPhase`/`ProjectGenerationObservation` typed owner，复用 Core
profiler 的 bounded span/counter store；disabled build 不读取文件、不分配 counter batch、不获取 recorder lock，
capture inactive 时不发布 counter。metadata parse 改为一次 `read_to_string` 后复用同一 buffer，source/artifact/
write bytes 和 chunk 数全部取自原提交路径已有的内存数据，没有为观测增加文件 I/O。二次 foundation-DAG
复核拒绝了 Core transaction 直接调用 Runtime profiler 的首版接线：`zr_resource -> diagnostics/runtime` 生产
反向边已清零。Core 现在只返回 resource-neutral `DurableRecoveryReport`，Project adapter 在高层投影 recovery
rollback/cleanup/orphan-cleanup counters；Migration 只消费 typed result。该轮没有用 commit error 猜测 live
rollback；R7 后续已增加 caller-owned
`DurableCommitReport`：每次实际 restore 前累加 attempt，restore 成功后才另计 success；即使 journal
`RollingBack` transition 写失败也继续恢复 live 文件，并保留 journal 供 restart 幂等恢复；
Project adapter 在映射成功/错误前投影，Migration 仍不依赖 profiler。crash-after-replace 不计 live rollback，留给
restart recovery report 计数，避免与恢复路径重复。该补充已通过静态门，但尚未完成 managed compile/test。

最终 Project 调用链复核发现 `DurableCommitReport + Result<()>` 仍会吞掉提交点同步失败：Project 会安装新候选并
向同步调用方报告成功，而重启若丢失最后 `all_committed` frame 则会按 `active` 恢复旧代。R8 已把 Core 返回面
硬切为不可忽略的 `DurableCommitDisposition::{Durable, CommitRecoveryDeferred, CleanupDeferred}`；Project 文件提交
将未决结果保留为带 journal 路径的 typed outcome，资源发布层把 outcome 透传到文件、Project/source-path、
Resource apply 和 generation publish 之后，再向同步入口返回明确的 durability error。watch 路径先发布已安装代，
再广播 pending-recovery diagnostic；Migration 直接把同一 disposition 映射为 rerun-apply 错误。新增 Core
disposition contract 与 ProjectManager“先安装新代、再报告未决耐久性”回归测试；当前仅 exact rustfmt/diff-check
GREEN，尚未取得 current-hash managed compile/test ticket。
原 834 行 `project_asset_manager/runtime.rs` 收敛为 583 行，资源 prepare/reserve/file+project commit/apply 顺序
迁入 281 行 `resource_publication.rs`；`generation_observation.rs` 396 行、加入 phase 接线后的
`full_generation.rs` 680 行，均低于 800 行结构预警线。

2026-08-15 R7 default-feature Windows managed production build 已进入协调器唯一 D 盘 target pool；构建输出
11 errors / 209 warnings，R7 owned production errors 为 0。11 条均为共享 current source 外部错误：IBL 旧
`atomic_write` import 3 条、render scale 常量 1 条、Scene camera API 重复/歧义 3 条、Graphics buffer view
1 条、upscaler `Hash` 1 条、UI route move 后使用 2 条。由于 `zircon_runtime` lib 未生成，R6 crash tests 与
R7 profiling-feature tests 均未执行；default path 的 owned compile clean 不等于 milestone GREEN。随后执行的
`--features profiling` managed production build 输出 13 errors / 190 warnings，R7 profiling cfg owned errors 同样
为 0；新增 2 条均来自 Graphics text profiling 私有 re-export。Cargo/rustc 已退出；coordinator descriptor
在首次 release 阶段消失，恢复后已将 job `61641e9d350c48ccb649db1ff276c458` 以 request
`6f961162f36245c39637029ab065b55b` release，进程树为空，D 盘兼容池按 retained 策略保留。该轮仍因 lib RED、
test target 未生成而不是通过的 managed validation。
Resource-neutral report 边界修正发生在上述两次 managed build 后，并通过 exact rustfmt/diff-check 与生产
reverse-edge source scan。post-correction profiling managed job `56f4fc9dbc5e4d98bb20c275b7078ee3` 于
13:23:30--13:28:03 执行约 273 秒，wrapper wall 345.8 秒，输出 28 errors / 192 warnings，R7 owned error 为 0；
28 条均属于共享 current source：IBL 3、Graphics text profiling 2、render 2、Scene 3、Graphics buffer 1、
Script gameplay host 15、UI route 2。job 已于 13:28:11 release，进程树为空，D 盘 target pool retained；日志位于
`D:\zircon-frameworks01-r7-post-correction-profiling-build.log`，SHA-256 为
`ed2b1e72df96eb507315c50cd8c3a2957c2ef39bfd996f6f03e329f1c21dda0c`。lib RED 仍阻止新增 Core/Project
report tests 与 R6 crash/restart matrix 生成 test target，因此不能记 GREEN。R7 文档更新后全库 docs convention gate
再次返回 RED：652 条
既有 path violation、影响 235 份文档；
本记录与父计划仍命中 0 条，JSON 证据位于 `D:\zircon-frameworks01-r7-doc-conventions.json`。

2026-08-15 后续结构复核将 963 行 atomic-file flat owner 硬切为 folder-backed owner：根 façade 55 行，
`directory/pathing/platform/recovery/transaction` 分别为 104/46/188/153/274 行，测试 owner 180 行，生产与测试
文件均低于结构预算。拆分保持单文件 staged-write、backup、replace、directory barrier 与 Windows restore 的原有
顺序，不改变事务算法；Unreal PlatformFile、Godot FileAccess/DirAccess 与 Bevy file backend 只用于确认平台原语、
通用编排和上层 writer 的分层边界。原子写的 curated consumer 入口确认为 `core::resource::io::atomic_write`，已在
Resource I/O 根导出，因而 Shader06 三份 IBL blob 无需改写即可消除其 E0432。R8 对 Git tracked 与 nonignored
untracked current source 重扫后，完整旧 `core::resource::io::atomic_file::*` 清单为 31 个文件、35 处引用；本轮
已在同一 audited scope 中全部迁移到 `core::resource::io` 根 façade，并将 `atomic_file` 从 `pub mod` 收为私有
实现模块。产品公开面只保留 `atomic_write`，fault/recovery/staging helper 只以 `pub(crate)` 从 `io` 根提供给
Runtime 内部 owner，不保留 forwarding module 或 compatibility alias。目录化结构测试继续分别读取
transaction/platform child owner。

folder cut 后的 Windows `core-min` managed job `aa2ae675de25496bb20934955b48daba` 于
14:55:51--14:59:41 运行约 230.2 秒，输出 1 error / 83 warnings；唯一错误为 foreign
`core/framework/render/camera.rs:84` 缺少 `DEFAULT_RENDER_RESOLUTION_SCALE`，`atomic_file` 与三份 IBL consumer
均为 0 error。job 于 14:59:49 release，live process 为空，D 盘 pool retained；日志位于
`D:\zircon-frameworks01-r7-atomic-file-folder-build.log`，SHA-256 为
`69db92b7b5906fc489937cd15f8026471652a52051590cad93afd568768273a7`。该 build 发现并静态修正了 Windows
`platform.rs` 的非 Windows `std::fs` import 与 staged-write type re-export warning；由于 foreign lib error 阻止
test target 生成，目录化 test cfg、R6 crash/restart matrix 与里程碑验收仍未执行，不能记 GREEN。
拆分后 `core/resource` 为 57 个 Rust 文件、10,232 行；interface resource 保持 14 文件、923 行。
folder cut 后的全库 docs convention gate 返回 660 条 path violation、影响 238 份文档；本记录与父计划命中
0 条。已删除的 `core/resource/io/atomic_file.rs` 仍被 6 份 foreign 文档引用 8 次，必须由后续 audited
scope-transfer 同批迁移，当前 Session 不越权改写。JSON 证据位于
`D:\zircon-frameworks01-r7-atomic-folder-doc-conventions.json`，SHA-256 为
`3b535c65438a069e40fa2d19fdc0a65a4d134bba69ffa8e81699387f26f17c86`。

R8 在写入前完成最终 scope rotation：session
`frameworks01-m1-durable-file-transaction-hard-cut-r8-20260815` 的 immutable scope 为 158 项，补入此前漏扫的
`scene/world/project_io/document.rs` 与 product framebuffer proof consumer。ownership transfer preview
fingerprint 为 `c34b0bf4aeb11a3c5e8317607c0d5d45a3cfd1913d7d628ceb8d19c4903e0539`，129/129 current blobs
eligible、blocking reason 0；apply request `6e6f1810a7964d85b19a48f1cb829fa2` 原子成功。迁移后全工作区
Rust 旧模块路径为 0 文件/0 引用，`pub mod atomic_file` 为 0，公开根 `atomic_write` 导出恰为 1；32 个精确
Rust 文件的 rustfmt check 与 scoped diff-check 均为 GREEN。4 份 canonical 模块文档中的 6 处已删除文件路径
已迁到 folder owner；全 `docs` 只剩 2 份 foreign 历史 failure 记录中的 3 处旧路径，本 session 不冒领其历史
归属。atomic/durable owner 当前为 20 个 Rust 文件、4,644 行，最大 owner 742 行；其中 transaction 子树为
13 文件、3,644 行。

单文件 façade 仍有一个必须独立 hard-cut 的返回语义问题：existing/new target 在 replace 后发生 committed-file
sync、parent barrier 或 backup cleanup 错误时，`atomic_write -> io::Result<()>` 无法告诉 consumer 新字节是否已经
可见。R8 的 multi-file owner 只用它发布唯一命名的 immutable Intent；该错误不会触碰 live generation，并由
journal 或严格保留名 orphan 在重启时清理，因此不影响本轮 generation 原子性。先前 31 个文件是旧模块路径
迁移清单，不等于 live call owner 数；后续必须先完成逐 owner 行为审查，再统一切换 typed atomic disposition，
本轮不以吞错或兼容 wrapper 伪装解决。

### Atomic write commit-point hard-cut review (2026-08-16)

current source 重扫得到 23 个 façade import 文件，其中 21 个是 production owner；实际执行
`core::resource::io::atomic_write` 的文件为 15 个，其中 14 个是 production owner。其余 import 只用于
policy adapter、source contract 或测试。另有 font bake 与 shader variant cache 的同名本地函数，不属于本
façade，不能用文本命中数混入 API scope。

现实现的真实 commit point 不是函数返回 `Ok(())`，而是下列平台原语成功：

| 路径 | commit point | commit point 后仍可能失败 | target 状态 |
|---|---|---|---|
| new target | `rename` / `MoveFileExW` 成功 | parent directory barrier | 新 target 已可见，重启持久性未确认 |
| existing Unix target | staging `rename` 覆盖成功 | parent barrier、backup remove/barrier | 新 target 已可见，backup 可能保留 |
| existing Windows target | `ReplaceFileW` 成功 | committed-file sync、parent barrier、backup cleanup | 新 target 已可见，backup 可能保留 |
| Windows replace failure | replace 原语返回失败 | backup restore 也可能失败 | old target 已恢复，或 canonical 缺失且 backup 保留 |

因此 public hard cut 必须返回不可忽略的 typed commit，而不是 `io::Result<()>`：

```rust
#[must_use]
pub struct AtomicWriteCommit {
    pub durability: AtomicWriteDurability,
    pub cleanup: AtomicWriteCleanup,
}

pub enum AtomicWriteDurability {
    Confirmed,
    RecoveryDeferred,
}

pub enum AtomicWriteCleanup {
    Complete,
    Deferred { recovery_path: PathBuf },
}

pub struct AtomicWriteError {
    pub target: AtomicWriteFailureDisposition,
    pub source: io::Error,
}

pub enum AtomicWriteFailureDisposition {
    Unchanged,
    Restored,
    RecoveryRequired { backup_path: PathBuf },
}
```

最终命名可在 implementing successor 中按 Rust owner 复审微调，但信息维度不得压缩。`Ok(AtomicWriteCommit)`
只表示新字节已经发布；`Confirmed + Complete` 才等价于旧调用者以为的 durable success。commit point 前失败
返回 `Unchanged/Restored`，canonical 缺失且只剩 backup 时返回 `RecoveryRequired`。错误类型必须保留原始
`io::Error` source 和 recovery path，禁止让 consumer 解析 Display 文本。public surface 仍只从
`core::resource::io` 根导出，`atomic_file` 实现模块保持 private；旧 `io::Result<()>` wrapper、forwarding module
或 `Into<()>` escape hatch 全部禁止。

consumer policy 分为四类：

| consumer class | owner 示例 | `RecoveryDeferred` | `Cleanup::Deferred` |
|---|---|---|---|
| WAL/bootstrap | durable transaction initial frame | 不允许进入 live replace；停止 transaction 并保留/清理证据 | durability confirmed 后可记录 orphan cleanup，再继续 |
| authoritative document | scene/project document、workspace、Hub shared state、HDRI preference | 返回 typed pending durability，禁止调用方按“未写入”盲目 rollback/retry | 新值保持 authoritative，返回 warning/diagnostic |
| immutable/regenerable | artifact chunk/manifest、IBL derived/cache、pipeline cache | 可按 owner policy 报 pending 或丢弃后重建，但必须记录 counter/diagnostic | 接受已发布值并异步/启动清理 orphan |
| proof/tool output | framebuffer proof 等 | 保持非 durable result 可见于测试/CLI | 输出 retained path，不能静默吞掉 |

`transaction::journal::write_initial_frame` 是最严格 caller：只有 `Confirmed` 才允许继续 live document replace；
它不能因为 journal path 当前可读就把 parent barrier 失败当成功。cache caller 可以接受可再生结果，但该 policy
属于 cache owner，不进入通用 Resource IO。authoritative caller 必须把“已发布但 durability pending”向上投影，
不能再用普通 `?` 把它伪装成“目标未改变”。

当前 R8 immutable scope 缺少以下 live caller，因而本 session 不实施半迁移：

- `zircon_runtime/src/asset/artifact/ibl_bake_artifact_asset_derived.rs`；
- `zircon_runtime/src/asset/artifact/ibl_bake_artifact_cache.rs`；
- `zircon_editor/src/core/hub_link/focus_signal.rs`；
- `zircon_editor/src/core/hub_link/handshake.rs`；
- `zircon_editor/src/core/hub_link/recent_writeback.rs`；
- `zircon_app/src/bin/zircon_shader_pbr_viewer/hdri.rs`。

其中两份 IBL owner 归 Shader06，UI12 的 E0432 仍是 stale source fingerprint，不是 consumer rollback 理由；
三份 Hub owner 与 App owner 需要各自 active plan 同意 ownership transfer。successor 必须一次性包含 Core type/
platform/recovery/tests、全部 15 个 call owner、23 个 import/source-contract owner 及对应文档，按 current blob hash
preview/apply 后才允许改 public signature。

验收至少覆盖 new/existing target 的 pre-commit write/sync/replace、post-commit committed sync/parent barrier/
cleanup、Windows backup restore success/failure、Unix hard-link fallback、stale staging/orphan recovery，以及四类
consumer policy。fault matrix 必须断言 bytes visibility、durability disposition、backup/staging existence 和调用者
authority 是否安装，不能只断言 `is_err()`。这是 correctness hard cut，不是 barrier batching 性能优化；在同机
WPR/ETW 证明 directory/file sync 是 material bottleneck 前，不减少 correctness barrier。

- [x] 复核 atomic-file current state machine 和平台 commit point；
- [x] 区分 23 个 import owner、15 个 live call owner与同名本地 helper；
- [x] 固定 typed commit/error 信息维度和四类 consumer policy；
- [ ] 取得 Shader06、Editor Hub、App owner 的完整 current-hash transfer；
- [ ] 单次 hard cut public signature、全部 caller 与 fault matrix，不保留旧返回面；
- [ ] managed Windows/Unix-specific compile、fault/recovery test、独立 review 与性能 baseline。

本轮同时读取 `docs/plans/optimize/zircon_runtime/03` 与 `04` 的 current-source 工程审查。R8 只是 durable
foundation 与 façade hard cut，不是算法性能优化；后续 R7-C/R7-D 继续受可信 telemetry、source fingerprint、
raw sample、p50/p95/p99、RSS/I/O/功耗和同条件复测门约束。Asset exact identity、async residency、semantic
sections、last-good reload、bundle mount/update 等 P1/P2 属于后续 Runtime04/对应 numbered plan，不在本切片
临时扩张或重复实现。

R8 的 Windows managed `core-min` production build job
`e4947b70644e4ff7822156eba264a5f9` 于 16:44:06--16:50:44 执行约 398 秒，Cargo build 成功，
`zircon_runtime` lib 为 281 warnings、`zircon_runtime_interface` 为 4、export-pack bin 为 8，合计 293 warnings、
0 errors；D 盘 target pool 为
`D:\cargo-targets\zircon-engine\pool\f9fef644bf8e441a49ad1c139495499657f126cd246ffca80d13868db535561d`。
该 build 唯一命中本切片的新增 warning 是 production 图未使用的 `NEXT_ATOMIC_FILE_ID` 根重导出，随后已把它
收为 `#[cfg(test)] pub(crate)`；该修正不改变 production 行为，但使源码指纹变化，因此不把前一张 build 票据
冒充最终 current-hash 验收票。

最终 current-hash 的 Windows managed lib-test job `bf4d78e6e007484ea18fadc7e7a92fe9` 于
16:58:48--17:08:43 执行约 596 秒并以 exit code 1 结束；测试二进制未生成、测试数为 0。D 盘 Cargo
fingerprint 的结构化 rustc 诊断为 58 errors / 1,352 warnings，`core/resource/io` 与三份 IBL consumer 均为
0 条直接诊断。58 条编译错误分布于 14 个 foreign source owners：Asset project runtime/service/full-generation
tests 9 条、Render 2 条、ResourceManager transaction test 1 条、Scene/ECS 46 条；其中包括 runtime tests 的
4 处 stale `include_str!`、Scene identity-storage 测试的 25 条 missing type、以及 Scene schedule/test API drift。
由于 Cargo 的 test filter 仍先编译完整 `zircon_runtime` lib-test 图，`core::resource::io` 聚焦测试没有运行，
不能把 production build GREEN 或 owned-error=0 提升为 R6-C/R6-D 验收完成。该 job 已 release，进程树为空；
随后共享 D 盘池由 UI12 managed `zircon_editor supersamples` job 接管，本 Session 未取消或并发启动第二张 Cargo。

从该结构化诊断中，R8 registered scope 内有 10 条直接 test-contract 漂移：拆分后的 runtime test owner 有
4 个 stale `include_str!` 相对路径，service-contract tests 缺 2 个 `ResourceMutationBatch` 解析，full-generation
tests 缺 `ResourceRecord` 导入与 2 个 caller-owned `DurableCommitReport` 参数，ResourceManager transaction test
有 1 个 consumed record 后继续取 id。R8 已按当前生产签名完成最小同步：只修正相对路径、显式 test import、
传入默认 typed report，并在 consume 前保存 `ResourceId`，不改变 durable transaction 或 asset-generation 算法。
4 文件 exact rustfmt/diff-check GREEN，4 个 `include_str!` 目标均存在；剩余 observed 48 条 foreign error 未改写。

修复后的首次受管复验在进入 Cargo 前失败：validator 注册阶段读取 coordinator 生成的 `%TEMP%` failure snapshot
时返回 Windows `Access denied`，随后服务状态为 `offline/descriptor_absent`；没有创建新 Cargo job、没有 test
result，也未删除 C 盘临时目录或绕过协调器。因此该次只能记 infrastructure preflight failure，待 daemon
rollover 恢复后仍须执行 current-hash managed compile/test。

随后对 durable transaction 全状态机的第二轮 current-source 复核发现两个结构性错误。其一，所有 live 文件
已替换后，`all_committed` frame 的完整写入与 `sync_all` 失败被压缩成普通 `Err`，Project caller 会拒绝安装
当前可见的新内存代际；其二，`all_committed` 已确认后，`cleanup` transition 或 artifact 删除失败仍返回普通
提交错误，同样造成磁盘新代与内存旧代分叉。R8 已硬切为 typed commit-point outcome：完整 frame 写入前失败
继续恢复旧代；完整 frame 已可见但 durability barrier 失败时安装当前可见新代、保留全部 journal/backup 并
记录 `deferred_commit_recovery_count`，下一次启动按 commit frame 是否真正持久化裁决新旧代；已确认
`all_committed` 后的 cleanup 失败返回提交成功、保留恢复证据并记录 `deferred_cleanup_count`。Project adapter
投影两项 Resource transaction counter；Migration 没有长期内存 authority，遇到 deferred commit recovery 时
返回明确的待恢复裁决错误，不静默宣称 durable apply 成功。rollback 路径的再次复核发现，`rolling_back`
transition 失败后继续恢复 live 文件、却保留 `Active + Committed` journal，会使启动校验把已恢复旧 bytes 判为
非法；若失败 append 留下 torn tail，后续继续 append 还会把尾部损坏变成非尾部损坏。R8 现已在首次 append
失败后停止全部后续 journal append，继续逆序恢复 live 文件；全部恢复成功时先删除 journal、再清理 backup，
恢复未完成时保留完整证据，并允许 `Active/Committed` 文档以 old-or-new bytes 进入幂等启动回滚。
原 5 条 Core fault contract 继续覆盖 commit-point incomplete write、commit-point unsynced complete frame、
cleanup transition、committed artifact cleanup 和 rollback transition failure，另新增 1 条 active committed old-bytes
restart contract；精确 Rust 文件已通过 `rustfmt --check`。Migration 另有 1 条 adapter contract，锁定 unsynced
commit point 必须报告 pending recovery 而非 durable apply。这属于正确性/恢复算法修复，不是性能优化，未绕过
R7-C/R7-D 的实测门。

同一轮 staging 顺序复核还发现 staged file 只执行了 file `sync_all`，却在发布 durable `prepared` frame 前漏掉
父目录 barrier；掉电后可能保留 prepared journal、同时丢失 staging 目录项。R8 已把 staging parent directory sync
固定在 file sync 之后、backup 与 `record_prepared` 之前，并增加 pre-publication fault contract：barrier 失败必须保持
旧 live target、清空 journal/staging，不得进入 active phase。

启动恢复的后续复核又发现 decoder 虽会忽略 torn terminal frame，recovery 却会直接在 torn bytes 后追加 transition，
使下一次启动把原尾部损坏判成非尾部损坏。R8 现先完成所有 journal/policy/digest 验证，再把每份 journal 截断到
最后一个完整 frame 并 `sync_all`，之后才允许 recovery append；`detect_pending_transactions` 继续保持只读。新增 frame
合同验证“torn tail -> truncate -> append -> fold”仍得到合法 active journal。

pre-active abort 的清理顺序也从 artifact-first 修正为 journal-first。此前 staging/backup 已删除后若 intent journal
删除失败，会留下引用缺失证据的不可恢复记录；现在 journal 删除失败时全部 evidence 保持不动，journal 删除成功后
才 best-effort 清理保留名 artifact。新增合同以不可删除的 journal owner 验证 staging evidence 不会被提前删除。

同一规则已延伸到 restart recovery 的 `Intent` 分支。该阶段尚未发布 live 文件，恢复会先移除 journal authority，
再清理 staging/backup；journal 删除失败时保留全部证据，删除成功后的 artifact 清理失败最多留下无权威的保留名文件。
已进入 `cleanup`/`cleanup_rollback` 的 durable phase 仍按 artifact-first 收尾，因为该阶段允许证据逐项消失。

`rollback_completed` 的重启证据也已校正：live target 已恢复且 phase 已持久化后，先前发布过的文档不可能仍有
被 replace 消耗的 staging 文件。恢复因此只允许该 phase 的 staging 缺失，仍强制校验旧 live bytes、backup 与
retired evidence；两文件 fault contract 覆盖“第一份已发布、第二份提交前失败、回滚完成即崩溃、重启清理”。

写入端现与读取端共享 journal 总量上限。普通 transition 与 commit-point frame 在 append 前读取当前文件长度并
拒绝超过 128 MiB 的累计结果，避免引擎自己生成下一次恢复必然拒绝的 journal；64 MiB 单帧上限保持不变。
稀疏满额文件合同验证两类 append 均失败且文件长度不变。

该修复后的 Windows managed focused lib-test 尚未进入 Cargo：validator operational session 的两次
`session.register` 请求 `60a0db4ef36745f59a71b7cb9b782532` 与
`8fe89f5a515549a49c005f3f3da16feb` 均被 coordinator 接受，但客户端 15 s reconciliation 窗口内无终态；
精确 `request-status` 后两者均为 `completed`，只完成 operational session 注册，没有 Cargo job、target 写入或
测试结果。daemon rollover 后一次 acquire 返回 `admission_checkpoint_stale`；随后共享 target 的
`101b0d362f8347d88905d4f9fd44a5bb` 与 `863935ab0cc34aa3ab0645b48cd3b214` 两个外部
`validate-matrix` job 在本 Session acquire 时仍被报告 `cargo_process_tree_alive`，协调器事后均记录为
`orphaned`、`live_process_pids=[]`，且没有为本 Session 创建 managed run。所有进程树均自然退出，未 cancel、kill、
cleanup 或绕过协调器；因此这几次仍只有 control-plane/shared-target preflight 证据，没有新的源码编译或测试结果。
current-hash compile/test 与独立 reviewer 仍是 R8 accepted closeout 的必要条件。

为排除共享 pool 干扰，又执行了一次脚本正式支持的 `-Ephemeral` 受管复现；协调器分配全新的
`F:\cargo-targets\zircon-engine\ephemeral\test\c74fcae9084f4858baf5a4203b21b6d9`，仍在
`cargo start` 前返回 `cargo_process_tree_alive`，把 validator PowerShell root PID `16832` 及其协调器客户端子进程
误判为既有 Cargo tree。job `c74fcae9084f4858baf5a4203b21b6d9` 的终态为 `started_at=null`、
`orphaned`、`live_process_pids=[]`，F 盘 ephemeral target 已由服务删除且 cleanup error 为 null。这证明阻塞位于
`.codex/skills/zircon-dev/scripts/validate-matrix.ps1` 的 acquire/start PID 协议或 coordinator Cargo start
process-tree 判定，而不是 Frameworks 源码或共享 target 内容。修复验收必须覆盖普通 managed 与 ephemeral lane：
acquire owner shell 在调用 `cargo start` 客户端期间产生子进程不得被当成预存 Cargo tree；真正遗留的 cargo/rustc
后代仍必须被拒绝；原聚焦命令必须实际进入 Cargo 并产生 terminal result。Coordinator01 当前 primary 的 immutable
scope 只覆盖另一条 failure closeout，故本 Session 不越权创建或修改 Tooling failure 文件；handoff 等正确 owner
scope rotation 后物化，本计划继续保持 validation pending。

2026-08-15 21:15，Tooling owner 已以 commit `543497826ef0b48890a3d16c8f628ac5866cff75`
完成 pre-start 修复；focused Pester pre-start 3/3 与 managed environment 3/3 均为 GREEN，coordinator
successor schema 63 healthy/read-write，R8 原 Cargo 启动阻塞已解除。共享窗口随后由 UI12 managed
focused job 与 coordinator-owned Shader06/IBL contract Cargo 占用；R8 遵守短窗口协调，不取消、不并发，
等待既有任务自然释放后立即执行 current-hash production build 与 focused lib-test。

UI12 报告时的三份 IBL consumer 隔离快照均使用 `crate::core::resource::io::atomic_write`，与 R8 私有
`atomic_file` owner、根级公开 façade 的 hard cut 一致；该快照不需要退回旧模块，也不应重新公开
`atomic_file` 模块。本轮未改写任何 Shader06/IBL 文件。最新 tracked + nonignored untracked 扫描在当前实际
存在的 17,208 个 Rust 文件中得到旧模块路径 0、公开 `atomic_file` 模块 0、根 façade 导出 1；共享 checkout
并发移动导致 Git 列表中另有 57 个 foreign 路径在读取前已消失，已从 current-source 统计中显式剔除。

2026-08-16 复核 UI12 的隔离编译指纹后，三条 E0432 被归类为 patch 依赖顺序，而不是新的 consumer
migration：隔离图包含当时的 Shader06 IBL patch，却没有包含尚未 service-commit 的 Frameworks01
`core/resource/io/mod.rs` façade patch。共享 current source 中 `core::resource` 公开 `io`、`io` 恰好公开一份
`atomic_write`，且 `atomic_file` 保持私有。baseline epoch 320 的再次取证显示，
`ibl_bake_artifact_asset_derived.rs` 以根 `io::{atomic_write, sync_parent_directory}` 导入，current hash 为
`0e5faaeafaab9cc8d14c1c5f9f9092aea4583d92c813956a0172863e0603e724`；
`ibl_bake_artifact_cache.rs` 继续直接导入 `io::atomic_write`，current hash 为
`09a8bae2c523c6e5c9cce8591a49d48a97b796228078517c7ea4762f08420b18`。前者 attribution 仍指向已 stale 的
`shader06-m8-bundle-transaction-20260815`，后者指向 active 但当前无 live lease 的
`shader06-m8-atomic-ibl-publication-20260815`；两者当前 matrix state 都是 `unowned`，R8 不借此冒领。

`ibl_source_cubemap_staging.rs` 已由共享 Shader06 工作改用 `io::transaction::{commit_prepared_files, ...}`，并有
source contract 明确拒绝 `atomic_write(`；current hash 为
`4b5f677ba92fcb7e90eaa78d7b1fa287f45cf93d37f310773c4456d7a71c47e7`，旧 attribution 仍指向 stale
`shader06-m8-bundle-transaction-20260815`，且报告 `attribution_hash_stale`/无 live lease。它不再是
`atomic_write` consumer，也不构成恢复旧公开模块的理由。R8 不 transfer、不改写三份 foreign blob；集成顺序仍固定为
先提交 Frameworks01 façade，再由 Shader06/UI12 基于该 SHA 与当前 Shader blobs 重建隔离快照。最新 focused
fingerprint 中只剩 cubemap staging 测试作用域缺少 `IblSourceCubemapStagingError` 名称解析，已明确归属 Shader06，
不是 resource I/O facade 错误。

共享 Cargo 窗口释放后，current-hash Windows managed production build 已实际进入 Cargo 并通过：job
`c7bdd679030543da9a74443f929d8dde` 执行 `cargo build -p zircon_runtime --no-default-features
--features core-min --locked --verbose`，在 D 盘共享 target 上以 6m27s 完成，0 errors；`zircon_runtime`
library 为 278 warnings，export-pack binary 为 8 warnings。该结果替代此前 0 errors / 293 warnings 的旧源码图。

随后 job `bfbe3eddc4154e8d8b3a36b9fa1adf4c` 首次执行 focused lib-test compile，得到 60 errors / 1,352
warnings / 0 tests；其中只有 2 条属于 R8 租约范围：ProjectAssetManager 测试缺少 `Result<(), _>` 约束，以及
ResourceManager stale-revision 测试在 move 后再次读取 `original.id`。两处 test-contract drift 已修复并通过 exact
rustfmt/diff-check。中间 job `c1a18fc837264adeb7cb54d03788a036` 暴露第一次同形文本修正命中错误位置，随后按
唯一上下文恢复非目标断言并修正真正 stale-revision 断言。最终 job `42d8bfd2f882484a990ca6dc04767de9`
在 5m21s 后得到 58 errors / 1,352 warnings / 0 tests，Cargo fingerprint 结构化复核为 R8-owned diagnostics 0。

剩余 58 条均为 current shared checkout 的外部 owner 编译门禁：Render06 `view_family.rs` 11 条、Render07
`post_process/pass_graph.rs` 1 条、Runtime08/Scene 46 条。最低归属已有活动或 resolving-failure primary：
`render06-view-family-contract-foundation-20260815`、`render07-postprocess-phase-routing-20260815` 与
`runtime08-world-query-current-source-recovery-r2-20260815`；Runtime08 的既有 open failure records 已覆盖 ECS
columnar storage、deferred command buffer、dynamic-scene spawn 等根因域，Render06/07 也已有各自 failure graph。
R8 不重复注册 primary、不物化同计划 local failure、不改写 58 条错误所在 foreign 文件。focused 测试尚未运行，
因此 accepted milestone、service commit、量化企微通知与独立复审仍保持 pending；生产实现本身不再受 Cargo
pre-start 缺陷阻塞。

等待外部门禁期间继续执行非验收状态机复审，发现 128 MiB journal bound 与 restart recovery 存在结构性死锁：
writer 侧 append 失败后已能幂等恢复 live 文件并 journal-first 清理，但 restart recovery 仍强制追加
`RollingBack`、`RollbackCompleted` 或 `Cleanup` frame；若 crash 发生在最后一个可容纳 frame 之后，journal 有效且
备份完好，恢复却会永久停在 bounded append。R8 已将 recovery 与 writer 收敛到同一策略：首个 state append 失败后
停止后续 append，完成全部幂等 restore，再先删除 journal、后清理 artifacts；RollbackCompleted/AllCommitted 的 phase
append 失败同样使用 journal-first cleanup，避免 artifact-first 破坏可重启证据。新增
`active_recovery_uses_journal_first_cleanup_when_transition_append_fails` 合同注入 bounded append failure，验证旧字节
恢复、journal/staging/backup/rollback-staging 全部清理且不再追加 phase。

该修复的 current-source managed production job `0204ff1180ff421ea1524cb9e788cc1f` 在 D 盘 target 以 5m55s
完成，0 errors、281 library warnings + 8 binary warnings。随后 focused lib-test compile job
`217c33355be94168a4c83f9aed6b5eff` 为 59 errors / 1,352 warnings / 0 tests；结构化 fingerprint 中 R8-owned
diagnostics 仍为 0，证明新增 recovery helper 与合同测试已通过 lib-test 编译。相较前一图新增的 1 条是
`core/runtime/tests/activation/behavior/activation.rs` 的 `Sender<()>` / `SyncSender<()>` drift，协调器 current hash
明确归属活动会话 `runtime-core-lifecycle-m0-veto-atomicity-20260815`。最新外部门禁因此为 Render06 11、Render07 1、
RuntimeCore 1、Runtime08/Scene 46；R8 未改写任一 foreign 路径。

继续复核 transaction 输入与 recovery immutable intent 后，发现原目标去重只做 slash 替换与 Windows ASCII
小写，`..`、目录 symlink/junction 和完整 Unicode 大小写 alias 可以进入同一批次并生成两套 staging/backup；后写
会覆盖前一 intent 的物理目录项，属于代际原子性的结构性错误。R8 对照 Unreal `FPaths::NormalizeFilename`、
relative-directory collapse 与本仓库 `ProjectPaths` 的 physical-ancestor/`CompareStringOrdinal` 规则后，将调用者路径
在 journal materialization 前收敛为物理 operation path；最终目录项若本身是 symlink 则保留，使既有 non-link
校验继续 fail closed。engine 与 recovery 共享 `PathIdentity`，Windows 使用预计算 UTF-16 key 和
`CompareStringOrdinal(ignore_case)`，其余平台按规范化 `PathBuf` 排序；`BTreeSet` 去重上界为 `O(W log W)`，不再
依赖平均复杂度或易碰撞的字符串折叠。recovery 还要求 journal 中每个 target/stage/backup/retired path 精确等于
重新解析后的物理 operation path，旧别名或被篡改 intent 不会被执行。

新增 engine 合同覆盖 lexical `nested/..` target、Windows missing-target Unicode case alias 和 Unix
directory-symlink alias；
三者都必须在创建 journal 前返回 duplicate error，保持 live bytes、report 与 journal owner 不变；另一个 recovery
合同直接注入 lexical alias intent，要求在 recovery I/O 前以 non-normalized physical path 拒绝，identity set、target 与
journal 均保持未物化。current-source Windows
managed production job `c555b198299445ffb2ecc2a0b3b38595` 于 01:01:37--01:06:27 执行约 290.6 秒，
`core-min` build 以 exit code 0 完成；target 为
`D:\cargo-targets\zircon-engine\pool\f9fef644bf8e441a49ad1c139495499657f126cd246ffca80d13868db535561d`，
release 后 live process 为空。后续 focused lib-test 已实际取得 Cargo 窗口，但仍被共享 Asset/Runtime/Render/Scene
编译门禁阻断；生产 GREEN 不提升为测试验收。

该正确性 hard cut 没有被包装成性能优化。当前 missing-target physical resolution 最坏需要按路径深度 `D` 探测
existing ancestor，因此整批静态上界为 `O(W * D + W log W)`，同 parent 多写会重复 metadata 查询；直接缓存 parent
identity 又会引入 symlink/junction retarget 的 TOCTOU 语义，不能凭静态猜测实施。R7-C 必须在 1/100/1k/10k write
batch 上增加 path-resolution CPU sample、File I/O metadata query count、wall p50/p95/p99 与 cold/warm 分组；只有
该阶段达到 material bottleneck，才评估 per-transaction verified-parent cache，并复测 alias rejection 与 race controls。

WAL barrier 的静态盘点暴露了更高优先级的结构性性能假设：每个 document 当前分别追加并同步 `Prepared`、
`Committing`、`Committed`，因此仅 journal 就是约 `3W + O(1)` 次 `sync_all`，另有 staging、backup、live replace、
artifact cleanup 的 `O(W)` file/directory barriers。Unreal `FDiskJournal` 把内存 `Entries` 一次写出，并用
`IasOpCount` 显式核算 write/flush/commit；`FCacheJournalSectioned` 只序列化 dirty pages，全部 page 写完后统一一次
`Flush(true)`，两者均有 CPU trace scope。它们不是 generation transaction 的可复制实现，但确认“journal durability
boundary 按批次而非按 entry”是应优先测量的参考方向。

候选 hard cut 是保留 fsync'd immutable intent；全部 staging/evidence frame 先 append，进入 `Active` 前统一一次
journal barrier；`Active` 的恢复语义改为每个 document 都允许 old-or-new digest 并全部逆序回滚，使 per-document
`Committing` 不再是发布前的 correctness barrier；全部 live replace 完成后只用一个 `AllCommitted` frame 作为代际
commit point，progress frame 只能是无权威的诊断/恢复加速信息。目标将 journal barrier 从 `O(W)` 收敛为 `O(1)`，
但 staging/backup/live namespace 的数据与目录 barrier 仍保持 `O(W)`。该候选必须先用 ETW/WPR 取得
FlushBuffers/File I/O count、journal sampled CPU 与 p50/p95/p99，并对现实现和 prototype 执行 every-transition crash、
torn-tail、old/new digest、retired file 与 power/RSS 复测；在结果证明 journal barrier 是 material bottleneck 前不实现。

本轮 focused validator 第一次在 acquire 阶段与 foreign job `fb40d0ac53364356b0b6cb0e543549fd` 竞态，返回
`cargo_reuse_pool_busy`；该 Runtime Interface `recorder_` job 随后运行至 01:34:25，rustc 自然退出但 descriptor 终态为
`orphaned`、live PID 为空、未写 `released_at/exit_code`。再次通过正常 validator acquire 时，Coordinator 返回
`unmanaged_artifacts_detected`，列出 foreign D/E/F fixture/target cleanup reservations；本 Session 未删除、release、
recover 或接管这些产物。两次尝试均未创建本 Session Cargo job，不能记测试结果；focused lib-test 继续等待
Coordinator 全局 artifact gate 由正确 owner 收敛。

Coordinator 后续恢复正常 acquire 后，current-source Windows managed focused lib-test job
`ae53fb57f30745a7b50b0ee6aee249e0` 于 01:50:10--01:56:51 实际进入 Cargo，descriptor 执行约 401.8 秒并以
exit code 1 结束，01:57:04 正常 release；target 仍为上述 D 盘 pool，测试二进制未生成、0 tests。rustc
fingerprint `zircon_runtime-2213debbe0169866` 记录 61 条 source diagnostics + 1 条终止诊断、1,352 warnings；R8
拥有的 `pathing.rs`、`engine.rs`、`engine/tests.rs`、`recovery.rs`、`recovery/tests.rs` 命中 0 error。61 条外部
门禁按 owner family 为 Render06 11、Render07 1、RuntimeCore 2、Runtime08/Scene 46、Asset/IBL 1；相较 job
`217c33355be94168a4c83f9aed6b5eff` 只新增 `core/runtime/handle/activation/batch.rs` 的 `Arc<[RegistryName]>`
迭代错误和 `asset/artifact/ibl_source_cubemap_staging.rs` 测试作用域的
`IblSourceCubemapStagingError` 名称解析错误。三份 Shader06 IBL consumer 的 `io::atomic_write` 导入没有出现在
本轮错误指纹中，证明共享 current source 的 facade 路由正确；UI12 isolated patch 仍必须等待该 facade 的 service
commit 后重建，不能把工作树可见性冒充已集成依赖。

2026-08-16 UI12 转交的旧 runtime 指纹仍把三份 IBL consumer 记录为 unresolved `io::atomic_write`。current-source
重审指纹为 `io/mod.rs=118cad19d58c0cbd7bee23e79b4b02dcb5b12126bf983a13595cf397a6c0202c`、
`asset-derived=7753c37d22d7852bfc727479191fa6d6f37cf9758b0e0e8b22901e629d8b5132`、
`cache=09a8bae2c523c6e5c9cce8591a49d48a97b796228078517c7ea4762f08420b18`、
`source-staging=6604af834715d74c284c92177ce6416db45e6114c325f3b59d08ad22e4c83a82`：Resource IO façade
明确 `pub use atomic_file::atomic_write`，asset-derived/cache 继续消费这条 curated publication，source-staging
则已改为 durable bundle transaction 且不再导入 `atomic_write`。结论是保留公开 façade 导出，不做 Shader06
consumer 回退迁移；UI12 必须在 façade service commit 后以 current source 重编译，旧快照 E0432 不计当前阻断。

R8 文档修复后最新全库 docs convention gate 为 RED：671 条既有 path violation（669 missing、2 absolute）、
影响 241 份文档、检查 75,212 个结构化路径和 2,507 份文档；本子计划命中 0 条。全 `docs` 中旧
`atomic_file.rs` 路径仍只属于 foreign 历史 failure 记录，不改写历史归属。

下一轮物理 `zr_resource` hard cut 的 2026-08-15 前置清单保存在
`D:\zircon-frameworks01-r7-zr-resource-consumer-inventory.json`，其 SHA-256 为
`212d1f5456ba4352f4de305675b4a14938ca26d65918d8cf9e0dd391276d7917`。baseline epoch 321 下又执行两次
独立只读重扫，均得到 current inventory fingerprint
`3cd3b1c9940561c606f326fae346e7ca325eecd848e0c451aec6c499d450e0a2`：17,180 个 tracked + nonignored
untracked Rust 候选中，61 个 foreign tracked-deleted 路径均不是旧 consumer；连续文本 matcher 命中 463 个
显式 consumer，253 个含生产引用、210 个仅含测试引用，production/test hit 为 287/286。相较旧清单新增 Editor16
`hub_link/{focus_signal,handshake,recent_writeback}.rs` 三个生产 consumer，93 个既有 consumer blob 已漂移；
`focus_signal.rs` attribution 指向 active Editor16 session 但无 live lease，其余两份 attribution missing，必须经
coordinator transfer 后再进入原子迁移。`editor_event_runtime_access.rs` 当前是 foreign tracked deletion，mixed-blob
freeze 仍有效，不能因本次不存在而被永久漏出最终 scope。

literal matcher 不能展开 `use crate::core::{resource::{...}}`，所以它只作为路径 guard 而不是完整依赖图。复用仓库
`runtime_domain_dependency_audit` 的注释/字符串屏蔽与嵌套 use-tree lexer 后，确认 444 个真实 use consumer，新增
7 个 structured-only owner：TextureImporter 1、Graphics 2、Scene 4，共 19 个 use leaf。两集合 union 为 470 文件，
production 259、test-only 211。tracked 与 731 个 nonignored untracked Rust 文件中的 crate alias、root glob、
`core::*` glob 均为 0；Runtime lib 根唯一 `extern crate self as zircon_runtime` 已由标准根路径覆盖。最终 scope 必须
以 470-file union 为准。

结构化 use graph 共 83 个 Resource leaf path。高频产品面是稳定 DTO/handle/record/locator、ResourceManager/
registry/snapshot/generation 与唯一 `io::atomic_write`；`ResourceRegistryStaging`、`ResourceReadinessRow`、
`approximate_event_bytes`、fault/stage/sync helpers、durable transaction helpers 和跨 crate 返回类型
`PreparedResourceMutation` 全部进入 `zr_resource::assembly`，禁止从 Runtime 产品 façade re-export。

同一 current graph 的 `core/resource` 为 57 个 Rust 文件、11,480 行，tree fingerprint
`b06500a6f558b36880d5f051d566dddb054f2bf7bc23b370abdd06f4c16b9538`；interface resource 为 14 文件、923 行，
tree fingerprint `f22927513772a6e664676d32a6be7872067cfef05ab97288c41c049d1322c7ad`。production 对其他 Runtime
domain 的直接 `crate::` 反向引用为 0，Asset/Diagnostics/Framework 命中全部来自
`management_generation/tests/hard_cut.rs` 的 mutation 字符串；读取 Runtime 源树的 architecture guard 必须迁往
Runtime integration/absorption owner，内部行为测试才迁入 `zr_resource`。R8 immutable scope 不包含根/Runtime/
interface manifests、`Cargo.lock`、新 crate 路径和全部 470 consumers，因此本 slice 不物化空 crate、不复制实现、
不保留 forwarding module；下一 owner 必须先完成 coordinator scope rotation，再在同一 current fingerprint 原子硬切。

R5 实现量化：`scan_and_import.rs` 从 811 行收敛为 332 行门面；新增
`projected_inventory.rs` 115 行、`full_generation.rs` 556 行，生产 owner 均低于 800 行预算。
full generation 删除 import loop 内的 sidecar save、直接 artifact manifest write 和落盘后 registry 重扫；
changed sidecar、artifact manifest、asset registry 统一进入一个 rollback-capable file commit。open、watch、
reconciliation 与 `reimport_all` 均在 Resource typed reservation 后执行 file commit，再安装 project/source-path
state，最后 Resource apply/event。全部 inventory sidecar 在提交时按 stripe 排序去重加锁，并在锁内验证
prepare 时原文档；changed 文档才生成写入，未改写但参与 registry 投影的文档同样受快照校验，拒绝并发
preview/editor 变更造成 registry/sidecar 代次分叉。

R6 实现量化：旧 AssetMigration `commit/journal/schema/stage` 四份重复 owner 与 ProjectManager
`targeted_transaction.rs` 已删除；Core 事务 façade/engine/commit/journal/observation/owner-lock/recovery/schema/stage
当前分别为 23/373/353/625/143/123/806/344/249 行；recovery 超过 800 行 review 线但低于约 1000 行强制拆分阈值，
其余 production owner 均低于 800 行。journal v4 使用
`length + BLAKE3 + TOML payload` frame：immutable intent 是首 frame，transition 只追加单 frame；尾部不完整
frame 不发布状态，非尾部损坏 fail closed。128 MiB journal 总量在 append 与恢复读取前同时检查，64 MiB 单帧
在分配前检查；intent 原子写崩溃
遗留的严格保留命名 sibling 在全目录验证通过后清理。固定 sibling `.zrlock` 使用 OS 文件锁串行化同 owner 的
commit/detect/recover，锁内发现待恢复条目即拒绝新事务。replace 已进入 live namespace 后再发生 durability
错误时按“可能已发布”处理并强制逆序恢复，不能再因晚置 `committed` 标志漏回滚。

## Architecture Decision

### Private build unit and writer authority

`zr_resource` 只允许成为 `zircon_runtime/crates/` 下 `publish = false` 的私有编译单元；产品与跨包
canonical surface 仍是 `zircon_runtime::core::resource`。这不是恢复已删除的顶层 `zircon_resource`，
App/Editor/plugins 也不得直接依赖 `zr_resource` 或其 hidden assembly surface。

本轮完整复审确认 Core durable transaction 的 `.zrlock` 只串行化同一个 journal owner；IBL 的 standalone
derived writer 与 source/derived bundle transaction 已刻意复用相同 lock path，而 project-generation 与
asset-migration 使用不同 journal owners。Core 因而不宣称对任意目标路径提供全局 writer authority，也不在
通用 Resource IO 中新增带项目语义的 process-global/per-path lock。Unreal `UPackage::GetAsyncSavePipe` 将
save 生命周期置于上层统一 pipeline，`FinalizeTempOutputFiles` 再以 temp/backup/restore/finalize 发布文件；
该参考支持“上层单写者 authority + 下层 durable primitive”，而不是让底层文件函数推断 package/project owner。

Zircon 的 project writer 排他继续由项目 admission owner 收敛，复用
`docs/plans/zircon_editor/editor/16/failure-2026-07-23-project-session-lock-reuse-for-recovery.md` 的唯一
`SessionGuard` 合同；Editor/Hub/commandlet 不得创建第二把项目锁。该 Editor16 failure 未关闭前，
Frameworks01 只保证同 journal owner 的 commit/detect/recover 串行和崩溃恢复，不把跨工具并发写包装成已解决，
也不以 speculative IO lock 代替上层 write-authority 修复。

full scan 必须硬切为一个 `PreparedFullProjectGeneration`，由同一份确定顺序的
`ProjectedMetaInventory` 生成 artifact manifest、`.zmeta`、asset registry、ProjectManager
候选状态和 Resource mutation batch。准备阶段不得改写任何权威文件或 live state。

唯一成功顺序固定为：

1. 一次发现并解析 source/meta inventory，完成 URI、kind、compound membership 和 duplicate GUID
   归一化；
2. 从该投影完成 import、依赖解析、asset registry 和 catalog generation 构建；
3. 准备全部 immutable chunks、artifact manifests、sidecars 和 registry bytes；
4. `ResourceManager::prepare_commit` 取得 typed reservation，失败时磁盘和项目状态保持不变；
5. durable owner lock 内先发布 immutable intent，完成 staging/backup/digest，再追加 `active`；
6. 按确定顺序替换全部 live 文件并追加 per-document transition；`all_committed` 之前退出一律恢复旧代；
7. 追加 `all_committed` 后只允许保留新代并完成 cleanup，随后释放 journal owner lock；
8. 安装 source-path index、ProjectManager snapshot 和 watcher transition，执行无失败 Resource apply，最后发布
   Resource event 和 catalog/resource generation；若在文件终态后、内存安装前退出，下一次 open 从已提交磁盘代
   重建内存 authority。

旧的 mid-scan `AssetMetaDocument::save`、full-scan `ArtifactStore::write`、副作用型
`prepare_duplicate_guids`、完成 import 后再从磁盘 `rebuild_after_import`，以及 full scan 在 Resource
reservation 前自行提交文件的路径全部删除，不保留兼容 wrapper、双轨 API 或静默补偿分支。

## Pre-R5 Failure Model

R5 硬切前的 full-scan 流程按当时实际代码顺序执行：

1. `collect_import_sources` 先递归查找并解析所有 `.zmeta` 以识别 compound source，再次递归收集源文件；
2. `prepare_reference_resolution_metadata` 对每份已有 sidecar 先 `load`，再由
   `load_or_create_meta` 第二次 `load`，并立即保存 URI/kind 归一化结果；
3. `prepare_duplicate_guids` 再次递归扫描并解析全部 sidecar，且在所谓 preflight 内立即保存 remint；
4. 主 import loop 再次加载 sidecar；成功、恢复和失败路径都可能在循环内保存 sidecar，成功路径还立即
   发布 artifact manifest；
5. `rebuild_after_import` 再次递归扫描并解析 sidecar，随后才持久化 asset registry 和安装内存状态；
6. 上层 runtime 之后才可能执行 Resource typed preflight。

因此，具有现有 sidecar 的普通项目在一次 full scan 中最多会发生约 `6N` 次 metadata
反序列化：compound detection `N`、reference preflight `2N`、duplicate pass `N`、import `N`、
registry rebuild `N`。项目树至少经历 compound-meta、source-file、duplicate-meta 和 rebuild-meta 四次
递归遍历；package resolver 的 `inspect_project` 还是独立的额外 inventory。这里的计数是静态调用图
上界，不是运行时性能样本。

当前存在五类确定性故障窗口：

- sidecar URI/kind 已保存，duplicate/collision/import 随后失败；
- duplicate GUID 已 remint，后续 source import 或 registry build 失败；
- 某些 artifact manifest/sidecar 已发布，后续 source 失败；
- 全部 source 文件已改写，asset registry 持久化失败；
- ProjectManager 文件已提交，随后 Resource reservation 因 locator/kind/revision 冲突失败。

`targeted_transaction::commit_prepared_files` 已能拒绝重复目标、预存原内容、一次 staging 并在进程内
逆序 rollback；它不是 durable journal，进程在文件替换或 rollback 中退出时无法恢复。Artifact
`prepare_write` 会先发布内容寻址的 immutable chunks，再返回小型 manifest bytes；未被 manifest 引用的
完整 chunks 可由后续 GC 回收，不能把它们视为已发布 generation。

## Reference-Engine Decisions

### Unreal primary reference

`FAssetRegistryState` 由一个 state owner 维护 ObjectPath 与 package/path/class/tag/dependency accelerators；
`AddAssetData`、`UpdateAssetData` 和 remove family 在同一 owner 更新相关索引。`InitializeFromExisting`
与 `ConsumeExisting` 从已拥有的 state 构建或移动状态，而不是要求调用方先把中间 sidecar 写到磁盘再重扫。
`Serialize` 明确路由到 `Save`/`Load`；`AssetRegistryGenerator` 先从内存 `DevelopmentState` 序列化到
`FBufferArchive64`，随后才 `SaveArrayToFile`。Zircon 采用的是“权威内存投影与序列化分离”原则，
不移植 Unreal allocator、package 格式或 cooker 线程模型。

### Bevy corroborating reference

Bevy `AssetInfos` 在一个 owner 内同步 `path_to_index` 与 `infos`。AssetProcessor 对 asset bytes 和 meta
共用一把 per-path transaction lock，避免读到新 bytes 配旧 meta；写入前 flush `BeginProcessing`，成功后
flush `EndProcessing`，重启时把 unfinished transaction 视为必须重处理。Zircon 不能只复制这份较粗的
per-asset WAL，因为 full generation 同时拥有多份 sidecar、artifact manifests、registry 和 Resource event；
可采用的是 write-ahead、读写隔离和启动恢复先于正常加载三个原则。

R7 观测接口另参考 Bevy `DiagnosticPath::const_new` 与 `Diagnostics::add_measurement`：路径由 typed/static owner
定义，measurement closure 只在 diagnostic enabled 时求值；render asset diagnostic 通过原有 atomic asset count
发布，不为诊断重走 asset collection。Zircon 因此复用现有 profiler，不新建第二套全局 diagnostic store。

Unreal `AssetDataGatherer.cpp` 在 `Tick`、`ReadAssetFile`、directory read/wait、cache update/save/build shards
等真实 owner boundary 放置 scope。Zircon 对应地把 phase 放在 discovery、metadata/import/dependency/registry
projection、serialize、resource projection/reservation、file commit、project install、resource apply、generation
publish 和 recovery 边界，而不是在每条记录或每个 chunk 上制造高基数 span。

### Fyrox and Godot boundary references

Fyrox `ResourceRegistryRefMut` 修改 metadata 后在 `Drop` 自动保存 registry，错误只记录日志；它不能提供
多文件崩溃原子性，作为 Zircon 的反例边界而非目标。Godot `ResourceUID` 在 mutex 下同步 UID/path 与可选
reverse cache，并将 cache load/save/update 与内存 mutation 分开；其 append cache 同样不是 Zircon
full-generation journal。两者支持“一个身份 owner、持久化独立”的方向，但不降低 durable gate。

## Required Owner Graph

### Projected inventory

新增 folder-backed `scan_and_import/projected_inventory.rs`，拥有：

- 按 normalized meta path 排序的 `BTreeMap<PathBuf, ProjectedMetaDocument>`；
- original/current document、source URI、source unit、included paths 和 dirty reason；
- pure duplicate GUID normalization 及 rename changes；
- 从 borrowed documents 构建 resolver/asset registry 的唯一入口；
- changed sidecar 的确定顺序序列化，不执行保存。

`asset/registry/rebuild.rs` 的 duplicate normalization 已增加 caller-owned document 纯投影入口；full scan
不再调用会保存 sidecar 的磁盘 wrapper。`inspect_loaded_meta_document_refs` 成为 full generation 的正式
consumer；既有 `inspect_loaded_meta_documents` 保留为当前外部只读 inventory consumer，不是旧 full-scan
兼容路径；`inspect_project` 只保留外部只读/启动重建用途。

### Full generation

新增 folder-backed `scan_and_import/full_generation.rs`，拥有
`PreparedFullProjectGeneration`：candidate Resource staging、asset registry、shader dependency index、catalog
generation、ready payloads、全部 `PreparedFileWrite` 和导入结果。它只允许 `prepare`、只读观察和一次性
`commit`；drop 未提交对象不改变 live authority。

`scan_and_import.rs` 降为薄编排/facade，保持生产文件低于 800 行 review warning；新 owner 也必须低于
800 行。不得用等行数切分或保留旧行为 wrapper。

### Durable file transaction

现有 `asset/migration/transaction` 的 immutable intent、append-only transitions、per-document
`prepared/committing/committed/rolling_back`、digest、backup、recovery 和 fault injection 已比 targeted
rollback 完整。后续不是复制第二套协议，而是把与 TOML/migration policy 无关的引擎下沉到
`core::resource::io::transaction`，物理 crate 化时直接迁入 `zr_resource::io`；AssetMigration 通过 policy
adapter 继续使用同一实现。

journal 的代际裁决已固定为：`all_committed` 之前重启一律按 digest 证据恢复旧代；`all_committed` 之后只
完成 cleanup，并从磁盘重建 Project/Resource 内存 authority。正常项目打开先恢复 journal，再加载 package/
registry/sidecar，禁止 loader 观察半代文件。Project policy 只接受精确 asset registry、合法 ResourceId
`.zasset` manifest 与 asset-root 内 `.zmeta`，拒绝 authoring source、chunk 和 staging namespace；Migration
policy 只接受 scanner 本轮给出的 target，并校验 project root 与 `.meta.toml -> .zmeta` 配对。

## Complexity Target

设源文件数为 `F`、sidecar 数为 `N`、sidecar entry 总数为 `A`、本代变更文件数为 `W`、artifact bytes
为 `B`。

| 阶段 | 当前静态上界 | 目标 |
|---|---:|---:|
| 项目目录递归遍历 | 至少 4 次，另有 package inventory | 1 次项目 discovery；package inventory 各 root 1 次 |
| project sidecar 反序列化 | 最多约 `6N` | `N`，每份文档单 owner |
| duplicate/index/dependency build | 多次 `O(A)`，中间伴随写盘 | 各一次 `O(N + A)` pure projection |
| sidecar/manifest/registry publish | source loop 内分散发布 | prepare `O(W + B)`，一次 generation commit |
| durable journal barriers | 约 `3W + O(1)` 次同步 | profile 证明后收敛为 `O(1)` correctness commit points |
| duplicate lookup | `HashMap` 平均 `O(1)` | 保持平均 `O(1)`；不引入全表二次查找 |
| 确定顺序 | 多轮文件系统枚举后局部排序 | inventory/path/write order 一次规范化 |

第一实现切片允许复用当前 `collect_import_sources`，把后续重复解析从约 `5N` 收敛为一个投影 owner；
这属于正确性重构的附带收益，不宣称优化完成。将 compound detection 与 source collection 合并为单遍
discovery 属于性能切片，必须由测量证明目录枚举/metadata parse 是 material bottleneck 后再实施。

## Measurement Before Optimization

先在现有 frame/profile contract 之外增加 asset-generation typed observation，不改变调度算法。记录
source/meta/path 数、metadata deserialize count/bytes、artifact chunk/manifest bytes、prepared/committed write
数、live rollback restore attempt/success、recovery rollback/cleanup/orphan-cleanup 数，以及 discovery、parse、import、
artifact preparation、registry projection、file stage、file commit、project install、Resource apply 的 elapsed time。
disabled 状态不得增加文件读取、分配或锁。

Windows 受管验证使用相同 source fingerprint、磁盘、杀毒配置、AC 状态和 target pool，覆盖 `1/100/1k/10k`
source、`0/1/10/100%` changed ratio、single/compound 和 duplicate/collision/failure controls：

1. 使用 WPR/ETW CPU sampling + File I/O 记录 baseline 与 hard-cut 版本；WPA 核对递归枚举、metadata parse、
   hash/compress、fsync/rename 和 rollback 的实际占比；
2. 每组丢弃 warm-up，保存至少 31 个 settled warm samples；cold filesystem 只在可复现清缓存条件下单独报告，
   不与 warm 数据混合；
3. 报告 median、p95、MAD、peak working set、read/write count/bytes、disk active time 和 CPU sampled time；
4. 有可校准 package/board telemetry 时报告 energy/job 与平均功率；工具不可用或返回 0 W 时明确记
   `unavailable`，禁止用耗时代理功耗；
5. 只有 profile 证明瓶颈后，才允许单遍 discovery、parallel import、index layout 或 batching 优化。

性能接受要求不是“更快一次”：输出 registry/resource records 与 diagnostics 完全一致，故障/恢复合同保持，
p95 和 I/O 指标超出 MAD 噪声包络地改善，RSS、磁盘放大和能耗不退化。没有同机参考工作负载时，不声称
与 Unreal/Bevy/Fyrox/Godot 经验值接近，也不声称算法已达到最优规模。

## Ordered Implementation Milestones

- [x] R4-A：复核 current-source、现有 migration journal 和四个本地参考引擎，锁定 owner/state machine。
- [x] R4-B：记录静态 I/O/复杂度上界、故障窗口和 profiler/功耗验证矩阵。
- [x] R5-A：scope rotation 后实现 pure projected inventory、pure duplicate normalization 和 borrowed registry
  build；删除 mid-scan sidecar saves 和 full-scan disk rebuild。
- [x] R5-B：实现 `PreparedFullProjectGeneration`，全部 artifact/meta/registry writes 进入一次进程内 file
  transaction，并在 runtime 中接入 Resource reservation/event-last 顺序。
- [x] R6-A：抽取通用 durable transaction/journal/recovery，删除两套旧 file transaction owner，
  AssetMigration 与 targeted/full project generation 使用同一 Core engine。
- [x] R6-B：实现 frame WAL、digest/evidence 验证、跨进程 owner lock、restart 代次裁决、Project/Migration
  target policy、post-replace durability rollback 和 intent orphan cleanup。
- [ ] R6-C：执行 every-transition crash/fault/restart matrix、focused/upward tests 与独立复审；测试已经写入，
  新增 5 条 commit-point/cleanup/rollback-transition fault contract、1 条 active committed old-bytes restart
  contract 与 staging-directory barrier、torn-tail repair frame、journal-first abort/Intent recovery cleanup、
  rollback-completed consumed-staging restart、bounded append contract 均 exact rustfmt GREEN。Tooling fix
  `543497826` 已解除 pre-start 缺陷，current-hash focused lib-test compile 已连续实际进入 Cargo；R8-owned 2 条
  test-contract drift 修复后，新增 bounded recovery append fallback 与合同也已进入 lib-test 图。最新 job
  `ae53fb57f30745a7b50b0ee6aee249e0` 为 61 foreign errors / 1,352 warnings / 0 tests，R8-owned diagnostics 0；
  Asset/IBL、Render06/07、RuntimeCore 与 Runtime08/Scene owner 门禁收敛前 focused tests 和独立复审仍保持
  validation pending。
- [~] R6-D：atomic persistence 963 行 owner 的 folder-backed 拆分、`io::atomic_write` curated façade、31 个
  current-source consumer/35 处旧路径迁移和 `atomic_file` 模块私有化的非验收实现已完成；exact
  rustfmt/diff-check GREEN，旧 Rust 模块路径为 0。最新 `core-min` production managed build job
  `0204ff1180ff421ea1524cb9e788cc1f` 为 0 errors、281 library warnings + 8 binary warnings、5m55s；后续状态机
  复核又完成 typed commit-point outcome、deferred cleanup、rollback transition/torn-tail fail-safe 与 bounded recovery
  append fallback 修复，并将 typed outcome 从 Core 贯穿至 Project/Resource/generation publish 入口，避免
  profiler-only success 吞掉未决耐久性；physical-path identity hard cut 后的 production job
  `c555b198299445ffb2ecc2a0b3b38595` 同样 0 errors。最新 focused compile job `ae53fb57f30745a7b50b0ee6aee249e0`
  的 R8-owned diagnostics 为 0，但 61 条 foreign errors 使测试数仍为 0。最大 production owner 为 806 行 recovery validation/replay，
  略过 800 行 review 线但低于约 1000 行强制拆分阈值，继续观察而不做等行数切分；新增 alias contracts、最终
  current-hash managed lib-test 与独立复审尚未完成，因此保持 validation pending。
- [x] R7-A：复核 Zircon profiler、Project/Resource/transaction current call graph，以及 Unreal AssetDataGatherer
  phase scopes 与 Bevy deferred diagnostics；锁定单一 observer owner 和零额外 I/O 约束。
- [~] R7-B：实现 project-generation phase/counter 与 durable-transaction activity，拆出 resource publication
  owner；default 与 profiling-feature managed production build 的 R7 owned errors 均为 0，test target 仍受外部
  lib errors 阻断；build 后又将 Core->profiler 反向边硬切为 neutral recovery/commit report + Project adapter，
  live rollback restore attempt/success、deferred commit recovery 与 deferred cleanup 已由中立报告实现；
  post-correction profiling managed build 的 R7 owned errors 为 0，但共享 lib 的外部错误与最新 validator
  control-plane/shared-target acquire 阻塞仍阻止 current-hash test target，不能记 GREEN。
- [ ] R7-C：取得相同 fingerprint 的 Windows WPR/ETW/RSS/I/O/可用功耗 baseline，按规模与 changed ratio 输出
  median/p95/MAD；当前没有样本。
- [ ] R7-D：只实现 profile 证明的单遍 discovery、parallel import、index layout 或 batching 优化，并复测行为、
  耗时、I/O、RSS 和可用功耗；当前没有开始算法优化。
- [ ] Acceptance：focused/upward/product tests、独立复审、managed validation ticket、计划状态、coordinator
  milestone commit 和 service-managed WeCom 全部完成后，才提升 Frameworks01 M1。

## Scope And Coordination

R6 由 session `frameworks01-m1-durable-file-transaction-successor-r6-20260815` 在独立 scope rotation 下接管
AssetMigration transaction、Core resource I/O transaction、ProjectManager durable adapter 和对应测试；不吸收
三份既有跨计划 failure，也不改写 Editor01 的 `editor_event_runtime_access.rs` 或 Editor09 的
`asset_workspace_state.rs` mixed blobs。任何 ownership transfer 继续以 coordinator current hash 为准，不以
工作树可见性冒领；未取得测试 ticket 前不申请 milestone commit 或 service-managed WeCom。
R8 由 session `frameworks01-m1-durable-file-transaction-hard-cut-r8-20260815` 接管最终 façade hard cut；R7 仅因
补齐两个漏扫 consumer 做 scope migration 而取消，不代表里程碑完成。两份 Editor mixed blob 继续冻结且未改写。
