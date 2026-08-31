---
title: Editor Build/Export、Preset、Pipeline、Cook、Pack、Platform Bundle、Publishing、Resume 与 Determinism 当前工作树复审
category: zircon_editor
report_id: Editor269
review_date: 2026-08-31
baseline_head: working-tree
observed_head: 18481bc218dc544d3232d7d8826ac5fb97f7cb0c
canonical_owner: docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
related_reports:
  - docs/plans/optimize/zircon_editor/257-editor-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-current-working-tree-review.md
  - docs/plans/optimize/zircon_editor/266-editor-filesystem-project-scene-autosave-journal-session-io-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/204-runtime-filesystem-resource-io-path-atomic-transaction-recovery-security-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/207-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-cook-package-incremental-build-worker-determinism-current-working-tree-review.md
plan_sources:
  - docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
  - docs/plans/zircon_editor/editor/15/2026-07-12-m1-1-export-pipeline-contract.md
  - docs/plans/zircon_editor/editor/15/2026-07-18-export-generation-inventory.md
  - docs/plans/zircon_editor/editor/15/2026-07-12-typed-export-error-hard-cutover.md
  - docs/plans/zircon_editor/editor/15/failure-2026-07-17-export-overlapping-recursive-digests.md
  - docs/plans/zircon_editor/editor/15/failure-2026-07-22-export-output-tail-durability-backpressure.md
  - docs/plans/zircon_editor/editor/15/failure-2026-07-22-export-pack-byte-clone-pipeline.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Programs/AutomationTool/Scripts/BuildCookRun.Automation.cs
  - dev/UnrealEngine/Engine/Source/Programs/AutomationTool/AutomationUtils/ProjectParams.cs
  - dev/godot/editor/export/editor_export_platform.h
  - dev/godot/editor/export/editor_export_platform.cpp
  - dev/godot/editor/export/editor_export_preset.h
  - dev/godot/editor/export/editor_export_preset.cpp
  - dev/godot/editor/export/editor_export_plugin.h
  - dev/godot/editor/export/editor_export_plugin.cpp
  - dev/bevy/crates/bevy_asset/src/processor/mod.rs
  - dev/bevy/crates/bevy_asset/src/processor/log.rs
  - dev/Fyrox/fyrox-build-tools/src/export/mod.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/BuildProcessors/CoreBuildData.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/BuildProcessors/CorePreprocessBuild.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/ShaderStripping/IVariantStripper.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/ShaderStripping/ShaderPreprocessor.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/ShaderStripping/ShaderStrippingReport.cs
doc_type: current_source_review
review_status: review_complete
implementation_status: pending
source_recheck_required: true
tooling_scope: excluded_by_user_request
coordination_tracking: skipped_by_user_request
---

# Editor Build/Export、Preset、Pipeline、Cook、Pack、Platform Bundle、Publishing、Resume 与 Determinism 当前工作树复审

## 1. 结论

当前代码已经形成若干值得保留的局部底座：八阶段 `ExportStage` 已提升到 `zircon_runtime_interface`；pipeline 能拒绝重复节点、缺失依赖和环，显式记录 `Skipped`；`ExportGenerationInventory` 能对同 generation 的重叠目录复用摘要、以 64 KiB 缓冲哈希文件并检查读取前后身份；wizard 子进程具备进程树取消、完整日志落盘、512 行有界 tail 和 typed failure source；core `CompileHost`/`PlatformBundle` resume 会重新校验已有输出摘要。这些都不是临时空壳。

但产品级 Export 仍未形成一个可信的工程系统。当前最危险的五项 P0 全部可从生产路径到达：

1. 同一 `workbench.build_export.execute.<profile>` 可以按 control id 进入完整 wizard，也可以 fallback 到旧 `DesktopExportJobQueue`。两条路径各自拥有队列、状态和成功权威；旧路径完全不执行 `CookAssets`、`Pack`、最终 `PlatformBundle` 和 `Report`。
2. 旧路径把含 fatal diagnostics 的 plan 和 Cargo 非零退出都包装成 `Ok(EditorExportBuildReport)`；worker 把任何 `Ok(report)` 当作成功，`DesktopExportExecutionSummary::from_report` 又无条件写入 `Exported`。
3. wizard 的 core `CompileHost` adapter 丢弃子进程 `exit_code`，把非零退出转换为 `Ok(ZirconBuildCommandExecution)`。core report 因而可持久化为 `Passed`；下次 resume 会跳过编译并合成 `exit_code: Some(0)`。
4. `ExportPreset` 和 core stage executor 不携带 target platform。`CompileHost` 只按 client/server 选择 `hub,editor,runtime`，`PlatformBundleLayout` 使用编译 Editor 的宿主 `cfg(target_os)` 和 `EXE_SUFFIX`。Android/iOS/Web/Linux/macOS profile 仍可被 UI 标为 Ready，却消费当前宿主的 Editor/Hub/Runtime artifact。
5. wizard 声明了 `expected_stdout_keys`、consumed/produced artifacts，但执行器从不验证这些合同。常规阶段只要退出码为 0 且未打印 fatal progress 就算 Passed；UI 还会把 planned artifact path 与真实 artifact 合并显示。最终 bundle、pack 或 report 不存在时仍可能呈现成功。

因此，Editor15 计划文本中的“生产向导/core stage executor 已完成”只能解释为局部切片完成，不能解释为工程级 Export 完成。当前账本为：P0 **5 Open / 0 Partial / 0 Closed**，P1 **48 Open / 12 Partial / 0 Closed**，P2 **14 Open / 0 Partial / 0 Closed**；48 个资格门为 **40 Fail / 7 Partial / 1 Pass**。

所有权也需要修正：Editor 应只拥有 preset authoring、job presentation、operator decision 和 diagnostics projection；canonical build/cook/pack/platform bundle/publish service、artifact receipt、resume journal 与 final export truth 应由 Runtime 拥有，并由 Editor、App/commandlet 与 CI 通过同一 facade 调用。Tooling 本轮按用户要求排除，现有 Python 命令只作为需要被 Rust service 接管的数据合同/交接点审查，不提出本轮 Python 实现优化。

## 2. 审查边界与 currentness

### 2.1 冻结清单

本轮冻结时间为 `2026-08-31T16:32:39.1700886+08:00`。工作树存在大量并发修改，`observed_head` 只记录提交基线；下面的选择集指纹才表示本报告实际读取的源码内容。

| 范围 | files | lines | non-empty | bytes | tests | ignored | fingerprint |
|---|---:|---:|---:|---:|---:|---:|---|
| Editor export/core/wizard/legacy queue/UI production set | **103** | **15,554** | **14,247** | **541,268** | **80 markers** | **16** | `0b9e1d8956c5964bf950b76f1805aff9e9950ada35cbabdd339b6d6896ee7b1f` |
| Runtime Interface/Runtime/App export handoff set | **49** | **7,913** | **7,205** | **319,384** | **48 markers** | **2** | `b2473c76ee7c93f048a59560c4e6487c4260695bbabb9471cc349bfee1a72502` |
| Unreal/Godot/Bevy/Fyrox/Unity Graphics references | **16** | **12,036** | **10,466** | **488,675** | **0** | **0** | `1f4b4b7f80a4c61e56099e6f782a57870f915e266fc6e4cfda5cbde597734025` |

Editor 选择集递归覆盖 `core/export`、完整 wizard production tree、legacy export manager/Cargo/process、native dynamic preparation、retained action/job queue/projection/session、Build/Export pane conversion和action routing；排除独立 `tests/`、`tests.rs`、`*_tests.rs`。Runtime handoff集合覆盖 interface `export`、Runtime `export_build_plan`、project export profile/manifest 和 App export bootstrap。Runtime pack/cook底层由 Runtime207 保持 canonical owner，本报告只复核其 Editor handoff，不重复统计底层 finding。

指纹算法与近期报告一致：workspace-relative、小写 `/` 路径；每个文件先取 SHA-256，再把 `path + NUL + file_hash + LF` 依序输入总 SHA-256。

### 2.2 证据等级与限制

- **E3**：逐文件读取 preset、inventory、pipeline、core stages、wizard plan/execution/job/session/output/UI、legacy plan/materialize/Cargo/job queue、profile projection与Runtime handoff。
- **E2**：读取相邻 source-shape/unit tests、Editor15计划/failure记录，并对照本地五套参考源码。
- **E1**：本轮没有运行 Cargo、真实 Editor、真实 Python export、Android/iOS/Web toolchain、签名、安装、运行或 crash/fault E2E。测试标记数量不是通过证据。
- **E0**：没有固定项目、目标平台和硬件上的全量/增量导出 p50/p95/p99、CPU、RSS、I/O、cache hit、产物大小或与 Unreal 的同口径 benchmark，因此不得宣称性能或表现优于 Unreal。

## 3. 当前实现事实

### 3.1 Shared DTO 已存在，但服务 owner 放错层

`zircon_runtime_interface/src/export/stage.rs` 定义了 `Validate`、`SourceTemplate`、`NativeDynamic`、`CompileHost`、`CookAssets`、`Pack`、`PlatformBundle`、`Report` 八阶段；artifact 有 key、locator、可选 256-bit digest；report 能记录 stage、I/O、status 和 diagnostics。`ExportPipelinePlan` 也能建立拓扑顺序、拒绝环并在 fingerprint 匹配时显式产生 `Skipped`。

问题是 orchestration 位于 `zircon_editor::core::export`，而不是 Runtime build service。CLI/CI/App 没有消费相同 pipeline 的生产入口，Editor 反而成为 cooked truth、resume report和平台 layout 的 owner。更严重的是 `ExportPipelineReport` 只有 `stages`；没有 schema/version、request digest、project/profile revision、target triple、BuildSet/toolchain/environment identity、generation、attempt、时间、资源统计、cancel/rollback disposition或 final receipt。

### 3.2 Preset 与 Profile 分裂，target platform 没有进入 core request

Runtime `ExportProfile` 拥有 `target_platform`、target mode、runtime profile、packaging strategies、build mode、output name、plugins/features和asset filter；Interface `ExportPreset` 只有字符串 `profile_ref`、target mode、debug、filter、entry/keep、plugin subset、cook options和customized files。向导加载 preset 后只查找同名 profile并比较 target mode，随后把 target platform降为显示字符串。

这不是有效绑定：profile revision/digest、project manifest generation、plugin catalog generation、target platform/arch/triple、SDK/toolchain、platform capability和signing policy都没有冻结到 immutable request。preset schema还是 version 0、迁移链为空，validation 只检查 profile非空和plugin id空值/重复；include/exclude字符串、entry/keep URI、customized files、plugin features之间没有交叉验证或复杂度预算。

### 3.3 八阶段只是 UI 计划，core executor 实际只有两阶段

`ZirconBuildStageExecutor` 只实现 `CompileHost` 与 `PlatformBundle`，其他 stage 返回 `UnsupportedStage`；`zircon_build_stage_plan()` 固定为两节点。wizard 的 `plan.rs` 另行生成完整八阶段命令，六个阶段通过 `python -m tools.zircon_export` 执行，只有 CompileHost/PlatformBundle带 `core_projection`。

因此生产结构实际上是：

```text
Wizard eight-stage command list
  Validate / SourceTemplate / NativeDynamic -----> Python command authority
  CompileHost -----------------------------------> Editor core plan, then wizard projection
  CookAssets / Pack ------------------------------> Python command authority
  PlatformBundle --------------------------------> core existence check, then Python command
  Report ----------------------------------------> Python command authority

Legacy DesktopExportJobQueue
  Runtime ExportBuildPlan -> native staging -> materialize -> optional Cargo
  (no CookAssets, Pack, final PlatformBundle, Report)
```

这两条路径没有共同 request、journal、artifact store、stage receipt或 terminal export receipt，不能证明同 preset 产生同一产品。

### 3.4 Core CompileHost resume 可把失败永久缓存成成功

常规 `SystemZirconBuildCommandRunner` 会检查 `status.success()`；但 wizard adapter `ExportWizardZirconBuildRunner::run` 调用 process runner后，仅把 stdout/stderr拼成 bytes并返回 `Ok(ZirconBuildCommandExecution)`，没有检查 `execution.exit_code`。上层 core pipeline随即给 CompileHost写 `Passed`，只要旧 staged engine仍能被摘要。

随后外层 `execute_export_wizard_stage_with_output` 才看到非零 exit并把当次 UI stage标 fatal，造成同一 stage同时拥有 core `Passed` 和 wizard `Fatal` 两个事实源。下一次 `load_core_pipeline_report` 读取 core report，匹配 fingerprint和旧输出后跳过 CompileHost；`run_core_compile_host` 在 runner没有新execution时合成 `exit_code: Some(0)`。这构成可重复的 failure-to-success resume poisoning。

此外，损坏 core JSON被 `serde_json::from_slice(...).ok()`静默当作没有 resume；staging filename仅含 PID；report没有 schema/generation/attempt；Windows replace和Unix rename后没有共同 parent-directory durability receipt。

### 3.5 CompileHost 构建的是当前引擎工具，不是目标游戏产品

client mode固定调用 `zircon_build.py --targets hub,editor,runtime`，server mode固定 `runtime`；没有 `--target`、arch、SDK、linker、sysroot或platform adapter。client layout还要求 `zircon_hub`、`zircon_editor`、runtime library和assets目录，说明它验证的是 staged engine开发工具树，而不是目标游戏host。

`PlatformBundleLayout` 使用当前编译宿主的 `cfg(target_os)`选择 DLL/dylib/so，使用 `std::env::consts::EXE_SUFFIX`选择可执行文件名；它完全不读取 Runtime `ExportProfile.target_platform`。validation只有 `is_file/is_dir`，不验证digest、ABI、BuildSet、dependent libraries、RPATH/install name、runtime profile、pack linkage、template revision、signature或launcher smoke test。

### 3.6 Mobile/Browser 是 scaffold，却在产品层显示 Ready

内置profile包括 Windows、Linux、macOS、Android、iOS、WebGPU、WASM和Headless。Android/iOS/Web使用 `client_scaffold_export_profile`，但仍含 `SourceTemplate + LibraryEmbed`；target row只要plan没有 fatal diagnostics就显示 `Ready`。wizard随后仍用 host-derived layout作为 `host_executable`，并只把平台转换成“Android”“iOS”“WebGPU”等字符串传给命令。

当前没有 platform adapter qualification、SDK/template availability、cross compiler、device ABI、codesign/provisioning、browser loader/WASM glue、install package或run receipt。把 scaffold呈现为可导出的 Ready target会直接误导用户。

### 3.7 Cook/Pack 的数据闭包没有进入同一 runtime graph

wizard默认把 source asset manifest指向输出目录内的 `assets/assets.json`，而不是绑定 project registry generation、entry scene closure、keep list、dynamic load policy和cook recipe的 immutable manifest。`entry_scenes`、`keep_list`、`customized_files`、cook deterministic/compression和plugin subset虽然存在于 preset DTO，但 core executor没有 CookAssets/Pack stage去消费它们。

Pack只在 plan层检查 previous/delta path是否成对出现；没有证明 base/target format、project/profile、encryption/compression、chunk table或artifact generation兼容。Runtime207已经记录 pack最终 `Vec<u8>`、raw source manifest重新读取、缺content-addressed cooked closure和delta/installer全包owner问题，本报告继承这些 owner结论，不重复建立新 P0。

### 3.8 Declared artifact 是展示元数据，不是执行合同

每条 wizard command都包含 `consumed_artifacts`、`produced_artifacts` 和 `expected_stdout_keys`。但 `expected_stdout_keys` 除 plan与测试外只被复制，没有执行校验；produced artifact主要供 output capture定位三份log和UI planned rows，stage completion不检查业务artifact存在、类型、大小、digest或与input receipt关系。

PlatformBundle尤其明显：core pass只验证 `build_output_root/ZirconEngine`，随后 Python PlatformBundle只按exit code判断；最终计划路径是 `out/bundle/<profile>`，core report却仍指向staged engine root。Report stage的 `pipeline_report`也可仅作为planned path显示。

### 3.9 Legacy Desktop Export 存在确定性的假成功

legacy manager在plan含fatal diagnostics时仍materialize diagnostic files并返回 `Ok(EditorExportBuildReport)`。Cargo manifest缺失时 `invoke_cargo_build`返回 `success: false`的普通 invocation；Cargo非零退出同样只是 invocation数据和字符串diagnostics，manager没有把它转为 `fatal_diagnostics` 或 `Err`。

worker对任何非取消的 `Ok(report)`返回成功 ticket；summary constructor无条件写 `DesktopExportExecutionState::Exported`。状态消息虽然可能拼出fatal文本，但标签仍是“Exported”。报告本身只有generated/copied path计数与字符串diagnostics，没有cook/pack/bundle/sign/install状态、artifact digest、target receipt、duration、generation或smoke test。

### 3.10 Job、cancel、日志有局部工程化，但没有 durable operation

wizard和legacy都接入 `EditorJobSystem`；wizard子进程设置进程树取消并轮询，完整stdout/stderr流式写盘，tail限制512行，每行上限16 KiB。`ExportGenerationInventory`使用强文件身份快筛、流式hash、重叠Merkle projection和同generation tool probe，这些应保留。

但两条队列都在内存中：wizard job id由profile字符串构造，legacy id是process-local `next_id`；没有 request digest、attempt、generation、lease、durable journal、restart reattach、terminal index或output-root lock。wizard session `Drop`只释放tool lease，不主动cancel controller。legacy与wizard互不知晓，同profile/同output可同时执行并相互覆盖。

### 3.11 Output backpressure 与 UI truth 仍有断链

完整日志的 stdout、stderr、manifest依次 `sync_all`，执行线程串行等待三次durability barrier；manifest不含command、exit code、stage/request/generation、started/finished时间或environment digest。

event channel容量192。StageOutput每stage最多发送16条，但计数只增不减，consumer drain后producer也不会恢复额度，因此每stage第17条以后永久不再实时显示。非output事件使用blocking `send`，UI不drain时producer可停在progress/terminal event。coalesced count只能说明丢了多少event，不能恢复缺失的live tail。

Report UI只从最多512行stdout tail中扫描第一个花括号对象并静默 `serde_json::from_str(...).ok()`；长报告或前置输出会丢失正文。artifact列表先复制planned artifacts再合并actual stdout artifacts，report path也会fallback到planned path；不存在的文件仍能显示为Pipeline Report或bundle artifact。

## 4. 本地参考源码对照

| 参考 | 可验证事实 | Zircon 应吸收 | 不应照搬 |
|---|---|---|---|
| Unreal AutomationTool | `BuildCookRun.Automation.cs:259-267`按 `Build -> Cook -> CopyBuildToStagingDirectory -> Package -> Archive -> Deploy -> Run`调用；`ProjectParams`显式拥有client/server target platforms、configs、cook/pak/stage/package/archive/deploy/run、incremental与signing参数。 | immutable request必须携平台/配置/阶段策略；每个阶段有独立资格、artifact与可重跑边界；stage/package/deploy/run不能退化为一个exit code。 | 不复制AutomationTool庞大flag面、全局静态状态和C#继承层级。 |
| Godot export | Preset直接绑定platform；platform执行template/project validation、dependency closure、pack/zip/patch/encryption/shared object；plugin有begin/file/end、platform support、dynamic options和customization hash。 | 建立Runtime platform adapter、content closure、typed plugin contribution、cache key与patch compatibility；Editor preset只编辑并引用已注册platform能力。 | 不复制同步主线程export、String/Dictionary弱类型或单进程全局Editor状态。 |
| Bevy asset processor | 以source full hash决定重处理，声明deterministic/lossless/configurable；`ProcessorTransactionLog`在开始前写Begin，成功后写End，启动时识别unfinished并重处理；processor共享状态可并行。 | CookAssets/Pack必须消费不可变source/recipe identity、WAL与generation；unfinished action要fail-close重做，不能靠stage label和旧文件存在。 | Bevy processor只覆盖资产处理，不足以替代平台build/package/sign/publish。 |
| Fyrox build tools | `ExportOptions`明确target platform与build target；按PC/WASM/Android分派build、资源copy、binary copy、run，并传cancel flag。 | 即使较小型实现也证明target platform/target triple必须到达真实build command，cancel必须贯穿child。 | 其轮询、字符串错误和简单目录copy只是下限，不是Zircon目标架构。 |
| Unity Graphics | `CoreBuildData`按BuildTarget及Standalone Player/Server subtarget采集build-scoped数据；pre/post build负责创建/Dispose；shader stripper是可发现扩展点，支持before/after scope，并报告input/output variant与耗时。 | Render cook/strip应消费target-qualified build context，扩展点有lifecycle，报告保留可归因数量与耗时；不能在Editor host上下文猜target。 | 本地Graphics仓库不是完整Unity Player exporter，不能把shader build processor推断成完整发布架构。 |

## 5. Finding 账本

### 5.1 汇总

| 级别 | Open | Partial | Closed | 合计 |
|---|---:|---:|---:|---:|
| P0 | 5 | 0 | 0 | 5 |
| P1 | 48 | 12 | 0 | 60 |
| P2 | 14 | 0 | 0 | 14 |

### 5.2 P0

| Finding | 状态 | 当前证据与退出条件 |
|---|---|---|
| **ED-EXPORT-P0-001** 两套生产export权威 | Open | control匹配进入wizard，fallback进入legacy queue；两者阶段、队列、结果和输出互不关联。硬切为一个Runtime ExportService与一个terminal receipt，删除legacy生产执行权。 |
| **ED-EXPORT-P0-002** legacy fatal/Cargo失败仍标Exported | Open | manager返回`Ok(report)`，worker接受，summary无条件`Exported`。任何fatal、缺manifest、非零exit、产物验证失败必须产生typed failed receipt，UI只投影receipt。 |
| **ED-EXPORT-P0-003** core CompileHost失败可被resume成成功 | Open | wizard adapter丢exit code，core写Passed；resume skip合成0。runner结果必须包含status并在artifact publish前fail-close，旧有冲突report须quarantine。 |
| **ED-EXPORT-P0-004** target platform未进入build/layout | Open | core只见target mode，compile/layout由当前host决定；mobile/browser scaffold仍Ready。冻结target platform/triple/arch/SDK/toolchain并由platform adapter资格化后才允许执行。 |
| **ED-EXPORT-P0-005** 声明产物未验证仍可完成 | Open | `expected_stdout_keys`无consumer，planned path可显示为artifact，最终bundle/report未校验。每阶段必须提交typed artifact receipt，final success要求bundle/sign/install/smoke policy全部满足。 |

### 5.3 P1：所有权、Request、Preset 与 Profile（01-12）

| Finding | 状态 | 当前证据与退出条件 |
|---|---|---|
| **ED-EXPORT-P1-001** pipeline service位于Editor | Partial | shared DTO已在Interface；执行、inventory、resume report、platform layout仍由Editor拥有。迁至Runtime service，Editor只保留authoring/presentation。 |
| **ED-EXPORT-P1-002** preset只用字符串引用profile | Open | 无profile revision/digest/manifest generation；request builder必须解析并冻结immutable profile identity。 |
| **ED-EXPORT-P1-003** preset/profile重复target mode | Open | 两份值只比较相等，仍可独立漂移；effective request只能有一个qualified target mode。 |
| **ED-EXPORT-P1-004** preset不含target platform binding | Open | core完全看不到platform；绑定qualified platform adapter id与revision。 |
| **ED-EXPORT-P1-005** Runtime profile词汇较完整但未贯穿 | Partial | platform/host/resource strategy已存在；没有进入core fingerprint、stage receipt和final product。 |
| **ED-EXPORT-P1-006** 无target triple/arch/ABI | Open | command不传Cargo target；增加triple、arch、ABI、CPU/features、link policy。 |
| **ED-EXPORT-P1-007** 无SDK/toolchain/platform capability receipt | Open | 只有工具version文本probe；资格化compiler/linker/SDK/template/codesign/device。 |
| **ED-EXPORT-P1-008** preset schema无演进 | Partial | versioned envelope存在；VERSION=0且migration chain为空。建立兼容矩阵、migration artifact与downgrade拒绝。 |
| **ED-EXPORT-P1-009** preset validation过浅 | Open | 未验证filters、AssetRef、customized path、feature关系、content budget。建立data-only semantic validation。 |
| **ED-EXPORT-P1-010** plugin subset不绑定compiled plugin plan | Open | package/features只是字符串集合；冻结catalog/provider/package/artifact digests。 |
| **ED-EXPORT-P1-011** project/content generation未冻结 | Open | entry/keep没有registry/project snapshot token；所有closure query必须绑定同一ProjectSnapshot。 |
| **ED-EXPORT-P1-012** build configuration只有debug bool | Open | 缺Development/Shipping、checks、LTO、symbols、strip、sanitizer、determinism policy与config digest。 |

### 5.4 P1：Graph、Stage、Receipt 与 Resume（13-25）

| Finding | 状态 | 当前证据与退出条件 |
|---|---|---|
| **ED-EXPORT-P1-013** shared stage enum/拓扑仅是局部底座 | Partial | 八阶段、环检测、显式Skipped存在；生产executor不完整且入口不统一。 |
| **ED-EXPORT-P1-014** core只实现2/8 stage | Open | 其余返回UnsupportedStage；所有stage必须进入同一Runtime registry/DAG。 |
| **ED-EXPORT-P1-015** Python command链与core双authority | Open | 同stage有command report和core report；Rust service必须成为唯一状态机，tool invocation只是executor detail。 |
| **ED-EXPORT-P1-016** pipeline强制串行 | Open | 拓扑顺序逐节点执行；引入依赖DAG、resource-vector admission和deterministic scheduling policy。 |
| **ED-EXPORT-P1-017** fingerprint/输出disk revalidation局部有效 | Partial | core两阶段会hash输入/输出；环境闭包和其余stage无同等保证。 |
| **ED-EXPORT-P1-018** trait默认`can_reuse=true`不安全 | Open | 新executor忘记override即可复用未校验artifact；默认必须fail-close或要求显式ReuseDecision。 |
| **ED-EXPORT-P1-019** failed record把expected_outputs写成outputs | Open | 计划路径被记录为失败输出；区分declared、materialized、verified、published artifact。 |
| **ED-EXPORT-P1-020** report无schema/request identity | Open | 只有stages；增加schema、operation/request/payload/profile/project/buildset digests。 |
| **ED-EXPORT-P1-021** report无generation/attempt/lineage | Open | 无法区分重试或旧文件；所有stage receipt携generation、attempt、parent、lease fence。 |
| **ED-EXPORT-P1-022** report无时间/资源/环境 | Open | 无started/finished/duration/CPU/RSS/I/O/cache/toolchain/env policy；补可比较metrics。 |
| **ED-EXPORT-P1-023** typed error链只在局部保留 | Partial | wizard ticket有typed source；report diagnostics和多数stage仍是`Vec<String>`。建立code/severity/domain/source/action/redaction。 |
| **ED-EXPORT-P1-024** malformed resume被静默忽略 | Open | decode `.ok()`退回fresh run；损坏/未来版本必须quarantine并产生typed recovery decision。 |
| **ED-EXPORT-P1-025** resume没有事务journal | Open | 一个pretty JSON snapshot不能证明Begin/commit/publish；建立WAL或append-only action journal与terminal index。 |

### 5.5 P1：Compile、Cook、Pack、Bundle 与 Publishing（26-40）

| Finding | 状态 | 当前证据与退出条件 |
|---|---|---|
| **ED-EXPORT-P1-026** CompileHost产物是Editor/Hub工具树 | Open | client固定`hub,editor,runtime`；目标应是project product host及其runtime closure。 |
| **ED-EXPORT-P1-027** compile source closure手工枚举 | Open | 漏环境变量、SDK、linker、RUSTFLAGS、platform config和完整tool binary identity；由build graph声明输入。 |
| **ED-EXPORT-P1-028** tool identity仅version输出 | Open | 同version不同binary/config仍命中；记录binary digest、resolved path、SDK/sysroot、env allowlist digest。 |
| **ED-EXPORT-P1-029** inventory有可保留优化但owner错误 | Partial | streaming hash、identity cache、overlap reuse存在；应由Runtime artifact/build inventory共享。 |
| **ED-EXPORT-P1-030** inventory Drop同步持久化且吞错 | Open | Drop pretty-encode/write/fsync并忽略错误；显式bounded persistence ticket与receipt。 |
| **ED-EXPORT-P1-031** PlatformBundle按host OS命名 | Open | `cfg(target_os)`/`EXE_SUFFIX`决定目标；改由qualified platform adapter。 |
| **ED-EXPORT-P1-032** bundle只做存在性校验 | Open | 无digest/size/type/ABI/dependency/buildset/profile校验；提交canonical manifest和verified receipts。 |
| **ED-EXPORT-P1-033** final bundle与core staged engine混淆 | Open | core输出staged root，wizard计划输出`bundle/<profile>`；建立独立Stage/Bundle/Package artifact types。 |
| **ED-EXPORT-P1-034** 无签名/公证/安装包policy | Open | desktop/mobile/platform发布不可资格化；定义可选但显式的Sign/Package/Notarize adapters。 |
| **ED-EXPORT-P1-035** 无launcher/install/smoke receipt | Open | file存在即成功；至少执行manifest/loader启动握手和bounded smoke policy。 |
| **ED-EXPORT-P1-036** CookAssets不在core service | Open | preset cook字段没有Runtime stage consumer；迁入Runtime asset/cook graph。 |
| **ED-EXPORT-P1-037** 默认source manifest来自输出目录 | Open | `out/assets/assets.json`不是qualified project closure；消费registry generation + entry/keep/dynamic policy receipt。 |
| **ED-EXPORT-P1-038** Pack无content-addressed cooked handoff | Open | 继承Runtime207：仍缺immutable cooked chunks/manifest与bounded streaming全链。 |
| **ED-EXPORT-P1-039** delta pack只检查两个path同时出现 | Open | 增加base/target schema、project/profile、compression/encryption/chunk compatibility和apply receipt。 |
| **ED-EXPORT-P1-040** ExportPlugin三钩未实现 | Open | 无Runtime begin/file/end contribution、owner lease、platform support、configuration hash和deterministic verdict chain。 |

### 5.6 P1：Job、Cancellation、Durability 与 Output（41-52）

| Finding | 状态 | 当前证据与退出条件 |
|---|---|---|
| **ED-EXPORT-P1-041** wizard/legacy job均只在内存 | Open | 重启不能list/reattach/resume；Runtime journal必须是job truth。 |
| **ED-EXPORT-P1-042** job id不足以去重 | Open | wizard按profile字符串，legacy按process counter；使用OperationId + PayloadDigest + Attempt。 |
| **ED-EXPORT-P1-043** 子进程取消局部可用 | Partial | wizard/Cargo有进程树终止；无stage cancel acknowledgement、deadline和artifact quarantine receipt。 |
| **ED-EXPORT-P1-044** core System runner无cancel/deadline | Open | 直接wait；所有executor必须消费scope cancellation/deadline并reap child。 |
| **ED-EXPORT-P1-045** 两队列无共同resource admission | Open | 可同output并发；建立output lease、CPU/RAM/I/O/process slots和fair scheduling。 |
| **ED-EXPORT-P1-046** 完整日志+有界tail是局部底座 | Partial | full log、BLAKE3、16 KiB line cap、512 tail存在；未与operation receipt关联。 |
| **ED-EXPORT-P1-047** tail逐出已为O(1) | Partial | `VecDeque`修复旧failure；terminal Vec仍有一次drain/insert，但不是当前主P0。 |
| **ED-EXPORT-P1-048** durability barrier阻塞执行链 | Open | stdout、stderr、manifest串行sync；交给bounded Runtime I/O ticket和commit fence。 |
| **ED-EXPORT-P1-049** output manifest字段不足 | Open | 缺command/exit/stage/request/generation/time/truncation policy/environment；无法审计一次执行。 |
| **ED-EXPORT-P1-050** event channel虽有界但live语义错误 | Partial | 192 cap与try_send存在；per-stage count永不递减，第17条后永久coalesce。 |
| **ED-EXPORT-P1-051** 非output event可能阻塞worker | Open | blocking `send`可被不drain的UI卡住；terminal必须走不可丢但有deadline的journal/ack。 |
| **ED-EXPORT-P1-052** session Drop不请求取消或终止 | Open | 只release tool lease；定义detach/cancel/reattach策略和明确terminal ownership。 |

### 5.7 P1：UI Truth、产品接线、测试与性能（53-60）

| Finding | 状态 | 当前证据与退出条件 |
|---|---|---|
| **ED-EXPORT-P1-053** planned artifact与actual混显 | Open | `merged_artifacts`先加入planned，report path也fallback；UI只显示verified receipt。 |
| **ED-EXPORT-P1-054** report正文从bounded stdout解析 | Open | 长输出可截断JSON，失败静默；UI按artifact id分页读取typed report。 |
| **ED-EXPORT-P1-055** target maturity显示不诚实 | Open | scaffold无fatal即Ready；显示Unavailable/ScaffoldOnly/Qualified及缺失capability。 |
| **ED-EXPORT-P1-056** Build/Export pane缓存有局部性能底座 | Partial | source/overlay cache存在；仍需绑定authoritative profile/preset generation和job receipt。 |
| **ED-EXPORT-P1-057** Editor/App/CLI/CI产物等价未证明 | Open | 只有Editor两条不同路径；一个service、同request、byte/receipt parity E2E。 |
| **ED-EXPORT-P1-058** 真实platform E2E矩阵缺失 | Open | 无Windows/Linux/macOS/Android/iOS/Web/Headless build-install-run fixture。 |
| **ED-EXPORT-P1-059** crash/fault/resume矩阵缺失 | Open | 无每个Begin/write/fsync/rename/publish/cancel点kill测试。 |
| **ED-EXPORT-P1-060** 无相对Unreal的统计性能证据 | Open | 建立同项目/硬件/质量的cold/warm/incremental p50/p95/p99、CPU/RSS/I/O/artifact size benchmark。 |

### 5.8 P2

| Finding | 状态 | 当前差距 |
|---|---|---|
| **ED-EXPORT-P2-001** | Open | Export Request/Receipt Inspector不存在。 |
| **ED-EXPORT-P2-002** | Open | stage DAG、critical path与cache lineage可视化不存在。 |
| **ED-EXPORT-P2-003** | Open | platform capability/SDK/template诊断面板不存在。 |
| **ED-EXPORT-P2-004** | Open | content closure解释器不存在，用户不能知道资产为何入包或被排除。 |
| **ED-EXPORT-P2-005** | Open | pack/chunk/dedup/compression统计浏览器不存在。 |
| **ED-EXPORT-P2-006** | Open | deterministic diff与跨机artifact comparison不存在。 |
| **ED-EXPORT-P2-007** | Open | failed/cancelled generation quarantine与清理控制面不存在。 |
| **ED-EXPORT-P2-008** | Open | durable job history、reattach、retry lineage和operator notes不存在。 |
| **ED-EXPORT-P2-009** | Open | codesign/notarization/provisioning credential policy UI不存在。 |
| **ED-EXPORT-P2-010** | Open | patch/base compatibility和rollout channel管理不存在。 |
| **ED-EXPORT-P2-011** | Open | publish destination/provider SPI及promotion/rollback receipt不存在。 |
| **ED-EXPORT-P2-012** | Open | SBOM、license、third-party notice和supply-chain provenance输出不存在。 |
| **ED-EXPORT-P2-013** | Open | crash-point simulator与artifact corruption laboratory不存在。 |
| **ED-EXPORT-P2-014** | Open | Export性能预算、趋势、回归bisect和Unreal对照dashboard不存在。 |

## 6. Canonical 资格门

| Gate | 状态 | 当前裁决 |
|---|---|---|
| `EXPORT-GATE-01` one production authority | Fail | wizard与legacy并存。 |
| `EXPORT-GATE-02` immutable request identity | Fail | 无operation/payload digest。 |
| `EXPORT-GATE-03` profile revision binding | Fail | 仅字符串ref和mode equality。 |
| `EXPORT-GATE-04` project/content generation binding | Fail | 无ProjectSnapshot/registry generation。 |
| `EXPORT-GATE-05` target platform qualification | Fail | core不见target platform。 |
| `EXPORT-GATE-06` target triple/arch/ABI | Fail | build command不传target。 |
| `EXPORT-GATE-07` SDK/toolchain capability receipt | Fail | 只有version文本。 |
| `EXPORT-GATE-08` shared stage vocabulary/topology | Pass | 八阶段DTO与cycle/duplicate/missing dependency拒绝已存在。 |
| `EXPORT-GATE-09` all stages one executor graph | Fail | core仅2/8，其余独立命令。 |
| `EXPORT-GATE-10` declared input verification | Fail | consumed artifacts不被执行器强制检查。 |
| `EXPORT-GATE-11` declared output verification | Fail | expected stdout/produced artifact不被强制检查。 |
| `EXPORT-GATE-12` final export receipt | Fail | 无bundle/package/install/smoke terminal receipt。 |
| `EXPORT-GATE-13` legacy failure propagation | Fail | fatal/Cargo failure仍Ok/Exported。 |
| `EXPORT-GATE-14` core exit propagation | Fail | wizard adapter丢exit code。 |
| `EXPORT-GATE-15` resume schema/currentness | Fail | core report无schema/generation。 |
| `EXPORT-GATE-16` resume corruption fail-close | Fail | malformed JSON静默fresh。 |
| `EXPORT-GATE-17` resume artifact revalidation | Partial | core两阶段有digest recheck，其余stage没有共同合同。 |
| `EXPORT-GATE-18` begin/commit/publish journal | Fail | 单snapshot，无WAL。 |
| `EXPORT-GATE-19` terminal replay/dedup | Fail | 无terminal index和payload conflict。 |
| `EXPORT-GATE-20` cancellation propagation | Partial | wizard/Cargo可取消；core/完整stage receipt不统一。 |
| `EXPORT-GATE-21` child tree termination/reap | Partial | wizard实现，System runner和所有adapter未统一。 |
| `EXPORT-GATE-22` deadline/retry budget | Fail | 无stage/request deadline。 |
| `EXPORT-GATE-23` output-root lease/CAS | Fail | 两队列可同root并发。 |
| `EXPORT-GATE-24` failed generation quarantine | Fail | 旧staged output可被resume复用。 |
| `EXPORT-GATE-25` atomic final publication | Fail | 无完整bundle generation promotion协议。 |
| `EXPORT-GATE-26` parent-directory durability | Fail |多个writer缺共同durable receipt。 |
| `EXPORT-GATE-27` typed diagnostic contract | Partial | error source局部typed，stage/report仍字符串。 |
| `EXPORT-GATE-28` full logs and bounded tail | Partial | 已落盘且tail有界，manifest/commit仍不足。 |
| `EXPORT-GATE-29` live output backpressure correctness | Fail | 16-event counter不随drain恢复。 |
| `EXPORT-GATE-30` terminal event non-blocking durability | Fail | blocking send依赖UI drain。 |
| `EXPORT-GATE-31` actual-only UI artifacts | Fail | planned path被当作artifact展示。 |
| `EXPORT-GATE-32` typed report artifact consumption | Fail | 从stdout tail猜JSON。 |
| `EXPORT-GATE-33` target maturity truth | Fail | scaffold显示Ready。 |
| `EXPORT-GATE-34` product host output | Fail | client构建Hub/Editor/Runtime工具树。 |
| `EXPORT-GATE-35` ABI/BuildSet/dependency qualification | Fail | bundle只is_file/is_dir。 |
| `EXPORT-GATE-36` sign/notarize/package policy | Fail | 无adapter/receipt。 |
| `EXPORT-GATE-37` install/launch/smoke | Fail | 无运行资格证明。 |
| `EXPORT-GATE-38` content closure receipt | Fail | 默认读取out/assets/assets.json。 |
| `EXPORT-GATE-39` deterministic cook recipe | Fail | preset字段未进入Runtime cook graph。 |
| `EXPORT-GATE-40` bounded streaming pack | Partial | Runtime207已部分streaming，最终pack/installer owner仍未闭合。 |
| `EXPORT-GATE-41` patch compatibility/apply receipt | Fail | 仅path pair检查。 |
| `EXPORT-GATE-42` export plugin lifecycle | Fail | begin/file/end和configuration hash不存在。 |
| `EXPORT-GATE-43` Editor/CLI/CI parity | Fail | 共享生产入口不存在。 |
| `EXPORT-GATE-44` restart reattach/history | Fail | job state仅内存。 |
| `EXPORT-GATE-45` crash/fault matrix | Fail | 无kill-point E2E。 |
| `EXPORT-GATE-46` large corpus bounded resources | Partial | inventory/tail局部有界；完整pipeline/pack/job无统一预算证据。 |
| `EXPORT-GATE-47` cross-platform/cross-machine determinism | Fail | 无golden/parity evidence。 |
| `EXPORT-GATE-48` outperform Unreal benchmark | Fail | 无同口径统计artifact。 |

汇总：**40 Fail / 7 Partial / 1 Pass**。Pass仅说明shared stage vocabulary和静态拓扑拒绝成立，不代表任何平台可发布。

## 7. 目标架构

```text
Editor ExportPresetDraft
  -> Runtime ExportRequestBuilder
       bind ProjectSnapshot / AssetRegistryGeneration
       bind ExportProfileRevision / PlatformAdapterRevision
       bind CompiledPluginPlan / BuildSet / Toolchain / EnvironmentPolicy
       qualify target platform + triple + arch + SDK + signing capability
  -> Immutable ExportRequest(OperationId, PayloadDigest, Generation, Attempt)
  -> Runtime ExportService.compare_or_begin(request)
  -> Durable BuildGraphJournal
       Qualify
       ResolveContentClosure
       CompileTarget
       CookArtifacts
       AssemblePack
       StagePlatformBundle
       SignPackagePublish
       InstallLaunchSmoke
       FinalizeReport
  -> each node emits StageReceipt
       input/output CAS refs + digests
       target/toolchain/environment identities
       started/finished/resource metrics
       diagnostics + cancel/retry/reuse disposition
  -> Final ExportReceipt(bundle/package/publish/smoke policy)

Editor / App commandlet / CI
  -> the same Runtime ExportService facade
  -> project job snapshots and verified receipts only
```

关键不变量：

1. 相同 `OperationId + PayloadDigest`只能执行一次；不同payload复用operation必须返回conflict；terminal请求只能replay receipt。
2. target platform/triple/arch、profile revision、project/content generation、BuildSet、plugin plan、toolchain和environment policy必须进入request digest。
3. stage只有在所有declared output被materialize、验证并发布后才能Passed；expected/planned path永远不是artifact。
4. failed/cancelled generation进入quarantine，未提交artifact不能被后续resume当作成功输出。
5. Editor、CLI、CI不得各自解释成功；只有Runtime `ExportReceipt`是terminal truth。
6. Mobile/browser target在platform adapter未qualified前必须是Unavailable/ScaffoldOnly，不能是Ready。
7. Cook输出是content-addressed immutable chunks + manifest；Pack、delta、bundle只消费这些artifact，不重读raw source建立第二套truth。
8. UI只能投影verified receipt和typed diagnostics；完整报告按artifact读取，不能从tail猜JSON。

## 8. 分层重构计划

### M0：先封闭假成功和并发破坏

- 删除或禁用legacy生产Execute路径；若迁移期仍需保留，只能作为同一Runtime service facade的薄adapter，不得拥有自己的manager/queue/report。
- 修正CompileHost runner contract，nonzero exit在写Passed/core report前终止；检测现有core/wizard冲突report并quarantine。
- 对八阶段强制验证declared inputs、expected stdout keys和produced artifacts；UI移除planned-as-actual fallback。
- 对output root建立operation lease和generation staging root；同root并发必须Busy/Conflict。
- Mobile/browser scaffold改为Unavailable/ScaffoldOnly，直到对应platform adapter通过资格门。

### M1：Runtime owner与immutable request硬切

- 把pipeline、stage registry、inventory、resume journal、artifact receipt与platform adapter移到Runtime；Interface只保留中立DTO。
- 建立`ExportRequestBuilder`，解析preset draft并冻结ProjectSnapshot、profile revision、plugin plan、BuildSet、target/toolchain/environment。
- report升级为versioned `StageReceipt/ExportReceipt`；diagnostics升级为typed records。
- Editor wizard、App commandlet、CI全部调用一个Runtime facade；删除Editor cooked truth和重复status authority。

### M2：Content closure、Cook 与 Pack闭环

- 从entry scenes、keep list、dynamic policy、customized file和plugin contribution构建qualified content closure。
- Cook action key包含source closure、import/cook recipe、target platform/quality、toolchain和function identity；输出CAS chunks和manifest。
- Pack writer只stream immutable chunks，设定in-flight bytes/I/O slots；delta按chunk table复用unchanged content并发布apply receipt。
- 加入asset为何入包/排除的provenance和license/SBOM handoff。

### M3：Platform adapter与发布闭环

- 每个平台实现qualification、compile/link、layout、dependency scan、manifest/icon、sign/package/install/run/smoke adapters。
- Desktop先建立Windows/Linux/macOS target triple和launcher smoke；Headless独立server policy；Android/iOS/Web不借用host layout。
- final success由policy决定：bundle-only、signed package、installed smoke或published promotion均产生不同typed disposition。

### M4：Durable jobs、恢复与可观测性

- 建立append-only journal、attempt/generation、lease fence、cancel acknowledgement、retry lineage、terminal index与restart reattach。
- output/log/report交给bounded Runtime persistence lane；terminal event写journal后UI异步观察，不由UI channel承担durability。
- 记录stage critical path、cache hit/miss、bytes read/written/hashed/copied、CPU/RSS、queue/backpressure和artifact sizes。

### M5：资格测试与性能

- 夹具项目覆盖scene closure、动态资产、native plugin、shader/cook variant、pack/delta、每个平台template和smoke handshake。
- 对每个journal/publish点做kill/fault/cancel/restart矩阵；覆盖corrupt/partial/stale output、PID reuse、same-output concurrency和disk-full。
- 验证Editor/CLI/CI同request产生相同receipt与byte-equivalent artifacts。
- 用固定项目/硬件/质量对比Zircon与Unreal的cold、warm、one-asset incremental、packaging和launch；只有p50/p95/p99、CPU/RSS/I/O/artifact-size均有source-bound证据后才能提出“优于Unreal”。

## 9. 禁止的临时方案

- 不得保留两套生产pipeline并用UI文案区分。
- 不得把nonzero exit、fatal diagnostics、缺artifact降级为普通warning或`Ok(report)`。
- 不得以planned path、旧文件存在或host OS文件名作为target artifact证明。
- 不得把platform继续作为显示字符串传递，或用Android/iOS/Web profile复用当前host layout。
- 不得新增test-only bypass、silent fallback、兼容alias或第三份report truth。
- 不得用更大channel/tail/RAM掩盖错误backpressure和全包owner。
- 不得把journal/fsync搬到无界私有线程；队列、bytes、age、deadline和shutdown必须有预算。
- 不得以version字符串代替toolchain binary/SDK/environment identity。
- 不得在没有同口径统计artifact时声明性能优于Unreal。

## 10. 本轮验证与未执行项

本轮为review-only，只写本报告及索引/coverage记录。已静态核对当前工作树103个Editor生产文件、49个Runtime/Interface/App交接文件、16个参考文件，并复核Editor15计划与三个open failure。未修改production Rust、Cargo、ABI、ZUI、测试或Tooling；未运行Cargo、Editor、真实export/cook/pack/platform bundle、fault、cross-platform、scale、soak或benchmark。按用户要求未查询、轮询、等待或实时跟踪协调器。
