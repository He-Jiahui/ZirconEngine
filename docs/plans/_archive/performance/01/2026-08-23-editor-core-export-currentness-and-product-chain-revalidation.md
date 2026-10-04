---
related_code:
  - zircon_editor/src/core/export
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard
  - zircon_editor/src/ui/host/export_process_support
  - zircon_editor/src/ui/host/native_dynamic_export_preparation
base_reports:
  - docs/plans/performance/01/2026-08-19-editor-core-export-single-pipeline-architecture-revalidation.md
  - docs/plans/performance/01/2026-08-22-editor-build-export-generation-and-background-job-architecture-review.md
owner_plans:
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_plugins/09-export-publishing.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Programs/AutomationTool/Scripts/BuildCookRun.Automation.cs
  - dev/UnrealEngine/Engine/Source/Programs/AutomationTool/AutomationUtils/ProjectParams.cs
  - dev/UnrealEngine/Engine/Source/Programs/UnrealBuildTool/Modes/BuildMode.cs
  - dev/UnrealEngine/Engine/Source/Programs/AutomationTool/Scripts/CopyBuildToStagingDirectory.Automation.cs
  - dev/UnrealEngine/Engine/Source/Editor/UATHelper/UATHelperModule.cpp
  - dev/Fyrox/editor/src/export/mod.rs
doc_type: implementation-evidence
status: static_current_revalidated_single_pipeline_cutover_required_dynamic_blocked
---

# Editor core Export与产品调用链currentness复核（2026-08-23）

## 当前冻结

- `zircon_editor/src/core/export/**`：**9/9 Rust文件、3,061 physical lines、100,932 bytes、24 tests**。
- ordered workspace-relative path + NUL + raw bytes + NUL SHA256：
  `361a4a15d3a4254ddbe2c7f5518320d5242091791bbf6843641ed1cc519ac4c0`。
- 该指纹与8月19日报告完全一致，scope当前clean；9文件已再次逐文件读取。
- 产品锚点已沿wizard 35文件/8,311行、process support 4文件/542行、native preparation
  8文件/822行复核关键图构造、process、output、job、report和inventory边界。这些相关文件正在被并发修改，
  本轮只读且不计入9/9 owner账本。
- 没有current-source可执行文件；本文不声明export wall time、CPU、I/O、RSS、power或产品启动通过。

## 9/9逐文件复核

| 文件 | current-source结果 |
|---|---|
| `inventory.rs` | 64 KiB streaming BLAKE3、strong file identity与同generation path reuse应保留；stable directory仍全enumerate/sort/canonicalize/metadata，tool每generation仍probe；Drop静默执行全cache clone、pretty JSON、write、sync和replace。 |
| `mod.rs` | 导出nested executor/plan，使wizard可创建第二语义流水线；hard cut后只导出唯一headless graph owner。 |
| `pipeline.rs` | typed DAG、deterministic order、tamper-aware reuse和move-owned failure report正确；固定8阶段算法本身不是热点，问题是产品创建多份graph/report/inventory。 |
| `preset.rs` | explicit preset load/save的versioned atomic persistence合理；小控制面whole-file I/O不是P0。 |
| `stages/compile_host.rs` | fallback仍有2个private reader threads、per-byte tail push/pop、2个log sync和1个manifest sync，且无deadline/cancel/process-tree owner；产品已有更强adapter，此fallback不应成为第二authority。 |
| `stages/executor.rs` | CompileHost prepare手写33个client/14个server broad roots、3/4 tool probes及tools helper scan；PlatformBundle的新executor重复prepare并递归revalidate staged tree。 |
| `stages/mod.rs` | stage导出，无独立工作。 |
| `stages/platform_bundle.rs` | 4项layout存在性检查便宜；其“engine development tree validation”与外层project distribution bundle同名不同义。 |
| `tests.rs` | 24项覆盖DAG/reuse/tamper/preset/layout；缺唯一product graph、一次inventory/tool generation、fingerprint cancel、stage execute<=1、Build/Cook/Stage/Package receipt链及scale/RSS门。 |

## 当前产品链变化

并发变化值得保留：process tree cancellation已路由共享core process owner；stdout/stderr全量写artifact，
单行capture上限16 KiB，UI tail上限512行，每stage buffered output event上限16；job poll与UI projection不再
要求直接在UI线程跑child process。这些是bounded output/control-plane进展，不等于single pipeline完成。

当前P0静态shape仍为：

| shape | current |
|---|---:|
| wizard内`ExportPipelinePlan::new`出现 | 3 |
| `.core.json`使用点 | 2 |
| wizard内`ZirconBuildStageExecutor::new` | 2 |
| inventory Drop persistence owner | 1 |
| core fingerprint cancel/deadline check | 0 |
| fallback private capture threads | 2 |
| fallback `sync_all` | 2 |
| fallback per-byte `pop_front` path | 1 |

`run_core_compile_host`仍创建单阶段core graph、私有inventory与`.core.json`；
`run_core_platform_bundle`随后创建新executor并运行CompileHost->PlatformBundle graph。第二次prepare会再次
遍历hand-written roots、helper、Python/Cargo/rustc/Node identity并验证staged tree。外层wizard同时拥有自己的
8-stage plan、progress和report。改善output queue没有消除这组重复工作。

## 结构性判定

### P0：唯一run却有多图、多report、多inventory

正确数据流必须是：

`ExportRunGeneration -> BuildProductManifest -> CookManifest -> PluginPackageManifest -> StageMapping -> Package/ArchiveReceipt`

UI、commandlet和CI共享一个headless graph owner；UI仅投影receipt。CompileHost不再在Editor中维护Rust/Node
依赖目录清单，而消费build system action receipt。Cook/Pack/Plugin/Stage按显式manifest传递内容地址和
generation，不从输出目录重新猜membership。每stage每run最多execute一次。

### P0：stable reuse先支付全树发现成本

persistent file digest避免stable file bytes重hash，但每次新inventory仍要枚举目录、排序、canonicalize、
读取metadata并启动tool version process，才知道stage能否reuse。client静态root清单33项，且多个项本身是
大目录。正确算法让build/cook/package owner发布action/product manifest；内容hash只验证changed/untrusted
entry和明确的tamper gate，不是每次F4的第一步。

### P0：取消、持久化和I/O预算不属于同一owner

fingerprint路径cancel/deadline check为0，inventory析构执行不可见durable I/O；fallback又拥有private threads
和durability barriers。Runtime11必须拥有enumeration/hash/process/log/report persist的entry/byte/age/deadline
预算和terminal receipt。Drop I/O、caller durability wait、orphan staging和silent persist error均为0。

### P1：output按行有界但完整驻留合同仍需测量

16 KiB单行、512 tail和16 buffered event是正确上限；仍需验证process capture chunk、job result、stage
snapshot、view model和terminal result不会再次复制同一tail/diagnostic。完整log只在artifact file中保留，
内存只传compact cursor/summary；UI按visible output range读取，不把1 GiB log重新materialize。

## Unreal主参考依据

- `BuildCookRun.Automation.cs:26-45,251-266`由一个top-level owner按Build、Cook、Stage、Package、Archive
  顺序调用并计量整个run；不是在每个stage里再创建一条流水线。
- `ProjectParams.cs:1562-1577,1611-1633,1714-1736,1849-1906`将Cook/Pak/Stage/Archive/Build及
  skip/reuse作为不同显式选择，支持独立stage receipt与resume语义。
- `BuildMode.cs:426-515,541-620`构建/合并action graph、mount action history并判断outdated actions；
  编译增量真相属于build owner，不属于Editor手写递归source hash。
- `CopyBuildToStagingDirectory.Automation.cs:1617-1642`建立staging manifest，`:2941-2966`按mapping做
  incremental copy，`:6528-6572`从同一manifest创建pak/copy并执行post-stage hooks。
- `UATHelperModule.cpp:382-489,524-540`串行化UAT process，绑定output/cancel/completion/failure，并通过
  graph task回投UI。Zircon现有EditorJobSystem/process-tree方向可保留，但不能让UI成为第二graph owner。
- Fyrox export只作为cancel/process cleanup旁证，不足以证明manifest、durability或大规模增量算法。

## Hard-cut计划目标

1. Editor15建立UI/commandlet/CI共享的唯一`ExportRunGeneration`和graph；删除
   `ExportWizardCoreStageProjection`、nested plan、`.core.json`及第二inventory。
2. build owner返回`BuildProductManifest { toolchain/action generation, target/config, produced files }`；
   Editor删除33/14项hand-written source dependency tree。
3. Runtime04 Cook/Pack和Plugins09 native package各返回immutable content-addressed manifest；Stage只合并
   explicit destination mapping，Package/Archive只消费该receipt。
4. Runtime11/Editor14拥有bounded enumeration/hash/copy/process/log/persist jobs；每个bounded slice检查
   cancel/deadline，persist显式返回receipt，Drop I/O=0。
5. EditorUI08只消费stage/job/output generations；stable UI filesystem calls和row rebuild均为0，完整log
   通过paged artifact cursor读取。
6. 保留strong file identity、streaming BLAKE3 fallback、deterministic order、atomic replace、tamper detection、
   16 KiB line/512 tail/16 buffered-event上限和process-tree cleanup。

## 验收矩阵

| gate | matrix | 必须满足 |
|---|---|---|
| ownership | UI/commandlet/CI，client/server，partial/full/resume/cancel/fail | graph=1、report journal=1、inventory generation=1、stage execute<=1/run、nested/core report=0 |
| discovery | files `1/1K/100K`，stable/1%/rename/delete，tool change | stable Editor full-tree walk=0；tool probe<=1/toolchain generation；work随changed manifest entries缩放 |
| artifacts | build/cook/plugin/pack/stage `1/1K/100K`，log delete，required tamper | log delete rebuild=0；tamper只invalidate exact stage/downstream；stable read/write/copy near 0 |
| scheduling | cancel during enumerate/hash/build/cook/copy/persist，output `1 MiB/1 GiB` | bounded cancel p95；private thread=0；Drop I/O=0；queued/running/result bytes/age/RSS有界 |
| product | cold/warm/1% changed F4，至少31次，launch client/server export | WPR CPU/waits/CSwitch/File+process I/O、allocator/RSS、power及p50/p95/p99/CI/effect size；产品可启动 |
| render | 启动produced current-source rendering product | RenderDoc pixel/draw/pass/GPU parity；不用于证明export CPU、I/O或power |

## 当前验证回执

- 9/9 full read、fingerprint与8月19日一致：GREEN。
- `rustfmt --edition 2021 --config skip_children=true --check` 9/9：GREEN。
- `test_editor15_export_generation_inventory_contract`：10/10 GREEN。
- scoped `git diff --check`：GREEN；core source无本轮改动。
- docs convention：本轮新增/更新文档owned violations `0`；仓库全局3,129 documents、275 affected、
  801 existing violations，故全局门仍RED。
- Managed Cargo、WPR/ETW、allocator/RSS、power、Tracy、export launch、RenderDoc：未执行；current-source
  executable缺失且managed Windows会话不可执行。
- 受保护账本与编号计划不改；动态门通过前不提交里程碑、不发送企微完成通知。
