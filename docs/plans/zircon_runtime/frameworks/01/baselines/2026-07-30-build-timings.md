# Frameworks01 M0 Windows Build Timings Baseline

Status: `measurement-in-progress`。结构与依赖剖析已经完成；四个 HTML 报告及数值仍必须来自受管 Cargo 自然终态，未运行前不填写推测值，也不把当前状态提升为 M0 accepted。

## 环境

- CPU: AMD Ryzen 7 5800H，8 cores / 16 logical processors，MaxClockSpeed 3201 MHz
- Memory: 39.86 GiB visible
- OS: Windows 11 Pro 10.0.26200 build 26200
- Rust: `rustc 1.94.1 (e408947bf 2026-03-25)`，host `x86_64-pc-windows-msvc`，LLVM 21.1.8
- Cargo: `cargo 1.94.1 (29ea6fb6a 2026-03-24)`
- 测量前可用空间：D 92.43 GiB、E 90.47 GiB、F 83.18 GiB
- 不可变源码：ZirconEngine `5671b4e3f31f337afc2adb8280b6b87b67c3daf5`；外部 `zr_vm` 固定为 `4437186c1d4aa65b7d0364c8bcfc56d83e76e099`
- 构建位置：源码副本与 Cargo target 均位于 `D:\cargo-targets\verify\<job-id>`；报告写入本文件同级目录（仓库位于 E 盘），不在 C 盘生成项目产物

## 方法

1. Workspace 使用一个全新 coordinator compatibility pool，先运行 `cargo +1.94.1 build --workspace --locked --timings --jobs 1 --color never` 取得 cold 报告，再在同一 pool、源输入未漂移时运行相同命令取得 incremental 报告。
2. Runtime 使用另一个全新 pool，先运行 `cargo +1.94.1 build -p zircon_runtime --locked --timings --jobs 1 --color never`，再在同一 pool、源输入未漂移时重复一次。
3. 每个作业记录完整 compile-input pre/post fingerprint、job/run id、exit code、Cargo Finished duration 与 timing HTML SHA-256。任何源漂移、非零终态、进程树残留或复用已有 target 都使该轮失效。
4. 不执行 `cargo clean`，不删除共享 target，不使用 repo-local target；通过不同 compatibility key 获得两个 M0 专属空 pool。

## 结构剖析与优化判断（2026-08-14）

| 指标 | 当前值 | 含义 |
|---|---:|---|
| root workspace members | 36 | workspace 已有多包形态，但 Runtime 主体仍是巨型编译单元 |
| `zircon_runtime` direct / normal / optional dependencies | 62 / 59 / 28 | 重型依赖仍集中进入 Runtime manifest |
| `zircon_runtime` features | 42 | feature 数量未转化为足够的物理编译边界 |
| Runtime 内部 `zr_*` crates | 2 | 当前只有 `zr_rhi`、`zr_rhi_wgpu` 完成物理拆分 |
| immutable HEAD Runtime Rust files / lines | 7,254 / 963,241 | 当前单 crate 的源码规模 |
| stable current-source Runtime Rust files / lines | 7,433 / 1,110,882 | 共享工作树 2026-08-14 原子只读快照；不作为 immutable timing 输入 |
| current worktree production refs / domain edges | 2,710 / 72 | 共享工作树动态审计值，不作为 immutable timing 输入 |
| 最大 `graphics -> core` 引用 | 896 | `core` 仍是跨域编译扇入中心 |
| `zr_math` 原子 hard-cut 消费者 / 外来脏消费者 | 1,043 / 383 | 当前不能在不吸收外来修改的前提下原子迁移 |
| `zr_resource` Runtime 旧路径消费者 / 外来脏消费者 | 370 / 165 | 当前不能保留旧 owner 或双轨 facade 来规避冲突 |

本轮结论是先优化编译拓扑与 owner 边界，而不是对未被 profiler 证明为热点的数学或资源局部循环做微优化。作为本地参考树的规模校验，Unreal Engine 有 744 个 `*.Build.cs` 模块，Bevy `crates/` 下有 73 个 Cargo manifest，Fyrox 源树有 24 个 Cargo manifest；Zircon Runtime 目前只有 2 个物理内部 crate。该数量对比不直接代表性能，但足以否定“继续把所有实现留在单 Runtime 编译单元”的结构方向。

### Current-source 依赖图探针

- 2026-08-14 使用仓库审计器 `tools/runtime_domain_dependency_audit.py` 对 `zircon_runtime/src/**/*.rs` 做内存审计；审计前后均为 7,433 个 Rust 文件、1,110,882 行，输入清单 SHA-256 均为 `5a2116bbb0cbbd6bebc8884252afaca817107684e99294c0bd91fbcd072861f3`，因此该轮没有被并发源码修改污染。
- 结果为 2,710 条 production cross-domain reference、72 条 domain edge；canonical in-memory JSON SHA-256 为 `ba62a277569e9f2e8f327f7a45f7026187b0b2e024b8438580c598bc7846c94f`。最大边仍为 `graphics -> core` 896、`asset -> core` 292、`scene -> core` 228、`plugin -> core` 211，证明 `core` 扇入是结构主问题。
- 已完成 hard cut 的 `graphics -> scene`、`graphics -> ui`、`asset -> text`、`scene -> animation`、`rhi -> rhi_wgpu` 在本轮均为 0，未发现旧边回流。
- 现有 `2026-07-30-runtime-domain-dependencies-production-only.json` 是 2,391 refs / 76 edges 的历史快照，文件 SHA-256 为 `cc3c01dce8aa4a5c200560984c056b30e9ee1b777bc0f37f7ba531b45af6deba`。本轮不覆盖该受管产物；须取得精确写所有权后，将上述 stable current-source 结果完整落盘并重新校验，才可把 M0 依赖图标为完成。

## Profiler 与功耗边界

- `cargo --timings` 是本里程碑的 crate/build-unit profiler；cold 与 incremental 必须在同一空池连续执行，并保存独立 HTML 与 SHA-256。
- Windows `Processor Information` 可提供 `% Processor Utility`、频率和性能限制计数，后续结构硬切前后可作为同机负载对比。
- 本机虽然注册了 `Power Meter(*)\Power`，但 3 次连续采样的所有实例均返回 `0 W`，不能据此给出瓦特或能耗结论。除非获得有效的硬件能量计数器或外部功率计，不声明“功耗已接近其他引擎”。
- M3 收官仍按计划以单域增量时间改善不少于 50%、cold 构建劣化不超过 10% 为结构验收目标；本 M0 只建立可复现基线。

### Resource management 算法重审与优化门

- 当前 `ResourceManagementGeneration` 使用 64 个 immutable shard；单 id 查询是单 shard hash lookup，locator 查询最坏为 `64 * log2(N/64)`，变更发布只复制并重排受影响 shard。该 copy-on-write owner 适合跨 Runtime/Editor 共享稳定 generation，不应在没有样本时推翻。
- `ResourceManagementScan::next_row` 是固定 64 路线性选最小项：每个匹配行执行 64 次 shard candidate check，过滤行只沿各 shard 前进一次，复杂度为 `O(N_filtered + 64M)`。`page(offset, limit)` 每次从 0 建游标并重扫，单次为 `O(N_filtered_prefix + 64(offset + limit))`；按连续页重复深分页时，candidate check 随页数二次累计。这是需要 profiler 验证的结构候选，不等于已证明的产品瓶颈。
- current-source 的主要产品调用是 asset manager 按 kind 扫描、scene reload reconciliation 的有界增量扫描，以及 Editor 对 catalog item 做 locator lookup；仓库尚无 resource management benchmark 或已保存的 profiler 样本。`profiling` feature 下已有 query-local scan/page work metrics，可直接验证 candidate checks、filtered rows 与 emitted/returned rows；它们尚未接入高层 recorder，因此报告中的同名 counter coverage 仍应为 not-emitted，不能把静态复杂度或局部 metrics 测试当成耗时结论。
- 2026-08-14 layer audit 曾发现 resource 直接调用 Runtime diagnostics recorder 后，`core/resource` 从 0 条外部 Runtime 依赖退化为唯一的 2 条 `core::resource -> core::diagnostics` 引用，违反固定的 `zr_resource -> zr_contracts -> zr_kernel -> zr_diagnostics` DAG。该实现已改为 query-local metrics；direct/alias/glob guard 同批加入，并继续覆盖相对 `super`、crate-self `zircon_runtime` 与 `use crate as ...` 绕过形式。全局 recorder 删除后遗留的 profiling scan clone 计数清零也已修复，clone 现在同时保留游标、yielded 状态和累计 query-local metrics，并由同态测试封印。layer guard 最终并入既有 `hard_cut.rs` 后，current exact5 行数为 production owner 430、support 790、hard-cut 566、projection 130、root mod 3；canonical manifest 的 UTF-8 输入格式为按 path 排序的 `path<TAB>bytes<TAB>lines<TAB>file-sha256<LF>`，SHA-256 为 `44127ff6379d227f48de27b2fdf06fcf5db10220ffd37fcab4ef0b07d460e9da`。该值已由 PowerShell/.NET 与 Node 两套独立实现按所述 exact-5 格式复算一致，纠正先前无法由声明输入重现的记录值；五个源文件内容未变。旧 owner tombstone 不存在，Rustfmt 与 scoped diff-check GREEN；正式 production dependency audit 仍为 2,710 refs / 72 edges，且 `resource` source domain 的外部边为 0。后续由高层 asset/scene 或 diagnostics adapter 批量记录同名 counter，物理迁移时仅通过计划批准的 hidden `zr_resource::assembly` 暴露最小内部 metrics。报告脚本可保留 coverage 合同，但不能反向决定 owner 依赖。
- Unreal 主参考采用独立 `AssetRegistry` module（依赖 Core/CoreUObject 等，而高层 AssetManager 位于 Engine），在 `FAssetRegistryState` 中按 package name/path/class/tag 维护专用索引；过滤时先执行预计结果最少的条件，并用显式成本模型在“索引结果求交”和“直接过滤已有结果”之间选择。这支持先把 `zr_resource` 做成零重依赖 canonical owner，再按量化 workload 引入 per-kind/state/locator 索引或 adaptive query，而不是继续把资源实现塞在 Runtime 门面或盲目替换扫描循环。
- Bevy 的 `bevy_asset` 是独立 crate：typed asset 主存储使用带 generation 的 dense index，`AssetInfos` 另持 `path_to_index`、erased index、dependency/dependent map。该实现说明 stable identity、主存储和查询索引应有明确数据结构，而不是依赖全局对象遍历；但 Bevy asset crate 本身依赖 ECS/task/diagnostic 等设施，不能直接作为 Zircon M1 的零重依赖边界照搬。
- Fyrox 的独立 `fyrox-resource` crate 将 `ResourceRegistry` 的 UUID -> path 关联与 `ResourceManagerState` 的 loaders、IO、task pool、watcher、live resources 分开建模；Godot 则把 `ResourceUID` 的 id -> path map、可按需启用的 reverse path cache、全局 `ResourceCache` 和 `ResourceLoader` 分开。这两组证据共同支持 Zircon 先固化轻量 identity/record/immutable generation owner，把 loader、watcher、project catalog 和 Editor projection 留在高层 adapter；reverse/per-kind/per-state index 必须由 profiler workload 触发，而不是随物理拆 crate 一并预设。
- mutation consistency 复核发现比 scan 复杂度更低层的 Critical：current `ResourceRegistry::upsert` 对同 locator、不同 id 会移除旧 `by_id`，但 ResourceManager 没有把旧 id 的 management row/summary、payload、runtime slot、readiness 与 Removed event 作为一个原子 displacement 处理；批量 lazy registration 还会把批内已被挤出的前项再次发布进 projection，occupied-target rename 则可能留下 locator 不可达的 `by_id` 记录。同 id、不同 locator 的普通 upsert 会发送 `Updated(previous_locator=None)`，绕过 `Renamed` 及旧路径失效。authority/management、payload、runtime、readiness 当前分属四把锁；`acquire` 的 payload read 与 refcount increment 之间可并发 remove/re-register，旧 lease release 还可删除同 id 的新 payload。Unreal `FAssetRegistryState` 以 ObjectPath 为唯一 key，重复 Add 报错并让 Update/Remove 同步维护全部 accelerator；Bevy `AssetInfos` 也在一个 owner 内同步 `infos` 与 `path_to_index`。因此当前先执行单一 coherent `ResourceAuthority` + staged `ResourceMutationBatch` 硬切：全批 preflight id/locator/kind 和显式 rename，typed collision 失败零修改；commit 同步 registry、management/readiness generation、payload/runtime residency，事件延迟到 commit 后发布；`ResourceLease` 携带 residency token，旧代 release 不得修改新代；删除 registry-only/publish split、跨四锁 mutation 与无 `Result` 兼容入口。生产迁移面集中在 asset facade insert、project resource sync/batch、ImportedAsset ready dispatch 三个 owner。该事务及 collision/same-id locator/batch rollback/residency race/event ordering/upward tests 未完成前，不把 query-local metrics benchmark 当成下一项，也不实施 secondary index/heap/token 优化。
- 对生产源码做首个 `#[cfg(test)]` 前截断并排除 tests 目录的调用清单：`register_record`、单条 `register_lazy_record` 和显式 `rename` 都没有生产 caller；`register_ready` 是 asset facade 1 处加 ImportedAsset 31 个 match arms；`register_lazy_records` 是 project runtime 3 处；`store_payload` 是 ImportedAsset 的另一组 31 个 match arms。该规模说明硬切可直接删除旧入口，并把 ImportedAsset 统一擦除为 `Arc<dyn ResourceData>` 后进入 batch，不需要兼容 adapter。`ensure_resident` 则必须把 expected record revision/project generation 作为 payload-install precondition，不能保留 registry check 与 store 的 TOCTOU split。
- 允许开始查询算法优化的前置样本锁定为 1k/10k/100k records、均匀与单 kind 偏斜两种分布，分别测量 generation publish、id/locator lookup、全量/按 kind/按 state scan、首屏与深分页；保存 wall time、allocation、candidate checks、filtered rows 和 payload rows。只有这些结果证明查询占主导后，才比较 binary heap k-way merge、按 kind/state 的有序 secondary index 与 continuation-token pagination；任何方案还必须保持 stable locator order、generation immutability 和有界 reconciliation drain。

## 采集状态

- 先前记录的 Frameworks01 focused boundary 子集为 24/24 GREEN；2026-08-14 扩展重跑五个 Frameworks01 boundary module 时为 13/14 GREEN。唯一 RED 是干净 HEAD 的 `test_frameworks_01_physics_settings_error_boundary` 仍硬编码已由 Frameworks05 hard cut 删除的 `core/framework/foundation.rs`，实际唯一 Foundation `CoreError` consumer 已位于 `core/framework/foundation/config_manager.rs`；这是跨计划静态守卫陈旧，不是 owner 回退。守卫修复须取得其精确写所有权后再落盘，当前不伪报全绿。
- convention entrypoint Python contracts：30/30 GREEN；guard 文档 63 条规则、49 条 MUST、0 违规。
- 全仓 docs 路径审计在共享工作树上为 RED：221 份文档、623 项路径问题；这些是外部并行迁移状态，不并入本 M0 候选，也不伪报通过。
- 第一个 workspace validation-copy run 因 PowerShell 5 把 Cargo 的 PDB filename-collision warning 误当成终止错误而失败，run `fe2dbd18c74b45a99396ab9327b1981f` 未产生有效 HTML。采集器已改为按 `cargo.exe` 真实退出码判断；后续仅在没有其他受管 Cargo 占用 CPU/磁盘时重跑。
- Runtime validation-copy job `a09839b195444932a3557326955c225a` 执行约 13 分 46 秒后，协调器返回 `internal_error: database is locked` 并将作业留在 `cleanup_pending`；该轮没有保留下 HTML、日志或自然终态证据，因此既不记 GREEN，也不记性能 RED。必须待协调器完成清理并确认无其他 Cargo 竞争后，以新作业重新采集。
- 清理完成后再次请求 Runtime validation-copy materialization，在 13.8 秒内直接返回 `database is locked`，未创建 job、未启动 Cargo、未产生项目产物。系统化诊断已确认根因：`session.register` 在 `BEGIN IMMEDIATE` 写事务内调用 `failure.import_repository`，后者先复制并解析完整 `docs/plans`；两次实际注册分别占用写锁 25.3 秒与 35.2 秒，超过数据库 10 秒 busy timeout。并发 maintenance recovery 随后在记录 `validation_copy.recovery_failed` 时再次争抢同一写锁，使 watch thread 退出。
- 根因与回归要求已经交给协调器维护 owner：把 immutable failure snapshot 的收集/解析移到 accepted transaction 之前，事务内只校验 fingerprint、应用图并注册；同时保证 recovery 错误记录遇到 DB busy 不会杀死 watch thread。修复并热加载前不再重复提交 Cargo 作业，也不以提高 timeout 掩盖长写事务。
- `validate-matrix.ps1 -Package zircon_runtime -CargoProfile profiling -SkipBuild -LibTests -TestFilter resource_management -DryRun` 已成功渲染受管命令，目标为 `D:\cargo-targets\zircon-engine\pool\69f151e2e14bcfa0a738484eeed92b0c4f43859a69e1bdd059edb69a5602ad49`，未创建构建产物；其后首个真实请求因 `admission_checkpoint_stale` 在作业创建前拒绝，未启动 Cargo/rustc。协调器恢复为 `codex_sync=healthy` 后不复用旧请求，才执行下一轮。
- 2026-08-14 18:56:32 创建的受管 Windows profiling 作业 `25f05854c2114f1ea657d76fea939358` 使用 `D:\cargo-targets\zircon-engine\pool\db56c12236a53c610d6972c9c21587e9a24485cea07287a8a82b610e3fe13b61`，18:56:46 启动、18:57:37 终止，作业执行 51.3 秒、端到端 108.6 秒，状态 `released`、wrapper exit 1。`rustc` 在构建 `zircon_runtime` lib test 时发现 `scene/ecs/schedule_runner.rs` 引用不存在的 `scene/ecs/schedule_runner/tests/typed_worker_structural.rs`，因此 ResourceManagement 测试尚未执行，不能记 GREEN。该变更归属活跃 Runtime08 Session `runtime08-ecs-bundle-width-current-compile-r1-20260808`；Frameworks01 保留失败证据并继续独立工作，不补写、不回退外部 blob。2026-08-14 19:57 只读复核确认 `typed_worker_structural.rs` 与同模块声明的 `worker_callback_order.rs` 均已出现在 current source，原编译前置已解除；新的受管精确 gate 尚未执行，状态仍为 pending 而非 GREEN。
- 20:02:35 创建的新受管 Windows profiling 作业 `ceaaffb6bb374111a92c40aee8cdb722` 复用上述 D 盘兼容池，20:02:37 启动、20:14:00 结束、20:14:03 确认进程树退出并释放；执行 683.3 秒、端到端 695 秒，wrapper exit 1 / Cargo exit 101。该轮已经跨过 Runtime08 include 阻塞，但 current `zircon_runtime` lib-test 在生成测试二进制前累计 361 个编译错误与 1,520 个 warning，故 `resource_management` 过滤测试仍为 0 executed。可见末条诊断是外部 Text blob `text/cache/rich_cache.rs:477` 把 `String` 赋给 `Arc<str>`；同期 `zircon_runtime/src` 有 1,970 个 dirty path，不能把全局 current-source 失败归因于本 exact5，也不能记 focused GREEN。作业已自然释放且 D 盘 target 继续由协调器保留，没有在 C 盘产生项目产物。
- Transaction successor 在同一受管 Windows check compatibility pool `D:\cargo-targets\zircon-engine\pool\f9fef644bf8e441a49ad1c139495499657f126cd246ffca80d13868db535561d` 连续取得三个自然终态：`2073ee323c054aa3bf905b5b45effc2a`（213.96 秒、14 errors）、`4c8cfe4b33274318aa2fb793caaa1e6a`（114.13 秒、12 errors）、`1a4f6ffa6598415a94733b64181f68cd`（151.07 秒、3 errors），均已 released、`live_process_pids=[]`。第三轮 Resource/Asset 自有编译错误为 0，唯一剩余 cause 是 Runtime08 活跃 scope 内的三个 Scene duplicate-method diagnostics。随后加入的 net-zero batch 与 commit-publication gate 尚未取得 Cargo 自然终态，不能把“到达外部阻塞点”写成 current-source GREEN。
- 23:59:07 启动的后续 managed check job `32c6a4b503734d86805ef1d4065824a7` 使用 D 盘 pool `db56c12236a53c610d6972c9c21587e9a24485cea07287a8a82b610e3fe13b61`，00:06:12 进程树退出，执行窗口 425.84 秒。外层 `validate-matrix.ps1` 调用超时导致 coordinator 记录 `orphaned` 且没有 wrapper exit code；保留的最新 `zircon_runtime-9c96844cadef12ec/output-lib-zircon_runtime` 结构化诊断为 3 errors / 85 warnings，三个 error 仍全部是 Runtime08 的 `first_stable_camera_entity` 重复定义与调用歧义，Resource/Asset 自有 error 为 0。该 job 已由 coordinator 正常 release，`live_process_pids=[]`，target 按 `retained` policy 保留；由于 supervisor receipt 不是 passed ticket，且后续又加入 reservation 取消测试，本轮仍不能记 GREEN 或提交验收。
- 2026-08-15 同一 D 盘 pool 的三张 transaction successor job 均取得自然终态并释放：`d9a80db1a103477981c1d936d9096cfa` 于 00:45:32--00:46:34 执行 62.15 秒，覆盖 reservation 取消、last-good catalog refresh、targeted file preflight 与 runtime owner 拆分后的 production 编译面，结果为 Runtime08 3 errors / 85 warnings、自有 error 0；事件发布顺序统一改造后的 `e614bf66c642471cb9145916ec68dbe8` 于 01:34:59--01:37:10 执行 131.04 秒，发现本 scope 1 条 E0382；修复闭包捕获顺序后的 `ef6b2a1355bd458c85c1169afae36c0c` 于 01:39:30--01:41:39 执行 129.92 秒，重新收敛为 Runtime08 3 errors / 85 warnings、自有 error 0。三张均为 `released`、`live_process_pids=[]`，目标目录均为 `D:\cargo-targets\zircon-engine\pool\db56c12236a53c610d6972c9c21587e9a24485cea07287a8a82b610e3fe13b61`。命令使用 `-SkipTest`，且外部错误发生在 lib build，因此新增 transaction/event-order tests 未编译执行，不能记 GREEN 或提交验收。
- 上述数字是编译修复进度，不是 Resource 查询 runtime benchmark，也没有隔离 CPU/磁盘竞争，禁止用于宣称 transaction 性能改善。1k/10k/100k records、uniform/skew、publish/lookup/scan/page 的 profiling matrix 仍 pending；`Power Meter` 仍只返回 0 W，因此本轮继续不提供瓦特、能耗或“接近其他引擎”的结论。

## 待产物

| 范围 | cold | incremental | 状态 |
|---|---|---|---|
| root workspace | `workspace-cold.html` | `workspace-incremental.html` | pending |
| `zircon_runtime` package | `zircon-runtime-cold.html` | `zircon-runtime-incremental.html` | pending |

M0 只有在四个报告均为 current-source GREEN、依赖图与 crate/CI 锁定同步后才可完成；本文件存在不等于 timings 已通过。
