---
related_code:
  - zircon_runtime/src/builtin
  - zircon_runtime/runtime-feature-presets.toml
  - zircon_runtime/build.rs
  - zircon_runtime/src/core/framework/platform/runtime_target_mode.rs
  - zircon_runtime/src/core/framework/project/runtime_profile_id.rs
  - zircon_runtime/src/core/framework/project/project_plugin_manifest
  - zircon_runtime/src/plugin/runtime_profile
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog
  - zircon_runtime/src/plugin/extension_registry
  - zircon_runtime/src/dynamic_api/session/linked_plugins.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/plugins/groups.rs
  - zircon_app/src/plugins/groups/resolution.rs
tests:
  - zircon_runtime/src/builtin/runtime_modules/tests
  - zircon_runtime/src/builtin/runtime_modules/assembly/registration_inputs/tests.rs
  - zircon_runtime/src/builtin/runtime_modules/ids/plugin_id.rs
plan_sources:
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md
  - docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md
  - docs/plans/optimize/zircon_runtime/21-zr-language-parser-type-system-semir-bytecode-package-loader-vm-runtime-review.md
  - docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/ModuleDescriptor.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/ModuleDescriptor.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Modules/ModuleManager.cpp
  - dev/bevy/crates/bevy_app/src/plugin.rs
  - dev/bevy/crates/bevy_app/src/plugin_group.rs
  - dev/bevy/crates/bevy_app/src/app.rs
  - dev/godot/modules/register_module_types.h
  - dev/godot/core/extension/gdextension_manager.h
  - dev/godot/core/extension/gdextension_manager.cpp
  - dev/godot/core/extension/gdextension_library_loader.cpp
  - dev/godot/core/extension/gdextension.h
  - dev/godot/core/extension/gdextension.cpp
  - dev/Fyrox/fyrox-impl/src/plugin/mod.rs
  - dev/Fyrox/fyrox-impl/src/plugin/dylib.rs
  - dev/Fyrox/fyrox-impl/src/engine/executor.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/package.json
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/package.json
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Unity.RenderPipelines.Core.Runtime.asmdef
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Unity.RenderPipelines.HighDefinition.Runtime.asmdef
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 42 · Builtin Runtime Module / Profile / Target / Feature / Extension Assembly 工程化差距

## 1. 结论

`zircon_runtime::builtin`不是一次性拼接脚本。当前实现已经把6个内建Profile和12个内建模块写入单一TOML源，由`build.rs`生成typed preset；Profile装配会从同一target candidate set补齐内建依赖闭包，最终调用Core模块拓扑排序；插件可用性区分Available、Linked、NativeDynamic、Externalized、Stub、Target/Maturity blocked和MissingRequired；load report也已把Core、Feature和Asset Importer错误结构化。这些基础应保留。

但它还不是工程级Composition Compiler。当前至少有四套相互重叠的真值：`runtime-feature-presets.toml`描述Profile，`builtin/runtime_modules`重新决定target/module/plugin，`RuntimePluginCatalog`另行完成feature/provider/extension merge，`zircon_app`又先构造一个内建PluginGroup再覆盖模块、追加Editor和plugin descriptor，并再次构建catalog。它们没有共享不可变plan、generation、BuildSet或commit receipt。

最明确的正确性断路位于registration过滤。`active_plugin_registration_refs()`只检查registration自身的`project_selection.enabled/supports_target`，完全不检查本次装配的project manifest。因此调用者只要传入默认enabled的registration，即使项目未选择或已禁用该插件，其asset importer和render extension仍会进入Asset/Graphics模块。Feature路径也只按available feature id匹配registration；同一feature存在多个provider时，所有同ID registration都可被拍平注入，而`RuntimePluginCatalog`的正式project extension report只选择一个provider。一次启动由此可能拥有两份不同的extension truth。

Profile合同也没有真正执行。`required_capabilities`只被生成和序列化，没有进入builtin assembly的任何admission；`#[cfg]`会从`BuiltinRuntimeModuleId`枚举和生成的Profile成员中直接删除Graphics/Script，使同一序列化schema和同一Profile名称随build feature改变；Target快捷入口则不选择Profile，Client/Editor装配全部candidate，Server固定保留Input却禁止Script。这里的target gate是少量条件分支，不是由产品角色、平台、capability、loading phase和artifact closure共同求解的装配计划。

本报告新增 **0项P0、52项P1、14项P2和42个资格门**。Plugins06已经拥有required selection、标准Profile与compiled provider不闭合等shipping P0；App01拥有产品composition receipt和角色/停机P0；Runtime01拥有模块activation/teardown P0。本篇不重复计数，而负责把builtin/profile/target/feature/extension输入收敛为一份可验证、可提交、可观测的`RuntimeCompositionPlan`。

## 2. 审查边界、语料与 currentness

### 2.1 冻结语料

| 子域 | 文件 / 行 / bytes | test属性或宏 / ignored | 结论 |
|---|---:|---:|---|
| `zircon_runtime/src/builtin`完整目录 | 30 / 3,283 / 127,500 | 35 / 1 | E3逐文件检查公开入口、ID、manifest、availability、profile/target assembly、extension flatten、load report与测试 |
| Profile schema、availability、catalog merge与extension registry | 26 / 3,919 / 139,039 | 12 / 0 | E3反查生成源、capability消费、provider membership、正式merge/order/finalize路径 |
| App与dynamic session真实consumer | 6 / 1,248 / 44,384 | 3 / 0 | E3核对重复composition、final graph、fatal handling与lifecycle observer安装 |
| 父报告与唯一owner | 7 / 3,032 / 336,615 | 6 / 0 | E2核对P0归属、生命周期、脚本、产品host和first-party catalog边界 |
| Unreal、Bevy、Godot、Fyrox、Unity Graphics | 22 / 16,511 / 613,477 | 35 / 0 | E2/E3核对module phase、target policy、plugin lifecycle、package/assembly依赖和reload边界 |
| selected combined scope | 91 / 27,993 / 1,261,015 | 91 / 1 | 工作树fingerprint `17ba34f9403b619cb65d77da5828c68d4b96f5675750b8c34bd5fdd9881230ed` |

指纹按91个selected path去重排序，对每个文件取lowercase SHA-256，再以`forward/slash/path|hash`和LF连接、无末尾LF后取总SHA-256。测试数字是静态Rust/C++/C#标记，不表示本轮已编译或通过。唯一ignored是`RuntimePluginId` generation churn benchmark。

### 2.2 检查方法

本轮按`profile source -> build generation -> target/profile selection -> manifest overlay -> provider availability -> plugin/feature registration filter -> extension collection -> core/plugin module materialization -> topology sort -> load report -> App PluginGroup -> Core activation/lifecycle observer`逐段阅读，并反向搜索全部非`dev/`生产consumer。每段分别核对identity、selection authority、target/build closure、dependency order、conflict、provenance、fatal policy、generation、容量、性能和teardown。

### 2.3 动态证据边界

1. 本轮是review-only，没有修改Runtime、App、Editor、Plugin、Hub或Interface生产代码和测试。
2. 未重新运行已知耗时或无关的全工作区编译；既有Editor、Hub、WOC和plugin metadata阻断保持原状。
3. `active_plugin_registration_refs`不读取manifest、`active_feature_registration_refs`不读取selected provider、Profile required capabilities没有runtime consumer，均是可由静态调用图直接证明的事实，不依赖推测测试结果。
4. 未执行Client2D/3D/Editor/Dev/Server的clean feature powerset、1000插件装配、重复provider冲突、hot reload、DLL unload或跨版本Profile反序列化，因此这些资格保持未通过。
5. 实施前必须重取fingerprint并复核当前工作树；本报告是2026-08-16审查快照，不是长期稳定基线。

## 3. 必须保留的工程基础

1. 保留`runtime-feature-presets.toml`作为内建Profile声明源，但升级schema和生成产物，不退回散落Rust常量。
2. 保留build期`deny_unknown_fields`、exact built-in row、重复ID、错误target/maturity和module feature gate检查。
3. 保留`BuiltinRuntimeModuleId`的typed membership和module-name映射方向，同时让wire identity不随`cfg`消失。
4. 保留Profile从target-owned candidate registry选择模块并补内建依赖闭包的算法。
5. 保留descriptor只求值一次并在最终sort复用的优化；后续plan compiler应扩大到全部模块。
6. 保留Core的`sort_module_activation_order`作为最终依赖验证owner，不在App或builtin复制第二套图算法。
7. 保留`RuntimePluginId`对第三方动态key的支持和generation owner释放，不引入进程级永久字符串泄漏。
8. 保留required/optional、target、maturity、packaging和Linked/NativeDynamic分类，但把它们并入同一resolution row。
9. 保留`RuntimePluginCatalog`的project plan cache、module dependency order、extension merge、collision diagnostic和registry finalize方向。
10. 保留Asset Importer跨registry重新注册时的typed error，而不是first-wins静默覆盖。
11. 保留feature dependency report对blocked feature与definition diagnostic的区分。
12. 保留load report不直接打印的底层能力；文本渲染应留在diagnostic sink边界。
13. 保留App最终向Core提交完整ModuleDescriptor图的方向，但App只应消费已冻结plan，不再自行重投影。
14. 保留Runtime01、Plugins01/06、App01各自的生命周期、包装、安全和产品角色ownership，不在Runtime42建立平行authority。

## 4. 当前代码事实与断路

| 链路 | 当前事实 | 工程后果 |
|---|---|---|
| Profile source | 6个Profile、12个builtin module；build.rs硬编码预期集合与顺序 | 内建表有单源，但新增角色/模块必须同时改schema、硬编码表、枚举和materializer |
| Feature cfg | Graphics/Script枚举variant与generated membership受`#[cfg]`删除 | 同一Profile和serde schema随BuildSet改变；缺模块不会形成typed unavailable row |
| Target assembly | Client/Editor取全部core candidate；Server只排Graphics/Script | Target不是Profile；Server仍带Input/Asset/Scene，却不能带Script，规则无法解释能力意图 |
| Profile assembly | typed member+内建依赖闭包+最终sort | 能验证存在性/依赖，但required capability与artifact/provider闭包不进入判定 |
| Target manifest | Client/Editor仅在编译`ui`时默认required UI；Server为空 | 默认选择随binary feature改变，缺UI时不是MissingRequired而是从意图中消失 |
| Manifest overlay | 只对baseline runtime ID canonicalize；其他raw alias保留 | raw/canonical identity可形成双row，required/disabled/packaging合并语义分裂 |
| Availability | 每次重建builtin descriptor Vec和projection；Target用synthetic permissive Profile | 有结构化分类，但没有BuildSet、capability、selection/provider完整receipt |
| Registration filter | 只读registration自身selection，不读effective manifest | 未选择/已禁用registration的Importer/Render extension可泄漏到core module |
| Feature filter | 以available feature id匹配全部registration | 多provider feature会同时注入，和catalog选择一个provider的结果不一致 |
| Extension merge | builtin把多个registry拍成Importer+9组graphics Vec | provenance、owner、dependency order、generation和统一冲突裁决丢失 |
| Catalog merge | 另一条路径按module dependency排序、merge全extension family并finalize | 更完整的authoritative实现已存在，却没有被builtin module construction消费 |
| Plugin module | builtin loader只认识UI；Linked/NativeDynamic availability会跳过module | 其余module靠App再次遍历registration追加，Runtime API自身不是完整composition |
| App composition | 先构造Default/Minimal/Dev/Headless group，再覆盖builtin report并重新sort | 一个启动至少执行两次Profile装配；report module graph不是最终提交图 |
| Catalog duplication | Runtime feature assembly建catalog，App又建catalog/report/lifecycle state | 额外clone/投影；两次输入或merge规则漂移时没有hash证明等价 |
| Load report | `modules`与availability公开，fatal只是查询函数 | 类型允许调用者忽略fatal并使用部分图；legacy convenience API已经这样做 |
| Diagnostics | Core/Unknown/Feature/Importer枚举混合自由字符串 | 无stable code、stage、owner、selection index、source span、remediation或correlation |
| Tests | 有行为测试，但248行structure test主要锁源码布局；1个perf benchmark ignored | 能防文件回并，不能证明selection隔离、multi-provider、BuildSet closure或规模SLO |

## 5. 参考实现给出的边界

### 5.1 Unreal

`FModuleDescriptor`同时声明module host type、loading phase、platform/target/program allow/deny policy，并分别判断“是否编入当前configuration”和“是否应在当前configuration加载”；`LoadModulesForPhase`返回逐模块failure。`FPluginManager`先配置enabled plugin和required availability，再按loading phase装载；`FModuleManager`保留load failure reason、query status、pre-unload/shutdown以及shutdown reverse completion order。Zircon不应复制宏和全局singleton，但必须吸收“build eligibility、runtime selection、phase activation、failure receipt、reverse teardown是同一条可查询状态机”的边界。

### 5.2 Bevy

Bevy `Plugin`拥有build、ready、finish、cleanup和unique identity；`PluginGroupBuilder`集中处理add/set/enable/disable/order，App持有唯一plugin registry并推进全体状态。其group依赖主要靠显式顺序而非Zircon式descriptor graph，不能作为复杂动态依赖上限；可借鉴的是调用者不应同时维护第二份plugin truth，最终lifecycle必须由同一App registry持有。

### 5.3 Godot

Godot module与GDExtension共享Core/Servers/Scene/Editor initialization level；manager严格逐级initialize、按当前level逆序deinitialize，并把AlreadyLoaded、NotLoaded、NeedsRestart与Failed分开。Library loader还解析entry symbol、platform-specific library和compatibility minimum。Zircon需要类似的phase/restart分类和一致的卸载边界，但应进一步加入事务、capability和generation receipt。

### 5.4 Fyrox

Fyrox plugin把register、init、on_loaded、on_deinit、graphics context created/destroyed等hook置于同一plugin contract；Executor等资源registry ready后才enable plugin，动态plugin reload会重新fill/register并保留明确state。它比Zircon builtin的“先拍平部分extension、再由App安装observer”更连贯，但其Rust dylib和全进程故障域不是Zircon的安全目标上限。

### 5.5 Unity Graphics

本地Graphics镜像把SRP Core/HDRP表达为带精确package version dependency的包，并在asmdef中独立声明assembly reference、platform inclusion、auto-reference和version define。它证明“package依赖”和“代码assembly/build条件”应分层但可验证闭包；不能从该镜像推断Unity完整运行时插件生命周期。

### 5.6 Zircon的超越目标

目标不是拥有更多Profile函数，而是让任意产品角色只产生一份`RuntimeCompositionReceipt`：输入绑定project/profile/target/platform/BuildSet/catalog generation；输出逐selection解释provider和capability，给出完整module/extension graph、phase、owner与artifact；所有fatal在commit前关闭，commit后Core、App、Editor、DLL只消费同一generation。性能目标应是一次线性projection加一次图验证，而不是多入口重复clone/catalog/sort。

## 6. 目标架构

```text
ProductRole + Platform + BuildSet + ProjectManifest + ProfileIntent
                              |
                              v
                    CompositionCompiler
  IdentityRegistry -> SelectionResolver -> Capability/Provider Solver
                              |
                              v
              Frozen RuntimeCompositionPlan (generation/hash)
      selection rows / package+provider rows / module graph / phases
      merged extension registry / artifact closure / diagnostics / budgets
                              |
                              v
                    CompositionTransaction
       preflight -> stage registries -> validate -> activate -> publish
                              |
                              v
                    RuntimeCompositionReceipt
       Ready | Degraded | Rejected / rollback / teardown order / evidence
```

建议最小合同：

```text
QualifiedPluginId { namespace, name, major }
ProductBuildIdentity { role, target, platform, features, engine_build }
SelectionResolutionRow { source_index, raw_id, canonical_id, provider, outcome }
ModulePlanRow { stable_id, owner, phase, host_policy, descriptor_hash }
ExtensionPlanRow { family, stable_id, owner, provider, conflict_policy }
RuntimeCompositionPlan { generation, source_hashes, rows, graph, registry }
RuntimeCompositionReceipt { outcome, activated, degraded, diagnostics, teardown }
```

`RuntimePluginCatalog::project_plan_for()`应成为这条compiler的基础而不是旁路。Builtin module candidate、first-party source、generated export和native provider都提交descriptor/extension proposal；compiler按effective manifest和provider choice只合并一次。App不能再拿registration Vec重算feature report或追加未进入plan的module。

## 7. P0 唯一归属与依赖路由

本篇不新增P0。以下现有阻断是Runtime42的前置或联合交付：

| Canonical owner | 现有阻断 | Runtime42责任 |
|---|---|---|
| Plugins06 FP-CATALOG-P0-001 | required selection可在provider lookup中无声消失 | plan保留每个selection row并要求required terminal success |
| Plugins06 FP-CATALOG-P0-002 | 标准Profile required plugin与App feature closure冲突 | compiler消费统一BuildSet/provider matrix，不再让cfg静默改变Profile事实 |
| Plugins06 FP-CATALOG-P0-003/004 | runtime/editor source provider catalog不完整 | 接受generated provider set并返回NotCompiled/Unsupported，不自行补catalog |
| Plugins01 P0 | native load-before-select、ABI行为与安全缺口 | plan只接收通过admission的provider receipt，不直接加载未知代码 |
| App01 P0-3 | 无统一产品shutdown coordinator | Runtime42输出activation/teardown plan与census，App拥有跨服务停机 |
| Runtime01 P0 | production module deactivation/cleanup不闭环 | composition transaction接入Core lifecycle，不建立第二套unload算法 |
| Runtime21/07 | server/script/plugin generation和动态行为闭环 | Runtime42允许Server Script/ZrVM由role capability选择，算法仍归语言/插件owner |

## 8. P1：Composition Authority 与事务边界

### P1-01：registration active判定不消费effective manifest

`active_plugin_registration_refs()`只读取registration自产生的selection。改为由统一selection resolution plan输出selected provider indices；任何未选择、disabled、target mismatch或failed row不得进入extension/module staging。

### P1-02：feature assembly不执行provider选择

`active_feature_registration_refs()`只比较feature id，多个同ID provider会全部激活。必须消费catalog完成后的`feature_id -> selected provider package/generation`映射，并给未选provider明确Inactive outcome。

### P1-03：builtin绕过正式catalog merge

Catalog已有dependency order、全extension family merge、collision diagnostic和finalize；builtin却重新flat-map部分Vec。删除平行merge路径，让Core module constructors只接收一个已冻结的merged registry或typed projection。

### P1-04：registration diagnostics未进入module load fatal truth

普通plugin registration中的`diagnostics`不会加入`RuntimeModuleLoadReport`；App仅`eprintln!`后继续，dynamic session则通过另一份extension report把它们设为fatal。所有consumer必须看到同一severity和terminal outcome。

### P1-05：Runtime与App重复构建catalog/project plan

feature route在Runtime建一次catalog，App随后再次建catalog、feature report和extension report。返回并共享`Arc<FrozenCompositionPlan>`，禁止通过相同输入“期望两次结果相等”。

### P1-06：App PluginGroup先独立装配再覆盖

`plugin_group_builder_for_config()`先按Client3D/Server/Minimal/Dev构造一套group，随后把Runtime report逐项set/add并再次sort。未来Profile成员一旦分化就可能留下base group多余模块；App应直接从final plan构造group。

### P1-07：公开API形成参数组合爆炸

Target/Profile、manifest/no manifest、linked IDs/registration reports、feature/no feature排列成多组长函数名，已有分支在baseline merge、availability和clone策略上不同。改为一个builder/input DTO和一个compile入口，便捷函数只能构造输入。

### P1-08：linked ID与registration report是两种不等价provider合同

linked path只有`HashSet<String>`，无法携descriptor、packaging、artifact、diagnostic或extension；report path又把membership留给availability projection。统一为typed provider resolution rows，禁止字符串集合代表“可安全执行”。

### P1-09：load report无法在类型上区分Ready与Rejected

`modules`公开且fatal需调用方主动查询，调用方可以把Rejected report的部分图注册到Core。使用`Result<AcceptedComposition, RejectedComposition>`或sealed outcome；Rejected只能提供诊断/partial plan，不能暴露可提交模块。

### P1-10：legacy convenience API丢弃错误

`builtin_runtime_modules()`直接取`.modules`，`runtime_core_modules()`对sort失败`expect`；两者无生产consumer。标记deprecated并硬切到fallible plan，测试也不得继续证明“忽略report”是支持合同。

### P1-11：没有composition generation、hash或source identity

report不绑定manifest、Profile schema、BuildSet、catalog generation、provider artifact或descriptor hash。任何后续App/Editor/DLL重投影都无法证明同代；receipt必须携完整输入hash与engine build identity。

### P1-12：装配不是可回滚事务

当前先收集部分extension、构造modules、返回fatal字符串，再由App注册Core并安装observer；失败点跨多个owner。建立stage registry、全图validate、activation commit和逆序rollback，失败receipt列出未发布与已回滚资源。

## 9. P1：Profile、Target、BuildSet 与 Capability Closure

### P1-13：Profile required capabilities是未执行metadata

全仓生产代码只生成/保存`RuntimeProfileDescriptor.required_capabilities`，builtin assembly从不验证它。compiler必须把capability解析到具体module/provider/artifact row，缺required capability直接Rejected。

### P1-14：`cfg`改变BuiltinRuntimeModuleId的serde schema

Graphics/Script variant在feature关闭时物理消失，同一JSON/TOML在不同binary可能无法反序列化。稳定ID集合必须与compiled availability分离；缺实现返回NotCompiled，不能改变wire enum。

### P1-15：`cfg`静默改变同名Profile的module membership

生成代码给Graphics/Script成员加`#[cfg]`，测试比较的是同样被裁剪后的descriptor，因此会绿。Profile intent应保持完整，BuildSet preflight显式报告每个missing module/capability。

### P1-16：build schema只验证形状，不验证工程闭包

build.rs检查ID、重复、feature token和局部gate，却不验证module dependency、required capability、default plugin provider、Cargo dependency、artifact和target closure。接入generated ProductProviderBuildMatrix并在build/export/startup三层复核。

### P1-17：Target default manifest随编译feature改变

Client/Editor仅在`ui`已编译时才产生required UI选择；无UI build会把需求从manifest删除。默认意图必须稳定，availability负责解释NotCompiled；否则相同project在不同binary不产生可比较receipt。

### P1-18：Target入口不是Profile选择器

`runtime_modules_for_target(ClientRuntime)`装配全部client candidate，无法区分Minimal、Client2D、Client3D、Commandlet或Embedded。Target只应是solver constraint；产品必须提供明确role/profile intent。

### P1-19：Target availability使用synthetic permissive Profile

临时descriptor固定minimum maturity Experimental、空required capabilities和允许策略默认值；`RuntimeProfileId`映射只是标签。删除伪Profile，以明确`TargetCompositionPolicy`表达无Profile场景，或要求所有产品提供Profile。

### P1-20：三值RuntimeTargetMode无法表达产品角色

Client/Server/EditorHost不能区分桌面、移动、Web、commandlet、cook worker、editor play child、dedicated service和embedded。角色taxonomy与artifact由App01拥有；Runtime42消费其typed constraints。

### P1-21：Server模块gate表现为临时条件分支

Server固定包含Input/Asset/Scene，却无论`script`是否编译都排除Script；这既不能表达virtual input，也阻断脚本型服务器。由capability/role policy决定模块，不在constructor写`target != Server`。

### P1-22：EditorModule不在Editor Profile图中

Editor Profile声明editor capability，但Runtime plan看不到`zircon_editor::EditorModule`，App在sort前临时追加。跨crate模块应通过product composition proposal进入同一graph，Profile只引用稳定module capability/ID。

### P1-23：built-in集合扩展需修改多处硬编码

build.rs要求恰好6个Profile和12个module并固定顺序，新增内建模块还要改Rust枚举/materializer。保留内建schema严格性，但从declaration registry生成expected set，并提供project/plugin profile extension层。

### P1-24：Profile default、Target baseline与Project override merge规则分裂

Profile路径可直接使用profile manifest，Target路径先加UI baseline，App在无manifest时又自行选择profile/default；alias只在baseline命中时canonicalize。定义一个merge policy，逐row保留source、precedence和conflict，不允许raw String覆盖隐含规则。

## 10. P1：Identity 与 Catalog 边界

### P1-25：`RuntimePluginId::from_static`绕过全部不变量

公开`const fn`可创建空、带斜杠、非canonical或alias key，与`parse_key`的规则冲突。限制为crate/generated use，或改为const-validated macro/newtype，任何公开构造均fallible。

### P1-26：`RuntimePluginId::new`对外部数据panic

`new()`内部`expect`，调用者稍有输入错误就终止线程。保留`FromStr/TryFrom`为唯一动态入口；构建期常量走verified declaration。

### P1-27：hard-coded alias会在升级时劫持既有动态ID

今天未知key会成为Dynamic，未来把同key加入built-in alias后，旧项目身份会被重解释。alias必须属于版本化identity registry，带迁移epoch、冲突检查和原始ID receipt。

### P1-28：runtime ID、package ID与provider membership被混用

`with_linked_plugins`参数名看似runtime ID，availability实际用descriptor package ID查membership；多数first-party两者恰好相同掩盖了差异。建立`QualifiedPluginId`、`PackageId`、`ProviderId`不同类型。

### P1-29：ID缺namespace、major version与publisher域

允许任意短key且built-in alias占用全局空间，无法安全承载第三方同名、并行major或vendor provider。采用稳定qualified identity，显示别名不参与hash/equality。

### P1-30：label硬编码在ID类型

27个static key通过match返回英文label，第三方直接显示raw key。显示名、本地化、publisher和deprecated alias属于descriptor/catalog metadata，不属于identity primitive。

### P1-31：UnknownPlugin只表示语法非法，不表示catalog unknown

合法但未声明key会被解析为Dynamic，随后availability可能归为MissingCatalog/Stub；枚举名和用户诊断容易混淆。区分InvalidSyntax、UnknownIdentity、KnownNoProvider、UnsupportedBuild和UnavailableArtifact。

### P1-32：builtin descriptor catalog每次装配重建

`runtime_plugin_descriptors()`每次分配完整Vec，feature/App重复composition进一步放大。由BuildSet generation持有`Arc<CatalogProjection>`，manifest只做索引查找。

### P1-33：UI是唯一builtin module special case

`module_for_plugin()`只识别UI，availability也对UI单独按cfg判断；其余static ID依赖外部catalog。把UI也建模为普通compiled provider/module proposal，删除身份级特权分支。

## 11. P1：Extension、Module Graph 与 Lifecycle

### P1-34：extension flatten丢失provenance

Importer和9组graphics extension被克隆为裸集合，无法回答来自哪个package/module/feature/provider/generation。每条extension plan row必须保留owner与selection resolution link。

### P1-35：flatten顺序取决于caller输入，不取决于module graph

Catalog merge会按runtime module dependency排序，builtin直接按registration迭代顺序flat-map。任何order-sensitive collector/provider都可能随catalog枚举顺序改变；使用统一ordered merge。

### P1-36：只有Asset Importer冲突在builtin层产生typed error

Importer通过新registry注册并收集error；render feature、geometry、shading、executor、collector和provider只是拼Vec。所有family必须在staging registry执行同样的ID/owner/conflict/policy验证。

### P1-37：shader source去重规则不足以裁决冲突

当前只按`owner_id + import_path + content_hash`去重；同import path不同内容会同时保留，重复provider的相同内容又会被无声折叠。以import identity为key，identical duplicate要有明确coalesce receipt，different content必须fatal。

### P1-38：builtin只投影部分RuntimeExtensionRegistry family

Managers、modules、systems、resources、events、interfaces、components、UI components、options等由后续App/lifecycle路径处理，Asset/Graphics却提前取走。一个registry generation应作为整体提交，领域模块只借用所需只读view。

### P1-39：Linked/NativeDynamic module materialization依赖App旁路

target assembly发现Linked/NativeDynamic便跳过builtin module，真正plugin descriptors由App再次遍历registration追加。Runtime API直接调用者得不到完整图；module proposal必须随provider row进入plan。

### P1-40：core candidate target gate不能表达headless/offscreen/compute变体

Graphics和Script仅以Server布尔排除，Platform/Input始终构造。模块proposal应声明host role、platform、device、window、cooked/editor和capability constraints，由solver解释原因。

### P1-41：RuntimeModuleLoadReport中的module graph不是最终Core graph

App随后加入EditorModule、plugin modules、base group feature并再次sort，report无法作为审计或复现证据。只有final plan/receipt可公开`module_graph`，中间候选集合不得叫load report。

### P1-42：没有loading phase与explicit-load policy

所有模块落入一次拓扑activation，不能表达pre-config、platform/server、scene、default、post-engine、editor-only、commandlet或按需加载。给descriptor增加少量稳定phase和host policy，并验证跨phase依赖方向。

### P1-43：plan不记录unload/reload/restart资格

selection只回答“现在是否可用”，不说明模块能否卸载、是否需重启、有哪些live owners和state schema。消费Runtime01/Plugins01 lifecycle receipt，生成逆序teardown和NeedsRestart分类。

## 12. P1：Report、Diagnostics、性能与测试

### P1-44：diagnostic没有stable code和stage

FeatureDefinition与registration diagnostic仍是String，消息变更会破坏automation。统一`CompositionDiagnostic { code, severity, stage, owner, source, context, remediation }`，文本只是renderer。

### P1-45：report没有逐selection resolution row

无法从结果恢复selection index、raw/canonical ID、required、packaging、chosen provider、artifact和outcome。Plugins06的resolution receipt必须成为Runtime42 plan的直接输入/输出，而不是旁路文档概念。

### P1-46：availability report由公开可变分类Vec组成

调用者可构造同一plugin同时Available和MissingRequired，类型没有互斥不变量。内部保存单一row map/ordered rows，category只是只读projection；序列化时附schema/generation。

### P1-47：diagnostic和extension顺序没有规范化合同

多registration/feature输入顺序会影响error与Vec输出，跨机器/catalog provider枚举可能产生不同receipt。定义canonical order：selection source order用于UX，graph/extension按qualified ID和dependency稳定排序，诊断带stable sequence key。

### P1-48：App把warning/registration diagnostic写到`eprintln!`

没有结构化log sink、correlation、startup summary或Editor surface，发布构建也无法聚合。通过diagnostic service发布一次composition summary，CLI/Editor/telemetry分别渲染。

### P1-49：feature route深clone report且产品重复projection

Runtime把plugin/feature report cloned成Vec，Catalog内部又clone；App再构建Catalog和extension report，PluginGroup还重新装配/sort。用borrowed input snapshot加Arc-owned frozen generation，建立1/100/1,000/10,000 row allocation/time基准。

### P1-50：装配输入没有容量与复杂度预算

manifest selections、features、registration、modules、extensions、dependencies和diagnostics均可无界增长。定义全局/单package/单family上限、checked work estimate、拒绝原因和export-time更高离线预算。

### P1-51：测试重点偏源码布局，缺关键负例

248行structure test以`include_str().contains()`锁定owner和禁止词，行为测试没有“manifest禁用但report enabled”“同feature多provider”“cfg缺required module”“registration diagnostic必须fatal”等用例。保留少量边界守卫，增加API/transaction行为测试。

### P1-52：没有真实Product BuildSet与规模资格

现有测试证明局部Profile成员和可用性分类，不证明标准feature set clean build/startup、source/native等价、deterministic receipt、rollback、reload或startup SLO。建立五Profile、多平台、包装形态、冲突和规模矩阵。

## 13. P2：超越参考引擎的长期能力

| ID | 能力 | 目标 |
|---|---|---|
| P2-01 | Constraint-based provider solver | 按capability、target、quality、trust、license、budget选择provider并解释决策 |
| P2-02 | Profile inheritance/composition | Project和第三方可声明受schema约束的Profile delta，不复制完整built-in表 |
| P2-03 | Precompiled composition artifact | Cook/export生成content-addressed plan，运行时只验证BuildSet/artifact并快速提交 |
| P2-04 | Incremental composition diff | manifest变化生成add/remove/replace diff、影响面和rollback，不全量重建 |
| P2-05 | Multi-provider quality tiers | 同feature支持CPU/GPU/vendor/fallback实现与quality downgrade receipt |
| P2-06 | Isolated provider domains | 不可信或高风险provider在进程/VM sandbox中，通过capability broker接入 |
| P2-07 | Versioned quantitative capabilities | capability表达版本、limits、resource class、permission和negotiated value |
| P2-08 | Composition graph explorer | Editor可视化selection、provider、module、phase、extension、artifact与missing chain |
| P2-09 | Fleet composition provenance | crash/telemetry携receipt hash，可按BuildSet/provider generation聚合 |
| P2-10 | Deterministic composition replay | 保存输入与decision log，在CI/用户机器复现同一plan或精确解释差异 |
| P2-11 | Partitioned/on-demand modules | 大型项目按world/content partition加载显式模块域并受owner lease保护 |
| P2-12 | Profile-guided startup parallelism | 在phase/dependency/resource约束内并行prepare，commit仍确定且可回滚 |
| P2-13 | Signed release closure | receipt绑定artifact signature、SBOM、license、SDK/ABI/schema和revocation状态 |
| P2-14 | Formal/model-based composition tests | 对selection/merge/rollback/hot replace状态机做property、fuzz与model checking |

## 14. 分阶段重构计划

### M0 · 冻结Identity、Schema与结果语义

定义稳定module/plugin/package/provider/capability ID；移除公开panic/bypass构造；冻结Profile schema、BuildSet identity、diagnostic code与Ready/Degraded/Rejected结果。先写迁移器和兼容fixture，再改生产入口。

### M1 · 建立单一Composition Compiler

以`RuntimePluginCatalog::project_plan_for`为基础，接入builtin/module/product proposal，输出逐selection/provider/capability/module/extension row和generation hash。所有旧API改为输入adapter，不再拥有选择逻辑。

### M2 · 闭合Profile、BuildSet与Product Role

消费Plugins06 generated provider matrix与App01 role taxonomy；Profile intent不受cfg裁剪，compiled缺口显式Rejected。验证Client2D、Client3D、Editor、Dev、Server真实Cargo feature与artifact closure。

### M3 · Transactional Extension 与 Module Graph

删除builtin flat Vec merge；把全RuntimeExtensionRegistry在staging generation按依赖合并、冲突检查、finalize。把Editor和plugin module proposal纳入同一graph，增加phase/host/reload policy。

### M4 · Product Cutover 与 Lifecycle

App、dynamic session、Editor和export只消费同一plan/receipt；删除第二次PluginGroup/Profile/Catalog装配。接入Core activation rollback、Runtime01 reverse teardown和Plugins01 provider owner fence。

### M5 · Diagnostics、预算、性能与发布资格

接入统一diagnostic sink、startup summary和graph explorer；增加容量预算、linear complexity instrumentation、clean BuildSet matrix、conflict/rollback/reload/powerset/fuzz/soak测试，发布门只接受fresh receipt。

## 15. 42个资格门

### G0 · Identity 与 Schema（1-7）

1. `BuiltinRuntimeModuleId`和Profile wire schema在graphics/script on/off build间保持可解析，缺实现返回NotCompiled。
2. 所有动态plugin ID构造fallible；公开API不能创建空、非法或非canonical static ID。
3. alias registry有version、migration、collision和raw-to-canonical receipt，新增alias不会静默劫持旧ID。
4. RuntimePlugin、Package、Provider、Module和Capability ID为不同类型，禁止String误传编译通过。
5. Profile schema、catalog schema、diagnostic schema和composition receipt均带版本与兼容策略。
6. BuildSet identity至少绑定engine build、target triple、product role、Cargo features和provider catalog generation。
7. 旧manifest/Profile fixture经过显式migration后产生稳定canonical hash，未知future schema fail closed。

### G1 · Selection、Profile 与 Plan（8-16）

8. 每个manifest selection恰有一个terminal resolution row，disabled/target mismatch也可解释。
9. required selection非Resolved/Ready时composition必为Rejected；optional才允许typed Degraded。
10. manifest禁用plugin时，即使传入enabled registration，其任何extension/module均不进入plan。
11. 同feature多provider时只有明确选中的provider进入plan，其余行标记Inactive/NotSelected。
12. Profile每个required capability解析到具体module/provider/artifact，缺项报告完整dependency chain。
13. Client2D/3D/Editor/Dev/Server intent在不同BuildSet中不静默删成员。
14. Target、Profile、Project和render overlay按一个有source/precedence/conflict的merge policy执行。
15. 无Profile的产品必须提供typed TargetCompositionPolicy，不能使用synthetic permissive Profile。
16. 相同输入在进程、机器和registration枚举顺序变化下生成相同plan hash。

### G2 · Extension、Module 与事务（17-24）

17. 所有extension family通过一个staging `RuntimeExtensionRegistry`合并并保留owner/provider/generation。
18. duplicate identical、duplicate conflict和override policy逐family有确定receipt，禁止silent first/last wins。
19. Shader import path相同内容可显式coalesce，不同内容必定fatal并指出两个owner。
20. Plugin/module dependency order先于extension merge；input Vec顺序不影响结果。
21. Builtin、Editor、source plugin、generated export和native module进入一张最终ModuleDescriptor图。
22. 跨loading phase依赖方向被验证，phase failure能rollback本phase及后续已提交资源。
23. Rejected plan不暴露可注册模块；activation partial failure返回完整rollback census。
24. 每个module/provider记录Unloadable/Reloadable/NeedsRestart及state schema/owner fence。

### G3 · Product、Lifecycle 与包装形态（25-31）

25. App启动只执行一次Profile/Catalog/Module composition，instrumentation断言project plan build count为1。
26. App、Editor、dynamic DLL和export runtime消费相同receipt或验证等价hash。
27. Server可按role选择Script/VM或明确Unsupported，Input/Graphics等模块均有可解释constraint。
28. EditorModule和editor runtime mirror在同一graph中出现，不再由调用点临时append。
29. SourceTemplate、LibraryEmbed、NativeDynamic和generated export对相同selection产生同schema outcome。
30. session destroy先关闭composition admission、撤销extension owner、逆序deactivate，再释放provider code。
31. 需要重启的module/provider变化返回NeedsRestart，不尝试半热更改并假装成功。

### G4 · Diagnostics、预算、测试与性能（32-42）

32. 每个diagnostic有stable code、severity、stage、owner、source index、correlation和remediation。
33. CLI、Editor和telemetry消费同一structured diagnostic；生产入口不直接`eprintln!`业务结果。
34. Availability由单一互斥row集合投影，无法同时标记Available和MissingRequired。
35. manifest/feature/module/extension/dependency/diagnostic都有全局与单owner容量预算。
36. 1/100/1,000/10,000 selection基准报告时间、allocation、clone、projection build和graph work。
37. 正常产品启动只构建一次builtin catalog projection、一次project plan并执行一次最终sort。
38. 负例覆盖disabled registration泄漏、multi-provider、alias conflict、missing cfg module和registration diagnostic。
39. 五Profile按真实Cargo features执行clean build、startup、receipt golden和shutdown。
40. 高风险feature组合、source/native parity、冲突、rollback、reload和DLL unload进入CI矩阵。
41. property/fuzz测试覆盖manifest merge、ID parser、graph cycle、conflict和diagnostic determinism。
42. 发布证据绑定source fingerprint、BuildSet、plan hash、测试结果和性能阈值，任一变化自动recheck。

## 16. 明确禁止的继续实现方式

1. 禁止再增加`runtime_modules_for_*_with_*`排列组合函数来修单一路径。
2. 禁止在App、Editor、DLL或export侧重新实现manifest/profile/provider选择。
3. 禁止继续用registration自身selection替代effective project selection。
4. 禁止把feature id相同解释为“所有provider都应激活”。
5. 禁止从多个registry直接flat-map裸Vec后绕过catalog merge/finalize。
6. 禁止让`#[cfg]`删除稳定wire enum variant或同名Profile意图。
7. 禁止required capability只存在于metadata/tests而不进入admission。
8. 禁止用String ID同时表示runtime alias、package、provider和module owner。
9. 禁止在Rejected report中继续暴露可提交module Vec。
10. 禁止用`eprintln!`、自由字符串和调用者自觉检查fatal维持产品正确性。
11. 禁止以源码`contains()`、manifest数量或catalog schema绿灯替代行为/产品资格。
12. 禁止在没有generation、owner fence和rollback receipt时增加hot reload或运行期切换。

## 17. 收口判定

Runtime42完成不等于“6个Profile测试通过”，而是满足以下终态：

1. 一个产品输入只生成一个不可变、可hash、可解释的composition plan。
2. 每个selection、provider、capability、module、extension和artifact均可追溯到owner与generation。
3. disabled/unselected/failed provider在commit前被隔离，不能通过旁路向Core注入任何贡献。
4. Profile intent、compiled BuildSet和runtime availability三者分层且闭合，任何缺口fail closed。
5. Core/App/Editor/DLL/export共享同一最终module graph和extension registry truth。
6. activation失败可rollback，shutdown按receipt逆序排空，动态代码释放有owner fence。
7. 标准Profile、包装形态、冲突、规模、性能和重载证据均为fresh并绑定plan hash。

在这些条件成立前，当前builtin assembly应视为“有typed基础的过渡composition层”，不能视为已经具备Unreal级module/plugin product assembly，更不能把新增Profile函数、更多hard-coded ID或结构测试数量当作工程化完成度。
