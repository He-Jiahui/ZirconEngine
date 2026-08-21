---
related_code:
  - Cargo.toml
  - .github/workflows/profile-feature-contract.yml
  - tools/check-runtime-profile-features.ps1
  - tools/runtime-profile-feature-presets.py
  - tools/ui-profile-capture.ps1
  - tools/profile-capture-manifest.ps1
  - tools/profile-capture-paths.ps1
  - tools/ui-profile-latency-evidence.ps1
  - tools/ui-profile-native-resize.ps1
  - tools/ui-profile-process-evidence.ps1
  - tools/ui-profile-scale-fixture.ps1
  - tools/zircon_profile_shader_pbr_viewer.ps1
  - tools/zircon_summarize_shader_pbr_profile.py
  - tools/write_zircon_shader_pbr_build_provenance.ps1
  - tools/shader-pbr-profile-contract.ps1
  - tools/mvp/Capture-RenderExtractBaseline.ps1
  - tools/mvp/Build-RenderExtractProfilingInputs.ps1
  - tools/mvp/Write-RenderExtractBaselineReport.ps1
  - tools/mvp/RenderExtractBaselineEvidence.psm1
  - tools/mvp/RenderExtractBaselineMetrics.psm1
  - tools/mvp/RenderExtractFrozenInput.psm1
  - tools/mvp/RenderExtractProcessJob.psm1
  - zircon_runtime/src/plugin/native_plugin_loader/benchmark_harness.rs
  - zircon_runtime/src/diagnostic_log/sink.rs
  - zircon_app/src/bin/editor.rs
  - zircon_app/src/bin/runtime_preview.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
tests:
  - tools/tests/profile_feature_contract.Tests.ps1
  - tools/tests/ui-profile-capture-output-contract.Tests.ps1
  - tools/tests/ui-profile-latency-evidence.Tests.ps1
  - tools/tests/ui-profile-native-resize.Tests.ps1
  - tools/tests/ui-profile-process-evidence.Tests.ps1
  - tools/tests/zircon_profile_shader_pbr_viewer.Tests.ps1
  - tools/tests/test_zircon_summarize_shader_pbr_profile.py
  - tools/tests/render-extract-baseline-capture.Tests.ps1
  - tools/tests/render-extract-baseline-report.Tests.ps1
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
  - docs/plans/optimize/zircon_app/02-pbr-viewer-tool-runtime-evidence-renderdoc-review.md
  - docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
reference_engines:
  - dev/bevy/benches/Cargo.toml
  - dev/bevy/benches/README.md
  - dev/bevy/benches/benches/bevy_ecs/main.rs
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Windows/WindowsPlatformCrashContext.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/GenericPlatform/GenericPlatformCrashContext.cpp
  - dev/UnrealEngine/Engine/Source/Developer/CrashDebugHelper/Private/CrashDebugHelper.cpp
  - dev/UnrealEngine/Engine/Source/Programs/CrashReportClient/Private/CrashReportClient.cpp
  - dev/godot/platform/windows/crash_handler_windows_seh.cpp
  - dev/godot/platform/windows/crash_handler_windows_signal.cpp
  - dev/Graphics/Tests/SRPTests/Packages/com.unity.testing.visualeffectgraph/PerformanceTests/Runtime/VFXPerformanceRuntimeTests.cs
  - dev/Graphics/Tests/SRPTests/Packages/com.unity.testing.visualeffectgraph/PerformanceTests/Editor/VFXPerformanceEditorTests.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 07 · 性能基准、Profile/Capture、符号、崩溃与长期证据基线工程化差距

## 1. 结论

ZirconEngine 已积累大量性能计划、手工 profile、截图、RenderDoc capture 和若干高质量的专用采集脚本，但这些资产还没有收敛成一套能够证明“性能优于当前 Unreal”的工程系统。仓库外 `dev` 参考代码除外，本轮扫描了 161 个 Cargo manifest，未发现任何 `[[bench]]`、`autobenches`、Criterion、Divan 或 Bencher 配置；相反，139 个 Rust 文件包含 190 个 `#[ignore]`，其中 118 处与 performance、benchmark、manual、visual、GPU 或 profile 有关。现有性能证据主要仍是被忽略测试、手工工具输出和日期命名文档，缺少统一场景身份、统计模型、硬件/驱动清单、可比较基线、回归预算、趋势存储与合并门。

`docs/plans/performance` 的规模本身已经成为治理信号：562 个文件、40,057 行、4,096,858 bytes，其中 541 个以日期命名；主审计文档单文件 1,594 行、783,692 bytes，包含连续 634 个 `PERF-MVP` finding，但 25 个 checklist 只有 4 个完成。`review.md` 的 accepted table 仍为空，`pending.md` 与 `review.md` 的 Rust 文件总数分别停在 17,106 与 17,013，而当前 tracked、可见 untracked 合并口径为 17,261。这里不是“报告不够多”，而是报告尚未投影为机器可判定的性能资产与验收状态。

采集链存在两个立即阻断可信度的 P0。第一，`ui-profile-capture.ps1` 把未校验的 `Scenario`/`ScenarioList` 原样拼入 `SessionId`，再用于 profile、project、日志和 tracked screenshot 目录；例如 `alpha\..\..\escape` 可把目标解析到 `E:\escape-measured-01`，越出默认 `E:\zircon-profiles`。第二，PBR build provenance writer 与 capture script 明确使用 schema 2 / `zircon_managed_viewer_artifact_provenance`，最终 Python summarizer 却强制 schema 1 / `zircon_local_viewer_capture_provenance`。因此当前合法 managed capture 必然在最终汇总阶段失败；21 项 Python unit tests 仍构造旧 schema，形成 false green。

崩溃与符号链相对 Unreal/Godot 的差距更大。当前产品级能力只有 Rust panic hook 在两个 binary 中先 bounded flush 已排队日志，再调用默认 hook；panic message/backtrace 本身不会进入 file-backed diagnostic log。仓库未找到 minidump、Crashpad/Breakpad、Windows SEH、Unix signal/core、hang watchdog、GPU crash、线程栈、Crash Reporter、symbol server 或自动 symbolication owner。build/export 会复制相邻 `.pdb`/`.dbg`/`.dSYM`，但没有 build/debug ID 索引、独立受控 symbol bundle、上传/保留策略或 crash-to-symbol binding。

也有应保留的工程化基础。native plugin benchmark harness 有 source manifest、release-profile拒绝debug assertion、并发barrier与结构化schema；RenderExtract baseline 有冻结输入、dirty-byte source fingerprint、exclusive lease、Windows Job Object、timeout、no-overwrite evidence 和证据哈希；PBR capture 会校验 viewer/HDRI fingerprint、冷暖运行与 Coordinator receipt。这些基础应收敛到统一 `PerformanceEvidenceService`、`CrashEnvelope` 与内容寻址 artifact store，而不是继续演化三套互不兼容脚本协议。

本轮记录 4 个 P0、56 个 P1 和 10 个 P2。未修改生产 Rust、PowerShell、Python、workflow、性能计划或 `docs/tests` 证据，只新增审查和索引。

## 2. 审查边界与证据

### 2.1 物理范围

| 子域 | 规模 | 本轮深度 |
|---|---:|---|
| Cargo benchmark surface | 161 manifest；0 conventional bench target/framework | E2 全量 manifest 搜索，E3 代表性 ignored performance harness |
| ignored Rust evidence | 139 文件 / 190 个 `#[ignore]`；118 处性能/手工/GPU相关 | E2 全量定位，E3 native plugin/event/log/ECS代表链 |
| UI profile toolchain | 7 个主/辅助 PowerShell，主脚本 2,644 行 | E3 参数、路径、source manifest、process、WPR/Tracy、evidence gate、输出逐链追踪 |
| Shader PBR profile | PowerShell capture 883 行；Python summarizer 935 行 | E3 provenance、cold/warm、WPR、GPU timing、RenderDoc replay、summary publication |
| RenderExtract baseline | 8 个 owner script/module；主capture 922行 | E3 frozen input、lease、process tree、raw evidence与report publication |
| diagnostic crash/log | 25 个 diagnostic log source/test文件及2个binary调用点 | E3 panic/flush/file identity；native crash/symbol能力全仓搜索 |
| performance plan | 562 文件 / 40,057 行 / 4,096,858 bytes | E2 文件/日期/ledger/checklist inventory；主审计与两个ledger E3 |
| tracked test evidence | 1,213 文件 / 862,902,570 bytes | E2 extension/size/命名/catalog/LFS/retention inventory；代表PBR/Editor evidence E3 |

本轮 combined tracked scope 共 1,838 条 Git index record，SHA-256 为 `5550d84ad745a0e88fff4228c5f5682fd1b345892aa5a6d97a808096175db4a7`。实现前必须重取该指纹，并把当前 worktree 中其他 Session 的改动视为外部状态。

### 2.2 定量与动态证据

| 检查 | 结果 | 可支持结论 |
|---|---|---|
| Cargo manifest bench scan | 161 个 manifest；无 `[[bench]]`/Criterion/Divan/Bencher | 没有常规 benchmark owner；不能把 ignored tests 等价为 bench suite |
| Rust ignored scan | 139 文件 / 190 ignore；118处性能/手工/GPU关键词 | 性能证据默认不进入普通测试门，且形态高度分散 |
| PBR summarizer unit suite | 21 passed / 12.055秒 | summarizer内部旧schema自洽；不证明当前writer→capture→summarizer集成可用 |
| PBR provenance cross-read | writer/capture要求schema 2 managed；summarizer要求schema 1 local | 当前合法managed流程最终必失败，属于协议级P0 |
| UI path resolution probe | `alpha\..\..\escape` 解析到 `E:\escape-measured-01` | 未执行写入也足以证明child path未被root containment约束 |
| performance ledger recount | current union 17,261；pending 17,106；review 17,013 | 文档清单分别落后155/248，不能作为current-source authority |
| `docs/tests` inventory | 1,213 文件 / 约823 MiB；37 RDC约632 MiB、651 PNG约119 MiB | 大型二进制证据直接进入Git；没有root catalog/retention/currentness合同 |

本轮没有运行 GPU、WPR、Tracy、RenderDoc、跨硬件 benchmark 或故意 crash。静态报告不能替代这些动态验收，也不会把历史截图/RDC默认判为无效；结论是它们缺少统一的可追溯治理。

### 2.3 正向基线

- native plugin benchmark harness 要求 source manifest 和 Cargo profile，拒绝 debug assertion，使用 start/completion barrier，并输出 `zircon.native.benchmark/2`；可作为统一 microbenchmark result schema 的迁移输入。
- RenderExtract capture 拒绝非空输出目录，使用 `CreateNew` exclusive lease，冻结四个输入artifact并反复核对哈希；source fingerprint覆盖 HEAD、raw diff、changed tracked bytes和untracked bytes。
- RenderExtract process owner 使用 Windows Job Object、timeout、stdout/stderr drain 和进程树终止；evidence writer采用 no-overwrite publication，并验证关键JSON/PNG非空及哈希。
- PBR capture 区分 cold/warm、重复运行、viewer/HDRI fingerprint、GPU timing与可选RenderDoc replay；也明确披露未清理驱动/DX12 cache，避免虚构严格cold状态。
- diagnostic log 有 bounded queue、backpressure metrics、flush fence与 `sync_data`；panic hook先flush再委托原hook的顺序合理，只是缺少crash envelope与panic内容持久化。
- build/export 已知如何发现并stage `.pdb`/`.dbg`/`.dSYM`，staging manifest也能记录hash；缺口是symbol identity、分离、存储与symbolication，不是完全没有sidecar意识。

### 2.4 参考边界

- Bevy 维护独立 Criterion benchmark crate，覆盖 ECS、render、scene、tasks、transform 等子域，并使用可保存/比较的baseline；Zircon需要同等明确的bench ownership，但不要求复制其crate布局。
- Unreal Windows/Generic crash context 形成异常捕获、minidump、线程/模块/版本/日志上下文、CrashReportClient 与后续 symbol/debug helper 链；它证明 crash artifact、上传和symbolication必须是Build Set的一部分。
- Godot Windows SEH/signal handler 提供模块/ASLR信息、stack walk、symbol/line输出、engine version与script backtrace，是低于完整Crash Reporter规模时仍可达到的最低native crash基线。
- Unity Graphics performance tests 固定warmup/frame数量，记录marker CPU/GPU sample、memory和确定性scene/settings，并以测试标签接入runner；这比“手工打开场景观察Profiler”更接近可比较产品证据。
- Fyrox在本轮选定源码中未发现与上述 benchmark lab、Crash Reporter或symbol service等价的集中owner；报告不据此虚构参考能力。

## 3. 当前 P0

### TOOL-PERF-P0-001 · Scenario 可逃逸 profile 与 tracked evidence 根目录

`ui-profile-capture.ps1` 对 `Scenario`/`ScenarioList` 没有 `ValidateSet`、slug codec或containment检查。`SessionId` 直接由timestamp、raw scenario、phase与ordinal拼接，再进入 `Join-Path $OutputPath $SessionId`、profile-project路径和 `VerificationScreenshotRoot/$SessionId`。这些位置会创建目录、删除ready marker、重定向日志、写profile与复制截图。必须先把scenario解析为注册表中的stable ID，再以 `GetFullPath` + component-boundary containment验证所有child path；未知场景必须在任何文件系统动作前拒绝。补充恶意 `..`、绝对路径、UNC、separator、reserved device name和symlink/reparse测试。

### TOOL-PERF-P0-002 · PBR provenance schema 使合法 managed capture 必然汇总失败

`write_zircon_shader_pbr_build_provenance.ps1:76-77` 生成schema 2 / `zircon_managed_viewer_artifact_provenance`，capture在`zircon_profile_shader_pbr_viewer.ps1:135`明确只接受该组合；但最终必调的 `zircon_summarize_shader_pbr_profile.py:436-437` 只接受schema 1 / `zircon_local_viewer_capture_provenance`。必须定义单一versioned provenance schema与generated decoder，先让summarizer兼容并验证schema 2完整字段，再硬切旧local schema。加入writer→capture contract→summarizer的真实临时目录集成测试；发布前必须原子写completion receipt，失败目录标为incomplete。

### TOOL-PERF-P0-003 · 没有可接受的性能基线却允许形成性能完成结论

仓库有634个PERF finding和562份performance文档，但accepted ledger为0；没有常规bench target、硬件归一化、统计置信、regression budget或merge/release consumer。当前流程可以生成日志和截图，却不能回答同一Build Set是否比基线快、变化是否超出噪声，更不能支持“优于当前Unreal”的可审计声明。必须建立performance promotion gate：场景、输入、Build Set、硬件、OS/driver、power policy、sample distribution与budget全部绑定，只有独立acceptor签名的comparison receipt才能更新baseline。

### TOOL-PERF-P0-004 · 产品崩溃后没有可符号化的native crash artifact

Rust panic hook只flush既有日志，随后让默认hook向stderr输出；access violation、native DLL fault、abort、OOM、hang与GPU fault没有统一捕获。崩溃机器上因窗口关闭或父进程退出丢失stderr后，当前没有minidump、线程栈、模块清单、build/debug ID或symbolication请求，无法定位引擎级故障。Windows最低门应是out-of-process-safe crash writer + minidump + bounded crash context；Linux/macOS分别定义signal/core与platform report策略。Crash handler不得依赖普通allocator、全局锁、日志worker或网络上传成功。

## 4. Benchmark 与长期趋势差距

### TOOL-PERF-P1-001 · 没有独立 benchmark target 或 crate

161个manifest中没有 conventional bench声明或框架依赖。建立按runtime core、ECS/task、asset/scene、renderer、editor workflow和tooling划分的bench workspace，禁止把benchmark继续藏在普通unit test命名空间。

### TOOL-PERF-P1-002 · 性能测试默认由 `#[ignore]` 隔离

native plugin、event bus、diagnostic log、ECS 100k fixture和多类WGPU/manual visual证据都依赖ignored test。为每类定义明确lane、resource class、timeout与artifact policy；`ignore`只能表达环境要求，不能成为永久不可见状态。

### TOOL-PERF-P1-003 · 代表性harness缺少跨进程统计模型

native plugin harness只有一个外层elapsed interval和可选bounded percentile sample，没有跨process distribution、outlier policy、confidence interval、effect size或noise calibration。统一runner至少输出raw samples、median、MAD、p95/p99、bootstrap confidence和environment variance。

### TOOL-PERF-P1-004 · 没有 baseline comparator 与趋势数据库

结果大多打印到stderr/stdout或独立JSON，仓库没有统一ingestion、baseline pointer、time series或bisect入口。新增append-only result store和content-addressed raw artifact；Git只保存小型promotion receipt与baseline引用。

### TOOL-PERF-P1-005 · benchmark schema 按测试各自发明

当前可见 `zircon.native.benchmark/2`、`EVENTBUS_BENCH_V1` 与纯文本scale输出并存。定义versioned `BenchmarkRunEnvelope`，以typed metric ID、unit、aggregation、direction和sample vector替代自由文本parser。

### TOOL-PERF-P1-006 · Coordinator benchmark grant没有结果准入消费者

Coordinator已有benchmark resource grant与Job Object基础，但没有中央服务验证result是否来自获批ticket、冻结输入和实际process receipt。把grant ID、creation identity、command/action digest与result envelope绑定；caller不能自行声明结果可信。

### TOOL-PERF-P1-007 · `profile-feature-contract` 名称与行为不符

workflow只执行feature组合的 `cargo check`，没有采样profile或benchmark。若目标只是编译合同，应重命名为`profiling-feature-build-contract`；若保留现名，必须增加至少一个真实capture smoke和artifact decoder验证。

### TOOL-PERF-P1-008 · workflow从未启用三种profiling feature

当前lane没有构建 `profiling-chrome`、`profiling-tracy`、`profiling-memory` 的真实产品组合，因而不能发现backend、feature传播或linker问题。矩阵需覆盖互斥/组合policy和Editor/Runtime产品入口。

### TOOL-PERF-P1-009 · profile合同只有Ubuntu stable

WPR、PDB、DX12、Job Object和主要采集脚本均是Windows能力，但workflow没有Windows consumer；toolchain也未固定到repo pin。增加managed Windows lane，并将Rust/tool/SDK版本写入action identity。

### TOOL-PERF-P1-010 · workflow缺少超时、并发取消和artifact currentness

profile feature workflow没有job timeout、同PR concurrency cancellation或结果artifact检查。采用与引擎CI一致的bounded job、cancel-in-progress和schema decoder smoke，防止卡住或只验证命令exit。

## 5. UI Profile/Capture 差距

### TOOL-PERF-P1-011 · `SkipBuild` 实际是强制前置managed build确认

未传 `-SkipBuild` 时脚本总是throw，脚本自身从不build。重命名为`UseExistingManagedBuild`或改为显式`BuildReceiptPath`，并验证receipt与Editor/runtime DLL哈希；不要用“跳过”表达唯一允许路径。

### TOOL-PERF-P1-012 · 场景不是versioned registry

未知scenario会落入generic instruction，而不是被拒绝；参数、fixture、counter gate和操作序列分散在条件分支。建立versioned scenario registry，记录stable ID、owner、input schema、warmup、actions、required counters和budgets。

### TOOL-PERF-P1-013 · 默认是无界手工运行

`Scenario=manual`、`AutoCloseSeconds=0` 会同步启动Editor并等待人工退出，不产生可重复deadline。手工exploration与acceptance capture必须分成不同命令；acceptance lane强制自动操作、deadline和terminal receipt。

### TOOL-PERF-P1-014 · evidence gate默认关闭

`RequireScenarioEvidence` 是opt-in；缺counter、interaction、latency或截图一致性仍可留下看似完整目录。managed capture必须默认fail closed，exploratory模式则在manifest与目录名中明确`non_acceptance`。

### TOOL-PERF-P1-015 · WPR缺失只告警继续

请求 `UseWpr` 但找不到`wpr.exe`时仍继续，产物语义从system trace静默降级。请求的collector必须全部成功启动并在receipt列出；optional collector应由scenario policy声明，不能运行时悄悄变化。

### TOOL-PERF-P1-016 · WPR是未租约的机器全局资源

脚本直接 `wpr -start CPU -filemode` / `-stop`，没有通过Coordinator取得system capture lease，可能与其他Session互相停止或污染trace。为WPR/ETW、RenderDoc、Tracy server和GPU counter定义machine-scoped fenced lease。

### TOOL-PERF-P1-017 · Editor进程树没有统一supervisor

自动关闭围绕root `Process` 和 `Stop-Process`，没有Job Object、descendant ownership、creation identity和daemon restart语义。复用RenderExtract/Coordinator的managed process abstraction，保证子进程、继承pipe和超时可回收。

### TOOL-PERF-P1-018 · Tracy GUI被detached启动且无owner

`Start-Process $TracyProfiler` 后立即丢弃handle，capture结束不验证连接、不保存trace、不关闭实例。把Tracy分成collector server与可选viewer；acceptance只依赖可验证collector artifact。

### TOOL-PERF-P1-019 · evidence publication不是事务

UI链至少8处直接 `Set-Content`/`Add-Content`，并在同一目录逐步写manifest、日志、metrics与截图；没有staging generation、fsync/rename或final completion marker。中断后partial目录可能被误读为完整run。

### TOOL-PERF-P1-020 · 每次capture直接复制截图进tracked docs树

`Export-VerificationScreenshots` 以raw SessionId在 `docs/tests/editor/profile-captures` 建目录，既扩大P0路径影响，也把探索性运行默认变成Git工作区污染。默认写外部artifact store，只有promotion动作可生成不可变receipt与精选golden。

### TOOL-PERF-P1-021 · 三次measured run没有跨运行汇总

`MeasuredRunCount=3` 只是循环创建三个目录；脚本没有聚合distribution、比较run间噪声或对baseline判定。增加batch envelope，全部child receipt完成后才计算统计并发布parent completion。

### TOOL-PERF-P1-022 · 缺少完整机器与负载清单

capture option记录了操作参数，却未形成CPU/GPU/内存、BIOS、OS build、driver、显示模式、电源计划、温度/频率、后台负载与虚拟化状态的强制manifest。无法在机器间比较，也无法识别thermal throttling。

## 6. PBR 与 RenderExtract 采集差距

### TOOL-PERF-P1-023 · PBR与UI source manifest依赖手工关键文件清单

两条链都没有使用编译器dependency closure或Build Set action graph；新依赖未加入脚本时，旧binary仍可能被判fresh。source identity应来自实际build receipt及其完整input tree，而不是手工维护“critical sources”。

### TOOL-PERF-P1-024 · dirty fingerprint只hash `git status`文本

UI/PBR helper把dirty identity建立在porcelain状态行，内容变化但path/status相同会碰撞。迁移到RenderExtract的tracked diff bytes + untracked content hashes，最终由Build Set CAS tree统一拥有。

### TOOL-PERF-P1-025 · PBR多文件输出没有原子完成协议

PowerShell `Set-Content` 与Python `write_text` 逐项发布，schema P0触发时会留下partial profile。所有raw/derived artifact先写unique staging root，逐个hash，最后以create-new completion receipt提交。

### TOOL-PERF-P1-026 · PBR的WPR同样没有全局lease

PBR脚本虽在请求WPR时fail closed，但仍直接控制系统级WPR session。接入统一capture resource lease，并在stop失败时发布明确aborted状态而非普通partial目录。

### TOOL-PERF-P1-027 · RenderDoc定位与图形API策略硬编码

默认RenderDoc DLL依赖本机D盘路径，采集固定DX12，缺少tool package digest、版本兼容表和Vulkan/其他backend policy。工具必须由versioned toolchain manifest解析，capture记录实际API/device/driver。

### TOOL-PERF-P1-028 · cold/warm cache语义不完整

PBR明确不清理driver/DX12 cache，这是诚实披露，但现有名称仍不足以比较“cold”。定义engine cache、shader cache、OS file cache、driver cache四层状态；无法控制的层标为observed/uncontrolled，不进入严格cold claim。

### TOOL-PERF-P1-029 · PBR evidence继承Coordinator ticket信任缺口

capture校验ticket与artifact receipt是优点，但Coordinator报告06已确认caller可写passed且ticket不绑定candidate bytes。PBR promotion必须等待worker-signed、candidate-bound validation receipt，不能把现有ticket当最终信任根。

### TOOL-PERF-P1-030 · 单元测试固化旧provenance schema

21项summarizer测试全部通过，却只构造schema 1 local provenance，未调用当前writer/capture contract。增加跨语言contract fixture与真实脚本集成测试，并为旧schema只保留显式migration test。

### TOOL-PERF-P1-031 · offline summarizer没有独立验证artifact receipt

PowerShell capture检查Coordinator receipt，Python summarizer单独运行时只检查旧provenance字段；移动/替换文件后仍可能被汇总。summarizer必须验证同一Build Set receipt、artifact digest、tool identity与run manifest。

### TOOL-PERF-P1-032 · RenderExtract场景集固定且覆盖面窄

当前四个固定场景只覆盖pipelined first/steady、synchronous steady和editor first。把场景从脚本分支迁入versioned registry，扩展为规模梯度、资源压力、shader/PSO miss、streaming、multi-view与异常路径。

### TOOL-PERF-P1-033 · RenderExtract尚无CI或趋势消费者

其frozen input与receipt质量较高，但仓库未发现workflow自动运行、上传、比较或阻止回归。先在稳定专用Windows runner建立nightly trend，再选择低噪声指标进入PR budget。

### TOOL-PERF-P1-034 · RenderExtract输出根硬编码E盘

`E:\ZirconBuilds\mvp-perf` 是本机布局，不是resource policy。由Coordinator分配受租约artifact root并记录volume identity、free space与filesystem capability；禁止脚本自己假定盘符。

## 7. 崩溃与日志差距

### TOOL-PERF-P1-035 · panic内容不会进入file-backed log

hook先flush已有queue，再调用previous default hook；`panic_info`没有写入diagnostic sink。因此日志可durable却不含导致flush的panic message/location/backtrace。增加allocation-bounded emergency crash record通道，先写panic envelope，再flush普通日志。

### TOOL-PERF-P1-036 · panic hook只由两个binary安装

只有Editor与runtime preview调用 `install_process_log_panic_flush`；Hub、工具、测试host、generated product host和未来server没有process bootstrap contract。把crash/log bootstrap放进统一产品host入口并测试所有shipping binaries。

### TOOL-PERF-P1-037 · native fault与abort绕过Rust panic hook

FFI/plugin access violation、`abort`、stack overflow、allocator failure与runtime session abort不会走Rust unwind hook。建立platform crash layer，明确unwind、abort、SEH/signal和GPU device fault各自的capture path。

### TOOL-PERF-P1-038 · 没有backtrace、线程栈与模块清单

仓库未发现 `std::backtrace::Backtrace` 生产捕获，更没有all-thread stack/module/base-address清单。CrashEnvelope至少绑定faulting thread、other thread IDs/names、module load addresses和register/context摘要。

### TOOL-PERF-P1-039 · 没有独立Crash Reporter与延迟上传队列

崩溃进程本身不应承担网络、压缩和UI。新增最小out-of-process reporter：本地spool、用户隐私选择、附件预算、重试、去重、redaction和可离线导出；服务端可后置，但本地envelope协议必须先稳定。

### TOOL-PERF-P1-040 · 日志目录只有秒级时间身份

同一channel在同一秒启动可共享目录/文件并append，缺少PID、process creation identity、session ID与Build Set ID，两个执行的日志会混合。使用不可冲突run ID并在每条日志头绑定process/build/session。

### TOOL-PERF-P1-041 · 早期启动崩溃没有file sink上下文

hook安装早于log初始化，早期panic只能落默认stderr。提供bootstrap emergency file或由supervisor预创建crash pipe，使loader、配置和日志初始化阶段也有artifact。

### TOOL-PERF-P1-042 · 没有hang、OOM和GPU crash分类

崩溃并非唯一终态；deadlock、长帧、device removed和内存耗尽需要不同证据预算。定义watchdog heartbeat、OOM reserve、GPU device-fault breadcrumb与manual hang dump，避免统一归为“进程没响应”。

## 8. 符号与 Build Identity 差距

### TOOL-PERF-P1-043 · release/profile没有显式symbol与panic策略

根Cargo只定义profiling继承release、`debug=true`、`strip=false`，没有shipping release的debug/split-debuginfo/strip/panic矩阵。按platform/product/channel声明code artifact与symbol artifact策略，并用CI检查实际binary metadata。

### TOOL-PERF-P1-044 · debug sidecar与产品包没有分离安全边界

build/export直接复制相邻`.pdb/.dbg/.dSYM`，可能让内部符号进入可分发目录。staging必须产生独立code bundle与restricted symbol bundle，各自有ACL、retention与promotion receipt。

### TOOL-PERF-P1-045 · 没有build/debug ID到symbol bundle的索引

当前manifest可hash文件，却没有PE CodeView GUID/age、ELF build-id、Mach-O UUID或Rust artifact identity索引。symbol service必须按平台debug identity查询，不能靠文件名猜测。

### TOOL-PERF-P1-046 · 同名PDB碰撞没有系统性消除

仓库历史证据已出现 `zircon_runtime.pdb` filename collision warning。以target triple、profile、feature/Build Set、crate instance和debug ID生成唯一symbol record；stage时检测同名不同hash并fail。

### TOOL-PERF-P1-047 · 没有crash-to-symbol集成测试

现有FFI panic guard和source contract不等于crash pipeline test。每个平台用subprocess故意触发panic/native fault，验证dump非空、build ID可解析、symbol bundle可定位、top frames可符号化且敏感路径已redact。

## 9. Evidence 与计划治理差距

### TOOL-PERF-P1-048 · `docs/tests` 已成为约823 MiB二进制仓库

1,213个tracked文件共862,902,570 bytes；37个RDC约632MiB、651个PNG约119MiB，另有ZIP/JPG/HDR等。Git历史会永久放大clone/fetch成本。将大raw artifact迁移到CAS/object storage，Git保留小型receipt、thumbnail和必要golden。

### TOOL-PERF-P1-049 · 没有LFS、CAS或artifact backend contract

仓库没有`.gitattributes`为RDC/PNG定义filter/diff/merge，也没有root级artifact provider。建立hash-addressed blob API、immutable upload、checksum-on-read、ACL、quota与offline cache；迁移前先做历史容量和引用盘点。

### TOOL-PERF-P1-050 · 证据根没有catalog/manifest

`docs/tests` 只有少量局部JSON/Markdown，没有枚举owner、scenario、source/build、tool、hardware、status与supersedes关系的root catalog。生成机器可读catalog，并让每个报告只引用receipt ID而非猜日期文件名。

### TOOL-PERF-P1-051 · 没有retention、currentness与promotion policy

1,105个证据文件名带日期，但日期不能证明仍对应current source；旧capture也没有superseded/tombstone状态。定义raw、accepted baseline、golden、failure、diagnostic各自保留期与晋升规则，删除只针对unreferenced CAS blob。

### TOOL-PERF-P1-052 · performance ledger计数已漂移

`pending.md` 与 `review.md` 对Rust总量分别落后当前口径155与248。清单必须由同一scanner生成，记录commit/tree、include/exclude pathspec和scanner version；手写总数只作为历史说明。

### TOOL-PERF-P1-053 · 634个finding仍没有accepted projection

`review.md` accepted table为空，不能区分“未验证”“已拒绝”“已修复待复测”和“当前基线”。建立finding state machine及独立acceptor receipt；文档checkbox不是验收事务。

### TOOL-PERF-P1-054 · 单个783 KiB审计文档不可维护

634行级表项集中在一个Markdown文件，review、diff、owner分配与机器解析成本过高。将finding存为typed records，按subsystem生成只读视图；保留稳定ID和历史链接。

### TOOL-PERF-P1-055 · 562份文档没有统一机器状态

541份日期文件、failure/fixed与其他记录并存，但没有schema、index或生成规则。引入performance registry并从receipt/finding自动生成索引，禁止继续用新日期文档表达状态迁移。

### TOOL-PERF-P1-056 · 对比Unreal缺少同条件方法学

“优于Unreal”必须定义版本、相同内容/画质、API、分辨率、硬件、driver、编译配置、warmup/cache、采样窗口和统计检验；否则结论不可复现。为每个claim发布comparison protocol、raw evidence、已知不等价项与置信边界。

## 10. P2 维护性差距

### TOOL-PERF-P2-001 · profile脚本参数语义过载

manual/acceptance、build selection、collector、自动交互、scale fixture与visual diff都集中在一个参数表。改为typed scenario manifest和少量命令模式，减少互斥组合。

### TOOL-PERF-P2-002 · 多个profile owner已超过可审查体量

UI capture 2,644行、PBR capture 883行、summarizer 935行。按path/build/process/collector/evidence/analysis拆模块，并让入口只编排typed phase。

### TOOL-PERF-P2-003 · D/E/F盘符被当作工具发现策略

profile path与RenderDoc候选依赖固定盘符。改由toolchain manifest、Coordinator resource provider和显式CLI override解析，报告实际resolved path及digest。

### TOOL-PERF-P2-004 · screenshot threshold只适合粗筛

采样RGBA channel delta阈值可发现大差异，但不能替代色彩空间、HDR、alpha、perceptual或region-aware oracle。保留为smoke metric，并为产品golden引入线性空间/感知差异。

### TOOL-PERF-P2-005 · console输出是人读协议

多个harness仍靠`println!`/`Write-Host`提示路径与结果。所有acceptance状态应由schema receipt表达，console只渲染receipt摘要。

### TOOL-PERF-P2-006 · 历史日期命名缺少稳定alias

日期可用于审计时间，但消费者需要stable scenario/baseline ID。由catalog维护`current accepted`指针，不要通过目录排序推断最新证据。

### TOOL-PERF-P2-007 · evidence文件扩展名没有diff策略

即便迁移到artifact store，少量golden仍需明确binary/text diff和review preview。为保留在Git的类型配置attributes与生成thumbnail规则。

### TOOL-PERF-P2-008 · profile preset工具与workflow命名不一致

PowerShell/Python helper、Cargo feature和workflow对profile概念使用不同词汇。定义canonical profile ID与generated projection，避免脚本分别解释feature组合。

### TOOL-PERF-P2-009 · 性能计划混合审计、失败、修复与验收记录

四类记录生命周期不同，应分typed registry或明确目录，不靠文件名前缀推断语义。

### TOOL-PERF-P2-010 · 本地工具版本只被零散记录

WPR、Tracy、RenderDoc、Perfetto和Python/PowerShell版本应进入统一tool manifest；展示工具版本不等于校验其digest和兼容范围。

## 11. 目标架构

### 11.1 统一证据模型

建立由 Coordinator 或独立服务拥有的 `PerformanceEvidenceService`，核心对象为：

1. `PerformanceScenario`：stable ID、schema version、owner、输入生成器、warmup/cache contract、required collectors、metric definitions与budgets。
2. `BuildSetReceipt`：source tree、toolchain、target/profile/features、binary与symbol identities、完整action/input closure。
3. `CaptureSession`：resource leases、process creation identity、machine manifest、collector receipts、child run状态和terminal completion。
4. `BenchmarkRunEnvelope`：raw samples、units、aggregation direction、statistics、noise calibration与artifact hashes。
5. `ComparisonReceipt`：candidate/baseline identity、方法学、effect/confidence、budget decision、acceptor与supersedes关系。
6. `CrashEnvelope`：crash GUID、process/build/session、platform context、dump/log/trace附件、privacy/redaction状态。
7. `SymbolBundleReceipt`：platform debug IDs、bundle hash、ACL、retention、source mapping与symbolication validation。

raw artifact进入内容寻址store；Git只保存schema、精选golden、小型receipt与baseline pointer。任何“passed”“accepted”“current”都必须由完整terminal receipt表达，不能由目录存在或caller布尔值推断。

### 11.2 进程与采集边界

UI、PBR、RenderExtract、benchmark与crash subprocess统一使用managed process tree：creation identity、Job Object/process group、bounded pipes、deadline、cancel、restart reconciliation和terminal receipt。WPR/ETW、Tracy、RenderDoc、GPU counters使用machine-scoped fenced lease。collector start/stop失败必须使run进入aborted/failed，不留下可晋升的半成品。

### 11.3 Crash/Symbol 最小产品链

shipping host在最早入口初始化emergency crash channel，由platform crash layer生成bounded local envelope和dump；独立reporter负责spool、redaction、用户交互与上传。Build阶段抽取debug ID并发布受限symbol bundle；symbolication worker按dump identity解析，结果回写immutable derived receipt。日志、profile与crash以同一Build Set/run ID关联。

## 12. 依赖顺序

### M0 · 立即阻断错误产物

修复Scenario path containment与PBR provenance schema；加入跨脚本集成测试和partial目录拒绝规则。M0完成前，不得把新UI/PBR capture晋升为accepted evidence。

### M1 · Evidence schema 与Build Set绑定

定义scenario/build/capture/result/comparison receipt，统一source dirty-byte fingerprint与tool identity；修复Coordinator worker-signed ticket和candidate binding依赖。

### M2 · Benchmark lab

建立独立bench workspace、统计runner、稳定Windows/Linux runner、noise calibration、trend ingestion和nightly baseline；从native plugin/event/ECS/logging迁移首批ignored tests。

### M3 · Capture pipeline收敛

把UI、PBR和RenderExtract接入统一process supervisor、resource lease、staging publication与artifact store；按versioned scenario registry扩展场景。

### M4 · Crash与symbol链

完成Windows minidump/CrashEnvelope、panic/native fault/hang基础；建立symbol extraction/index/store/symbolication与故意crash集成测试，再扩展Linux/macOS。

### M5 · Evidence catalog与迁移

生成 `docs/tests` 引用图和catalog，上传大raw artifacts到CAS，保留可审计receipt；制定retention/currentness/supersedes policy，避免无引用删除。

### M6 · 性能晋升与跨引擎比较

为关键帧、CPU/GPU、memory、startup、cook/import/editor interaction定义预算和comparison protocol；nightly稳定后再选择低噪声PR gate。任何“优于Unreal”声明必须绑定公开方法学和accepted receipt。

## 13. 验收门

1. 任意scenario字符串都不能让profile/project/log/screenshot路径越出allocated root；UNC、绝对路径、`..`、separator、reserved name和reparse用例通过。
2. PBR schema 2 writer→capture→summarizer集成通过；旧schema只能通过显式migration入口读取。
3. 中断任一capture phase后不存在可被catalog识别为complete的目录。
4. 所有acceptance capture默认fail closed；exploratory输出带不可晋升标记。
5. WPR/Tracy/RenderDoc/GPU collector均持有fenced lease并有start/stop receipt。
6. UI/PBR/RenderExtract全部由统一managed process tree监督，timeout后无残留descendant或继承pipe。
7. 至少六个核心子域有独立benchmark target，普通`cargo test`与benchmark lane所有权清晰。
8. benchmark result包含raw samples、machine/tool/build/input identity和统计置信，不只包含summary文本。
9. baseline更新需要独立comparison acceptor；caller不能自行写passed/accepted。
10. nightly trend可定位回归引入的Build Set，并保留raw evidence。
11. Windows故意Rust panic与native access violation均生成非空dump、CrashEnvelope和file-backed panic/context记录。
12. crash handler在日志未初始化、worker deadlock和低内存模拟下仍能生成bounded emergency artifact。
13. dump携带可解析debug ID，symbol service能定位唯一bundle并符号化至少顶部引擎frames。
14. code bundle与symbol bundle物理分离；shipping package不含未授权private symbols。
15. 同名不同hash PDB在stage时fail，不再靠warning继续。
16. hang watchdog与GPU device-lost路径产生不同分类的envelope和附件。
17. `docs/tests` 有机器可读catalog，所有accepted evidence可追到source/build/tool/hardware/scenario。
18. 大型RDC/trace/raw screenshot不再直接新增到Git；CAS read-back校验hash。
19. retention只删除无引用blob，accepted baseline/golden和审计receipt不可被误删。
20. performance ledger由scanner生成，Rust文件计数与同一tree/pathspec可复现。
21. 634个既有finding全部进入明确state，不再由空accepted表和散落checkbox推断。
22. profile feature workflow构建真实profiling feature组合，并在Windows运行schema/capture smoke。
23. performance workflow具有timeout、并发取消、artifact currentness与失败诊断。
24. 跨Unreal比较记录版本、内容/画质、硬件、driver、配置、cache、采样和统计方法，并列出不可比项。

## 14. 本轮边界

- 本轮没有修改或执行生产capture、WPR、Tracy、RenderDoc、GPU benchmark、crash handler、symbol upload或artifact迁移。
- 运行时 profiler ring、scope、counter、Perfetto exporter本身由 `zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md` 拥有；本报告只拥有工具编排、证据准入和长期基线。
- PBR viewer渲染正确性、offscreen/native swapchain差距继续由 `zircon_app/02-pbr-viewer-tool-runtime-evidence-renderdoc-review.md` 拥有；此处只拥有其capture/provenance/publication链。
- Coordinator认证、validation ticket、candidate binding和process supervisor根缺口由报告06拥有；本报告只定义性能/采集消费者对可信receipt与lease的要求。
- `docs/tests` 迁移前必须生成引用图、备份与逐blob校验；本报告不授权删除历史证据。
- source、性能计划与证据目录均可能继续变化，实施前必须重取scope fingerprint并复核所有P0。
