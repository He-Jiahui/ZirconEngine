---
related_code:
  - tools/mvp/Build-MvpProductInputs.ps1
  - tools/mvp/Build-RenderExtractProfilingInputs.ps1
  - tools/mvp/Capture-RenderExtractBaseline.ps1
  - tools/mvp/Invoke-MvpAcceptance.ps1
  - tools/mvp/MvpAcceptanceNativeFileSystem.psm1
  - tools/mvp/MvpAcceptanceStagingProjection.psm1
  - tools/mvp/MvpAcceptanceStagingSnapshot.psm1
  - tools/mvp/MvpAcceptanceStagingTreeManifest.psm1
  - tools/mvp/MvpBuildSummaryEvidence.psm1
  - tools/mvp/MvpPersistenceComparison.psm1
  - tools/mvp/MvpProcessTimingEvidence.psm1
  - tools/mvp/MvpProductInputManifest.psm1
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
  - tools/mvp/RenderExtractBaselineEvidence.psm1
  - tools/mvp/RenderExtractBaselineMetrics.psm1
  - tools/mvp/RenderExtractFrozenInput.psm1
  - tools/mvp/RenderExtractProcessJob.psm1
  - tools/mvp/ResourceManagementScaleInventory.psm1
  - tools/mvp/Set-ResourceManagementScaleProjectChangeSet.ps1
  - tools/mvp/Stage-MvpProducts.ps1
  - tools/mvp/Write-RenderExtractBaselineReport.ps1
  - tools/mvp/Write-ResourceManagementBaselineReport.ps1
  - tools/mvp/mvp-authoring-automation.json
  - tools/mvp/mvp-reopen-automation.json
  - .github/workflows/mvp-editor-windows.yml
tests:
  - tools/tests/Invoke-MvpAcceptanceTestDriver.ps1
  - tools/tests/mvp_editor_windows_workflow.Tests.ps1
  - tools/tests/mvp-acceptance-staging-snapshot.Tests.ps1
  - tools/tests/mvp-acceptance.Tests.ps1
  - tools/tests/mvp-product-build.Tests.ps1
  - tools/tests/mvp-product-inputs.Tests.ps1
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
implementation_status: pending
source_recheck_required: true
---

# Tooling 15：MVP BuildSet、产品进程、验收证据与资源基线控制面审查

## 1. 结论

`tools/mvp`已经超过“临时验收脚本”的规模：33个tracked文件、14,789行、675,385 bytes，其中31个PowerShell source/module、2个automation JSON；对应17个直接相关测试文件共约9,012行。它同时承担产品构建、输入清单、项目复制、editor/runtime启动、超时清理、日志、截图、场景持久化断言、F5归档、render-extract性能采集，以及resource-management基线计划和报告。这个控制面已经是事实上的Build/Test/Artifact Service，却仍以一组互相import的本机PowerShell脚本存在。

本轮必须先区分“强基础”与“结论不成立”。`MvpAcceptanceNativeFileSystem`、staging snapshot/projection/tree-manifest实现了Windows no-follow handle、volume/file/creation identity、ancestor lease、reparse拒绝、copy后验证、partial publication和按句柄rename；acceptance归档先锁住source tree，再复制、核对manifest并原子发布。RenderExtract还有Job Object、frozen input和timeout。这些机制不应删除，而应成为统一Artifact/Evidence Service的Windows backend。

但其上层信任链仍断裂。产品构建只在可变工作树上反复采样fingerprint，并不从不可变source snapshot构建；fingerprint还把全部unignored untracked文件算作source，本轮实际包含2,373个文件、1,680,960,238 bytes，绝大多数来自`tools/`生成目录。两次无修改顺序采样分别花13.228秒和11.318秒，aggregate test期间又会被并发无关输出改变。一个文件在构建期间发生A→B→A变化仍可能通过前后采样，不同产品可由不同瞬时source产生却共享同一最终fingerprint。

产品运行与结果Oracle也没有分权：runtime/editor自己写diagnostic marker与PNG，Stage解析这些marker、检查PNG非透明且至少100个像素不同于左上角，Invoke再验证同一批自报内容的hash和固定值。一个能输出预期字符串和非空PNG的错误产品可以自我认证。resource-management链更严重：仓库有1/1K/100K项目生成器、change-set和reporter，却没有任何tracked产品观测执行器；reporter只要收到结构匹配的caller JSON，就无条件产生`measurement_status = measured`。这不是性能基线，只是一个可以把合成数字盖章为“已测量”的格式转换器。

2026-08-16在PowerShell 7.4.18/Pester 3.4.0上运行14个focused文件，结果84 total、81 passed、3 failed、265.235秒。三个失败均暴露测试控制面脆弱性：`mvp-acceptance.Tests.ps1`的跨行错误regex不兼容当前格式；两个resource测试分别被Pester 3的`Should Throw`参数绑定行为和StrictMode下访问缺失property击中，直接探针确认production拒绝/变更行为符合其局部意图。`mvp-staging.Tests.ps1`单独exit 0但Pester报告`TotalCount = 0`，因为它实质是脚本级自定义测试。31个PowerShell文件AST解析0 error，但required workflow没有运行这些控制面suite，也没有运行resource或render-extract focused suite。

`docs/plans/mvp/06-f5-acceptance-wave.md`仍标记`blocked_by_f4`，所有验收checkbox未完成，并明确说明需要clean coordinator run与真实上传artifact检查；当前workspace/editor编译又有既存阻断。因此历史artifact或局部driver测试不得升级为current F5 qualification。本轮登记 **6项P0、60项P1、14项P2**；只完成review，没有修改production或test。

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

仓库没有生成observation manifest的产品runner；git全仓只找到plan generator、project/change generator、reporter、tests和计划文档。`Write-ResourceManagementBaselineReport.ps1`验证caller提供的schema、source fingerprint、plan hash、scenario、attempt和counter形状后，固定输出`measurement_status = measured`。它不绑定product receipt、process identity、trace/session、collector、machine、toolchain、cache state或签名。测试fixture正是合成任意counter后得到measured report。必须先实现受信`ObservationProducer`，其run receipt绑定真实product/trace/frame；reporter只能聚合经过producer签名和schema验证的observation，禁止由数据形状推导“measured”。

### MVPCTL-P0-005 · Stage没有可靠的进程树 containment 与有界IO

Stage用普通`Diagnostics.Process`启动产品，stdout/stderr用`ReadToEndAsync`无限积累内存，完成后才整体写日志；超时依赖`taskkill /PID /T`和扫描`Win32_Process.ExecutablePath`是否位于staging目录。child复制/启动到外部路径即可逃逸，PID存在复用竞态；诊断目录也会整树读成string，没有byte/file/depth预算。与同目录RenderExtract已有Job Object形成明显分裂。所有product run必须在Job/cgroup/process-group或平台等价containment中，流式有界日志、heartbeat、cancel、CPU/memory/file/process quotas与crash collection由supervisor拥有，任何失联/逃逸都失败并保留receipt。

### MVPCTL-P0-006 · 当前F5与性能基线没有可发布的current qualification

F5计划仍为`blocked_by_f4`，所有M6.1-M6.4和退出checkbox未完成，并明确要求clean coordinator workflow与真实artifact检查；当前editor/workspace编译存在既有阻断。控制面focused aggregate又是81/84而非GREEN，workflow不运行这些suite，resource lane没有observer。必须把`planned/implemented/tested/qualified/promoted`分开；在immutable BuildSet、required control-plane suites、真实product run、independent evidence和promotion receipt全部通过前，任何manifest、计划或UI不得显示current F5 accepted或measured baseline。

## 4. P1：工程化重构清单

### 4.1 BuildSet、ProductReceipt与输入发布

| ID | 差距 | 当前证据 | 目标合同 |
| --- | --- | --- | --- |
| MVPCTL-P1-001 | production build依赖repo-local Codex skill | `Build-MvpProductInputs.ps1:125`定位`.codex/skills/.../validate-matrix.ps1` | 稳定的workspace CLI/service拥有build；skill只调用公共入口 |
| MVPCTL-P1-002 | artifact root固定Windows盘符和命名regex | product D-F、resource E盘，多处硬编码`ZirconBuilds` | platform storage policy返回approved root/volume capability，不在业务脚本写盘符 |
| MVPCTL-P1-003 | product build直接写最终非空目录 | 每个group顺序创建；中途失败留下future run拒绝的partial root | per-action temp root、abort receipt、完整验证后原子发布 |
| MVPCTL-P1-004 | build receipt缺ToolchainSet | 不记录cargo/rustc binary hash、host、target、linker、SDK、profile/codegen | toolchain、SDK、target、profile和action digest进入receipt identity |
| MVPCTL-P1-005 | build product closure只含四个主文件 | 无DLL依赖、PDB、shader/assets、licenses、plugins或runtime data | typed BuildProducts + RuntimeDependencies + optional/debug products |
| MVPCTL-P1-006 | producer与receipt无认证关系 | JSON由driver本地写，只有内容hash | producer build ID、worker/session、签名/attestation与trusted timestamp |
| MVPCTL-P1-007 | untracked input没有owner allowlist | 全部2,373个unignored文件被hash，可能包含生成物或敏感材料 | declared source roots；unknown untracked fail/ignore按policy并输出审计清单 |
| MVPCTL-P1-008 | source closure不表达submodule/LFS/generated dependency与env | HEAD+diff不足以描述外部materialization | BuildSet显式列git object tree、submodule commit、LFS object、generated input和env allowlist |
| MVPCTL-P1-009 | build无admission/cancel/resource budget | 四个request顺序调用子PowerShell，无job/lease/timeout | build scheduler提供queue、cancel、timeout、CPU/RAM/disk/process预算与operation receipt |
| MVPCTL-P1-010 | product规格硬编码为四项Windows artifact | module常量固定runtime/editor exe+dll | TargetProfile/Role/Configuration registry生成build matrix和expected products |
| MVPCTL-P1-011 | staging记录的toolchain可能不是build toolchain | workflow/Invoke在后续阶段调用当前`rustc -Vv` | 只消费builder签发的ToolchainSetId，不允许stager补写build事实 |
| MVPCTL-P1-012 | CI先build binary又由input builder重复build | workflow前段Cargo build，后段`Build-MvpProductInputs`再次构建 | BuildGraph节点产物复用；同一receipt供test/stage/package消费 |

### 4.2 Staging input、Scenario与过程状态

| ID | 差距 | 当前证据 | 目标合同 |
| --- | --- | --- | --- |
| MVPCTL-P1-013 | input遍历静默跳过reparse entry | `Get-MvpOperationalFileList`对file/dir reparse只不加入结果 | required input遇reparse直接fail并记录path/kind；不得无声缺文件 |
| MVPCTL-P1-014 | derived-state排除列表硬编码 | `.zircon/autosave/cache/play/registry/thumbnails`写死在Stage | project schema声明source/derived/generated ownership与copy policy |
| MVPCTL-P1-015 | stage input与产品输出共用可变root | binary、project、logs、captures、summary都进入同一tree | immutable input mount + isolated writable work/output roots |
| MVPCTL-P1-016 | initial staging manifest不是最终phase graph | manifest写完后root持续被产品和driver修改 | 每阶段独立input/output manifest，以RunGraph edge连接且不可回写上游 |
| MVPCTL-P1-017 | automation request没有schema/version | 两个JSON只有bindings数组 | versioned `ScenarioSpec`，有migration、unknown-field policy和schema validator |
| MVPCTL-P1-018 | automation selector与payload是固定UI细节 | cube node id 3、固定control path、X=42、scale=1.25 | stable semantic selector/CommandId、precondition、expected transition与fallback policy |
| MVPCTL-P1-019 | 没有capability/scenario registry | driver内按固定顺序写死create/render/edit/reopen | ScenarioRegistry声明capability、roles、steps、oracle、artifacts、variants和owner |
| MVPCTL-P1-020 | repetition与timeout是局部整数参数 | RunCount范围1-4，product timeout统一值 | per-scenario statistical/reliability policy，timeout由step与device class决定 |

### 4.3 Product process supervisor与日志

| ID | 差距 | 当前证据 | 目标合同 |
| --- | --- | --- | --- |
| MVPCTL-P1-021 | child继承几乎全部parent environment | ProcessStartInfo默认继承，只移除少数`ZIRCON_*` | deny-by-default env allowlist，记录每项来源/敏感性/digest |
| MVPCTL-P1-022 | process journal缺关键identity | 只有phase、start/end、exit、outcome | RunId、PID+creation identity、exe hash、args、cwd、env digest、BuildSet/ProductReceipt/Scenario IDs |
| MVPCTL-P1-023 | journal在进程结束后才append一条 | crash/power loss时没有start intent或中间状态 | append-only start/heartbeat/exit/cleanup events，sequence与hash chain可恢复 |
| MVPCTL-P1-024 | stdout/stderr无限保存在内存 | 两个`ReadToEndAsync()` | bounded ring buffer + streaming artifact writer + dropped-byte counters |
| MVPCTL-P1-025 | 日志完成后才整体落盘 | `WriteAllText`在WaitForExit之后 | 实时flush、rotation、structured event stream和tail cursor |
| MVPCTL-P1-026 | diagnostics整树/整文件读取无预算 | 多个`ReadAllText/ReadAllBytes/Get-Content -Raw` | schema先验size/depth/count限制，stream parser和总artifact quota |
| MVPCTL-P1-027 | Stage没有Job Object | 只保存root `Process` | 复用`RenderExtractProcessJob`抽象，Windows Job kill-on-close与resource limits |
| MVPCTL-P1-028 | cleanup用CIM path筛选和裸PID | `Win32_Process.ExecutablePath.StartsWith(stage)` + `taskkill` | supervisor持有process creation identity和job membership，不依赖全机扫描 |
| MVPCTL-P1-029 | helper可通过外部路径逃逸 | 只扫描exe path位于staging的进程 | descendant containment按job/process group，不按image location |
| MVPCTL-P1-030 | 无heartbeat与phase liveness | 只有总timeout/exit | typed heartbeat、startup-ready、frame progress、save completion和hang classifier |
| MVPCTL-P1-031 | crash没有dump/symbol/build linkage | stdout/stderr与exit code是主要失败证据 | crash dump、module list、symbol IDs、ProductReceipt和structured crash event |
| MVPCTL-P1-032 | staging失败没有完整aborted run receipt | throw后可留下partial work/log，缺统一terminal manifest | 无论success/fail/cancel/timeout均原子发布bounded terminal receipt与cleanup outcome |

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
| MVPCTL-P1-042 | build summary gate表硬编码且whole-file读取 | module固定9个command并`ReadAllBytes` | GateRegistry/RunGraph receipt；bounded streaming schema reader |
| MVPCTL-P1-043 | JSON validator手写且默认允许unknown property | property-by-property取值、强制cast散落31个脚本 | 单一IDL/schema生成reader/writer/validator，strict unknown-field和精确numeric type |
| MVPCTL-P1-044 | 多个局部`schema_version = 1`无迁移系统 | 39处schema marker，不共享registry/migration | SchemaId/version/compatibility window/migrator/fixture corpus统一管理 |
| MVPCTL-P1-045 | snapshot对每个entry持句柄并全树复制/hash | deep no-follow递归与marker stream，规模随文件数线性增长 | CAS/clone/hardlink按policy、分段manifest与bounded handle pool |
| MVPCTL-P1-046 | acceptance snapshot没有item/byte/depth/time预算 | 任何合规大树都可耗尽handle、disk或时间 | admission先算预算；超限fail with bounded inventory，支持分片/流式copy |
| MVPCTL-P1-047 | evidence只有self hash，没有外部attestation | manifest和artifact由同一local run写 | CI/OIDC或worker签名、trusted timestamp、source/build/run/policy identities |
| MVPCTL-P1-048 | 没有Evidence Catalog、retention与promotion | 本地approved drive + CI 7天upload | immutable catalog、retention class、legal/security scrub、review/promotion/revoke receipt |

### 4.6 CI、test runner与currentness

| ID | 差距 | 当前证据 | 目标合同 |
| --- | --- | --- | --- |
| MVPCTL-P1-049 | workflow环境可漂移 | `windows-latest`、Rust `stable`、action major tags | pinned runner image/toolchain/action digest与environment receipt |
| MVPCTL-P1-050 | required lane不运行控制面focused suites | workflow只跑Cargo feature/F0-F5 product路径 | AST/schema/unit/failure/containment/resource/report suites成为required graph nodes |
| MVPCTL-P1-051 | test语义依赖Pester版本和输出换行 | 81/84；regex换行、Pester3 binding/StrictMode差异 | pin Pester/PowerShell，structured exception assertion，结果schema与exact case count |
| MVPCTL-P1-052 | 脚本级test能exit 0但TotalCount为0 | `mvp-staging.Tests.ps1`单跑104秒、Pester0 case | 全部case注册到统一runner，支持filter、timeout、JUnit、per-case artifact和coverage |

### 4.7 Resource-management观测与统计

| ID | 差距 | 当前证据 | 目标合同 |
| --- | --- | --- | --- |
| MVPCTL-P1-053 | baseline plan没有执行/观测owner | 只生成9个scenario与project inventory | `ResourceObservationRunner`启动真实product、执行query、采集trace并签发observation |
| MVPCTL-P1-054 | observation不绑定product/process/environment | caller只声明source/plan/inventory/attempt/counters | ProductReceipt、RunId、process creation、machine/CPU/RAM/OS、collector/trace/frame identity |
| MVPCTL-P1-055 | 统计模型不足以做回归判定 | 3-20 repetitions；min/median/p95/max/mean/total | warmup、sample sufficiency、noise/outlier/confidence/effect size与raw samples |
| MVPCTL-P1-056 | 没有baseline comparator与budget | reporter只汇总单次输入 | approved baseline cohort、absolute/relative budget、significance与regression decision |
| MVPCTL-P1-057 | workload固定1/1K/100K和一种Data asset | 三个规模、cold/stable/change、固定query集合 | extensible workload/scenario schema，覆盖asset kinds、dependency graph、tag/cardinality与mixed query |
| MVPCTL-P1-058 | cache/cold/warm协议没有系统控制 | mode名称不证明OS cache/DDC/index状态 | cache provenance、purge/prime receipt、warmup phase、order randomization和machine quiescence |
| MVPCTL-P1-059 | report只输出摘要Markdown且无gate verdict | 表格仅Median/P95，无threshold/status | machine-readable comparison、decision reason、trend/bisect links与human summary |
| MVPCTL-P1-060 | RenderExtract与MVP/resource重复拥有process/evidence primitives | 同目录存在Job/FrozenInput/Evidence与另一套Stage process/manifest | 收敛统一Runner/Observation/Evidence库；Render、MVP、resource只提供scenario plugin |

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
| production/test修复 | pending | 本轮review-only，无代码或测试修改 |
| F5 current qualification | blocked | plan未完成、compile blocker、control-plane suite RED、无clean promoted artifact |
| resource measured baseline | blocked | 无受信observation producer，现有reporter不得发布measured结论 |
