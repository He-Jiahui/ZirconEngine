---
related_code:
  - .github/workflows/ci.yml
  - .github/workflows/mvp-editor-windows.yml
  - .github/workflows/profile-feature-contract.yml
  - Cargo.toml
  - zircon_hub/Cargo.toml
  - zircon_hub/package.json
  - zircon_hub/src/lib.rs
  - zircon_hub/tests/project_page_copy_contract.rs
  - zircon_hub/tests/project_workflow_contract.rs
  - zircon_hub/tests/tauri_react_shell_contract.rs
  - zircon_editor/src/tests/support.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/visual_screenshot/asset_browser_content.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/visual_screenshot/assets_drawer.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/welcome_visual_screenshot.rs
  - zircon_editor/tests/editor_world_sync_watch_map.rs
  - zircon_app/src/entry/runtime_library/tests.rs
  - zircon_app/tests/editor_mvp_authoring.rs
  - zircon_runtime/src/foundation/tests.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests/device_contract.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/gpu_readback_queue/tests.rs
  - zircon_runtime_interface/src/serialization/tests/write_contract.rs
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1
  - tools/check_conventions.py
  - tools/session_coordinator/web/package.json
  - tools/session_coordinator/web/scripts/run-tests.mjs
  - tools/session_coordinator/web/scripts/verify-dist.mjs
  - tools/editor-workbench-preview/package.json
tests:
  - .codex/skills/zircon-dev/scripts/validate-matrix.Tests.ps1
  - tools/tests/test_check_conventions.py
  - tools/tests/test_frameworks_06_ci_toolchain_contract.py
  - tools/tests/test_frameworks_06_dependency_governance_contract.py
  - tools/tests/mvp-acceptance.Tests.ps1
  - tools/tests/mvp-staging-release.Tests.ps1
  - tools/tests/session-coordinator-smoke.Tests.ps1
  - tools/tests/ui-profile-latency-evidence.Tests.ps1
  - tools/zircon_export/tests/test_pack_stage_cli.py
  - tools/zircon_export/tests/test_pipeline_report_compile_host_stage_schema.py
  - tools/session_coordinator/tests/test_action_execution.py
  - tools/session_coordinator/tests/test_validation_tickets.py
  - tools/session_coordinator/tests/test_workspace_copy.py
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_hub/01-project-engine-build-editor-launch-process-persistence-delivery-review.md
  - docs/plans/optimize/zircon_tooling/01-workspace-toolchain-ci-validation-and-developer-entrypoints-review.md
  - docs/plans/optimize/zircon_tooling/02-cargo-zircon-plugin-scaffold-manifest-validation-native-probe-review.md
  - docs/plans/optimize/zircon_tooling/03-export-preset-build-cook-pack-platform-bundle-release-review.md
  - docs/plans/optimize/zircon_tooling/05-woc-content-codegen-build-scripts-generated-artifact-incremental-review.md
  - docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md
  - docs/plans/optimize/zircon_tooling/07-performance-benchmark-profile-capture-symbol-crash-evidence-baseline-review.md
  - docs/plans/optimize/zircon_tooling/09-release-channel-artifact-repository-install-update-rollback-operations-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/AutomationTest.h
  - dev/UnrealEngine/Engine/Source/Programs/AutomationTool/Gauntlet/Framework/Base/Gauntlet.TestNode.cs
  - dev/bevy/tools/ci/src/ci.rs
  - dev/bevy/tools/ci/src/commands/test_check.rs
  - dev/bevy/tools/ci/src/commands/compile_fail.rs
  - dev/godot/tests/test_main.cpp
  - dev/godot/tests/test_macros.h
  - dev/Fyrox/.github/workflows/ci.yml
  - dev/Graphics/.yamato/wrench/validation-jobs.yml
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 10 · Test Architecture、Partition/Selection、Isolation/Fixture、Flake 与 Result 工程化差距

## 1. 结论

ZirconEngine拥有很大的测试源码体量，但还没有工程级测试系统。本轮排除`dev`参考源码、`docs/tests`证据文件、计划文档和非源码资产后，仍识别到5,582个测试源码文件、959,107行、35,916,766 bytes；其中Rust源码出现21,211个`#[test]`。这些数字只能说明仓库投入了大量测试代码，不能证明默认验证路径会执行它们、它们彼此隔离、失败可诊断，或一个绿色结果覆盖了发布所需能力。

最直接的反例在Hub：`zircon_hub/Cargo.toml`对库显式设置`test = false`，Cargo metadata也把该lib target标记为不可测试；但Hub的98个Rust source中有61个`#[cfg(test)]`模块和258个inline `#[test]`。因此默认`cargo test --workspace`不会构建或运行这258项单元测试。Hub的39个外部测试文件另有270项测试，但其中81次读取源码、189次使用`.contains()`断言，主要验证源码/文案形状，不能替代被关闭的行为测试。

CI也没有覆盖仓库的多语言测试面。三个workflow中，主CI只显式运行3个Python模块，随后执行约定检查和Cargo build/test；而`tools`相关Python测试共有659个文件、4,295个测试方法。36个`*.Tests.ps1`、Session Coordinator Web的自定义Node runner、Hub前端测试以及WOC的完整check链均没有通用CI lane。Hub `package.json`甚至没有`test`脚本或测试依赖。当前“workspace tests”主要是Rust命令名，不是全仓测试计划。

结果层也没有统一可信对象。仓库没有TestPlan/TestAttempt/TestResult/TestArtifact schema，没有required-test manifest、变更到测试的依赖映射、attempt identity、result currentness、JUnit/coverage汇总、flaky quarantine或release admission。此前轮次已经分别复现root/plugin/Hub编译阻断、export suite 1,642项中667项失败，以及Coordinator全量discovery约904秒超时；这些失败的owner不由本报告重复接管，但它们证明当前不存在一份能够表达“哪些required lanes实际执行、对哪个source/build执行、结果是否仍新鲜”的权威绿色验证记录。

测试隔离同样是局部而非系统能力。Editor等crate已有`TestEnvironmentLock`和若干RAII环境恢复器，这是应保留的正向基础；但它们只在单个test binary进程内生效。根Cargo metadata包含130个显式integration test target，测试又广泛修改`ZIRCON_CONFIG_PATH`、`SLINT_BACKEND`、`ZIRCON_RUNTIME_LIBRARY`等进程环境，并使用用户目录、临时目录、端口、子进程、GPU和固定证据路径。没有跨进程resource lease、唯一sandbox root、端口broker、虚拟时钟、device capability scheduler或统一process supervisor，局部mutex不能提供整仓隔离。

因此本轮给出三个P0：恢复Hub被关闭的258项inline单元测试并建立unreachable-test守卫；把遗漏的Python/PowerShell/Web/WOC/Hub前端纳入显式required test plan；建立source/build-bound、可证明完整性的统一测试结果与发布准入协议。在三者完成前，不得用测试源码数量、`cargo test --workspace`的命令存在、源码字符串合同、单个MVP smoke或局部绿色模块声称“全仓测试通过”。

本轮记录3个P0、52个P1和10个P2。未修改生产Rust、Python、PowerShell、JavaScript/TypeScript、Cargo manifest、workflow、测试实现或测试证据，只新增审查与索引。

## 2. 审查边界与证据

### 2.1 物理范围

| 子域 | 范围 | 本轮深度 |
|---|---:|---|
| 测试源码总体 | 5,582个源码文件 / 959,107行 / 35,916,766 bytes | E2全量inventory；按domain、语言、attribute、规模与测试形状统计 |
| Rust/Cargo | 161个manifest、36个package、184个target、130个integration test target | E3 manifest/metadata/default command/ignored/inline与integration边界 |
| Tool Python | 659个文件 / 4,295个test methods | E2全量统计；CI 3模块、export与Coordinator代表链E3并复用既有动态证据 |
| PowerShell/Web | 36个`*.Tests.ps1`、2个前端package、custom Node runner | E3 runner、workflow consumer、脚本形状与absence proof |
| 隔离与fixture | env/CWD/temp/path/port/subprocess/GPU/sleep/ignored tests | E2全量搜索；代表lock、visual、MVP、Coordinator链E3 |
| result与advanced testing | result artifact、coverage、property/fuzz、compile-fail、sanitizer、flake | E2 absence proof；目标协议E3 |
| reference engines | Unreal Automation/Gauntlet、Bevy CI tool、Godot Doctest、Fyrox CI、Unity Graphics Wrench | E3责任对照，不把参考仓库不存在的能力写成事实 |

本轮选定42条Git index record，共23,176行、975,128 bytes，Git-index fingerprint为`aba86c646d27add5bd1bc0e3ed41c05f75b27c87c993d91207a3a6090cb1cd7d`。选定集合覆盖workflow、Cargo/Hub入口、代表性Rust/工具/前端测试与本地验证器；总体数量由排除`dev`、`docs/tests`、计划与generated/vendor目录后的仓库扫描得到，不表示每个测试文件均已逐项动态执行。

### 2.2 定量与动态证据

| 检查 | 结果 | 可支持结论 |
|---|---|---|
| 测试源码inventory | 5,582文件；959,107行；35,916,766 bytes | 测试资产巨大，需要catalog、partition和owner，不适合一个黑盒命令 |
| Rust test attribute | 21,211个`#[test]`、190个ignored、8个`should_panic` | test count高，但manual/ignored与负向测试治理不足 |
| Cargo manifest/metadata | 161 manifest；0 `autotests=false`；1个`test=false`；2个显式`[[test]]`；0 `[[bench]]`/`harness=false` | 大多数依赖Cargo隐式发现；Hub lib是明确例外；无独立benchmark harness |
| Hub test reachability | 98个source、61个test module、258个inline test；lib target `test=false` | 258项不进入默认Cargo lib test harness |
| Hub external tests | 39文件、270 test、81次source read、189次`.contains()` | 外部合同测试偏源码形状，不能替代inline behavior |
| Tool Python corpus | 659文件、4,295 test methods；CI显式3模块 | CI只命名极小子集，不能代表工具测试资产 |
| CI Python聚焦执行 | 35 tests passed / 11.195秒 | 三个现有CI模块内部可运行；不证明其余656文件 |
| PowerShell inventory | 36个`*.Tests.ps1`；23个Pester、13个standalone | 两种执行语义并存，workflow没有统一发现/运行 |
| Rust test targets | 36 packages、184 targets、130 integration test targets | crate内mutex无法跨多个test process提供资源互斥 |
| 高规模测试文件 | 45个测试源码文件达到或超过1,000行 | 测试本身出现模块化、fixture与review成本问题 |
| source-shape assertions | 892次`read_to_string`、3,663次Rust `source.contains`、3,774次Python `assertIn` | 结构守卫占比显著，存在false-green与重构脆弱性 |
| property/fuzz framework | 0 fuzz target/file；0 proptest/quickcheck/arbitrary/libFuzzer/AFL/Bolero dependency | 解析、序列化、FFI、网络与资产格式没有系统生成式测试 |

此前Tooling 03和06的全量动态失败/超时由对应报告拥有，本轮不重复运行长达15分钟或已知667项失败的集合。Hub Rust全量也未重复运行：Hub 01已经以managed Windows Cargo复现当前tracked source的编译P0，而本轮通过manifest与Cargo metadata即可独立证明inline test不可达。Tooling 07拥有benchmark/profile/crash evidence的性能结论，本报告只拥有测试分类、调度和结果协议。

### 2.3 正向基线

- Rust测试覆盖runtime、editor、plugins、app、Hub、examples与runtime interface多个域，21,211项不是空壳；后续应保留行为覆盖，避免为缩短时间直接删除。
- Editor的`TestEnvironmentLock`可从mutex poison恢复，部分app/editor测试会保存并恢复原环境变量；这是统一resource lease/RAII fixture的种子。
- `validate-matrix.ps1`能通过Coordinator获取Windows managed Cargo target，对build/test filter、filtered ignored tests、export/profile合同与Cargo环境lease进行编排；可作为统一runner的Cargo backend。
- MVP Windows workflow对关键F1-F5场景使用exact filter、`--test-threads=1`、明确日志、source fingerprint和always-upload evidence；这些局部做法应提升为通用结果协议。
- Session Coordinator Web有`typecheck -> custom tests -> build -> verify-dist`的自包含`check`命令；问题是CI未调用且结果未汇总，而不是完全没有测试入口。
- 多数visual artifact测试用带原因的`#[ignore]`隔离人工写图动作，避免默认测试任意改写tracked evidence；应继续演化为受调度capture lane。
- source-shape合同对模块吸收、owner boundary、generated currentness和预算守卫有价值；应保留为Architecture lane，但不得冒充runtime behavior。

### 2.4 参考边界

- Unreal `EAutomationTestFlags`把Editor/Client/Server/Commandlet/Program context、Smoke/Engine/Product/Perf/Stress/Negative filter和priority编码为测试元数据；Gauntlet再为设备型测试提供Start/Tick/Stop/Restart/Cleanup、max duration、typed result/status/stop reason、artifact与telemetry。Zircon应学习分类与orchestration/result分层，不照搬宏或设备协议。
- Bevy把Rust CI入口集中在`tools/ci`，明确拆分format、clippy、unit/test compile、integration、doc、compile-fail、examples与bench compile，并统一jobs、test threads和no-fail-fast参数。它证明CI workflow可以路由到单一计划owner，而非复制命令。
- Godot以Doctest context/CLI、reporter、listener和pending/may-fail macro提供测试发现与结果语义；这可对照Zircon缺失的expected-failure/quarantine metadata，不代表Godot解决了所有跨进程fixture问题。
- Fyrox当前CI以跨OS workspace all-features test、fmt与all-target clippy提供较简洁基线；它适合作为最低矩阵参考，不足以支撑Zircon的多产品、多工具和GPU设备调度。
- Unity Graphics Wrench先pack package，再跨Editor版本、OS运行Editor/Playmode tests，设置timeout/retry、clean-library rerun，并上传XML、logs、crash dumps和results。它是package validation与artifact参考，不是Unity整引擎测试控制面源码。

## 3. 当前P0

### TOOL-TEST-P0-001 · Hub显式关闭lib harness，使258项inline单元测试不可达

`zircon_hub/Cargo.toml`的`[lib] test = false`使Cargo metadata将Hub lib target标为不可测试。与此同时，Hub source中保留61个`#[cfg(test)]`模块和258项`#[test]`。默认root `cargo test --workspace`只会构建Hub integration targets所依赖的普通lib，不会启用这些inline模块。外部source-shape合同不能补偿该缺口。

必须先确认关闭lib harness的原始约束，随后恢复`test = true`或把确需运行的测试迁入显式、可发现的integration target；CI增加metadata guard，拒绝任何含inline tests但target `test=false`的crate。恢复后必须记录实际discovered/executed count，不能只以compile成功验收。

### TOOL-TEST-P0-002 · Required CI计划遗漏绝大多数非Rust测试资产

主CI显式执行3个Python模块，而Tool Python corpus有659个测试文件；36个PowerShell test文件没有统一workflow consumer；Session Coordinator Web的`npm run check`未进入CI；Hub前端没有test script；WOC完整check链和独立工具suite也不属于一个required plan。Rust workspace test不会发现这些语言和runner。

必须建立versioned `TestPlan`，逐lane声明owner、runner、selection、platform、timeout、resource、required/optional与artifact。PR和main至少执行受影响required lanes；nightly/qualification执行全量。任何已登记required suite被跳过、未发现、零测试或runner缺失都必须fatal，并产生typed skip reason，而非静默绿色。

### TOOL-TEST-P0-003 · 没有source/build-bound统一结果与完整性证明，无法形成发布准入

当前Cargo文本、Python unittest输出、Pester/custom PowerShell、Node runner和MVP JSON各自表达结果；没有统一plan digest、source tree、Build Set、runner/toolchain、attempt、selected/discovered/executed/skipped、artifact digest与expiry。由此无法证明一次“通过”覆盖了全部required lanes，也无法阻止旧结果、局部结果或不同build结果被复用为release evidence。

必须定义不可变`TestPlanManifest`、`TestAttemptReceipt`、`TestCaseResult`和`TestArtifactManifest`。aggregator只接受绑定同一source/build/toolchain/target的签名或受信worker receipt，验证required lane闭包、零测试、timeout/cancel、quarantine budget和artifact currentness后生成`ValidationSet`。Tooling 09的Release Candidate只能消费该ValidationSet digest；缺失或过期必须阻断promotion。

## 4. Test Control Plane 与 Taxonomy 差距

### TOOL-TEST-P1-001 · 没有单一Test Domain owner

workflow、Cargo、`validate-matrix.ps1`、Python discovery、PowerShell、npm和MVP脚本各自决定测试。建立独立Test Service/CLI，唯一拥有plan解析、选择、执行、结果聚合和退出语义；现有runner作为backend接入。

### TOOL-TEST-P1-002 · 测试没有稳定TestId

当前身份主要是语言runner输出的文件/函数名，重命名即丢历史。定义`domain/suite/case/parameter`稳定ID及source location，保留rename alias，供历史、quarantine、ownership和impact mapping使用。

### TOOL-TEST-P1-003 · 没有统一测试类别

unit、integration、contract、architecture/source-shape、compile-fail、visual、GPU、performance、stress、negative、manual被文件名或ignore reason隐式表达。建立受控taxonomy及互斥/可组合规则，禁止任意字符串分类。

### TOOL-TEST-P1-004 · 没有application context声明

Editor、Client、Server、Commandlet/tool、Hub、headless和device context没有一等元数据。测试必须声明可运行context和禁止context，scheduler据此准备host与依赖。

### TOOL-TEST-P1-005 · 没有required capability模型

GPU backend/feature、display、audio device、network、filesystem、admin、RenderDoc、平台SDK等依赖靠失败或ignore发现。定义capability query与worker advertisement，缺能力产生typed unscheduled而非随机失败。

### TOOL-TEST-P1-006 · 没有test owner与升级路径

失败结果不能稳定映射crate/domain owner，也没有SLA或escalation。Test catalog绑定owner、reviewers、triage channel与blocking policy，owner变化通过代码审查更新。

### TOOL-TEST-P1-007 · 没有测试依赖图

工具suite、生成物、构建、cook、package、device和runtime测试有顺序依赖，却由workflow步骤隐式表达。TestPlan引用Build Set action/artifact digest，显式形成DAG并禁止测试内部偷偷重建不同输入。

### TOOL-TEST-P1-008 · 没有change-to-test impact mapping

PR只能全跑少数命令或人工filter，无法以模块、crate、asset schema、generated consumer和platform依赖选择。建立静态owner graph加历史覆盖映射；选择结果必须可解释并保留全量nightly兜底。

### TOOL-TEST-P1-009 · 没有lane budget与分片策略

21,211个Rust test、4,295个tool Python method和超大suite没有可审计的PR/nightly/qualification时长预算。按历史时长与resource class稳定分片，设置per-case/per-suite/per-lane timeout和总预算。

### TOOL-TEST-P1-010 · 本地与CI使用不同计划器

本地`validate-matrix.ps1`管理Cargo target/lease，CI直接重复Cargo命令且不调用它。让本地与CI解析同一TestPlan；环境适配只在executor层，选择与required集合不得分叉。

### TOOL-TEST-P1-011 · `ignored`承担过多不同语义

190个ignored tests混合性能、scale、人工截图、真实GPU/RenderDoc、已迁移或退休代码，部分只有裸`#[ignore]`。用typed metadata区分manual、scheduled、capability-gated、quarantined、retired；每项要求owner、reason、lane和expiry。

### TOOL-TEST-P1-012 · 零测试与发现漂移没有统一fatal policy

filter拼写错误、target关闭或测试迁移后，runner可能成功但执行0项。所有required suite记录expected lower bound或catalog digest；discovered/executed不符时单独失败，MVP exact-count做法推广到所有runner。

## 5. Rust/Cargo Test Architecture 差距

### TOOL-TEST-P1-013 · 21,211项Rust测试被压进过粗的workspace命令

单一`cargo test --workspace`不能表达domain、resource、priority、timeout或artifact策略。按crate/domain生成显式suite catalog，仍复用Cargo harness，但由TestPlan分组和分片。

### TOOL-TEST-P1-014 · Root与plugin双workspace结果没有统一闭包

CI分别运行root和plugin workspace，失败与artifact不汇总为一个ValidationSet。建立workspace-independent suite identity，并由同一plan明确两者均为required及其dependency lock/source identity。

### TOOL-TEST-P1-015 · 130个integration test target缺乏进程拓扑治理

Cargo会创建大量独立test process；crate内static mutex无法跨target协调。catalog必须声明process/resource scope，executor提供跨进程lease、并发上限与冲突诊断。

### TOOL-TEST-P1-016 · inline与integration边界没有可执行守卫

除Hub外，未来任何`test=false`、feature/cfg或target设置都可能让inline tests静默消失。metadata lint比较源码test inventory与Cargo target可达性，要求每个test owner映射到至少一个suite。

### TOOL-TEST-P1-017 · feature组合测试只做零散profile check

profile workflow主要`cargo check`七个feature组合，不执行各组合行为测试。为关键产品feature set定义build+test matrix，禁止default-feature绿色替代server/editor/client具体组合。

### TOOL-TEST-P1-018 · doctest结果没有独立可见性

Cargo默认可能在workspace test尾部运行eligible doctests，但当前结果不分lane、无count/currentness和超时。显式建立doc-test lane并统计被禁用/无docs的crate，避免隐藏在长Cargo输出中。

### TOOL-TEST-P1-019 · 没有compile-fail/UI test harness

宏、ABI、feature互斥和诊断合同没有`trybuild`或等价compile-fail snapshot。建立受版本控制的diagnostic cases，规范化路径/版本噪声，并在toolchain升级时显式review差异。

### TOOL-TEST-P1-020 · negative/panic coverage过薄且不可分类

全仓只有8个`should_panic`，并不意味着其他Result型负向测试不存在，但没有negative taxonomy可证明错误面覆盖。catalog记录error class、fault injection和expected failure mode，避免只测happy path。

### TOOL-TEST-P1-021 · Rust test文件自身出现巨型模块

45个测试源码超过1,000行，若干达到2,000至4,000行，fixture、scenario和断言混合。按domain拆scenario module、共享fixture builder和assertion library，保持case identity与review边界。

### TOOL-TEST-P1-022 · 没有统一Rust runner配置

仓库没有`nextest.toml`或等价配置来表达slow test、retry、threads、groups与archive；部分说明要求手工`--test-threads=1`。由统一executor生成稳定Cargo参数或引入可审计runner配置，不能依赖开发者记忆。

## 6. Python、PowerShell、Web 与 Contract Test 差距

### TOOL-TEST-P1-023 · Python suite只有文件约定，没有中央manifest

661个Python测试文件、4,313个test methods主要使用`unittest`，tools子集即659/4,295。建立suite manifest和分层discovery，明确package root、pattern、fixtures、timeout与expected count。

### TOOL-TEST-P1-024 · Python全量执行不具备可接受反馈时间

Coordinator 1,175项full discovery约904秒仍超时，说明当前进程/fixture拓扑不能作为PR单lane。按数据库、process、Git、HTTP、workspace copy等资源拆分并行安全分片，保存慢用例历史。

### TOOL-TEST-P1-025 · Export suite的大规模失败没有隔离为known baseline

Tooling 03复现1,642项中667项失败；当前无法区分新回归、协议迁移和陈旧shape test。建立失败inventory、owner和hard-cutover迁移批次；禁止用跳过整个suite恢复绿色。

### TOOL-TEST-P1-026 · PowerShell存在Pester与standalone双协议

36个`*.Tests.ps1`中23个使用Pester，13个自带assert/exit语义。定义一个adapter contract，将两者转换为同一case/result schema；长期把共享fixture、mock和cleanup收敛到Pester或等价统一runner。

### TOOL-TEST-P1-027 · PowerShell测试没有CI发现清单

workflow没有统一列举或运行`*.Tests.ps1`。CI先加入discovery-only守卫和Windows分片，再按资源分级执行；任何新增文件必须自动进入catalog或显式标记manual。

### TOOL-TEST-P1-028 · Session Coordinator Web的check不在CI

package已提供typecheck/test/build/verify-dist，但workflow没有consumer。将其作为required tooling-web lane，绑定Node/npm lock与dist currentness artifact；失败由Coordinator owner处理。

### TOOL-TEST-P1-029 · Hub前端没有测试入口

Hub package只有dev/build/typecheck/tauri命令，没有test script、DOM/component runner或browser E2E。建立view-model/component行为测试和少量Tauri boundary E2E，不能继续由Rust源码字符串检查替代。

### TOOL-TEST-P1-030 · Workbench preview的Playwright仅是工具依赖

preview package声明Playwright但没有test script，现有scripts围绕design export/verify。将视觉生成工具与browser regression runner分开，显式登记viewport、browser、baseline和pixel policy；不能从依赖存在推断E2E覆盖。

### TOOL-TEST-P1-031 · Source-shape合同权重过高

大量`read_to_string`、`source.contains`、`assertIn/assertNotIn`验证名字和文本存在。Architecture lane应改用Cargo metadata、Rust AST、typed schema或实际consumer行为；保留必要owner boundary，删除与行为测试重复的脆弱字符串断言。

### TOOL-TEST-P1-032 · Source-shape测试可对dead code产生false green

Hub正好证明“字符串存在”与“harness可执行”并不等价。每个关键source contract必须链接至少一个compile/runtime/consumer test，catalog展示shape-only覆盖，禁止其单独满足feature acceptance。

## 7. Isolation、Fixture 与 Flake 治理差距

### TOOL-TEST-P1-033 · 环境变量隔离没有跨进程owner

测试广泛修改`ZIRCON_CONFIG_PATH`、`SLINT_BACKEND`、`ZIRCON_RUNTIME_LIBRARY`和`ZIRCON_EDITOR_CONFIG_PATH`。统一fixture保存/恢复值，并通过executor resource key串行化跨binary冲突；直接`set_var/remove_var`由lint限制。

### TOOL-TEST-P1-034 · 用户目录与配置根没有强制sandbox

测试不能依赖真实HOME/AppData、默认项目或用户配置。每个attempt分配唯一user/config/cache/data root，production locator通过正式override注入；teardown输出残留清单。

### TOOL-TEST-P1-035 · 临时目录命名与清理策略分散

Python大量使用`TemporaryDirectory`是正向基础，但Rust/PowerShell仍有process-id、固定目录和手工`remove_dir_all/Remove-Item`。提供统一attempt-scoped temp allocator、ownership marker、no-follow cleanup与失败保留策略。

### TOOL-TEST-P1-036 · 端口没有broker

HTTP/control/device tests各自绑定端口或先探测再使用，存在TOCTOU和并行碰撞。executor提供保留socket/port lease并把endpoint注入fixture，禁止固定开发端口进入并行测试。

### TOOL-TEST-P1-037 · 子进程生命周期不统一

Python与PowerShell大量启动Cargo、Hub、Coordinator或产品进程，超时/取消可能只结束root process。复用Tooling 06要求的Process Supervisor/Job Object/process group，attempt结束必须证明process tree归零。

### TOOL-TEST-P1-038 · wall-clock sleep替代事件与虚拟时钟

Rust、Python和PowerShell存在毫秒到秒级sleep/poll，既拖慢suite又造成负载相关flake。引入event barrier、bounded condition wait和injectable clock；保留真实时间测试时声明timing resource和宽容区间。

### TOOL-TEST-P1-039 · 随机性与顺序没有统一seed

需要生成输入或调度扰动的测试没有attempt-level seed协议。所有随机/顺序测试记录seed与case parameters，失败命令可精确重放；默认执行包含固定seed和周期性随机seed lane。

### TOOL-TEST-P1-040 · GPU/display测试没有device scheduler

真实WGPU、visual、RenderDoc和adapter测试靠ignore、软件backend或单个Windows MVP worker区分。建立GPU capability inventory、adapter/driver identity、exclusive/shared lease、device-lost policy和headless/display topology。

### TOOL-TEST-P1-041 · Flaky test没有一等状态与预算

仓库没有flake history、confirmation rerun、quarantine owner/expiry或maximum budget。aggregator基于attempt history识别不稳定，允许有限诊断rerun但首败仍保留；quarantine不得满足required gate且必须到期。

### TOOL-TEST-P1-042 · Fixture版本与currentness没有身份

project、asset、shader、capture、database和generated fixture经常由源码或脚本隐式生成。fixture manifest绑定schema/generator/tool/input digest；consumer验证currentness，禁止旧fixture让新代码false green或任意重写tracked证据。

## 8. Result、Coverage 与 Test Quality 差距

### TOOL-TEST-P1-043 · 没有跨runner标准结果格式

Cargo文本、unittest、Pester/custom script和Node输出无法统一查询。adapter产出稳定case ID、status、duration、stdout/stderr references、failure category、attempt和artifact列表，同时保留原生日志。

### TOOL-TEST-P1-044 · CI没有通用JUnit/结构化结果artifact

主CI不上传测试result；MVP只上传其专有evidence root。每个lane即使setup失败也通过`always()`发布manifest、logs和部分results，并由aggregator区分infra/test/product failure。

### TOOL-TEST-P1-045 · 没有coverage基线

仓库没有llvm-cov/tarpaulin、Cobertura/LCOV、Codecov或等价链。先按crate/domain建立line/function/branch基线和可排除generated规则，再用趋势和关键owner阈值治理；不以总百分比替代质量。

### TOOL-TEST-P1-046 · 没有测试到需求/风险的traceability

测试名与MVP F1-F5、plan finding、ABI/schema或安全边界没有统一关系。Test catalog允许关联requirement/finding/risk ID，acceptance gate验证每个blocking requirement至少有可执行证据。

### TOOL-TEST-P1-047 · 没有property-based testing

序列化、parser、path canonicalization、manifest、网络协议和resource descriptor适合生成式性质验证，但无proptest/quickcheck等。按稳定invariant逐域引入，失败自动最小化并保存seed/corpus。

### TOOL-TEST-P1-048 · 没有fuzzing与corpus生命周期

全仓无fuzz target和libFuzzer/AFL等入口。优先覆盖不可信资产、网络、FFI、shader/source parser与压缩/图像边界，维护seed corpus、crash dedup、minimization、sanitizer和修复回归。

### TOOL-TEST-P1-049 · 没有并发模型检查

大量cache、job、lease、queue和runtime state依赖并发，却无Loom或等价systematic concurrency lane。对小型核心状态机建立可穷举模型，真实线程stress只作补充。

### TOOL-TEST-P1-050 · 没有sanitizer/Miri专项lane

native ABI、unsafe、FFI和跨线程代码没有ASan/TSan/UBSan/Miri的统一计划。按平台可用性建立nightly/qualification lanes，记录unsupported原因、toolchain与suppression owner。

### TOOL-TEST-P1-051 · 没有mutation testing或断言有效性审计

大量source-shape和happy-path测试可能在实现被破坏时仍通过。对关键parser、policy、transaction和security validator周期性运行mutation/sample fault injection，衡量测试是否真正拒绝错误行为。

### TOOL-TEST-P1-052 · 失败历史、趋势与回归窗口不可查询

没有按TestId聚合pass/fail/flake/duration、首次失败commit、最后绿色Build Set与关联artifact。建立append-only result store和查询UI，为bisect、owner告警、时长预算与release waiver提供事实。

## 9. P2：开发者体验与持续治理

### TOOL-TEST-P2-001 · 缺少统一`zircon test`入口

提供`list/plan/run/replay/explain/collect`命令，默认输出短摘要并保留JSON；内部路由Cargo、Python、PowerShell、Node和device executor。

### TOOL-TEST-P2-002 · 缺少“为什么选中/跳过”解释

每个case显示由change impact、required lane、capability、quarantine或manual policy导致的选择原因，避免开发者猜测CI。

### TOOL-TEST-P2-003 · 缺少失败重放包

生成最小repro manifest，包含source/build/fixture/seed/env allowlist、runner参数和artifact引用；敏感值只保存引用或redacted hash。

### TOOL-TEST-P2-004 · 缺少本地快速层级

定义pre-commit、PR-fast、PR-full、nightly、qualification层级和时长目标，保证快速层是完整计划的可解释子集。

### TOOL-TEST-P2-005 · 缺少测试目录可视化

生成按domain/category/platform/resource/owner的catalog页面，展示unreachable、ignored、quarantined、slow和无行为配对的shape tests。

### TOOL-TEST-P2-006 · 缺少统一日志关联ID

test plan、attempt、case、process、device和artifact共享correlation ID，跨Cargo/Python/PowerShell/前端日志可跳转查询。

### TOOL-TEST-P2-007 · 缺少fixture builder文档与模板

为project/asset/UI/GPU/network/process fixture提供最小模板、ownership与cleanup规则，减少复制超大fixture代码。

### TOOL-TEST-P2-008 · 缺少慢测试预算反馈

PR显示新增/变慢case、lane临界路径和历史分位数；预算超标需owner解释或拆分，不把固定timeout当性能指标。

### TOOL-TEST-P2-009 · 缺少quarantine与waiver看板

集中显示owner、首败、复现率、expiry、blocking capability和修复链接；过期自动恢复阻断而不是永久忽略。

### TOOL-TEST-P2-010 · 缺少测试代码质量守卫

为超大test file、直接环境修改、固定端口、裸sleep、无reason ignore和未登记runner建立lint及逐步收紧预算。

## 10. 目标架构

```text
Change / Qualification Request
              |
              v
       Test Plan Resolver
       - source/build identity
       - required lane closure
       - change impact + policy
       - platform/capability/resource
              |
              v
        Test Scheduler
       - stable shards
       - sandbox + fixture lease
       - port/GPU/process ownership
       - timeout/cancel/retry policy
              |
       +------+------+------+------+
       | Cargo|Python|Pwsh |Node  | Device/GPU executors
       +------+------+------+------+
              |
              v
       Attempt/Case Receipts
       - discovered/executed/skipped
       - logs/crash/capture/coverage
       - fixture/seed/toolchain digest
              |
              v
       Validation Aggregator
       - required closure/currentness
       - flake/quarantine budget
       - result/artifact integrity
              |
              v
       ValidationSet digest
       -> PR gate / nightly / Release Candidate
```

建议的核心对象：

- `TestDescriptor`：稳定TestId、source location、owner、category、context、capabilities、resources、timeout和requirement links。
- `TestSuiteManifest`：runner、discovery、expected count policy、fixture和shard strategy。
- `TestPlanManifest`：source tree、Build Set、target/profile、required/optional suites、selection explanation和policy digest。
- `TestAttemptReceipt`：worker/toolchain/environment allowlist、start/end/exit/cancel、selected/discovered/executed与process cleanup proof。
- `TestCaseResult`：typed status/failure、duration、seed/parameters、stdout/stderr和artifact references。
- `TestArtifactManifest`：logs、JUnit、coverage、screenshots、captures、crash dumps的digest、mime、retention与privacy class。
- `ValidationSet`：同一source/build下required closure、waiver/quarantine、currentness与最终admission verdict。

## 11. 重构里程碑

### M0 · Stop-the-line 与Inventory

- 恢复或迁移Hub 258个inline tests，增加`inline test + test=false` metadata fatal guard。
- 冻结“全仓测试通过”表述，列出现有root/plugin/Hub/export/Coordinator阻断与owner。
- 生成全语言Test catalog，登记190个ignored、36个PowerShell tests和所有custom runner。
- 给现有CI每个lane增加selected/discovered/executed/zero-test记录。

### M1 · Schema 与统一Plan

- 定义TestDescriptor、Suite、Plan、Attempt、CaseResult、Artifact和ValidationSet schema。
- 将root/plugin Cargo、CI三项Python、PowerShell和两个Web package登记为显式suite。
- 本地与CI共同解析一个versioned plan；`validate-matrix.ps1`成为Cargo adapter。
- 引入TestId、owner、category、context、capability与resource metadata。

### M2 · Isolation 与可靠执行

- 建立attempt sandbox、config/user/cache roots、fixture manifest和安全cleanup。
- 接入跨进程environment/path/port/GPU/device lease。
- 所有子进程统一进入Process Supervisor，timeout/cancel后证明process tree归零。
- 用event/virtual clock替换高风险sleep，记录seed并支持单case replay。

### M3 · 多语言Required Lanes

- 分片执行Tool Python、PowerShell、Coordinator Web、Hub frontend和WOC check。
- 将source-shape合同独立为Architecture lane，并绑定行为/consumer证据。
- feature/context矩阵覆盖Editor/Client/Server/Tool/Hub与关键平台。
- ignored tests迁移为scheduled/manual/capability/quarantine/retired typed状态。

### M4 · Result Store、Coverage 与 Flake

- 所有runner发布统一case result、JUnit/native log与artifact manifest。
- 建立source/build-bound ValidationSet和release admission consumer。
- 引入coverage基线、历史时长、flake detection、有限confirmation rerun和expiry quarantine。
- PR提供impact selection解释，nightly执行完整闭包验证选择器无漏测。

### M5 · Advanced Quality Lanes

- 为macro/ABI/feature diagnostics建立compile-fail tests。
- 为parser/serialization/path/network/asset schema引入property与fuzz corpus。
- 为unsafe/FFI/concurrency建立Miri、sanitizer和Loom/模型检查lane。
- 对关键transaction/policy/security组件执行mutation或fault-injection审计。

### M6 · Qualification 与持续收敛

- GPU/device/visual tests按capability跨adapter/driver/platform调度。
- qualification ValidationSet绑定Tooling 09 Release Candidate，过期或缺lane阻断promotion。
- 逐步拆分45个超大测试文件并降低shape-only acceptance比例。
- 以历史数据持续治理时长、flake、quarantine、coverage与fixture currentness预算。

## 12. 验收门

1. Cargo metadata中不存在“源码含inline test但所有对应target `test=false`”的crate。
2. Hub 258项原inline tests均映射到可发现suite，CI记录实际执行数量与结果。
3. 全仓每个测试源码文件都映射到Test catalog、owner和runner，或有显式retired/manual记录。
4. Required TestPlan包含root Cargo、plugin Cargo、Tool Python、PowerShell、Coordinator Web、Hub frontend与WOC lanes。
5. required suite runner缺失、发现0项、count异常、skip或timeout均不能生成绿色ValidationSet。
6. 本地与CI对同一source/target解析出相同required suite digest。
7. 每个result绑定source tree、Build Set、toolchain、target/profile、plan digest和attempt identity。
8. Release Candidate只接受未过期且required closure完整的ValidationSet digest。
9. Cargo、Python、PowerShell和Node结果可查询同一稳定TestId与原生日志。
10. setup失败、timeout和cancel也会发布部分result、process cleanup proof与诊断artifact。
11. 190个ignored tests全部拥有typed状态、owner、lane、reason和expiry；不再有裸ignore。
12. 环境变量、用户目录、临时目录和固定输出路径均由attempt sandbox或resource lease管理。
13. 并行integration targets无法竞争同一config root、port、artifact path或GPU exclusive device。
14. attempt完成后没有遗留子进程、端口、mount、锁或无owner临时目录。
15. wall-clock sleep受lint与allowlist治理，关键异步测试使用事件或虚拟时钟。
16. flaky confirmation rerun保留首败，quarantine不满足required gate且到期自动阻断。
17. source-shape test在catalog中独立标记，关键feature至少有一个compile/runtime/consumer行为测试。
18. compile-fail lane覆盖public macro、ABI和关键feature互斥诊断。
19. property/fuzz lane覆盖至少序列化、资产manifest、路径和网络不可信输入，并保存可重放corpus。
20. sanitizer/Miri/concurrency lanes拥有明确平台支持矩阵、suppression owner和结果artifact。
21. coverage按domain发布且generated/vendor排除规则受版本控制；不得只报告全仓总百分比。
22. 测试历史可查询duration、flake、首败commit、最后绿色Build Set和相关artifact。
23. PR impact selection能解释每个selected/skipped suite，并由nightly全量反证漏测率。
24. 任何“全仓测试通过”状态可从ValidationSet反查全部required case、attempt和artifact，而非依赖控制台文本。

## 13. 跨报告所有权

| 问题 | 主owner | 本报告边界 |
|---|---|---|
| workspace/toolchain/dependency/CI基础设施 | Tooling 01 | 只拥有test plan、selection和result语义，不重复供应链结论 |
| plugin workspace编译/catalog/native probe | Tooling 01/02与Plugins 01 | 作为required lane当前阻断输入，不重写插件修复方案 |
| export 667项失败与协议实现 | Tooling 03 | 本报告拥有失败分类、suite分片和结果接入 |
| WOC generated/currentness | Tooling 05 | 本报告拥有WOC check作为required lane及其result adapter |
| Coordinator安全、进程、validation伪通过 | Tooling 06 | 复用Process Supervisor；本报告拥有测试attempt模型 |
| benchmark/profile/capture/crash/symbol | Tooling 07 | 本报告只拥有performance/GPU/manual test调度与分类 |
| release candidate/promotion/install | Tooling 09 | Release消费ValidationSet；Test不拥有发行实现 |
| Hub当前编译阻断与产品行为 | Hub 01 | 本报告独立拥有lib harness关闭和test reachability |

## 14. 最终判断

ZirconEngine不缺“测试文件”，缺的是把这些测试变成可信工程系统的控制面。当前最危险的不是某个assert写得不够多，而是258个Hub单元测试能在manifest中被整体关闭，数百个工具suite不在CI，且各种runner的局部绿色无法聚合为source/build-bound完整证明。继续增加零散source-string合同只会扩大维护面积，不会提高发布可信度。

正确顺序是先恢复可达性并阻断零测试，再建立全语言TestPlan与统一result schema；随后处理跨进程隔离、fixture/currentness、Process Supervisor和GPU capability调度；最后引入coverage、property/fuzz、sanitizer、并发模型检查与release qualification。只有当每个required case都能回答“为什么执行、在哪个环境执行、对哪个build执行、产生了什么可验artifact、结果是否仍新鲜”，测试数量才真正转化为工程质量。
