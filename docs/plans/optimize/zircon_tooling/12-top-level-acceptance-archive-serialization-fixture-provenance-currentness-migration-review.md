---
related_code:
  - tests/acceptance
  - tests/fixtures/serialization
  - tools/tests/test_runtime_ui_table_module_structure.py
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_schema.rs
  - zircon_runtime_interface/src/project/tests/manifest_summary.rs
  - zircon_runtime/src/asset/tests/project/manifest.rs
  - zircon_hub/src/projects/validation.rs
  - zircon_runtime/src/scene/tests/ecs_reflect/foundation/versioned_json.rs
  - zircon_runtime/tests/plan11_scene_serialization_contract.rs
tests:
  - tests/fixtures/serialization/project-manifest/v1/zircon-project.toml
  - tests/fixtures/serialization/project-manifest/v2/zircon-project.toml
  - tests/fixtures/serialization/project-manifest/future/zircon-project.toml
  - tests/fixtures/serialization/project-manifest/invalid/zircon-project.toml
  - tests/fixtures/serialization/scene-dynamic/v0/dynamic-scene.json
  - tests/fixtures/serialization/scene-reflection/v0/reflected-value.json
plan_sources:
  - .codex/skills/zircon-project-skills/evidence-driven-wsl-validation/acceptance-and-evidence/SKILL.md
  - docs/plans/milestone-validation-policy.md
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_tooling/07-performance-benchmark-profile-capture-symbol-crash-evidence-baseline-review.md
  - docs/plans/optimize/zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md
  - docs/plans/optimize/zircon_runtime_interface/02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/AutomationTest.h
  - dev/UnrealEngine/Engine/Source/Developer/AutomationController/Private/AutomationReport.cpp
  - dev/UnrealEngine/Engine/Source/Programs/AutomationTool/Gauntlet
  - dev/bevy/tools/ci/src/commands/test.rs
  - dev/bevy/tools/ci/src/ci.rs
  - dev/Fyrox/.github/workflows/ci.yml
  - dev/godot/tests/test_main.cpp
  - dev/godot/tests/test_macros.h
  - dev/Graphics/.yamato/postprocessing-win-dx12.yml
  - dev/Graphics/TestProjects/PostProcessing_Tests/Packages/manifest.json
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 12 · 顶层 Acceptance 历史归档与序列化 Fixture 来源、时效、迁移工程化差距

## 1. 结论

顶层`tests/`当前有106个tracked文件、5,988行、532,197 bytes。其中`tests/acceptance`的100篇Markdown占5,938行/530,914 bytes，`tests/fixtures/serialization`的6个TOML/JSON样本占50行/1,283 bytes。这两个目录物理相邻，但产品语义完全不同：前者已被仓库skill明确宣布为历史归档，当前唯一权威证据应写入注册计划的`状态与产出记录`；后者仍被Runtime、Runtime Interface与Hub的真实迁移测试消费。

Acceptance归档还没有完成从“曾经的验收真值”到“只读历史材料”的硬切。100篇中只有41篇有frontmatter、39篇有`Date`、25篇在frontmatter里有`status`、31篇有`Acceptance Decision`；63篇包含Accepted语义，91篇包含passed，同时77篇也包含failed、33篇包含in-progress、30篇包含pending。这些是可重叠文本命中而非互斥状态，仓库没有机器规则区分“某个局部切片曾通过”“完整里程碑当前通过”“历史失败仍存在”或“已被新计划取代”。

没有一篇记录包含可识别的source fingerprint/commit/build identity，也没有一篇链接机器结果receipt；只有2篇出现artifact hash/digest类词。29篇共保存372个Windows绝对路径，主要落在`E:\zircon-build`、`D:\cargo-targets`、`E:\cargo-targets`和`E:\tmp`。正文共出现492个Cargo、53个Python和78个PowerShell命令标记，但它们是手工转录，不是CI可执行plan或结果对象。历史命令、当前结论和外部阻断在同一自由文本里累积，无法自动失效。

41篇frontmatter虽都能被YAML parser读取，却不是稳定schema：总计664个list item混合586个路径、59条命令/备注、17个测试filter/identifier和2个glob；8个未加引号的`user:`条目被解析成mapping而不是string。14篇存在113组跨字段重复值，共231次出现。按排除命令、identifier和glob后的精确路径核对，14篇仍有46次失效引用、涉及32个唯一路径；其中11次是output record迁到`docs/plans/_archive`后未更新，另外35次是源码、测试或session路径已经移动/删除。

外部反向引用也未收敛。仓内82个文件形成187次`tests/acceptance/*.md`引用，命中68个名字；其中7个目标文件根本不存在，现存100篇中只有61篇被引用、39篇没有任何入站引用。更严重的是两条required test owner直接读取历史Markdown：Python结构测试要求`runtime-ui-table-mutation-owner-split.md`存在且含owner path，Runtime absorption测试要求`frameworks-05-asset-ui-loader-hard-cutover.md`仍含两段指定文本。修改历史叙述即可让结构门通过或失败，测试没有检查实际owner manifest、module contract或当前行为。

6个serialization fixture是可保留底座。四个project manifest分别覆盖v1迁移、v2当前、future拒绝与invalid semver；两个JSON覆盖retired AssetRef shape和legacy DynamicScene。全部能被标准TOML/JSON parser读取，四个project样本由Interface/Runtime/Hub共享，两个scene样本由真实Rust测试`include_str!`消费。但目录没有README、corpus manifest、schema ID/version范围、原始writer与版本、不可变digest、license/provenance或negative-case reason。`scene-dynamic/v0`内仍写`format_version = 1`，目录代际与payload代际没有说明。维护者可以同时改旧fixture和新期望，让迁移测试继续绿色却丢失真实历史兼容性。

本篇不重复Tooling10的通用test inventory/selection/isolation/result protocol，也不重复Tooling07的性能EvidenceSet。Runtime04/05与Interface02继续拥有project/scene/reflection实际schema、reader、writer和migration。本篇只拥有`tests/acceptance`历史归档硬切、旧引用迁移、archive catalog，以及`tests/fixtures/serialization`历史样本来源与不可变性。本轮登记 **3项P0、60项P1和14项P2**。

## 2. 审查边界与物理清单

### 2.1 顶层目录

| 子域 | 文件 | 行数 | bytes | 当前角色 |
|---|---:|---:|---:|---|
| `tests/acceptance` | 100 Markdown | 5,938 | 530,914 | policy已定义为历史归档，但仍混有current/accepted语义与直接test consumer |
| project manifest fixtures | 4 TOML | 22 | 696 | v1/v2/future/invalid，共享给Interface/Runtime/Hub |
| scene fixtures | 2 JSON | 28 | 587 | reflected value v0与dynamic scene v0迁移输入 |
| **合计** | **106** | **5,988** | **532,197** | **archive与executable fixture需要分型治理** |

100篇按文件名粗分包含50篇Runtime、11篇Editor、10篇Frameworks、8篇Render、6篇Plugins、8篇其他UI、2篇Session tooling，以及Navigation、Particles和其他跨域记录。它们是多个旧计划阶段逐次写入的结果，不是同一时间点、同一schema或同一测试矩阵的快照。

### 2.2 Metadata与结果形状

| 字段/信号 | 文件数 | 含义 |
|---|---:|---|
| frontmatter | 41 | 59篇无任何机器头 |
| frontmatter `status` | 25 | 75篇无头部状态；已用status值大多为一次性复合句 |
| `Date` | 39 | 61篇正文也无显式日期 |
| `Scope` section | 50 | 一半记录没有统一范围段 |
| Tooling/Validation/Test section | 33 | 67篇无法按章节提取命令 |
| Results section | 28 | 72篇没有统一结果段 |
| Acceptance Decision | 31 | 69篇没有统一决策段 |
| source fingerprint/hash/commit identity | 0 | 无法绑定当前source |
| machine result link | 0 | 无JUnit/SARIF/result receipt owner |
| Windows absolute path | 29 / 372次 | historical environment不可移植 |

Accepted、Not accepted、Open、In progress、Pending、Passed、Failed、Historical和Current命中分别为63、6、2、33、30、91、77、41、66篇。它们高度重叠，不能相加，也不能据此生成产品资格。

### 2.3 引用与consumer

| 关系 | 数量 | 风险 |
|---|---:|---|
| inbound引用 | 187次 / 82 source文件 | 没有中央迁移表，移动/删除fan-out不可见 |
| 被引用target name | 68 | 其中7个不存在 |
| 现存且被引用archive | 61 | 历史路由仍广泛散落在docs |
| 现存无入站archive | 39 | 没有retention/disposition状态 |
| production/tool direct reader | 2 | 测试把历史Markdown字符串当current contract |
| frontmatter exact missing path | 46次 / 32 unique / 14文件 | 11次archive迁移漂移，35次source/test/session漂移 |

7个不存在的target是`accesskit-bridge.md`、`material-layout-foundation.md`以及Navigation计划引用的五篇验收文件。当前没有link gate阻止新断链。

### 2.4 Serialization fixture

| Fixture | 直接消费 | 当前覆盖 |
|---|---|---|
| project v1 | Interface summary、Runtime manifest、Hub validation | 迁到v2并报告`migrated_from=1` |
| project v2 | 同上 | semver、asset roots、settings、Runtime/Interface projection一致 |
| project future | Interface/Runtime/Hub | format 3对supported 2 fail-close |
| project invalid | Interface/Runtime/Hub | 非法semver返回typed error |
| reflected-value v0 | Runtime scene unit test | 精确retired `{uuid,url}`迁到AssetRef，重存幂等 |
| dynamic-scene v0 | Runtime integration test | legacy无envelope输入迁移、重存、重载字节一致 |

全部6份文件本轮静态解析成功、无BOM且有终止换行；当前SHA-256可计算，但仓内没有被版本控制的corpus digest或origin receipt消费这些hash。

## 3. 参考引擎约束

- Unreal把测试定义与执行结果分型：`FAutomationTestInfo`持有唯一full test path、flags、participant count、source file/line、asset path和tags；`FAutomationTestExecutionInfo`持有success、duration、entries、error/warning count和telemetry，AutomationController再建立report tree。Zircon无需复制UI，但不能用一篇自由文本同时充当test identity、run result和长期archive。
- Bevy的CI tool把format、clippy、workspace tests、integration、doc、compile-fail、bench和example check建成可执行命令集合，并保留失败退出状态；历史说明文档不参与测试判定。
- Fyrox CI在Windows/Linux/macOS执行workspace all-target/all-feature build/test，并单独验证PC/Android/WASM template generation与upgrade。矩阵定义在workflow，某次结果由CI run拥有，而不是回写为永久“passed”Markdown。
- Godot用doctest runner与reporter listener执行真实test case，运行前后管理engine singleton/queue状态；test source和runner是定义面，运行输出是结果面。
- Unity Graphics的Yamato job固定platform/GPU/API/suite/testproject/timeout，UTR把`test-results`、logs与players发布为artifact；TestProject的package manifest固定依赖。Reference image或fixture是版本化测试输入，不等于某次通过结论。

## 4. 可保留的正确基础

1. Policy已经明确停止向`tests/acceptance`新增per-feature记录，并指定编号计划产出表为唯一权威证据位置。
2. 100篇记录保留了大量失败、外部阻断、具体命令和局部范围，没有普遍把红色aggregate删除。
3. 较新的41篇已有`related_code`、`plan_sources`与`output_records`雏形，可用于构建迁移映射。
4. 6个fixture体积小、可读、可由标准parser处理，适合作为版本迁移golden input。
5. project manifest fixture由Interface、Runtime、Hub共享，避免三套手写旧样本。
6. scene测试真实执行load→migrate→save→reload和future fail-close，不只是JSON shape snapshot。
7. invalid/future样本已有typed error断言，为扩展negative corpus提供基础。

## 5. P0：证据与兼容性真值硬阻断

### ACCEPTANCE-ARCHIVE-P0-001 · Required测试直接读取历史Markdown叙述

两个test owner把archive文件存在和特定字符串当作current结构合同。历史记录的措辞、路径或归档动作会改变测试结果，实际production/module owner即使正确也可能因archive文本漂移失败；反之只改Markdown就能通过门。必须把断言迁到owner manifest、module docs的typed anchor或生产API行为，再让archive完全退出test dependency graph。

### ACCEPTANCE-ARCHIVE-P0-002 · 100篇历史记录没有fail-closed archive catalog与canonical evidence迁移

Policy称它们是历史archive，但文件自身没有统一`archive_state`、canonical plan row、superseded evidence、source/build identity或expiry。63篇含Accepted、66篇含Current，7个外部引用目标已消失，仍无法机器判断哪些陈述只能作历史背景。建立manifest并完成100篇disposition前，任何工具、报告或人工索引都不得从这些文本推导当前通过状态。

### ACCEPTANCE-ARCHIVE-P0-003 · 迁移fixture可与reader期望一起被改写，不能证明真实历史兼容性

6份样本没有origin writer/build、schema identity、immutable digest和negative reason。当前测试只证明“当前仓库里的fixture与当前reader彼此一致”，不能证明fixture仍是某一历史发行版真实输出。必须冻结raw bytes、记录producer/version/license/hash并让变更走新fixture代际；旧fixture除明确corruption case外不可原地重写。

## 6. P1：Archive Identity 与 Schema

| ID | 当前差距 | 重构要求 |
|---|---|---|
| ACCEPTANCE-ARCHIVE-P1-001 | 59篇没有frontmatter | 全部进入独立archive manifest，不要求污染历史正文 |
| ACCEPTANCE-ARCHIVE-P1-002 | 61篇没有Date | manifest记录created/last_verified/archived时间与timezone |
| ACCEPTANCE-ARCHIVE-P1-003 | 只有25篇有header status | 使用有限枚举historical/superseded/quarantined，不复用运行结果状态 |
| ACCEPTANCE-ARCHIVE-P1-004 | 复合status值无法查询 | run result、scope result与archive lifecycle拆成不同字段 |
| ACCEPTANCE-ARCHIVE-P1-005 | 只有15篇声明`doc_type` | manifest统一record kind与schema version |
| ACCEPTANCE-ARCHIVE-P1-006 | 无stable record ID | ID包含domain/plan/milestone/record generation，不以文件名充当身份 |
| ACCEPTANCE-ARCHIVE-P1-007 | 无canonical owner | 每篇指向唯一编号计划与具体output row |
| ACCEPTANCE-ARCHIVE-P1-008 | 无supersedes/superseded_by | 迁移链显式、无环并可定位last authoritative evidence |
| ACCEPTANCE-ARCHIVE-P1-009 | 8个`user:`项被YAML解析为mapping | 所有scalar按schema校验，禁止隐式类型漂移 |
| ACCEPTANCE-ARCHIVE-P1-010 | 113组值跨字段重复 | path/test/command/record reference用typed对象去重 |

## 7. P1：Result Provenance 与 Currentness

| ID | 当前差距 | 重构要求 |
|---|---|---|
| ACCEPTANCE-ARCHIVE-P1-011 | 0篇绑定source fingerprint | archived run至少记录source revision与dirty-state digest |
| ACCEPTANCE-ARCHIVE-P1-012 | 0篇绑定BuildSet | toolchain/lock/features/target/profile统一用BuildSetId |
| ACCEPTANCE-ARCHIVE-P1-013 | 0篇链接machine result | 迁移时关联可验证receipt；无artifact明确标记narrative-only |
| ACCEPTANCE-ARCHIVE-P1-014 | passed/failed是正文token | terminal outcome只由Tooling10 result schema提供 |
| ACCEPTANCE-ARCHIVE-P1-015 | Accepted与pending/failed可同篇出现 | scope-level outcome与aggregate outcome分层，不从全文正则推导 |
| ACCEPTANCE-ARCHIVE-P1-016 | historical/current无有效期 | current只来自未过期qualification，不允许archive自称current |
| ACCEPTANCE-ARCHIVE-P1-017 | 手写test count不可验证inventory | 记录selected/discovered/executed/omitted/ignored inventory digest |
| ACCEPTANCE-ARCHIVE-P1-018 | 外部阻断没有typed owner/expiry | failure handoff绑定owner、source generation与resolution receipt |
| ACCEPTANCE-ARCHIVE-P1-019 | 没有attempt/retry/timeout模型 | 区分not-run、compile-blocked、timeout、cancel、failed与passed |
| ACCEPTANCE-ARCHIVE-P1-020 | 没有证据retention状态 | artifact missing/expired必须降级，不保留qualified结论 |

## 8. P1：路径、链接与 Source Drift

| ID | 当前差距 | 重构要求 |
|---|---|---|
| ACCEPTANCE-ARCHIVE-P1-021 | 46次frontmatter物理路径失效 | 生成迁移报告并分类moved/deleted/retired/typo |
| ACCEPTANCE-ARCHIVE-P1-022 | 11次output record遗漏`_archive`迁移 | canonical record ID解析路径，禁止手工复制旧路径 |
| ACCEPTANCE-ARCHIVE-P1-023 | 35次source/test/session路径失效 | 保留historical path同时记录resolved successor或terminal removal |
| ACCEPTANCE-ARCHIVE-P1-024 | 7个外部target不存在 | link gate覆盖tracked Markdown与manifest reference |
| ACCEPTANCE-ARCHIVE-P1-025 | 39篇现存archive无入站引用 | disposition为retain/indexed、deduplicate或remove candidate |
| ACCEPTANCE-ARCHIVE-P1-026 | 187次引用散在82文件 | 用record ID与生成索引替代path fan-out |
| ACCEPTANCE-ARCHIVE-P1-027 | list同时承载path/glob/filter/command | 每种值有独立typed field与validator |
| ACCEPTANCE-ARCHIVE-P1-028 | glob没有展开时点与匹配集 | 存glob、source revision与expanded inventory digest |
| ACCEPTANCE-ARCHIVE-P1-029 | 372个Windows绝对路径不可移植 | environment root token化，archive raw command单独保存 |
| ACCEPTANCE-ARCHIVE-P1-030 | 无自动source drift标记 | path/content fingerprint变化令record进入stale/quarantined |

## 9. P1：执行、CI 与 Archive 边界

| ID | 当前差距 | 重构要求 |
|---|---|---|
| ACCEPTANCE-ARCHIVE-P1-031 | 2个测试直接读取archive | M1前迁到production/module manifest与行为assertion |
| ACCEPTANCE-ARCHIVE-P1-032 | Python test要求archive存在 | archive删除/压缩不能改变owner结构测试结果 |
| ACCEPTANCE-ARCHIVE-P1-033 | Runtime test匹配中文历史句子 | test只验证当前ZUI importer owner与API，不读叙述 |
| ACCEPTANCE-ARCHIVE-P1-034 | archive目录没有CI角色声明 | CI明确排除其作为test definition/result input，仅做link/archive audit |
| ACCEPTANCE-ARCHIVE-P1-035 | 492个Cargo命令只是文本 | required command迁入machine-readable TestPlan |
| ACCEPTANCE-ARCHIVE-P1-036 | 53个Python/78个PowerShell命令同样不可执行 | command ID、cwd、env、timeout、dependency统一编排 |
| ACCEPTANCE-ARCHIVE-P1-037 | 历史target dir被误当环境要求 | raw transcript与reproducible command template分离 |
| ACCEPTANCE-ARCHIVE-P1-038 | 无runner version/exit/result artifact | run receipt记录runner、exit、duration、stdout/stderr/artifact digest |
| ACCEPTANCE-ARCHIVE-P1-039 | 无当前计划到archive的单向projection | 只允许canonical row链接archive背景，禁止archive反向覆盖状态 |
| ACCEPTANCE-ARCHIVE-P1-040 | archive重命名会破测试和文档 | migration tool原子更新manifest/links，direct code consumers必须为0 |

## 10. P1：Serialization Fixture Corpus

| ID | 当前差距 | 重构要求 |
|---|---|---|
| ACCEPTANCE-ARCHIVE-P1-041 | 无corpus manifest | 每个fixture登记schema、case、expected outcome、owner与digest |
| ACCEPTANCE-ARCHIVE-P1-042 | 无原始writer/version | 保存producer product、source/build、platform与生成命令 |
| ACCEPTANCE-ARCHIVE-P1-043 | 无immutable digest gate | 旧样本变更必须拒绝，新增修订使用新case/generation |
| ACCEPTANCE-ARCHIVE-P1-044 | project样本只有裸`format_version` | manifest明确schema ID与version interpretation owner |
| ACCEPTANCE-ARCHIVE-P1-045 | `scene-dynamic/v0`内含version 1 | 区分envelope generation、payload generation与writer release |
| ACCEPTANCE-ARCHIVE-P1-046 | invalid corpus只覆盖非法semver | 增加syntax/type/path/traversal/duplicate/budget/encoding负例 |
| ACCEPTANCE-ARCHIVE-P1-047 | future只覆盖紧邻version 3 | 覆盖极大version、unknown schema、valid-looking future payload fail-before-decode |
| ACCEPTANCE-ARCHIVE-P1-048 | reflected样本只覆盖一个AssetRef shape | 增加额外字段、partial shape、nested/list/map与unknown type历史样本 |
| ACCEPTANCE-ARCHIVE-P1-049 | project v2缺plugin/export/provider合同 | 随schema owner增加真实完整writer fixture，不原改最小样本 |
| ACCEPTANCE-ARCHIVE-P1-050 | 无size/depth/collection budget corpus | 增加边界与超限case，解析前后都验证typed拒绝 |

## 11. P1：治理、保留与参考对齐

| ID | 当前差距 | 重构要求 |
|---|---|---|
| ACCEPTANCE-ARCHIVE-P1-051 | 目录无README/owner | 声明historical-only、禁止新增、canonical路径与维护流程 |
| ACCEPTANCE-ARCHIVE-P1-052 | 无archive index | 生成按domain/plan/date/disposition查询的catalog |
| ACCEPTANCE-ARCHIVE-P1-053 | 无retention policy | 定义legal/debug/regression价值、保留期与删除审批 |
| ACCEPTANCE-ARCHIVE-P1-054 | 无重复记录检测 | 基于scope/source/result/digest识别复制或相互矛盾记录 |
| ACCEPTANCE-ARCHIVE-P1-055 | 无敏感信息扫描 | 命令、绝对路径、用户名、token/URL进入archive前做redaction audit |
| ACCEPTANCE-ARCHIVE-P1-056 | 无artifact tombstone | 已过期artifact保留typed tombstone与原retention，不留假链接 |
| ACCEPTANCE-ARCHIVE-P1-057 | 无schema migration tool | archive manifest升级有dry-run、backup、roundtrip与rollback |
| ACCEPTANCE-ARCHIVE-P1-058 | 无当前qualification consumer边界 | Editor/Hub/release/dashboard只消费EvidenceSet，不扫描Markdown |
| ACCEPTANCE-ARCHIVE-P1-059 | 无参考引擎式test definition/result分层 | TestInfo、RunReceipt、ArtifactSet、ArchiveRecord独立身份 |
| ACCEPTANCE-ARCHIVE-P1-060 | 性能/画质片段可能被当最终基线 | 只链接Tooling07同workload paired benchmark/reference registry |

## 12. P2：可维护性与可读性

| ID | 改进项 |
|---|---|
| ACCEPTANCE-ARCHIVE-P2-001 | archive catalog按domain/日期/计划生成导航，不手工维护100条列表 |
| ACCEPTANCE-ARCHIVE-P2-002 | frontmatter scalar统一quote，避免`user:`隐式mapping |
| ACCEPTANCE-ARCHIVE-P2-003 | 路径统一仓库相对正斜杠，raw historical command放独立verbatim字段 |
| ACCEPTANCE-ARCHIVE-P2-004 | 状态显示使用固定label，不把复合句塞进enum |
| ACCEPTANCE-ARCHIVE-P2-005 | 长命令transcript折叠为artifact link与摘要，减少同文重复 |
| ACCEPTANCE-ARCHIVE-P2-006 | 统一Scope/Environment/Result/Decision历史展示模板 |
| ACCEPTANCE-ARCHIVE-P2-007 | 生成broken-link与successor建议报告，禁止静默自动改历史正文 |
| ACCEPTANCE-ARCHIVE-P2-008 | fixture目录增加case README与expected typed error名称 |
| ACCEPTANCE-ARCHIVE-P2-009 | fixture文件名表达producer/version/case，不只用`invalid` |
| ACCEPTANCE-ARCHIVE-P2-010 | corpus manifest按字节排序生成，保持LF、UTF-8、无BOM与终止换行 |
| ACCEPTANCE-ARCHIVE-P2-011 | hash使用统一小写SHA-256并由validator重算 |
| ACCEPTANCE-ARCHIVE-P2-012 | archive引用在渲染时显示Historical/Superseded水印 |
| ACCEPTANCE-ARCHIVE-P2-013 | canonical plan row提供反向历史链接，但不内嵌整篇旧结果 |
| ACCEPTANCE-ARCHIVE-P2-014 | 文档lint报告文件规模、绝对路径与重复字段趋势，不以行数作为功能门 |

## 13. 目标合同

### 13.1 AcceptanceArchiveManifest

每条记录包含`record_id`、`archive_schema_version`、`path`、`content_digest`、`domain_owner`、`plan_id`、`milestone_id`、`created_at`、`archived_at`、`archive_state`、`canonical_output_record`、`supersedes`、`source_identity`、`build_set_id`、`run_receipt_ids`、`artifact_tombstones`、`inbound_reference_count`和`disposition`。Manifest是archive治理真值，不把历史正文转换为current evidence。

### 13.2 TestInfo / RunReceipt / EvidenceSet

由Tooling10拥有test identity、selection、attempt与terminal result，由Tooling07拥有性能/画质artifact qualification。Archive只保存这些对象当时的ID和摘要；对象缺失或过期时显示unverifiable，不从命令文本重新推断pass。

### 13.3 SerializationFixtureCorpus

每个case登记`fixture_id`、schema ID、envelope/payload version、producer product/build/platform、raw byte digest、license/origin、expected parser outcome、expected migration chain、budget class和consumer test IDs。历史positive input immutable；negative corruption样本也记录故意破坏点。

### 13.4 单向依赖

Production/test owner可以依赖fixture corpus，但不得依赖acceptance archive。Canonical plan/evidence可以链接archive背景；archive不得改变plan状态、产品capability或required test结果。

## 14. 实施里程碑

### M0 · Truth Freeze

禁止新增acceptance Markdown；生成100篇只读inventory、全量inbound link、46次失效path与2个direct consumer清单。所有archive-derived current claim fail-close。

### M1 · Detach Test Consumers

把Python与Runtime的两处Markdown字符串断言迁到实际module/owner contract，验证archive rename/delete不改变测试结果。

### M2 · Archive Catalog 与 Link Migration

建立manifest，给100篇分配record ID/disposition/canonical output row；修复7个不存在target的调用方或记录terminal tombstone，更新`_archive` successor。

### M3 · Evidence Projection

接入Tooling10/07 schema，只从current run receipt生成计划状态；旧文本显示historical/unverifiable，禁止被Editor/Hub/release消费。

### M4 · Fixture Corpus Hardening

为6个样本补origin/digest/case manifest；从真实历史writer重新捕获或明确标记synthetic，扩充negative/future/budget矩阵。

### M5 · CI Gates

加入archive schema/link/direct-consumer-zero、fixture digest/parse/consumer coverage与migration roundtrip gate，结果机器可读。

### M6 · Retention 与 Release Qualification

Archive有可审计retention/tombstone；release只消费未过期EvidenceSet与冻结fixture compatibility matrix。

## 15. 资格门

1. `tests/acceptance`新增文件数保持0，除批准的manifest/index迁移产物。
2. 100篇archive全部有record ID、digest、state、owner、disposition。
3. Archive Markdown direct code/test consumer数量为0。
4. 历史文本修改、重命名或移除不改变任何production/test行为结果。
5. 现存61篇入站archive与39篇无入站archive均有明确处理结果。
6. 7个不存在target全部修复或替换为typed tombstone。
7. 46次frontmatter失效路径全部分类，不能把deleted误映射到任意同名文件。
8. `user:`等frontmatter值按schema保持string，0 implicit mapping。
9. Archive status只能是生命周期枚举，不能承载test outcome复合句。
10. Current qualification只来自source/build-bound RunReceipt/EvidenceSet。
11. 每次required run记录discovered/selected/executed/omitted/ignored与terminal outcome。
12. 绝对target/temp路径不参与reproducible command identity。
13. 6个fixture均有producer/origin、schema/version、digest、expected outcome。
14. 旧fixture byte变更会在CI fail，新增历史case使用新ID。
15. project/scene/reflection migration对真实旧样本执行load/migrate/save/reload。
16. future/invalid输入在分配或业务decode前按预算typed拒绝。
17. Interface、Runtime、Hub对同一project fixture给出一致version/outcome。
18. Archive link、fixture manifest和corpus digest validator输出machine-readable result。
19. Editor/Hub/release/dashboard不扫描历史Markdown推导capability/pass。
20. `git diff --check`、frontmatter schema、链接、LF/BOM与索引同步全部通过。

## 16. 本轮验证与限制

本轮逐文件统计100篇acceptance与6个fixture，解析41份YAML frontmatter、4份TOML和2份JSON；核对metadata section、状态词、命令标记、绝对路径、frontmatter entry、入站引用、direct consumer与fixture consumer。静态读取Unreal Automation、Bevy CI、Fyrox CI、Godot test runner和Unity Graphics Yamato/TestProject作为边界参考。

本轮没有修改production、tests、fixtures、workflow或历史acceptance正文，也没有重跑当前source条件未变化的Editor/WOC失败lane。Fixture“当前能被parser读取”不等于Rust迁移test在当前workspace通过；既有compile blocker仍需其owner修复后再执行。本篇产出只是review与重构计划，所有P0/P1/P2仍为pending。
