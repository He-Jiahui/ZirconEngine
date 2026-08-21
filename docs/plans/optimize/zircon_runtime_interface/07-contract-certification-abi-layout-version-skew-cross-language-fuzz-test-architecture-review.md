---
related_code:
  - zircon_runtime_interface/Cargo.toml
  - zircon_runtime_interface/src/lib.rs
  - zircon_runtime_interface/src/runtime_api
  - zircon_runtime_interface/src/plugin_api.rs
  - zircon_runtime_interface/src/buffer.rs
  - zircon_runtime_interface/src/status.rs
  - zircon_runtime_interface/src/version.rs
tests:
  - zircon_runtime_interface/src/tests
  - zircon_runtime_interface/tests/ui_binding_control_prop_ref.rs
  - zircon_runtime_interface/src/serialization/tests
  - zircon_runtime_interface/src/export/tests.rs
  - zircon_runtime_interface/src/hub_protocol/tests.rs
  - zircon_runtime_interface/src/project/tests
  - zircon_runtime_interface/src/project/session_lock/tests.rs
  - zircon_runtime_interface/src/ui
  - zircon_runtime/src/dynamic_api/tests/api_table.rs
  - zircon_runtime/src/dynamic_api/tests/session_entry_points.rs
  - zircon_runtime/src/dynamic_api/tests/session_lifecycle.rs
  - zircon_runtime/tests/runtime_owned_result_v7.rs
  - zircon_runtime_host/src/foreign_output/tests.rs
  - zircon_app/src/entry/runtime_library/tests.rs
  - zircon_app/src/entry/runtime_library/runtime_session/tests.rs
  - zircon_editor/src/core/gateway/session/tests.rs
  - zircon_editor/src/tests/runtime_event_consumer_bounded_pump/real_runtime_abi.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md
  - docs/plans/optimize/zircon_runtime_interface/02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md
  - docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
  - docs/plans/optimize/zircon_runtime_interface/04-profiling-plugin-event-script-diagnostic-manifest-crate-ownership-consolidation-review.md
  - docs/plans/optimize/zircon_runtime_interface/05-runtime-host-foreign-output-safe-api-ownership-admission-budget-fuse-observability-review.md
  - docs/plans/optimize/zircon_runtime_interface/06-project-manifest-session-lock-hub-protocol-recent-project-cross-process-contract-product-integration-review.md
  - docs/plans/optimize/zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/AutomationTest.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/BuildVersion.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Modules/BuildVersion.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleManifest.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Modules/ModuleManifest.cpp
  - dev/godot/core/extension/gdextension_interface.json
  - dev/godot/core/extension/gdextension_interface.schema.json
  - dev/godot/core/extension/gdextension_interface_header_generator.cpp
  - dev/godot/core/extension/make_interface_header.py
  - dev/godot/tests/compatibility_test/run_compatibility_test.py
  - dev/godot/tests/compatibility_test/src/compat_checker.c
  - dev/godot/tests/compatibility_test/src/compat_checker.h
  - dev/bevy/tools/ci/src/commands/test_check.rs
  - dev/bevy/tools/ci/src/commands/compile_fail.rs
  - dev/bevy/crates/bevy_reflect/compile_fail/tests/derive.rs
  - dev/Fyrox/fyrox-impl/src/plugin/dylib.rs
  - dev/Graphics/.yamato/wrench/api-validation-jobs.yml
  - dev/Graphics/Packages/com.unity.shadergraph/Tests/Editor/IntegrationTests/SerializationTests.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 07 · Runtime Interface 契约认证、ABI 布局、版本偏差、跨语言与 Fuzz 测试工程化差距

## 1. 结论

`zircon_runtime_interface` 并非没有测试。本轮识别到 crate 内 401 个 `#[test]`，分布在 72 个含测试的 Rust 文件；其中中央 `src/tests` 为 35 个文件、13,563 行、234 项测试、0 ignored，其他 inline/domain/integration suite 为 167 项测试。现有正向基础包括 V7 table 的本机 offset/size 检查、null/misaligned table 拒绝、opaque allocation exactly-once release、同进程并发释放、unknown field 拒绝、serialization malformed/migration case、UI/input/render DTO 行为以及 contract-only dependency boundary。这些测试应保留。

但它们还不能认证一个可发布的工程级动态库和公共协议。最高风险是当前 required 测试没有构建并正向加载真实 `zircon_runtime.dll`，再由真实 App/Editor host 完成握手、调用、并发、销毁和卸载。Runtime 的“真实 ABI”测试直接调用同一 test process 中链接的 `zircon_runtime_get_api_v7`；App 大量测试使用手工函数表；常规动态加载测试只拿系统库验证“缺少符号”错误。唯一 `LoadedRuntime::load_default()` 正向产品测试在缺少特定 feature 时被 ignore，而且没有形成 BuildSet、target、symbol、test selection 和 artifact digest 绑定的发布资格记录。Rust 编译器在同一编译单元内看到 producer/consumer 的同一个类型定义，无法暴露真正 DLL 边界上的旧头文件、不同编译器、错误 calling convention、导出可见性、卸载后回调或构建偏差问题。

ABI guard 本身也主要是源码文本检查。`abi_safety_contracts.rs` 用 `find`、`lines` 和字符串前缀识别 `repr(C)`、字段数、禁止 token 和 retired symbol；它不解析 Rust AST，不比较字段类型/offset/alignment，不检查全部公开 FFI record，也不验证生成的 C/C++ header。Plugin V3/V4 的若干 size 常量只证明当前 64-bit Rust target 的结果；32-bit、不同 ABI/compiler/packing 或大端目标没有资格矩阵。仓库没有 InterfaceSpec/IDL、ABI manifest、symbol map、layout snapshot、C consumer、compile-fail SDK suite或 ABI diff 工具。

JSON/DTO 测试的主要模式同样偏向“当前 writer -> 当前 reader”。中央测试中出现 213 次 `round_trip` 调用/定义和 233 次 `serde_json` 使用，但没有按已发布版本冻结的 golden corpus、N/N-1 reader/writer artifact、另一语言 consumer 或全 DTO schema catalog。当前同版本 round-trip 可以在 writer 与 reader 同时错误改名、删字段或改变默认值时继续通过。现有 serialization 子域的 malformed/migration 测试明显强于其他 DTO 家族，应成为全 Interface 的标准，而不是局部例外。

本轮新增 1 项 P0、48 项 P1、12 项 P2。它不重复 Interface01 的 ABI 实现缺陷、Interface02/03/04/06 的具体 schema finding，也不接管 Tooling10 的全仓 test scheduler/result control plane；本文只拥有“Runtime Interface 的公开承诺需要什么证据才可被认证”。在本文 32 项资格门通过前，401 项 crate 测试或一次 `cargo test` 绿色都不能支持“稳定 ABI”“跨版本兼容”“跨语言 SDK 可用”或“性能优于 Unreal”的产品声明。

## 2. 审查边界与证据

### 2.1 物理范围

| 集合 | 文件 / 行 / bytes | 测试属性 | 证据等级 |
|---|---:|---:|---|
| Interface 中央契约测试 | 35 / 13,563 / 499,228 | 234 test / 0 ignored / 2 should-panic | E3：逐文件 test inventory、helper/断言/源码形状/布局/serde模式审读 |
| Interface 全部含测试源码 | 72 / 21,831 / 766,910 | 401 test / 1 ignored | E3：中央、serialization、project、Hub、export、profiling、UI inline/domain及package-root integration suite |
| 选定 producer/host/consumer test | 9 / 3,737 / 132,663 | 95 test / 1 ignored | E3：Runtime linked API、Host foreign output、App loader/session、Editor gateway/real-ABI |
| 本轮 Zircon 冻结集合 | 81 / 25,568 / 899,573 | 496 test / 2 ignored | fingerprint `eea8fdb2c7e7d9042f381d8c8995f6d53675503137b6fc7f9937b9bc3a093f1e` |
| 参考引擎集合 | 18 / 17,485 / 613,618 | 不混算测试数量 | fingerprint `5028e6d929c5a0199c22039386b6cda8458e66be18488d087ecce14b5d2d33e5` |

两组指纹都按去重后的workspace相对路径进行ordinal排序，对每个文件计算lowercase SHA-256，再以 `path<TAB>hash`、LF连接且无末尾LF后计算总SHA-256。Zircon集合只包含可由本表和frontmatter确定性重建的72个Interface测试文件与9个跨crate生产者/消费者测试文件；结构审计脚本和CI配置不混入该冻结统计。冻结日期为2026-08-19，基线提交为`25e09a23178000f2e783ce2143cf70a8b118d404`。选定的Editor real-runtime ABI文件当前有其他Session的未提交修改，因此本文保持`source_recheck_required: true`；指纹固定的是本轮观察点，不是clean build receipt。

### 2.2 测试形状

| 指标 | 当前观察 | 解释边界 |
|---|---:|---|
| Interface crate `#[test]` | 401 | 数量不等于协议族覆盖或 release qualification |
| 中央 `src/tests` | 234 test、2,191个 assert 宏命中 | 断言丰富，但多个文件承担过多协议族 |
| 中央 same-version round-trip 命中 | 213 | 可证明当前 reader/writer 自洽，不能证明历史兼容 |
| 中央 `serde_json` 命中 | 233 | JSON shape 被大量触达，但没有统一版本 corpus |
| 中央 `.contains()` / source read | 104 / 18 | architecture guard 有价值，但大量依赖源码文本 |
| 中央 layout primitive 命中 | 117 | 只在当前 target/compiler 运行，未形成跨 target manifest |
| property/fuzz framework | 0 proptest，0 fuzz target | parser、wire、FFI carrier没有生成式输入覆盖 |
| 全 crate target-specific test cfg | 0 | 没有显式 32/64-bit、endianness、OS ABI矩阵 |
| package-root integration test | 1文件 / 3项 | 只验证一个 UI binding parser 的公开 Rust API；没有覆盖 ABI/header/SDK/历史版本 consumer |

`contracts.rs` 为 2,865 行、47项测试，`render_contracts.rs` 为 1,831 行、36项测试；`window_input_contracts.rs` 虽只含2个顶层 test，却有903行并在单项中枚举大量输入家族。测试体量本身不是问题，问题是协议声明、兼容样本、行为 oracle、source-shape guard和大规模 fixture没有被分层，失败时难以判断是 ABI、wire、semantic、architecture还是 performance gate。

### 2.3 动态证据边界

本轮是 review-only，没有修改 production/test code，也没有运行 Cargo、编译 C/C++ SDK、构建真实 DLL、运行 Miri/sanitizer/fuzz、加载旧构建或执行性能基准。`audit_runtime_structure.py --json` 在 184.1 秒后超时且没有产生结果，因此不作为证据。此前 Editor lib lane受大量既有编译错误阻断；本轮不重复该高成本失败路径。

静态证据足以确认测试调用的是链接符号、fake table或系统库，确认 crate 只有一个聚焦 UI binding parser 的 package-root integration 文件且没有fuzz target、C header和dev test framework，也足以识别字符串 parser 的能力边界；它不能证明未来 harness 的跨进程故障行为、平台 ABI 或性能已经合格。

### 2.4 参考边界

- Unreal 的 `FBuildVersion`/`FModuleManifest`把 `BuildId`、changelist、compatible changelist与module path装入构建/模块身份；Automation Test又要求 application context、filter和priority。应学习“构建集合身份 + 测试分类”，不能把 Unreal 内部 C++ module机制误写成通用稳定 C ABI。
- Godot 的 GDExtension interface由 JSON/schema驱动生成 header，并用一个真实 C extension against多个历史 tag的API hash做兼容测试。它直接证明旧 consumer 能否向新 engine解析接口，比在同一 C++/Rust源码树内检查字段数更接近 Zircon 的目标。
- Bevy明确把 compile-fail suite作为 workspace普通测试之外的 CI command，并为 reflect derive保留 compiler diagnostic fixtures。Bevy dylib不承诺公开稳定ABI；本文只借鉴“负向编译合同独立成资格 lane”。
- Fyrox `dylib.rs`直接加载 Rust trait-object plugin并实现文件复制/热重载，它适合观察同工具链编辑器热重载流程，不适合作为跨语言稳定 ABI 标杆。Zircon不能因为 Fyrox也使用 Rust dylib就省略 C ABI manifest和构建偏差认证。
- Unity Graphics的API validation job先依赖package pack，再在指定Editor/OS中运行vetting并上传XML/log/crash artifact；ShaderGraph serialization test覆盖polymorphic/legacy remapping。应学习package artifact与目标版本绑定，但不能照搬其当前 job里由后续parser解释的退出码处理。

## 3. 当前 P0

### RI-CERT-P0-001 · 没有真实 Runtime DLL 正向发布资格门，unsafe 边界可在同进程测试全绿时失效

Runtime侧测试直接调用链接进test binary的`zircon_runtime_get_api_v7`，Editor所谓real-runtime ABI也使用同一路径；App loader测试主要构造`ZrRuntimeApiV7`或加载系统库验证missing symbol。`LoadedRuntime::load_default()`产品测试在缺少backend feature时被ignore，且没有固定构建出的DLL、Host二进制、BuildSet、target、feature、symbol/layout manifest、进程边界、unload/reload和artifact digest。

必须建立独立 qualification：先从同一 BuildSet 产出 App/Editor host与`zircon_runtime` cdylib，验证导出symbol和manifest，再在child process中完成握手、每个required slot的最小调用、foreign output release、并发close、callback quiescence、destroy与DLL unload；随后用mismatched/old/corrupt fixtures验证fail-closed。该lane必须是 release candidate 的required gate，不能依赖开发者环境变量临时启用。

## 4. P1 差距

### 4.1 InterfaceSpec、ABI Manifest 与静态布局

#### RI-CERT-P1-001 · 没有 canonical InterfaceSpec/IDL

Rust声明、手工re-export、文档和测试各自维护接口。建立versioned InterfaceSpec，生成Rust carrier、C header、schema、symbol/layout manifest、SDK docs和测试inventory。

#### RI-CERT-P1-002 · `repr(C)` guard是字符串 parser

`find("pub struct")`与最近`#[repr(C)]`不能理解cfg、宏、泛型、visibility或attribute组合。改由`syn`/rustdoc JSON或IDL AST验证。

#### RI-CERT-P1-003 · `repr(C)` inventory只覆盖选定 table

当前名单没有系统枚举全部跨FFI record/enum/callback carrier。由InterfaceSpec生成全量 ABI surface，漏登记公开 carrier 必须失败。

#### RI-CERT-P1-004 · field-count guard不检查字段类型

字段数不变但`usize -> u64`、pointer mutability、callback signature或semantic改变仍可通过。manifest必须记录type、offset、size、align、nullability、ownership和version。

#### RI-CERT-P1-005 · forbidden-token blacklist不能证明公开签名安全

只匹配`Box<dyn`、`Rc<`等少数文本，可被type alias、多行签名、generic wrapper或Rust-layout type绕过。使用编译器可见public item graph和FFI-safe type closure。

#### RI-CERT-P1-006 · 没有可审计 ABI diff artifact

PR无法看到symbol/record/callback/constant的结构化差异。每次构建输出旧新manifest diff并按compatible/breaking分类。

#### RI-CERT-P1-007 · 没有导出 symbol map 与 visibility gate

源码出现entry name不等于最终PE/ELF/Mach-O导出正确。对构建artifact读取export table并与manifest精确比较。

#### RI-CERT-P1-008 · 没有生成式 C header

公开“C ABI”目前没有可发布、版本化、带packing/static assert的C头文件。由同一spec生成，不允许手抄第二份。

#### RI-CERT-P1-009 · 没有 C/C++ consumer compile-link-run

Rust自身layout test无法证明C声明、calling convention和ownership文档可用。至少用MSVC/clang/gcc编译最小consumer并真实调用DLL。

#### RI-CERT-P1-010 · layout常量只证明当前64-bit target

Plugin V3/V4测试写死88/104/128等大小；它们没有target限定，也没有32-bit期望。明确supported target集合并为每个target生成layout snapshot。

#### RI-CERT-P1-011 · 没有 compiler/linker/CRT 矩阵

MSVC/GNU/Clang、Rust版本、debug/release、LTO与CRT边界未被认证。BuildSet必须记录toolchain，qualification覆盖声明支持的组合。

#### RI-CERT-P1-012 · packing、alignment与endianness政策未成为manifest

`repr(C)`不是完整跨平台政策。定义natural alignment、禁止pragma pack、endianness、pointer width与scalar representation，并在consumer static assert中验证。

#### RI-CERT-P1-013 · padding与初始化字节没有检查

按值跨边界的record可能携带未初始化padding，破坏hash/trace/IPC或触发工具告警。定义逐字段copy/zero-init策略，禁止对raw struct bytes作协议身份。

#### RI-CERT-P1-014 · enum/constant/raw code没有统一生成清单

当前各测试零散验证部分raw值。生成完整value registry，新增、复用、删除或unknown policy变化均产生diff。

#### RI-CERT-P1-015 · API slot coverage没有单一 registry

Interface、Runtime producer、Host policy、App/Editor consumer分别维护列表。生成每个slot的owner、required/optional、input/output budget、release、fault与test mapping。

#### RI-CERT-P1-016 · source/doc phrase guard替代了可执行协议不变量

测试要求文档包含特定句子或源码包含特定wrapper，只能守住拼写。文档currentness可保留为architecture lane，ABI correctness必须由artifact/consumer行为证明。

### 4.2 DLL、进程、生命周期与故障

#### RI-CERT-P1-017 · 正向动态加载测试默认不可达

唯一产品正向`load_default`场景依赖feature和环境，默认可能ignore。将最小headless DLL smoke从示例游戏/VM backend解耦，成为所有supported target必跑lane。

#### RI-CERT-P1-018 · missing-symbol测试复用系统库而非受控fixture

`kernel32.dll`/`libSystem`只能证明没有Zircon symbol，不能测试错误导出、错误版本、截断table或恶意entry。构建versioned fixture DLL family。

#### RI-CERT-P1-019 · 没有真实 unload/reload 资格

没有证明所有session、allocation、callback、thread和function pointer在unload前quiescent。用独立进程循环load/use/destroy/unload/reload并检查owner census归零。

#### RI-CERT-P1-020 · 没有 stale handle/callback after unload 测试

旧generation句柄、wake callback或releaser指针在新DLL加载后可能命中新对象或悬空。测试epoch-qualified rejection与no-callback-after-unload。

#### RI-CERT-P1-021 · 没有 Host/Runtime BuildSet skew 矩阵

当前只传API/ABI数字。构造same version/different build、feature、target、schema fingerprint组合，要求握手在调用任何slot前fail-closed。

#### RI-CERT-P1-022 · 没有历史 table binary fixture

source里删除旧类型不等于旧host行为明确。保存已发布header/manifest/fixture binary，按支持政策验证accept或明确诊断reject。

#### RI-CERT-P1-023 · panic boundary主要由源码包含测试证明

检查wrapper文本不能证明每个slot在优化构建下实际捕获panic并返回正确状态。用注入panic的fixture逐slot执行。

#### RI-CERT-P1-024 · 没有 process-isolated abort/SEH/signal 测试

访问冲突、stack overflow、abort和foreign exception不能在普通unit test安全恢复。child process必须报告classified termination、dump和cleanup outcome。

#### RI-CERT-P1-025 · 没有 guard-page/bad-pointer adversarial harness

null/misaligned检查有基础，但悬空、跨页、错误len、只读output、fake callback address需要guard page与子进程，不得在主test process制造UB。

#### RI-CERT-P1-026 · 没有 Miri/ASan/UBSan lane

unsafe carrier、raw pointer、allocation registry与callback函数指针缺少动态内存/未定义行为工具证据。按工具适用范围拆lane，不要求一个工具覆盖DLL全部行为。

#### RI-CERT-P1-027 · 没有 callback reentrancy 矩阵

callback内destroy、unsubscribe、release、nested call与panic的允许/拒绝语义没有逐slot验证。生成reentrancy matrix并检查无死锁、无双重终结。

#### RI-CERT-P1-028 · 并发测试没有系统调度模型

少量双线程release是好基础，但未覆盖call/close/fuse/unload/callback交错。对小状态机使用loom/model test，对真实DLL使用stress与deterministic barriers。

#### RI-CERT-P1-029 · wall-clock性能断言混在普通正确性测试

`runtime_owned_result_v7`和foreign-output suite在共享机器上断言p99/throughput，易flake且没有硬件/频率/build artifact身份。移到受控performance qualification，unit test只验证计数与算法边界。

#### RI-CERT-P1-030 · DLL测试结果没有 artifact currentness receipt

日志不能证明测试的是哪一个DLL。结果必须携带host/runtime digest、symbol/layout manifest、BuildSet、command、target、toolchain与selection digest，并交给Tooling10聚合。

### 4.3 Schema、版本与生成式输入

#### RI-CERT-P1-031 · same-version round-trip占主导

当前writer和reader一起变化仍可绿色。保留round-trip作为smoke，同时增加固定bytes/JSON与独立旧reader/newwriter矩阵。

#### RI-CERT-P1-032 · 没有全 Interface golden corpus

为每个public wire family保存canonical valid/minimal/maximal/legacy/future/malformed fixture和manifest；fixture必须绑定schema/version与producer digest。

#### RI-CERT-P1-033 · 没有 N/N-1 reader-writer artifact 矩阵

同一source tree无法证明升级、降级或rolling skew。保存旧版reader/writer可执行artifact，按每个schema的support window运行双向矩阵。

#### RI-CERT-P1-034 · 没有机器可枚举 Schema Catalog

大量serde DTO没有统一SchemaId、owner、codec、unknown policy、limit、current version与migration chain。由catalog生成test cases并拒绝未登记wire type。

#### RI-CERT-P1-035 · default/unknown-field策略依赖零散手写测试

部分DTO deny unknown，部分为旧payload default，缺少系统规则。catalog按closed/open/extension-map分类并自动生成missing/unknown/duplicate field tests。

#### RI-CERT-P1-036 · duplicate key、Unicode与canonical parse未横向覆盖

project/serialization局部测试不能代表profile、UI、world、Hub和plugin event。统一adversarial corpus覆盖duplicate key、escape、normalization、surrogate、NaN/Inf与number边界。

#### RI-CERT-P1-037 · payload预算没有逐schema边界测试

接口常量与Host policy存在，但没有生成每个input/output在limit-1/limit/limit+1、item/depth/string组合的producer+consumer纵向测试。

#### RI-CERT-P1-038 · 没有 property/fuzz target

为binary/text serialization、project/Hub parser、UI/input/world/profile JSON、FFI shape validator建立seeded property与coverage-guided fuzz；crash corpus进入版本化回归集。

#### RI-CERT-P1-039 · 没有另一语言 JSON/DTO consumer

Rust serde约定可能与C++/C#/TypeScript实现分歧。对公开跨进程格式至少选择一个独立consumer验证canonical fixtures与unknown policy。

#### RI-CERT-P1-040 · binary corpus没有跨arch/endianness资格

serialization已有 malformed 深度，但没有来自不同target的历史artifact。明确binary format是否target-neutral；若是则跨arch比较，若否则在header中拒绝错误target。

#### RI-CERT-P1-041 · migration test不绑定发布版本currentness

迁移step在当前源码内可运行，不证明历史fixture未被改写。corpus manifest记录immutable digest，删除support window必须有显式breaking release决策。

#### RI-CERT-P1-042 · DTO测试很少经过真实 producer/consumer semantic oracle

许多测试只构造公共类型调用helper。关键协议必须由Runtime producer产生payload，经真实Host预算/解码，再由App/Editor消费并验证产品状态或receipt。

### 4.4 Coverage、可维护性与资格声明

#### RI-CERT-P1-043 · package-root integration没有承担公开契约认证

现有package-root integration只覆盖一个UI binding parser；其余crate内unit test可访问私有模块并共享编译上下文。增加独立integration/SDK crate，只通过发布的public API/header构建。

#### RI-CERT-P1-044 · public item到测试/owner没有覆盖映射

401项测试无法回答某个新增DTO、enum、slot或re-export是否被认证。生成PublicContractId -> schema/layout/behavior/skew/fault test映射，缺required class即失败。

#### RI-CERT-P1-045 · 中央测试文件形成新的复合owner

`contracts.rs`和`render_contracts.rs`混合多个协议代际与行为。按public contract family拆分，fixture与oracle放共享support，禁止再建立“所有合同”文件。

#### RI-CERT-P1-046 · round-trip/fixture helper大量重复

重复小helper容易让各文件采用不同serde配置、限制与诊断。建立显式`WireHarness`，要求调用者声明schema、version、policy和预算。

#### RI-CERT-P1-047 · source-shape、semantic、performance测试混在同一Cargo lane

三类失败的owner、运行时长和环境不同。按architecture、schema/unit、ABI artifact、fault、performance拆suite，并由Tooling10统一计划与结果。

#### RI-CERT-P1-048 · ignored/manual资格没有required manifest

real-runtime ABI benchmark的ignore理由说明需managed独跑，但没有机器声明何时required、谁运行、结果何时过期。登记capability、runner、timeout、artifact与release criticality；遗漏不得静默绿色。

## 5. P2 差距

### RI-CERT-P2-001 · `runtime_10`命名与V7 public table混用

测试错误信息引用计划代号而非稳定ContractId，后续版本难以检索。诊断使用`runtime_api/v7`和manifest revision。

### RI-CERT-P2-002 · `abi_v3_layout_is_stable`只是调用其他测试函数

它不增加独立覆盖，还使test count显得更高。改成参数化layout case或删除包装计数。

### RI-CERT-P2-003 · 700行module budget是未命名政策

行数可作维护提醒，但不是ABI安全。把阈值放结构治理manifest并允许带owner/reason/expiry的waiver。

### RI-CERT-P2-004 · blacklist可被别名与格式变化绕过

`std::fs`、`Box<dyn`等文本守卫应降级为快速lint，权威结果来自依赖图/AST。

### RI-CERT-P2-005 · 部分wire tag只用`.contains()`确认

字符串出现不证明字段位置、唯一性或canonical shape。优先比较完整结构化value或golden bytes。

### RI-CERT-P2-006 · literal layout size没有target标签

失败只显示数字，不显示target triple、pointer width、compiler和manifest revision。所有layout diagnostic补齐上下文。

### RI-CERT-P2-007 · 两项`should_panic`承担legacy builder拒绝语义

公共协议构造失败优先返回typed error；若panic是内部不变量，测试名和文档应明确不属于外部输入路径。

### RI-CERT-P2-008 · “stable”测试名强于实际证据

本机size或same-version serde不能称跨版本stable。改名为current-layout/current-roundtrip，直到qualification通过。

### RI-CERT-P2-009 · public API缺少可编译SDK示例

增加最小Rust host、C consumer、plugin callback和JSON consumer example，并作为doc/compile test运行。

### RI-CERT-P2-010 · 164项inline/domain测试散落在36个非中央文件

允许贴近实现，但catalog必须发现并分类，避免只运行`src/tests`就误报完整。

### RI-CERT-P2-011 · dependency allowlist没有结构化waiver元数据

新增compile-fail/property工具会触发手工常量修改。保留严格边界，但以owner/reason/scope/expiry记录dev-only依赖审批。

### RI-CERT-P2-012 · ignored real-ABI用例同时承担correctness与benchmark

拆成快速bounded correctness gate和受控规模performance gate；前者不应因性能环境缺失而整体跳过。

## 6. 目标认证架构

### 6.1 单一契约来源

建立 `InterfaceSpec`，至少生成以下不可独立手改的产物：

1. Rust producer/consumer carrier与callback signatures；
2. C/C++ header、static asserts与SDK examples；
3. ABI manifest：symbol、record、field、offset、size、align、enum、ownership、thread/reentrancy；
4. Schema Catalog、JSON schema/canonical corpus manifest与migration matrix；
5. slot policy：required/optional、capability、budget、output owner、fault和test classes；
6. compatibility diff和release note input。

生成器不能掩盖架构决策。`usize`、path/string identity、ownership、unknown-field与version policy必须先由Interface owner明确，再编码进spec。

### 6.2 五层资格模型

| 层 | 目的 | 最小证据 |
|---|---|---|
| C1 · Unit/Schema | 当前实现语义、canonical codec、负向输入 | deterministic unit、golden corpus、property/fuzz regression |
| C2 · Static ABI/SDK | layout、symbol、header、compile contract | per-target manifest、ABI diff、C/C++ compile-link、compile-fail |
| C3 · Real DLL | 真实loader、调用、ownership、shutdown | child-process built DLL smoke、all-slot coverage、unload census |
| C4 · Version Skew | BuildSet/API/schema支持窗口 | N/N-1 host/runtime/plugin/reader/writer matrix与typed rejection |
| C5 · Fault/Scale | UB、crash、race、budget与性能 | guard page、Miri/sanitizer、model/stress、fuzz、受控perf artifact |

Tooling10继续拥有TestPlan/TestAttempt/TestResult/CI聚合；Interface owner拥有每个PublicContractId需要哪些C1-C5 case以及pass criteria。Release tooling只消费绑定同一BuildSet和artifact digest的资格结果。

### 6.3 真实 DLL fixture family

固定一组只用于测试的动态库artifact：valid current、missing symbol、wrong version、truncated table、oversized same-version、wrong BuildSet、panic callback、hang callback、bad output shape、leaked allocation、late callback、old supported和old retired。所有fixture由source生成并带manifest/digest，禁止继续借系统库模拟除“任意非Zircon库”之外的协议错误。

## 7. 重构顺序

### M0 · Truth Freeze 与 Coverage Catalog

- 为全部public contract分配ContractId并生成current surface inventory；
- 把401项Interface测试和关键producer/consumer测试映射到contract/test class；
- 将现有测试声明降级为它实际能证明的current-layout/current-roundtrip；
- 建立真实DLL资格lane的required计划和artifact receipt schema。

### M1 · InterfaceSpec 与生成式 ABI

- 先覆盖V7 runtime table、V4 plugin host和foreign output carrier；
- 生成Rust/C header、symbol/layout manifest和per-target static assert；
- PR运行ABI diff，breaking change必须创建新version或显式hard-cut release。

### M2 · Real DLL 与 Cross-Language Qualification

- 构建fixture DLL family和headless external host；
- 在Windows MSVC首个产品target闭合load/call/release/destroy/unload；
- 增加C/C++ consumer、symbol map和BuildSet mismatch；
- 再扩展Linux/macOS与声明支持的compiler/arch。

### M3 · Schema Corpus 与 Version Matrix

- 将serialization现有malformed/migration做法推广到所有wire family；
- 建立immutable golden corpus与N/N-1 reader/writer artifact；
- 生成unknown/default/limit cases并加入独立语言consumer。

### M4 · Fault、Concurrency 与 Fuzz

- child process/guard page隔离bad pointer、abort和hang；
- Miri/sanitizer覆盖可适用的owner/carrier代码；
- model/stress覆盖call-close-callback-unload；
- fuzz corpus进入版本化回归，不以单次无crash代替预算/semantic oracle。

### M5 · Release Admission 与性能证据

- C1-C5结果绑定BuildSet、host/DLL/SDK/corpus digest；
- required case缺失、ignored、零发现、超时、artifact过期均阻断candidate；
- 性能在固定硬件/场景/统计协议下与Unreal等对照，普通unit test墙钟不作为“更快”证据。

## 8. 资格门

| Gate | 必须满足 |
|---|---|
| G01 | 全部public FFI carrier、callback、symbol、enum和constant进入InterfaceSpec/ABI manifest |
| G02 | Rust declaration、C header、SDK docs和layout tests由同一spec生成且无手工漂移 |
| G03 | Windows x64 MSVC Rust/C/C++ layout与static assert一致 |
| G04 | 每个声明支持的target都有独立layout/symbol manifest和artifact digest |
| G05 | ABI diff能区分compatible、new-version、hard-cut与illegal in-place change |
| G06 | 构建出的真实runtime DLL由external host正向加载并完成握手 |
| G07 | V7每个required slot至少有真实DLL smoke、fault和owner/release mapping |
| G08 | missing/wrong/truncated/oversized/BuildSet mismatch fixture均在首调用前fail-closed |
| G09 | session、allocation、callback、thread census在DLL unload前归零 |
| G10 | unload/reload后旧handle、callback、releaser和table pointer全部被拒绝或不可达 |
| G11 | panic、abort、access violation、hang在child process中得到classified result和artifact |
| G12 | bad pointer/len/alignment/readonly/guard-page case不在主runner制造未隔离UB |
| G13 | Miri/ASan/UBSan各自适用范围通过且skip有typed reason |
| G14 | callback reentrancy matrix对每种允许/拒绝行为有无死锁证据 |
| G15 | call/close/fuse/callback/unload model与stress gate通过 |
| G16 | 每个wire family登记SchemaId、owner、codec、version、unknown policy和budget |
| G17 | valid/minimal/maximal/legacy/future/malformed golden corpus完整且digest不可变 |
| G18 | N/N-1 reader/writer/host/runtime/plugin矩阵按声明支持窗口通过 |
| G19 | 同版本writer/reader同时变化不能在golden diff缺失时绿色 |
| G20 | duplicate key、Unicode、depth、items、bytes、NaN/Inf和numeric edge统一覆盖 |
| G21 | limit-1/limit/limit+1从producer到Host consumer的纵向预算测试通过 |
| G22 | fuzz/property覆盖所有高风险parser/carrier，crash corpus自动回归 |
| G23 | 至少一个独立非Rust JSON consumer通过canonical corpus |
| G24 | public ContractId到layout/schema/behavior/skew/fault测试映射无缺口 |
| G25 | architecture/source-shape、unit/schema、DLL、fault和performance suite分离 |
| G26 | ignored/manual test有required条件、owner、timeout、artifact和expiry，遗漏不静默绿色 |
| G27 | test discovery记录401项当前crate基线并能解释后续增删 |
| G28 | 所有资格结果绑定source、BuildSet、target、toolchain、host/DLL/SDK/corpus digest |
| G29 | performance结果绑定硬件、频率策略、workload、warmup、samples和统计置信信息 |
| G30 | 同场景与Unreal对照前先证明功能、画质、输入、失败条件和采样协议等价 |
| G31 | Tooling10聚合器拒绝partial、zero-test、timeout、stale artifact和跨BuildSet结果 |
| G32 | Interface01-06相关实现finding未通过其自身门时，本文测试不得将其标为fixed |

## 9. Owner 与排除边界

1. Interface01继续拥有DLL ABI、version、handle、foreign ownership的生产合同；本文拥有这些合同的认证方法和release evidence。
2. Interface02/03/04/06继续拥有serialization/resource/UI/profiling/plugin/project具体schema缺陷；本文只要求它们进入统一catalog/corpus/skew/fuzz矩阵。
3. Interface05继续拥有`zircon_runtime_host` safe abstraction、admission、budget和fuse；本文拥有真实DLL/guard-page/sanitizer/coverage gate。
4. Plugins01继续拥有native plugin SDK/loader产品生态；本文只拥有由同一InterfaceSpec生成和验证的ABI/SDK证据。
5. App/Editor/Runtime各自拥有真实producer/consumer行为；不得在Interface unit test里复制fake实现后宣称纵向闭合。
6. Tooling10拥有全仓测试控制面、分类、runner、隔离、flake和结果聚合；本文输出ContractId所需test classes及pass criteria供其消费。
7. Tooling01/09拥有CI/release promotion；缺本文required qualification时必须阻断，但不由Interface自行发布artifact。

## 10. 当前状态

本报告为 `review_complete / implementation pending / source_recheck_required`。它新增 1项P0、48项P1、12项P2和32项资格门，没有修改生产或测试代码，没有声称Cargo、DLL、跨语言、fuzz、sanitizer、性能或Unreal对照已经通过。下一步实施必须从M0 truth freeze和M1 InterfaceSpec开始，不能继续用更多same-version round-trip或source `.contains()`测试掩盖缺失的真实动态库与版本偏差认证。
