---
related_code:
  - tools/mvp/Build-MvpProductInputs.ps1
  - tools/mvp/Build-RenderExtractProfilingInputs.ps1
  - tools/mvp/Capture-RenderExtractBaseline.ps1
  - tools/mvp/Invoke-MvpAcceptance.ps1
  - tools/mvp/MvpAcceptanceNativeFileSystem.psm1
  - tools/mvp/MvpAcceptanceSnapshotAdmission.psm1
  - tools/mvp/MvpAcceptanceStagingProjection.psm1
  - tools/mvp/MvpAcceptanceStagingSnapshot.psm1
  - tools/mvp/MvpAcceptanceStagingTreeManifest.psm1
  - tools/mvp/MvpArtifactStoragePolicy.psm1
  - tools/mvp/MvpBuildSummaryEvidence.psm1
  - tools/mvp/MvpBuildGateRegistry.psm1
  - tools/mvp/MvpPersistenceComparison.psm1
  - tools/mvp/MvpProcessTimingEvidence.psm1
  - tools/mvp/MvpProductInputManifest.psm1
  - tools/mvp/MvpProductSourceIdentity.psm1
  - tools/mvp/MvpProductProfileRegistry.psm1
  - tools/mvp/MvpProjectOpenEvidence.psm1
  - tools/mvp/MvpProjectSaveEvidence.psm1
  - tools/mvp/MvpScenePersistenceEvidence.psm1
  - tools/mvp/MvpStagingPreflight.psm1
  - tools/mvp/MvpStagingPreflightEvidence.psm1
  - tools/mvp/MvpStagingRelease.psm1
  - tools/mvp/MvpTestFixturePaths.psm1
  - tools/mvp/New-RenderExtractScaleProject.ps1
  - tools/mvp/New-ResourceManagementBaselinePlan.ps1
  - tools/mvp/New-ResourceManagementScaleProject.ps1
  - tools/mvp/ResourceManagementComparison.psm1
  - tools/mvp/ResourceManagementExecutionProtocol.psm1
  - tools/mvp/ResourceManagementJsonEvidence.psm1
  - tools/mvp/ResourceManagementObservationContext.psm1
  - tools/mvp/ResourceManagementSchema.psm1
  - tools/mvp/ResourceManagementSchemaRegistry.psm1
  - tools/mvp/ResourceManagementStatistics.psm1
  - tools/mvp/ResourceManagementWorkloadRegistry.psm1
  - tools/mvp/RenderExtractBaselineEvidence.psm1
  - tools/mvp/RenderExtractBaselineMetrics.psm1
  - tools/mvp/RenderExtractFrozenInput.psm1
  - tools/mvp/RenderExtractProcessJob.psm1
  - tools/mvp/RenderExtractSourceIdentity.psm1
  - tools/mvp/ResourceManagementScaleInventory.psm1
  - tools/mvp/Set-ResourceManagementScaleProjectChangeSet.ps1
  - tools/mvp/Stage-MvpProducts.ps1
  - tools/mvp/Write-RenderExtractBaselineReport.ps1
  - tools/mvp/Write-ResourceManagementBaselineReport.ps1
  - tools/mvp/Write-ResourceManagementComparisonReport.ps1
  - tools/mvp/mvp-authoring-automation.json
  - tools/mvp/mvp-artifact-storage-policy.json
  - tools/mvp/mvp-build-gate-registry.json
  - tools/mvp/mvp-product-profile-registry.json
  - tools/mvp/mvp-reopen-automation.json
  - tools/mvp/resource-management-schema-registry.json
  - tools/mvp/resource-management-workload-registry.json
  - .github/workflows/mvp-editor-windows.yml
tests:
  - tools/tests/Invoke-MvpAcceptanceTestDriver.ps1
  - tools/tests/mvp_editor_windows_workflow.Tests.ps1
  - tools/tests/mvp-acceptance-staging-snapshot.Tests.ps1
  - tools/tests/mvp-acceptance.Tests.ps1
  - tools/tests/mvp-artifact-storage-policy.Tests.ps1
  - tools/tests/mvp-product-build.Tests.ps1
  - tools/tests/mvp-product-inputs.Tests.ps1
  - tools/tests/mvp-required-script-contracts.Tests.ps1
  - tools/tests/mvp-staging-editor-runtime-library.Tests.ps1
  - tools/tests/mvp-staging-release.Tests.ps1
  - tools/tests/mvp-staging.Tests.ps1
  - tools/tests/mvp-test-fixture-paths.Tests.ps1
  - tools/tests/render-extract-baseline-capture.Tests.ps1
  - tools/tests/render-extract-baseline-report.Tests.ps1
  - tools/tests/render-extract-profiling-inputs.Tests.ps1
  - tools/tests/render-extract-scale-project.Tests.ps1
  - tools/tests/resource-management-baseline-plan.Tests.ps1
  - tools/tests/resource-management-baseline-report.Tests.ps1
  - tools/tests/resource-management-comparison.Tests.ps1
  - tools/tests/resource-management-scale-project.Tests.ps1
plan_sources:
  - docs/plans/mvp/06-f5-acceptance-wave.md
  - docs/plans/mvp/06/failure-2026-08-01-f5-evidence-package-incomplete.md
  - docs/plans/performance/01/2026-08-14-resource-query-metrics-ownership-and-index-gate.md
  - docs/plans/optimize/zircon_app/07-renderable-empty-project-template-create-import-render-export-evidence-product-integration-review.md
  - docs/plans/optimize/zircon_tooling/07-performance-benchmark-profile-capture-symbol-crash-evidence-baseline-review.md
  - docs/plans/optimize/zircon_tooling/09-release-channel-artifact-repository-install-update-rollback-operations-review.md
  - docs/plans/optimize/zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md
  - docs/plans/optimize/zircon_tooling/12-top-level-acceptance-archive-serialization-fixture-provenance-currentness-migration-review.md
  - docs/plans/optimize/zircon_tooling/13-repository-codex-skill-hook-structural-audit-governance-security-currentness-review.md
  - docs/plans/optimize/zircon_tooling/14-editor-workbench-design-spec-screenshot-export-visual-evidence-prototype-governance-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Programs/UnrealBuildTool/System/TargetReceipt.cs
  - dev/UnrealEngine/Engine/Source/Programs/AutomationTool/BuildGraph/BuildGraph.cs
  - dev/UnrealEngine/Engine/Source/Programs/AutomationTool/BuildGraph/TempStorage.cs
  - dev/UnrealEngine/Engine/Source/Programs/AutomationTool/BuildGraph/Tasks/CreateArtifactTask.cs
  - dev/UnrealEngine/Engine/Source/Programs/AutomationTool/Gauntlet/Framework/Base/Gauntlet.AppConfig.cs
  - dev/UnrealEngine/Engine/Source/Programs/AutomationTool/Gauntlet/Framework/Base/Gauntlet.AppInstance.cs
  - dev/UnrealEngine/Engine/Source/Programs/AutomationTool/Gauntlet/Framework/Base/Gauntlet.TargetDevice.cs
  - dev/UnrealEngine/Engine/Source/Programs/AutomationTool/Gauntlet/Framework/Base/Gauntlet.TestReport.cs
  - dev/UnrealEngine/Engine/Source/Programs/AutomationTool/Gauntlet/Framework/Utils/Gauntlet.ScreenshotCompare.cs
  - dev/Graphics/TestProjects/PostProcessing_Tests/Assets/CommonAssets/Scripts/PostProcessingGraphicsTests.cs
  - dev/Graphics/TestProjects/PostProcessing_Tests/Assets/CommonAssets/Scripts/PostProcessingGraphicsTestSettings.cs
  - dev/Graphics/.yamato/postprocessing-win-dx12.yml
  - dev/Graphics/.yamato/postprocessing-linux-vulkan.yml
  - dev/bevy/tools/example-showcase/src/main.rs
  - dev/bevy/.github/workflows/send-screenshots-to-pixeleagle.yml
  - dev/godot/main/main.cpp
  - dev/godot/tests/test_main.cpp
  - dev/Fyrox/fyrox-build-tools/src/build.rs
  - dev/Fyrox/fyrox-build-tools/src/export/mod.rs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: partial_mvpctl_p0_004_p0_005_control_plane
source_recheck_required: true
---

# Tooling 15：MVP BuildSet、产品进程、验收证据与资源基线控制面审查

## 1. 结论

`tools/mvp`已经超过“临时验收脚本”的规模：33个tracked文件、14,789行、675,385 bytes，其中31个PowerShell source/module、2个automation JSON；对应17个直接相关测试文件共约9,012行。它同时承担产品构建、输入清单、项目复制、editor/runtime启动、超时清理、日志、截图、场景持久化断言、F5归档、render-extract性能采集，以及resource-management基线计划和报告。这个控制面已经是事实上的Build/Test/Artifact Service，却仍以一组互相import的本机PowerShell脚本存在。

本轮必须先区分“强基础”与“结论不成立”。`MvpAcceptanceNativeFileSystem`、staging snapshot/projection/tree-manifest实现了Windows no-follow handle、volume/file/creation identity、ancestor lease、reparse拒绝、copy后验证、partial publication和按句柄rename；acceptance归档先锁住source tree，再复制、核对manifest并原子发布。RenderExtract还有Job Object、frozen input和timeout。这些机制不应删除，而应成为统一Artifact/Evidence Service的Windows backend。

但其上层信任链仍断裂。产品构建只在可变工作树上反复采样fingerprint，并不从不可变source snapshot构建；fingerprint还把全部unignored untracked文件算作source，本轮实际包含2,373个文件、1,680,960,238 bytes，绝大多数来自`tools/`生成目录。两次无修改顺序采样分别花13.228秒和11.318秒，aggregate test期间又会被并发无关输出改变。一个文件在构建期间发生A→B→A变化仍可能通过前后采样，不同产品可由不同瞬时source产生却共享同一最终fingerprint。

产品运行与结果Oracle也没有分权：runtime/editor自己写diagnostic marker与PNG，Stage解析这些marker、检查PNG非透明且至少100个像素不同于左上角，Invoke再验证同一批自报内容的hash和固定值。一个能输出预期字符串和非空PNG的错误产品可以自我认证。resource-management链更严重：仓库有1/1K/100K项目生成器、change-set和reporter，却没有任何tracked产品观测执行器。此前reporter会把结构匹配的caller JSON直接标为`measurement_status = measured`。2026-08-24已改为fail-closed的`measurement_status = unverified`与`measurement_status_reason = no-trusted-observation-producer`：统计只可用于诊断，不能被视为性能基线或对外性能数据。

2026-08-16在PowerShell 7.4.18/Pester 3.4.0上运行14个focused文件，结果84 total、81 passed、3 failed、265.235秒。三个失败均暴露测试控制面脆弱性：`mvp-acceptance.Tests.ps1`的跨行错误regex不兼容当前格式；两个resource测试分别被Pester 3的`Should Throw`参数绑定行为和StrictMode下访问缺失property击中，直接探针确认production拒绝/变更行为符合其局部意图。`mvp-staging.Tests.ps1`单独exit 0但Pester报告`TotalCount = 0`，因为它实质是脚本级自定义测试。31个PowerShell文件AST解析0 error，但required workflow没有运行这些控制面suite，也没有运行resource或render-extract focused suite。

`docs/plans/mvp/06-f5-acceptance-wave.md`仍标记`blocked_by_f4`，所有验收checkbox未完成，并明确说明需要clean coordinator run与真实上传artifact检查；当前workspace/editor编译又有既存阻断。因此历史artifact或局部driver测试不得升级为current F5 qualification。本轮登记 **6项P0、60项P1、14项P2**；已实施MVPCTL-P0-004的fail-closed报告状态，以及MVPCTL-P0-005的场景环境策略、Windows Job Object containment、共享有界stdout/stderr、进程数/内存/CPU限额、可保留的lifecycle journal、取消/非零退出crash/进度事件。受信产品观测执行器、磁盘与文件配额、产品语义化liveness、crash dump/symbol linkage和current qualification仍未实现。

## 2. 物理清单与当前证据

### 2.1 所有权与规模

| 物理面 | 规模 | 当前职责 | 结论 |
| --- | ---: | --- | --- |
| `tools/mvp` | 33 files / 14,789 lines / 675,385 bytes | build、stage、process、evidence、performance、resource baseline | 单目录混合六个控制域 |
| PowerShell | 31 files | 39处`schema_version`、18处`ConvertFrom-Json`、24处whole-file read、20处硬编码approved drive | schema、IO、root policy分散手写 |
| automation JSON | 2 files / 7 bindings | 固定点击、数值输入、保存/重开 | 无schema/version/scenario/expected state |
| 直接测试 | 17 files / 约9,012 lines | Pester与脚本级failure probes | test runner与结果协议不统一 |
| Windows workflow | 1 file | F0-F5 Cargo gate、build、stage、accept、上传 | 未要求`tools/mvp` focused suites |
| 当前untracked source集合 | 2,373 files / 1,680,960,238 bytes | 被source fingerprint逐个hash | owner不受控，包含无关生成物 |

33个tool中有23个在此前118篇专项报告的frontmatter从未被逐路径纳入；RenderExtract主链已由Tooling07审查，release/staging publication与Tooling09重叠。本篇只把它们作为统一控制面的边界证据，不重复登记既有性能真实性或release channel finding。

### 2.2 动态验证

| 检查 | 结果 | 可证明/不可证明 |
| --- | --- | --- |
| 31个PS1/PSM1 AST parse | 31/31，0 parse error | 语法可加载；不证明运行语义 |
| 2个JSON parse | 2/2 valid | JSON合法；均无`schema_version` |
| source fingerprint顺序复算 | hash一致；13.228秒、11.318秒 | 静止工作树可重算；代价高且不形成snapshot |
| aggregate focused suite | 84 total / 81 pass / 3 fail / 265.235秒 | 当前控制面整体不是GREEN |
| `mvp-staging.Tests.ps1`单跑 | exit 0 / 104.282秒 / Pester TotalCount 0 | 自定义断言可能执行，但runner无法计数、筛选和归档case |
| `mvp-acceptance.Tests.ps1`单跑 | exit 1 / 20.545秒 | production正确拒绝非法root，test regex被换行格式击中 |
| resource direct probes | invalid count正确拒绝；4 assets/25%只改1项且无sidecar | 两个aggregate失败是harness兼容问题，不是把production误报为坏 |
| required workflow检索 | 0个mvp/resource/render-extract focused suite调用 | 合并门不保护控制面自身 |

没有重跑已知必败的完整Cargo/product acceptance：`zircon_editor --lib`当前已有239个compile errors，F5计划也未完成clean validation前置；重复运行不能增加新证据，且会写大量build/staging artifact。

### 2.3 应保留的工程机制

1. acceptance source tree使用no-follow native handles、directory lease和identity tuple阻止reparse替换、rename/delete竞态。
2. source snapshot、partial evidence tree、projection与tree manifest在发布前后重新验证，最终目录用no-overwrite rename提升。
3. artifact写入多处使用`CreateNew`与flush，避免静默覆盖既有证据。
4. product input manifest固定四个logical product的package/bin/features/output group并验证byte size/SHA-256。
5. project路径通过共享Windows resolver归一化，产品边界仍传`--project .`，没有把物理staging路径变成项目虚拟URI。
6. process timing、project open/save、scene snapshot和persistence comparison已拆出局部validator，而不是只检查exit 0。
7. RenderExtract已经有Job Object、frozen input、bounded timeout和独立reporting primitives，可作为统一runner的迁移样板。

这些能力说明正确方向不是重写成另一个更大的脚本，而是提炼稳定的typed control-plane service与backend。

## 3. P0：先恢复可声明真实性的边界

### MVPCTL-P0-001 · Source fingerprint不是不可变BuildSet

`Get-MvpSourceFingerprint`把HEAD、tracked raw diff、变更tracked content hash、全部unignored untracked path与content hash拼成SHA-256；`Build-MvpProductInputs`只在每个build前后重新计算并比较。它没有checkout、filesystem snapshot、content-addressed input projection或build lease，因而不能阻止A→B→A瞬态变化，也不能保证四个product从相同bytes构建。当前2,373个untracked文件/1.68 GB还使任意session artifact进入全局source identity；aggregate执行时fingerprint已因无关并发输出漂移。必须建立immutable `BuildSetId`：由明确allowlist的repo tree、submodule/LFS/materialized dependency、toolchain/config/env closure组成，builder只能读snapshot/CAS，不得从活动工作树取源。

### MVPCTL-P0-002 · Product receipt不能证明artifact由声明配置构建

build manifest只有source fingerprint、package/bin/features/output group、artifact path/bytes/hash。实际执行依赖`.codex/skills/zircon-dev/scripts/validate-matrix.ps1`和`powershell.exe -ExecutionPolicy Bypass`；没有记录真正的Cargo/rustc hash、target triple、linker/Windows SDK、profile/codegen flags、Cargo graph、environment、runtime dependencies、symbols或producer identity。Stage随后记录“当前`rustc -Vv`”，它可能不是产生binary的toolchain。必须让build owner直接产生签名`ProductReceipt`，完整绑定BuildSet、ToolchainSet、TargetProfile、BuildAction、BuildProducts、RuntimeDependencies、symbols/SBOM和producer，而不是让stager事后猜 provenance。

### MVPCTL-P0-003 · 产品同时充当被测对象、证据生产者和Oracle

runtime/editor自行写marker、diagnostic summary和PNG；Stage从产品输出解析预期字符串，Invoke再比较同一自报字段。PNG只检查尺寸、可见像素、相对左上角背景至少100个不同像素以及hash；before/after只要求pixel hash不同。错误实现可以生成约定文本与任意非空图而通过。必须把Oracle移到独立observer：从外部窗口/进程/telemetry transport采集，使用版本化事件、approved scene state、semantic probes与独立visual baseline/diff；产品自报只能是一个不可信signal，不能单独决定pass。

### MVPCTL-P0-004 · Resource reporter可把任意caller JSON铸造成“measured”

仓库没有生成observation manifest的产品runner；git全仓只找到plan generator、project/change generator、reporter、tests和计划文档。此前`Write-ResourceManagementBaselineReport.ps1`验证caller提供的schema、source fingerprint、plan hash、scenario、attempt和counter形状后，固定输出`measurement_status = measured`。它不绑定product receipt、process identity、trace/session、collector、machine、toolchain、cache state或签名，测试fixture正是合成任意counter后得到measured report。2026-08-24已把输出改为fail-closed的`measurement_status = unverified`和`measurement_status_reason = no-trusted-observation-producer`：结构合法的观察值仍可汇总用于诊断，但不能成为性能资格或对外性能数据。该变更不实现受信`ObservationProducer`；仍须由其run receipt绑定真实product/trace/frame，reporter才能聚合经过producer签名和schema验证的observation，并允许升级为`measured`。

### MVPCTL-P0-005 · Stage没有可靠的进程树 containment 与有界IO

2026-08-24前，Stage用普通`Diagnostics.Process`启动产品，stdout/stderr用`ReadToEndAsync`无限积累内存，完成后才整体写日志；超时依赖`taskkill /PID /T`和扫描`Win32_Process.ExecutablePath`是否位于staging目录。现已将生命周期抽到`StagedProcessSupervisor.psm1`并复用`RenderExtractProcessJob`：在进程恢复前完成Job assignment和capture端点建立，timeout/cancel以Job终止，根进程退出后以有界轮询的Job active-process accounting拒绝并清理仍在运行的后代；Job限制最多8个active process、4 GiB总内存和75% CPU hard cap。stdout/stderr从byte stream流式写入`CreateNew`前缀文件，共享一次run级retained-byte预算，并并行建立受64 KiB单流上限的tail artifact。journal记录start/heartbeat/progress/cancellation/exit/crash/cleanup/terminal，所有事件绑定RunId、PID+creation identity、executable hash、cwd、argument/environment digest、environment policy、resource limits、event sequence/hash chain；active journal以1 MiB上限轮换为immutable segment，hash chain与cursor跨segment连续，最多保留64个archive segment，裁剪前将段范围、数量和聚合SHA写入新事件。产品diagnostic限制为64个文件、8层目录、每文件1 MiB、每次聚合4 MiB并在超限时fail-close。该修复覆盖外部路径后代、无界内存、缺少中间liveness、无资源上限和journal无限增长的当前风险，但仍缺磁盘/文件配额、产品语义化startup/frame/save/hang classifier、crash dump/module/symbol/ProductReceipt linkage和受信run qualification；这些项完成前，Stage不得被视为完整qualification证据。

### MVPCTL-P0-006 · 当前F5与性能基线没有可发布的current qualification

F5计划仍为`blocked_by_f4`，所有M6.1-M6.4和退出checkbox未完成，并明确要求clean coordinator workflow与真实artifact检查；当前editor/workspace编译存在既有阻断。控制面focused aggregate又是81/84而非GREEN，workflow不运行这些suite，resource lane没有observer。必须把`planned/implemented/tested/qualified/promoted`分开；在immutable BuildSet、required control-plane suites、真实product run、independent evidence和promotion receipt全部通过前，任何manifest、计划或UI不得显示current F5 accepted或measured baseline。

## 4. P1：工程化重构清单

### 4.1 BuildSet、ProductReceipt与输入发布

| ID | 差距 | 当前证据 | 目标合同 |
| --- | --- | --- | --- |
| MVPCTL-P1-001 | production build仍依赖repo-local Codex skill | 2026-08-26复核确认`Build-MvpProductInputs`从BuildSet内解析`.codex/skills/zircon-dev/scripts/validate-matrix.ps1`；RenderExtract input builder和`tools/build-editor.ps1`也直接引用同一路径。validator、其107 KiB tests和三个caller当前均有其他owner未提交改动，本会话未跨写迁移 | 稳定的workspace CLI/service拥有build；skill只调用公共入口。迁移必须由validator owner做一次实现硬迁移+skill wrapper并同步现有dot-source tests，不能只加一层仍依赖skill实现的假workspace wrapper |
| MVPCTL-P1-002 | MVP production物理root授权、Stage volume probe、acceptance/terminal receipt重验与运行叶级namespace已收敛，product绑定仍未完成 | 32 KiB受限、严格UTF-8 policy集中3个D/E/F approved roots、默认root、capability class与11个namespace；product、profiling、resource、RenderExtract以及Stage/acceptance/fixture均消费typed policy。Stage保留显式unsafe测试开关，acceptance与Coordinator-issued fixture path按approved-root capability授权且继续校验PID/lease相对路径。production扫描仅策略模块自身保留1处root authority，其他物理root/regex为0；root迁移契约先RED 5/7后GREEN 7/7，相关7文件AST全通过。共享真实I/O capability API要求fixed local NTFS/ReFS、检查required/available free space、在唯一临时目录执行`WriteThrough + Flush(true)`及same-directory move、校验published bytes并清理，返回值绑定policy/root identity；Stage批准根在input publication前消费并把evidence写入manifest/返回对象，unsafe测试路径显式为null。acceptance新增exact-field validator，以原始Stage root而不是detached snapshot path重验current policy receipt、root/drive/filesystem、required/available bytes、两项capability真值和UTC，并把已验证evidence保留到最终manifest/result；terminal receipt升级schema v3，对三个经过preflight的终态路径重验同一Stage root、required bytes与capability evidence，published终态缺证据时fail closed，admission阶段保持显式null。policy contract经过探针API RED 7/8→GREEN 8/8、Stage绑定RED 7/8→GREEN 8/8、validator RED 8/9→GREEN 9/9、acceptance绑定RED 8/9→GREEN 9/9；terminal契约新增第6个缺证据拒绝case并通过轻量AST/调用点守卫。新增`mvp-staging-runs`、`mvp-acceptance-evidence`、`mvp-test-fixtures`三个互不重叠的leaf authority；Stage安全路径只允许`mvp-f0-*`，acceptance按Staging/Evidence角色分别允许production namespace或Coordinator fixture namespace，fixture create/remove及parent均重验namespace。namespace契约RED 6/9→GREEN 9/9，7文件AST通过，production调用计数1/1/3。此前四个干净批次policy+scale 11/11、capture 26/26、baseline report 30/30、BuildSet 17/17，共84/84；同时修复PowerShell 7 JSON UTC `DateTime`反序列化和`\\?\` BuildSet child path分隔符 | approved-root authorization、Stage probe、acceptance/terminal重验与运行叶namespace已完成；下一步只把已验证evidence绑定真实ProductReceipt |
| MVPCTL-P1-003 | product input已具备临时root、原子成功发布与独立abort终态 | 完整输入只在staged root内构建并验证；成功时以no-follow operational path原子move到空target，失败时以同目录临时文件原子发布唯一`<artifact>.aborted.json`同级收据，既有成功目录或首个abort均拒绝覆盖；abort只记录输出叶名、失败分类、消息长度和最多4096字符前缀的SHA-256，不暴露原始消息或绝对路径 | 已完成当前per-action publication边界；后续统一Artifact Service必须保留CreateNew、durable flush、同级终态和first-writer语义 |
| MVPCTL-P1-004 | build receipt仍缺production ToolchainSet签发 | 2026-08-27只读审计发现其他owner worktree中的未跟踪`tools/cargo-zircon/src/build/receipt`已定义cargo/rustc/linker SHA、SDK/environment digest、target/profile/codegen/Cargo graph、action、products/dependencies/symbols/SBOM和attestation原语，但目前没有production build命令签发，也未接入`Build-MvpProductInputs`；相关Cargo入口、validator与builder均有该owner未提交修改，本会话未跨写，fixture receipt不得提升qualification | 由build owner完成production issuer、真实toolchain/action采集和builder/Stage verifier接线后，toolchain、SDK、target、profile和action digest才能进入受信receipt identity |
| MVPCTL-P1-005 | build product closure只含四个主文件 | 无DLL依赖、PDB、shader/assets、licenses、plugins或runtime data | typed BuildProducts + RuntimeDependencies + optional/debug products |
| MVPCTL-P1-006 | producer与receipt无认证关系 | JSON由driver本地写，只有内容hash | producer build ID、worker/session、签名/attestation与trusted timestamp |
| MVPCTL-P1-007 | product、RenderExtract、resource与staging source identity已统一到BuildSet，owner policy与产品性能收据仍待完成 | ProductInputManifest与RenderExtract profiling manifest的兼容`source_fingerprint`均直接使用已自验证BuildSetId。RenderExtract profiling builder/scale generator/最小repeat=3 capture链路的active-checkout扫描从1/1/15降到0；resource generator/change-set从1/1降到0，模板只复制BuildSet manifest登记文件；staging删除缺BuildSet时的1次条件fallback并在admission fail closed。resource project/change-set schema v2同时绑定`source_fingerprint == build_set_id`和ProductInputManifest SHA。所有production调用点、旧API export及git diff/ls-files实现均已删除；修改范围5套合同65/65且workflow contract通过。历史2,373文件/1,680,960,238 bytes的单次legacy scan为11.318-13.228秒，仅作已移除工作量参考，不是本轮产品P50/P95 | 为untracked/generated input定义owner allowlist与fail/ignore审计receipt；以ProductReceipt绑定的真实产品采集补齐P50/P95，未取得该收据前不得声明性能达标 |
| MVPCTL-P1-008 | BuildSet已物化tracked closure并拒绝不完整外部source，generated/env仍缺失 | schema v1列出HEAD revision、tracked dirty overlay digest及每个materialized tracked file的path/SHA/bytes；发布/消费均复验，submodule、symbolic link和未物化LFS pointer fail closed。Git二进制stdout capture现返回Windows PowerShell兼容的typed `(buffer, validLength)` carrier，消费者按有效长度解码，消除`MemoryStream.ToArray()`的完整复制；4 MiB capture合成50次由237.8 ms/209,964,792 allocated bytes降至5.2 ms/168,136 bytes（-97.8% elapsed）。UTF-8 encoder改为module级单实例，供index decode、BuildSetId、JSON writer和manifest reader复用；100,000次合成调用由2,957.6 ms/219,413,280 bytes降至1,628.2 ms/164,856,096 bytes（-44.9% elapsed）。NUL cursor scan候选在10,000 records x3中由704.5 ms/55,588,344 bytes退化至1,110.2 ms/75,270,504 bytes（+57.6% elapsed），已拒绝并恢复`Split`实现。Wave141把child resolver拆为兼容wrapper与normalized内核，tracked/manifest caller各只冻结一次root/prefix；10,000 paths x3由6,168.8 ms/1,493,679,608 bytes降至5,922.3 ms/1,465,259,408 bytes（-4.0% elapsed）。snapshot traversal直接对枚举器`FullName`执行共享prefix containment/substr，删除每文件双`GetFullPath` helper；10,000 paths x3由1,639.6 ms/94,109,608 bytes降至690.2 ms/43,709,632 bytes（-57.9% elapsed）。Wave142把manifest/file-entry expected property names冻结为module级string arrays，exact-property validator以scalar count和小型ordinal嵌套比较直接遍历PSProperty集合，消除每file entry的expected/actual/unknown数组、pipeline与HashSet；10,000 entries由3,778.6 ms/259,278,880 bytes降至932.0 ms/64,358,120 bytes（-75.3% elapsed）。Wave143以module级regex检查空/`.`/`..` segment并复用normalized relative path，10,000 paths x3由5,778.7 ms/1,449,522,832 bytes降至2,605.7 ms/133,254,432 bytes（-54.9% elapsed）；inventory成功路径复用验证阶段已有ordinal HashSet并调用`SetEquals`，取消完整expected projection与双NUL join，10,000 paths x3由186.4 ms/56,148,376 bytes降至6.6 ms/104,072 bytes（-96.5% elapsed）。逐PSObject索引候选虽减分配但耗时退化130.6%，已拒绝。以上均为工具合成数据而非ProductReceipt P50/P95；allocation/path/schema/inventory合同8/8、Pester4相邻四-suite聚合67/67通过。未声明generated inputs、env allowlist、submodule/LFS materialization receipt | 扩展BuildSet为git tree/object、允许的submodule/LFS/generated input及环境receipt；外部source必须物化并进入identity，不能只从当前拒绝状态推导完整支持；后续不得恢复Git stdout exact-array copy、每调用encoder构造、逐文件root规范化、traversal relative helper、逐entry property集合分配、path segment split pipeline或inventory全量join |
| MVPCTL-P1-009 | build无admission/cancel/resource budget | `Build-MvpProductInputs`仍顺序同步调用四个外层`powershell.exe`；受保护validator没有timeout/cancellation参数，也不返回action receipt。既有`StagedProcessSupervisor`强绑定StageRoot、RunId、qualification context与stage journal，不能伪造这些事实后复用 | validator/coordinator owner提供通用ActionSupervisor：queue/cancel/timeout、CPU/RAM/disk/process预算、外层进程树终止和签名operation receipt；builder随后只消费该receipt |
| MVPCTL-P1-010 | 当前Windows MVP product规格已注册化，跨平台产品闭包仍待扩展 | production module内四个规格对象已从4降为0；一个64 KiB受限、严格UTF-8、版本化registry声明2个TargetProfile/Role/Configuration profile和4个typed products，feature保持token数组。builder从BuildSet snapshot只解析一次并生成matrix；ProductInputManifest schema v2绑定registry SHA-256/size receipt，Stage拒绝当前authority漂移 | 已完成当前Windows development exe/dll matrix；继续注册release/profiling、其他platform、RuntimeDependencies/debug/optional products，并由P1-004 action receipt绑定实际target/toolchain |
| MVPCTL-P1-011 | staging记录的toolchain可能不是build toolchain | workflow/Invoke在后续阶段调用当前`rustc -Vv` | 只消费builder签发的ToolchainSetId，不允许stager补写build事实 |
| MVPCTL-P1-012 | mutable-checkout重复binary build已删除，跨gate共享BuildSet仍未完成 | workflow不再先执行孤立的editor/runtime `cargo build`；F4 integration gate继续验证editor composition，最终Stage只消费`Build-MvpProductInputs`从immutable BuildSet发布并哈希的四个package/feature产物。每run显式重复product build命令减少2条，builder的四个真实性请求不变 | 继续让F1-F5全部消费同一BuildSet/target/CAS；`zircon_app`与`zircon_runtime`跨package合并发布需要validator owner提供多package receipt，不能绕开validator直接复制target产物 |

### 4.2 Staging input、Scenario与过程状态

| ID | 差距 | 当前证据 | 目标合同 |
| --- | --- | --- | --- |
| MVPCTL-P1-013 | required input reparse已fail-closed，仍缺统一ownership schema | `Get-MvpOperationalFileList`遇file/dir reparse立即拒绝并报告路径，BuildSet与run artifact预算也使用no-follow枚举 | required input遇reparse直接fail并记录path/kind；不得无声缺文件 |
| MVPCTL-P1-014 | 当前MVP project source/derived/generated copy policy已版本化 | 64 KiB上限、strict UTF-8、exact-property的schema v1声明默认`source/include`，并为autosave/cache/play/registry/thumbnails声明ordinal-sorted、non-overlap的`derived|generated/exclude-subtree`规则；Stage只读冻结policy bytes/SHA/size，物理排除字面量从5降到0。遍历先拒绝file/directory reparse point，再在目录入栈和文件计数前按冻结policy剪枝，避免先扫描整棵derived tree再过滤；receipt绑定staging manifest、startup summary和返回结果。policy snapshot现一次性预计算5个排除前缀；逐路径分类不再包装rules、不再`Split`组件、也不再逐规则拼接前缀，每次分类消除2个数组和最多5个临时字符串。focused contract 8/8、策略/存储批次17/17、相邻8-suite合并批次74/74通过 | 当前MVP Stage copy policy及分类热路径已闭环；后续把同一ownership/copy原语并入全局project schema，新增规则必须保持no-follow与subtree非重叠约束 |
| MVPCTL-P1-015 | stage input与产品输出共用可变root | binary、project、logs、captures、summary都进入同一tree | immutable input mount + isolated writable work/output roots |
| MVPCTL-P1-016 | initial staging manifest不是最终phase graph | manifest写完后root持续被产品和driver修改；当前发布前projection核对与tree manifest inventory仍保留no-follow/reparse拒绝、逐文件SHA及最终deterministic sort，但遍历已从全树`Get-ChildItem -Recurse`数组和每目录child数组切为typed `Stack<DirectoryInfo>`/`Queue<DirectoryInfo>`加`EnumerateFileSystemInfos`。projection峰值由`O(total entries)`降为`O(depth)`并消除每entry的`Get-Item`；tree manifest消除每目录1个数组和string queue payload。两个inventory各在入口只规范化一次root并构造一次containment prefix，枚举器`FullName`经ordinal containment后直接substring；逐entry消除2次`GetFullPath`和1次prefix string构造。projection对象现同时冻结normalized root/prefix，source/owned writers只规范化传入path一次，随后通过normalized helper处理全部ancestor；10,000路径三轮合成均值由1,807.2 ms降至620.5 ms（-65.7%），仅为工具helper微基准而非产品P50/P95。动态`PSCustomObject` descriptor改为typed immutable tuple，所有目录共享一个只读descriptor；inventory直接比较枚举元数据，文件size不符时在SHA前拒绝，消除每个实际entry的descriptor对象。manifest reader的公开resolver签名保持不变，内部已规范化resolver以ordinal string检查替代每entry 2个`Split`数组、1个filtered segment array及join；reader复用root/prefix，每entry只做一次separator normalization。排序深度由一次字符扫描写入内部`sort_depth`，排序后移除，对外entry schema不变，并再消除每entry 1个`Split`数组。reader直接消费parsed entries并以array/scalar分支计数，不再复制最多100,000个引用的完整数组；kind改为exact `file|directory`双比较，每entry再消除1个二元素literal array并拒绝大小写漂移。projection聚焦合同5/5、required相邻12-suite批次100/100通过 | 每阶段独立input/output manifest，以RunGraph edge连接且不可回写上游；后续遍历/reader不得退回全树/每目录物化、逐entry重复root/path规范化、动态directory descriptor、segment split数组或完整entries引用复制 |
| MVPCTL-P1-017 | automation request已有v1 schema，仍缺migration与registry | authoring/reopen JSON携带固定`schema_version`、`scenario_kind`、`scenario_id`；Stage在产品启动前以64 KiB上限、strict UTF-8、exact root fields和expected ID验证 | versioned `ScenarioSpec`，有migration、unknown-field policy和schema validator |
| MVPCTL-P1-018 | automation selector与payload是固定UI细节 | cube node id 3、固定control path、X=42、scale=1.25 | stable semantic selector/CommandId、precondition、expected transition与fallback policy |
| MVPCTL-P1-019 | 已有versioned ScenarioRegistry并进入Stage证据链，driver执行顺序与设备矩阵仍未注册表驱动 | 5个MVP scenario以exact schema声明capability、roles、steps、oracle、artifacts、variants和owner；Stage在input publication前验证并记录registry SHA-256/bytes，automation ScenarioSpec绑定registry ID | ScenarioRegistry继续驱动RunGraph顺序、独立oracle实现和真实device/RHI variants，不保留driver内固定编排 |
| MVPCTL-P1-020 | repetition与timeout已由ScenarioRegistry逐场景管理 | 5个场景各声明device class、attempt min/default/max、progress inactivity和与steps同序的timeout；resolver由step预算求和process timeout，CLI兼容参数只能在attempt范围内选择或收紧timeout，Stage记录5份resolved policy并传给各supervisor | 已完成host.default策略闭环；后续device matrix扩展需增加对应variant policy，不得回退到全局整数直传 |

### 4.3 Product process supervisor与日志

| ID | 差距 | 当前证据 | 目标合同 |
| --- | --- | --- | --- |
| MVPCTL-P1-021 | child环境策略已versioned，仍缺共享schema registry/migration | policy固定`schema_version=1`和`zircon.mvp-process-environment-policy` kind；四类场景有稳定ID和精确host/declared集合，supervisor先验证exact schema再清空环境并记录source/sensitivity/value digest | deny-by-default env allowlist，记录每项来源/敏感性/digest |
| MVPCTL-P1-022 | process journal已有run/build/scenario上下文，仍缺真实ProductReceipt与Device identity | `started`记录32 KiB上限内raw args和完整versioned context；每条事件携带context ID，并绑定RunId、source fingerprint、可选BuildSet、ScenarioRegistry SHA、Scenario/variant、PID+creation identity、exe/cwd/args/env digest；ProductReceipt数组当前为空且状态固定`unqualified_missing_product_receipt` | RunId、PID+creation identity、exe hash、args、cwd、env digest、BuildSet/ProductReceipt/Scenario/Device IDs，受信receipt缺失时不得提升qualification状态 |
| MVPCTL-P1-023 | journal事件流已versioned，仍缺统一IDL/migration | 每条事件固定`schema_version=1`和`zircon.mvp-process-lifecycle-event` kind；恢复已有active journal时以严格UTF-8 `StreamReader`逐行扫描，要求末字节为LF后才把最后完整行作为cursor，拒绝非法UTF-8、截断末行、缺失或不兼容schema，再继续sequence/hash chain | append-only start/heartbeat/exit/cleanup events，sequence与hash chain可恢复；后续仍须统一IDL/migration |
| MVPCTL-P1-024 | stdout/stderr主artifact与tail均已具备共享原子预算 | 主artifact继续共享run级retained-byte budget；tail由同一个`MvpProcessOutputCaptureBudget.ReserveExact`原子预算约束，总容量固定64 KiB并公平请求每流最多32 KiB，第二路无法超额预留；终态stdout/stderr凭据分别记录`tail_capacity_bytes`，聚焦合同5/5。共享预算回归仍生成约1.5 KiB双流输出并要求`dropped_bytes > 0`，但无关fixture输出由512行降为64行，测试I/O减少87.5%，并把非timeout语义的调度预算与真正timeout case解耦；相邻8-suite合并批次72/72通过 | 已完成bounded ring buffer、streaming artifact writer、dropped-byte counters与两流共享tail容量上限；后续不得退回每流独占64 KiB |
| MVPCTL-P1-025 | journal已具备rotation、retention和有界tail cursor | active segment限制为1 MiB并轮换为六位immutable archive；hash chain和cursor跨segment连续，默认最多保留64段，裁剪前先写入最大段数、裁剪范围/数量和聚合SHA回执，再删除旧段。恢复从`ReadAllText`整段字符串改为8 KiB严格UTF-8逐行读取，仅保留最后event line，峰值由`O(segment bytes)`降为`8 KiB + O(max event line)`；tail从`MemoryStream`增长后`ToArray`再复制改为按冻结remaining bytes一次精确分配并直接读取，消除最多1 MiB的重复byte copy；tail segment解析不再调用完整resume state，复杂度从`O(active segment bytes + archives)`降为只枚举最多64个archive名称的`O(archives)`。strict/non-strict UTF-8 encoder现为module级复用实例，常态event的pruned path保持scalar `null`直到确有裁剪；每次非rotation event固定分配减少2个encoder对象和1个空array，rotation另少1个encoder对象。初始化确保归档数不超过上限且单次rotation只新增1段，因此rotation retention现以`EnumerateFiles`单遍选最小segment，复杂度由最多65段的`O(n log n)`全排序降为`O(n)`，消除每段1个`PSCustomObject`及records/pruned/path 3个数组；发现超过`max + 1`则失败关闭。Wave139将payload/rotation/retention string SHA改为共享UTF-8 encoder配合`ArrayPool<byte>`单缓冲hash，常态event消除1个完整payload byte array，rotation最多消除3个；started event直接序列化冻结的typed environment array，消除每次启动1个完整引用数组复制。2000次synthetic hash由872.0 ms降至854.1 ms（-2.1%）；另一个pooled append候选在500次写入中由957.3 ms退化至1154.8 ms（+20.6%），已拒绝并恢复`AppendAllText`。这些数据仅用于实现选择，不是ProductReceipt P50/P95。聚焦合同10/10、required相邻12-suite合并批次104/104通过 | 已完成实时flush、rotation、structured event stream、tail cursor与当前1 MiB段内存/查询/固定对象/归档选择/hash buffer收敛；后续不得恢复整段字符串、双byte-buffer、每事件encoder、payload byte-array hash、全归档排序或tail触发active段重扫 |
| MVPCTL-P1-026 | run artifact已有统一增量预算，其他artifact schema仍未统一 | Stage发布后只创建一次versioned budget baseline；基线文件增长按差额计费、新文件按全长计费、删除基线文件不返还额度，并以no-follow/64层/100K扫描上限防御目录；policy receipt和terminal measurement不暴露内部map。heartbeat扫描复用枚举器已有`FileInfo.Length`并以typed stacks替代目录`PSCustomObject`，每轮消除`1 FileInfo / file`与`1 PSCustomObject / directory`；目录stack直接持有枚举器返回的`DirectoryInfo`，继续遍历子目录不再把`FullName`压栈后重复构造，每轮消除`1 DirectoryInfo / child directory`。root解析现把已校验的同一个`DirectoryInfo`传入扫描器，进一步每轮消除1个root对象；文件直接复用枚举器的绝对`FullName`，消除`1 GetFullPath / file / scan`。measurement在扫描时以case-insensitive `HashSet`保留duplicate拒绝并直接累计growth/new-file，删除current length dictionary的第二遍遍历。run budget持有并跨heartbeat复用directory stack、depth stack、duplicate-detection HashSet和内部scan-result object，每轮`Clear()`/覆写；固定容器分配由3个降为0，内部结果对象分配由1个降为0，容量仍受64层/100K文件硬上限约束，对外measurement仍每轮独立；聚焦合同10/10、相邻8-suite合并批次74/74通过 | 已完成当前run级总artifact quota及扫描对象/路径规范化/遍历/固定容器收敛；其他artifact仍须统一schema先验size/depth/count与stream parser |
| MVPCTL-P1-027 | Stage已有CPU/内存/进程硬限额与磁盘/文件监管，磁盘仍非kernel hard cap | Job限制8个active process、4 GiB内存和75% CPU；同一Stage共享512 MiB evidence reserve与4096新文件预算，heartbeat及drain后测量，超限写`artifact_quota_exceeded`并终止Job；一次poll内仍可能短暂overshoot。artifact baseline SHA保持原有`path-byte-count + UTF8 path + Int64 length`字节合同，但以一个`ArrayPool<byte>`租用缓冲和`CryptoStream/BinaryWriter`取代每文件path/int/int64三个临时byte数组；10,000条合成路径三轮均值由659.9 ms降至434.5 ms（-34.2%，digest相同），仅作为工具实现微基准，不是ProductReceipt或产品P50/P95。heartbeat现复用run-owned `DirectoryInfo`和containment prefix，每轮仍执行`Refresh + Exists + ReparsePoint`安全重验，消除重复root规范化、元数据对象和prefix字符串。`Measure-MvpRunArtifactBudget`保留默认新快照语义，但允许supervisor显式传入process-owned terminal measurement scratch；每heartbeat进一步消除1个`PSCustomObject`，1,000次空目录合成heartbeat三轮均值由1,687.0 ms降至1,577.1 ms（-6.5%，非产品资格数据）。artifact聚焦合同13/13、artifact+supervisor批次32/32、required相邻12-suite批次102/102通过 | 复用`RenderExtractProcessJob`抽象，Windows Job kill-on-close与resource limits；artifact heartbeat不得退回逐轮root对象/prefix/measurement重建，baseline hash不得恢复逐文件byte-array framing |
| MVPCTL-P1-028 | 已以Windows Job作为唯一process-tree权威 | child以suspended状态创建并在resume前原子assign Job；completion等待并复核Job empty，Stage生产路径不再调用`Win32_Process`或`taskkill`，成功/失败各只保留目录rename句柄探测 | supervisor持有process creation identity和job membership，不依赖全机扫描 |
| MVPCTL-P1-029 | 外部路径helper containment已由Job闭环 | descendant containment按Job membership、active-process accounting和kill-on-close处理，Stage不再按child image location枚举或终止进程 | descendant containment按job/process group，不按image location |
| MVPCTL-P1-030 | Stage已有registry-declared typed产品语义进度 | ScenarioRegistry为5个scenario声明有序namespaced/versioned progress event IDs；collector只用raw diagnostic marker定位startup/frame/open/save/teardown，probe按registration映射后才写journal，supervisor按process最多256个ID去重，inactivity归类`progress_stalled`并终止Job。每次liveness poll继续受64文件/8层/每文件1 MiB/总读4 MiB约束，但目录遍历取消与语义无关的最多64文件`Sort-Object`；每个probe state复用typed directory/depth stacks、diagnostic `List<FileInfo>`、active/detected两个HashSet、path/offset/bytes/carry四个snapshot lists、progress/stale-path lists和一个8 KiB读取buffer，每poll仅`Clear()`，不再创建目录`PSCustomObject`、每日志snapshot对象或inventory array。失效游标清理不再用`@($State.file_offsets.Keys)`物化最多64-key数组，而是先写入复用list再删除。全部progress IDs发出后入口常数时间裸返回，不再访问inventory或分配空array；无新marker轮次也在progress list创建/遍历前裸返回，常见空闲轮次的结果容器分配由1个list加1个空array降为0；事件轮次复用progress list且只为调用方物化结果array。supervisor不再以`@(& probe)`为每个空poll建立数组，并由process state复用一个progress result object；常见空poll进一步消除1个空array和1个`PSCustomObject`，null/scalar/array仍受每poll 16 milestone上限约束。progress journal writer现直接返回recorded timestamp标量，caller复用已持有的progress name；每个有效milestone再消除1个二字段`PSCustomObject`，单process上限为256个。部分完成时每个8 KiB chunk只对尚未emitted的milestone执行`IndexOf`，例如runtime首帧后由3次降为1次；liveness聚焦合同13/13、supervisor聚焦合同19/19、artifact+supervisor批次32/32、required相邻12-suite批次102/102通过 | 已完成typed heartbeat、startup-ready、frame progress、save completion、hang classifier与当前poll容器/排序/读取buffer/marker搜索/空结果/result object收敛 |
| MVPCTL-P1-031 | 已有structured非零退出crash事件，仍缺取证与build linkage | crash event绑定同一launch identity、exit code和`nonzero_exit`种类；stdout/stderr/tail仍是主要失败证据，没有dump、module list、symbol IDs或ProductReceipt。故障诊断文件读取从`MemoryStream`增长后`ToArray()`再复制改为冻结文件长度、一次分配`byte[length]`并直接填满，读后再探测并发增长；每文件内存从增长缓冲加最多1 MiB副本收敛为一个不超过1 MiB的精确快照，聚焦分组2/2、相邻8-suite合并批次67/67通过 | crash dump、module list、symbol IDs、ProductReceipt和structured crash event；后续不得恢复诊断文件双byte-buffer |
| MVPCTL-P1-032 | Stage/process终态已绑定qualification context与storage capability，仍缺ProductReceipt/Device | request CLI原子发布bounded cancellation request，Stage各phase probe并记录`cancellation_requested`；process terminal携带context ID，Stage terminal schema v3以canonical context-set receipt/SHA绑定5个scenario并覆盖success/fail/cancel/timeout与cleanup，同时对三个preflight终态重验原始Stage root的storage capability；published终态缺证据fail closed，admission终态显式为null；当前聚合状态固定`unqualified_missing_product_receipt`。取消请求消费从4 KiB读取buffer、`MemoryStream`和`ToArray()`副本改为一次实际长度分配并完整读取，最坏同时byte storage由两份4 KiB级缓冲降为一份不超过4096 bytes的快照；聚焦合同5/5、相邻8-suite合并批次67/67通过 | 无论success/fail/cancel/timeout均原子发布bounded terminal receipt、qualification context与cleanup outcome，补齐ProductReceipt/Device后仍需独立Observation authority判定；取消轮询命中路径不得恢复双缓冲 |

### 4.4 Oracle、视觉与产品行为覆盖

| ID | 差距 | 当前证据 | 目标合同 |
| --- | --- | --- | --- |
| MVPCTL-P1-033 | PNG判定只是blank heuristic | 尺寸、visible、相对左上角100像素、hash | blank guard保留，但不能代替approved visual/semantic oracle |
| MVPCTL-P1-034 | 没有独立approved baseline/diff | capture由product写，driver只hash | candidate/incoming/diff/approved/promotion分离，按RHI/device/DPI key比较 |
| MVPCTL-P1-035 | before/after只要求pixel hash不同 | 任意噪声或错误页面变化也满足 | expected region/scene object/transform/UI field的语义与像素双重断言 |
| MVPCTL-P1-036 | marker与summary来自同一产品通道 | Stage/Invoke重复解析固定字符串 | out-of-process observer与in-product telemetry双通道交叉验证，冲突即失败 |
| MVPCTL-P1-037 | 只验证first/presented few frames | 固定capture/exit env驱动短运行 | startup、steady-state、transition、teardown和long-run checkpoints |
| MVPCTL-P1-038 | acceptance只覆盖一个RenderableEmpty意图 | cube/light/camera与固定transform | capability matrix覆盖asset/import/scene/render/input/save/reopen/package等独立scenario |
| MVPCTL-P1-039 | 无RHI/adapter/device/platform矩阵 | Windows hosted lane与单WGPU路径 | supported RHI、GPU class、driver、OS、headless/windowed、editor/player role矩阵 |
| MVPCTL-P1-040 | 无DPI/locale/input/accessibility变体 | automation依赖固定window/control坐标语义 | DPI、locale、font、theme、keyboard/gamepad/IME、accessibility mode作为ScenarioVariant |
| MVPCTL-P1-041 | 无负向、恢复、升级、并发和soak场景 | happy-path create→edit→reopen→render | corrupt/missing asset、crash-save recovery、old schema upgrade、multi-process contention、long soak |

### 4.5 Evidence schema、snapshot与publication

| ID | 差距 | 当前证据 | 目标合同 |
| --- | --- | --- | --- |
| MVPCTL-P1-042 | GateRegistry authority与summary receipt已闭环，仍缺完整RunGraph | 64 KiB上限/strict UTF-8的registry以一次有界读取冻结原始bytes、SHA-256和size；workflow从同一snapshot执行ordered profile 7项/workspace 2项并在两个schema v2 summary中绑定exact四字段receipt。validator拒绝unknown receipt字段、错误JSON integer、digest/size/kind漂移；acceptance fixture也消费同一snapshot，不再持有第四份gate表。summary仍限制1 MiB、单gate evidence 64 MiB | GateRegistry identity闭环已完成；仍须增加gate node/edge、producer、dependency与执行结果的RunGraph receipt，并把whole summary reader进一步收敛为流式schema reader |
| MVPCTL-P1-043 | resource schema原语与identity registry已收敛，仍缺生成式IDL | baseline reporter与comparison通过同一`ResourceManagementSchema`执行exact required/optional property、uppercase SHA-256、JSON number与非负整数判定；`ResourceManagementSchemaRegistry`集中19类artifact的SchemaId/current/minimum/identity。两侧仅保留语义命名和错误文本薄包装；plan/report/change-set JSON入口仍按64 KiB至64 MiB角色预算读取 | 继续把scale/change-set等剩余手写validator迁移到registry，并从单一IDL生成exact reader/writer/migrator/fixture corpus |
| MVPCTL-P1-044 | 19类resource schema identity已有registry，仍缺迁移实现 | 64 KiB/strict UTF-8 registry以bounded snapshot SHA receipt注册current/minimum version、exact compatibility与kind identity；新增approval trust/receipt/verification identity，拒绝unregistered/stale/future/mismatched artifact。observation、report、policy、context、execution/sample protocol、sample process context及workload/approval artifact已消费同一authority，嵌套对象不再匿名 | 仍须为非exact compatibility定义window/migrator、golden fixture corpus与migration receipt；当前registry明确只允许exact current version，不得误记为迁移系统完成 |
| MVPCTL-P1-045 | snapshot每entry句柄与整树复制仍线性，复制/hash对象分配已收敛 | deep no-follow递归仍持有完整entry handle/marker集合并复制全部bytes；本轮把1 MiB copy buffer从每文件创建改为整次materialization复用，把SHA-256实例从每文件创建改为整次manifest inventory复用，两类大对象/crypto对象创建均由O(files)降为O(1) | 继续设计不削弱no-follow/namespace freeze的分段manifest与bounded handle window；CAS/clone/hardlink必须有immutable/COW policy，不能以可变hardlink换取表面吞吐 |
| MVPCTL-P1-046 | acceptance snapshot资源admission与deadline已闭环 | manifest文件先以no-follow handle固定，超过64 MiB则在`ReadToEnd`前拒绝；JSON展开后又在resolved-path inventory前限制100,000项。exact schema v1 admission receipt同时绑定manifest bytes，并默认限制16 GiB文件总量、深度64和600秒。超限在逐项handle/hash/census/copy前fail closed；同一receipt贯穿entry lease、membership map、census、marker、recheck与递归copy，文件以1 MiB块复制并逐块检查deadline | 已完成manifest/count/payload-byte/depth/time预算与流式copy期限；CAS/分段manifest/bounded handle pool继续由MVPCTL-P1-045收敛，不在本项重复实现 |
| MVPCTL-P1-047 | approved baseline已有独立签名验证，其他evidence仍是self hash | RSA-PSS-SHA256 detached signature绑定issuer/key、report/workload/promotion字段；固定有界trust registry支持key有效期、disabled issuer与receipt revoke，篡改直接拒绝。默认registry刻意为空，尚无CI/OIDC/worker身份或trusted timestamp | 由Security/operations owner配置不可导出的issuer key、CI/OIDC或worker identity与可信时间戳，并把source/build/run/policy attestation扩展到全部evidence；本地测试key不得进入生产trust root |
| MVPCTL-P1-048 | promotion receipt治理字段已闭环，全局Evidence Catalog仍缺失 | approved baseline receipt schema v2把PromotionId、EvidenceSetId、review、`accepted-baseline` retention期限、legal/security scrub SHA与可空supersedes链纳入签名载荷；trust registry可撤销receipt并拒绝自引用链 | Tooling07/09共享owner仍须实现content-addressed append-only catalog、retention lock/GC引用、review actor authority与完整promotion/revoke ledger；当前resource receipt不是全局catalog |

### 4.6 CI、test runner与currentness

| ID | 差距 | 当前证据 | 目标合同 |
| --- | --- | --- | --- |
| MVPCTL-P1-049 | Rust与action已固定，hosted runner镜像仍可漂移 | MVP workflow固定Rust 1.94.1，并把checkout/rust-toolchain/rust-cache/upload-artifact全部固定为上游commit SHA；environment receipt记录实际`ImageOS/ImageVersion`、toolchain policy、action SHA及rustc/cargo输出；`windows-latest`仍由GitHub滚动更新 | 剩余项是不可变/self-hosted runner image identity；在此之前每次run必须保留具体image receipt，不得把同label视为同环境 |
| MVPCTL-P1-050 | Windows required lane已覆盖MVP、RenderExtract与resource控制面 | workflow在Cargo/F0-F5后以一个聚合批次运行32个schema/input/storage-policy/project-copy-policy/snapshot-admission/output-capture/journal/supervisor/liveness/cancel/terminal/registry/context/Job/quota/log/RenderExtract/resource suites共352例；profiling-input、RenderExtract scale、resource BuildSet/ProductInput identity、change-set receipt、storage capability revalidation、BuildSet Git stdout buffer/length capture/UTF-8 encoder/root-prefix/traversal-relative/exact-property/split-free admission/SetEquals inventory复用、BuildSetId BinaryWriter/CryptoStream流式framing与三槽segment vector复用、staging projection/tree manifest typed streaming/root-prefix复用/typed immutable descriptor/split-free path admission/sort-depth/parsed-entry复用、project derived-tree pruning与无分配分类、stdout/stderr共享tail预算/fixture收敛、journal流式恢复/精确tail buffer/archive-only segment查询/encoder复用/单遍rotation retention/pooled payload hash/launch environment直接序列化、artifact quota单遍扫描/run scratch/typed directory/scan-result/measurement/root metadata/absolute path/run-owned root-prefix复用与baseline hash池化、liveness全poll scratch/完成态/remaining-marker/空结果/stale-key与supervisor progress result/timestamp scalar复用、取消请求与supervisor诊断文件精确byte snapshot均在同一批次，随后仍运行3个已注册script contract case | 已完成当前Tooling15 focused graph；新增控制面suite必须同步membership contract与exact count |
| MVPCTL-P1-051 | required lane已固定PowerShell/Pester、exact count和结构化结果 | control-plane从官方SHA-256固定的PowerShell 7.4.19便携包启动；直接验证发现Pester 5.9.0拒绝仓库既有legacy `Should Be`并丢失顶层fixture run scope，因此兼容pin改为受管Pester 4.10.1并使用其`-Script`入口。lane要求358/358且输出NUnit XML，workflow contract固定runtime、digest、32-suite membership与结果路径 | Wave147已按当前production/test source异步提交358+3多任务批次到独立stdout/stderr日志，本会话不轮询。提交前5个PowerShell文件AST、32-suite membership、358/3 exact count、Wave146归一化模板零差异、BuildSet allocation/path/schema/inventory/identity/list-handoff合同16/16、受管Pester4完整BuildSet批次33/33及workflow contract均通过；Wave146相邻未改套件此前73/73。本地required-script既有两项失败仍归因共享脏树的staging旧fallback断言与受保护P1-045 snapshot竞态，不计入Wave147通过声明且本轮未跨写。Wave139的5.9.0提交不得升级为可接受验证结果；协调器结果仍待后续以结构化NUnit校验，不得把提交成功写成验证通过 |
| MVPCTL-P1-052 | 三个脚本级required gate已注册，staging内部断言仍是单一case | `mvp-staging`、`mvp-acceptance`、workflow contract现由同一pinned Pester调用注册为3个可筛选case，各有2-15分钟预算、进程树超时终止、独立stdout/stderr和一个NUnit结果；不再裸执行后只看exit code | 已完成脚本级TotalCount与结果归档；仍须把`mvp-staging.Tests.ps1`内部约2111行断言按fixture/phase拆成独立case并增加coverage，不得把当前3个外层case视为细粒度迁移完成 |

### 4.7 Resource-management观测与统计

| ID | 差距 | 当前证据 | 目标合同 |
| --- | --- | --- | --- |
| MVPCTL-P1-053 | baseline plan没有执行/观测owner | 只读审计确认产品profiling导出与scan计数器存在，但scan埋点位于其他owner未提交的Rust改动；page只有测试级metrics且没有产品发射点，`asset_workspace.snapshot`只存在于工具计划/报告，现有project automation也不接受resource query workload | `ResourceObservationRunner`启动真实product、执行query、采集trace并签发observation；在产品owner发布完整三类遥测与自动化命令前不得由工具合成缺失counter |
| MVPCTL-P1-054 | observation context已绑定，producer真实性仍未受信 | observation/report schema v3通过共享authority在根级绑定ProductReceipt ID/source/build/executable、RunId、匿名machine ID/CPU/logical processors/RAM/OS/architecture与collector/version/clock；每个sample单独绑定product PID/process creation UTC/trace/frame range，query frame必须落在本sample区间。plan的`fresh-process`要求每次不同进程实例，`same-process`要求全attempt复用同一实例；comparison schema v3重验这些receipt并拒绝machine或collector contract漂移。所有声明仍来自未签名caller，measurement保持`unverified/untrusted-observation-context` | 仍须由真实ResourceObservationRunner从已验证ProductReceipt与进程句柄采集并签名context，加入issuer/timestamp/revoke与trace artifact digest验证后才能升级qualification |
| MVPCTL-P1-055 | warmup/sample sufficiency/noise/confidence/raw samples与outlier receipt已闭环 | plan/observation/report hard-cut为schema v2；统一policy要求3次warmup、20-50次measurement和至少20个样本。独立统计模块保留raw samples并计算Welford标准差、median/P95、MAD、CV、保守95%均值区间与relative margin；comparison对baseline/candidate分别签发`retain-all` MAD receipt，记录identified/removed/retained且禁止静默剔除 | 已完成样本资格、warmup排除、噪声/confidence与outlier审计；effect size/significance由MVPCTL-P1-056 comparator拥有 |
| MVPCTL-P1-056 | cohort比较、budget、significance及approved receipt验证已闭环，trusted observation仍缺失 | `ResourceManagementComparison`从raw samples重算两组cohort，分离absolute/relative budget、保守95%均值差区间与Hedges' g；writer以1 MiB有界strict-UTF8读取实际approval receipt，策略必须绑定其真实文件SHA。receipt再绑定approved report/workload并经固定trust registry执行RSA-PSS issuer/expiry/disabled/revoke验证，结果写入comparison schema v3 | approval链代码已完成，但默认trust registry无生产issuer，且两侧report仍为`untrusted-observation-context`，所以qualification保持`unverified`；必须由真实runner和operations trust owner共同闭环后才能接性能gate |
| MVPCTL-P1-057 | workload profile合同已可扩展，实际覆盖仍只有Data flat | 64 KiB/strict UTF-8 workload registry登记`json-data-flat-v1`的asset kinds、dependency graph shape、tag cardinality、allowed query mix、scale bounds与change percent；baseline plan schema v3绑定registry snapshot receipt/profile，report schema v4继承，comparison在cohort前拒绝registry/profile漂移 | 继续添加真实generator+product telemetry支持的Mesh/Material/Audio等profile、dependency DAG、tag/cardinality与mixed workload；禁止只登记无runner/telemetry的虚假覆盖 |
| MVPCTL-P1-058 | cache/order/quiescence receipt结构已闭环，执行真实性仍未受信 | 根级execution protocol固定Fisher-Yates SHA-256随机化算法/seed/order receipt、`os+ddc+resource-index` cache scope与quiescence policy；每个sample保留全局连续唯一sequence、cold-open=`cold/purge`或stable/change=`warm/prime`、cache receipt、quiescence receipt及同product PID。report/comparison重验全部字段并拒绝协议漂移；warmup/measurement phase与attempt继续由plan绑定 | 仍须真实runner执行并签名purge/prime、order与quiescence receipt，绑定OS/DDC/index generation、background load和collector artifact；当前caller声明不得升级performance qualification |
| MVPCTL-P1-059 | diagnostic comparison artifact与approval verification已闭环，qualified gate仍由P1-054阻断 | schema-v3 comparison JSON汇总regression/within-budget/inconclusive及原因，绑定baseline/candidate/policy/实际approval receipt SHA，保留approval verification、两侧observation/execution protocol context及trend/bisect HTTPS链接；Markdown展示验证状态与每个scenario/query的两组median、增量、effect和decision。writer对两份report各限64 MiB、approval receipt限1 MiB、policy限4 MiB，严格UTF-8读取并以CreateNew staging目录一次move发布 | 已完成机器/人工诊断产物；qualification仍固定unverified，只有trusted ObservationProducer与operations配置的issuer trust root闭环后才能把同一decision接入gate |
| MVPCTL-P1-060 | resource JSON evidence已收敛，跨Render/MVP runner仍重复 | baseline reporter与comparison writer已删除各自文件打开、稳定读取、strict UTF-8和SHA实现，统一调用`ResourceManagementJsonEvidence`并只拥有语义标签/4或64 MiB预算；RenderExtract与Stage仍各有Job/FrozenInput/Evidence/process primitive | 继续收敛统一Runner/Observation/Evidence库；Render、MVP、resource最终只提供scenario plugin，当前resource内收敛不得被误记为跨产品完成 |

## 5. P2：成熟度增强项

| ID | 增强项 | 目标 |
| --- | --- | --- |
| MVPCTL-P2-001 | 跨平台runner backend | Windows Job、Linux cgroup/process namespace、macOS process group统一接口 |
| MVPCTL-P2-002 | 远程设备池 | reservation、health、firmware/driver profile、install/run/cleanup和device quarantine |
| MVPCTL-P2-003 | scenario sharding | 基于历史时长/资源/互斥约束分片，保持同一BuildSet和可重组结果 |
| MVPCTL-P2-004 | declarative scenario authoring | typed DSL/SDK生成step、role、oracle、artifact与failure policy，不再手写大脚本 |
| MVPCTL-P2-005 | evidence catalog UI | 按BuildSet/product/scenario/device查询，展示logs、trace、visual diff和promotion history |
| MVPCTL-P2-006 | live progress/cancel UX | Hub/Editor/CLI共享OperationId、phase、ETA、cancel与failure action |
| MVPCTL-P2-007 | flake intelligence | retry不覆盖首败，分类infra/product/observer，按历史概率隔离并保留paired evidence |
| MVPCTL-P2-008 | remote CAS与delta transfer | product/runtime dependency、fixture和evidence按content-address复用，避免全树重复copy |
| MVPCTL-P2-009 | bounded support bundle | 一键导出脱敏后的run graph、environment、logs tail、dump、trace和manifest |
| MVPCTL-P2-010 | release qualification联动 | accepted evidence promotion成为channel发布输入，撤销/安全事件可反向失效qualification |
| MVPCTL-P2-011 | performance trend与bisect | cohort趋势、change point、自动回归候选commit和hardware-normalized视图 |
| MVPCTL-P2-012 | quality budget矩阵 | startup、frame、memory、asset query、visual、accessibility与stability预算统一治理 |
| MVPCTL-P2-013 | schema migration CLI | inspect/validate/migrate/diff receipt与scenario/evidence，保留原始artifact和迁移链 |
| MVPCTL-P2-014 | soak与fault injection farm | process kill、disk full、device loss、driver reset、network/cache corruption和restart恢复 |

## 6. 参考引擎给出的约束

| 参考 | 本轮确认的结构 | 对Zircon的约束 | 不应机械复制 |
| --- | --- | --- | --- |
| Unreal `TargetReceipt` | target/platform/architecture/configuration/launch、BuildProducts、RuntimeDependencies、plugins、version分别入receipt | Zircon product manifest必须从“几个exe/hash”升级为完整可启动artifact closure | 不复制Unreal文件格式或所有AdditionalProperties |
| Unreal BuildGraph/TempStorage | node/output tag、manifest、artifact store/retrieve组成可恢复依赖图 | build/test/stage/package必须消费同一节点产物和identity，而不是重复build/目录猜测 | 不要求先实现完整Horde |
| Unreal Gauntlet | AppConfig、AppInstance、TargetDevice、streaming log reader、exit/kill、artifact、test state/event/telemetry分层 | Scenario、Device、Process、Report是独立owner；产品不应自判pass | 不把C#类层级原样搬到Rust |
| Unity Graphics Tests | scene case、wait frames、settings、reference image、ImageAssert与DX12/Vulkan等lane分离 | visual acceptance必须有scene/settings/reference/RHI matrix和独立artifact | 不把Unity阈值直接当Zircon阈值 |
| Bevy example-showcase | example registry、required features、WGPU backend、fixed frame time、screenshot/stop frame、passed/failed/no-screenshot报告 | 即使轻量runner也应把scenario选择、backend、deterministic time与结果分类显式化 | showcase不是完整release qualification系统 |
| Godot main/test entry | product CLI统一提供project path、headless、rendering method/driver、quit-after、benchmark JSON与test runner入口 | Zircon应有稳定跨平台产品CLI和可组合test/benchmark command，不把控制全塞进env marker | Godot CLI本身不能替代外部process supervisor |
| Fyrox build tools | build/export是engine工具crate，UI可流式pump stdout/stderr并提供stop | build owner应是版本化产品工具API，Editor/Hub只是consumer | Fyrox当前规模不是最终性能/安全上限 |

参考实现用于验证职责边界，而不是用“另一引擎也有脚本”合理化当前状态。Zircon的目标高于当前Unreal性能/表现时，更需要先让input、artifact、run和evidence可证明；没有这一层，任何性能领先声明都无法复现。

## 7. 目标控制面

```text
Source Resolver ──> immutable BuildSet/CAS ──> Build Graph ──> signed ProductReceipt
                                                        │
Scenario Registry ──> Variant/Device Reservation ──> Product Supervisor
                                                        │
                         independent Observer/Oracle <──┤──> product telemetry
                                                        │
                              Raw Observation/Artifact ──> Evidence Validator
                                                                  │
                                             Candidate ──> Review/Policy ──> Promotion
                                                                  │
                                                   Evidence/Qualification Catalog
```

核心identity至少包括：

| Identity | 必须绑定 |
| --- | --- |
| `BuildSetId` | repo objects、declared dirty overlay、submodule/LFS/generated inputs、dependency lock、source policy |
| `ToolchainSetId` | cargo/rustc/linker/SDK/target/profile/build env与binary hashes |
| `ProductReceiptId` | BuildSet、ToolchainSet、action、build products、runtime deps、symbols、SBOM、producer |
| `ScenarioId@version` | capability、roles、steps、pre/postconditions、oracle、artifacts、budgets、variants |
| `DeviceProfileId` | OS/image、CPU/RAM/GPU/driver/RHI/display/DPI/locale与health receipt |
| `RunId` | ProductReceipt、Scenario、Device、process tree、environment、timeline、terminal outcome |
| `ObservationId` | collector、channel、trace/frame/time、raw data hash和RunId |
| `EvidenceSetId` | validator/policy versions、observations、diffs、errors、completeness与attestation |
| `PromotionId` | candidate、approved baseline/qualification、reviewer/policy、timestamp、revoke chain |

## 8. 分阶段重构与验收门

### M0：冻结错误状态传播

- `F5 accepted`和resource `measured`只允许由promotion service写；现有driver输出标记为`candidate/unqualified`。
- required CI先加入AST/JSON schema、exact focused test count和已知failure regression；Pester/PowerShell版本固定。
- 验收：当前三个harness failure有结构化修复，所有test实际注册且0个`TotalCount=0`伪通过。

### M1：BuildSet与ProductReceipt

- 从活动checkout生成allowlisted immutable snapshot/CAS；builder只能挂载只读BuildSet。
- 把build owner从`.codex/skills`迁到版本化workspace CLI/crate，签发完整ProductReceipt。
- 验收：A→B→A mutation、无关1.68 GB untracked output、submodule/LFS/toolchain变化都有确定行为；四个product共享同一BuildSet且无需重复build。

### M2：统一Product Supervisor

- 提炼RenderExtract已有Job/frozen-input机制，替换Stage的裸Process/WMI/taskkill路径。
- 建立env allowlist、streaming bounded log、heartbeat/cancel/quotas/crash artifact和terminal receipt。
- 验收：child逃逸、helper外部复制、log flood、hang、timeout、crash、disk quota与cancel均不遗留进程，且有bounded evidence。

### M3：Scenario Registry与独立Oracle

- automation JSON迁到versioned ScenarioSpec；固定node/control/delta转为semantic selector与expected state transition。
- 外部observer采集window/frame/process，产品telemetry只作交叉signal；visual candidate与approved baseline分离。
- 验收：伪造marker、随机非空PNG、错误UI字段和错误scene object都无法通过；同scenario可扩展RHI/DPI/locale/device variant。

### M4：Resource Observation Service

- 实现真实产品runner/collector，observation绑定ProductReceipt、RunId、trace/frame、machine/cache protocol。
- 加入warmup、randomized order、raw sample、noise/confidence、approved baseline与budget decision。
- 验收：任意手写JSON不能得到`measured`；1/1K/100K只是registry中的三个workload，不是硬编码上限。

### M5：Evidence publication与qualification

- 统一MVP、RenderExtract、resource的schema/validator/snapshot/catalog；Windows no-follow backend迁入共享库。
- candidate经policy/review/attestation后才能promotion；CI artifact只承载bounded projection，catalog保留长期identity。
- 验收：F5 clean BuildSet真实运行、required gates全绿、artifact可下载复验、promotion/revoke可审计后，才关闭`blocked_by_f4`并声明current qualification。

## 9. 与既有专项的canonical边界

| 既有报告 | canonical owner | 本篇新增边界 |
| --- | --- | --- |
| Tooling07 Performance/Evidence | benchmark、profile、trace、symbol、crash与render-extract证据总体能力 | resource reporter没有observation producer；MVP/Render/resource runner需收敛 |
| Tooling09 Release/Install/Update | channel、artifact repository、签名、install/update/rollback | F5 candidate到qualification promotion的输入真实性，不重计channel功能 |
| Tooling10 Test Architecture | workspace测试分区、selection、flake、fixture与结果 | `tools/mvp` Pester/script混合runner和required lane缺口的具体实例 |
| Tooling12 Acceptance Archive | 顶层archive、serialization fixture provenance/currentness | MVP stage source snapshot与product evidence trust boundary |
| Tooling13 Repo Control Plane | Codex skill、hook、structural audit治理 | production build依赖skill的具体反向依赖 |
| Tooling14 Visual Evidence | Workbench DesignSpec与candidate/approved screenshot治理 | 真实runtime/editor product capture仍为self-attested且缺Oracle |
| App07 RenderableEmpty | template create/import/render/export产品闭环 | 该单一fixture不能代替通用Scenario/Qualification系统 |

本篇finding不重复计入上述报告的通用P0；它们在实施时通过共享owner `BuildSet/Runner/Observation/Evidence/Promotion`建立依赖。

## 10. 完成状态

| 项目 | 状态 | 证据 |
| --- | --- | --- |
| 33个`tools/mvp`文件逐文件静态扫描 | review_complete | 14,789行；31 PS + 2 JSON |
| 17个直接测试与Windows workflow扫描 | review_complete | 约9,012行；required lane缺focused control-plane suites |
| PowerShell/JSON静态解析 | review_complete | 31/31 AST通过；2/2 JSON通过但无schema version |
| focused dynamic validation | review_complete_red | 84 total / 81 pass / 3 fail；另有Pester TotalCount 0脚本suite |
| 本地参考引擎对照 | review_complete | Unreal BuildGraph/Receipt/Gauntlet、Unity Graphics、Bevy、Godot、Fyrox |
| production/test修复 | partial | MVPCTL-P0-004已fail-closed：未受信observation只能输出`unverified`；MVPCTL-P0-005已实现场景环境策略、Job containment、共享有界输出、8-process/4-GiB/75%-CPU limits、可保留journal及取消/非零退出crash/进度事件。受信producer、磁盘/文件quota、typed product liveness、dump/symbol/ProductReceipt linkage与current qualification尚未实现 |
| F5 current qualification | blocked | plan未完成、compile blocker、control-plane suite RED、无clean promoted artifact |
| resource measured baseline | blocked | 无受信observation producer，现有reporter不得发布measured结论 |
