---
related_code:
  - examples/woc/tools/package.json
  - examples/woc/tools/current_head_parity_materialize.mjs
  - examples/woc/tools/reference_full_trace_probe.ts
  - examples/woc/tools/trace_symbol_codegen.mjs
  - examples/woc/tools/wtr1_encode.mjs
  - examples/woc/tools/wtr1_verify.mjs
  - examples/woc/reference/current-head/source_manifest.json
  - examples/woc/reference/current-head/parity_scenarios.json
  - examples/woc/reference/current-head/parity/scenarios.json
  - examples/woc/reference/current-head/parity/golden
  - examples/woc/reference/current-head/trace_symbols.json
  - examples/woc/native/crates/woc_parity/Cargo.toml
  - examples/woc/native/crates/woc_parity/src/lib.rs
  - examples/woc/native/crates/woc_parity/src/golden.rs
  - examples/woc/native/crates/woc_parity/src/rng.rs
  - examples/woc/native/crates/woc_parity/src/trace.rs
  - examples/woc/native/crates/woc_parity/src/wire.rs
  - examples/woc/native/crates/woc_parity/src/generated_trace_symbols.rs
  - examples/woc/scripts/woc_game/src/parity/wire.zr
  - examples/woc/scripts/woc_game/src/parity/m3_lifecycle_trace.zr
  - examples/woc/scripts/woc_game/src/parity/m3_locomotion_trace.zr
  - examples/woc/scripts/woc_game/src/parity/m3_roster_trace.zr
  - examples/woc/scripts/woc_game/src/parity/m3_targeting_trace.zr
  - examples/woc/scripts/woc_game/woc_lifecycle_trace_dump.zrp
  - examples/woc/scripts/woc_game/woc_lifecycle_trace_tests.zrp
  - examples/woc/scripts/woc_game/woc_locomotion_trace_dump.zrp
  - examples/woc/scripts/woc_game/woc_locomotion_trace_tests.zrp
  - examples/woc/scripts/woc_game/woc_roster_trace_dump.zrp
  - examples/woc/scripts/woc_game/woc_roster_trace_tests.zrp
  - examples/woc/scripts/woc_game/woc_targeting_trace_dump.zrp
  - examples/woc/scripts/woc_game/woc_targeting_trace_tests.zrp
tests:
  - examples/woc/native/crates/woc_parity/tests/goldens.rs
  - examples/woc/native/crates/woc_parity/tests/trace_vectors.rs
  - examples/woc/native/crates/woc_parity/tests/trace_wire.rs
  - examples/woc/scripts/woc_game/src/parity/wire_test_main.zr
  - examples/woc/scripts/woc_game/src/parity/m3_lifecycle_trace_test_main.zr
  - examples/woc/scripts/woc_game/src/parity/m3_locomotion_trace_test_main.zr
  - examples/woc/scripts/woc_game/src/parity/m3_roster_trace_test_main.zr
  - examples/woc/scripts/woc_game/src/parity/m3_targeting_trace_test_main.zr
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_app/03-woc-product-role-host-zrvm-transaction-state-client-server-integration-review.md
  - docs/plans/optimize/zircon_runtime/13-woc-combat-casting-effect-aura-damage-threat-death-runtime-review.md
  - docs/plans/optimize/zircon_runtime/17-woc-world-terrain-collision-locomotion-spawn-spatial-targeting-runtime-review.md
  - docs/plans/optimize/zircon_runtime/18-woc-generated-content-catalog-buildset-install-query-runtime-review.md
  - docs/plans/optimize/zircon_runtime/19-woc-command-protocol-payload-codec-admission-movement-outcome-runtime-review.md
  - docs/plans/optimize/zircon_tooling/05-woc-content-codegen-build-scripts-generated-artifact-incremental-review.md
  - docs/plans/optimize/zircon_tooling/07-performance-benchmark-profile-capture-symbol-crash-evidence-baseline-review.md
  - docs/plans/optimize/zircon_tooling/09-release-channel-artifact-repository-install-update-rollback-operations-review.md
  - docs/plans/optimize/zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/AutomationTest.h
  - dev/UnrealEngine/Engine/Source/Developer/AutomationController/Private/AutomationControllerManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/DemoNetDriver.h
  - dev/UnrealEngine/Engine/Source/Runtime/TraceLog/Public/Trace/Trace.h
  - dev/bevy/crates/bevy_app/src/schedule_runner.rs
  - dev/bevy/crates/bevy_ecs/src/schedule/stepping.rs
  - dev/godot/core/debugger/engine_profiler.h
  - dev/godot/core/debugger/engine_profiler.cpp
  - dev/Fyrox/fyrox-core/src/visitor/mod.rs
  - dev/Fyrox/fyrox-core/src/visitor/error.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Editor/RenderGraphTests.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Editor/NativePassCompilerRenderGraphTests.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 11 · WOC Oracle、Trace、Golden、Differential Replay 与 Evidence 工程化差距

## 1. 结论

WOC已经拥有一组有价值的参考资产，但当前还没有形成“参考实现执行一次、Zircon产品执行一次、两边对同一场景产生同一版本trace、差异可重放且结果进入发布准入”的工程级parity系统。当前最强的已实现能力是：固定commit的54份参考golden可以从本地`dev/world-of-claudecraft`仓库用`git show`重新物化并逐文件校验SHA-256；1,070项trace symbol可以从这些golden重新生成到JSON、Zr与Rust三端；native `woc_parity`有canonicalization、WTR1 decoder、JSON首差异与若干wire负向测试。这些应保留。

但“参考资产自洽”目前被当成了“Zircon行为一致”。native唯一的`compare_double_run`测试先读取expected JSON，再用`|| expected.clone()`执行两次，随后与同一个expected比较；全仓没有任何真实ZrVM或WOC产品runner调用`GoldenSuite`、`compare_double_run`或`decode_vm_trace`。因此该测试只能证明复制同一个JSON两次仍相等，不能证明Zircon执行过场景。

catalog更直接暴露了缺口。`reference/current-head/parity_scenarios.json`为54个场景逐项声明了唯一`woc_owner`，但54条路径全部位于不存在的`scripts/woc_game/tests/parity/*.zr`。物化到native消费的`parity/scenarios.json`时，`index`、`factory`、`ownership_class`和`woc_owner`又被主动丢弃，只剩name/source owner/golden/hash/coverage；native suite无法检查Zircon owner存在、可编译、可执行或覆盖完整。

现有四个Zr M3 trace也不是产品actual sampler。lifecycle、locomotion、roster三个文件分别含715、661、562行和1,756、1,546、1,039个数字字面量，却各自只调用一次对应kernel的`scenarioContractTest()`；其余逻辑直接手写golden字段、symbol ID、digest、frame与WTR1 bytes。targeting虽读取24次`scenarioMetric()`，仍由1,433个数字字面量拼出固定模板。四个dump binary目录存在，四个对应test binary目录全部不存在；四条trace覆盖的只是`mob_lifecycle`、`mob_locomotion`、`entity_roster`、`mob_targeting`，且物理owner与catalog声明不同，不能替代54条产品场景。

参考侧也有身份分叉。物化器从固定commit读取golden，`reference_full_trace_probe.ts`却直接import参考仓当前工作树。审查时参考仓HEAD虽然等于`5ef9f7cb21cd8875b6d2c49701015dfcd78de35a`，但工作树存在7个tracked修改和6个untracked文件；实际运行`entity_roster` probe立即因首帧ID、equipment、aura、stats、nextId与state/event digest漂移而失败。这个失败不是golden错误的充分证据，而是证明当前runner没有把“执行哪份源码”绑定到manifest宣称的commit。

因此当前所有WOC parity结论必须收缩为：54份参考golden与1,070项symbol在静态物化层自洽；native wire/canonical helper有局部单元合同；没有一条当前source/build-bound证据证明Zircon产品actual与固定参考oracle相等。`examples/woc/contracts/m4-combat.md`此前也明确记录reference-derived WTR1 fixture不改变real-M2的0/16 acceptance，本报告将这一边界提升为统一owner、schema和release gate。

本轮记录6个P0、68个P1和16个P2。未修改生产Rust、Zr、TypeScript/JavaScript、manifest、测试、golden、generated artifact或参考仓源码，只新增审查与索引。

## 2. 审查边界与证据

### 2.1 物理范围

| 子域 | 当前物理量 | 审查深度 |
|---|---:|---|
| native `woc_parity` production | 6个Rust文件 / 2,070行 / 66,792 bytes | E3逐文件，调用者与wire边界全仓反查 |
| native `woc_parity` tests | 3个文件 / 351行 / 11,311 bytes / 12个`#[test]` | E3逐项确认actual来源与负向合同 |
| Zr parity | 14个Zr文件 / 2,812行 / 110,327 bytes | E3逐文件；四条M3 trace的kernel调用与字面量统计 |
| Zr project/artifact | 8个dump/test manifest；4个dump binary目录；0个test binary目录 | E3 manifest、entry与物理artifact减法 |
| current-head scenario catalog | 54场景 / 54个唯一场景名 / 53个唯一source owner anchor / 54个唯一WOC owner | E3字段、路径、owner存在率与materialize投影 |
| reference golden | 54 JSON / 2,538,832 bytes / 104,070行 / 793 frame / 296 full frame | E2全量结构、尺寸、frame与coverage统计 |
| trace dictionary | 1,070个大小写敏感唯一symbol / 完整SHA-256 / 60-bit wire fingerprint | E3 codegen、三端输出与codec消费 |
| reference parity source | 8个source identity / 54个场景 / source commit与逐文件SHA-256 | E3固定commit、工作树执行与动态首差异 |
| 参考引擎 | Unreal、Bevy、Godot、Fyrox、Unity Graphics | E3责任对照，不把参考引擎局部能力外推成完整parity实现 |

### 2.2 可复核检查

| 检查 | 结果 | 能证明什么 |
|---|---|---|
| `node current_head_parity_materialize.mjs --check` | PASS，54 current-head parity goldens | 固定commit中的golden bytes与已物化目录一致 |
| `node trace_symbol_codegen.mjs --check` | PASS，1,070 symbols，fingerprint `77ea6e0966f861b` | 当前golden到JSON/Zr/Rust字典输出一致 |
| source owner commit存在性 | 54/54路径在固定commit中存在；都指向`tests/parity/scenarios.ts` | 参考scenario factory有真实source owner文件 |
| WOC owner存在性 | 0/54存在 | catalog宣称的Zircon场景实现尚未落地 |
| native consumer反查 | `GoldenSuite`、`compare_double_run`、`decode_vm_trace`只有crate自身tests消费 | 没有产品或VM actual runner接线 |
| Zr kernel调用统计 | lifecycle/locomotion/roster各1次contract；targeting 24次metric/contract | 四条trace大部分内容由常量模板产生 |
| 参考actual probe | FAIL，`entity_roster reference recorder drifted from golden` | probe执行工作树而非固定commit；当前身份不封闭 |
| WOC native workspace | 既有动态证据：132.6秒后6个`woc_protocol`编译错误，0 tests执行 | 当前native parity tests不构成可消费绿色证据 |
| WOC npm check | 既有动态证据：typed payload 157/expected 148，后续步骤短路 | aggregate check不会到达完整parity lane |

本轮没有重复native/npm已知阻断。参考probe只在系统临时目录请求输出，断言在写文件前失败，随后已删除创建的空临时目录；参考仓现有修改未被触碰。

### 2.3 正向基础

- `source_manifest.json`不仅记录commit，还记录8个parity source和54个golden的逐文件SHA-256及目录digest；问题是parity runner没有消费这份身份。
- 参考项目原始`parity.test.ts`会真实执行每个scenario两次，再与committed golden比较；其determinism意图正确，可作为oracle adapter的语义来源。
- `reference_full_trace_probe.ts`能把所有采样帧临时提升为full frame，并在输出前投影回golden shape校验；它适合作为hermetic reference runner的原型。
- WTR1 decoder有magic、version、dictionary fingerprint、trailing bytes、duplicate key、depth、collection、frame与总bytes限制；这些负向合同不应因重构而丢失。
- `GoldenSuite::compare`能返回第一个JSON structural path及expected/actual；可作为首差异分析器的最低基础。
- 54个golden覆盖251条声明的coverage reference，场景跨战斗、AI、pet、arena、delve、quest、economy、social与progression；问题是Zircon实际覆盖为0/54，不是参考场景本身太少。
- 现有四个M3 kernel contract与trace fixture仍有局部价值，但必须重新分类为kernel fixture/codec vector，不能命名或汇总为产品parity。

### 2.4 参考实现责任对照

- Unreal Automation把execution entries、error、telemetry、comparison artifact和跨device test report作为结构化结果；AutomationController会把comparison incoming/difference/approved文件挂到具体test/pass/device。Zircon应学习result与artifact归属，不照搬宏或消息总线。
- Unreal `UDemoNetDriver`把frame、checkpoint、scrub、fast-forward、task、streamer、event data与失败结果放在同一replay lifecycle中；这说明“首差异”之后仍需要可定位checkpoint与重放输入，而不是只打印JSON path。
- Unreal TraceLog显式拥有session/trace GUID、channel、thread registration、tail/cache buffer和file/remote output；WOC trace也需要session/build/schema/channel identity，不能只靠scenario name和60-bit字典fingerprint。
- Bevy `ScheduleRunnerPlugin`把run-once/loop与wait语义显式化，`Stepping`保存schedule order、system cursor、break/continue/always-run策略。WOC deterministic runner应显式固定schedule和可步进边界，而不是让产品loop与测试loop各自推进。
- Godot `EngineProfiler`通过注册名、toggle options、frame data与四类frame time建立可启停的采集owner。WOC sampler应是受控observer，不应由手写fixture假装运行期采样。
- Fyrox Visitor以稳定版本枚举、CURRENT_VERSION、tree/field IR和typed `VisitError`管理持久格式演进。WTR/trace schema也需要独立版本、迁移/拒绝策略和typed decode error，而不只版本常量`1`。
- Unity Graphics RenderGraph tests会强制生成compiler audit，再直接断言compiled passes、resource use、culling与exception；它体现测试应观察真实编译/执行产物。WOC同理必须采样actual world/runtime结果，不能把预写expected bytes当作actual。

## 3. 当前P0

### WOC-PARITY-P0-001 · native double-run复制expected，产品和ZrVM从未产生actual

`double_run_compares_duplicates_before_the_golden`读取一个golden后把`|| expected.clone()`交给`compare_double_run`。该closure没有启动WOC product、ZrVM、world、fixed tick或scenario driver；全仓也没有其他caller。必须建立真实`WocActualTraceRunner`，每次调用都从独立sandbox启动目标BuildSet、执行输入、读取VM trace，再做actual-vs-actual和actual-vs-oracle比较。删除或重命名当前假绿测试，禁止它进入parity通过计数。

### WOC-PARITY-P0-002 · 54个Zircon scenario owner全部不存在，materialize又删除ownership字段

source catalog的54条`woc_owner`全部不存在，而native manifest删除了`index/factory/ownership_class/woc_owner`。必须由单一catalog compiler保留reference owner与actual owner，验证54/54路径、manifest、binary entry、capability和runner registration；missing owner必须在执行前fatal。四条`src/parity/m3_*`不能暗中替代声明的54条owner。

### WOC-PARITY-P0-003 · 四条M3 trace由expected常量模板生成，不是运行期world sampler

lifecycle、locomotion、roster只用一次`scenarioContractTest()`作为总开关，trace body与kernel state无数据依赖；targeting也只把少量metrics填入大规模常量模板。产品行为大面积变化但contract仍返回1时，固定bytes仍可通过self-test。必须从实际WOC authority/world snapshot、event stream和RNG observer构造trace；hand-authored WTR1仅允许留作codec test vector，并在结果taxonomy中明确标记`fixture`而非`parity`。

### WOC-PARITY-P0-004 · oracle manifest绑定固定commit，actual probe却执行可变工作树

物化器通过`git show SOURCE_COMMIT:path`读取bytes，probe却相对import `dev/world-of-claudecraft`工作树。本轮动态运行已证明HEAD相同仍可因dirty source漂移。必须让reference runner从detached、read-only、digest-verified checkout或等价content-addressed source tree执行；运行前验证manifest的8个source hashes、package/lock/toolchain和依赖安装身份，禁止工作树状态影响oracle结果。

### WOC-PARITY-P0-005 · golden promotion不是事务，native guard是无consumer的死表面

参考项目用`UPDATE_PARITY=1`直接覆写golden；native `GoldenUpdateGuard`只检查环境值和commit confirmation，没有更新API或caller。没有candidate/approved分区、old/new trace diff、owner approval、全量rerun、atomic publication、rollback或promotion receipt。必须建立两阶段`GoldenPromotionCoordinator`：runner先产生immutable candidate与差异包，批准后一次性发布catalog/goldens/dictionary/digest，并以故障注入证明失败不会留下混合代。

### WOC-PARITY-P0-006 · parity结果未进入required TestPlan/BuildSet/Release admission

materialize、reference probe、WTR encode/verify不在工具package的`generate/check`脚本中；native tests当前又被上游protocol编译错误阻断。没有统一结果能证明54 required scenarios全部被选择、实际执行、比较并绑定同一reference identity、Zircon BuildSet、trace schema和artifact digest。必须由Tooling 10的TestPlan调度reference/actual/differential lanes，生成完整`ParityValidationSet`，再由Tooling 09 release admission消费；缺失、旧代、零场景或部分场景均为红。

## 4. Catalog、Oracle 与 Source Identity 差距

| ID | 差距与重构要求 |
|---|---|
| WOC-PARITY-P1-001 | 没有versioned `ParityPlanManifest`；54、路径和工具常量分散。建立唯一suite schema与compiler。 |
| WOC-PARITY-P1-002 | scenario identity只有可重命名name/filename。引入稳定`ScenarioId`、display name和rename alias。 |
| WOC-PARITY-P1-003 | materialize删除index、factory、ownership class和actual owner。投影必须保持执行所需字段或以版本化引用关联。 |
| WOC-PARITY-P1-004 | `source_owner`只验证字符串和文件，不验证`#factory`符号存在且正好对应scenario。catalog compiler必须解析并验证anchor。 |
| WOC-PARITY-P1-005 | `EXPECTED_GOLDENS = 54`在多个实现硬编码。required count应由签名manifest导出，常量只作生成结果。 |
| WOC-PARITY-P1-006 | `current-head`名称暗示移动目标，实际却固定旧commit。目录和suite ID应使用不可变source identity，alias单独管理。 |
| WOC-PARITY-P1-007 | source commit在materializer、trace codegen、protocol/native等多处复制。统一从BuildSet/ParityPlan读取并检查一致。 |
| WOC-PARITY-P1-008 | reference runner不记录dirty/untracked/submodule状态。hermetic runner应拒绝或完全隔离工作树。 |
| WOC-PARITY-P1-009 | 工具通过相对路径依赖兄弟`dev/world-of-claudecraft`仓库。定义显式oracle source input与content digest，缺失给typed error。 |
| WOC-PARITY-P1-010 | tools package未声明`tsx`，probe依赖参考仓偶然安装的binary。锁定Node/package manager/dependency artifact。 |
| WOC-PARITY-P1-011 | probe没有package script，也不在`check`中。注册独立hermetic oracle lane，不能靠人工命令。 |
| WOC-PARITY-P1-012 | probe monkey-patch全局`Recorder.prototype`。改为显式sampler policy/constructor injection，保证并发和失败恢复。 |

## 5. Scenario Driver、Runtime Actual 与 Determinism 差距

| ID | 差距与重构要求 |
|---|---|
| WOC-PARITY-P1-013 | 没有reference oracle adapter接口，脚本直接import具体TS模块。建立`ReferenceOracleAdapter`并返回typed run receipt。 |
| WOC-PARITY-P1-014 | 没有Zircon 54场景dispatch registry。每个ScenarioId必须绑定driver、actual owner、entry和capability。 |
| WOC-PARITY-P1-015 | reference与Zircon未共享versioned input/drive IR。定义命令、时间推进、fixture和assertion-free observation序列。 |
| WOC-PARITY-P1-016 | fixed tick、wall clock、timer和schedule注入未形成runner contract。测试时必须使用显式virtual clock与固定schedule。 |
| WOC-PARITY-P1-017 | 场景间world、static、service、RNG和artifact reset没有证明。每次run使用新generation并验证teardown。 |
| WOC-PARITY-P1-018 | filesystem、network、process、user config等副作用没有sandbox capability声明。默认deny并记录实际使用。 |
| WOC-PARITY-P1-019 | 没有Zircon actual-vs-actual双跑。54场景每个至少独立冷启动两次并比较trace与side-effect receipt。 |
| WOC-PARITY-P1-020 | seed只有单个固定值，缺少受控seed corpus。保留canonical seed并增加不改golden的determinism/property lane。 |
| WOC-PARITY-P1-021 | 没有single-thread、多线程和不同合法schedule的确定性边界声明。先定义保证范围，再做矩阵验证。 |
| WOC-PARITY-P1-022 | 没有Windows/Linux、debug/release、interpreter/binary语义矩阵。每项声明required/unsupported及理由。 |
| WOC-PARITY-P1-023 | runner没有typed timeout/cancel/crash/VM trap/host exit结果。禁止把无trace统一解释成JSON mismatch。 |
| WOC-PARITY-P1-024 | 没有scenario capability admission。缺模块、artifact、VM feature或world owner时应unschedulable-fatal，不得生成fixture。 |
| WOC-PARITY-P1-025 | 四个M3 self-test只比较自身length/FNV和二次build。重分类为fixture tests，并增加actual state mutation sensitivity test。 |
| WOC-PARITY-P1-026 | actual runner没有经过App 03拥有的真实WOC ProductHost/transaction/world entry。禁止另建只供测试的第二套simulation。 |

## 6. Trace Schema、Canonicalization 与 WTR 差距

| ID | 差距与重构要求 |
|---|---|
| WOC-PARITY-P1-027 | WTR version 1没有绑定独立TraceSchemaId、world schema或BuildSet。header加入完整qualified identity与compatibility policy。 |
| WOC-PARITY-P1-028 | dictionary只用SHA-256前15 hex字符即60 bit fingerprint。artifact identity保留完整cryptographic digest，短值仅作显示。 |
| WOC-PARITY-P1-029 | state/event/RNG/byte identity使用32-bit FNV-1a，碰撞可产生假相等。parity gate使用至少128/256-bit digest并保留结构diff。 |
| WOC-PARITY-P1-030 | event只保存32-bit digest，没有event bodies或可追踪引用。失败包必须能展开首个不同事件及source system。 |
| WOC-PARITY-P1-031 | 中间frame在golden中通常只有state digest，碰撞时不可诊断。保留可按需取回的full checkpoint/content-addressed snapshot。 |
| WOC-PARITY-P1-032 | Entity/META exclusion list由参考实现手写，新字段可能被无审查排除。建立schema registry、field policy与coverage guard。 |
| WOC-PARITY-P1-033 | 1e-6量化、负半值、整数和非有限数语义没有独立schema文档/版本。用跨语言vector固定所有边界。 |
| WOC-PARITY-P1-034 | canonicalization把function静默变为null。遇到非数据类型应typed reject并给字段路径。 |
| WOC-PARITY-P1-035 | undefined与null合并，可能掩盖presence语义。schema逐字段声明optional/null/default，禁止全局隐式折叠。 |
| WOC-PARITY-P1-036 | Map/Set都降为普通array，类型身份丢失。trace value保留collection kind或schema提供不可歧义解释。 |
| WOC-PARITY-P1-037 | Map/Set排序在canonical key相等时无稳定tie-breaker。拒绝重复canonical key/value或定义完整顺序。 |
| WOC-PARITY-P1-038 | native `TraceValue`支持Map/Set，WTR codec却没有对应tag。收敛单一value algebra及跨语言roundtrip。 |
| WOC-PARITY-P1-039 | WTR所有字符串必须预先存在golden-derived dictionary。实际新增字符串会在首差异前编码失败；增加受限literal/fallback。 |
| WOC-PARITY-P1-040 | symbol只从expected goldens生成，actual新字段/值无法表达。dictionary应来自versioned schema，未知值作为诊断数据而非假绿。 |
| WOC-PARITY-P1-041 | Rust `trace_symbol_id`每次线性扫描1,070项。生成双向表/perfect map并记录lookup budget。 |
| WOC-PARITY-P1-042 | decoder限制单collection entries但没有aggregate node/value work budget。增加总节点、字符串、allocation和CPU预算。 |

## 7. Differential、First Divergence 与 Replay 差距

| ID | 差距与重构要求 |
|---|---|
| WOC-PARITY-P1-043 | decoder不验证frame tick单调、范围和final tick。TraceValidator在比较前执行跨frame不变量。 |
| WOC-PARITY-P1-044 | `sampleEvery`、ticks、frame count和label checkpoint关系未验证。scenario plan声明采样策略并校验。 |
| WOC-PARITY-P1-045 | top-level draws/drawDigest未与final frame RNG一致性关联。增加累计关系与每draw可定位索引。 |
| WOC-PARITY-P1-046 | nextId、entity identity和spawn/despawn集合没有跨frame不变量。由world schema validator报告非法复用/倒退。 |
| WOC-PARITY-P1-047 | trace coverage未与catalog coverage逐项匹配。runner验证无缺失、无未知、无静默降级。 |
| WOC-PARITY-P1-048 | label是自由symbol，没有phase schema和唯一性。定义checkpoint kind、ordinal与owner system。 |
| WOC-PARITY-P1-049 | JSON compare只返回首个path和值，没有frame/tick/system/input上下文。输出结构化`DivergenceRecord`。 |
| WOC-PARITY-P1-050 | 没有二分checkpoint/逐tick缩小。用deterministic replay在首个不同digest前后自动定位。 |
| WOC-PARITY-P1-051 | 没有字段语义分类，数值、identity、ordering、missing/extra都只是JSON不同。schema驱动分类与严重度。 |
| WOC-PARITY-P1-052 | 没有输入、RNG、clock、build、snapshot与logs组成的replay bundle。失败必须可离线重现。 |
| WOC-PARITY-P1-053 | 没有最小化driver；54场景大trace失败后只能整段重跑。提供prefix cut与受控delta-debugging。 |
| WOC-PARITY-P1-054 | 没有验证comparison对单字段、单event、单RNG draw、单tick drift均必红。建立mutation-sensitivity suite。 |

## 8. Golden Repository、Promotion 与 Supply Chain 差距

| ID | 差距与重构要求 |
|---|---|
| WOC-PARITY-P1-055 | native用manifest path的basename读取golden，目录语义被丢弃。使用root-relative规范路径并拒绝逃逸/歧义。 |
| WOC-PARITY-P1-056 | JSON读取没有schema validation，未知/缺失top/frame字段可能晚到compare才暴露。加载时严格验证。 |
| WOC-PARITY-P1-057 | coverage rows未校验唯一、稳定ID和owner。把coverage从描述字符串提升为versioned `CoverageId`。 |
| WOC-PARITY-P1-058 | golden reader没有bytes/depth/node预算。先限额与流式hash，再解析受信程度不同的artifact。 |
| WOC-PARITY-P1-059 | root `source_manifest`的逐文件identity未被native GoldenSuite消费。ParityPlan引用并验证完整source tree digest。 |
| WOC-PARITY-P1-060 | golden SHA锁定bytes，但没有trace schema/compiler/toolchain identity。artifact manifest绑定所有producer输入。 |
| WOC-PARITY-P1-061 | native update confirmation只比较一个commit字符串且无caller。删除死表面或接入真实promotion state machine与principal。 |
| WOC-PARITY-P1-062 | reference `UPDATE_PARITY=1`可直接改写tracked goldens。改为candidate输出，默认命令永不覆写approved目录。 |

## 9. Orchestration、Evidence 与性能差距

| ID | 差距与重构要求 |
|---|---|
| WOC-PARITY-P1-063 | materialize/probe/WTR encode/verify缺少package scripts和统一CLI。注册明确generate/check/run/compare/promote命令。 |
| WOC-PARITY-P1-064 | aggregate npm check在更早payload count失败后短路，无法表达parity未运行。TestPlan产生per-lane terminal result。 |
| WOC-PARITY-P1-065 | native workspace编译阻断时12个parity test均未执行。结果必须区分build-failed与test-failed并保持required红。 |
| WOC-PARITY-P1-066 | 4个dump binary目录存在而4个test binary缺失，没有clean-clone artifact currentness。构建receipt声明8个预期产物及digest。 |
| WOC-PARITY-P1-067 | trace/golden结果没有source/build/runner/host/toolchain/attempt/timestamp/expiry receipt。统一写入ValidationSet。 |
| WOC-PARITY-P1-068 | hex scalar bridge把binary扩大2倍并逐字串接，54场景没有时间/内存/bytes预算。建立基准、上限和binary transport gate。 |

## 10. P2 可维护性与体验差距

| ID | 改进项 |
|---|---|
| WOC-PARITY-P2-001 | CLI支持按ScenarioId、tag、coverage、owner和changed domain查询，但输出仍保留required闭包。 |
| WOC-PARITY-P2-002 | 为54场景建立稳定shard排序和历史时长均衡，避免文件顺序改变分片。 |
| WOC-PARITY-P2-003 | 失败摘要同时输出人类文本和JSON，不再依赖Node assertion的超长对象dump。 |
| WOC-PARITY-P2-004 | first-difference viewer显示前后checkpoint、input、event、RNG和schema owner。 |
| WOC-PARITY-P2-005 | 大型full snapshots采用content-addressed压缩，manifest保留未压缩digest。 |
| WOC-PARITY-P2-006 | 建立golden bytes/frame/node/dictionary增长趋势与review budget。 |
| WOC-PARITY-P2-007 | 为symbol/schema/coverage提供可点击source provenance，不把数值ID留给人工反查。 |
| WOC-PARITY-P2-008 | 生成dictionary reverse lookup与collision audit报告，便于review generated变更。 |
| WOC-PARITY-P2-009 | replay bundle支持单命令本地重放并打印所需BuildSet/artifact缺失项。 |
| WOC-PARITY-P2-010 | 支持保留最近N次通过和全部失败bundle，策略由artifact retention配置。 |
| WOC-PARITY-P2-011 | promotion UI/报告并列展示old/reference candidate/Zircon actual三方差异。 |
| WOC-PARITY-P2-012 | 对超大scenario提供checkpoint索引与随机访问，避免每次从tick 0重放。 |
| WOC-PARITY-P2-013 | 增加schema migration dry-run，提前报告旧golden不可迁移字段。 |
| WOC-PARITY-P2-014 | 统计每个coverage ID最近通过、首差异频率和owner，不用总通过率掩盖热点。 |
| WOC-PARITY-P2-015 | 为reference oracle依赖安装提供离线cache提示和明确修复命令。 |
| WOC-PARITY-P2-016 | 统一术语：fixture/vector、kernel contract、reference replay、product parity、qualification不得混用。 |

## 11. 目标 Owner 与 Schema

### 11.1 Owner 边界

| Owner | 唯一职责 | 不拥有 |
|---|---|---|
| `WocParityCatalogCompiler` | 编译reference/actual owner、scenario、coverage、artifact与required集合 | 不执行游戏逻辑 |
| `WocReferenceOracleAdapter` | 从content-addressed参考source执行scenario并产出oracle trace receipt | 不写approved golden |
| `WocActualTraceRunner` | 通过App 03 ProductHost启动目标BuildSet、注入driver、采样实际world | 不复制expected状态 |
| `WocTraceSchemaRegistry` | value/field/exclusion/quantization/event/RNG/checkpoint/WTR schema | 不决定业务正确值 |
| `WocDifferentialEngine` | 验证trace、比较、定位首差异、生成replay bundle | 不批准行为变化 |
| `WocGoldenRepository` | immutable approved/candidate artifact、digest、版本与读取限额 | 不从环境变量直接覆写 |
| `WocGoldenPromotionCoordinator` | candidate、owner approval、全量rerun、atomic publish、rollback receipt | 不绕过required gate |
| `WocParityTestAdapter` | 接入Tooling 10 TestPlan并生成ValidationSet | 不重新实现runner或release |

App 03继续拥有真实WOC product host/VM/world transaction；Runtime 13-19继续拥有战斗、world、content、protocol业务语义；Tooling 05拥有通用codegen currentness；Tooling 07拥有性能证据；Tooling 10拥有测试计划与结果闭包；Tooling 09拥有release promotion。本报告只拥有WOC-specific oracle/actual/differential/golden/replay evidence链。

### 11.2 最低 Schema

`ParityPlanManifest`至少包含：`schema_version`、`suite_id/version`、reference source tree/lock/toolchain identity、Zircon BuildSet、trace/WTR schema digest、required ScenarioId集合；每个场景包含reference owner、actual owner、driver artifact、seed/clock/schedule、capabilities、coverage IDs、golden artifact digest和预算。

`TraceEnvelope`至少包含：schema/build/source/run/session identity、ScenarioId、driver digest、clock/schedule、RNG algorithm/seed/draw stream identity、ordered frames；frame包含tick/checkpoint、world generation、state/event/RNG digest以及可取回full snapshot引用。digest必须是cryptographic，FNV只可作非安全快速checksum。

`DivergenceRecord`至少包含：scenario/attempt、first differing tick/checkpoint/path/category、reference/actual typed values、preceding input/event/RNG位置、source owner、full artifact references和replay command。

`GoldenPromotionReceipt`至少包含：old/candidate/new suite digest、reference source identity、producer BuildSet/toolchain、全部scenario results、approver principal、reason、timestamp、atomic commit与rollback状态。

`ParityValidationSet`至少包含：plan/build/source/schema digest、selected/discovered/executed/passed/failed/unscheduled counts、per-scenario attempt、artifact digests、expiry和required-closure verdict。只有54/54同代通过才能输出release-consumable绿色结果。

## 12. 分层重构里程碑

### M0 · False-green hard cut

将expected-clone double-run和四条常量M3 trace从产品parity统计中移除/重分类；catalog owner缺失、零actual runner和部分执行全部显式红。保留现有fixture tests，但改名和结果类型必须不可误读。

### M1 · Hermetic oracle 与 immutable source

以source manifest构建read-only fixed-commit checkout，锁定Node/package manager/dependencies，运行54个reference scenario双跑；dirty主工作树不得影响输出。产出oracle run receipt与full trace artifact。

### M2 · Catalog 与 trace schema冻结

定义ScenarioId/CoverageId、54个真实actual owner、driver IR、TraceEnvelope/WTR schema、完整digest与limits。先生成catalog completeness报告，再允许写actual runner。

### M3 · 真实actual sampler纵切

通过App 03产品host实现`entity_roster`端到端纵切：同一driver启动真实ZrVM/world，运行期observer采样，不允许expected输入。随后迁移现有lifecycle/locomotion/targeting，删除常量trace authority。

### M4 · 54场景 differential 与 replay

逐domain补齐54 actual owners，每场景cold double-run、oracle compare、mutation sensitivity、first divergence和replay bundle。Runtime 13-19的业务P0必须按依赖先修，不得在sampler中补第二套规则。

### M5 · Golden promotion transaction

建立candidate/approved仓库、三方diff、owner审批、全量rerun、dictionary/schema联动、原子发布、故障恢复和rollback。环境变量只可请求candidate，不可直接改approved。

### M6 · TestPlan、CI 与 release qualification

把reference、actual、differential、wire robustness、performance作为独立required lanes接入Tooling 10；输出ValidationSet给Tooling 09。PR可按impact selection执行但必须可解释，nightly/qualification执行54/54全量。

## 13. 验收门

| Gate | 必须证明 |
|---|---|
| WOC-PARITY-G01 | catalog中54/54 reference owner与actual owner均存在、可解析、可编译、可注册；第55个意外golden也fatal。 |
| WOC-PARITY-G02 | reference runner在主参考仓clean/dirty/缺失三种状态下都只消费manifest source；同input产出同digest。 |
| WOC-PARITY-G03 | 修改真实Zircon world字段/事件/RNG draw的mutation test必然改变actual trace并使gate红。 |
| WOC-PARITY-G04 | 单独修改手写fixture/expected不能产生产品actual通过结果；结果provenance显示真实runner。 |
| WOC-PARITY-G05 | 54个Zircon场景各冷启动双跑，actual-vs-actual 54/54一致，0未调度、0零帧。 |
| WOC-PARITY-G06 | fixed oracle与Zircon actual在同plan/schema下54/54结构一致；每项有独立attempt receipt。 |
| WOC-PARITY-G07 | 任一source/build/driver/schema/dictionary/golden digest变化都会使旧结果过期。 |
| WOC-PARITY-G08 | WTR fuzz/negative corpus覆盖truncation、unknown tag/symbol、duplicate、overflow、depth、aggregate budget和trailing bytes。 |
| WOC-PARITY-G09 | state/event/RNG identity使用cryptographic digest；构造FNV碰撞不能使parity通过。 |
| WOC-PARITY-G10 | 单tick drift自动定位到首个tick、input/event/RNG/field path及owner，不只输出整份JSON。 |
| WOC-PARITY-G11 | 每个失败bundle在隔离机器可用单命令重现相同first divergence或给出typed missing artifact。 |
| WOC-PARITY-G12 | promotion在每个写入阶段故障后approved suite仍为完整old或完整new，不出现混合代。 |
| WOC-PARITY-G13 | native workspace clean build后12个woc_parity tests实际discovered/executed，新增真实runner tests另计。 |
| WOC-PARITY-G14 | npm/tool plan即使早期lane失败也记录parity为blocked/not-run，不能静默省略或绿色。 |
| WOC-PARITY-G15 | clean clone能从manifest生成8个Zr dump/test artifacts并验证source/build digest，不依赖tracked旧binary。 |
| WOC-PARITY-G16 | ValidationSet严格记录54 selected/discovered/executed/passed；53/54、54 selected/0 executed都阻断release。 |
| WOC-PARITY-G17 | 54全量记录wall/CPU/peak memory/trace bytes，超预算失败并保留最大场景证据。 |
| WOC-PARITY-G18 | interpreter/binary及required平台矩阵按plan执行；unsupported必须在catalog声明，不能运行时跳过。 |

## 14. 禁止的实施顺序

1. 禁止先补更多常量WTR1模板或把54个golden机械翻译成Zr代码；这会扩大假actual。
2. 禁止在reference工作树dirty时用`UPDATE_PARITY=1`重基线来消除probe差异；先修hermetic source identity。
3. 禁止让native `GoldenSuite`继续删除actual owner字段后，再靠人工表格声明覆盖完整。
4. 禁止只把四个dump命令接入CI就声称parity存在；必须证明trace来自真实product world。
5. 禁止用更宽量化、更多exclude、只保留digest或降低采样率隐藏业务差异；schema变化必须走promotion。
6. 禁止让Tooling层重写Runtime 13-19的业务规则以匹配golden；actual runner只能驱动和观察真实authority。
7. 禁止在native/npm build仍红时发布“parity tests passed”；只能报告build-failed、not-run和静态asset checks各自事实。
8. 禁止在54/54 source/build-bound证据完成前将WOC作为Zircon超越Unreal的功能或性能证明。

## 15. 最终判断

WOC parity并非从零开始：参考场景、golden、source manifest、dictionary、canonical helper和wire negative tests都值得保留。但当前系统把三类不同证据混在一起：reference golden自洽、kernel contract fixture稳定、产品行为一致。前两类局部成立，第三类尚未建立。

最关键的重构不是增加比较断言，而是恢复真实数据流：immutable reference source执行产生oracle，真实Zircon ProductHost执行产生actual，versioned trace schema连接两者，differential/replay解释差异，golden promotion管理有意变化，TestPlan和Release admission证明required闭包。只有这条链在54个场景上逐项形成同代receipt，WOC parity才可从“参考资产集合”升级为工程级回归与迁移证据。
