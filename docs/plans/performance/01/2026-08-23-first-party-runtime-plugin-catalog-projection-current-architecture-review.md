---
related_code:
  - zircon_plugins/first_party_runtime_catalog/src
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_runtime/src/plugin/runtime_plugin/registration_report/plugin.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/registration
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/performance/02/2026-08-15-runtime-plugin-catalog-extension-bridge-current-architecture-review.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
  - docs/plans/zircon_plugins/11-plugin-call-bridge.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Modules/ModuleManager.cpp
tests:
  - current first-party runtime catalog 5 of 5 Rust files and 11 tests reviewed
  - App engine/editor projection and Runtime registration/catalog construction chains reviewed
  - plugin structure audit passed with zero schema, registration, capability and distribution violations
  - focused rustfmt 1.94.1 passed
  - current-source Cargo, startup allocation profile, WPR and power pending
doc_type: implementation-evidence
status: source_reviewed_structural_plan_dynamic_blocked
---

# First-party runtime plugin catalog投影复审（2026-08-23）

## 范围与当前性

已逐行复读`zircon_plugins/first_party_runtime_catalog/src/**`当前**5/5**个Rust文件、**1,239行、43,872 B、
11 tests**，path+NUL+raw bytes+NUL manifest为
`70ed00820b19b28802a974b8ace3e87d8cb46601c0cc555b70edcf37e423dee8`。其中生产owner只有
`lib.rs` **103行**，其余4个测试文件共**1,136行**。`Cargo.toml`、`lib.rs`、provider snapshot与runtime projection
已有其他Session加入UI document importer的未提交改动，本轮按current source复审并保留，不回退、不覆盖。

同时沿`zircon_app` config/profile/editor startup、`RuntimePluginRegistrationReport::from_plugin`和
`RuntimePluginCatalog::from_registration_reports`复核完整构建链。`tools/audit_plugin_structure.py --json --repo-root .`
以内存JSON运行通过：manifest **39/39**、runtime descriptor roots **29**、capability roots **16**，schema、generated
header、registration builder、single-source descriptor、capability、editor mirror、distribution dependency与compatibility
shim violation均为**0**，M1为`classified-and-clear`。这证明声明单源当前有效，不应靠删验证或绕过descriptor改善启动。

## 当前源码判定

### P0：每次manifest投影都重新构造并验证完整registration

`first_party_runtime_plugin_registrations_for_manifest`每次调用遍历S个selection、parse/normalize ID、HashSet去重并对每个
命中执行provider `plugin_registration()`。SDK/helper不是返回静态descriptor引用，而是新建plugin；
`RuntimePluginRegistrationReport::from_plugin`随后新建`RuntimeExtensionRegistry`、clone module descriptor、执行plugin
register、构造完整package manifest/project selection、注册shader与package contributions，并重新跑descriptor、manifest、
interface和system-anchor验证。R个命中provider的工作量因此至少为O(S + R*(descriptor + extensions + validation bytes))，
且返回值深拥有manifest、extensions和diagnostics。

App的config、runtime profile、engine entry和editor preparation都暴露可重复调用的投影入口；下游再以owned reports构造
capabilities、module selection和`RuntimePluginCatalogProjection`。前序PERF-MVP-629已在135/135文件范围确认同一
registrations在App bootstrap和dynamic session至少建立两次catalog。这里是那个重复authority的首方producer，不能
用单函数局部memo clone修复：cache `Vec<RuntimePluginRegistrationReport>`后返回clone仍会深复制所有rows，返回borrow又
无法跨现有ABI/lifecycle安全持有。

Plan02 M1/M5与Plugins01应建立process级`FirstPartyProviderCatalogGeneration`：每个compiled provider的descriptor、
validated registration、capability与factory slot只物化一次；project/profile投影只发布有序selection handles/ranges，
Runtime session和Editor共享同一Arc generation。项目差异只构建selection delta，catalog+project+target generation作为
plan key，不重新执行plugin register/validation。

### P0：线性feature分派不是首要复杂度，但缺少compiled slot

当前每个enabled selection最多穿过约15个cfg裁剪后的typed equality分支；S、P增长时为O(S*P)。HashSet clone用于把
alias/case规范化后的`RuntimePluginId`去重，外部serde manifest可直接含重复row，因此不能删除。先registration再去重也
会让duplicate重新构造宽report，明显更差。

终态应由generated provider table将canonical first-party ID映射到dense slot/factory，lookup均摊O(1)，unknown/open
第三方ID返回None；open string-newtype不要求首方compiled table也线性if。当前
`runtime_plugin_id_non_copy_contract_reaches_workspace_consumers`源码形状测试强制`seen.insert(runtime_id.clone())`、
typed if equality且禁止match，`runtime_projection.rs`又强制HashSet/Vec/for-loop源码文本。这些测试验证实现拼写而非
duplicate/order/unknown/alloc行为，会阻止slot/generation硬切；应以行为与counter替代，不能让测试形状定义架构。

### P1：测试体量与手写parser增加编译维护成本，但不是产品热点

测试代码占本目录**91.7%行数**。`tests.rs`维护一套约300行的简化TOML parser、字段投影和String/Vec构造，同时多个
测试复用一次Python audit但clone完整`std::process::Output`。这些只影响测试编译/执行，不应抢占MVP产品优化；
Plugins12后续应直接消费cargo-zircon生成的typed snapshot或标准TOML parser，并保留少量端到端parity门。当前审计已
GREEN，本轮不改验证代码，也不把test-only分配计入启动热点。

### 已确认正确边界

manifest target/enabled过滤发生在provider构造前；canonical ID去重也发生在registration前，避免duplicate宽构建。
result和seen按selection upper bound预分配，输出保留manifest order。current UI document importer接线具备feature gate、
provider snapshot和实际投影行为测试。上述行为必须在generation重构中保持，不能为了benchmark跳过validation、排序或
可选provider。

## Unreal源码依据

`PluginManager.cpp:2034-2080`的`ConfigureEnabledPlugins`在`PluginsToConfigure`非空时完成discover、target/program筛选、
mark/process/mount，成功后清空待配置集合；稳定后重复调用只返回已完成状态。`2884-2978`的phase load消费既有
`AllPlugins`与enabled标记，只对当前loading phase执行module load，并记录phase进度，不重新生成每个plugin descriptor。

`ModuleManager.cpp:992-1061`先`FindModule`，已加载模块直接返回；仅miss才进入Add/find/load/initialize。这些实现并非
Zircon数据结构模板，但明确支持“discovery/configuration generation一次发布，stable lookup消费slot，phase load不重建
catalog”。Zircon的Rust/plugin report可保留更强typed validation，同时把它移到provider generation构建期。

## 量化验收

矩阵为compiled providers 0/1/15/100、manifest rows 0/1/100/1k、duplicates/aliases 0/1/100%、profiles/projects
1/2/100、targets 3、cold/stable/reload。记录ID normalize/hash/branch/slot probes、plugin constructors、registration/
validation/catalog/projection builds、descriptor/manifest/extension clone+alloc bytes、locks、startup p50/p95、RSS与energy。

终态要求同一compiled generation provider construct/registration/validation **<=1/provider**，stable重复投影为0；project
selection近O(S)，provider lookup O(1)，duplicate registration build=0；同batch catalog authority=1，两个同target
project不互相驱逐；unknown第三方ID、manifest order、target/feature、diagnostics和UI document importer行为等价。

本轮没有在foreign-dirty catalog上做局部源码修改。focused rustfmt与plugin audit通过，但current-source Cargo、startup
allocator/WPR/power尚未执行；该非渲染catalog切片不使用RenderDoc。继续留`pending.md`，不迁入`review.md`、不提交
milestone、不发送完成企微。
