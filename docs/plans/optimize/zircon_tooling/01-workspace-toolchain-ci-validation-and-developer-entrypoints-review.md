---
related_code:
  - Cargo.toml
  - Cargo.lock
  - deny.toml
  - zircon_plugins/Cargo.toml
  - zircon_plugins/Cargo.lock
  - zircon_runtime/Cargo.toml
  - zircon_runtime/runtime-feature-presets.toml
  - zircon_app/Cargo.toml
  - .github/workflows/ci.yml
  - .github/workflows/profile-feature-contract.yml
  - .github/workflows/mvp-editor-windows.yml
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1
  - tools/check_conventions.py
  - tools/check-conventions.ps1
  - tools/convention_exemptions.py
  - tools/runtime_domain_dependency_audit.py
  - tools/check-runtime-domain-features.ps1
  - tools/check-runtime-profile-features.ps1
  - tools/runtime-profile-feature-presets.py
  - tools/dev-fast-build.ps1
  - tools/dev-fast-aliases.ps1
  - tools/dev-module-interactive.ps1
  - tools/README-fast-build.md
tests:
  - .codex/skills/zircon-dev/scripts/validate-matrix.Tests.ps1
  - tools/tests/test_check_conventions.py
  - tools/tests/test_frameworks_03_domain_feature_matrix.py
  - tools/tests/test_frameworks_03_profile_feature_presets.py
  - tools/tests/test_frameworks_06_ci_toolchain_contract.py
  - tools/tests/test_frameworks_06_dependency_governance_contract.py
  - tools/tests/test_runtime_domain_dependency_audit.py
  - tools/tests/dev-fast-build.Tests.ps1
  - tools/tests/mvp_editor_windows_workflow.Tests.ps1
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
reference_engines:
  - dev/bevy/Cargo.toml
  - dev/bevy/.github/workflows/ci.yml
  - dev/bevy/.github/actions/install-linux-deps/action.yml
  - dev/bevy/deny.toml
  - dev/bevy/tools/ci/src/commands/clippy.rs
  - dev/Fyrox/.github/workflows/ci.yml
  - dev/Fyrox/fyrox-build-tools/src/export
  - dev/godot/.github/workflows/linux_builds.yml
  - dev/godot/SConstruct
  - dev/UnrealEngine/Engine/Build/InstalledEngineBuild.xml
  - dev/UnrealEngine/Engine/Build/LowLevelTests.xml
  - dev/UnrealEngine/Engine/Build/Graph/Tasks/PGOProfileProject.xml
  - dev/UnrealEngine/Engine/Build/Graph/Tests/DDCVerify.xml
  - dev/Graphics/.yamato/wrench/wrench_config.json
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 01 · Workspace、Toolchain、CI、Validation 与 Developer Entrypoint 工程化差距

## 1. 结论

ZirconEngine已经拥有一些值得保留的工程化底座：两个lockfile均被提交，`deny.toml`默认拒绝未知source并保持advisory零忽略，Runtime profile有机器可读TOML，Windows验证器会把Cargo派生物限制到受管物理盘并生成哈希证据，MVP Windows workflow也显式校验了多条产品链。这些不是临时脚本的空壳。

但当前底座尚未形成“一个可复现的Build Set authority”。根workspace显式列出10个member，Cargo实际解析出36个member，其中26个又属于独立的139包`zircon_plugins` workspace；同一批包因此受两个workspace root、两个lockfile和两套profile上下文影响。该歧义已经从架构风险变成当前P0：`zircon_plugins/Cargo.lock`与manifest失配，`cargo metadata --locked --manifest-path zircon_plugins/Cargo.toml`在任何编译前确定失败，所以CI中的plugin check/build/test/dist和plugin `cargo-deny`均无法开始。

CI目前也不能证明“支持的平台、profile和发布物真的可用”。正常workspace只在Linux执行完整build/test；Windows只有窄MVP链；所谓八平台export matrix全部运行在Ubuntu，实际只执行一个读取环境变量的policy unit test，没有安装target SDK、交叉编译、链接、打包、启动、部署或验证制品。349个tracked Python测试模块与35个tracked PowerShell测试文件绝大多数不进CI；本轮抽取62项合同测试就复现1项已漂移失败。当前工程更接近“拥有大量局部守卫”，而不是Unreal BuildGraph/AutomationTool、Godot SCons平台矩阵、Fyrox真实模板export或Bevy跨平台CI那样的统一验证产品。

本轮记录1个P0、40个P1和8个P2。没有修改生产代码、manifest、lockfile、workflow或脚本；`cargo-zircon`、export/package实现、derive/codegen、Session Coordinator、性能采集和release promotion将在后续`zircon_tooling`报告中独立深审。

## 2. 审查边界与可复验证据

### 2.1 物理范围

| 子域 | 文件/规模 | 本轮状态 |
|---|---:|---|
| Workspace/toolchain/dependency | 根与plugin Cargo manifest/lock、`deny.toml` | E3：静态读取并执行locked metadata、workspace member和duplicate dependency探测 |
| GitHub Actions | 3个workflow / 731行 / 17个job-like mapping | E3：逐job读取触发器、toolchain、matrix、命令、缓存、artifact与failure policy |
| Windows统一验证器 | 2个文件 / 3,488行 / 104个Pester `It` block | E3静态、E2行为：实现与测试逐段读取；未运行完整Pester/Cargo矩阵 |
| Convention/feature/profile/domain guards | 10个核心脚本与聚焦测试 | E3：入口、解析、命令计划、失败传播和产品调用点 |
| Developer fast-build入口 | 8个脚本/包装/文档/测试 | E3：参数、feature映射、环境lease、CMD转发和文档命令 |
| `tools`总体 | 约338,551 tracked lines | E1 inventory：仅用于证明规模；export、coordinator、profile和其余测试未据此宣称深审 |

本轮clean scoped set fingerprint为`5129c03c88beb3b577f4d94f9e9f0b7f3cce75b272d9b54002982888a427bb2e`。算法是按报告列出的核心文件顺序计算Git blob hash，再对hash列表计算SHA-256；它只用于实施前识别源漂移，不等价于release Build Set ID。工作树存在其他Session修改，本报告没有回退或吸收那些修改。

### 2.2 动态验证

以下只读命令通过：

```powershell
cargo metadata --format-version 1 --no-deps --locked
cargo metadata --format-version 1 --locked
cargo tree --duplicates --workspace --locked --depth 0
python tools/runtime_domain_dependency_audit.py --pretty
```

结果包括：根workspace实际有36个package；34个dependency family同时存在多个版本，共72个版本实例。domain audit用约18.1秒输出19,557行，报告2,741条production reference和72条direct domain edge，但无论发现何种edge，程序最终都返回0。

以下命令确定失败：

```powershell
cargo metadata --format-version 1 --locked --manifest-path zircon_plugins/Cargo.toml
```

Cargo在编译前返回：`cannot update the lock file ... zircon_plugins/Cargo.lock because --locked was passed`。同样原因，plugin workspace的locked duplicate tree也无法生成。没有移除`--locked`或重写lockfile，因为本轮是审查且用户未授权依赖解析变更。

Developer wrapper也通过只读调用复现参数失败：

```powershell
cmd /c tools\dev-fast-client-check-debug.cmd
```

`dev-fast-build.ps1`拒绝wrapper传入的`client`；合法集合是`minimal/client2d/client3d/editor/dev/server`。交互脚本仍传`client`，并请求当前manifest不存在的`plugin-graphics-base`、`plugin-physics`、`plugin-sound`等feature。

聚焦Python合同测试结果为61通过、1失败：

```powershell
python -m unittest `
  tools.tests.test_check_conventions `
  tools.tests.test_frameworks_03_domain_feature_matrix `
  tools.tests.test_frameworks_03_profile_feature_presets `
  tools.tests.test_frameworks_06_ci_toolchain_contract `
  tools.tests.test_frameworks_06_dependency_governance_contract `
  tools.tests.test_runtime_domain_dependency_audit -v
```

失败项`test_minimal_app_does_not_mount_diagnostic_log_startup_parsing`仍要求`zircon_app/src/composition/mod.rs`包含`diagnostic_log_args`模块，当前clean tracked source已不存在该声明。这是守卫实现形状漂移的直接证据，不在审查轮次中猜测应恢复旧模块还是重写行为合同。

### 2.3 已知外部阻断

Hub 01报告已经独立复现`zircon_hub`的`persist_unchecked(None)`编译P0。即使本报告的plugin lock P0修复，根`cargo build/test --workspace --locked`仍会先被Hub编译错误阻断。本报告不重复分配Hub finding ID，只把它列为验证前置依赖。

## 3. Build Set 与 Workspace 所有权差距

### TOOL-CI-P0-001 · Plugin lockfile已使全部locked plugin门禁失效

`zircon_plugins/Cargo.toml`和`zircon_plugins/Cargo.lock`当前不一致。CI明确对plugin workspace执行`cargo check/build/test --workspace --all-features --locked`、dist tests和`cargo deny ... --locked`语义；这些job都会在解析阶段失败。P0修复不能只是本机运行一次`cargo update`：必须确定canonical workspace ownership、在预期toolchain下生成lock、审查依赖差异，并增加`cargo metadata --locked`作为最前置、秒级的lock freshness gate。

### TOOL-WS-P1-001 · Package同时落入两个workspace上下文

根manifest显式10个member，但root metadata通过path dependency收纳26个plugin package，合计36个；从某个plugin子manifest执行`cargo locate-project --workspace`又解析到`zircon_plugins/Cargo.toml`。同一package由调用位置决定workspace root、target目录、profile、patch和lock语义，不具备单一owner。需要在“单一monorepo workspace”与“明确exclude、独立发布的plugin workspace”之间硬切，不接受隐式双归属。

### TOOL-WS-P1-002 · 两套lock/profile根没有Build Set绑定

根与plugin workspace各自定义`profiling` profile并提交lockfile，却没有顶层build manifest记录两个lock hash、engine revision、Rust toolchain、host/target、feature/profile和生成器版本。任何“root通过、plugin另一个依赖图”的组合都可能被误称同一引擎版本。应发布不可变Build Set manifest，所有binary/package/symbol/evidence都引用同一个ID。

### TOOL-WS-P1-003 · Rust toolchain与MSRV未被仓库固定

仓库没有`rust-toolchain.toml`，所有root/plugin package均无`rust-version`。CI一部分固定1.94.1，一部分使用随时间变化的`stable`，feature脚本默认`nightly`，普通本地Cargo又取用户默认。当前本机是1.94.1并不能证明复现性。需要分别定义开发、CI、MSRV和可选nightly工具链，禁止不同job无声漂移。

### TOOL-WS-P1-004 · Workspace lint policy不存在

根manifest未声明`[workspace.lints.rust]`或`[workspace.lints.clippy]`，也没有root `clippy.toml`/`rustfmt.toml`。只有少数crate被专项clippy，unsafe、panic、large error、undocumented unsafe、unexpected cfg等规则无法统一继承。应由workspace拥有分层lint集，例外必须是可到期、可定位、可计数的结构化记录。

### TOOL-WS-P1-005 · Shipping/release profile仍是Cargo默认值

两套workspace只增加`profiling = release + debug=true + strip=false`。没有shipping、editor、server、size、wasm、dev-optimized等profile，也没有LTO、codegen-units、panic、strip、split debuginfo、incremental与平台覆盖策略。对于以性能优于Unreal为目标的引擎，不能用默认`release`作为最终性能或交付合同。

### TOOL-WS-P1-006 · Public/private package与版本发布政策未定义

根36个package中34个未显式`publish = false`，plugin 139个全部未指定；所有package缺`rust-version/repository/readme`，并有缺description项。这会让内部实现crate在误操作时进入发布面，也无法生成可信package catalog。需要按internal、SDK、tool、first-party plugin、redistributable runtime分类，而不是给所有crate补同一模板。

### TOOL-DEP-P1-007 · 多版本依赖只告警，没有关键singleton政策

当前有34个duplicate family，包括`glam` 0.30/0.32/0.33和多代`hashbrown`。`deny.toml`只把multiple versions设为warn，没有为数学ABI、window handle、accessibility、render graph关键类型建立deny list。应先识别跨ABI/序列化/资源身份传播的singleton，再按owner收敛；不能机械强求所有transitive dependency单版本。

### TOOL-DEP-P1-008 · 依赖变更缺少自动更新、review和供应链制品

仓库没有Dependabot/Renovate、dependency review、SBOM、license bundle、provenance或attestation workflow。`cargo-deny`是必要底线，但不能证明二进制使用了哪个source tree、generator和toolchain。release promotion必须消费Build Set manifest，而不是重新解析浮动环境。

## 4. CI 与平台验证差距

### TOOL-CI-P1-009 · 正常workspace只在Linux完整build/test

主CI没有Windows/macOS完整workspace build/test。`mvp-editor-windows`只覆盖特定产品链，无法发现平台cfg、filesystem semantics、linker、window backend、DLL ownership或macOS framework问题。至少要有Linux/Windows/macOS三平台的分层build/test，并把昂贵产品验收建立在快速compile/link smoke之后。

### TOOL-CI-P1-010 · “八平台export matrix”没有构建任何目标

matrix名称列出Windows、Linux、macOS、Android、iOS、WebGPU、WASM、headless，却全部在Ubuntu执行单个Python policy test并注入target名字。它既没有`rustup target add`，也没有SDK/toolchain、link、template generation、package、launch或device验证。UI上呈现平台矩阵会制造false-green；应改名为policy contract，真实export验证由目标平台/交叉工具链job拥有。

### TOOL-CI-P1-011 · Feature/profile门只证明`cargo check`

domain feature和profile matrix只编译`--lib`或少数bin，没有负组合、pairwise组合、all-targets test、artifact内容或运行时profile选择验证。feature能编译不代表该模块被装配、启动、停机或打包。每个canonical profile应有compile contract、composition snapshot、启动/停机smoke和artifact inventory。

### TOOL-CI-P1-012 · Profile合同存在多份手写真相源

独立workflow、Windows validator、MVP F5 YAML、PowerShell脚本和Python测试都手写相近的七条Cargo命令；Runtime TOML又拥有六个逻辑profile。当前测试主要通过regex保证副本保持某种文本形状。应由一个typed manifest生成CI matrix、validator argv、MVP evidence和文档，禁止复制command string作为“canonical”。

### TOOL-CI-P1-013 · 绝大多数工具测试从未进入CI

仓库跟踪349个`tools/tests/test_*.py`模块和35个PowerShell Tests文件，主CI只直接选择少数Python module，并通过convention间接触发一个layering test；没有统一Pester job。本轮抽样已经发现1个clean source上的失败。需要测试发现、分片、timeout、JUnit和quarantine机制，并对“新增测试未进入任何suite”直接失败。

### TOOL-CI-P1-014 · Clippy只覆盖两个小crate

Convention gate只对`zircon_runtime_interface`和`zircon_app`执行clippy，未覆盖runtime、editor、RHI、Hub、derive、cargo-zircon和plugins；workspace build又不把warning提升为error。应建立workspace/all-targets/受支持feature集的clippy层，并为平台特定代码提供目标runner，而不是永久豁免。

### TOOL-CI-P1-015 · Doc、rustdoc与public API兼容性没有门禁

CI没有`cargo doc`、`RUSTDOCFLAGS=-D warnings`、broken intra-doc link、public API diff或SDK semver检查。对引擎/插件SDK而言，能编译不等于文档和兼容面可消费。需把internal crate与public SDK的要求分层，public surface还应生成版本化API snapshot。

### TOOL-CI-P1-016 · 缺少MSRV、Miri和sanitizer证据

没有MSRV job、Miri、ASAN/UBSAN/TSAN或平台内存工具。当前代码包含FFI、owned buffer、dynamic DLL、线程池和GPU lifetime，单一stable debug/release测试无法覆盖其主要风险。应先从runtime interface、plugin SDK、allocator ownership和任务系统建立小而稳定的高风险suite。

### TOOL-CI-P1-017 · 没有性能、内存、编译时和体积回归门

CI没有benchmark baseline、frame/CPU/GPU budget、memory high-water、shader/PSO warmup、binary/package size、incremental/full build时间或noise model。超越Unreal的目标必须转成场景、硬件、统计窗口和退化阈值；只保留`profiling` profile无法证明性能。

### TOOL-CI-P1-018 · GitHub Action与权限未做供应链加固

workflow使用`checkout@v5`、`setup-python@v5`、`rust-cache@v2`、`cargo-deny@v2`等tag，没有commit SHA pin；顶层未声明最小`permissions`，checkout也未统一关闭credential persistence。应建立允许列表、自动更新机器人和审核流程，第三方action不得靠可移动tag进入release链。

### TOOL-CI-P1-019 · CI运行控制不足

主CI没有`concurrency`/cancel-in-progress，大多数job无timeout，也缺`merge_group`、手动重跑入口和统一失败摘要。昂贵workspace job可能在旧commit继续占用runner，卡住的child process也没有有界退出。每个job必须有timeout、取消传播和日志/artifact保留政策。

### TOOL-CI-P1-020 · Linux依赖安装逻辑重复

同一大段`apt-get`包列表在多个job复制，版本、failure诊断和缓存策略会漂移。应使用仓内composite action或版本化container image，并把图形/音频/窗口/headless依赖拆成能力集。

### TOOL-CI-P1-021 · Release真实性未被验证

没有从immutable artifact执行install/launch/smoke、symbol关联、license/SBOM检查、签名验证或promotion；大多数job只验证源码树内Cargo输出。后续export报告需要把build、stage、package、install、run、uninstall和rollback串成同一receipt链。

## 5. 统一验证器与本地工程流差距

### TOOL-VAL-P1-022 · Validator被硬编码为Windows D/E/F盘产品

1,311行validator只接受`D:\cargo-targets`、`E:\cargo-targets`或`F:\cargo-targets`下的物理路径。保护系统盘目标的意图合理，但C-only开发机、Linux/macOS runner和容器都无法采用同一入口。应抽象storage policy与platform adapter，并允许显式配置的受管root，不把盘符写成架构。

### TOOL-VAL-P1-023 · Validator没有覆盖完整质量门

主动作只有build/test，加两个feature/profile特例；没有fmt、workspace clippy、doc、deny、MSRV、sanitizer、coverage、bench或all-targets。这使“统一验证器”实际只是Cargo命令包装。应由验证manifest声明gate graph、依赖、输入、输出、timeout与owner。

### TOOL-VAL-P1-024 · 默认验证不fail-fast也不做依赖调度

build失败后仍继续尝试test和合同矩阵，增加噪声和时间；相反，某些可并行的独立gate又完全串行。应区分hard dependency、always-run diagnostics和independent lane，失败传播由DAG决定，而不是脚本控制流偶然决定。

### TOOL-VAL-P1-025 · 低磁盘处理直接清空整个leased target

低于50GB时validator无条件`cargo clean`当前target，不先报告最大consumer、不做LRU、不保留可复用registry/source cache，也不在清理后重新验证可用空间。它可能毁掉数小时incremental cache后仍然失败。需要容量预算、分层eviction、lease-aware LRU和清理receipt。

### TOOL-VAL-P1-026 · Lane私有`CARGO_HOME`破坏用户/组织Cargo配置

validator和feature/dev-fast脚本把`CARGO_HOME`替换为target内目录。这样会丢失用户/组织的registry、source replacement、credential和net配置，并为每lane重复下载registry。应分离immutable shared registry/cache、credential/config projection和lane-local writable state，敏感信息不得复制进artifact。

### TOOL-VAL-P1-027 · Artifact证据没有可追溯manifest

当前发布逻辑主要复制profile目录里的平面文件并计算哈希，没有run identity、source revision、两个lock hash、toolchain、target triple、feature/profile、command argv、environment allowlist、symbol bundle、SBOM或签名。必须先定义Build Set/artifact manifest schema，再允许Hub/export/CI消费。

### TOOL-VAL-P1-028 · Test filter语义可能误跑或漏跑

validator普通test filter不自动附加`--exact`，ignored模式只追加`--ignored`。同名substring可执行多项或零项，却仍让调用方误以为验证了指定合同。入口应支持typed test selector，并对expected count、ignored/include ignored和package/target进行显式校验。

### TOOL-VAL-P1-029 · Compatibility identity没有覆盖所有编译输入

当前lane identity没有完整纳入`CARGO_ENCODED_RUSTFLAGS`、RUSTDOCFLAGS、linker、Cargo config/source replacement、lock hash和generator输入。Cargo自己的fingerprint能覆盖部分编译输入，但验证器不能因此把不同Build Set的证据放入同一逻辑lane。应由manifest记录外部输入，Cargo fingerprint继续负责内部增量正确性。

### TOOL-VAL-P1-030 · Validator测试偏重源码形状且本身不进CI

2,177行Pester测试含104个`It`，大量断言通过读取脚本文本、workflow regex和`Should Match`锁实现形状，真正mock进程/文件系统/失败恢复的比例很低。应保留少量结构合同，但把核心转为module级行为测试、临时受管root、fake cargo process和golden receipt；并在Windows CI实际运行。

### TOOL-VAL-P1-031 · JSON convention模式会缓冲完整child输出

`check_conventions.py --json`用捕获模式等待子进程完成，没有per-command timeout和live log forwarding。长Cargo输出会被完整留在内存，runner界面也可能长时间无进度。需要流式日志、bounded tail、heartbeat、timeout和结构化结果文件。

### TOOL-VAL-P1-032 · Convention入口绕过统一验证会话

Convention的fmt/clippy等命令直接调用Cargo，普通本地运行会落入repo `target`，没有managed target/session lease、run manifest或并发隔离。统一入口应该消费同一个gate graph；convention只选择gate，不再自行拥有Cargo执行器。

## 6. Convention、Feature 与 Domain Guard 差距

### TOOL-GUARD-P1-033 · 文档frontmatter使用手写子集解析器

路径审计没有使用YAML parser，只识别有限key/list形状；malformed frontmatter、缺closing delimiter、inline/nested YAML、glob、带空白测试引用等可能被忽略。既然文档已经是工程输入，应定义schema并使用标准parser，给出文件/行/字段级错误。

### TOOL-GUARD-P1-034 · “Guard coverage”不证明规则真的被执行

规则表审计主要验证guard label是否合法和文本存在；`review`、`process`等标签可以让MUST规则通过，却没有对应命令、测试、owner或失败证据。每条强制规则应链接可执行gate ID，纯人工规则必须有review owner和到期复核。

### TOOL-GUARD-P1-035 · Rust exemption治理只覆盖两个crate

allow/exemption审计集中在`zircon_app`和`zircon_runtime_interface`，未覆盖runtime、editor、RHI、Hub、derive、tools和plugins；workspace发现又只看显式root member，漏掉当前36个effective member和独立plugin workspace。应从Cargo metadata获得canonical package graph，并让每个例外绑定owner、reason、issue、expiry和budget。

### TOOL-GUARD-P1-036 · Runtime domain auditor不是架构门禁

803行自制Rust lexer维护硬编码domain列表：实际`zircon_runtime/src`已有`operation`和`runtime_diagnostics`却未列入，列表仍含不存在的`rhi`目录。工具只枚举`crate::domain`直接引用，忽略跨crate/plugin/interface边界，没有allowed direction或baseline，最终总返回0。19,557行报告证明它适合inventory，不证明依赖方向正确。

### TOOL-GUARD-P1-037 · Feature domain名单分散且只测单域正组合

domain集合在PowerShell、Python测试和CI中重复，未从manifest或模块descriptor生成；检查只组合`core-min + 单个domain`并限制`--lib`，不能发现非法反向依赖、成对feature冲突、bin/example/test目标缺失。应声明feature capability graph并生成正/负/pairwise矩阵。

### TOOL-GUARD-P1-038 · Profile逻辑身份与Cargo feature身份混淆

canonical TOML区分`client2d/client3d`以及`editor/dev`，但这些profile对分别映射同一Cargo feature；dev-fast只传Cargo feature，不把逻辑profile ID交给runtime composition。除非另有runtime参数，两个逻辑产品会生成和启动同一选择面。需要明确哪些差异是compile-time、assembly-time和runtime policy，并在artifact manifest中保留三者。

## 7. Developer Entrypoint 差距

### TOOL-DEV-P1-039 · 已提交的client快捷CMD确定不可用

`dev-fast-client-check-debug.cmd`与release wrapper仍传已删除的`client` profile；父wrapper还只判断是否为Release，不转发其余用户参数。README把这些入口描述为推荐路径，当前开发者会在编译前失败。入口必须由canonical profile manifest生成，并有真正执行到fake cargo argv的测试。

### TOOL-DEV-P1-040 · Interactive module工具同时使用过期profile和feature

`dev-module-interactive.ps1`的runtime target使用非法`Profile="client"`，并引用`plugin-graphics-base`、`plugin-physics`、`plugin-sound`、`plugin-animation`、`plugin-net`、`plugin-navigation`、`plugin-particles`、`plugin-texture`、`plugin-vg`、`plugin-gi`等当前不存在feature。runtime/editor两个选项都不能形成有效命令。应删除手写module表，改读package/catalog/capability manifest。

## 8. 次要但应纳入治理的差距

### TOOL-CI-P2-001 · 主CI缺少`workflow_dispatch`与merge queue合同

无法用同一revision手动复跑完整主链，也没有`merge_group`验证合并结果。应避免为手动入口复制workflow，统一复用同一gate graph。

### TOOL-CI-P2-002 · 仓库没有CODEOWNERS

workspace、lockfile、CI、FFI、shader和release脚本缺少强制review owner。CODEOWNERS不能替代架构，但能确保高风险路径不会只靠通用批准。

### TOOL-CI-P2-003 · 失败摘要与JUnit归档不统一

部分workflow上传MVP evidence，普通unit/tool tests没有统一structured summary。需要在不泄漏绝对路径/credential的前提下发布gate状态、duration、log hash和bounded diagnostics。

### TOOL-VAL-P2-004 · 本机Pester版本要求未声明

当前环境只发现Pester 3.4.0，仓库没有固定module版本或bootstrap contract。即使测试语法碰巧兼容，也无法复现Windows runner。应锁定并校验Pester版本。

### TOOL-GUARD-P2-005 · Domain audit默认输出不可供人审查

19,557行pretty JSON没有summary-first、top violations、baseline diff或机器/人类双格式。结果规模会掩盖新增边。应默认输出摘要和增量，完整graph作为artifact。

### TOOL-DEV-P2-006 · Fast-build README路径和profile词汇已经漂移

文档从repo root示例调用`.\scripts\...`，实际文件位于`tools`；profile仍写`client/server/editor`，与六个canonical ID不一致，并推荐已坏interactive入口。文档应从命令schema生成或由文档测试执行示例。

### TOOL-DEV-P2-007 · Fast-build固定共享target存在并发冲突

脚本按逻辑profile选择固定目录且绕过Session Coordinator。不同Session、toolchain或feature组合可能争用同一lease；反过来，映射相同Cargo feature的逻辑profile又重复target。应以Build Set compatibility key分lane，并由coordinator仲裁。

### TOOL-DEV-P2-008 · Fast-build测试没有覆盖公开入口

现有Pester测试只静态检查managed path/cache环境，没有执行CMD参数转发、六个profile、interactive feature解析、失败退出码或并发lease；同时不进CI。公开开发命令必须有argv golden和process-level smoke。

## 9. 参考引擎对照

### 9.1 Bevy：小型Rust workspace仍需要完整工程合同

Bevy root明确`rust-version`、edition 2024、resolver 3、workspace exclude和workspace lint；CI在Linux/Windows/macOS执行build/test，并有MSRV、Miri、WASM、docs、timeouts、最小permissions、concurrency、SHA-pinned action与`persist-credentials: false`。它还把Linux依赖安装抽成composite action，clippy覆盖workspace/all-targets/all-features并拒绝warning。Zircon无需照抄feature集合，但应达到同等的可复现性和门禁所有权。

### 9.2 Fyrox：平台名必须落到真实项目和制品

Fyrox CI实际生成并构建PC、Android和WASM template，Windows/Linux/macOS跑workspace all-targets/all-features；export工具拥有PC/WASM/Android的build、copy、package与run路径。Zircon当前八平台matrix只验证字符串合同，不能宣称同等平台覆盖。

### 9.3 Godot：Build configuration是typed product surface

Godot `SConstruct`显式拥有target、module、cache、SCU、LTO、sanitizer、arch、test和werror选项；workflow真实构建Android/iOS/Linux/macOS/Web/Windows，并在Linux覆盖GCC/Clang、ASAN/UBSAN/TSAN、precision与editor/template变体。Zircon需要先定义自己的typed build schema，再生成Cargo/SKD/CI命令，而不是让YAML、PowerShell和CMD各自手写。

### 9.4 Unreal：BuildGraph连接编译、安装、符号、测试与性能证据

`InstalledEngineBuild.xml`把Editor/Client/Server、平台、DDC、signing、source indexing、debug info、strip和Build ID组织成依赖图；LowLevelTests覆盖平台/部署/设备，PGO和DDC验证也是可执行graph task。Zircon不需要复制UBT/AutomationTool体量，但必须拥有同类的artifact graph、receipt和promotion authority。

### 9.5 Unity Graphics：package验证与promotion是依赖链

Yamato/Wrench配置把package catalog/schema、platform pretest、Editor/Playmode版本矩阵、日志/结果归档、pack和promotion连接起来。Zircon plugin workspace当前连locked metadata都不成立，更不能把“139个package存在”视为生态已工程化。

## 10. 重构路线

### M0 · 恢复可信绿线，不扩大功能面

1. 决定root/plugin的唯一workspace ownership，禁止隐式双归属。
2. 在选定toolchain下审查并重建plugin lockfile，先加`metadata --locked` freshness gate。
3. 独立修复Hub已有编译P0，恢复root workspace compile入口。
4. 修正或删除已坏CMD/interactive入口；对公开wrapper建立argv行为测试。
5. 将当前62项聚焦测试失败分类为产品回归或守卫漂移，不能简单删除断言换绿。

### M1 · 建立Build Set manifest与单一命令schema

1. 定义engine revision、workspace/lock hash、toolchain、host/target、profile、feature、generator、environment allowlist和artifact schema。
2. 从一个typed build/profile manifest生成CI matrix、validator command、MVP gate、dev aliases和文档。
3. package声明public/private、SDK compatibility、publish和metadata policy。
4. workspace统一lint、format、MSRV和critical dependency singleton政策。

### M2 · 把validator变成跨平台gate executor

1. gate以DAG声明依赖、timeout、owner、输入、输出、always-run诊断和cache key。
2. storage policy与Windows盘符adapter解耦；Linux/macOS/容器使用同一receipt schema。
3. 共享read-only dependency cache与lane-local build state分离，保留Cargo config/credential projection。
4. 生成source-bound log、JUnit、artifact、symbol和manifest，所有consumer按Build Set ID读取。

### M3 · 补齐快速CI质量面

1. Linux/Windows/macOS compile/link/test分层矩阵。
2. workspace fmt/clippy/doc、MSRV、dependency policy、lock freshness和工具测试发现。
3. Windows实际运行Pester；Python/PowerShell测试输出JUnit并检测orphan suite。
4. 高风险FFI/task/plugin loader引入Miri/sanitizer小套件。
5. action SHA pin、最小permissions、timeout、concurrency和reusable dependency action。

### M4 · 建立真实平台与release链

1. 将当前export policy matrix改名，避免false-green。
2. 在真实或受支持交叉环境执行target install、compile、link、package和artifact inspection。
3. PC平台执行install/launch/shutdown；移动/Web执行device/browser smoke和有界日志。
4. package、symbol、SBOM、license、signature和provenance由同一receipt绑定。
5. promotion只接受已验证immutable artifact，不在发布阶段重新构建。

### M5 · 性能与长期工程证据

1. 定义代表性Editor、runtime、server、render和asset workload。
2. 记录CPU/GPU frame、memory、load/cook、shader/PSO、binary size和compile-time baseline。
3. 采用warmup、重复次数、噪声区间、硬件标签和退化budget。
4. PGO/DDC/cache验证消费真实产品运行，不接受synthetic env-token测试。
5. dashboard只展示可追溯到Build Set、场景和原始结果的指标。

## 11. 验收门

以下全部满足前，本切片不得标记“工程化完成”：

1. root与plugin每个package只有一个canonical workspace owner。
2. 两个workspace若继续独立存在，其lock/profile/source revision由同一Build Set manifest绑定。
3. root和plugin `cargo metadata --locked`在CI首阶段通过，lock漂移在60秒内失败。
4. 仓库固定开发/CI toolchain并声明MSRV；没有隐式`stable/nightly`漂移。
5. Linux、Windows、macOS均完成受支持workspace compile/link/test层。
6. 所有宣称支持的平台至少有真实target compile与artifact inspection，不用环境变量policy test冒充export。
7. 六个Runtime profile由单一schema生成命令，并验证composition、启动、停机和artifact inventory。
8. Workspace fmt/clippy/doc覆盖全部canonical package，warning和例外政策可审计。
9. Public SDK有API compatibility和文档门，internal crate显式不可发布。
10. Critical dependency singleton列表有owner、理由和收敛预算。
11. 349个Python模块与35个PowerShell测试均被发现、分片或明确quarantine；新增orphan test会失败。
12. Validator跨平台消费同一gate graph，不把D/E/F盘符作为公共架构。
13. 每个gate有timeout、取消传播、精确argv、structured result和bounded live log。
14. Low-disk清理有容量复检、LRU/lease政策和receipt，不无条件摧毁整个target。
15. Build artifact携带source、lock、toolchain、target、profile、feature、command、symbol、SBOM和hash。
16. Test selector验证exact count；零测试和多测试不能被当作指定合同通过。
17. Domain dependency gate从canonical module graph生成，并对非法方向返回非零。
18. Convention强规则链接真实gate ID；YAML frontmatter使用schema parser。
19. 所有提交的dev wrapper在干净checkout上通过process-level smoke，README示例可执行。
20. CI使用SHA-pinned action、最小permissions、credential isolation、concurrency和job timeout。
21. Release artifact经过install/launch/shutdown或目标等价device smoke，promotion不重建。
22. 性能门具有固定workload、硬件标签、统计噪声模型和Build Set追溯。

## 12. 后续审查边界

下一批`zircon_tooling`报告按所有权拆分：

1. export、`cargo-zircon`、平台打包、receipt、install/run与release promotion；
2. `zircon-engine-derive`、代码生成、schema/version、增量生成与诊断；
3. Session Coordinator、Build/Editor工具应用、并发lease与跨进程取消；
4. benchmark/profile/capture、DDC/cache、symbol/crash和长期性能基线；
5. 其余349个Python/35个PowerShell测试的分区抽样与测试架构。

本报告不把未进入上述后续纵向审查的工具代码视为已完成，也不以当前P0数量推断整个tooling只有一个阻断。
