---
related_code:
  - .github/workflows/ci.yml
  - .github/workflows/mvp-editor-windows.yml
  - .github/workflows/profile-feature-contract.yml
  - .codex/config.toml
  - .codex/hooks/pre_tool_use_cargo_guard.py
  - .codex/hooks/zircon_session_sync.py
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1
  - .codex/skills/zircon-project-skills/close-session-goal-milestones/scripts/check-closeout.ps1
  - .codex/skills/zircon-project-skills/close-session-goal-milestones/scripts/read-closeout-evidence.py
  - .codex/skills/zircon-project-skills/write-plan-output-records/scripts/audit_plan_output_records.py
  - tools/zircon_build.py
  - tools/zircon_export/__main__.py
  - tools/zircon_export/cli.py
  - tools/session_coordinator/__main__.py
  - tools/session_coordinator/cli.py
  - tools/session_coordinator/soak.py
  - tools/session_coordinator/run-control-validation.ps1
  - tools/zircon-session.ps1
  - tools/build-editor.ps1
  - tools/check-conventions.ps1
  - tools/check_conventions.py
  - tools/runtime_domain_dependency_audit.py
  - tools/runtime-profile-feature-presets.py
  - tools/dev-fast-build.ps1
  - tools/dev-fast-aliases.ps1
  - tools/dev-module-interactive.ps1
  - tools/cleanup-stale-targets.ps1
  - tools/install-codex-session-hook.ps1
  - tools/install-session-coordinator-task.ps1
  - tools/install-session-tray-startup.ps1
  - tools/mvp/Build-MvpProductInputs.ps1
  - tools/mvp/Stage-MvpProducts.ps1
  - tools/mvp/Invoke-MvpAcceptance.ps1
  - tools/mvp/Capture-RenderExtractBaseline.ps1
  - tools/profile-capture-manifest.ps1
  - tools/ui-profile-capture.ps1
  - tools/ui-profile-process-evidence.ps1
  - tools/zircon_profile_shader_pbr_viewer.ps1
  - tools/zircon_summarize_shader_pbr_profile.py
  - tools/zircon_validate_shader_pbr_gpu_timing_evidence.py
  - tools/zircon_validate_shader_pbr_renderdoc_replay.py
  - tools/zircon_validate_shader_pbr_viewer_evidence.py
  - examples/woc/tools/package.json
  - examples/woc/tools/package-lock.json
  - examples/woc/tools/reference_inventory.mjs
  - examples/woc/tools/command_codegen.mjs
  - tools/editor-workbench-preview/package.json
  - tools/editor-workbench-preview/package-lock.json
  - tools/session_coordinator/web/package.json
  - tools/session_coordinator/web/package-lock.json
  - zircon_hub/package.json
  - zircon_hub/package-lock.json
  - docs/ui-and-layout/ai-workbench-style/component-prototype/package.json
tests:
  - tools/tests/dev-fast-build.Tests.ps1
  - tools/tests/mvp_editor_windows_workflow.Tests.ps1
  - tools/tests/test_check_conventions.py
  - tools/tests/test_frameworks_06_ci_toolchain_contract.py
  - tools/tests/test_zircon_build_cargo_environment.py
  - tools/tests/test_zircon_export_cli_owner_boundaries.py
  - tools/tests/test_zircon_summarize_shader_pbr_profile.py
  - tools/session_coordinator/tests/test_milestone_cli.py
  - tools/session_coordinator/tests/test_powershell_wrapper_arguments.py
  - tools/session_coordinator/tests/test_soak.py
plan_sources:
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_tooling/01-workspace-toolchain-ci-validation-and-developer-entrypoints-review.md
  - docs/plans/optimize/zircon_tooling/03-export-preset-build-cook-pack-platform-bundle-release-review.md
  - docs/plans/optimize/zircon_tooling/05-woc-content-codegen-build-scripts-generated-artifact-incremental-review.md
  - docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md
  - docs/plans/optimize/zircon_tooling/07-performance-benchmark-profile-capture-symbol-crash-evidence-baseline-review.md
  - docs/plans/optimize/zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md
  - docs/plans/optimize/zircon_tooling/13-repository-codex-skill-hook-structural-audit-governance-security-currentness-review.md
  - docs/plans/optimize/zircon_tooling/14-editor-workbench-design-spec-screenshot-export-visual-evidence-prototype-governance-review.md
  - docs/plans/optimize/zircon_tooling/15-mvp-build-staging-product-process-acceptance-evidence-resource-baseline-control-plane-review.md
  - docs/plans/optimize/zircon_tooling/16-capability-truth-placeholder-noop-fallback-degraded-qualification-control-plane-review.md
  - docs/plans/optimize/zircon_tooling/17-repository-content-source-set-ignore-generated-vendor-license-distribution-review.md
  - docs/plans/optimize/zircon_tooling/18-executable-target-entrypoint-cli-process-receipt-product-qualification-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Programs/AutomationTool/AutomationUtils/BuildCommand.cs
  - dev/UnrealEngine/Engine/Source/Programs/AutomationTool/AutomationUtils/Automation.cs
  - dev/UnrealEngine/Engine/Build/InstalledEngineBuild.xml
  - dev/godot/SConstruct
  - dev/godot/methods.py
  - dev/bevy/tools/ci/src/main.rs
  - dev/bevy/tools/ci/src/commands/mod.rs
  - dev/Fyrox/fyrox-build-tools/src/lib.rs
  - dev/Fyrox/fyrox-build-tools/src/build.rs
  - dev/Graphics/.yamato/wrench/wrench_config.json
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Tooling 19：Script Entrypoint、Interpreter、Command Registry、CLI 与 Operation Receipt 审查

## 1. 结论

本轮审查全部tracked PowerShell/Python/CMD/BAT/Shell/Node脚本形态、5个Node package的323条script、3个GitHub workflow的33个`run`步骤，以及跨域直接调用入口。物理仓库排除`dev`后有85个PS1、21个PSM1、1,170个Python、3个Shell、383个JS、423个MJS、41个TS和84个TSX。扩展名不等于入口：36个PS1是Pester测试，PSM1主要是库，621个含`__main__`标记的Python中绝大多数是测试。排除`tests`/fixture和`test_*`后，本轮确认23个直接Python入口、49个非测试PS1、3个Shell skill helper、323个npm command alias及33个workflow shell block是需要纳入命令控制面的候选集合。

现有脚本不是普遍低质量。49个非测试PS1全部有参数块，常用`CmdletBinding`、`Set-StrictMode`与`ErrorActionPreference=Stop`；23个直接Python入口中20个采用`argparse`、18个处理JSON、全部通过`sys.exit`/`SystemExit`返回terminal status。`zircon_export`、Session Coordinator和`zircon_build`已经把入口与大部分domain逻辑拆开；安装脚本提供`SupportsShouldProcess`/`DryRun`，WOC 304个npm alias大量成对提供`generate:*`与`check:*`，4个执行型Node项目有lockfile。这些机制应进入统一registry，不应被改写成一个巨型脚本宿主。

缺口是脚本“可被调用”仍由文件路径、npm alias、workflow自由文本、README命令或本机wrapper决定，没有稳定CommandId、kind、owner、interpreter/tool dependency、input/output、capability、mutation scope、consumer、CLI schema、exit domain或qualification profile。`.codex` hook、build/export、Coordinator、MVP、installer、cleanup和evidence validator都能改变代码、进程、计划、系统启动项、artifact或资格状态，但调用方无法从一个canonical manifest判断这是纯查询、生成器、安装器、控制面、证据producer还是测试fixture。

当前也没有跨语言InterpreterSet和OperationReceipt。仓库没有tracked `pyproject.toml`、requirements/lock或`.python-version`；`zircon_build.py`在代码中要求Python 3.11，CI只有部分job显式安装3.11，其余`python`取环境默认。106个PS1/PSM1没有一处`#requires -Version`；Node package都没有`engines`/`packageManager`。所有2,210个script-like tracked file都是Git mode 100644，3个Shell helper能否直接执行依赖caller显式选择shell。另有11个本机`tools/dev-*.cmd` wrapper存在但全部ignored/untracked，clean clone没有这些入口；source membership由Tooling01/17拥有，本篇只记录catalog无法区分它们与受控命令。

参数、输出与终态也没有统一automation contract。49个非测试PS1混用`Write-Host`、`Write-Output`、`ConvertTo-Json`、throw与少量`exit 0/1`；23个Python入口混用人类文本、JSON、`return 0/1/2/3`与异常；npm alias把多条命令用shell `&&`串联，workflow又直接嵌入shell。现有局部协议可以保留，但“process 0退出”不能证明artifact原子提交、evidence有效、所有子步骤执行、没有fallback或调用的是同代source/interpreter。

本篇登记 **2项P0、54项P1、12项P2**。它只拥有ScriptEntrypointManifest、InterpreterSet、CommandInvocation、MutationScope、ScriptOperationReceipt与跨语言CLI/exit协议；workspace/toolchain由Tooling01拥有，export由Tooling03拥有，WOC codegen由Tooling05拥有，Coordinator由Tooling06拥有，performance evidence由Tooling07拥有，test discovery由Tooling10拥有，Codex hook由Tooling13拥有，Workbench/MVP由Tooling14/15拥有，source membership由Tooling17拥有，Rust executable由Tooling18拥有。本轮没有修改任何脚本、workflow、package、lock、production或test。

## 2. 物理清单与入口分类

### 2.1 语言与载体规模

| 载体 | tracked规模 | 直接入口候选 | 当前解释 |
|---|---:|---:|---|
| PowerShell | 85 PS1 / 21 PSM1 | 49非测试PS1 | 36个Pester；PSM1为module，不按文件全部算命令 |
| Python | 1,170 files | 23过滤后直接入口 | 621含main marker，主要是343个`tools/tests`、159个export tests和97个Coordinator tests |
| Shell | 3 SH | 3 skill helper | 都在`.codex/skills`，无通用产品shell入口 |
| Node source | 383 JS / 423 MJS / 41 TS / 84 TSX | 323 npm aliases + 少数直接node文件 | WOC占304个aliases；大量文件是module/test/generated bundle |
| GitHub Actions | 3 workflows | 33 `run` blocks | 另有第三方action，不按script入口重复计数 |
| CMD/BAT | 0 tracked | 11 ignored/untracked local CMD | dev-fast wrapper只在当前工作树存在，clean clone不可达 |

所有script-like tracked文件mode均为100644。这对由`python file`、`pwsh -File`、`node file`调用的脚本并非错误；问题是manifest没有声明合法interpreter/argv，所以Unix direct execution、Windows association和CI shell选择只能由caller猜测。

### 2.2 直接Python入口

| 家族 | 数量 | 代表入口 | 已有owner |
|---|---:|---|---|
| Repo/Codex control | 9 | cargo guard、session sync、plan/structure audit | Tooling13 |
| Root build/validation | 5 | `zircon_build`、conventions、domain/profile audit | Tooling01/03 |
| Coordinator | 2 | CLI、soak | Tooling06/07 |
| Export | 1 public module entry | `python -m tools.zircon_export` | Tooling03 |
| PBR/render evidence | 5 | profile summary、GPU timing、RenderDoc、viewer evidence、measurement validator | Tooling07/App02 |
| Historical plan utility | 1 | RenderDoc capture audit | Tooling07/12 |

这23项中20项使用`argparse`、18项读写JSON、7项启动subprocess、9项具有写盘/删除/生成候选行为。它们的内部正确性不在本篇重复审查；本篇关注caller能否在执行前知道command identity、interpreter、权限与mutation scope，并在执行后得到typed terminal receipt。

### 2.3 PowerShell与系统操作

49个非测试PS1全部有参数块；29项包含JSON转换、19项包含process launch候选、19项包含写盘/复制/删除/安装候选。代表性高影响入口包括：

1. `install-session-coordinator-task.ps1`可安装/更新/删除Scheduled Task或用户启动项，并提供ShouldProcess/DryRun；
2. `install-session-tray-startup.ps1`写HKCU Run并验证安装/删除；
3. `cleanup-stale-targets.ps1`清理受管target；
4. `zircon-session.ps1`桥接Python Coordinator、stdin/JSON和timeout；
5. MVP脚本构建、stage、启动产品并生成acceptance/evidence；
6. capture/profile脚本创建GPU/UI/performance evidence；
7. dev-fast/build脚本解析profile后启动Cargo。

这些命令已有若干保护，但没有统一`MutationScope::{None, Workspace, ArtifactStore, ProcessTree, UserProfile, System}`声明，也没有principal/admission/rollback/receipt字段。`SupportsShouldProcess`只解决交互确认，不证明automation authorization或事务完成。

### 2.4 Node与workflow命令面

| package | scripts | 当前特点 |
|---|---:|---|
| WOC tools | 304 | 大量generate/check pairs，也有超长聚合`check`/`generate` shell chain |
| Editor Workbench preview | 5 | start/export/verify及negative reference check |
| Session Coordinator web | 5 | dev/typecheck/test/build/check |
| Zircon Hub | 5 | dev/build/typecheck/tauri dev/build |
| docs component prototype | 4 | start/dev/serve/check；没有lockfile |

5个package均private；除docs prototype外都有package-lock，但全部没有Node `engines`和`packageManager`。`npm run check`的名字在不同package分别表示不同stage和evidence强度，不能仅按alias名聚合资格。WOC的长`&&`链可以正确短路，却没有step inventory/receipt说明后续多少项被omitted；当前已知typed contract首项失败会阻断其余21个step，就是“失败传播正确但结果完整性未知”的例子。

## 3. P0：先建立脚本命令与执行资格真实性

### TOOL-SCRIPTENTRY-P0-001 · 高影响脚本没有canonical CommandId/registry/admission，文件路径和alias成为隐式权限边界

build、export、install、cleanup、Coordinator、Codex hook、MVP、codegen和evidence producer分散在PowerShell、Python、Node package与workflow自由文本中。caller通过相对路径、模块名、npm alias或复制命令选择入口，不能机器验证其kind、owner、support tier、mutation scope、required principal、consumer、deprecation或replacement。11个本机ignored CMD还能表现为“可用入口”，clean clone却完全不存在。新增/改名/复制脚本不会自动触发command catalog、CI reachability或security review。

硬切建立versioned `ScriptEntrypointManifest`：每个可直接调用的脚本/alias/workflow command登记稳定CommandId、kind、owner、source entry、interpreter、argv schema、capability、mutation scope、consumer、terminal protocol和qualification profile；测试/module/generated bundle不进入public command集合。CI比较Git tree、package scripts、workflow调用和manifest，未登记高影响入口、orphan consumer、本地only command或重复alias identity时fail closed。具体操作权限仍由Tooling06/13/15等domain owner执行。

### TOOL-SCRIPTENTRY-P0-002 · Interpreter、source、invocation与terminal result没有同代OperationReceipt，成功输出不能证明可复现或完成

同一命令的行为由本机`python`/PowerShell/Node/npm、当前PATH、cwd、env、lock install状态和自由shell解析共同决定。仓库没有统一Python environment或PowerShell version，Node缺engines/packageManager；部分CI固定Python 3.11，普通wrapper直接解析系统`python`。终态又混用throw、return code、JSON、human text和shell short-circuit。当前无法证明生成artifact/evidence的interpreter/tool dependencies、输入、实际子步骤、omitted项、输出digests和negative states，也无法将process exit 0安全提升为Qualified。

硬切建立`InterpreterSetReceipt -> CommandInvocation -> ScriptOperationReceipt`链，绑定CommandId、SourceSet/BuildSet、interpreter executable/version/digest、dependency lock、canonical argv/env/cwd、principal、input digests、step inventory、stdout/stderr protocol、output digests、mutation commit/rollback和terminal category。任何missing interpreter constraint、unlocked dependency、omitted required step、partial write、fallback或unstructured success只能产生Unqualified/Partial，不得进入artifact/evidence/release promotion。

## 4. P1：跨语言脚本控制面重构

### 4.1 Command catalog与identity

1. **TOOL-SCRIPTENTRY-P1-001**：定义稳定`CommandId`，不得从文件basename、npm alias或workflow step name临时推导。
2. **TOOL-SCRIPTENTRY-P1-002**：定义`CommandKind::{Query, Build, Validate, Generate, Package, Install, ControlPlane, Evidence, Migration, Cleanup, Hook}`。
3. **TOOL-SCRIPTENTRY-P1-003**：每项记录owner、support tier、public/internal、consumer、platform和replacement/deprecation policy。
4. **TOOL-SCRIPTENTRY-P1-004**：manifest区分entrypoint、library/module、test、fixture和generated bundle，禁止扩展名推断角色。
5. **TOOL-SCRIPTENTRY-P1-005**：扫描PS1/Python main/npm scripts/workflow commands并与manifest双向比较，未分类项明确输出。
6. **TOOL-SCRIPTENTRY-P1-006**：alias只引用CommandId及固定argument projection，不复制完整shell command。
7. **TOOL-SCRIPTENTRY-P1-007**：相同名字的`check/build/start`按namespace区分，CI不得按裸alias聚合完成度。
8. **TOOL-SCRIPTENTRY-P1-008**：本机ignored wrapper只可作为DeveloperState；文档/CI/required workflow不得依赖其存在。
9. **TOOL-SCRIPTENTRY-P1-009**：command removal保留tombstone、最后支持版本与替代入口，历史receipt仍可解释。

### 4.2 Interpreter、dependency与source closure

10. **TOOL-SCRIPTENTRY-P1-010**：定义`InterpreterId`和最低/最高兼容版本、platform/architecture与invocation mode。
11. **TOOL-SCRIPTENTRY-P1-011**：Python命令使用受控environment/lock；纯stdlib也显式声明并由import audit验证。
12. **TOOL-SCRIPTENTRY-P1-012**：`zircon_build.py`的3.11要求进入InterpreterSet，wrapper与CI不再靠运行时ImportError发现。
13. **TOOL-SCRIPTENTRY-P1-013**：PowerShell命令声明Desktop/Core版本、edition、Windows-only module和execution policy要求。
14. **TOOL-SCRIPTENTRY-P1-014**：Node package声明engines与packageManager，执行前验证lockfile和frozen install receipt。
15. **TOOL-SCRIPTENTRY-P1-015**：Shell helper声明bash/POSIX方言；100644文件统一经解释器调用，不依赖checkout executable bit猜测。
16. **TOOL-SCRIPTENTRY-P1-016**：CommandArtifact绑定source content digest及所有import/dot-source/module dependency，不只hash入口文件。
17. **TOOL-SCRIPTENTRY-P1-017**：workflow使用同一InterpreterSet projection，不能另行选择浮动`stable`/system runtime。
18. **TOOL-SCRIPTENTRY-P1-018**：依赖安装、cache和tool download进入BuildSet/Action digest，联网获取不得发生在未声明命令内部。

### 4.3 参数、环境、路径与权限

19. **TOOL-SCRIPTENTRY-P1-019**：argv由versioned schema描述类型、required/default、mutual exclusion、unknown flag和response file政策。
20. **TOOL-SCRIPTENTRY-P1-020**：PowerShell/Python/Node parser生成或验证同一schema，文档不再手写第二套合法值。
21. **TOOL-SCRIPTENTRY-P1-021**：cwd是显式Workspace/Project/Artifact root identity；脚本不得静默从调用位置猜测不同owner。
22. **TOOL-SCRIPTENTRY-P1-022**：环境变量采用allowlist、类型、secret/redaction和inheritance policy，记录canonical digest。
23. **TOOL-SCRIPTENTRY-P1-023**：路径经过统一canonicalization、containment、symlink/reparse、case与Windows长路径政策。
24. **TOOL-SCRIPTENTRY-P1-024**：定义MutationScope并在执行前由principal/capability policy admission；查询命令保证无副作用。
25. **TOOL-SCRIPTENTRY-P1-025**：install/task/registry操作记录用户/机器scope、目标identity、previous value和rollback receipt。
26. **TOOL-SCRIPTENTRY-P1-026**：hook stdin payload和stdout decision使用versioned schema、bytes/depth预算和unknown-field政策。
27. **TOOL-SCRIPTENTRY-P1-027**：secret/token/path/command line在human log、JSON、exception和evidence中统一redact。

### 4.4 Operation、subprocess与事务

28. **TOOL-SCRIPTENTRY-P1-028**：每次调用生成OperationId，记录parent/child CommandId、attempt、deadline和cancel token。
29. **TOOL-SCRIPTENTRY-P1-029**：subprocess使用argument list和显式shell policy；仅需要管道/重定向时允许shell并记录方言。
30. **TOOL-SCRIPTENTRY-P1-030**：stdout/stderr并发有界pump，避免deadlock、无限内存和丢失terminal tail。
31. **TOOL-SCRIPTENTRY-P1-031**：child process tree由supervisor持有，timeout/cancel/shutdown传播到全部后代。
32. **TOOL-SCRIPTENTRY-P1-032**：多步npm/workflow/PowerShell链生成step inventory，失败后明确Passed/Failed/Omitted/Cancelled。
33. **TOOL-SCRIPTENTRY-P1-033**：generate/write/install/delete采用preflight、stage、commit、verify与rollback，失败不暴露半成品。
34. **TOOL-SCRIPTENTRY-P1-034**：DryRun执行同一解析/admission/plan路径，只跳过commit，不能维护第二套近似逻辑。
35. **TOOL-SCRIPTENTRY-P1-035**：idempotent command声明idempotency key和NoChange语义；retry不能重复安装、追加或删除。
36. **TOOL-SCRIPTENTRY-P1-036**：外部工具调用记录resolved executable、version、digest和working set，PATH同名替换fail closed。

### 4.5 输出、错误、exit与receipt

37. **TOOL-SCRIPTENTRY-P1-037**：声明machine mode时stdout只承载schema-versioned event/terminal envelope，human log进入stderr。
38. **TOOL-SCRIPTENTRY-P1-038**：PowerShell host/progress/color输出与pipeline data分离，redirect后不改变业务结果。
39. **TOOL-SCRIPTENTRY-P1-039**：跨语言ExitDomain区分Usage、Rejected、Drift、Failed、Partial、Cancelled与InternalFault。
40. **TOOL-SCRIPTENTRY-P1-040**：保留命令私有numeric code，但必须稳定映射category、retryability和mutation outcome。
41. **TOOL-SCRIPTENTRY-P1-041**：uncaught Python/PowerShell/Node exception产生typed internal fault和durable diagnostic locator。
42. **TOOL-SCRIPTENTRY-P1-042**：ScriptOperationReceipt记录所有required step及omitted原因，不能把短路链的局部结果称为全套结果。
43. **TOOL-SCRIPTENTRY-P1-043**：生成器记录input/output digest、schema、generator/interpreter和atomic commit；文本“wrote”不是artifact receipt。
44. **TOOL-SCRIPTENTRY-P1-044**：evidence producer记录observer、workload、clock/device/source和qualification profile，caller JSON不能自认证。
45. **TOOL-SCRIPTENTRY-P1-045**：receipt写入CAS/指定artifact并原子提交；stdout只输出小型envelope或locator。

### 4.6 Test、CI、currentness与治理

46. **TOOL-SCRIPTENTRY-P1-046**：registry test证明全部49 PS1、23 Python entry、323 npm alias和workflow commands完成分类或显式排除。
47. **TOOL-SCRIPTENTRY-P1-047**：每个public command测试help/version/bad args/success/typed failure/cancel和unknown protocol。
48. **TOOL-SCRIPTENTRY-P1-048**：高影响command增加sandbox/fault tests，覆盖permission、disk full、partial write、stale generation和rollback。
49. **TOOL-SCRIPTENTRY-P1-049**：跨Windows/Linux/macOS验证支持的interpreter组合；unsupported平台在admission阶段拒绝。
50. **TOOL-SCRIPTENTRY-P1-050**：CI从registry生成command matrix，新增入口若无lane/owner/qualification即失败。
51. **TOOL-SCRIPTENTRY-P1-051**：Node长链拆为可观测DAG或runner，不要求把304个业务alias全部合并成单命令。
52. **TOOL-SCRIPTENTRY-P1-052**：source fingerprint变化使CommandArtifact/receipt自动失效；历史成功不能继续代表当前脚本。
53. **TOOL-SCRIPTENTRY-P1-053**：记录调用频率、duration、failure class和orphan/deprecated consumer，telemetry不包含secret argv。
54. **TOOL-SCRIPTENTRY-P1-054**：Tooling10/15/16只接受完整ScriptOperationReceipt；partial/omitted/fallback/local-only入口不计required green。

## 5. P2：开发体验与生态成熟度

1. **TOOL-SCRIPTENTRY-P2-001**：提供`cargo zircon command list/describe/run`作为registry前端，同时保留语言原生入口。
2. **TOOL-SCRIPTENTRY-P2-002**：从argv schema生成PowerShell/bash completion和机器可读help。
3. **TOOL-SCRIPTENTRY-P2-003**：生成按owner/kind/platform/support tier组织的命令文档，避免README副本漂移。
4. **TOOL-SCRIPTENTRY-P2-004**：为常用开发操作提供短alias，但alias必须可追溯CommandId和版本。
5. **TOOL-SCRIPTENTRY-P2-005**：提供本地environment doctor，报告Python/PowerShell/Node/npm与外部工具差异。
6. **TOOL-SCRIPTENTRY-P2-006**：跨语言structured log使用一致RunId/OperationId，保留各工具自己的domain字段。
7. **TOOL-SCRIPTENTRY-P2-007**：命令历史可查询source/interpreter/result/artifact，不记录明文secret或完整敏感argv。
8. **TOOL-SCRIPTENTRY-P2-008**：CI展示command catalog增删、consumer和shipping/support影响差异。
9. **TOOL-SCRIPTENTRY-P2-009**：支持response file和stdin artifact，避免Windows命令行长度与shell quoting问题。
10. **TOOL-SCRIPTENTRY-P2-010**：提供跨语言golden harness，验证stdout/stderr/exit/receipt而不锁死人类文案。
11. **TOOL-SCRIPTENTRY-P2-011**：废弃命令在兼容期输出structured warning和replacement，不静默重定向到不同mutation scope。
12. **TOOL-SCRIPTENTRY-P2-012**：为安装/清理/迁移操作生成可读plan和post-condition摘要，仍以receipt为机器真相。

## 6. 目标架构

### 6.1 核心对象

| 对象 | 最小字段 | 不得替代为 |
|---|---|---|
| `ScriptEntrypointDescriptor` | CommandId、kind、owner、entry/interpreter、argv、capability、mutation、consumer、protocol | path、basename、npm alias、README命令 |
| `InterpreterSetReceipt` | executable/version/digest、dependency mode/lock、platform、tool resolution | `python`/`node`/`powershell`在PATH可见 |
| `CommandInvocation` | CommandArtifact、principal、argv/env/cwd、inputs、deadline、parent operation | 拼接shell string或workflow自由文本 |
| `ScriptStepObservation` | step id、started/terminal、child process、outputs、omitted reason | 一段混合console log |
| `ScriptOperationReceipt` | source/interpreter/invocation、step inventory、mutation commit、outputs、terminal category、negative states | exit 0、`Write-Host Success`、非空JSON |

### 6.2 分层状态

命令声明与解析：

`Declared -> Resolved -> InterpreterQualified -> InputsValidated -> Admitted`

执行与提交：

`Started -> StepsObserved -> OutputsStaged -> MutationCommitted -> PostconditionVerified -> TerminalReceipt`

资格求值：

`TerminalReceipt + RequiredStepSet + NegativeStates -> Qualified | Partial | Rejected`

`Failed`、`Cancelled`、`TimedOut`和`Partial`均保留已执行step与rollback结果。对纯Query命令，MutationCommitted必须为NotApplicable且执行审计证明无写操作；对Install/Cleanup/Migration，必须有before/after identity和恢复策略。

## 7. 与已有报告的责任边界

| 脚本家族 | 本篇拥有 | canonical domain owner |
|---|---|---|
| build/dev/CI | command/interpreter/invocation/receipt envelope | Tooling01的workspace、toolchain、profile与CI语义 |
| export/package | Script CommandId与跨语言terminal协议 | Tooling03的validate/cook/pack/bundle/release正确性 |
| WOC 304 npm aliases | registry、step inventory、interpreter receipt | Tooling05及Runtime12-20的codegen/content/domain语义 |
| Coordinator/install/cleanup | mutation scope与command receipt接口 | Tooling06的auth/lease/process/Git/system operation |
| profile/capture/evidence | invocation与producer receipt envelope | Tooling07的measurement/oracle/statistics/evidence validity |
| repo Codex hooks/skills | hook command identity和schema接入 | Tooling13的permission/fail-open/governance |
| Workbench/MVP | command catalog与step completeness | Tooling14/15的visual/product/evidence qualification |
| ignored CMD wrapper | catalog不得视为source command | Tooling01 developer UX、Tooling17 SourceSet/ignore |
| Rust binaries | 不重复登记 | Tooling18 ExecutableTargetManifest |

## 8. 参考实现差异

| 参考 | 观察到的机制 | 对Zircon的约束 | 不外推的内容 |
|---|---|---|---|
| Unreal AutomationTool/BuildGraph | 集中发现BuildCommand、列出/help、逐命令执行并返回typed ExitCode；BuildGraph承载安装构建图 | 高影响automation需要可发现command registry、统一执行边界和graph/terminal identity | 不复制其反射发现、C#参数风格或全部AutomationTool历史复杂度 |
| Godot SCons | `SConstruct`集中注册/校验build options、platform支持和失败退出，helper在`methods.py`共享 | interpreter script也需要单一option/平台truth和早期compatibility rejection | 不要求Zircon改用SCons或单体build script |
| Bevy CI tool | 编译型`tools/ci`通过typed command modules组织compile/test/doc/bench/example等入口 | 大量CI操作应由typed catalog与模块化实现驱动，不靠workflow复制命令 | 不要求所有Zircon脚本立刻重写为Rust |
| Fyrox build tools | 可序列化CommandDescriptor/BuildProfile记录program、argv、env与build/run queue | invocation可序列化、验证、展示并由owner消费 | 其descriptor本身不等于安全admission、事务或qualification |
| Unity Graphics Wrench | schema化package目录、pre-test/pre-pack command与发布集合 | package命令矩阵应由versioned config生成并明确发布范围 | 本地镜像不证明Unity全产品脚本安全或CLI协议 |

## 9. 重构里程碑

### M0 · Inventory Freeze

- 固定49个非测试PS1、23个Python入口、323个npm alias、33个workflow run及3个Shell helper的分类口径；
- 显式标记test/module/generated/local-only，不把文件数当作command数；
- 只输出diagnostic，不改变调用行为。

### M1 · Registry 与 InterpreterSet

- 建立CommandId/Kind/MutationScope/consumer manifest；
- 为Python、PowerShell、Node、Shell声明版本、dependency与invocation；
- 先禁止新增未登记高影响入口和required local-only wrapper。

### M2 · CLI 与 Invocation Protocol

- 统一argv/env/cwd/principal/input identity；
- 从manifest生成wrapper、workflow和文档projection；
- unknown command/flag/interpreter/source generation fail closed。

### M3 · Operation Runner

- 接入process tree、bounded IO、deadline/cancel、step inventory和fault reporting；
- Query/Generate/Install/Cleanup采用不同mutation transaction；
- shell chain只作为受控step实现，不再是结果聚合器。

### M4 · Receipt 与 Domain迁移

- 优先迁移build/export/Coordinator/MVP/evidence等发布关键入口；
- WOC、Codex、Workbench随后按既有owner迁移；
- 保留语言原生入口作为registry adapter。

### M5 · CI/Qualification Hard Cutover

- CI从registry生成lane并比较consumer closure；
- required operation只接受同代完整receipt；
- 删除私有command副本、ignored required wrapper和自由文本状态解析。

## 10. 验收门

| Gate | 验收条件 |
|---|---|
| G01 | 全部直接脚本/npm/workflow命令恰好分类为entry/module/test/fixture/generated/local-only，0 unclassified |
| G02 | 每个public/high-impact command有唯一CommandId、owner、kind、interpreter、mutation scope、consumer和protocol |
| G03 | Python/PowerShell/Node/Shell interpreter与dependency版本可由clean clone/CI复现并产生receipt |
| G04 | required command不依赖11个ignored CMD或其他DeveloperState；clean clone入口集合一致 |
| G05 | wrapper、npm、workflow和文档由registry projection生成或校验，0 orphan/重复私有真相 |
| G06 | argv/env/cwd/path/principal admission在启动前完成，secret在全部输出面redact |
| G07 | subprocess tree、stdout/stderr、timeout/cancel和resource budget由统一runner管理 |
| G08 | 多步链完整报告Passed/Failed/Omitted/Cancelled，短路结果不能冒充全套执行 |
| G09 | generate/install/delete/migrate fault injection证明原子提交、postcondition和rollback |
| G10 | machine mode stdout、stderr、ExitDomain和terminal receipt通过跨语言golden |
| G11 | artifact/evidence receipt绑定CommandId、source、interpreter、inputs、steps和output digest |
| G12 | Tooling10/15/16只将完整、同代、无fallback/partial/local-only的ScriptOperationReceipt计为qualified |

## 11. 状态与产出记录

| 项目 | 状态 | 日期 | 证据 |
|---|---|---|---|
| Script-like物理清单 | review_complete | 2026-08-16 | 85 PS1、21 PSM1、1,170 Python、3 SH、931 JS/MJS/TS/TSX |
| 直接入口分类 | review_complete | 2026-08-16 | 49非测试PS1、23 Python、323 npm alias、33 workflow run、3 Shell helper、11 local-only CMD |
| CLI/interpreter/operation抽样 | review_complete | 2026-08-16 | 参数、JSON、process、mutation、exit、lock/version与consumer边界逐家族复核 |
| 参考automation对照 | review_complete | 2026-08-16 | Unreal AutomationTool、Godot SCons、Bevy CI tool、Fyrox build tools、Unity Wrench |
| ScriptEntrypoint/Interpreter/OperationReceipt设计 | design_complete | 2026-08-16 | 本篇第6节；尚未实现manifest、runner或迁移 |
| Production/script/workflow重构 | pending | - | 本篇不修改任何执行文件或配置 |
