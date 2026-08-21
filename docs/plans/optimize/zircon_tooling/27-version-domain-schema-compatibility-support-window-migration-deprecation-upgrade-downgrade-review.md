---
related_code:
  - Cargo.toml
  - zircon_runtime_interface/src/version.rs
  - zircon_runtime_interface/src/serialization/payload_header.rs
  - zircon_runtime_interface/src/serialization/schema_id.rs
  - zircon_runtime_interface/src/serialization/versioned_schema.rs
  - zircon_runtime_interface/src/serialization/load.rs
  - zircon_runtime_interface/src/serialization/write.rs
  - zircon_runtime_interface/src/serialization/migration/chain.rs
  - zircon_runtime_interface/src/serialization/migration/error.rs
  - zircon_runtime_interface/src/serialization/migration/execute.rs
  - zircon_runtime_interface/src/serialization/migration/step.rs
  - zircon_runtime_interface/src/serialization/migration/validate.rs
  - zircon_runtime_interface/src/project/manifest_summary/migration.rs
  - zircon_runtime_interface/src/project/manifest_summary/parse.rs
  - zircon_runtime_interface/src/project/manifest_summary/summary.rs
  - zircon_runtime_interface/src/hub_protocol/protocol_version/mod.rs
  - zircon_runtime_interface/src/export/preset.rs
  - zircon_runtime_interface/src/ui/template/asset/schema/policy.rs
  - zircon_runtime_interface/src/ui/template/asset/schema/report.rs
  - zircon_runtime/src/asset/project/manifest/project_manifest.rs
  - zircon_runtime/src/asset/migration/mode.rs
  - zircon_runtime/src/asset/migration/options.rs
  - zircon_runtime/src/asset/migration/report.rs
  - zircon_runtime/src/asset/migration/run.rs
  - zircon_runtime/src/asset/migration/scan.rs
  - zircon_runtime/src/asset/migration/sidecar.rs
  - zircon_runtime/src/asset/migration/transaction.rs
  - zircon_runtime/src/asset/migration/transaction/journal_owner.rs
  - zircon_runtime/src/asset/migration/transaction/recovery.rs
  - zircon_runtime/src/scene/dynamic_scene/document/schema.rs
  - zircon_runtime/src/scene/dynamic_scene/document/migration/mod.rs
  - zircon_runtime/src/scene/reflect/json_document/schema.rs
  - zircon_runtime/src/scene/reflect/json_document/migration.rs
  - zircon_runtime/src/ui/template/asset/schema/migrator.rs
  - zircon_runtime/src/script/vm/plugin/state_migration.rs
  - zircon_runtime/src/plugin/native_plugin_loader/compatibility.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_distribution_manifest.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/versions.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/versions/segments.rs
  - zircon_editor/src/core/settings/io.rs
  - zircon_editor/src/core/commandlet/runner.rs
  - zircon_editor/src/ui/workbench/project/constants.rs
  - zircon_editor/src/ui/workbench/project/editor_workspace_document.rs
  - zircon_editor/src/ui/workbench/project/editor_workspace_persistence.rs
  - zircon_hub/src/projects/install_receipt.rs
  - zircon_hub/src/projects/recent_project.rs
  - tools/cargo-zircon/src/plugin/scaffold/templates.rs
  - tools/zircon_export/plugin_validate_engine_version.py
  - tools/zircon_export/plugin_validate_distribution_engine_compat.py
  - tools/session_coordinator/migrations.py
  - examples/woc/native/crates/woc_contract_codegen/src/contract.rs
  - examples/woc/native/crates/woc_protocol/src/contracts.rs
  - examples/woc/scripts/woc_game/src/progression/talent_loadout_migration.zr
tests:
  - zircon_runtime_interface/src/serialization/tests/migration_contract.rs
  - zircon_runtime_interface/src/serialization/tests/migration_failure_contract.rs
  - zircon_runtime_interface/src/serialization/tests/load_contract.rs
  - zircon_runtime/src/asset/tests/migration/project_commandlet/transaction_recovery.rs
  - zircon_editor/src/core/commandlet/tests.rs
  - tools/session_coordinator/tests/test_migrations.py
  - tools/cargo-zircon/tests/plugin_commands.rs
  - examples/woc/native/crates/woc_contract_codegen/tests/contract_generation.rs
  - examples/woc/native/crates/woc_protocol/tests/protocol.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md
  - docs/plans/optimize/zircon_runtime/21-zr-language-parser-type-system-semir-bytecode-package-loader-vm-runtime-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md
  - docs/plans/optimize/zircon_runtime_interface/02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md
  - docs/plans/optimize/zircon_editor/13-layout-profile-workspace-state-docking-tab-window-restore-migration-review.md
  - docs/plans/optimize/zircon_editor/24-data-table-structured-data-schema-import-validation-save-game-slot-migration-platform-cloud-storage-authoring-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_tooling/04-reflection-derive-script-host-macros-schema-codegen-review.md
  - docs/plans/optimize/zircon_tooling/09-release-channel-artifact-repository-install-update-rollback-operations-review.md
  - docs/plans/optimize/zircon_tooling/12-top-level-acceptance-archive-serialization-fixture-provenance-currentness-migration-review.md
  - docs/plans/optimize/zircon_tooling/20-cargo-package-workspace-feature-dependency-target-graph-build-receipt-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Serialization/CustomVersion.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Serialization/CustomVersion.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/UObject/PackageFileSummary.h
  - dev/UnrealEngine/Engine/Source/Programs/UnrealBuildTool/Configuration/Descriptors/PluginDescriptor.cs
  - dev/godot/core/io/resource_format_binary.h
  - dev/godot/core/io/resource_format_binary.cpp
  - dev/Fyrox/fyrox-core/src/visitor/mod.rs
  - dev/Fyrox/fyrox-core/src/visitor/reader/binary.rs
  - dev/bevy/_release-content/migration_guides.md
  - dev/bevy/_release-content/migration_guides_template.md
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Core/Migration/MigrationDescription.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Core/Migration/MigrationStep.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Tests/Editor/MigrationTests.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Lighting/ProbeVolume/ProbeVolume.Migration.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 27 · Version Domain、Schema Compatibility、Support Window、Migration、Deprecation、Upgrade 与 Downgrade 审查

## 1. 结论

Zircon并不是完全没有版本化与迁移基础。根workspace用`workspace.package.version = 0.1.0`并由160份Cargo manifest中的157份继承；Runtime Interface已有`SchemaId + schema_version` envelope、future-version拒绝、完整线性迁移链校验和typed error；Project Manifest已到format v2并保留可选`engine_version_req`；Dynamic Scene、Reflect JSON、Editor Settings和Export Preset实现`VersionedSchema`；UI asset migrator返回局部migration step report；Runtime asset migration更具备DryRun/Apply、全量preflight、formal reader复验、journal、rollback、recovery与多处fault injection；Session Coordinator数据库已经执行65个单调migration并拒绝未来schema。这些是真实工程基础，后续必须保留。

问题是这些基础没有共享版本语义。当前production-like SourceSet中有1,505处版本字段信号分布于498文件，兼容性信号564处/136文件，迁移信号1,029处/212文件；但没有canonical `VersionDomainId`、版本向量、`CompatibilityDecision`、支持窗口、`MigrationPlan/Receipt`或`DeprecationRegistry`。同一个裸`u32`可能表示runtime DLL ABI、API table、Hub协议、项目格式、资源格式、UI source schema、script state、render artifact、数据库schema或工具report schema；同一个`0.1.0`又同时充当Cargo crate版本、engine产品版本和plugin兼容判定输入。局部检查正确，不等于跨产品组合正确。

具体断点已经可复核。Native plugin loader用手写major/minor/patch parser处理`engine_compat`，另一个runtime plugin validator又维护不同的SemVer规则；39份plugin manifest中的41个distribution声明全部固定`abi_version = 3`和`>=0.1, <0.2`，却没有BuildSet/target/profile验证矩阵支撑这整个区间。Project Manifest缺失format字段时默认v1，v1到v2只补字段，没有SavedByBuild、CompatibleWithBuild、schema向量或migration history；Editor workspace对format只做精确相等，旧版本直接不可恢复；Settings把version zero登记成“migration step”但实际强制拒绝。`MigrationChain`要求从0到current每一步永久存在，却没有minimum readable/writable version、跳跃/合并、降级或长期归档reader合同。

版本退出治理尤其薄弱。本轮对九个产品/代码family的13,093个production-like tracked文件、约2,054,144行做词法路由，没有找到任何Rust `#[deprecated]`；精确`deprecated/obsolete`只有20处/14文件且多数是maturity枚举、测试名或普通描述，而`legacy`仍有735处/123文件。也就是说工程持续积累兼容分支，却没有deprecated-since、replacement、removal milestone、usage telemetry、warning budget和最终删除门。

本篇不重复Interface02的serialization wire/unknown-field细节、Runtime04/05的资源场景迁移、Editor13/24的布局与SaveGame流程、Plugins01的ABI admission、Tooling04的schema codegen、Tooling09的release update或Tooling12的历史fixture。本篇拥有O03的跨产品Version Domain、Compatibility Decision、Support Window、Migration Receipt、Deprecation Registry与BuildSet-bound资格；局部owner继续实现adapter。**没有新增P0，登记40项P1和12项P2。**

## 2. 审查边界与Evidence

| Evidence | 本轮结果 |
|---|---|
| E1 production-like inventory | 9个family，13,093个tracked source/config/document文件，约2,054,144行；排除显式target/node_modules/vendor/generated/gen/dist/fixture/test/bench目录与常见test文件名 |
| E2 version signal routing | version field 1,505处/498文件；compatibility 564处/136文件；migration/upgrade/downgrade 1,029处/212文件；future/newer version 16处/9文件 |
| E3 legacy/deprecation routing | legacy 735处/123文件；精确deprecated/obsolete 20处/14文件；Rust `#[deprecated]`为0 |
| E4 common-contract exact search | `VersionDomain`、`CompatibilityDecision`、`MigrationReceipt`、`SupportWindow`、`DeprecationRegistry`合计0处 |
| E5 Cargo/package identity | 160份Cargo manifest：157份继承workspace version、3份显式version、0份缺失；继承一致是基础，不等于产品兼容矩阵 |
| E6 plugin range | 39份plugin manifest中41个distribution ABI/engine范围字段全部固定为ABI 3与`>=0.1, <0.2` |
| E7 migration control flow | 已读shared serialization、project/scene/UI/script、asset transaction、plugin compatibility、Editor persistence、Hub state、Coordinator DB与WOC contract代表路径 |
| E8 dynamic validation | 未执行；本篇是review-only，既有Editor/Hub/WOC/plugin构建与测试阻断未变化 |
| Currentness | revision `ae2be3d865a937b9ed368bf965592045346c64e3`，branch `main`；31个关键源文件clean，fingerprint `4ba51804ee6005ecd4e050ef332d21b46d695e0eddf00e387a7cb6549632df42` |

词法统计只用于owner路由，不作以下错误外推：

1. `version`字段多不等于兼容性成熟，也不等于这些字段都有问题。
2. `legacy`可能表示合法历史reader、命名兼容或测试fixture，不能机械删除。
3. `migration`也会命中数据库、引用修复、API指南和普通conversion；只有逐读控制流后才登记finding。
4. 根workspace版本一致不证明engine、editor、runtime DLL、plugin、project、artifact和protocol能按相同SemVer演进。
5. 参考引擎机制只提供结构约束，不证明其全部兼容决策、性能或历史包袱都应复制。

## 3. 必须保留的工程基础

### 3.1 Shared serialization已有正确的schema入口

`PayloadHeader`把`SchemaId`与`schema_version`分开，text/binary reader先校验schema和future version，再执行完整migration chain；current version也验证chain完整性。`serde(deny_unknown_fields)`用于若干canonical DTO。这是公共wire层的正确起点，不应退回“字段能serde就算兼容”。

### 3.2 Asset migration已有工程级事务雏形

Asset migration区分DryRun与Apply，先构建安全inventory、预检sidecar和resolver，再用formal reader复验输出；apply路径有staging、backup、journal、rollback、pending recovery及commit/stage/restore/interruption fault injection。它应成为O05 migration executor的局部参考实现，而不是被一个抽象“全局迁移器”推倒重写。

### 3.3 Coordinator数据库迁移具备单调marker和未来拒绝

Coordinator维护1到65的migration表，在transaction中记录applied marker，并对高于当前版本的数据库fail-close；涉及VACUUM/compaction的特殊步骤还分开处理物理事务。这是工具数据库的局部authority，后续需要补migration digest、版本支持策略和receipt，不应把它强行塞进资源serde链。

### 3.4 UI与VM state已经认识到迁移需要报告和type identity

UI migrator记录source kind、source version与执行步骤；VM state migration使用reflected type registration、type hash、rename和schema version。两者说明“只改顶层format字段”不足。公共合同应复用其report/type identity形状，并保留各自domain语义。

### 3.5 Workspace继承消除了大部分crate version漂移

157/160份Cargo manifest继承根version，至少避免核心workspace成员手工散落版本。后续要把crate package version与EngineBuildId、ProductVersion、ABI/API/Protocol/Schema version明确分层，而不是取消继承。

## 4. 已确认的当前版本与迁移断点

### 4.1 Engine version实为runtime crate package version

Native compatibility直接读取`env!("CARGO_PKG_VERSION")`。今天所有核心crate继承同一workspace值，所以表面一致；但该值没有channel、commit、BuildSet、toolchain、target、feature/profile或compatible-with identity，不能承担可安装Engine Build或Project reader资格。

### 4.2 Plugin兼容解析存在双实现和区间自认证

Native loader接受major.minor[.patch]与逗号连接比较器，忽略SemVer prerelease/build语义，也不支持OR/range normalization；runtime package validator则要求完整三段数字并维护另一套规则。41个distribution声明统一覆盖整个0.1 minor range，但没有逐BuildSet、target、ABI table、capability和behavior schema验证记录。

### 4.3 Project Manifest只有单一format数字

缺失`format_version`被当作v1，v1到v2只写入current number并补`engine_version_req/asset_roots/settings`。Manifest没有SavedByBuild、CompatibleWithBuild、minimum reader/writer、子schema向量、plugin compatibility snapshot、migration chain digest或已应用步骤历史；optional engine requirement也没有统一parser/decision receipt。

### 4.4 Editor persistence对旧版本策略分裂

Workbench workspace只接受`format_version == EDITOR_PROJECT_FORMAT_VERSION`，旧版本产生字符串diagnostic并丢弃restore；Editor Settings虽然挂到shared migration chain，却把version zero作为retired error，要求用户重建。Layout具体迁移由Editor13拥有，本篇只登记“产品必须声明支持窗口与可恢复/不可恢复原因”。

### 4.5 Linear chain没有支持窗口与归档策略

`MigrationChain`用`steps[version as usize]`执行0..target的连续步骤，并要求step数量严格等于current version。该设计适合小型单schema链，却把“永远保留从0开始的每一步”当作唯一模式；没有minimum readable version、snapshot checkpoint、skip edge、long-term reader、downgrade或外部migration provider。

### 4.6 局部report不能拼成产品migration receipt

UI有step report、asset migration有change/issue/metric report、serialization只返回`migrated_from`、Coordinator返回latest整数、workspace restore返回diagnostic。它们没有共同source/target domain、input/output digest、BuildSet、step implementation digest、transaction ID、backup/recovery位置、duration/budget、warning与terminal outcome。

## 5. P1：Version Domain、Identity 与 Compatibility Decision

### TOOL-VERSION-P1-001 · 没有canonical Version Domain Registry

Engine、Product、Crate、ABI、API table、Protocol、Schema、Artifact、Database、Policy和Evidence版本没有稳定domain ID、owner、value type、comparison rule和serialization codec。

### TOOL-VERSION-P1-002 · 裸数字版本可以跨domain误比较

大量`u16/u32/u64`字段只靠变量名区分；没有type-level `AbiVersion/SchemaVersion/ProtocolVersion`或domain-qualified vector，日志、DTO和工具容易比较两个语义无关的`3`。

### TOOL-VERSION-P1-003 · Engine Build与Cargo package version混用

`CARGO_PKG_VERSION`无法表达commit、channel、toolchain、target、features、runtime dependency与artifact digest；plugin/project兼容不能只绑定crate发布号。

### TOOL-VERSION-P1-004 · 没有SavedBy与CompatibleWith双身份

持久artifact不能同时记录实际producer build和声明的minimum compatible build；hotfix、backport、branch build与format-compatible rebuild无法可靠区分。

### TOOL-VERSION-P1-005 · Compatibility没有typed decision

各入口返回bool、Option<String>、enum error或直接丢弃对象；缺`Compatible/Incompatible/RequiresMigration/ReadOnly/Quarantined/Unknown`、稳定reason code与remediation。

### TOOL-VERSION-P1-006 · 支持窗口没有single source of truth

minimum readable/writable、latest writer、LTS archive reader、plugin ABI grace、protocol overlap和数据库升级跨度没有机器可读policy及owner。

### TOOL-VERSION-P1-007 · 版本向量不能表达组合产品

项目、插件、runtime DLL、script package、resource schema、content BuildSet和render artifact各自变化；单个engine SemVer无法证明组合closure兼容。

### TOOL-VERSION-P1-008 · Unknown/Future/TooOld行为不一致

shared serialization拒绝future，workspace丢弃任意非current，settings拒绝legacy，resource owners各自处理；没有统一fail-close、read-only、quarantine或external upgrader策略。

### TOOL-VERSION-P1-009 · Compatibility decision不绑定source与generation

判定结果没有绑定input digest、manifest revision、plugin artifact digest、BuildSet、policy generation和consumer generation，检查后替换文件可复用旧结论。

### TOOL-VERSION-P1-010 · 兼容错误缺稳定跨产品投影

Editor、Hub、CLI、runtime loader和release tool不能共享reason code、affected domain、found/supported range、migration availability、backup requirement与用户可执行修复。

## 6. P1：Schema Catalog、Reader/Writer 与 Persistent Composition

### TOOL-VERSION-P1-011 · SchemaId没有中央catalog与碰撞门

`SchemaId`是字符串常量，但没有全仓注册、owner、namespace reserve、alias/retirement、duplicate/collision和generated catalog validation。

### TOOL-VERSION-P1-012 · VersionedSchema生产采用面过窄

全仓production实现集中在Editor Settings、Dynamic Scene、Reflect JSON和Export Preset四类文件；大量format/schema字段没有进入相同reader/writer/future-version合同。

### TOOL-VERSION-P1-013 · 嵌套schema没有版本向量

Project Manifest的asset library、plugins、scripts、export profiles及Scene内组件只由顶层数字覆盖，子domain独立演进与部分兼容无法表达。

### TOOL-VERSION-P1-014 · Unknown-field保留策略未分domain

部分DTO `deny_unknown_fields`，部分value migration保留未触碰字段，部分serde结构会丢未知数据；没有Strict/PreserveOpaque/ExtensionBag/ForwardReadOnly分类与roundtrip证据。

### TOOL-VERSION-P1-015 · Reader与Writer能力没有分开声明

“能读current”常被当作“能写current且旧reader可读”；缺reader range、writer targets、canonical writer、downlevel writer和read-only compatibility声明。

### TOOL-VERSION-P1-016 · Schema evolution rule未机器校验

rename、optional/additive、required、enum variant、numeric widening、identity change、semantic default和field reuse没有统一compat classification与codegen gate。

### TOOL-VERSION-P1-017 · Persistent enum/flag没有reserved与unknown策略

ABI、protocol、artifact和serde enum增加值时，各owner自行选择reject/default/ignore；没有reserved range、unknown carrier与old-reader matrix。

### TOOL-VERSION-P1-018 · Schema与reflection/codegen未共享compat hash

Tooling04的type/field identity、VM type hash、serialization SchemaId和plugin behavior schema没有统一canonical descriptor与hash provenance。

## 7. P1：Migration Planning、Execution 与 Recovery

### TOOL-VERSION-P1-019 · 没有全局Migration Catalog

linear serde chain、asset project migration、UI migrator、VM state mapper、workspace loader和65步SQLite migration无法按domain/source/target/owner/implementation统一发现与审计。

### TOOL-VERSION-P1-020 · Migration edge只支持相邻线性前进

公共chain不支持checkpoint、skip edge、branch merge、external adapter或特定source-to-target optimized path；版本增长会永久增加启动验证和维护成本。

### TOOL-VERSION-P1-021 · Migration step缺实现身份与不可变digest

函数指针或Python callable没有稳定StepId、source/build digest、tool version、dependency、supersedes关系；同版本号下改变实现无法在receipt中识别。

### TOOL-VERSION-P1-022 · 没有公共Migration Plan

执行前不能列出所有domain、步骤、输入、预计写集、磁盘/内存/时间预算、backup、锁、停机要求、不可逆点和依赖顺序。

### TOOL-VERSION-P1-023 · 没有公共Migration Receipt

各局部report不能证明source/target、input/output digest、step sequence、warning、transaction、recovery、BuildSet和terminal outcome属于同一次操作。

### TOOL-VERSION-P1-024 · 幂等性与确定性不是统一门

Unity参考测试显式验证重复migration不再改变对象；Zircon只有部分owner测试。所有可重试/恢复步骤都应声明并证明idempotent或exactly-once boundary。

### TOOL-VERSION-P1-025 · Cross-resource依赖迁移没有DAG

Project、sidecar、scene、script state、plugin data与derived artifact可能互相引用；除asset局部事务外，没有全产品dependency order、cycle diagnostic和atomic publication generation。

### TOOL-VERSION-P1-026 · 事务能力没有按migration风险分级

有的value在内存转换，有的atomic replace，有的多文件journal，有的数据库transaction，有的直接拒绝；缺Pure/Staged/Transactional/Offline/External五类执行合同。

### TOOL-VERSION-P1-027 · 不可逆步骤与backup policy未声明

字段丢弃、identifier remap、数据库compaction、旧文件删除和plugin state rewrite没有统一irreversible marker、backup retention、free-space admission与restore验证。

### TOOL-VERSION-P1-028 · Downgrade与branch rollback没有政策

系统普遍只支持向前迁移；release rollback、团队切branch、旧Editor打开新项目、plugin回退与server/client版本交错时，没有只读、downlevel export或明确不可降级边界。

## 8. P1：Product、Plugin、Release 与 Runtime Coordination

### TOOL-VERSION-P1-029 · Project Manifest不记录完整兼容closure

缺SavedByBuild、CompatibleWithBuild、required product role、plugin/artifact/schema vector、migration history和last-known-good reader；optional字符串requirement不足以复现decision。

### TOOL-VERSION-P1-030 · Plugin engine range由声明者自认证

统一`>=0.1,<0.2`没有对应validated BuildSet矩阵、target/feature/profile、ABI table hash、behavior schema和capability contract证据。

### TOOL-VERSION-P1-031 · Plugin兼容parser与validator双真源

Native loader和runtime package validation分别解析版本；Tooling export还有Python实现。接受集合、诊断和未来SemVer扩展会漂移。

### TOOL-VERSION-P1-032 · ABI/API/behavior/state版本没有原子协商

Native ABI、descriptor ABI、runtime API、host callback、plugin behavior与state schema分别检查；没有单一negotiation receipt证明最终启用的组合。

### TOOL-VERSION-P1-033 · Hub/Editor/App没有共同upgrade coordinator

Hub安装/打开项目、Editor加载/迁移、App启动runtime与CLI commandlet各自决定何时升级；没有single-writer lease、preview/consent、product shutdown与resume handoff。

### TOOL-VERSION-P1-034 · Release rollback不约束data downgrade

Tooling09拥有binary/install rollback，但当前版本合同不能证明旧binary可读取新Editor/新runtime已写数据；rollback可能恢复程序而留下不可读项目或数据库。

## 9. P1：Deprecation、Legacy Debt 与 Qualification

### TOOL-VERSION-P1-035 · 没有Deprecation Registry

API、schema field、protocol message、plugin capability、CLI flag和artifact format没有deprecated-since、replacement、owner、removal release、compat window与status registry。

### TOOL-VERSION-P1-036 · Rust公开API没有compiler-aware deprecation

tracked产品源码未发现`#[deprecated]`；破坏性变化无法通过编译器/IDE给出since、note和replacement，也没有项目级例外策略。

### TOOL-VERSION-P1-037 · Legacy分支没有退出预算

735处/123文件的production-like legacy信号没有统一owner、usage count、fixture、last supported build、removal gate与过期告警，兼容成本只增不减。

### TOOL-VERSION-P1-038 · Breaking change没有强制migration guide

与Bevy参考流程相比，PR/CI没有把public API/schema/protocol/CLI破坏变更绑定可搜索的what/why/how、old/new示例和release note。

### TOOL-VERSION-P1-039 · 缺old-reader/new-writer兼容矩阵

测试多验证current reader、单步migration或局部fixture；没有按BuildSet生成N-2/N-1/N/current/future、read/write/migrate/read-only/reject的跨domain矩阵。

### TOOL-VERSION-P1-040 · Migration资格不绑定真实产品数据与规模

没有代表大项目、百万对象、长生命周期数据库、第三方plugin、崩溃恢复和性能预算的source/build-bound ValidationSet，局部unit test不能支撑发布支持窗口。

## 10. P2：长期演进能力

### TOOL-VERSION-P2-001 · 版本协商与rolling upgrade

支持server/client、Editor collaboration、Hub agent和out-of-process plugin在重叠窗口内协商capability/schema，并证明mixed-version期间ordering与state一致。

### TOOL-VERSION-P2-002 · Long-term Archive Reader

建立与主产品解耦、可冻结依赖和沙箱执行的历史项目/资产reader，输出current neutral IR与完整provenance。

### TOOL-VERSION-P2-003 · Generated compatibility adapter

对可证明的additive/rename/default变化生成adapter、matrix test和migration guide草稿；semantic migration仍必须人工owner审核。

### TOOL-VERSION-P2-004 · Migration semantic diff与explain

在执行前后提供typed data diff、引用变化、默认注入、丢失字段、大小/性能变化和不可逆原因，而不是只显示“version changed”。

### TOOL-VERSION-P2-005 · Remote Schema Registry federation

为组织、marketplace和第三方plugin提供签名schema/version catalog、namespace delegation、revocation和离线snapshot，不让远程registry成为启动单点。

### TOOL-VERSION-P2-006 · Multi-version plugin sidecar host

在不可信或旧ABI插件无法进程内兼容时，通过out-of-process adapter、版本固定host和typed bridge保留有限能力与隔离。

### TOOL-VERSION-P2-007 · Fleet migration rollout

对团队/CI/构建机/服务实例提供canary、wave、pause、rollback、compat telemetry与数据所有者审批，不把单机成功外推到fleet。

### TOOL-VERSION-P2-008 · Migration cost model与adaptive scheduling

按对象/字节/依赖边/IO/CPU/GPU估算成本，在预算内分批、后台或停机执行，并输出预测与实际差异。

### TOOL-VERSION-P2-009 · Branch-aware downgrade/export

支持开发分支间可声明的downlevel export、cherry-pick migration和conflict explanation；无法无损降级时明确只读或复制项目策略。

### TOOL-VERSION-P2-010 · Privacy-preserving compatibility telemetry

统计真实版本分布、legacy reader命中、失败reason与迁移耗时，同时只上传聚合、脱敏、可选择数据并绑定policy generation。

### TOOL-VERSION-P2-011 · Migration proof corpus fuzzing

对schema evolution、unknown field、truncation、duplicate、恶意深度、跨版本组合与fault point执行grammar/structure-aware fuzz和differential reader验证。

### TOOL-VERSION-P2-012 · Verified skip migration与snapshot compaction

当线性链过长时生成经历史corpus证明等价的checkpoint/skip edge，保留原路径作oracle并记录semantic digest，不能仅为速度删除历史步骤。

## 11. 参考实现对照

| 参考 | 可吸收机制 | Zircon差距 | 不照搬内容 |
|---|---|---|---|
| Unreal | `FCustomVersion`以GUID+version+friendly name+validator注册；Package summary同时保存file/custom/SavedBy/CompatibleWith版本；plugin descriptor区分file/version/engine/deprecated engine信息 | 缺version domain/container、producer与compatible双身份、全局compare/validator及package级版本向量 | 不复制UE历史全局版本枚举、包格式包袱或所有licensee规则 |
| Godot | binary resource header写engine major/minor与format version，reader在入口拒绝未来format/major，并按format分支读取 | Zircon许多artifact只有局部数字，reader策略不统一；Project/Artifact未统一记录producer engine与format | 不复制Godot单一资源格式或只按major判断的全部政策 |
| Fyrox | Visitor用结构化region/field树和reader/writer owner承载容错读取，字段缺失可由类型逻辑处理 | Zircon shared envelope是更明确基础，但采用面窄且没有跨domain catalog/support window | 不把Visitor弱schema或树格式当作唯一wire方案 |
| Bevy | breaking change PR必须产出migration guide；`#[deprecated]`是IDE/编译器入口但不能替代指南 | Zircon没有compiler deprecation、migration guide gate与release级退出流程 | 不把Bevy每个major的API破坏节奏直接套到稳定引擎产品 |
| Unity Graphics | enum版本、排序migration step、重复执行幂等测试、object dirty/save集成；具体组件保留obsolete字段到迁移完成 | Zircon局部step/report存在，但无公共step identity、产品transaction与deprecation removal gate | 不复制Unity对象序列化、Editor dirty机制或逐组件Awake迁移时机 |

## 12. 目标架构

建议公共控制链为：

`VersionDomain Registry -> VersionVector/BuildIdentity -> SupportWindow Policy -> CompatibilityDecision -> MigrationCatalog -> MigrationPlan -> domain executor -> MigrationReceipt -> qualification/deprecation telemetry`

最小公共schema：

1. `VersionDomainDefinition`：domain ID、owner、value type、comparison、current/min reader/min writer、unknown/future policy、registry generation。
2. `VersionVector`：EngineBuild、ProductRole、ABI/API/Protocol/Schema/Artifact/Plugin/Policy项及各自digest，不压成单一整数。
3. `CompatibilityDecision`：input identity、consumer identity、policy generation、outcome、reason code、required migration/read-only/quarantine与expiry。
4. `MigrationStepDefinition`：StepId、domain、source/target selector、implementation BuildSet/digest、preconditions、write set、budget、reversibility与idempotency。
5. `MigrationPlan`：DAG、locks、backup、space/time estimate、irreversible points、preview diff和approval。
6. `MigrationReceipt`：transaction、input/output digest、executed steps、warnings、recovery state、duration/resources、terminal outcome和post-read proof。
7. `DeprecationRecord`：subject、since、replacement、support/removal milestone、usage evidence、exception和owner。

公共层只定义schema、决策与qualification，不执行所有domain迁移。Shared serde、asset transaction、SQLite、plugin state、Editor document和release installer继续各自执行，并适配同一plan/receipt。

## 13. 重构里程碑

### M0 · Inventory与Truth Freeze

- 生成tracked `VersionFieldInventory`，为每个version字段标注domain/owner/wire/persistent/ephemeral。
- 冻结新增裸`schema_version/format_version/protocol_version/abi_version`，未登记domain不得进入public/persistent边界。
- 禁止以`CARGO_PKG_VERSION`单独证明Engine Build兼容，禁止扩大plugin支持区间而无matrix evidence。

### M1 · Version Domain与Compatibility基础

- 建立VersionDomain Registry、typed version wrappers、EngineBuildId/CompatibleWithBuild和VersionVector。
- 收敛Rust/Python/plugin三套SemVer/compat parser到同一规范与golden corpus。
- 统一CompatibilityDecision、reason code及Editor/Hub/CLI/runtime投影。

### M2 · Schema Catalog与Reader/Writer政策

- 注册SchemaId、namespace、owner、current/min reader/min writer、unknown-field和extension policy。
- Tooling04从canonical schema IR生成compat hash/evolution diagnostics；Interface02实现wire adapter。
- 给nested Project/Scene/Plugin/UI/Script数据建立schema vector，不再让顶层数字掩盖子版本。

### M3 · Migration Catalog与Receipt

- 登记现有serde、asset、UI、VM、workspace、Coordinator与WOC migration为稳定StepId。
- 提供Plan/Preview/Receipt公共schema；Asset migration先适配而不重写其事务。
- 为pure/staged/transactional/offline/external五类executor建立一致preflight与terminal outcome。

### M4 · Product Upgrade Coordinator

- Hub/Editor/App/CLI共享project upgrade single-writer lease、preview/approval、shutdown/resume和LKG handoff。
- Release rollback先检查data downgrade/read-only能力，再允许切换binary。
- Plugin协商原子覆盖engine/ABI/API/behavior/state/capability vector。

### M5 · Deprecation与历史兼容退出

- 建立Deprecation Registry、compiler attribute/diagnostic、migration guide与exception expiry。
- 为735处legacy信号分owner并区分retain/archive/remove；没有usage/fixture/支持窗口的分支不得无限保留。
- 固定LTS archive reader与fixture provenance，Tooling12拥有历史corpus。

### M6 · Qualification与规模证明

- 构建N-2/N-1/current/future old-reader/new-writer矩阵和跨domain product scenarios。
- 执行power-loss、disk-full、cancel、wrong build、mixed plugin、malicious payload、large project与fleet canary。
- Compatibility/Migration Qualification必须绑定immutable BuildSet、corpus digest、environment、performance budget和receipt set。

## 14. 必测矩阵

| 维度 | 必测值 |
|---|---|
| Version relation | too-old、min-readable、N-2、N-1、current、future、unknown domain、invalid shape |
| Reader/writer | old-read-new、new-read-old、current rewrite、read-only、downlevel export、roundtrip unknown |
| Product vector | Engine/Editor/App/Hub、runtime ABI/API、plugin behavior/state、project/resource/UI/script、protocol/database |
| Migration path | adjacent、multi-hop、checkpoint/skip、missing/duplicate/out-of-order、irreversible、external provider |
| Transaction fault | preflight、stage、backup、commit-before-marker、marker-before-cleanup、rollback、restart recovery、disk full |
| Composition | nested schema mismatch、cross-resource dependency、cycle、plugin absent/revoked、mixed BuildSet |
| Deprecation | warning、replacement、exception expiry、usage zero、removal build、old fixture rejection |
| Scale | largest supported project、million-object state、long DB history、many plugins、cold/warm cost与bounded memory |

## 15. Owner路由

| 范围 | Canonical owner |
|---|---|
| Version Domain、VersionVector、SupportWindow、CompatibilityDecision、MigrationPlan/Receipt、Deprecation Registry与全局qualification | 本篇 / O03 |
| Serialization envelope、SchemaId、unknown fields、reader/writer与value migration实现 | Runtime Interface 02 |
| Resource/project/scene/asset transaction与reference migration | Runtime04/05及其源码owner |
| Runtime DLL/plugin ABI、foreign ownership、native admission | Interface01、Plugins01、Runtime07 |
| Reflection/schema IR/codegen/compat hash | Tooling04 |
| Editor layout、workspace与SaveGame/domain authoring migration | Editor13、Editor24及各Editor domain报告 |
| Release channel、install/update/rollback与binary/data协调 | Tooling09、Hub01 |
| Historical fixture、archive writer provenance与compat corpus | Tooling12 |
| Cargo package/version/feature/target graph | Tooling20 |
| Stable identity/schema generation与persistent handle | Runtime24 |

## 16. 完成定义

只有同时满足以下条件，才能把本篇从review转为implemented：

1. 所有public/persistent version字段都有VersionDomainDefinition与唯一owner，unknown domain fail-close。
2. EngineBuild、CompatibleWithBuild和VersionVector贯穿project/plugin/artifact/process/receipt，不再只依赖crate SemVer。
3. Reader/Writer/SupportWindow可机器查询，future/too-old/read-only/migrate/reject在所有产品投影一致。
4. 所有migration step有稳定ID、implementation digest、precondition、budget、reversibility/idempotency与owner。
5. 所有effectful migration先产生plan，完成后产生source/build-bound receipt，并通过post-read与restart验证。
6. Asset/DB/Editor/Plugin等局部executor保留其domain事务，但统一接入compatibility与receipt schema。
7. Release rollback在数据不兼容时fail-close或进入明确read-only/downlevel流程。
8. Deprecation Registry、`#[deprecated]`/diagnostic、migration guide、usage evidence和removal gate共同运行。
9. N-2/N-1/current/future、mixed product/plugin、fault/scale/fuzz矩阵进入required CI或release qualification。
10. 在真实项目、真实plugin、真实历史fixture与immutable BuildSet上证明正确性、恢复、预算和支持窗口。

在这些门成立前，Zircon可以说“若干schema已有局部migration”，不能说“项目、插件、资产、协议和产品升级已经工程级兼容”。
